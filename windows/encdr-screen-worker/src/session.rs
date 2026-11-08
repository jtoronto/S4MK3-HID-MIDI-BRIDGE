//! Synchronous service calls are bounded by the containing supervisor process.
use anyhow::{Context, bail};
use crossbeam_channel::{Receiver, TryRecvError, bounded, select_biased};
use std::io::{Read, Write};

#[cfg(windows)]
use crate::protocol::{HEIGHT, WIDTH};
use crate::{
    frames::Encoder,
    protocol::{self, Record},
};

pub trait Screens {
    /// The actual upstream control notifications; channel closure is failure.
    fn control(&self) -> &Receiver<bool>;
    fn send_display(&self, display: u8, commands: &[u8]) -> anyhow::Result<()>;
}

fn control_change(active: bool) -> anyhow::Result<()> {
    if !active {
        bail!("NI service control lost");
    }
    Ok(())
}

fn check_control(control: &Receiver<bool>) -> anyhow::Result<()> {
    loop {
        match control.try_recv() {
            Ok(active) => control_change(active)?,
            Err(TryRecvError::Empty) => return Ok(()),
            Err(TryRecvError::Disconnected) => bail!("NI service control notifications closed"),
        }
    }
}

pub fn run<R: Read + Send + 'static, S: Screens>(
    mut input: R,
    output: &mut impl Write,
    connect: impl FnOnce() -> anyhow::Result<S>,
) -> anyhow::Result<()> {
    protocol::arm(&mut input)?;
    // The only call site capable of registering with the service follows ARM.
    let service = connect()?;
    while !service
        .control()
        .recv()
        .context("NI service control notifications closed before grant")?
    {}
    check_control(service.control())?;
    let encoder = Encoder::new()?;
    protocol::emit(output, "READY")?;

    // One bounded input slot prevents an eager producer growing the worker.
    // The reader may block in stdin; process exit reaps it. Never join it here.
    let (sender, records) = bounded(1);
    std::thread::Builder::new()
        .name("screen-input".into())
        .spawn(move || {
            loop {
                let record = protocol::record(&mut input);
                let terminal = !matches!(record, Ok(Record::Frame(_)));
                if sender.send(record).is_err() || terminal {
                    break;
                }
            }
        })
        .context("start screen input reader")?;

    loop {
        select_biased! {
            recv(service.control()) -> active => {
                control_change(active.context("NI service control notifications closed")?)?;
            }
            recv(records) -> record => {
                match record.context("screen input reader ended")?? {
                    Record::Stop => {
                        drop(service);
                        return protocol::emit(output, "STOPPED");
                    }
                    Record::Frame(pixels) => {
                        let commands = encoder.pair(&pixels)?;
                        for (display, frame) in [0, 1].into_iter().zip(&commands) {
                            check_control(service.control())?;
                            service.send_display(display, frame)?;
                        }
                        // A control loss during either blocking call cannot be
                        // acknowledged as a completed pair once observed.
                        check_control(service.control())?;
                        protocol::emit(output, "FRAME_OK")?;
                    }
                }
            }
        }
    }
}

#[cfg(windows)]
pub struct NativeScreens {
    session: encdr::usb::ni_service::ServiceSession,
    control: Receiver<bool>,
}

#[cfg(windows)]
impl NativeScreens {
    pub fn connect(serial: &str) -> anyhow::Result<Self> {
        use std::time::Duration;
        // Unmodified upstream may fall back to another reported serial.
        // This timeout does not bound every synchronous upstream pipe call.
        let session = encdr::usb::ni_service::ServiceSession::connect(
            0x1720,
            Some(serial),
            Duration::from_secs(5),
        )
        .map_err(anyhow::Error::msg)?;
        let changes = session.control_events();
        let (sender, control) = bounded(1);
        std::thread::Builder::new()
            .name("screen-control".into())
            .spawn(move || {
                while let Ok(active) = changes.recv_blocking() {
                    if sender.send(active).is_err() {
                        break;
                    }
                }
            })
            .context("start NI control notification reader")?;
        Ok(Self { session, control })
    }
}

#[cfg(windows)]
impl Screens for NativeScreens {
    fn control(&self) -> &Receiver<bool> {
        &self.control
    }

    fn send_display(&self, display: u8, commands: &[u8]) -> anyhow::Result<()> {
        self.session
            .send_display(display, WIDTH, HEIGHT, commands)
            .map_err(anyhow::Error::msg)
            .context("NI display send failed")
    }
}
