use super::{Error, written};
use std::io;
use std::time::Duration;

pub(super) const POLL: Duration = Duration::from_millis(2);

pub(super) async fn message(
    bytes: &[u8],
    mut send: impl FnMut(&[u8]) -> io::Result<usize>,
) -> Result<(), Error> {
    loop {
        let actual = send(bytes)?;
        if actual == 0 && !bytes.is_empty() {
            tokio::time::sleep(POLL).await;
        } else {
            return written(bytes.len(), actual);
        }
    }
}

pub(super) async fn guarded_message(
    bytes: &[u8],
    mut send: impl FnMut(&[u8]) -> io::Result<usize> + Send,
    guard: &(dyn Fn() -> bool + Sync),
) -> Result<bool, Error> {
    loop {
        if !guard() {
            return Ok(false);
        }
        let actual = send(bytes)?;
        if actual == 0 && !bytes.is_empty() {
            tokio::time::sleep(POLL).await;
        } else {
            written(bytes.len(), actual)?;
            return Ok(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::ni_service::{bounded, watch};
    use std::collections::VecDeque;
    use tokio::time::Instant;

    #[tokio::test(start_paused = true)]
    async fn retries_unchanged_message_after_zero_progress() {
        let mut results = VecDeque::from([0, 0, 4]);
        let mut attempts = Vec::new();
        let result = message(b"test", |bytes| {
            attempts.push(bytes.to_vec());
            Ok(results.pop_front().unwrap_or(usize::MAX))
        })
        .await;
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(attempts, vec![b"test".to_vec(); 3]);
    }

    #[tokio::test(start_paused = true)]
    async fn revoked_guard_prevents_retry_after_zero_progress() {
        let allowed = std::sync::atomic::AtomicBool::new(true);
        let mut attempts = 0;
        let result = guarded_message(
            b"play",
            |_| {
                attempts += 1;
                allowed.store(false, std::sync::atomic::Ordering::SeqCst);
                Ok(0)
            },
            &|| allowed.load(std::sync::atomic::Ordering::SeqCst),
        )
        .await;
        assert!(matches!(result, Ok(false)));
        assert_eq!(attempts, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn partial_error_and_empty_ack_are_not_retried() {
        for case in 0..3 {
            let mut attempts = 0;
            let bytes = if case == 2 { &b""[..] } else { &b"test"[..] };
            let result = message(bytes, |_| {
                attempts += 1;
                match case {
                    0 => Ok(2),
                    1 => Err(io::Error::from(io::ErrorKind::BrokenPipe)),
                    _ => Ok(0),
                }
            })
            .await;
            match case {
                0 => assert!(matches!(
                    result,
                    Err(Error::ShortWrite {
                        expected: 4,
                        actual: 2
                    })
                )),
                1 => assert!(matches!(result, Err(Error::Disconnected))),
                _ => assert!(result.is_ok()),
            }
            assert_eq!(attempts, 1);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn stalled_write_obeys_original_deadline_and_cancellation() {
        for cancel_first in [false, true] {
            let (cancel, mut receiver) = watch::channel(false);
            let mut attempts = 0;
            let deadline = Instant::now() + POLL * 3;
            let operation = message(b"test", |_| {
                attempts += 1;
                if cancel_first {
                    cancel.send(true).map_err(io::Error::other)?;
                }
                Ok(0)
            });
            let result = bounded(&mut receiver, deadline, operation).await;
            if cancel_first {
                assert!(matches!(result, Err(Error::Cancelled)));
                assert_eq!(attempts, 1);
            } else {
                assert!(matches!(result, Err(Error::Timeout)));
                assert_eq!(attempts, 4);
            }
            tokio::time::advance(POLL * 2).await;
            assert_eq!(attempts, if cancel_first { 1 } else { 4 });
        }
    }
}
