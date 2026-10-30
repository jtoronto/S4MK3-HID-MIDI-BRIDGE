//! One writer and neutral guard for the new platter experiments only.

use std::io::Write;
#[cfg(test)]
use std::time::Instant;

use anyhow::ensure;
use serde::Serialize;
use tracing::{info, warn};

use super::capture::RawFrame;
use super::motor::{neutral_report, report_for};
use super::platter::{Config, Experiment, Gate, Model, Sample, ring_initialization, ring_report};
use super::platter_rotation;
use super::platter_telemetry::{Clock, LogTiming, StopRequest};
use super::reader::Reader;
use super::reader_telemetry::Snapshot;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Record<'a> {
    PlatterSession {
        version: u8,
        config: &'a Config,
        position_basis: &'static str,
    },
    Raw {
        frame: &'a RawFrame,
    },
    PlatterMotion {
        sample: &'a Sample,
    },
    PlatterTransition {
        host_us: u64,
        reason: &'static str,
        gate: Gate,
        ring_held: bool,
    },
    PlatterOutput {
        host_us: u64,
        io_started_us: u64,
        io_completed_us: u64,
        io_elapsed_us: u64,
        stop_requested_at_us: Option<u64>,
        reason: &'a str,
        report: &'a [u8],
        requested_force: [i32; 2],
        written: Option<usize>,
        error: Option<String>,
    },
    PlatterEnd {
        host_us: u64,
        reason: &'a str,
        relative_counts: i64,
        travel_counts: u64,
        stop_requested_at_us: Option<u64>,
        log_timing: LogTiming,
    },
    PlatterHealth {
        host_us: u64,
        reader: Option<&'a Snapshot>,
        queued_frames: usize,
        consumed_frames: u64,
        consumer_lag_us: Option<u64>,
        max_consumer_lag_us: u64,
        stop_requested_at_us: Option<u64>,
        log_timing: LogTiming,
    },
    PlatterRotationTuning {
        algorithm: &'static str,
        speed_window_us: u64,
        overspeed_limit_rpm: Option<u16>,
        base_force: u16,
        force_per_rpm: u16,
        sensor_ticks_per_us: u64,
        proportional_force_per_rpm: u16,
        integral_force_per_rpm_second: u16,
        target_step: bool,
        force_rise_per_second: u32,
        startup_ms: u16,
        startup_force_rise_per_second: u32,
        startup_force_step: bool,
    },
}

/// Preserve both errors; cleanup failures must not hide the triggering failure.
pub fn combine<T>(first: anyhow::Result<T>, second: anyhow::Result<()>) -> anyhow::Result<T> {
    match (first, second) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(first), Err(second)) => Err(first.context(format!("additional failure: {second:#}"))),
    }
}

pub struct Session<'a, F: FnMut(&[u8]) -> anyhow::Result<usize>> {
    pub model: Model,
    writer: F,
    log: &'a mut dyn Write,
    finished: bool,
    force: [i32; 2],
    ring_last_us: Option<u64>,
    transition: (Gate, bool),
    clock: Clock,
    pub stop_request: StopRequest,
    log_timing: LogTiming,
    reader_health: Option<Snapshot>,
    queued_frames: usize,
    consumed_frames: u64,
    consumer_lag_us: Option<u64>,
    max_consumer_lag_us: u64,
    health_written_us: Option<u64>,
}

impl<'a, F: FnMut(&[u8]) -> anyhow::Result<usize>> Session<'a, F> {
    /// Infallible construction installs cleanup before any setup can fail.
    #[cfg(test)]
    pub fn new(config: Config, writer: F, log: &'a mut dyn Write) -> Self {
        let clock = Clock::Live(Instant::now());
        let stop = StopRequest::new(clock.clone());
        Self::with_telemetry(config, writer, log, clock, stop)
    }

    pub fn with_telemetry(
        config: Config,
        writer: F,
        log: &'a mut dyn Write,
        clock: Clock,
        stop_request: StopRequest,
    ) -> Self {
        Self {
            model: Model::new(config),
            writer,
            log,
            finished: false,
            force: [0; 2],
            ring_last_us: None,
            transition: (Gate::NeedRelease, false),
            clock,
            stop_request,
            log_timing: LogTiming::default(),
            reader_health: None,
            queued_frames: 0,
            consumed_frames: 0,
            consumer_lag_us: None,
            max_consumer_lag_us: 0,
            health_written_us: None,
        }
    }

    fn record(&mut self, record: &Record<'_>) -> anyhow::Result<()> {
        let started = self.clock.now_us();
        let result = (|| {
            serde_json::to_writer(&mut *self.log, record)?;
            self.log.write_all(b"\n")?;
            Ok(())
        })();
        self.log_timing
            .records
            .observe(started, self.clock.now_us(), result.is_err());
        result
    }

    fn flush_log(&mut self) -> anyhow::Result<()> {
        let started = self.clock.now_us();
        let result = self.log.flush().map_err(Into::into);
        self.log_timing
            .flushes
            .observe(started, self.clock.now_us(), result.is_err());
        result
    }

