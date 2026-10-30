//! Native full-profile snapshots bypass the S4's slow interrupt cadence.
use anyhow::{Context as _, ensure};
use hidapi::HidDevice;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadOperation {
    Get1,
    Get3,
    InterruptRead,
    Get2Analog,
}

pub trait ReadObserver {
    fn begin(&mut self, operation: ReadOperation) -> anyhow::Result<()>;

    fn end(
        &mut self,
        operation: ReadOperation,
        duration: std::time::Duration,
        failed: bool,
    ) -> anyhow::Result<()>;
}

pub fn read_analog(hid: &HidDevice) -> anyhow::Result<[u8; 79]> {
    read_analog_with_observer(
        hid,
        None,
        "request initial S4 analog report 2",
        "unexpected initial S4 analog snapshot framing",
    )
}

pub fn read_analog_with_observer(
    hid: &HidDevice,
    observer: Option<&mut dyn ReadObserver>,
    context: &'static str,
    framing: &'static str,
) -> anyhow::Result<[u8; 79]> {
    let mut report = [0_u8; 79];
    report[0] = 2;
    let mut observer = observer;
    let length = observed(ReadOperation::Get2Analog, &mut observer, || {
        hid.get_input_report(&mut report).context(context)
    })?;
    ensure!(length == report.len() && report[0] == 2, "{framing}");
    Ok(report)
}

pub fn read_batch(
    hid: &HidDevice,
    full: bool,
    buffers: &mut [[u8; 128]; 3],
) -> anyhow::Result<[usize; 3]> {
    read_batch_with_observer(hid, full, buffers, None)
}

pub fn read_batch_with_observer(
    hid: &HidDevice,
    full: bool,
    buffers: &mut [[u8; 128]; 3],
    observer: Option<&mut dyn ReadObserver>,
) -> anyhow::Result<[usize; 3]> {
    let mut observer = observer;
    read_batch_inner(hid, full, buffers, &mut observer)
}

fn read_batch_inner(
    hid: &HidDevice,
    full: bool,
    buffers: &mut [[u8; 128]; 3],
    observe: &mut Option<&mut dyn ReadObserver>,
) -> anyhow::Result<[usize; 3]> {
    let mut lengths = [0; 3];
    if full {
        // Touch/buttons precede jog movement, as in a complete HID report.
        for (index, id, size, operation) in [
            (0, 1, 23, ReadOperation::Get1),
            (1, 3, 60, ReadOperation::Get3),
        ] {
            buffers[index][0] = id;
            lengths[index] = observed(operation, observe, || {
                hid.get_input_report(&mut buffers[index][..size])
                    .with_context(|| format!("request S4 input report {id}"))
            })?;
            ensure!(
                lengths[index] == size && buffers[index][0] == id,
                "unexpected S4 snapshot {id} framing"
            );
        }
    }
    // In full mode this bounded read paces snapshots without a busy loop.
    // Minimal/probe mode retains the original interrupt-only transport.
    let length = observed(ReadOperation::InterruptRead, observe, || {
        hid.read_timeout(&mut buffers[2], if full { 2 } else { 100 })
            .context("read S4 HID interrupt report")
    })?;
    if full && length != 0 && buffers[2][0] == 2 {
        // A queued analog interrupt can predate the startup GET. Use it as
        // a change notification, not as authority over a newer physical snapshot.
        let report = observe.take().map_or_else(
            || read_analog(hid),
            |observer| {
                read_analog_with_observer(
                    hid,
                    Some(observer),
                    "request initial S4 analog report 2",
                    "unexpected initial S4 analog snapshot framing",
                )
            },
        )?;
        buffers[2][..report.len()].copy_from_slice(&report);
        lengths[2] = report.len();
    } else if accepts_interrupt(&buffers[2][..length], full) {
        lengths[2] = length;
    }
    Ok(lengths)
}

