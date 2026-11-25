//! S4 instance protocol over the installed `NIHardwareAgent`'s `CFMessagePort`s.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use s4_ni_message_port::{Local, Remote};
use tokio::sync::mpsc;
use tokio::time::Instant;

use super::{
    Cancellation, Connector, DeckOutput, Error, IO_TIMEOUT, Identity, Pipe, Raster, Session,
    protocol,
};

const BOOTSTRAP: &str = "NIHWMainHandler";
const NOTIFICATION_LIMIT: usize = 64;
const SEND_BUDGET: Duration = Duration::from_millis(25);

/// Both NI registrations for the selected S4, with one serialized output channel.
pub struct NiService(Session<Macos>);

impl NiService {
    /// Connect and acquire control. Requires a Tokio runtime with time enabled.
    ///
    /// # Errors
    /// Fails closed on missing service, identity mismatch, cancellation or timeout.
    pub async fn connect(identity: Identity, cancel: Cancellation) -> Result<Self, Error> {
        Session::connect(&Macos(BOOTSTRAP.to_owned()), identity, cancel)
            .await
            .map(Self)
    }

    /// Send both frames, verifying each service response (not physical output).
    ///
    /// # Errors
    /// Failure or an interrupted future closes this session.
    pub async fn publish_pair(&mut self, frames: [Raster<'_>; 2]) -> Result<(), Error> {
        self.0.publish_pair(frames).await
    }

    /// Consume pending notifications and verify ownership without sending output.
    ///
    /// # Errors
    /// Loss, disconnect, malformed input or cancellation closes this session.
    pub async fn poll_control(&mut self) -> Result<(), Error> {
        self.0.poll_control().await
    }

    /// Submit changed outputs for both decks. Submission is not hardware readback.
    ///
    /// # Errors
    /// Transport or control failure closes this session.
    pub async fn set_deck_output(&mut self, desired: [DeckOutput; 2]) -> Result<(), Error> {
        self.0.set_deck_output(desired).await
    }

    /// Like `set_deck_output`, but consult the guard immediately before each Play.
    /// A blocked Play returns `playing: false` for that deck.
    ///
    /// # Errors
    /// Transport or control failure closes this session.
    pub async fn set_deck_output_guarded(
        &mut self,
        desired: [DeckOutput; 2],
        allow_play: impl Fn(usize) -> bool + Send + Sync,
    ) -> Result<[DeckOutput; 2], Error> {
        self.0.set_deck_output_guarded(desired, allow_play).await
    }

    /// Submit the stock global `JOGT` property on the acquired instance.
    ///
    /// # Errors
    /// Transport or control failure closes this session.
    pub async fn set_jog_tension(&mut self, value: u8) -> Result<(), Error> {
        self.0.set_jog_tension(value).await
    }

    /// Submit both pauses before clearing rings and restoring JOG mode.
    ///
    /// # Errors
    /// Failure leaves the physical stop unconfirmed and closes this session.
    pub async fn quiesce_decks(&mut self) -> Result<(), Error> {
        self.0.quiesce_decks().await
    }

    /// Release this client's registrations without sending or flushing output.
    pub fn close(&mut self) {
        self.0.close();
    }
}

struct Macos(String);

struct Request(Arc<Remote>);

// A queued blocking task must not begin after the owning future is cancelled.
struct Abort(tokio::task::AbortHandle);

impl Drop for Abort {
    fn drop(&mut self) {
        self.0.abort();
    }
}

struct Notification {
    _local: Local,
    overflow: Arc<AtomicBool>,
    rx: mpsc::Receiver<Result<Vec<u8>, Error>>,
}

fn transport_error(error: s4_ni_message_port::Error) -> Error {
    use s4_ni_message_port::Error as Native;
    match error {
        Native::Unavailable => Error::Unsupported("NI message port unavailable"),
        Native::NameInUse => Error::Malformed("notification port already owned"),
        Native::Allocation => Error::Io(std::io::Error::other("Core Foundation allocation failed")),
        Native::Payload => Error::Malformed("NI message port payload length"),
        Native::Status(-1 | -2) => Error::Timeout,
        Native::Status(-5..=-3) => Error::Disconnected,
        Native::Status(status) => Error::Io(std::io::Error::other(format!(
            "CFMessagePort status {status}"
        ))),
    }
}

const fn message_id(message: &[u8]) -> Result<i32, Error> {
    let Some(bytes) = message.first_chunk::<4>() else {
        return Err(Error::Malformed("missing NI command"));
    };
    Ok(i32::from_le_bytes(*bytes))
}

impl Pipe for Request {
    async fn read(&mut self) -> Result<Vec<u8>, Error> {
        // A remote CFMessagePort has no separate unsolicited reply stream.
        if !self.0.is_valid() {
            return Err(Error::Disconnected);
        }
        std::future::pending().await
    }

