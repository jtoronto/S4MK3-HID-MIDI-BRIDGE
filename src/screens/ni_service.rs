//! `NIHardwareService` screen transport candidate; never opens USB or changes services.
//!
//! Protocol source: Encdr v0.8.4 `usb/ni_service/{protocol,windows}.rs`.
//! Its S4 frame/service acceptance is unverified. Registration changes NI client
//! state: callers must obtain operator authorization before `connect`.
//! No tasks or threads are spawned. Dropping a session closes only its own pipes.
//! An interrupted exchange poisons and closes the session so a late reply cannot
//! acknowledge a subsequent frame. Callers must retain desired frames across
//! errors and reconnect/redraw both screens, without a shutdown image.

#![cfg(any(windows, test))]

use std::collections::VecDeque;
use std::fmt;
use std::future::Future;
use std::io;
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::{Instant, timeout_at};

#[path = "ni_service/bootstrap.rs"]
mod bootstrap;
#[path = "ni_service/decks.rs"]
mod decks;
pub use super::DeckOutput;
#[cfg(test)]
#[path = "ni_service/fixtures.rs"]
mod fixtures;
#[cfg(feature = "attended-ni-jog")]
#[path = "ni_service/left_jog.rs"]
mod left_jog;
#[path = "ni_service/protocol.rs"]
mod protocol;
#[cfg(windows)]
#[path = "ni_service/windows.rs"]
mod windows;
#[path = "ni_service/write.rs"]
mod write;
#[cfg(windows)]
pub use windows::NiService;

const SETUP_TIMEOUT: Duration = Duration::from_secs(8);
const IO_TIMEOUT: Duration = Duration::from_millis(1500);

/// Terminal transport failures must not be converted into "displayed".
#[derive(Debug)]
pub enum Error {
    Unsupported(&'static str),
    Identity(&'static str),
    Rejected,
    Timeout,
    Malformed(&'static str),
    ControlLost,
    Cancelled,
    Disconnected,
    ShortWrite { expected: usize, actual: usize },
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NI screen transport: {self:?}")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::BrokenPipe
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::UnexpectedEof => Self::Disconnected,
            _ => Self::Io(error),
        }
    }
}

const fn written(expected: usize, actual: usize) -> Result<(), Error> {
    if actual != expected {
        return Err(Error::ShortWrite { expected, actual });
    }
    Ok(())
}

/// A paired, present S4 HID/display identity supplied by the platform enumerator.
///
/// The caller must enumerate the *complete* current candidate sets. Both sets
/// must contain exactly one unit, with the same container and known serial.
/// The service must subsequently report that exact serial; no fallback exists.
#[derive(Clone, Debug)]
pub struct Identity {
    serial: String,
}

#[derive(Clone, Copy, Debug)]
pub struct DeviceIdentity<'a> {
    pub vendor: u16,
    pub product: u16,
    pub container: &'a str,
    pub serial: &'a str,
}

impl Identity {
    /// Bind the exact serial of the uniquely selected live S4 HID collection.
    ///
    /// The caller must have rejected ambiguous S4 control collections. The NI
    /// service subsequently must report this same serial before registration.
    /// `MI_04` access is owned by the NI service, never opened by this adapter.
    ///
    /// # Errors
    /// Rejects missing or unsupported serial spelling without a device fallback.
    pub fn selected_hid(serial: &str) -> Result<Self, Error> {
        protocol::serial(serial.as_bytes())?;
        Ok(Self {
            serial: serial.to_owned(),
        })
    }

    #[must_use]
    pub fn serial(&self) -> &str {
        &self.serial
    }

