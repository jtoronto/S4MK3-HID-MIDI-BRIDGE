//! Native full-profile snapshots bypass the S4's slow interrupt cadence.
use anyhow::{Context as _, ensure};
use hidapi::HidDevice;

pub fn read_batch(
    hid: &HidDevice,
    full: bool,
    buffers: &mut [[u8; 128]; 3],
) -> anyhow::Result<[usize; 3]> {
    let mut lengths = [0; 3];
    if full {
        // Touch/buttons precede jog movement, as in a complete HID report.
        for (index, id, size) in [(0, 1, 23), (1, 3, 60)] {
            buffers[index][0] = id;
            lengths[index] = hid
                .get_input_report(&mut buffers[index][..size])
                .with_context(|| format!("request S4 input report {id}"))?;
            ensure!(
                lengths[index] == size && buffers[index][0] == id,
                "unexpected S4 snapshot {id} framing"
            );
        }
    }
    // In full mode this bounded read paces snapshots without a busy loop.
    // Minimal/probe mode retains the original interrupt-only transport.
    let length = hid
        .read_timeout(&mut buffers[2], if full { 2 } else { 100 })
        .context("read S4 HID interrupt report")?;
    if accepts_interrupt(&buffers[2][..length], full) {
        lengths[2] = length;
    }
    Ok(lengths)
}

const fn accepts_interrupt(report: &[u8], full: bool) -> bool {
    // These interrupt copies may predate the snapshots already processed.
    // Feeding either copy back to the parser would reverse motion/touch state.
    !(full && matches!(report.first(), Some(1 | 3)))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