    async fn write(&mut self, message: &[u8]) -> Result<(), Error> {
        // No reply wait, and the kernel send is bounded. A guarded Play cannot
        // sit in a user-space queue after the caller last checked desired state.
        self.0
            .send(message_id(message)?, message)
            .map_err(transport_error)
    }

    async fn request(&mut self, message: &[u8], deadline: Instant) -> Result<Vec<u8>, Error> {
        let id = message_id(message)?;
        let port = Arc::clone(&self.0);
        let message = message.to_vec();
        let remaining = deadline.saturating_duration_since(Instant::now());
        // CF has separate send and receive waits. A cancelled future drops the
        // owning registration; only an already-started call may finish locally.
        // No result or send can be reused by the next transaction.
        let deadline = std::time::Instant::now() + remaining;
        let mut call = tokio::task::spawn_blocking(move || {
            let Some(wait) = deadline
                .saturating_duration_since(std::time::Instant::now())
                .checked_sub(SEND_BUDGET)
                .filter(|wait| !wait.is_zero())
            else {
                return Err(Error::Timeout);
            };
            port.request(id, &message, wait, protocol::MAX_MESSAGE)
                .map_err(transport_error)
        });
        // Abort pending tasks on cancellation without waiting for running ones.
        let _abort = Abort(call.abort_handle());
        (&mut call)
            .await
            .map_err(|error| Error::Io(std::io::Error::other(error)))?
    }
}

impl Pipe for Notification {
    async fn read(&mut self) -> Result<Vec<u8>, Error> {
        let next = self.rx.recv().await;
        if self.overflow.load(Ordering::Acquire) {
            return Err(Error::Malformed("notification backlog"));
        }
        next.ok_or(Error::Disconnected)?
    }

    async fn write(&mut self, message: &[u8]) -> Result<(), Error> {
        // CF sends the empty acknowledgement from the callback, never via a
        // second message. This hook is only called when the session drains it.
        if !message.is_empty() {
            return Err(Error::Malformed("unexpected notification write"));
        }
        Ok(())
    }
}

impl Connector for Macos {
    type Request = Request;
    type Notification = Notification;

    async fn open(&self, name: &str) -> Result<Request, Error> {
        Remote::open(name)
            .map(|port| Request(Arc::new(port)))
            .map_err(transport_error)
    }

    fn serve(&self, name: &str) -> Result<Notification, Error> {
        let (tx, rx) = mpsc::channel(NOTIFICATION_LIMIT);
        let overflow = Arc::new(AtomicBool::new(false));
        let has_overflowed = Arc::clone(&overflow);
        let local = Local::bind(name, protocol::MAX_MESSAGE, move |_id, bytes| {
            if tx
                .try_send(bytes.map(<[u8]>::to_vec).map_err(transport_error))
                .is_err()
            {
                has_overflowed.store(true, Ordering::Release);
            }
            None
        })
        .map_err(transport_error)?;
        Ok(Notification {
            _local: local,
            overflow,
            rx,
        })
    }

    async fn bootstrap(&self, _name: &str, registration: &[u8]) -> Result<Vec<u8>, Error> {
        self.open(&self.0)
            .await?
            .request(registration, Instant::now() + IO_TIMEOUT)
            .await
    }

    async fn registration_ack(
        request: &mut Request,
        _notification: &mut Notification,
        _pending: &mut VecDeque<Vec<u8>>,
        ack: &[u8],
        deadline: Instant,
    ) -> Result<Vec<u8>, Error> {
        request
            .request(ack, deadline.min(Instant::now() + IO_TIMEOUT))
            .await
    }
}

#[cfg(test)]
#[path = "macos_tests.rs"]
mod tests;