    pub fn capture_reader_health(&mut self, reader: &Reader) -> anyhow::Result<()> {
        self.reader_health = reader.telemetry()?;
        self.queued_frames = reader.frames.len();
        Ok(())
    }

    pub fn consumed(&mut self, frame_us: u64, now_us: u64) {
        self.consumed_frames += 1;
        let lag = now_us.saturating_sub(frame_us);
        self.consumer_lag_us = Some(lag);
        self.max_consumer_lag_us = self.max_consumer_lag_us.max(lag);
    }

    fn health(&mut self, now_us: u64) -> anyhow::Result<()> {
        let started = self.clock.now_us();
        let result = (|| {
            serde_json::to_writer(
                &mut *self.log,
                &Record::PlatterHealth {
                    host_us: now_us,
                    reader: self.reader_health.as_ref(),
                    queued_frames: self.queued_frames,
                    consumed_frames: self.consumed_frames,
                    consumer_lag_us: self.consumer_lag_us,
                    max_consumer_lag_us: self.max_consumer_lag_us,
                    stop_requested_at_us: self.stop_request.at_us(),
                    log_timing: self.log_timing,
                },
            )?;
            self.log.write_all(b"\n")?;
            Ok(())
        })();
        self.log_timing
            .records
            .observe(started, self.clock.now_us(), result.is_err());
        result
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.neutral(0, "startup_neutral")?;
        // Borrow disjoint fields so settings are serialized without cloning.
        let started = self.clock.now_us();
        let result: anyhow::Result<()> = (|| {
            serde_json::to_writer(
                &mut *self.log,
                &Record::PlatterSession {
                    version: 2,
                    config: &self.model.config,
                    position_basis: "integrated_relative_raw_counts_not_absolute_angle",
                },
            )?;
            self.log.write_all(b"\n")?;
            Ok(())
        })();
        self.log_timing
            .records
            .observe(started, self.clock.now_us(), result.is_err());
        result?;
        if let Experiment::Rotate(settings) = self.model.config.experiment {
            self.record(&Record::PlatterRotationTuning {
                algorithm: "feedforward_pi_tunable_v7",
                speed_window_us: platter_rotation::WINDOW_US,
                overspeed_limit_rpm: settings.effective_overspeed_limit_rpm(),
                base_force: settings.base_force,
                force_per_rpm: settings.force_per_rpm,
                sensor_ticks_per_us: platter_rotation::SENSOR_TICKS_PER_US,
                proportional_force_per_rpm: settings.kp,
                integral_force_per_rpm_second: settings.ki,
                target_step: true,
                force_rise_per_second: settings.force_rise_per_second,
                startup_ms: settings.startup_ms,
                startup_force_rise_per_second: settings.startup_force_rise_per_second,
                startup_force_step: settings.startup_ms > 0
                    && settings.startup_force_rise_per_second == 0,
            })?;
        }
        let initialize = match self.model.config.experiment {
            Experiment::Measure => false,
            Experiment::Drive {
                initialize_ring, ..
            } => initialize_ring,
            Experiment::Rotate(settings) => settings.initialize_ring,
            Experiment::Ring { .. } => true,
        };
        if initialize {
            self.output(
                0,
                "selected_ring_initialization",
                &ring_initialization(self.model.config.common.side),
                [0; 2],
            )?;
        }
        info!(
            experiment = ?self.model.config.experiment,
            side = ?self.model.config.common.side,
            "PLATTER_READY: relative counts only; drive arms via selected MOVE release/press/release with top untouched; touch interrupts; Ctrl-C stops"
        );
        Ok(())
    }

    const fn outputs_enabled(&self) -> bool {
        match self.model.config.experiment {
            Experiment::Measure => false,
            Experiment::Drive { .. } | Experiment::Ring { .. } | Experiment::Rotate(_) => true,
        }
    }

    fn neutral(&mut self, now_us: u64, reason: &str) -> anyhow::Result<()> {
        if self.outputs_enabled() {
            self.output(now_us, reason, &neutral_report(), [0; 2])?;
        }
        Ok(())
    }

    pub fn sample(&mut self, frame: &RawFrame, now_us: u64) -> anyhow::Result<()> {
        let sample = match self.model.observe(frame, now_us) {
            Ok(sample) => sample,
            Err(error) => {
                // Even malformed frames are retained, but neutral precedes log I/O.
                let neutral = self.neutral(now_us, "decode_error_neutral");
                let raw = self.record(&Record::Raw { frame });
                return combine(Err(error), combine(neutral, raw));
            }
        };
        // Touch must remove drive before logging, not after a slow log write.
        if sample.requested_force == [0; 2] && self.force != [0; 2] {
            self.neutral(now_us, "sample_gate_neutral")?;
        }
        self.record(&Record::Raw { frame })?;
        self.record(&Record::PlatterMotion { sample: &sample })?;
        self.transition(now_us)
    }

