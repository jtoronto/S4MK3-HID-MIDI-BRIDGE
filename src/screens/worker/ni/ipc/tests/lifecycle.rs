//! Real worker requests exercise the synchronous adapter and subprocess boundary.

use super::*;

fn pages(active: bool) -> [page::Page; 2] {
    [page::Page::Stems {
        shifted: false,
        fx: page::FxWarning::default(),
        backspin: None,
        pads: [page::PadState {
            active: Some(active),
            held: false,
        }; 8],
    }; 2]
}

fn start(
    fixture: &Fixture,
    address: std::net::SocketAddr,
    first_mode: &'static str,
) -> anyhow::Result<(Worker, tokio::sync::mpsc::Receiver<(u64, State)>)> {
    let executable = fixture.executable();
    let (status, statuses) = tokio::sync::mpsc::channel(16);
    let mut first = true;
    let worker = Worker::start_ni(
        move |identity, cancellation| {
            assert_eq!(identity.serial(), "18F73BA2");
            let mut command = Command::new(&executable);
            command
                .arg(address.to_string())
                .arg(if first { first_mode } else { "normal" });
            first = false;
            Supervisor::spawn(command, cancellation, Limits::default())
        },
        false,
        move |id, state| assert!(status.try_send((id, state)).is_ok()),
    )?;
    worker
        .control()
        .bind_identity(Identity::selected_hid("18F73BA2")?)?;
    Ok((worker, statuses))
}

async fn status(
    statuses: &mut tokio::sync::mpsc::Receiver<(u64, State)>,
    expected: (u64, State),
) -> anyhow::Result<()> {
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(3), statuses.recv()).await?,
        Some(expected)
    );
    Ok(())
}

async fn accept(listener: &TcpListener) -> anyhow::Result<TcpStream> {
    Ok(
        tokio::time::timeout(Duration::from_secs(3), listener.accept())
            .await??
            .0,
    )
}

async fn pair(peer: &mut TcpStream, pages: [page::Page; 2]) -> anyhow::Result<()> {
    marker(peer, 1).await?;
    let mut payload = vec![0; 307_200];
    tokio::time::timeout(Duration::from_secs(3), peer.read_exact(&mut payload)).await??;
    let expected: Vec<u8> = pages
        .into_iter()
        .flat_map(|page| {
            let mut frame = Frame::default();
            page::render(&page, &mut frame);
            frame
                .pixels
                .into_iter()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(payload, expected);
    Ok(())
}

#[tokio::test]
async fn worker_requires_ack_coalesces_pages_and_retries_latest_state() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let (mut worker, mut statuses) = start(&fixture, listener.local_addr()?, "normal")?;
    worker.publish(pages(false));
    let control = worker.control();
    control.request(51, Action::On);
    status(&mut statuses, (51, State::Pending)).await?;
    let mut peer = accept(&listener).await?;
    pair(&mut peer, pages(false)).await?;
    assert!(statuses.try_recv().is_err(), "READY is not Active");
    worker.publish(pages(true));
    let latest = [page::Page::Empty, pages(true)[1]];
    worker.publish(latest);
    peer.write_all(b"A").await?;
    status(&mut statuses, (51, State::Active)).await?;
    pair(&mut peer, latest).await?;
    // Cancel while this ACK is deliberately withheld. This also proves OFF
    // does not append STOP to an incomplete transaction.
    control.request(52, Action::Off);
    status(&mut statuses, (52, State::Failed)).await?;
    worker.publish(pages(true));
    control.request(53, Action::Retry);
    status(&mut statuses, (53, State::Pending)).await?;
    let mut peer = accept(&listener).await?;
    pair(&mut peer, pages(true)).await?;
    assert!(statuses.try_recv().is_err());
    peer.write_all(b"A").await?;
    status(&mut statuses, (53, State::Active)).await?;
    drop(worker);
    marker(&mut peer, 2).await?;
    Ok(())
}

#[tokio::test]
async fn forced_release_keeps_off_failed_until_clean_retry_release() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let (mut worker, mut statuses) = start(&fixture, listener.local_addr()?, "blocked")?;
    worker.publish(pages(false));
    let control = worker.control();
    control.request(61, Action::On);
    status(&mut statuses, (61, State::Pending)).await?;
    let mut peer = accept(&listener).await?;
    marker(&mut peer, 3).await?;
    worker.publish(pages(true));
    control.request(62, Action::Retry);
    status(&mut statuses, (62, State::Failed)).await?;
    assert!(
        statuses.try_recv().is_err(),
        "stale generation became active"
    );
    control.request(63, Action::Off);
    status(&mut statuses, (63, State::Failed)).await?;
    control.request(64, Action::Retry);
    status(&mut statuses, (64, State::Pending)).await?;
    let mut peer = accept(&listener).await?;
    pair(&mut peer, pages(true)).await?;
    peer.write_all(b"A").await?;
    status(&mut statuses, (64, State::Active)).await?;
    control.request(65, Action::Off);
    marker(&mut peer, 2).await?;
    status(&mut statuses, (65, State::Inactive)).await?;
    drop(worker);
    Ok(())
}

#[tokio::test]
async fn retry_superseding_a_failed_stop_reports_its_own_failure() -> anyhow::Result<()> {
    let fixture = Fixture::build().await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let (mut worker, mut statuses) = start(&fixture, listener.local_addr()?, "hung-stop")?;
    worker.publish(pages(false));
    let control = worker.control();
    control.request(71, Action::On);
    status(&mut statuses, (71, State::Pending)).await?;
    let mut peer = accept(&listener).await?;
    pair(&mut peer, pages(false)).await?;
    peer.write_all(b"A").await?;
    status(&mut statuses, (71, State::Active)).await?;
    control.request(72, Action::Off);
    marker(&mut peer, 2).await?;
    control.request(73, Action::Retry);
    status(&mut statuses, (73, State::Failed)).await?;
    assert!(statuses.try_recv().is_err());
    control.request(74, Action::Retry);
    status(&mut statuses, (74, State::Pending)).await?;
    let mut peer = accept(&listener).await?;
    pair(&mut peer, pages(false)).await?;
    peer.write_all(b"A").await?;
    status(&mut statuses, (74, State::Active)).await?;
    drop(worker);
    marker(&mut peer, 2).await?;
    Ok(())
}
