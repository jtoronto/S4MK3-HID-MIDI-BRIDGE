use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};

use super::*;

mod lifecycle;
#[cfg(windows)]
mod native;
mod teardown;

struct Fixture(PathBuf);

impl Fixture {
    async fn build() -> anyhow::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "s4-screen-ipc-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        let fixture = Self(path);
        #[cfg(windows)]
        {
            let destination = fixture.executable();
            tokio::task::spawn_blocking(move || {
                std::fs::copy(
                    std::env::current_exe()?.with_file_name("s4-screen-ipc-fixture.exe"),
                    destination,
                )
                .context(
                    "required packaged s4-screen-ipc-fixture.exe beside the native test executable",
                )
            })
            .await??;
        }
        #[cfg(not(windows))]
        fixture.compile().await?;
        Ok(fixture)
    }

    #[cfg(not(windows))]
    async fn compile(&self) -> anyhow::Result<()> {
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/screens/worker/ni/ipc/fixture.rs");
        let output = Command::new("rustc")
            .arg("--edition=2024")
            .arg(source)
            .arg("-o")
            .arg(self.executable())
            .kill_on_drop(true)
            .output()
            .await?;
        anyhow::ensure!(
            output.status.success(),
            "compile IPC fixture: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    fn executable(&self) -> PathBuf {
        self.0.join(if cfg!(windows) {
            "supervisor.exe"
        } else {
            "supervisor"
        })
    }

    async fn connect(
        &self,
        mode: &str,
        cancellation: Cancellation,
    ) -> anyhow::Result<(Session, TcpStream)> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let mut command = Command::new(self.executable());
        command.arg(listener.local_addr()?.to_string()).arg(mode);
        let session = Session::connect(
            command,
            cancellation,
            Limits {
                stop: Duration::from_millis(100),
                ..Limits::default()
            },
        )
        .await?;
        let (stream, _) = tokio::time::timeout(Duration::from_secs(3), listener.accept()).await??;
        Ok((session, stream))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!("IPC fixture cleanup failed: {error}");
        }
    }
}

async fn marker(stream: &mut TcpStream, expected: u8) -> anyhow::Result<()> {
    let actual = tokio::time::timeout(Duration::from_secs(3), stream.read_u8()).await??;
    assert_eq!(actual, expected);
    Ok(())
}

fn frames() -> [Frame; 2] {
    let mut frames = [Frame::default(), Frame::default()];
    for (side, frame) in frames.iter_mut().enumerate() {
        for (index, pixel) in frame.pixels.iter_mut().enumerate() {
            *pixel = u16::try_from(index % 65_536).unwrap_or_default()
                ^ if side == 0 { 0x1234 } else { 0xabcd };
        }
    }
    frames
}

#[tokio::test]
async fn setup_cancellation_terminates_peer_before_returning_unconfirmed() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let mut command = Command::new(fixture.executable());
    command.arg(listener.local_addr()?.to_string()).arg("setup");
    let (cancel, cancellation) = watch::channel(false);
    let cancel_setup = async {
        let (mut peer, _) = listener.accept().await?;
        marker(&mut peer, 4).await?;
        cancel.send_replace(true);
        teardown::forced_peer_closed(&mut peer).await?;
        Ok::<(), anyhow::Error>(())
    };
    let connecting = tokio::task::spawn_blocking(move || {
        Supervisor::spawn(command, cancellation, Limits::default())
    });
    let (connected, cancelled) = tokio::join!(
        tokio::time::timeout(Duration::from_secs(3), connecting),
        cancel_setup
    );
    cancelled?;
    assert!(connected??.is_err_and(|error| error.is::<Unconfirmed>()));
    Ok(())
}

