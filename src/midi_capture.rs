//! Temporary, opt-in raw MIDI evidence. Producers never perform file I/O.

use std::fs::OpenOptions;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context as _, ensure};
use crossbeam_channel::{Receiver, Sender, TrySendError};
use serde::Serialize;

const CAPACITY: usize = 1024;
const MAX_BYTES: usize = 1024;
const MAX_MARKER: usize = 96;
const MAX_FILE_BYTES: usize = 64 * 1024 * 1024;
const SUMMARY_RESERVE: usize = 4096;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Input,
    OutputAttempt,
    Marker,
}

struct Packet {
    sequence: u64,
    elapsed_us: u128,
    kind: Kind,
    callback_us: Option<u64>,
    bytes: [u8; MAX_BYTES],
    len: usize,
}

#[derive(Debug, Default)]
struct Counters {
    attempted: AtomicU64,
    queued: AtomicU64,
    written: AtomicU64,
    full: AtomicU64,
    oversized: AtomicU64,
    disconnected: AtomicU64,
    truncated: AtomicBool,
    limit: AtomicU64,
    limit_queued: AtomicU64,
}

#[derive(Debug, Serialize)]
struct Summary {
    attempted: u64,
    queued: u64,
    written: u64,
    dropped_full: u64,
    dropped_oversized: u64,
    dropped_disconnected: u64,
    unwritten: u64,
    truncated: bool,
    dropped_limit: u64,
    /// Subset of `dropped_limit` already included in queued and unwritten.
    dropped_limit_queued: u64,
}