fn observed<T>(
    operation: ReadOperation,
    observe: &mut Option<&mut dyn ReadObserver>,
    read: impl FnOnce() -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    match observe.as_deref_mut() {
        Some(observe) => {
            observe.begin(operation)?;
            let started = std::time::Instant::now();
            let result = read();
            let duration = started.elapsed();
            let observation = observe.end(operation, duration, result.is_err());
            match (result, observation) {
                (Ok(value), Ok(())) => Ok(value),
                (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
                (Err(error), Err(observation_error)) => Err(error.context(format!(
                    "native read failed; telemetry observation also failed: {observation_error:#}"
                ))),
            }
        }
        None => read(),
    }
}

const fn accepts_interrupt(report: &[u8], full: bool) -> bool {
    // These interrupt copies may predate the snapshots already processed.
    // Feeding either copy back to the parser would reverse motion/touch state.
    !(full && matches!(report.first(), Some(1 | 3)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[derive(Debug, PartialEq)]
    enum RecordedCall {
        Begin(ReadOperation),
        End {
            operation: ReadOperation,
            duration: Duration,
            failed: bool,
        },
    }

    #[derive(Default)]
    struct Recorder {
        calls: Vec<RecordedCall>,
        fail_end: bool,
    }

    impl ReadObserver for Recorder {
        fn begin(&mut self, operation: ReadOperation) -> anyhow::Result<()> {
            self.calls.push(RecordedCall::Begin(operation));
            Ok(())
        }

        fn end(
            &mut self,
            operation: ReadOperation,
            duration: Duration,
            failed: bool,
        ) -> anyhow::Result<()> {
            self.calls.push(RecordedCall::End {
                operation,
                duration,
                failed,
            });
            if self.fail_end {
                anyhow::bail!("telemetry callback failed");
            }
            Ok(())
        }
    }

    #[test]
    fn snapshot_mode_does_not_replay_stale_jog_or_touch_interrupts() {
        // Given old jog/touch interrupts after fresh snapshots.
        for id in [1, 3] {
            // Then full mode rejects them, but the original profile keeps them.
            assert!(!accepts_interrupt(&[id], true));
            assert!(accepts_interrupt(&[id], false));
        }
        // Other report families still reach the original decoder.
        assert!(accepts_interrupt(&[2], true));
        assert!(accepts_interrupt(&[4], true));
    }

    #[test]
    fn observed_native_call_records_begin_and_success() -> anyhow::Result<()> {
        // Given a deterministic native-call substitute and recording callback.
        let (result, calls) = {
            let mut recorder = Recorder::default();
            let mut observer = Some(&mut recorder as &mut dyn ReadObserver);

            // When the shared operation wrapper completes a GET1-style call.
            let result = observed(ReadOperation::Get1, &mut observer, || {
                Ok::<_, anyhow::Error>(23)
            })?;
            (result, recorder.calls)
        };

        // Then begin/end are recorded around its unchanged result.
        assert_eq!(result, 23);
        assert!(matches!(
            calls.as_slice(),
            [
                RecordedCall::Begin(ReadOperation::Get1),
                RecordedCall::End {
                    operation: ReadOperation::Get1,
                    failed: false,
                    ..
                }
            ]
        ));
        Ok(())
    }

    #[test]
    fn observed_native_call_records_failure_before_propagation() {
        // Given a deterministic failing native-call substitute.
        let (result, calls) = {
            let mut recorder = Recorder::default();
            let mut observer = Some(&mut recorder as &mut dyn ReadObserver);

            // When the operation fails.
            let result: anyhow::Result<()> =
                observed(ReadOperation::InterruptRead, &mut observer, || {
                    anyhow::bail!("native read failed")
                });
            (result, recorder.calls)
        };

        // Then the failed result propagates and its terminal observation remains visible.
        assert!(result.is_err());
        assert!(matches!(
            calls.as_slice(),
            [
                RecordedCall::Begin(ReadOperation::InterruptRead),
                RecordedCall::End {
                    operation: ReadOperation::InterruptRead,
                    failed: true,
                    ..
                }
            ]
        ));
    }

    #[test]
    fn telemetry_end_failure_does_not_replace_native_failure() -> anyhow::Result<()> {
        // Given a failing native operation and a failing end-observation callback.
        let mut recorder = Recorder {
            fail_end: true,
            ..Recorder::default()
        };
        let mut observer = Some(&mut recorder as &mut dyn ReadObserver);

        // When both failures occur around the same native operation.
        let result: anyhow::Result<()> =
            observed(ReadOperation::InterruptRead, &mut observer, || {
                anyhow::bail!("native read failed")
            });
        let error = result
            .err()
            .ok_or_else(|| anyhow::anyhow!("native read should fail"))?;

        // Then the native error remains in the chain alongside observation context.
        assert!(
            error
                .to_string()
                .contains("telemetry observation also failed")
        );
        assert!(
            error
                .chain()
                .any(|cause| cause.to_string() == "native read failed")
        );
        Ok(())
    }
}
