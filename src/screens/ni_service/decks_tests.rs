use super::{
    ACTIVE, Error, LOST, Pipe, Raster, Session, TestResult, YES, fixture, identity, protocol, watch,
};
use crate::screens::ni_service::DeckOutput;
use anyhow::Context;

fn packet(command: u32, side: u32, value: u32) -> Vec<u8> {
    protocol::words(&[command, side, value])
}

fn drain(peer: &mut super::Wire) -> Vec<Vec<u8>> {
    let mut messages = Vec::new();
    while let Ok(message) = peer.rx.try_recv() {
        messages.push(message);
    }
    messages
}

#[tokio::test]
async fn deck_updates_are_safe_independent_and_preserve_screen_replies() -> TestResult {
    let (bus, service) = fixture();
    let (_cancel, rx) = watch::channel(false);
    let (session, peer) = tokio::join!(
        Session::connect(&bus, identity()?, rx),
        service.handshake("S4ABC123", Some(&ACTIVE)),
    );
    let mut session = session?;
    let mut peer = peer?;
    let off = DeckOutput::default();
    let playing = DeckOutput {
        turntable: true,
        playing: true,
        ring: Some(46),
    };
    session.set_deck_output([playing, off]).await?;
    let initial = drain(&mut peer.instance);
    assert_eq!(
        &initial[..2],
        &[packet(0x036a_736d, 0, 0), packet(0x036a_736d, 1, 0)]
    );
    assert!(initial.contains(&packet(0x036a_7353, 0, 0x4205_5555)));
    assert!(initial.contains(&packet(0x036a_7353, 1, 0x4205_5555)));
    let ticks = initial
        .iter()
        .filter(|p| p.starts_with(&0x036a_7378_u32.to_le_bytes()));
    assert_eq!(
        ticks.cloned().collect::<Vec<_>>(),
        [
            packet(0x036a_7378, 0, 0),
            packet(0x036a_7378, 1, 0),
            packet(0x036a_7378, 0, 0),
        ],
        "acquisition clears the NI rim detent setting on both wheels before enabling playback"
    );
    assert_eq!(initial.last(), Some(&packet(0x036a_736d, 0, 0x7472_7565)));
    assert!(!initial.contains(&packet(0x036a_736d, 1, 0x7472_7565)));
    session.set_deck_output([playing, off]).await?;
    assert!(drain(&mut peer.instance).is_empty());
    session.set_deck_output([playing, playing]).await?;
    let right = drain(&mut peer.instance);
    assert!(
        right
            .iter()
            .all(|message| message[4..8] == 1_u32.to_le_bytes())
    );
    assert_eq!(right.first(), Some(&packet(0x036a_736d, 1, 0)));
    assert_eq!(right.last(), Some(&packet(0x036a_736d, 1, 0x7472_7565)));
    let pixels = vec![0; protocol::PIXELS];
    let raster = Raster::new(&pixels)?;
    let (result, peer_result) = tokio::join!(session.publish_pair([raster, raster]), async {
        for side in 0..2 {
            let frame = peer.instance.read().await?;
            assert_eq!(frame.len(), 52);
            assert_eq!(frame[..4], [0x44, 0x73, 0x64, 0x03]);
            assert_eq!(frame[4], side);
            assert_eq!(frame[16..20], [32, 0, 0, 0]);
            assert_eq!(frame[22], side);
            peer.instance.write(&YES).await?;
        }
        Ok::<_, Error>(())
    });
    result?;
    peer_result?;
    session.quiesce_decks().await?;
    let shutdown = drain(&mut peer.instance);
    assert_eq!(
        &shutdown[..2],
        &[packet(0x036a_736d, 0, 0), packet(0x036a_736d, 1, 0)]
    );
    for side in 0..2 {
        assert!(shutdown.contains(&packet(0x036a_734c, side, 0)));
        assert!(shutdown.contains(&packet(0x036a_734d, side, 1)));
        assert!(shutdown.contains(&packet(0x036a_7378, side, 0)));
    }
    session
        .set_deck_output([
            DeckOutput {
                playing: true,
                ..off
            },
            off,
        ])
        .await?;
    assert_eq!(
        drain(&mut peer.instance),
        vec![packet(0x036a_736d, 0, 0x7472_7565)]
    );
    Ok(())
}

