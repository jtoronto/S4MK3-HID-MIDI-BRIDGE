use super::{ACTIVE, Error, LOST, Pipe, Session, TestResult, YES, fixture, identity, watch};
use std::time::Duration;
use tokio::time::{Instant, timeout};

// Wire values from run 032a5c32 and the retained NI ring setter analysis,
// with rim detents intentionally disabled on return to JOG.
const EXPECTED: [[u8; 12]; 15] = [
    [0x6d, 0x73, 0x6a, 3, 0, 0, 0, 0, 0, 0, 0, 0],
    [0x73, 0x73, 0x6a, 3, 0, 0, 0, 0, 0x55, 0x55, 5, 0x42],
    [0x53, 0x73, 0x6a, 3, 0, 0, 0, 0, 0x79, 0x1f, 1, 0x42],
    [0x78, 0x73, 0x6a, 3, 0, 0, 0, 0, 0, 0, 0, 0],
    [0x4d, 0x73, 0x6a, 3, 0, 0, 0, 0, 2, 0, 0, 0],
    [0x54, 0x73, 0x6a, 3, 0, 0, 0, 0, 0x65, 0x75, 0x72, 0x74],
    [0x43, 0x73, 0x6a, 3, 0, 0, 0, 0, 0x2e, 0, 0, 0],
    [0x4c, 0x73, 0x6a, 3, 0, 0, 0, 0, 1, 0, 0, 0],
    [0x6d, 0x73, 0x6a, 3, 0, 0, 0, 0, 0x65, 0x75, 0x72, 0x74],
    [0x6d, 0x73, 0x6a, 3, 0, 0, 0, 0, 0, 0, 0, 0],
    [0x78, 0x73, 0x6a, 3, 0, 0, 0, 0, 0, 0, 0, 0],
    [0x4d, 0x73, 0x6a, 3, 0, 0, 0, 0, 1, 0, 0, 0],
    [0x54, 0x73, 0x6a, 3, 0, 0, 0, 0, 0x65, 0x75, 0x72, 0x74],
    [0x4c, 0x73, 0x6a, 3, 0, 0, 0, 0, 0, 0, 0, 0],
    [0x43, 0x73, 0x6a, 3, 0, 0, 0, 0, 0, 0, 0, 0],
];

#[derive(Clone, Copy, Debug)]
enum DuringPlay {
    LetFinish,
    Interrupt,
    NoFrameReply,
    ControlLost,
}

#[tokio::test(start_paused = true)]
async fn left_only_jog_sequence_preserves_displays_and_bounds_shutdown() -> TestResult {
    // The peer uses the real handshake and never ACKs a jog request. Waiting
    // for a jog reply, consuming a display reply for a jog write, changing a
    // side, or failing to send Pause during a healthy stop breaks the witness.
    for scenario in [
        DuringPlay::LetFinish,
        DuringPlay::Interrupt,
        DuringPlay::NoFrameReply,
        DuringPlay::ControlLost,
    ] {
        let (bus, service) = fixture();
        let (_session_tx, session_rx) = watch::channel(false);
        let (stop_tx, stop_rx) = watch::channel(false);
        let (session, peer) = tokio::join!(
            Session::connect(&bus, identity()?, session_rx),
            service.handshake("S4ABC123", Some(&ACTIVE)),
        );
        let mut session = session?;
        let mut peer = peer?;
        let (client, server) = timeout(Duration::from_secs(11), async {
            tokio::join!(session.run_left_jog_once(stop_rx), async {
                let mut jogs = Vec::new();
                let mut screens = [0_u8; 4];
                let mut play_at = None;
                let mut acted = false;
                let mut previous_left = None;
                while let Ok(packet) = peer.instance.read().await {
                    if packet.len() == 12 {
                        jogs.push(packet);
                        if jogs.len() == 9 {
                            play_at = Some(Instant::now());
                        }
                        if jogs.len() == 10 {
                            let elapsed = Instant::now() - play_at.ok_or("no Play")?;
                            let limit = match scenario {
                                DuringPlay::Interrupt => Duration::from_secs(1),
                                _ => Duration::from_secs(3),
                            };
                            assert!(elapsed <= limit, "Pause late: {elapsed:?}");
                        }
                        continue;
                    }
                    assert_eq!(packet[..4], [0x44, 0x73, 0x64, 3]);
                    let command_bytes =
                        usize::try_from(u32::from_le_bytes(packet[16..20].try_into()?))?;
                    assert_eq!(packet.len(), 20 + command_bytes);
                    let phase = match jogs.len() {
                        3 => 0,
                        8 => 1,
                        9 => 2,
                        13 => 3,
                        count => {
                            return Err(format!("frame during incomplete setup ({count})").into());
                        }
                    };
                    let target = packet[4];
                    assert!(target <= 1);
                    assert_eq!(packet[22], target);
                    assert_eq!(packet[packet.len() - 8..], [3, 0, 0, 0, 0x40, 0, 0, 0]);
                    screens[phase] |= 1 << target;
                    if target == 0 {
                        let pixels = &packet[40..packet.len() - 8];
                        if let Some(previous) = previous_left.replace(pixels.to_vec()) {
                            assert_ne!(pixels, previous, "left frame stopped changing");
                        }
                    }
                    if phase == 2 && !acted {
                        acted = true;
                        match scenario {
                            DuringPlay::LetFinish => {}
                            DuringPlay::Interrupt => {
                                stop_tx.send_replace(true);
                            }
                            DuringPlay::NoFrameReply => continue,
                            DuringPlay::ControlLost => {
                                peer.instance_notes.write(&LOST).await?;
                                continue;
                            }
                        }
                    }
                    peer.instance.write(&YES).await?;
                }
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>((jogs, screens))
            })
        })
        .await?;
        let (jogs, screens) = server?;
        assert_eq!(bus.live.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_trial(scenario, client, &jogs, screens)?;
    }
    Ok(())
}

fn assert_trial(
    scenario: DuringPlay,
    result: Result<(), Error>,
    jogs: &[Vec<u8>],
    screens: [u8; 4],
) -> TestResult {
    match scenario {
        DuringPlay::LetFinish => {
            result?;
            assert_eq!(jogs, EXPECTED);
            assert_eq!(screens, [3, 3, 3, 3]);
        }
        DuringPlay::Interrupt => {
            assert!(matches!(result, Err(Error::Cancelled)));
            assert_eq!(jogs, EXPECTED);
            assert_eq!(screens, [3, 3, 3, 0]);
        }
        DuringPlay::NoFrameReply | DuringPlay::ControlLost => {
            assert!(matches!(
                (scenario, result),
                (DuringPlay::NoFrameReply, Err(Error::Timeout))
                    | (DuringPlay::ControlLost, Err(Error::ControlLost))
            ));
            assert_eq!(jogs, &EXPECTED[..9]);
            assert_eq!(screens, [3, 3, 1, 0]);
        }
    }
    Ok(())
}
