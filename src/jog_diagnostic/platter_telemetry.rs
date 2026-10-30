//! Monotonic observation only; none of these clocks changes control deadlines.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde::Serialize;

#[derive(Clone)]
pub enum Clock {
    Live(Instant),
    #[cfg(test)]
    Manual(Arc<AtomicU64>),
}

impl Clock {
    pub fn now_us(&self) -> u64 {
        match self {
            Self::Live(started) => u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
            #[cfg(test)]
            Self::Manual(now) => now.load(Ordering::Acquire),
        }
    }
}

#[derive(Clone)]
pub struct StopRequest {
    clock: Clock,
    requested_us: Arc<AtomicU64>,
}

impl StopRequest {
    pub fn new(clock: Clock) -> Self {
        Self {
            clock,
            requested_us: Arc::new(AtomicU64::new(u64::MAX)),
        }
    }

    pub fn request(&self) {
        // Preserve the first request independently of the event-loop receiver.
        let _previous = self.requested_us.compare_exchange(
            u64::MAX,
            self.clock.now_us(),
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    pub fn at_us(&self) -> Option<u64> {
        match self.requested_us.load(Ordering::Acquire) {
            u64::MAX => None,
            at => Some(at),
        }
    }
}

#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct Timing {
    pub calls: u64,
    pub errors: u64,
    pub started_us: u64,
    pub completed_us: u64,
    pub elapsed_us: u64,
    pub max_elapsed_us: u64,
}

impl Timing {
    pub fn observe(&mut self, started_us: u64, completed_us: u64, failed: bool) {
        self.calls += 1;
        self.errors += u64::from(failed);
        self.started_us = started_us;
        self.completed_us = completed_us;
        self.elapsed_us = completed_us.saturating_sub(started_us);
        self.max_elapsed_us = self.max_elapsed_us.max(self.elapsed_us);
    }
}

#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct LogTiming {
    /// Complete JSONL record serialization/write, including implicit buffer flush.
    pub records: Timing,
    /// Explicit flush calls, excluding the final flush of the end record itself.
    pub flushes: Timing,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::path::PathBuf;

    use anyhow::Context as _;

    use crate::jog_diagnostic::platter::{CommonOptions, Config, Experiment, Wheel};
    use crate::jog_diagnostic::platter_session::Session;

    #[test]
    fn first_stop_timestamp_survives_repeated_requests_including_epoch_zero() {
        // Given a clock at the session epoch and an unrequested latch.
        let now = Arc::new(AtomicU64::new(0));
        let stop = StopRequest::new(Clock::Manual(Arc::clone(&now)));
        assert_eq!(stop.at_us(), None);
        // When more than one request arrives, then only the first time survives.
        stop.request();
        now.store(9000, Ordering::Release);
        stop.request();
        assert_eq!(stop.at_us(), Some(0));
    }

    struct AdvancingLog {
        now: Arc<AtomicU64>,
        bytes: Vec<u8>,
    }

    impl Write for AdvancingLog {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.now.fetch_add(2, Ordering::AcqRel);
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.now.fetch_add(11, Ordering::AcqRel);
            Ok(())
        }
    }

    #[test]
    fn log_record_and_flush_durations_use_the_injected_clock() -> anyhow::Result<()> {
        // Given a passive session whose log has deterministic write/flush durations.
        let now = Arc::new(AtomicU64::new(0));
        let clock = Clock::Manual(Arc::clone(&now));
        let stop = StopRequest::new(clock.clone());
        let mut log = AdvancingLog {
            now,
            bytes: Vec::new(),
        };
        {
            let mut session = Session::with_telemetry(
                Config {
                    common: CommonOptions {
                        out: PathBuf::from("unused"),
                        side: Wheel::Left,
                        seconds: 1,
                    },
                    experiment: Experiment::Measure,
                    counts_per_revolution: None,
                },
                |_| anyhow::bail!("passive mode must never write HID"),
                &mut log,
                clock,
                stop,
            );
            session.start()?;
            session.consumed(1000, 1500);
            session.consumed(2000, 2200);
            // When final telemetry is emitted, then native-neutral remains disabled.
            session.finish(0, "test")?;
        }
        let records: Vec<serde_json::Value> = std::str::from_utf8(&log.bytes)?
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
        let end = records.last().context("end record")?;
        assert_eq!(records[0]["version"], 2);
        assert_eq!(end["log_timing"]["flushes"]["elapsed_us"], 11);
        assert_eq!(end["log_timing"]["flushes"]["calls"], 1);
        assert!(
            end["log_timing"]["records"]["max_elapsed_us"]
                .as_u64()
                .context("record time")?
                > 0
        );
        let health = records
            .iter()
            .find(|r| r["kind"] == "platter_health")
            .context("health record")?;
        assert_eq!(health["consumed_frames"], 2);
        assert_eq!(health["consumer_lag_us"], 200);
        assert_eq!(health["max_consumer_lag_us"], 500);
        Ok(())
    }
}