#[tokio::test]
async fn real_pair_is_little_endian_and_waits_for_ack_before_stop() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    let (_sender, cancellation) = watch::channel(false);
    let (mut session, mut peer) = fixture.connect("normal", cancellation).await?;
    let frames = frames();
    {
        let publication = session.publish(&frames);
        tokio::pin!(publication);
        let transfer = async {
            marker(&mut peer, 1).await?;
            let mut payload = vec![0; 307_200];
            peer.read_exact(&mut payload).await?;
            for (side, frame) in payload.chunks_exact(153_600).enumerate() {
                for (index, pixel) in frame.chunks_exact(2).enumerate() {
                    let expected =
                        u16::try_from(index % 65_536)? ^ if side == 0 { 0x1234 } else { 0xabcd };
                    assert_eq!(pixel, expected.to_le_bytes());
                }
            }
            Ok::<(), anyhow::Error>(())
        };
        tokio::select! {
            biased;
            result = &mut publication => anyhow::bail!("pair completed before FRAME_OK: {result:?}"),
            result = transfer => result?,
        }
        assert!(
            std::future::poll_fn(|context| std::task::Poll::Ready(
                publication.as_mut().poll(context).is_pending()
            ))
            .await,
            "publication completed with its ACK withheld"
        );
        peer.write_all(b"A").await?;
        publication.await?;
    }
    session.close().await?;
    marker(&mut peer, 2).await?;
    assert!(session.process.exited()?.is_some());
    Ok(())
}

#[tokio::test]
async fn blocked_stdin_cancellation_kills_and_reaps_without_stop_record() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    let (cancel, cancellation) = watch::channel(false);
    let (mut session, mut peer) = fixture.connect("blocked", cancellation).await?;
    let frames = frames();
    let cancellation = async {
        marker(&mut peer, 3).await?;
        cancel.send_replace(true);
        Ok::<(), anyhow::Error>(())
    };
    let (sent, cancelled) = tokio::join!(session.publish(&frames), cancellation);
    assert!(sent.is_err());
    cancelled?;
    assert!(
        session
            .close()
            .await
            .is_err_and(|error| error.is::<Unconfirmed>())
    );
    assert!(session.process.exited()?.is_some());
    teardown::forced_peer_closed(&mut peer).await?;
    Ok(())
}

#[tokio::test]
async fn rejected_malformed_and_partial_acks_poison_real_process() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    for reply in [b'W', b'E', b'L', b'P'] {
        let (_sender, cancellation) = watch::channel(false);
        let (mut session, mut peer) = fixture.connect("normal", cancellation).await?;
        let frames = frames();
        let respond = async {
            marker(&mut peer, 1).await?;
            let mut payload = vec![0; 307_200];
            peer.read_exact(&mut payload).await?;
            peer.write_all(&[reply]).await?;
            Ok::<(), anyhow::Error>(())
        };
        let (sent, replied) = tokio::join!(session.publish(&frames), respond);
        assert!(sent.is_err(), "reply {reply} must fail publication");
        replied?;
        assert!(
            session
                .close()
                .await
                .is_err_and(|error| error.is::<Unconfirmed>())
        );
        assert!(session.process.exited()?.is_some());
    }
    Ok(())
}

#[tokio::test]
async fn unresponsive_stop_reports_unconfirmed_and_reaps_process() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    let (_sender, cancellation) = watch::channel(false);
    let (mut session, mut peer) = fixture.connect("hung-stop", cancellation).await?;
    let (closed, observed) = tokio::join!(session.close(), marker(&mut peer, 2));
    observed?;
    assert!(closed.is_err_and(|error| error.is::<Unconfirmed>()));
    assert!(session.process.exited()?.is_some());
    Ok(())
}

#[tokio::test]
async fn missing_helper_fails_without_a_screen_process() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    let (_sender, cancellation) = watch::channel(false);
    let result = Session::connect(
        Command::new(fixture.0.join("absent-helper")),
        cancellation,
        Limits::default(),
    )
    .await;
    assert!(result.is_err_and(|error| !error.is::<Unconfirmed>()));
    Ok(())
}