impl Counters {
    fn summary(&self) -> Summary {
        let queued = self.queued.load(Ordering::Relaxed);
        let written = self.written.load(Ordering::Relaxed);
        Summary {
            attempted: self.attempted.load(Ordering::Relaxed),
            queued,
            written,
            dropped_full: self.full.load(Ordering::Relaxed),
            dropped_oversized: self.oversized.load(Ordering::Relaxed),
            dropped_disconnected: self.disconnected.load(Ordering::Relaxed),
            unwritten: queued.saturating_sub(written),
            truncated: self.truncated.load(Ordering::Acquire),
            dropped_limit: self.limit.load(Ordering::Relaxed),
            dropped_limit_queued: self.limit_queued.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
struct Ingress {
    sender: Sender<Packet>,
    started: Instant,
    counters: Arc<Counters>,
}

/// Weak ownership lets shutdown close the queue even with detached stdin alive.
#[derive(Clone, Debug, Default)]
pub struct Handle(Weak<Ingress>);

#[derive(Debug, Serialize)]
pub struct Receipt {
    accepted: bool,
    sequence: Option<u64>,
    elapsed_us: Option<u128>,
    reason: Option<&'static str>,
}

impl Receipt {
    const fn rejected(reason: &'static str) -> Self {
        Self {
            accepted: false,
            sequence: None,
            elapsed_us: None,
            reason: Some(reason),
        }
    }

    pub fn emit(&self, id: u64) -> anyhow::Result<()> {
        let mut status = serde_json::to_value(self)?;
        status["version"] = 1.into();
        status["state"] = "marker".into();
        status["request_id"] = id.to_string().into();
        emit(&status)
    }
}

fn emit(status: &serde_json::Value) -> anyhow::Result<()> {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "S4_MIDI_CAPTURE_STATUS {status}")?;
    stdout.flush()?;
    Ok(())
}

impl Handle {
    pub fn input(&self, callback_us: u64, bytes: &[u8]) {
        self.record(Kind::Input, Some(callback_us), bytes);
    }

    pub fn output(&self, bytes: &[u8]) {
        self.record(Kind::OutputAttempt, None, bytes);
    }

    pub fn marker(&self, label: &str) -> Receipt {
        if label.is_empty()
            || label.len() > MAX_MARKER
            || !label.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Receipt::rejected("invalid_label");
        }
        self.record(Kind::Marker, None, label.as_bytes())
    }

    fn record(&self, kind: Kind, callback_us: Option<u64>, bytes: &[u8]) -> Receipt {
        self.0.upgrade().map_or_else(
            || Receipt::rejected("inactive"),
            |ingress| {
                ingress.record(
                    kind,
                    callback_us,
                    bytes,
                    ingress.started.elapsed().as_micros(),
                )
            },
        )
    }
}

impl Ingress {
    fn record(
        &self,
        kind: Kind,
        callback_us: Option<u64>,
        bytes: &[u8],
        elapsed_us: u128,
    ) -> Receipt {
        let sequence = self.counters.attempted.fetch_add(1, Ordering::Relaxed);
        let reason = if self.counters.truncated.load(Ordering::Acquire) {
            self.counters.limit.fetch_add(1, Ordering::Relaxed);
            Some("recording_limit")
        } else if bytes.len() > MAX_BYTES {
            self.counters.oversized.fetch_add(1, Ordering::Relaxed);
            Some("oversized")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Receipt {
                accepted: false,
                sequence: Some(sequence),
                elapsed_us: Some(elapsed_us),
                reason: Some(reason),
            };
        }
        let mut packet = Packet {
            sequence,
            elapsed_us,
            kind,
            callback_us,
            bytes: [0; MAX_BYTES],
            len: bytes.len(),
        };
        packet.bytes[..bytes.len()].copy_from_slice(bytes);
        let reason = match self.sender.try_send(packet) {
            Ok(()) => {
                self.counters.queued.fetch_add(1, Ordering::Relaxed);
                None
            }
            Err(TrySendError::Full(_)) => {
                self.counters.full.fetch_add(1, Ordering::Relaxed);
                Some("queue_full")
            }
            Err(TrySendError::Disconnected(_)) => {
                self.counters.disconnected.fetch_add(1, Ordering::Relaxed);
                Some("writer_closed")
            }
        };
        Receipt {
            accepted: reason.is_none(),
            sequence: Some(sequence),
            elapsed_us: Some(elapsed_us),
            reason,
        }
    }
}

pub struct Capture {
    path: PathBuf,
    ingress: Option<Arc<Ingress>>,
    counters: Arc<Counters>,
    worker: Option<JoinHandle<anyhow::Result<()>>>,
}

impl Capture {
    pub fn start(path: &Path) -> anyhow::Result<Self> {
        ensure!(path.is_absolute(), "MIDI capture path must be absolute");
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!("create MIDI capture {}", path.display()))?;
        let started = Instant::now();
        let mut writer = BufWriter::new(file);
        let header = serde_json::json!({
            "type": "session", "version": 1,
            "unix_epoch_us": SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros(),
            "queue_capacity": CAPACITY, "max_packet_bytes": MAX_BYTES,
            "max_file_bytes": MAX_FILE_BYTES,
            "clock": "capture_monotonic_microseconds",
            "output_semantics": "attempt_before_send",
        });
        let mut header = serde_json::to_vec(&header)?;
        header.push(b'\n');
        ensure!(
            header.len() <= MAX_FILE_BYTES - SUMMARY_RESERVE,
            "MIDI capture header exceeds file budget"
        );
        writer.write_all(&header)?;
        writer.flush()?;
        let (sender, receiver) = crossbeam_channel::bounded(CAPACITY);
        let counters = Arc::new(Counters::default());
        let worker_counters = Arc::clone(&counters);
        let worker_path = path.to_owned();
        let worker = std::thread::Builder::new()
            .name("s4-midi-capture".to_owned())
            .spawn(move || {
                let result = write_packets(
                    &mut writer,
                    &receiver,
                    &worker_counters,
                    MAX_FILE_BYTES - header.len(),
                    || {
                        emit(&serde_json::json!({
                            "version": 1, "state": "limited", "path": worker_path,
                            "reason": "file_limit", "max_file_bytes": MAX_FILE_BYTES,
                            "counts": worker_counters.summary(),
                        }))
                    },
                );
                if let Err(error) = &result {
                    tracing::error!(%error, "MIDI_CAPTURE_WRITE_FAILED");
                }
                result
            })
            .context("start MIDI capture writer")?;
        tracing::info!(path = %path.display(), "MIDI_CAPTURE_STARTED");
        let capture = Self {
            path: path.to_owned(),
            ingress: Some(Arc::new(Ingress {
                sender,
                started,
                counters: Arc::clone(&counters),
            })),
            counters,
            worker: Some(worker),
        };
        emit(&serde_json::json!({
            "version": 1, "state": "started", "path": path, "max_file_bytes": MAX_FILE_BYTES
        }))?;
        Ok(capture)
    }

    pub fn handle(&self) -> Handle {
        Handle(self.ingress.as_ref().map_or_else(Weak::new, Arc::downgrade))
    }

