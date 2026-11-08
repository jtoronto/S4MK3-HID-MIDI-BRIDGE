//! Local peers only: no NI, HID, driver or service contact.

use crate::screens::ni_service::{
    Connector, Error, bounded,
    fixtures::CONNECT_REPLY,
    protocol, watch,
    windows::{Windows, path},
};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::net::windows::named_pipe::{NamedPipeServer, PipeMode, ServerOptions};
use tokio::time::{Instant, timeout};

const DEADLINE: Duration = Duration::from_secs(2);
static NEXT_PIPE: AtomicU64 = AtomicU64::new(0);

fn listener() -> io::Result<(String, NamedPipeServer)> {
    let name = format!(
        "s4-ni-bootstrap-test-{}-{}",
        std::process::id(),
        NEXT_PIPE.fetch_add(1, Ordering::Relaxed)
    );
    let server = ServerOptions::new()
        .first_pipe_instance(true)
        .reject_remote_clients(true)
        .pipe_mode(PipeMode::Message)
        .in_buffer_size(4096)
        .out_buffer_size(4096)
        .create(path(&name))?;
    Ok((name, server))
}

async fn read(server: &NamedPipeServer) -> io::Result<Vec<u8>> {
    let mut buffer = vec![0; 1024];
    loop {
        server.readable().await?;
        match server.try_read(&mut buffer) {
            Ok(count) => {
                buffer.truncate(count);
                return Ok(buffer);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error),
        }
    }
}

async fn closed(server: &NamedPipeServer) -> anyhow::Result<()> {
    match read(server).await {
        Ok(message) => anyhow::ensure!(message.is_empty(), "unexpected extra request"),
        Err(error) => anyhow::ensure!(
            matches!(error.raw_os_error(), Some(109 | 232 | 233)),
            "expected peer closure, got {error}"
        ),
    }
    Ok(())
}

async fn registration_prefix(server: &NamedPipeServer) -> anyhow::Result<()> {
    server.connect().await?;
    server.readable().await?;
    let mut prefix = [0; 1];
    assert_eq!(server.try_read(&mut prefix)?, 1);
    assert_eq!(prefix, [0x5a]);
    // This is a wire-level trigger, not proof of Mio's private write state.
    // Leave the rest of the oversized message unread.
    Ok(())
}

async fn closed_without_draining(server: &NamedPipeServer) -> anyhow::Result<()> {
    loop {
        match server.writable().await {
            Ok(()) => {}
            Err(error) if matches!(error.raw_os_error(), Some(109 | 232 | 233)) => return Ok(()),
            Err(error) => return Err(error.into()),
        }
        match server.try_write(&[0]) {
            Ok(1) => {}
            Ok(count) => anyhow::bail!("unexpected probe write length: {count}"),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) if matches!(error.raw_os_error(), Some(109 | 232 | 233)) => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
}

#[tokio::test]
async fn cancelling_backpressured_bootstrap_closes_without_draining_request() -> anyhow::Result<()>
{
    let (name, server) = listener()?;
    let registration = vec![0x5a; 8 * 1024 * 1024];
    let (cancel, mut cancellation) = watch::channel(false);
    timeout(DEADLINE, async {
        {
            let client = bounded(
                &mut cancellation,
                Instant::now() + DEADLINE,
                Windows.bootstrap(&name, &registration),
            );
            tokio::pin!(client);
            tokio::select! {
                biased;
                result = &mut client => anyhow::bail!("unexpected completion: {result:?}"),
                result = registration_prefix(&server) => result?,
            }
            cancel.send(true)?;
            assert!(matches!(client.await, Err(Error::Cancelled)));
        }
        closed_without_draining(&server).await
    })
    .await?
}

#[tokio::test]
async fn backpressured_bootstrap_obeys_original_deadline_and_closes() -> anyhow::Result<()> {
    let (name, server) = listener()?;
    let registration = vec![0x5a; 8 * 1024 * 1024];
    let (_cancel, mut cancellation) = watch::channel(false);
    let deadline = Instant::now() + Duration::from_millis(500);
    timeout(DEADLINE, async {
        {
            let client = bounded(
                &mut cancellation,
                deadline,
                Windows.bootstrap(&name, &registration),
            );
            tokio::pin!(client);
            tokio::select! {
                biased;
                result = &mut client => anyhow::bail!("completion before prefix: {result:?}"),
                result = registration_prefix(&server) => result?,
            }
            assert!(matches!(client.await, Err(Error::Timeout)));
            assert!(Instant::now() >= deadline);
        }
        closed_without_draining(&server).await
    })
    .await?
}

#[tokio::test]
async fn dropping_backpressured_bootstrap_closes_without_draining_request() -> anyhow::Result<()> {
    let (name, server) = listener()?;
    let registration = vec![0x5a; 8 * 1024 * 1024];
    timeout(DEADLINE, async {
        {
            let client = Windows.bootstrap(&name, &registration);
            tokio::pin!(client);
            tokio::select! {
                biased;
                result = &mut client => anyhow::bail!("unexpected completion: {result:?}"),
                result = registration_prefix(&server) => result?,
            }
        }
        closed_without_draining(&server).await
    })
    .await?
}

#[tokio::test]
async fn interrupted_bootstrap_runtime_drop_releases_peer_without_an_extra_drainer()
-> anyhow::Result<()> {
    #[derive(Clone, Copy, Debug)]
    enum Interrupt {
        Cancel,
        Timeout,
        Drop,
    }
    for interrupt in [Interrupt::Cancel, Interrupt::Timeout, Interrupt::Drop] {
        let (name, server) = listener()?;
        let (cancel, mut cancellation) = watch::channel(false);
        let client = tokio::task::spawn_blocking(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()?;
            runtime.block_on(async {
                let registration = vec![0x5a; 8 * 1024 * 1024];
                match interrupt {
                    Interrupt::Drop => {
                        let future = Windows.bootstrap(&name, &registration);
                        tokio::pin!(future);
                        tokio::select! {
                            biased;
                            () = super::super::cancelled(&mut cancellation) => {}
                            result = &mut future => {
                                anyhow::bail!("unexpected bootstrap completion: {result:?}");
                            }
                        }
                    }
                    Interrupt::Cancel | Interrupt::Timeout => {
                        let deadline = Instant::now() + Duration::from_millis(500);
                        let result = bounded(
                            &mut cancellation,
                            deadline,
                            Windows.bootstrap(&name, &registration),
                        )
                        .await;
                        assert!(
                            matches!(
                                (interrupt, result),
                                (Interrupt::Cancel, Err(Error::Cancelled))
                                    | (Interrupt::Timeout, Err(Error::Timeout))
                            ),
                            "wrong bootstrap interruption: {interrupt:?}"
                        );
                    }
                }
                anyhow::Ok(())
            })?;
            // Match Native::open's error path: no extra block_on, poller,
            // completion-port clone or test-only drainer after cancellation.
            drop(runtime);
            anyhow::Ok(())
        });
        timeout(DEADLINE, async {
            registration_prefix(&server).await?;
            if !matches!(interrupt, Interrupt::Timeout) {
                cancel.send(true)?;
            }
            client.await??;
            closed_without_draining(&server).await
        })
        .await??;
    }
    Ok(())
}

