//! Native pipe contracts; these tests never contact NI or a controller.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::timeout;

const DEADLINE: Duration = Duration::from_secs(2);
static NEXT_PIPE: AtomicU64 = AtomicU64::new(0);

async fn pair(buffer_size: u32) -> anyhow::Result<(Owned, Owned)> {
    let name = format!(
        "s4-ni-test-{}-{}",
        std::process::id(),
        NEXT_PIPE.fetch_add(1, Ordering::Relaxed)
    );
    let listener = PipeListenerOptions::new()
        .path(path(&name))
        .mode(PipeMode::Messages)
        .nonblocking(true)
        .accept_remote(false)
        .input_buffer_size_hint(buffer_size)
        .output_buffer_size_hint(buffer_size)
        .create_duplex::<pipe_mode::Messages>()?;
    let client = timeout(DEADLINE, Windows.open(&name)).await??;
    let server = listener.accept()?;
    Ok((client, Owned(Some(server))))
}

async fn read_message(peer: &mut Owned) -> anyhow::Result<Vec<u8>> {
    Ok(timeout(DEADLINE, peer.read()).await??)
}

#[tokio::test]
async fn pressure_retries_one_intact_message_after_peer_drains() -> anyhow::Result<()> {
    let (mut writer, mut peer) = pair(4096).await?;
    let filler = vec![0x5a; 4096];
    let mut queued = 0;
    loop {
        let count = writer.stream()?.send(&filler)?;
        if count == 0 {
            break;
        }
        assert_eq!(count, filler.len());
        queued += 1;
        anyhow::ensure!(queued < 1024, "local pipe never reached pressure");
    }
    let (signal, ready) = tokio::sync::oneshot::channel();
    let mut signal = Some(signal);
    let message = b"one intact message";
    let send = write::message(message, |bytes| {
        let count = writer.stream().map_err(io::Error::other)?.send(bytes)?;
        if count == 0 {
            if let Some(signal) = signal.take() {
                signal
                    .send(())
                    .map_err(|()| io::Error::other("peer closed"))?;
            }
        }
        Ok(count)
    });
    let drain = async {
        ready.await?;
        for _ in 0..queued {
            assert_eq!(read_message(&mut peer).await?, filler);
        }
        assert_eq!(read_message(&mut peer).await?, message);
        assert_eq!(peer.stream()?.peek_msg_len()?, 0);
        anyhow::Ok(())
    };
    let (write_result, peer_result) =
        timeout(DEADLINE, async { tokio::join!(send, drain) }).await?;
    write_result?;
    peer_result?;
    Ok(())
}

#[tokio::test]
async fn client_preserves_message_boundaries_and_disconnect_errors() -> anyhow::Result<()> {
    let (mut client, mut peer) = pair(4096).await?;
    timeout(DEADLINE, peer.write(b"first")).await??;
    timeout(DEADLINE, peer.write(b"second")).await??;
    assert_eq!(read_message(&mut client).await?, b"first");
    assert_eq!(read_message(&mut client).await?, b"second");
    drop(peer);
    assert!(timeout(DEADLINE, client.write(b"closed")).await?.is_err());
    Ok(())
}

#[tokio::test]
async fn empty_ack_count_does_not_establish_peek_delivery() -> anyhow::Result<()> {
    let (mut client, mut peer) = pair(4096).await?;
    assert_eq!(peer.stream()?.send(&[])?, 0);
    assert_eq!(client.stream()?.peek_msg_len()?, 0);
    // The production peek reader cannot distinguish this from no message.
    assert!(
        timeout(Duration::from_millis(50), client.read())
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn frame_write_respects_peer_capacity_and_original_deadline() -> anyhow::Result<()> {
    let frame = vec![0x3c; protocol::DISPLAY_BYTES];
    for capacity in [4096, 262_144] {
        let (mut writer, mut peer) = pair(capacity).await?;
        let result = timeout(Duration::from_millis(100), writer.write(&frame)).await;
        if capacity == 4096 {
            assert!(result.is_err(), "oversized message unexpectedly fit");
            assert_eq!(peer.stream()?.peek_msg_len()?, 0);
        } else {
            result??;
            let stream = peer.stream()?;
            assert_eq!(stream.peek_msg_len()?, frame.len());
            let mut buffer = MsgBuf::from(Vec::with_capacity(frame.len()));
            assert!(matches!(
                stream.recv_msg(&mut buffer, None)?,
                RecvResult::Fit
            ));
            assert_eq!(buffer.filled_part(), frame);
            assert_eq!(stream.peek_msg_len()?, 0);
        }
    }
    Ok(())
}