    /// Correlate present HID and `MI_04` inventory before making any service call.
    ///
    /// # Errors
    /// Rejects missing, ambiguous, mismatched or incomplete identities.
    pub fn correlate(
        hid: &[DeviceIdentity<'_>],
        mi04: &[DeviceIdentity<'_>],
    ) -> Result<Self, Error> {
        let ([hid], [display]) = (hid, mi04) else {
            return Err(Error::Identity("exactly one HID and MI_04 required"));
        };
        for item in [hid, display] {
            if item.vendor != 0x17cc || item.product != 0x1720 {
                return Err(Error::Identity("not an S4 MK3"));
            }
            if item.container.is_empty() || item.container.contains('\0') {
                return Err(Error::Identity("missing container"));
            }
            protocol::serial(item.serial.as_bytes())?;
        }
        if hid.container != display.container || hid.serial != display.serial {
            return Err(Error::Identity("HID and MI_04 do not correlate"));
        }
        Ok(Self {
            serial: hid.serial.to_owned(),
        })
    }
}

/// An exactly sized logical RGB565 raster (the existing renderer's pixel format).
#[derive(Clone, Copy)]
pub struct Raster<'a> {
    pixels: &'a [u16],
    order: protocol::PixelOrder,
}

impl<'a> Raster<'a> {
    /// # Errors
    /// Rejects a raster other than 320 by 240 pixels.
    pub const fn new(pixels: &'a [u16]) -> Result<Self, Error> {
        if pixels.len() != protocol::PIXELS {
            return Err(Error::Malformed("raster dimensions"));
        }
        Ok(Self {
            pixels,
            // Keep the attended Encdr diagnostic payload contract.
            order: protocol::PixelOrder::Bgr565,
        })
    }

    /// Construct a raster for the normal NI screen presenter.
    /// Its service payload uses RGB565: the raw USB BGR swap produces reversed
    /// colors on this path. `new` retains the attended Encdr diagnostic bytes.
    ///
    /// # Errors
    /// Rejects a raster other than 320 by 240 pixels.
    pub fn presenter(pixels: &'a [u16]) -> Result<Self, Error> {
        Ok(Self {
            order: protocol::PixelOrder::Rgb565,
            ..Self::new(pixels)?
        })
    }
}

/// Cancellation is level-triggered. Dropping the sender also cancels.
/// Keep a sender alive with `false`; set `true` to stop setup or an I/O wait.
pub type Cancellation = watch::Receiver<bool>;

async fn cancelled(cancel: &mut Cancellation) {
    loop {
        if *cancel.borrow_and_update() {
            return;
        }
        if cancel.changed().await.is_err() {
            return;
        }
    }
}

async fn bounded<T: Send>(
    cancel: &mut Cancellation,
    deadline: Instant,
    operation: impl Future<Output = Result<T, Error>> + Send,
) -> Result<T, Error> {
    tokio::select! {
        biased;
        () = cancelled(cancel) => Err(Error::Cancelled),
        result = timeout_at(deadline, operation) => result.map_err(|_| Error::Timeout)?,
    }
}

// Message-oriented boundary, implemented by safe nonblocking Windows pipes.
// read() is cancellation-safe: a pending call must not consume message bytes.
// Every write is ONE complete message, including zero-length notification ACKs.
trait Pipe: Send {
    fn read(&mut self) -> impl Future<Output = Result<Vec<u8>, Error>> + Send;
    fn write(&mut self, message: &[u8]) -> impl Future<Output = Result<(), Error>> + Send;

    fn write_guarded(
        &mut self,
        message: &[u8],
        guard: &(dyn Fn() -> bool + Sync),
    ) -> impl Future<Output = Result<bool, Error>> + Send {
        async move {
            if !guard() {
                return Ok(false);
            }
            self.write(message).await?;
            Ok(true)
        }
    }
}

trait Connector: Send + Sync {
    type Request: Pipe;
    type Notification: Pipe;
    fn open(&self, name: &str) -> impl Future<Output = Result<Self::Request, Error>> + Send;
    fn serve(&self, name: &str) -> Result<Self::Notification, Error>;