    pub fn finish(&mut self) -> anyhow::Result<()> {
        // Every in-flight producer releases its strong reference before receiver
        // disconnection. Thus the final summary includes all accepted attempts.
        self.ingress.take();
        if let Some(worker) = self.worker.take() {
            let result = worker
                .join()
                .map_err(|_| anyhow::anyhow!("MIDI capture writer panicked"))
                .and_then(std::convert::identity);
            let summary = self.counters.summary();
            tracing::info!(?summary, success = result.is_ok(), "MIDI_CAPTURE_FINISHED");
            let status = emit(&serde_json::json!({
                "version": 1, "state": if result.is_ok() { "finished" } else { "failed" },
                "path": self.path, "counts": summary,
                "error": result.as_ref().err().map(|error| format!("{error:#}")),
            }));
            result?;
            status?;
        }
        Ok(())
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if let Err(error) = self.finish() {
            tracing::error!(%error, "MIDI_CAPTURE_CLOSE_FAILED");
        }
    }
}

fn write_packets(
    writer: &mut impl std::io::Write,
    receiver: &Receiver<Packet>,
    counters: &Counters,
    mut remaining: usize,
    on_limit: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    ensure!(
        remaining >= SUMMARY_RESERVE,
        "MIDI capture summary reserve missing"
    );
    let mut on_limit = Some(on_limit);
    // A failed limit notification is retained here and reported only after
    // the queue drains and the reserved summary is written below.
    let mut notify_error: Option<anyhow::Error> = None;
    for packet in receiver {
        if counters.truncated.load(Ordering::Acquire) {
            counters.limit.fetch_add(1, Ordering::Relaxed);
            counters.limit_queued.fetch_add(1, Ordering::Relaxed);
            continue;
        }
        let bytes = &packet.bytes[..packet.len];
        let record = serde_json::json!({
            "type": "event", "sequence": packet.sequence,
            "elapsed_us": packet.elapsed_us, "kind": packet.kind,
            "callback_us": packet.callback_us,
            "bytes": if matches!(packet.kind, Kind::Marker) { &[][..] } else { bytes },
            "marker": if matches!(packet.kind, Kind::Marker) {
                Some(std::str::from_utf8(bytes)?)
            } else { None },
            "loss": counters.summary(),
        });
        let mut record = serde_json::to_vec(&record)?;
        record.push(b'\n');
        if record.len() > remaining - SUMMARY_RESERVE {
            counters.limit.fetch_add(1, Ordering::Relaxed);
            counters.limit_queued.fetch_add(1, Ordering::Relaxed);
            counters.truncated.store(true, Ordering::Release);
            if let Some(notify) = on_limit.take() {
                if let Err(error) = notify() {
                    tracing::error!(%error, "MIDI_CAPTURE_NOTIFY_FAILED");
                    notify_error = Some(error);
                }
            }
            continue;
        }
        writer.write_all(&record)?;
        writer.flush()?;
        remaining -= record.len();
        counters.written.fetch_add(1, Ordering::Relaxed);
    }
    let mut summary =
        serde_json::to_vec(&serde_json::json!({"type": "summary", "counts": counters.summary()}))?;
    summary.push(b'\n');
    ensure!(
        summary.len() <= SUMMARY_RESERVE,
        "MIDI capture summary exceeds reserved space"
    );
    let summary_result: anyhow::Result<()> = (|| {
        writer.write_all(&summary)?;
        writer.flush()?;
        Ok(())
    })();
    match (summary_result, notify_error) {
        (Ok(()), None) => Ok(()),
        (Ok(()), Some(error)) => Err(error),
        (Err(summary_error), None) => Err(summary_error),
        (Err(summary_error), Some(notify_error)) => Err(summary_error.context(format!(
            "MIDI capture limit notification also failed: {notify_error:#}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue(capacity: usize) -> (Arc<Ingress>, Receiver<Packet>) {
        let (sender, receiver) = crossbeam_channel::bounded(capacity);
        (
            Arc::new(Ingress {
                sender,
                started: Instant::now(),
                counters: Arc::new(Counters::default()),
            }),
            receiver,
        )
    }

    fn records(bytes: &[u8]) -> anyhow::Result<Vec<serde_json::Value>> {
        std::str::from_utf8(bytes)?
            .lines()
            .map(|line| Ok(serde_json::from_str(line)?))
            .collect()
    }

    #[test]
    fn file_limit_includes_header_and_summary_and_latches_at_exact_event_boundary()
    -> anyhow::Result<()> {
        // Calibrate the serialized event size so the boundary stays independent
        // of JSON field ordering or future additive metadata.
        let header = b"{\"type\":\"session\"}\n";
        let mut first_event_bytes = None;
        for reduction in [None, Some(0), Some(1)] {
            let (ingress, receiver) = queue(3);
            let counters = Arc::clone(&ingress.counters);
            ingress.record(Kind::Input, Some(1), &[0xf8], 10);
            ingress.record(Kind::Input, Some(2), &[127; MAX_BYTES], 11);
            let marker = ingress.record(Kind::Marker, None, b"queued_marker", 12);
            assert!(marker.accepted);
            assert_eq!(marker.sequence, Some(2));
            drop(ingress);
            let limit = match reduction {
                None => MAX_FILE_BYTES,
                Some(reduction) => {
                    header.len()
                        + SUMMARY_RESERVE
                        + first_event_bytes.context("calibrated event size")?
                        - reduction
                }
            };
            let mut output = header.to_vec();
            let mut notifications = 0;
            // When the real writer has unlimited, exact-fit, or one-byte-short space.
            write_packets(
                &mut output,
                &receiver,
                &counters,
                limit - header.len(),
                || {
                    notifications += 1;
                    assert!(counters.summary().truncated);
                    Ok(())
                },
            )?;
            let rows = records(&output)?;
            let counts = counters.summary();
            // Then only complete events fit, the summary stays inside the cap,
            // and a smaller later event cannot resume a truncated recording.
            let (written, limited) = match reduction {
                None => (3, false),
                Some(0) => (1, true),
                Some(_) => (0, true),
            };
            assert!(output.len() <= limit);
            assert_eq!(counts.written, written);
            assert_eq!(counts.truncated, limited);
            assert_eq!(notifications, usize::from(limited));
            assert_eq!(counts.dropped_limit, 3 - written);
            assert_eq!(counts.dropped_limit_queued, 3 - written);
            assert_eq!(counts.unwritten, counts.queued - counts.written);
            assert_eq!(
                counts.attempted,
                counts.queued
                    + counts.dropped_full
                    + counts.dropped_oversized
                    + counts.dropped_disconnected
                    + counts.dropped_limit
                    - counts.dropped_limit_queued
            );
            assert_eq!(rows.last().context("summary")?["type"], "summary");
            assert_eq!(
                rows.last().context("summary")?["counts"]["dropped_limit"],
                3 - written
            );
            if reduction.is_none() {
                first_event_bytes = Some(
                    output[header.len()..]
                        .iter()
                        .position(|byte| *byte == b'\n')
                        .context("first event newline")?
                        + 1,
                );
            }
        }
        Ok(())
    }

    #[test]
    fn limited_producers_report_loss_without_queueing_or_losing_marker_identity()
    -> anyhow::Result<()> {
        // Given the same terminal flag published by the bounded writer.
        let (ingress, receiver) = queue(1);
        let handle = Handle(Arc::downgrade(&ingress));
        ingress.counters.truncated.store(true, Ordering::Release);
        // When all producer classes arrive after the cap, including an oversized packet.
        handle.input(9, &[127; MAX_BYTES + 1]);
        handle.output(&[0x90, 0, 127]);
        let marker = handle.marker("after_limit");
        // Then no queue slot is consumed and the marker has an explicit rejection.
        assert!(!marker.accepted);
        assert_eq!(marker.sequence, Some(2));
        assert!(marker.elapsed_us.is_some());
        assert_eq!(marker.reason, Some("recording_limit"));
        assert!(receiver.is_empty());
        let counts = ingress.counters.summary();
        assert_eq!(counts.attempted, 3);
        assert_eq!(counts.queued, 0);
        assert_eq!(counts.dropped_limit, 3);
        assert_eq!(counts.dropped_limit_queued, 0);
        assert_eq!(counts.dropped_oversized, 0);
        assert_eq!(
            counts.attempted,
            counts.queued
                + counts.dropped_full
                + counts.dropped_oversized
                + counts.dropped_disconnected
                + counts.dropped_limit
                - counts.dropped_limit_queued
        );
        let counters = Arc::clone(&ingress.counters);
        drop(ingress);
        let mut output = Vec::new();
        write_packets(&mut output, &receiver, &counters, MAX_FILE_BYTES, || Ok(()))?;
        let rows = records(&output)?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["counts"]["dropped_limit"], 3);
        assert_eq!(rows[0]["counts"]["truncated"], true);
        Ok(())
    }

    #[test]
    fn bounded_queue_preserves_raw_packets_and_accounts_every_loss() -> anyhow::Result<()> {
        // Given a one-slot queue with no consumer, overflow is deterministic.
        let (ingress, receiver) = queue(1);
        let counters = Arc::clone(&ingress.counters);
        let mut raw = [0x7f; MAX_BYTES];
        raw[..4].copy_from_slice(&[0xf8, 0x90, 0, 127]);
        ingress.record(Kind::Input, Some(91), &raw, 12);
        ingress.record(Kind::OutputAttempt, None, &[0x80, 0, 0], 13);
        ingress.record(Kind::Input, Some(92), &[1; MAX_BYTES + 1], 14);
        // When shutdown disconnects the producer and drains accepted packets.
        drop(ingress);
        let mut output = Vec::new();
        write_packets(&mut output, &receiver, &counters, MAX_FILE_BYTES, || Ok(()))?;
        // Then rejected packets are counted whole, never silently truncated.
        let records = records(&output)?;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["bytes"], serde_json::json!(raw.as_slice()));
        assert_eq!(records[0]["callback_us"], 91);
        assert_eq!(records[0]["elapsed_us"], 12);
        assert_eq!(
            records[1]["counts"],
            serde_json::json!({
                "attempted": 3, "queued": 1, "written": 1,
                "dropped_full": 1, "dropped_oversized": 1,
                "dropped_disconnected": 0, "unwritten": 0,
                "truncated": false, "dropped_limit": 0, "dropped_limit_queued": 0
            })
        );
        Ok(())
    }

    #[test]
    fn writer_failure_accounts_queued_and_later_disconnected_attempts() {
        // Given an accepted packet and a sink that fails its first write.
        struct Failed;
        impl std::io::Write for Failed {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("injected storage failure"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let (ingress, receiver) = queue(2);
        ingress.record(Kind::Input, Some(1), &[0xf8], 10);
        // When storage fails and the consumer departs.
        assert!(
            write_packets(
                &mut Failed,
                &receiver,
                &ingress.counters,
                MAX_FILE_BYTES,
                || Ok(())
            )
            .is_err()
        );
        drop(receiver);
        ingress.record(Kind::OutputAttempt, None, &[0x90, 0, 127], 11);
        // Then the accepted but unwritten packet and later loss remain visible.
        let summary = ingress.counters.summary();
        assert_eq!(summary.attempted, 2);
        assert_eq!(summary.written, 0);
        assert_eq!(summary.unwritten, 1);
        assert_eq!(summary.dropped_disconnected, 1);
    }

    #[test]
    fn failed_limit_notification_still_drains_counts_and_writes_summary() -> anyhow::Result<()> {
        use std::io::ErrorKind;
        // Given three queued events and a budget that fits exactly the first.
        let first_event_bytes = {
            let (probe, receiver) = queue(8);
            probe.record(Kind::Input, Some(7), &[0xf8], 10);
            probe.record(Kind::OutputAttempt, None, &[0x90, 0, 127], 11);
            probe.record(Kind::Marker, None, b"cap_note", 12);
            let counters = Arc::clone(&probe.counters);
            drop(probe);
            let mut output = Vec::new();
            write_packets(&mut output, &receiver, &counters, MAX_FILE_BYTES, || Ok(()))?;
            output
                .iter()
                .position(|byte| *byte == b'\n')
                .context("first event newline")?
                + 1
        };
        let (ingress, receiver) = queue(8);
        ingress.record(Kind::Input, Some(7), &[0xf8], 10);
        ingress.record(Kind::OutputAttempt, None, &[0x90, 0, 127], 11);
        let marker = ingress.record(Kind::Marker, None, b"cap_note", 12);
        assert!(marker.accepted);
        let counters = Arc::clone(&ingress.counters);
        drop(ingress);
        // When the limit notification fails with a broken stdout pipe.
        let mut notifications = 0;
        let mut output = Vec::new();
        let Err(error) = write_packets(
            &mut output,
            &receiver,
            &counters,
            SUMMARY_RESERVE + first_event_bytes,
            || -> anyhow::Result<()> {
                notifications += 1;
                Err(std::io::Error::from(ErrorKind::BrokenPipe).into())
            },
        ) else {
            return Err(anyhow::anyhow!("limit notification failure expected"));
        };
        // Then the failure is the notification itself, yet every queued record
        // drained, the loss was counted, and the reserved summary was written.
        assert_eq!(
            error
                .downcast_ref::<std::io::Error>()
                .map(std::io::Error::kind),
            Some(ErrorKind::BrokenPipe)
        );
        assert_eq!(notifications, 1);
        assert!(receiver.is_empty());
        let rows = records(&output)?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["kind"], "input");
        assert_eq!(rows[1]["type"], "summary");
        let counts = counters.summary();
        assert_eq!(counts.queued, 3);
        assert_eq!(counts.written, 1);
        assert!(counts.truncated);
        assert_eq!(counts.dropped_limit, 2);
        assert_eq!(counts.dropped_limit_queued, 2);
        assert_eq!(counts.unwritten, 2);
        assert_eq!(rows[1]["counts"]["written"], 1);
        assert_eq!(rows[1]["counts"]["dropped_limit"], 2);
        assert_eq!(
            counts.attempted,
            counts.queued
                + counts.dropped_full
                + counts.dropped_oversized
                + counts.dropped_disconnected
                + counts.dropped_limit
                - counts.dropped_limit_queued
        );
        Ok(())
    }

    #[test]
    fn finish_drains_file_with_live_weak_marker_handle_and_preserves_existing_file()
    -> anyhow::Result<()> {
        // Given a real exclusively created file and a detached-style weak handle.
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "s4-midi-capture-{}-{}.jsonl",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut capture = Capture::start(&path)?;
        let handle = capture.handle();
        handle.input(44, &[0xf0, 1, 2, 0xf7]);
        let marker = handle.marker("before_play");
        assert!(marker.accepted);
        handle.output(&[0x90, 0, 127]);
        // When shutdown joins while that handle is still alive.
        capture.finish()?;
        capture.finish()?;
        assert_eq!(handle.marker("after_close").reason, Some("inactive"));
        let bytes = std::fs::read(&path)?;
        let result = (|| -> anyhow::Result<()> {
            let rows = records(&bytes)?;
            assert_eq!(rows.len(), 5);
            assert_eq!(rows[0]["type"], "session");
            assert_eq!(rows[1]["kind"], "input");
            assert_eq!(rows[1]["bytes"], serde_json::json!([240, 1, 2, 247]));
            assert_eq!(rows[2]["kind"], "marker");
            assert_eq!(rows[2]["marker"], "before_play");
            assert_eq!(rows[2]["sequence"], serde_json::json!(marker.sequence));
            assert_eq!(rows[2]["elapsed_us"], serde_json::json!(marker.elapsed_us));
            assert_eq!(rows[3]["kind"], "output_attempt");
            let times: Vec<_> = rows[1..4]
                .iter()
                .map(|row| row["elapsed_us"].as_u64().context("elapsed time"))
                .collect::<anyhow::Result<_>>()?;
            assert!(times.windows(2).all(|pair| pair[0] <= pair[1]));
            assert_eq!(rows[4]["counts"]["written"], 3);
            assert!(Capture::start(&path).is_err());
            assert_eq!(std::fs::read(&path)?, bytes);
            Ok(())
        })();
        std::fs::remove_file(path)?;
        result
    }

    #[test]
    fn markers_reject_unbounded_or_ambiguous_labels_before_enqueue() -> anyhow::Result<()> {
        let (ingress, receiver) = queue(1);
        let handle = Handle(Arc::downgrade(&ingress));
        for invalid in ["", "two words", "line\nbreak", "\u{7f}", &"a".repeat(97)] {
            assert_eq!(handle.marker(invalid).reason, Some("invalid_label"));
        }
        assert!(handle.marker("a_b-1").accepted);
        let full = handle.marker("overflow");
        assert!(!full.accepted);
        assert_eq!(full.reason, Some("queue_full"));
        assert_eq!(full.sequence, Some(1));
        let packet = receiver.try_recv()?;
        assert_eq!(&packet.bytes[..packet.len], b"a_b-1");
        drop(receiver);
        let closed = handle.marker("writer_gone");
        assert!(!closed.accepted);
        assert_eq!(closed.reason, Some("writer_closed"));
        assert_eq!(closed.sequence, Some(2));
        let summary = ingress.counters.summary();
        assert_eq!(summary.attempted, 3);
        assert_eq!(summary.dropped_full, 1);
        assert_eq!(summary.dropped_disconnected, 1);
        Ok(())
    }
}
