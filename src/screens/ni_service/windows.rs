//! Prearmed overlapped bootstrap; nonblocking message pipes after registration.

use super::{
    Cancellation, Connector, DeckOutput, Error, Identity, Pipe, Raster, Session, bootstrap,
    protocol, write,
};
use interprocess::os::windows::named_pipe::{
    DuplexPipeStream, PipeListener, PipeListenerOptions, PipeMode, pipe_mode,
};
use recvmsg::{RecvResult, prelude::*};
use std::fs::OpenOptions;
use std::io;
use std::os::windows::{fs::OpenOptionsExt as _, io::OwnedHandle};

type Stream = DuplexPipeStream<pipe_mode::Messages>;
type Listener = PipeListener<pipe_mode::Messages, pipe_mode::Messages>;
use write::POLL;

pub struct NiService(Session<Windows>);

impl NiService {
    /// Register with the running `NIHardwareService` and wait for actual control.
    ///
    /// Requires a Tokio runtime with time and I/O enabled. Bootstrap uses
    /// overlapped I/O without an independent worker. Setup is bounded to eight
    /// seconds, including open, write and reply.
    ///
    /// # Errors
    /// Returns identity, registration, protocol, cancellation or timeout failures.
    pub async fn connect(identity: Identity, cancel: Cancellation) -> Result<Self, Error> {
        Session::connect(&Windows, identity, cancel).await.map(Self)
    }

    /// Publish two full rasters, stopping on loss or interruption.
    ///
    /// # Cancel safety
    /// Dropping this future closes the session without flushing. Reconnect before
    /// publishing again. The caller retains desired frames for the next session.
    ///
    /// # Errors
    /// A failed or interrupted exchange closes the session. `Ok` establishes
    /// service acceptance only, never physical display acceptance.
    pub async fn publish_pair(&mut self, frames: [Raster<'_>; 2]) -> Result<(), Error> {
        self.0.publish_pair(frames).await
    }

    /// Process queued service/control notifications without sending an image.
    ///
    /// # Errors
    /// Loss, disconnect, malformed input, cancellation or ACK timeout closes the
    /// session. Dropping this future also closes it.
    pub async fn poll_control(&mut self) -> Result<(), Error> {
        self.0.poll_control().await
    }

    /// Submit changed deck outputs, including the JOG playback haptic state.
    ///
    /// # Errors
    /// Transport loss closes the session and leaves the physical state unconfirmed.
    pub async fn set_deck_output(&mut self, desired: [DeckOutput; 2]) -> Result<(), Error> {
        self.0.set_deck_output(desired).await
    }

    /// Submit the global JOG tension property on the acquired instance.
    ///
    /// The stock capture confirms the write but not service reply semantics.
    /// Success confirms transport submission only, not physical effect.
    ///
    /// # Errors
    /// Transport failure closes the session; no retry is attempted.
    pub async fn set_jog_tension(&mut self, value: u8) -> Result<(), Error> {
        self.0.set_jog_tension(value).await
    }

    /// Apply outputs, checking `allow_play` immediately before each Play write.
    ///
    /// Returns submitted states; a rejected Play has `playing` set to false.
    /// A rejected Play does not close the session.
    ///
    /// # Errors
    /// Transport loss closes the session and leaves physical state unconfirmed.
    pub async fn set_deck_output_guarded(
        &mut self,
        desired: [DeckOutput; 2],
        allow_play: impl Fn(usize) -> bool + Send + Sync,
    ) -> Result<[DeckOutput; 2], Error> {
        self.0.set_deck_output_guarded(desired, allow_play).await
    }

    /// Pause both motors, clear both rings and restore JOG mode.
    ///
    /// # Errors
    /// Transport loss closes the session; the physical stop is unconfirmed.
    pub async fn quiesce_decks(&mut self) -> Result<(), Error> {
        self.0.quiesce_decks().await
    }

    /// Run one attended left platter trial while publishing both screen rasters.
    ///
    /// # Errors
    /// Returns any transport failure and closes the session. An interrupted
    /// exchange can make motor-off delivery unverifiable.
    #[cfg(feature = "attended-ni-jog")]
    pub async fn run_left_jog_once(&mut self, stop: Cancellation) -> Result<(), Error> {
        self.0.run_left_jog_once(stop).await
    }

    /// Close this adapter's resources only, without a shutdown image or flush.
    pub fn close(&mut self) {
        self.0.close();
    }
}

struct Windows;

// Interprocess normally transfers dirty handles to a flushing thread on Drop.
// Explicit evade_limbo is essential here: cancellation must close immediately
// without preserving queued writes or starting any helper thread.
struct Owned(Option<Stream>);

impl Drop for Owned {
    fn drop(&mut self) {
        if let Some(stream) = self.0.take() {
            stream.evade_limbo();
        }
    }
}

impl Owned {
    fn stream(&mut self) -> Result<&mut Stream, Error> {
        self.0.as_mut().ok_or(Error::Disconnected)
    }
}

struct Notification {
    listener: Option<Listener>,
    stream: Owned,
}

fn path(name: &str) -> String {
    format!(r"\\.\pipe\{name}")
}

impl Connector for Windows {
    type Request = Owned;
    type Notification = Notification;