    // The default drives the existing message peers. Windows overrides only
    // this one-shot exchange: NI closes its bootstrap immediately after reply.
    fn bootstrap(
        &self,
        name: &str,
        registration: &[u8],
    ) -> impl Future<Output = Result<Vec<u8>, Error>> + Send {
        async {
            let mut pipe = self.open(name).await?;
            pipe.write(registration).await?;
            pipe.read().await
        }
    }
}

struct Registration<C: Connector> {
    request: C::Request,
    notification: C::Notification,
    pending: VecDeque<Vec<u8>>,
    ack_required: bool,
}

impl<C: Connector> Registration<C> {
    async fn open(
        connector: &C,
        registration: &[u8],
        cancel: &mut Cancellation,
        deadline: Instant,
    ) -> Result<Self, Error> {
        tracing::info!(stage = "bootstrap", kind = ?registration.get(..4), "NI_SERVICE_SETUP");
        let reply = bounded(
            cancel,
            deadline,
            connector.bootstrap("NIHWMainHandler", registration),
        )
        .await?;
        tracing::info!(stage = "bootstrap-reply", bytes = ?reply, "NI_SERVICE_SETUP");
        let ports = protocol::ports(&reply)?;
        tracing::info!(stage = "registration-ports", pipe = %ports.request, "NI_SERVICE_SETUP");
        // Create the listener BEFORE acknowledging its name to the service.
        let notification = connector.serve(&ports.notification)?;
        let request = bounded(cancel, deadline, connector.open(&ports.request)).await?;
        let mut result = Self {
            request,
            notification,
            pending: VecDeque::new(),
            ack_required: false,
        };
        let ack = protocol::ack(&ports.notification)?;
        bounded(cancel, deadline, result.request.write(&ack)).await?;
        tracing::info!(stage = "registration-ack-reply", "NI_SERVICE_SETUP");
        // ACK replies are required; an absent or unrecognized reply is not success.
        let reply = bounded(cancel, deadline, async {
            loop {
                tokio::select! {
                    biased;
                    note = result.notification.read() => {
                        if result.pending.len() == 64 {
                            return Err(Error::Malformed("notification backlog"));
                        }
                        result.pending.push_back(note?);
                        result.notification.write(&[]).await?;
                    }
                    reply = result.request.read() => return reply,
                }
            }
        })
        .await?;
        protocol::accepted(&reply)?;
        tracing::info!(stage = "registration-accepted", "NI_SERVICE_SETUP");
        Ok(result)
    }

    async fn wait_for_device(
        &mut self,
        identity: &Identity,
        cancel: &mut Cancellation,
        deadline: Instant,
    ) -> Result<(), Error> {
        loop {
            let message = bounded(cancel, deadline, self.notification()).await?;
            let state = protocol::notification(&message)?;
            bounded(cancel, deadline, self.acknowledge()).await?;
            match state {
                protocol::Notification::Device { present, serial } if serial == identity.serial => {
                    if !present {
                        return Err(Error::Disconnected);
                    }
                    return Ok(());
                }
                protocol::Notification::Device { .. } => {}
                protocol::Notification::Control(_) | protocol::Notification::Other => {
                    return Err(Error::Malformed("expected device identity notification"));
                }
            }
        }
    }

    async fn notification(&mut self) -> Result<Vec<u8>, Error> {
        if let Some(message) = self.pending.pop_front() {
            Ok(message)
        } else {
            let message = self.notification.read().await?;
            self.ack_required = true;
            Ok(message)
        }
    }