    fn transition(&mut self, now_us: u64) -> anyhow::Result<()> {
        let current = (self.model.gate, self.model.ring_held);
        if current != self.transition {
            let reason = if current.1 != self.transition.1 {
                if current.1 {
                    "touch_hold_or_interrupt"
                } else {
                    "touch_release_no_drive_rearm"
                }
            } else if matches!(self.transition.0, Gate::Driving { .. })
                && !matches!(current.0, Gate::Driving { .. })
            {
                "pulse_deadline_requires_rearm"
            } else {
                match current.0 {
                    Gate::NeedRelease => "pulse_deadline_requires_rearm",
                    Gate::Ready => "move_release_observed",
                    Gate::Pressed => "move_pressed",
                    Gate::Driving { .. } => "move_released_armed",
                }
            };
            self.record(&Record::PlatterTransition {
                host_us: now_us,
                reason,
                gate: current.0,
                ring_held: current.1,
            })?;
            info!(gate = ?current.0, ring_held = current.1, "PLATTER_STATE");
            self.transition = current;
        }
        Ok(())
    }

    /// Check every deadline before issuing an output; never catch up missed ticks.
    pub fn tick(&mut self, now_us: u64) -> anyhow::Result<Option<&'static str>> {
        if let Some(reason) = self.model.check_time(now_us)? {
            return Ok(Some(reason));
        }
        // Stop can arrive while sample logging runs, after the loop's selection.
        if self.stop_request.at_us().is_some() {
            return Ok(Some("ctrl_c"));
        }
        let force = self.model.force();
        match self.model.config.experiment {
            Experiment::Measure => {}
            Experiment::Drive { .. } => {
                self.output(now_us, "drive_tick", &report_for(force), force)?;
            }
            Experiment::Rotate(_) => {
                self.output(now_us, "rotation_tick", &report_for(force), force)?;
            }
            Experiment::Ring { .. } => {
                if self.ring_last_us.is_none_or(|last| now_us - last >= 20_000) {
                    let report =
                        ring_report(self.model.config.common.side, self.model.ring_position()?);
                    self.output(now_us, "relative_ring_tick", &report, [0; 2])?;
                    self.ring_last_us = Some(now_us);
                }
            }
        }
        self.transition(now_us)?;
        if self
            .health_written_us
            .is_none_or(|last| now_us.saturating_sub(last) >= 100_000)
        {
            self.health(now_us)?;
            self.flush_log()?;
            self.health_written_us = Some(now_us);
        }
        Ok(None)
    }

    fn output(
        &mut self,
        now_us: u64,
        reason: &str,
        report: &[u8],
        force: [i32; 2],
    ) -> anyhow::Result<()> {
        let io_started_us = self.clock.now_us();
        let transport = (self.writer)(report);
        let io_completed_us = self.clock.now_us();
        let written = transport.as_ref().ok().copied();
        let result = transport.and_then(|length| {
            ensure!(
                length == report.len(),
                "short platter write: {length}/{}",
                report.len()
            );
            Ok(length)
        });
        if result.is_ok() && report[0] == 0x31 {
            self.force = force;
        }
        let logging = self.record(&Record::PlatterOutput {
            host_us: now_us,
            io_started_us,
            io_completed_us,
            io_elapsed_us: io_completed_us.saturating_sub(io_started_us),
            stop_requested_at_us: self.stop_request.at_us(),
            reason,
            report,
            requested_force: force,
            written,
            error: result.as_ref().err().map(|error| format!("{error:#}")),
        });
        combine(result.map(|_| ()), logging)
    }

    pub fn finish(&mut self, now_us: u64, reason: &str) -> anyhow::Result<()> {
        if self.finished {
            return Ok(());
        }
        // All attempts happen even if an earlier cleanup or log write fails.
        let mut result = self.neutral(now_us, "final_neutral");
        if matches!(self.model.config.experiment, Experiment::Ring { .. }) {
            let mut clear = [0; 41];
            clear[..2].copy_from_slice(&[0x32, self.model.config.common.side.wire()]);
            result = combine(
                result,
                self.output(now_us, "selected_ring_clear", &clear, [0; 2]),
            );
        }
        result = combine(result, self.flush_log());
        result = combine(result, self.health(now_us));
        result = combine(
            result,
            self.record(&Record::PlatterEnd {
                host_us: now_us,
                reason,
                relative_counts: self.model.relative_counts,
                travel_counts: self.model.travel_counts,
                stop_requested_at_us: self.stop_request.at_us(),
                log_timing: self.log_timing,
            }),
        );
        // The end record reports the preceding flushes; measuring the flush
        // of its own timing record would require an endless chain of records.
        result = combine(result, self.log.flush().map_err(Into::into));
        self.finished = result.is_ok();
        result
    }
}

impl<F: FnMut(&[u8]) -> anyhow::Result<usize>> Drop for Session<'_, F> {
    fn drop(&mut self) {
        if !self.finished
            && let Err(error) = self.neutral(0, "drop_neutral")
        {
            warn!(%error, "PLATTER_NEUTRAL_FAILED");
        }
    }
}