    async fn bootstrap(&self, name: &str, registration: &[u8]) -> Result<Vec<u8>, Error> {
        bootstrap::transact(name, registration).await
    }

    async fn open(&self, name: &str) -> Result<Owned, Error> {
        loop {
            // The safe standard-library open does not call WaitNamedPipe. The
            // older Interprocess synchronous constructor would wait internally;
            // wrap this owned synchronous handle instead.
            match OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(0)
                .security_qos_flags(0x0001_0000) // SECURITY_IDENTIFICATION
                .open(path(name))
            {
                Ok(file) => {
                    let handle: OwnedHandle = file.into();
                    let stream = Stream::try_from(handle)
                        .map_err(|_| Error::Unsupported("not a message-mode service pipe"))?;
                    let mut owned = Owned(Some(stream));
                    owned.stream()?.set_nonblocking(true)?;
                    return Ok(owned);
                }
                Err(error) if error.raw_os_error() == Some(231) => {
                    tokio::time::sleep(POLL).await;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    return Err(Error::Unsupported("service request pipe unavailable"));
                }
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn serve(&self, name: &str) -> Result<Notification, Error> {
        let listener = PipeListenerOptions::new()
            .path(path(name))
            .mode(PipeMode::Messages)
            .nonblocking(true)
            .accept_remote(false)
            .input_buffer_size_hint(65_536)
            .output_buffer_size_hint(65_536)
            .create_duplex::<pipe_mode::Messages>()?;
        Ok(Notification {
            listener: Some(listener),
            stream: Owned(None),
        })
    }
}

impl Pipe for Owned {
    async fn read(&mut self) -> Result<Vec<u8>, Error> {
        loop {
            let stream = self.stream()?;
            let length = stream.peek_msg_len()?;
            if length > protocol::MAX_MESSAGE {
                return Err(Error::Malformed("pipe message exceeds capacity"));
            }
            if length != 0 {
                // There is exactly one reader; a peeked message cannot change
                // before receive. Capacity is bounded before any allocation.
                let mut buffer = MsgBuf::from(Vec::with_capacity(length));
                match stream.recv_msg(&mut buffer, None)? {
                    RecvResult::Fit if buffer.filled_part().len() == length => {
                        return Ok(buffer.filled_part().to_vec());
                    }
                    RecvResult::EndOfStream => return Err(Error::Disconnected),
                    _ => return Err(Error::Malformed("message length changed after peek")),
                }
            }
            tokio::time::sleep(POLL).await;
        }
    }

    async fn write(&mut self, message: &[u8]) -> Result<(), Error> {
        let stream = self.stream()?;
        write::message(message, |bytes| stream.send(bytes)).await
    }

    async fn write_guarded(
        &mut self,
        message: &[u8],
        guard: &(dyn Fn() -> bool + Sync),
    ) -> Result<bool, Error> {
        let stream = self.stream()?;
        write::guarded_message(message, |bytes| stream.send(bytes), guard).await
    }
}

impl Pipe for Notification {
    async fn read(&mut self) -> Result<Vec<u8>, Error> {
        while self.stream.0.is_none() {
            let listener = self.listener.as_ref().ok_or(Error::Disconnected)?;
            match listener.accept() {
                Ok(stream) => {
                    self.stream = Owned(Some(stream));
                    self.listener = None;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(POLL).await;
                }
                Err(error) => return Err(error.into()),
            }
        }
        self.stream.read().await
    }

    async fn write(&mut self, message: &[u8]) -> Result<(), Error> {
        self.stream.write(message).await
    }
}

#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "bootstrap_windows_tests.rs"]
mod bootstrap_tests;