    async fn acknowledge(&mut self) -> Result<(), Error> {
        if self.ack_required {
            self.notification.write(&[]).await?;
            self.ack_required = false;
        }
        Ok(())
    }
}

/// A session retains device and instance registrations until explicitly closed.
/// Generic internals also let offline tests drive the real session through messages.
struct Session<C: Connector> {
    resources: Option<Resources<C>>,
    cancel: Cancellation,
    identity: Identity,
    packet: Vec<u8>,
    decks: Option<[DeckOutput; 2]>,
    deck_speeds_initialized: bool,
}

struct Resources<C: Connector> {
    device: Registration<C>,
    instance: Registration<C>,
    active: bool,
}

impl<C: Connector> Session<C> {
    async fn connect(
        connector: &C,
        identity: Identity,
        mut cancel: Cancellation,
    ) -> Result<Self, Error> {
        let deadline = Instant::now() + SETUP_TIMEOUT;
        let mut device = Registration::open(
            connector,
            &protocol::device_connect(),
            &mut cancel,
            deadline,
        )
        .await?;
        tracing::info!(stage = "device-query", "NI_SERVICE_SETUP");
        bounded(
            &mut cancel,
            deadline,
            device.request.write(&protocol::words(&[0x0344_7143])),
        )
        .await?;
        tracing::info!(stage = "device-identity", "NI_SERVICE_SETUP");
        // Device reports for other serials are never a substitute for this unit.
        device
            .wait_for_device(&identity, &mut cancel, deadline)
            .await?;
        tracing::info!(stage = "instance-registration", "NI_SERVICE_SETUP");
        let mut instance = Registration::open(
            connector,
            &protocol::instance_connect(&identity.serial)?,
            &mut cancel,
            deadline,
        )
        .await?;
        let mut active = false;
        while let Some(note) = instance.pending.pop_front() {
            Self::instance_note(&mut active, &note)?;
        }
        for message in protocol::setup() {
            tracing::info!(stage = "instance-setup", command = ?message.get(..8), "NI_SERVICE_SETUP");
            bounded(&mut cancel, deadline, instance.request.write(&message)).await?;
        }
        // These configuration writes have no request replies. Ownership is
        // established only by the independent control notification below;
        // inspect rejects any queued unsolicited reply before a frame write.
        // Registration alone never authorizes a frame.
        tracing::info!(stage = "control-acquisition", "NI_SERVICE_SETUP");
        while !active {
            let message = bounded(&mut cancel, deadline, instance.notification()).await?;
            match protocol::notification(&message)? {
                protocol::Notification::Control(value) => active = value,
                protocol::Notification::Other => {}
                protocol::Notification::Device { .. } => {
                    return Err(Error::Malformed("device message on instance port"));
                }
            }
            bounded(&mut cancel, deadline, instance.acknowledge()).await?;
        }
        Ok(Self {
            resources: Some(Resources {
                device,
                instance,
                active,
            }),
            cancel,
            identity,
            packet: Vec::with_capacity(protocol::DISPLAY_BYTES),
            decks: None,
            deck_speeds_initialized: false,
        })
    }

    // Processes queued notifications before every frame and during each exchange.
    // A ready loss/cancel wins over a ready write/reply.
    async fn exchange(
        resources: &mut Resources<C>,
        serial: &str,
        cancel: &mut Cancellation,
        packet: &[u8],
        budget: Duration,
    ) -> Result<(), Error> {
        let deadline = Instant::now() + budget;
        Self::inspect(resources, serial, cancel, deadline).await?;
        Self::transact(resources, serial, cancel, packet, deadline).await
    }

    async fn inspect(
        resources: &mut Resources<C>,
        serial: &str,
        cancel: &mut Cancellation,
        deadline: Instant,
    ) -> Result<(), Error> {
        // Check already queued notification/reply state without waiting.
        loop {
            tokio::select! {
                biased;
                () = cancelled(cancel) => return Err(Error::Cancelled),
                () = tokio::time::sleep_until(deadline) => return Err(Error::Timeout),
                note = resources.device.notification() => {
                    Self::device_note(serial, &note?)?;
                    bounded(cancel, deadline, resources.device.acknowledge()).await?;
                }
                note = resources.instance.notification.read() => {
                    Self::instance_note(&mut resources.active, &note?)?;
                    if !resources.active { return Err(Error::ControlLost); }
                    bounded(cancel, deadline, resources.instance.notification.write(&[])).await?;
                }
                reply = resources.instance.request.read() => {
                    reply?;
                    return Err(Error::Malformed("unsolicited request reply"));
                }
                () = std::future::ready(()) => break,
            }
        }
        if !resources.active {
            return Err(Error::ControlLost);
        }
        Ok(())
    }