#[tokio::test]
async fn control_loss_reports_unconfirmed_shutdown_and_closes_session() -> TestResult {
    let (bus, service) = fixture();
    let (_cancel, rx) = watch::channel(false);
    let (session, peer) = tokio::join!(
        Session::connect(&bus, identity()?, rx),
        service.handshake("S4ABC123", Some(&ACTIVE)),
    );
    let mut session = session?;
    let mut peer = peer?;
    session.set_deck_output([DeckOutput::default(); 2]).await?;
    drain(&mut peer.instance);
    peer.instance_notes.write(&LOST).await?;
    assert!(matches!(
        session.quiesce_decks().await,
        Err(Error::ControlLost)
    ));
    assert!(drain(&mut peer.instance).is_empty());
    assert!(matches!(
        session.quiesce_decks().await,
        Err(Error::Disconnected)
    ));
    Ok(())
}

#[tokio::test]
async fn guard_rejection_keeps_session_usable_and_retries_unapplied_side() -> TestResult {
    let (bus, service) = fixture();
    let (_cancel, rx) = watch::channel(false);
    let (session, peer) = tokio::join!(
        Session::connect(&bus, identity()?, rx),
        service.handshake("S4ABC123", Some(&ACTIVE)),
    );
    let mut session = session?;
    let mut peer = peer?;
    let playing = DeckOutput {
        turntable: true,
        playing: true,
        ring: Some(46),
    };
    let applied = session
        .set_deck_output_guarded([playing; 2], |side| side == 1)
        .await?;
    assert!(!applied[0].playing);
    assert!(applied[1].playing);
    let requests = drain(&mut peer.instance);
    assert!(!requests.contains(&packet(0x036a_736d, 0, 0x7472_7565)));
    assert!(requests.contains(&packet(0x036a_736d, 1, 0x7472_7565)));
    let applied = session
        .set_deck_output_guarded([playing; 2], |_| true)
        .await?;
    assert_eq!(applied, [playing; 2]);
    assert_eq!(
        drain(&mut peer.instance),
        vec![packet(0x036a_736d, 0, 0x7472_7565)]
    );
    Ok(())
}

#[tokio::test]
async fn both_pauses_precede_any_configuration_on_transition_to_jog() -> TestResult {
    let (bus, service) = fixture();
    let (_cancel, rx) = watch::channel(false);
    let (session, peer) = tokio::join!(
        Session::connect(&bus, identity()?, rx),
        service.handshake("S4ABC123", Some(&ACTIVE)),
    );
    let mut session = session?;
    let mut peer = peer?;
    let playing = DeckOutput {
        turntable: true,
        playing: true,
        ring: Some(46),
    };
    session.set_deck_output([playing; 2]).await?;
    drain(&mut peer.instance);
    session.set_deck_output([DeckOutput::default(); 2]).await?;
    let requests = drain(&mut peer.instance);
    assert_eq!(
        &requests[..2],
        &[packet(0x036a_736d, 0, 0), packet(0x036a_736d, 1, 0)]
    );
    Ok(())
}

#[tokio::test]
async fn playing_mode_changes_pause_both_sides_before_reconfiguration() -> TestResult {
    let (bus, service) = fixture();
    let (_cancel, rx) = watch::channel(false);
    let (session, peer) = tokio::join!(
        Session::connect(&bus, identity()?, rx),
        service.handshake("S4ABC123", Some(&ACTIVE)),
    );
    let mut session = session?;
    let mut peer = peer?;
    let jog = DeckOutput {
        playing: true,
        ..DeckOutput::default()
    };
    let tt = DeckOutput {
        turntable: true,
        ..jog
    };
    session.set_deck_output([jog; 2]).await?;
    let initial = drain(&mut peer.instance);
    for side in 0..2 {
        assert!(initial.contains(&packet(0x036a_736d, side, 0x7472_7565)));
    }
    for (mode, output) in [(2, tt), (1, jog)] {
        session.set_deck_output([output; 2]).await?;
        let requests = drain(&mut peer.instance);
        assert_eq!(
            &requests[..2],
            &[packet(0x036a_736d, 0, 0), packet(0x036a_736d, 1, 0)]
        );
        for side in 0..2 {
            let configured = requests
                .iter()
                .position(|p| *p == packet(0x036a_734d, side, mode))
                .with_context(|| format!("missing mode {mode} on side {side}"))?;
            let played = requests
                .iter()
                .position(|p| *p == packet(0x036a_736d, side, 0x7472_7565))
                .with_context(|| format!("missing Play on side {side}"))?;
            assert!(configured < played);
        }
    }
    session.set_deck_output([DeckOutput::default(); 2]).await?;
    assert_eq!(
        drain(&mut peer.instance),
        vec![packet(0x036a_736d, 0, 0), packet(0x036a_736d, 1, 0)]
    );
    Ok(())
}
