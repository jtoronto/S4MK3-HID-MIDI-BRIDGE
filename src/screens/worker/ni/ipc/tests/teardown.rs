//! A killed fixture may close its observation socket with EOF or TCP reset.
//! This is never used for protocol reads or acknowledged clean shutdown.

use std::io::ErrorKind;

use tokio::io::{AsyncRead, AsyncReadExt as _, ReadBuf};

use super::*;

pub(super) async fn forced_peer_closed(
    reader: &mut (impl AsyncRead + Unpin),
) -> anyhow::Result<()> {
    let mut tail = Vec::new();
    match tokio::time::timeout(Duration::from_secs(3), reader.read_to_end(&mut tail)).await? {
        Ok(_) => {}
        // Windows reports WSAECONNRESET when this test deliberately kills the
        // subprocess. Keep bytes already read: a reset cannot hide extra data.
        Err(error) if error.kind() == ErrorKind::ConnectionReset => {}
        Err(error) => return Err(error.into()),
    }
    anyhow::ensure!(
        tail.is_empty(),
        "forced fixture close contained unexpected data; no shutdown record is expected"
    );
    Ok(())
}

struct Fault(ErrorKind);

impl AsyncRead for Fault {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        _: &mut ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Err(self.0.into()))
    }
}

#[tokio::test]
async fn forced_close_preserves_data_and_errors_when_accepting_reset() -> anyhow::Result<()> {
    for (bytes, fault, succeeds) in [
        (b"".as_slice(), ErrorKind::ConnectionReset, true),
        (b"extra".as_slice(), ErrorKind::ConnectionReset, false),
        (b"".as_slice(), ErrorKind::PermissionDenied, false),
    ] {
        let mut reader = std::io::Cursor::new(bytes).chain(Fault(fault));
        assert_eq!(forced_peer_closed(&mut reader).await.is_ok(), succeeds);
    }
    forced_peer_closed(&mut std::io::Cursor::new(b"")).await?;
    Ok(())
}
