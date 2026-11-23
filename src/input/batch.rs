//! Snapshot batches and the native HID transport boundary.

use anyhow::{Context as _, ensure};
use hidapi::HidDevice;

use super::{ReadObserver, ReadOperation, accepts_interrupt, normalized_length, observed};

trait InputDevice {
    fn get_input_report(&self, report: &mut [u8]) -> anyhow::Result<usize>;
    fn read_timeout(&self, report: &mut [u8], timeout_ms: i32) -> anyhow::Result<usize>;
}

impl InputDevice for HidDevice {
    fn get_input_report(&self, report: &mut [u8]) -> anyhow::Result<usize> {
        Ok(super::native_read(
            u64::from(report[0]),
            report,
            |report| Self::get_input_report(self, report),
        )?)
    }

    fn read_timeout(&self, report: &mut [u8], timeout_ms: i32) -> anyhow::Result<usize> {
        Ok(super::native_read(0, report, |report| {
            Self::read_timeout(self, report, timeout_ms)
        })?)
    }
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
    hid: &impl InputDevice,
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
                hid.get_input_report(&mut buffers[index][..if cfg!(windows) { 79 } else { size }])
                    .with_context(|| format!("request S4 input report {id}"))
            })?;
            lengths[index] = normalized_length(&buffers[index][..lengths[index]], cfg!(windows))?;
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
    let length = normalized_length(&buffers[2][..length], cfg!(windows))?;
    if full {
        // Snapshot analog state every batch, even without a report-2 notification.
        // Queued interrupts cannot overwrite newer physical values.
        buffers[2][0] = 2;
        lengths[2] = observed(ReadOperation::Get2Analog, observe, || {
            hid.get_input_report(&mut buffers[2][..79])
                .context("request S4 analog report 2")
        })?;
        ensure!(
            lengths[2] == 79 && buffers[2][0] == 2,
            "unexpected S4 analog snapshot framing"
        );
    } else if accepts_interrupt(&buffers[2][..length], full) {
        lengths[2] = length;
    }
    Ok(lengths)
}

#[cfg(test)]
#[path = "batch/tests.rs"]
mod tests;
