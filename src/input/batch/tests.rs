use std::cell::Cell;

use super::*;

struct Device {
    analog: Cell<u16>,
    interrupt: Vec<u8>,
    snapshots: Cell<usize>,
}

impl Device {
    fn new(interrupt: Vec<u8>) -> Self {
        Self {
            analog: Cell::new(2695),
            interrupt,
            snapshots: Cell::new(0),
        }
    }
}

impl InputDevice for Device {
    fn get_input_report(&self, report: &mut [u8]) -> anyhow::Result<usize> {
        self.snapshots.set(self.snapshots.get() + 1);
        let id = report[0];
        let size = match id {
            1 => 23,
            2 => 79,
            3 => 60,
            _ => anyhow::bail!("unexpected snapshot ID {id}"),
        };
        report.fill(0);
        report[0] = id;
        if id == 2 {
            report[1..3].copy_from_slice(&self.analog.get().to_le_bytes());
        }
        Ok(if cfg!(windows) { report.len() } else { size })
    }

    fn read_timeout(&self, report: &mut [u8], _: i32) -> anyhow::Result<usize> {
        report[..self.interrupt.len()].copy_from_slice(&self.interrupt);
        Ok(self.interrupt.len())
    }
}

#[test]
fn full_batches_refresh_analog_without_analog_notifications() -> anyhow::Result<()> {
    // No notification, old buttons, old jogs and old analog data must all
    // permit the current analog snapshot rather than replaying queued state.
    for (id, size) in [(0, 0), (1, 23), (3, 60), (2, 79)] {
        let mut interrupt = vec![0; size];
        if let Some(first) = interrupt.first_mut() {
            *first = id;
        }
        let device = Device::new(interrupt);
        let mut buffers = [[0; 128]; 3];
        for value in [2695_u16, 3006] {
            device.analog.set(value);
            let lengths = read_batch_inner(&device, true, &mut buffers, &mut None)?;

            assert_eq!(lengths, [23, 60, 79], "interrupt ID {id}");
            assert_eq!(buffers[0][0], 1);
            assert_eq!(buffers[1][0], 3);
            assert_eq!(buffers[2][0], 2);
            assert_eq!(u16::from_le_bytes([buffers[2][1], buffers[2][2]]), value);
        }
    }
    Ok(())
}

#[test]
fn minimal_batches_retain_interrupt_data_without_snapshots() -> anyhow::Result<()> {
    for (id, size) in [(0, 0), (1, 23), (2, 79), (3, 60)] {
        let mut interrupt = vec![0x5a; size];
        if let Some(first) = interrupt.first_mut() {
            *first = id;
        }
        let device = Device::new(interrupt.clone());
        let mut buffers = [[0; 128]; 3];

        let lengths = read_batch_inner(&device, false, &mut buffers, &mut None)?;

        assert_eq!(lengths, [0, 0, size]);
        assert_eq!(&buffers[2][..size], interrupt);
        assert_eq!(device.snapshots.get(), 0);
    }
    Ok(())
}