#[tokio::test]
async fn bootstrap_retains_full_reply_when_peer_replies_and_closes_immediately()
-> anyhow::Result<()> {
    let (name, server) = listener()?;
    let registration = protocol::device_connect();
    let peer = async {
        server.connect().await?;
        assert_eq!(read(&server).await?, registration);
        loop {
            server.writable().await?;
            match server.try_write(CONNECT_REPLY) {
                Ok(count) => {
                    assert_eq!(count, CONNECT_REPLY.len());
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
        }
        // No flush or receipt from the client: force the immediate-close race.
        server.disconnect()?;
        drop(server);
        anyhow::Ok(())
    };
    let (_cancel, mut cancellation) = watch::channel(false);
    let client = bounded(
        &mut cancellation,
        Instant::now() + DEADLINE,
        Windows.bootstrap(&name, &registration),
    );
    let (reply, peer) = timeout(DEADLINE, async { tokio::join!(client, peer) }).await?;
    peer?;
    let reply = reply?;
    assert_eq!(reply, CONNECT_REPLY);
    let ports = protocol::ports(&reply)?;
    assert_eq!(ports.request, "NIHWS17200045Request");
    assert_eq!(ports.notification, "NIHWS17200045Notification");
    Ok(())
}

#[tokio::test]
async fn stalled_bootstrap_cancellation_releases_client_handle() -> anyhow::Result<()> {
    let (name, server) = listener()?;
    let registration = protocol::device_connect();
    let (cancel, mut cancellation) = watch::channel(false);
    let peer = async {
        server.connect().await?;
        assert_eq!(read(&server).await?, registration);
        cancel.send(true)?;
        closed(&server).await
    };
    let client = bounded(
        &mut cancellation,
        Instant::now() + DEADLINE,
        Windows.bootstrap(&name, &registration),
    );
    let (result, peer) = timeout(DEADLINE, async { tokio::join!(client, peer) }).await?;
    peer?;
    assert!(matches!(result, Err(Error::Cancelled)));
    Ok(())
}

#[tokio::test]
async fn stalled_bootstrap_times_out_and_releases_client_handle() -> anyhow::Result<()> {
    let (name, server) = listener()?;
    let registration = protocol::device_connect();
    let (_cancel, mut cancellation) = watch::channel(false);
    let peer = async {
        server.connect().await?;
        assert_eq!(read(&server).await?, registration);
        closed(&server).await
    };
    let client = bounded(
        &mut cancellation,
        Instant::now() + Duration::from_millis(100),
        Windows.bootstrap(&name, &registration),
    );
    let (result, peer) = timeout(DEADLINE, async { tokio::join!(client, peer) }).await?;
    peer?;
    assert!(matches!(result, Err(Error::Timeout)));
    Ok(())
}

#[tokio::test]
async fn dropping_pending_bootstrap_releases_client_handle() -> anyhow::Result<()> {
    let (name, server) = listener()?;
    let registration = protocol::device_connect();
    timeout(DEADLINE, async {
        {
            let client = Windows.bootstrap(&name, &registration);
            tokio::pin!(client);
            tokio::select! {
                biased;
                result = &mut client => anyhow::bail!("unexpected completion: {result:?}"),
                request = async {
                    server.connect().await?;
                    read(&server).await
                } => assert_eq!(request?, registration),
            }
        }
        closed(&server).await
    })
    .await?
}
