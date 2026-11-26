//! The one-shot bootstrap must have a posted read before sending registration.

use super::Error;
use std::future::{Future, poll_fn};
use std::task::Poll;

// Poll explicitly: constructing or pinning a lazy read future does not post I/O.
// The caller owns the pipe inside its bounded future; cancellation drops it.
async fn prearmed(
    read: impl Future<Output = Result<Vec<u8>, Error>>,
    write: impl Future<Output = Result<(), Error>>,
) -> Result<Vec<u8>, Error> {
    tokio::pin!(read);
    poll_fn(|cx| {
        Poll::Ready(match read.as_mut().poll(cx) {
            Poll::Pending => Ok(()),
            Poll::Ready(Err(error)) => Err(error),
            Poll::Ready(Ok(_)) => Err(Error::Malformed("unsolicited bootstrap reply")),
        })
    })
    .await?;
    write.await?;
    read.await
}

#[cfg(windows)]
pub(super) async fn transact(name: &str, registration: &[u8]) -> Result<Vec<u8>, Error> {
    use super::{protocol, write::POLL, written};
    use std::io;
    use tokio::net::windows::named_pipe::{ClientOptions, PipeMode};

    let pipe = loop {
        // ClientOptions uses FILE_FLAG_OVERLAPPED and never WaitNamedPipe.
        // Its Mio registration posts the initial read before open returns;
        // prearmed also polls our reader explicitly before allowing the write.
        match ClientOptions::new()
            .pipe_mode(PipeMode::Message)
            .security_qos_flags(0x0001_0000) // SECURITY_IDENTIFICATION
            .open(format!(r"\\.\pipe\{name}"))
        {
            Ok(pipe) => break pipe,
            Err(error) if error.raw_os_error() == Some(231) => {
                tokio::time::sleep(POLL).await;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(Error::Unsupported("service request pipe unavailable"));
            }
            Err(error) => return Err(error.into()),
        }
    };
    // All valid connect replies fit in 498 bytes, below Mio's 4096-byte
    // overlapped read buffer. One extra byte makes overlong replies fail closed;
    // never assemble a second message or accept a truncated valid prefix.
    let read = async {
        let mut reply = vec![0; protocol::MAX_CONNECT_REPLY + 1];
        loop {
            pipe.readable().await?;
            match pipe.try_read(&mut reply) {
                Ok(0) => return Err(Error::Disconnected),
                Ok(count) if count > protocol::MAX_CONNECT_REPLY => {
                    return Err(Error::Malformed("connect reply size"));
                }
                Ok(count) => {
                    reply.truncate(count);
                    return Ok(reply);
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
        }
    };
    let write = async {
        loop {
            pipe.writable().await?;
            match pipe.try_write(registration) {
                // Exactly one full message; never send a short-write suffix.
                Ok(count) => return written(registration.len(), count),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
        }
    };
    prearmed(read, write).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::ni_service::{bounded, watch};
    use std::time::Duration;
    use tokio::sync::oneshot;
    use tokio::time::Instant;

    #[tokio::test(start_paused = true)]
    async fn read_is_polled_before_write_can_deliver_reply() -> Result<(), Error> {
        let (armed, posted) = oneshot::channel();
        let (reply, received) = oneshot::channel();
        let (_cancel, mut cancellation) = watch::channel(false);
        let read = async {
            armed.send(()).map_err(|()| Error::Disconnected)?;
            received.await.map_err(|_| Error::Disconnected)
        };
        let write = async {
            posted.await.map_err(|_| Error::Disconnected)?;
            reply.send(vec![1, 2, 3]).map_err(|_| Error::Disconnected)
        };
        let result = bounded(
            &mut cancellation,
            Instant::now() + Duration::from_secs(1),
            prearmed(read, write),
        )
        .await?;
        assert_eq!(result, [1, 2, 3]);
        Ok(())
    }

    #[tokio::test]
    async fn ready_reply_or_read_error_prevents_registration_write() {
        for reply in [Ok(vec![1]), Err(Error::Disconnected)] {
            let mut wrote = false;
            let result = prearmed(std::future::ready(reply), async {
                wrote = true;
                Ok(())
            })
            .await;
            assert!(matches!(
                result,
                Err(Error::Malformed("unsolicited bootstrap reply") | Error::Disconnected)
            ));
            assert!(!wrote);
        }
    }
}