    async fn transact(
        resources: &mut Resources<C>,
        serial: &str,
        cancel: &mut Cancellation,
        packet: &[u8],
        deadline: Instant,
    ) -> Result<(), Error> {
        // Pinning the whole exchange preserves one write and one reply across
        // unrelated notifications. Cancellation/loss drops the owning resources.
        let request = &mut resources.instance.request;
        let transaction = async {
            request.write(packet).await?;
            protocol::accepted(&request.read().await?)
        };
        tokio::pin!(transaction);
        loop {
            tokio::select! {
                biased;
                () = cancelled(cancel) => return Err(Error::Cancelled),
                () = tokio::time::sleep_until(deadline) => return Err(Error::Timeout),
                note = resources.device.notification() => {
                    Self::device_note(serial, &note?)?;
                    bounded(cancel, deadline, resources.device.acknowledge()).await?;
                }
                note = resources.instance.notification.read() => {
                    Self::instance_note(&mut resources.active, &note?)?;
                    if !resources.active { return Err(Error::ControlLost); }
                    bounded(cancel, deadline, resources.instance.notification.write(&[])).await?;
                }
                result = &mut transaction => return result,
            }
        }
    }

    fn device_note(serial: &str, message: &[u8]) -> Result<(), Error> {
        match protocol::notification(message)? {
            protocol::Notification::Device {
                present: false,
                serial: observed,
            } if observed == serial => Err(Error::Disconnected),
            protocol::Notification::Device { .. } | protocol::Notification::Other => Ok(()),
            protocol::Notification::Control(_) => Err(Error::Malformed("control on device port")),
        }
    }

    fn instance_note(active: &mut bool, message: &[u8]) -> Result<(), Error> {
        match protocol::notification(message)? {
            protocol::Notification::Control(value) => {
                *active = value;
                Ok(())
            }
            protocol::Notification::Other => Ok(()),
            protocol::Notification::Device { .. } => {
                Err(Error::Malformed("device on instance port"))
            }
        }
    }

    async fn publish_pair(&mut self, frames: [Raster<'_>; 2]) -> Result<(), Error> {
        self.publish_pair_with_budget(frames, IO_TIMEOUT).await
    }

    async fn publish_pair_with_budget(
        &mut self,
        frames: [Raster<'_>; 2],
        per_frame: Duration,
    ) -> Result<(), Error> {
        // Move ownership into this future. Dropping the future mid-exchange
        // closes all pipes rather than returning a half-used session to callers.
        let mut resources = self.resources.take().ok_or(Error::Disconnected)?;
        for (target_index, raster) in [0_u8, 1].into_iter().zip(frames) {
            protocol::display(target_index, raster.pixels, raster.order, &mut self.packet);
            if tracing::enabled!(target: "s4_screen_route_diag", tracing::Level::DEBUG) {
                tracing::debug!(
                    target: "s4_screen_route_diag",
                    target_index,
                    header_target = self.packet[4],
                    embedded_target = self.packet[22],
                    hash = crate::screens::route_fingerprint(raster.pixels),
                    "SCREEN_ROUTE_SEND"
                );
            }
            let result = Self::exchange(
                &mut resources,
                &self.identity.serial,
                &mut self.cancel,
                &self.packet,
                per_frame,
            )
            .await;
            tracing::debug!(target: "s4_screen_route_diag", target_index, result = ?result, "SCREEN_ROUTE_RESPONSE");
            result?;
        }
        self.resources = Some(resources);
        Ok(())
    }

    async fn poll_control(&mut self) -> Result<(), Error> {
        let mut resources = self.resources.take().ok_or(Error::Disconnected)?;
        Self::inspect(
            &mut resources,
            &self.identity.serial,
            &mut self.cancel,
            Instant::now() + IO_TIMEOUT,
        )
        .await?;
        self.resources = Some(resources);
        Ok(())
    }

    fn close(&mut self) {
        self.resources = None;
    }
}

#[cfg(test)]
#[path = "ni_service/tests.rs"]
mod tests;
