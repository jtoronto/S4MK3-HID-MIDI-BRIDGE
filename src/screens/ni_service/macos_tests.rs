#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use tokio::sync::watch;

pub const SERIAL: &str = "FIXTURE42";
const TRUE: u32 = 0x7472_7565;
const ACK: u32 = 0x0340_4300;
const QUERY: u32 = 0x0344_7143;
pub const DISPLAY: u32 = 0x0364_7344;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    WrongSerial,
    RejectSecond,
    BlockFrame,
    LoseControl,
}

pub struct Fixture {
    pub bootstrap: String,
    pub device_note: String,
    pub instance_note: String,
    _ports: [Local; 3],
    pub commands: Arc<Mutex<Vec<Vec<u8>>>>,
    observed: std::sync::mpsc::Receiver<[u8; 12]>,
    frame_began: Option<tokio::sync::oneshot::Receiver<()>>,
    release: Option<std::sync::mpsc::Sender<()>>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            release.send(()).ok();
        }
    }
}

#[must_use]
pub fn words(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

/// Read the command word of a checked fixture message.
///
/// # Panics
/// Panics if the message is shorter than a word.
#[must_use]
pub fn command(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(*bytes.first_chunk().expect("command word"))
}

fn ports(request: &str, note: &str) -> Vec<u8> {
    let mut bytes = words(&[TRUE]);
    for name in [request, note] {
        bytes.extend(words(&[u32::try_from(name.len() + 1).unwrap()]));
        bytes.extend(name.bytes());
        bytes.push(0);
    }
    bytes
}

fn notify(name: &str, bytes: &[u8]) {
    let port = Remote::open(name).unwrap();
    port.request(message_id(bytes).unwrap(), bytes, Duration::from_secs(1), 0)
        .unwrap();
}

fn bind_device(device: &str, note: String, mode: Mode) -> Local {
    Local::bind(device, 1024, move |id, bytes| {
        let bytes = bytes.ok()?;
        if message_id(bytes).ok()? != id {
            return None;
        }
        match command(bytes) {
            ACK if bytes.get(16..)?.strip_suffix(&[0])? == note.as_bytes()
                && Remote::open(&note).is_ok() =>
            {
                Some(words(&[TRUE]))
            }
            QUERY => {
                let mut event = words(&[0x0344_4e2b, 0, 0x1720, 10]);
                event.extend(if mode == Mode::WrongSerial {
                    b"FIXTURE43\0"
                } else {
                    b"FIXTURE42\0"
                });
                notify(&note, &event);
                None
            }
            _ => None,
        }
    })
    .unwrap()
}

impl Fixture {
    /// Bind fresh, private ports for this test.
    ///
    /// # Panics
    /// Panics if a fixture port cannot be created.
    #[must_use]
    pub fn start(mode: Mode) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let prefix = format!(
            "s4.fixture.{}.{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let bootstrap = format!("{prefix}.bootstrap");
        let device = format!("{prefix}.d.req");
        let device_note = format!("{prefix}.d.note");
        let instance = format!("{prefix}.i.req");
        let instance_note = format!("{prefix}.i.note");
        let device_ports = ports(&device, &device_note);
        let instance_ports = ports(&instance, &instance_note);
        let boot = Local::bind(&bootstrap, 1024, move |id, bytes| {
            let bytes = bytes.ok()?;
            if message_id(bytes).ok()? != id {
                return None;
            }
            if bytes == words(&[0x0344_7500, 0x1720, 0x4e69_4d32, 0x7072_6d79, 0]) {
                return Some(device_ports.clone());
            }
            let mut expected = words(&[0x0344_4900, 0x1720, 0x4e69_4d32, 0x7072_6d79, 10]);
            expected.extend(b"FIXTURE42\0");
            (bytes == expected).then(|| instance_ports.clone())
        })
        .unwrap();
        let dev = bind_device(&device, device_note.clone(), mode);
        let i_note = instance_note.clone();
        let commands = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
        let observed = Arc::clone(&commands);
        let (scalar, observed_scalar) = std::sync::mpsc::channel();
        let (frame_began, began) = tokio::sync::oneshot::channel();
        let frame_began = Mutex::new(Some(frame_began));
        let (release, unblock) = std::sync::mpsc::channel();
        let unblock = Mutex::new(unblock);
        let fixture_instance = Local::bind(&instance, 200_000, move |id, bytes| {
            let bytes = bytes.ok()?;
            if message_id(bytes).ok()? != id {
                return None;
            }
            observed.lock().unwrap().push(bytes.to_vec());
            if let Ok(bytes) = <[u8; 12]>::try_from(bytes)
                && let Err(error) = scalar.send(bytes)
            {
                tracing::debug!(%error, "fixture scalar observer closed");
            }
            match command(bytes) {
                ACK if bytes.get(16..)?.strip_suffix(&[0])? == i_note.as_bytes()
                    && Remote::open(&i_note).is_ok() =>
                {
                    Some(words(&[TRUE]))
                }
                0x0343_4300 if bytes == words(&[0x0343_4300, 0x7374_7274]) => {
                    notify(&i_note, &words(&[0x0343_4e00, TRUE]));
                    None
                }
                DISPLAY => {
                    if mode == Mode::BlockFrame {
                        frame_began.lock().unwrap().take()?.send(()).ok()?;
                        unblock
                            .lock()
                            .unwrap()
                            .recv_timeout(Duration::from_secs(2))
                            .ok()?;
                    }
                    if mode == Mode::LoseControl && bytes.get(4) == Some(&0) {
                        notify(&i_note, &words(&[0x0343_4e00, 0]));
                    }
                    Some(words(&[
                        if mode == Mode::RejectSecond && bytes.get(4) == Some(&1) {
                            0
                        } else {
                            TRUE
                        },
                    ]))
                }
                // Prove scalar reply cannot be mistaken for the next frame's.
                _ => Some(words(&[0])),
            }
        })
        .unwrap();
        Self {
            bootstrap,
            device_note,
            instance_note,
            _ports: [boot, dev, fixture_instance],
            commands,
            observed: observed_scalar,
            frame_began: Some(began),
            release: Some(release),
        }
    }

    /// Wait on service-side delivery of a scalar command, not the sender's receipt.
    ///
    /// # Errors
    /// Returns an error if the service never observed the command.
    pub fn await_scalar(&self, expected: &[u8], timeout: Duration) -> anyhow::Result<()> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if self
                .observed
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))?
                == expected
            {
                return Ok(());
            }
        }
    }

    fn connect(
        &self,
        cancel: Cancellation,
    ) -> impl Future<Output = Result<NiService, Error>> + Send + 'static {
        let connector = Macos(self.bootstrap.clone());
        async move {
            Session::connect(&connector, Identity::selected_hid(SERIAL)?, cancel)
                .await
                .map(NiService)
        }
    }
}

#[tokio::test]
async fn native_screens_and_decks_share_the_acquired_instance() {
    let fixture = Fixture::start(Mode::Normal);
    let (_keep, cancel) = watch::channel(false);
    let mut session = fixture.connect(cancel).await.unwrap();
    let pixels: Vec<_> = (0..protocol::PIXELS)
        .map(|i| if i % 3 == 0 { 0xf800 } else { 0x001f })
        .collect();
    let raster = Raster::presenter(&pixels).unwrap();
    session.publish_pair([raster; 2]).await.unwrap();
    let desired = [DeckOutput {
        turntable: true,
        playing: true,
        ring: Some(11),
    }; 2];
    let applied = session
        .set_deck_output_guarded(desired, |side| side == 1)
        .await
        .unwrap();
    assert!(!applied[0].playing && applied[1].playing);
    session.set_jog_tension(244).await.unwrap();
    session.publish_pair([raster; 2]).await.unwrap();
    session.quiesce_decks().await.unwrap();
    session.close();
    assert!(matches!(
        session.poll_control().await,
        Err(Error::Disconnected)
    ));
    assert!(Remote::open(&fixture.device_note).is_err());
    assert!(Remote::open(&fixture.instance_note).is_err());
    let commands = fixture.commands.lock().unwrap();
    let displays: Vec<_> = commands
        .iter()
        .enumerate()
        .filter(|(_, b)| command(b) == DISPLAY)
        .collect();
    assert_eq!(
        displays
            .iter()
            .map(|(_, b)| (b[4], b[22]))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 1), (0, 0), (1, 1)]
    );
    assert!(
        displays
            .iter()
            .all(|(_, b)| b.len() > 65_536 && b[40..44] == [0xf8, 0x00, 0x00, 0x1f])
    );
    let mixed = &commands[displays[1].0 + 1..displays[2].0];
    assert!(mixed.contains(&words(&[0x0356_7375, 0x4a4f_4754, 244])));
    assert!(mixed.contains(&words(&[0x036a_736d, 1, TRUE])));
    assert!(!mixed.contains(&words(&[0x036a_736d, 0, TRUE])));
    drop(commands);
}

#[tokio::test]
async fn native_wrong_serial_never_opens_an_instance() {
    let fixture = Fixture::start(Mode::WrongSerial);
    let (cancel, rx) = watch::channel(false);
    let connect = fixture.connect(rx);
    tokio::pin!(connect);
    // No wrong-serial fallback: the full setup waits for the exact serial.
    assert!(
        tokio::time::timeout(Duration::from_millis(250), &mut connect)
            .await
            .is_err()
    );
    cancel.send_replace(true);
    assert!(matches!(connect.await, Err(Error::Cancelled)));
    assert!(fixture.commands.lock().unwrap().is_empty());
    assert!(Remote::open(&fixture.device_note).is_err());
}

#[tokio::test]
async fn native_rejected_frame_and_control_loss_are_terminal() {
    for (mode, expected, count) in [
        (Mode::RejectSecond, Error::Rejected, 2),
        (Mode::LoseControl, Error::ControlLost, 1),
    ] {
        let fixture = Fixture::start(mode);
        let (_keep, cancel) = watch::channel(false);
        let mut session = fixture.connect(cancel).await.unwrap();
        let pixels = vec![0; protocol::PIXELS];
        let raster = Raster::presenter(&pixels).unwrap();
        let error = session.publish_pair([raster; 2]).await.unwrap_err();
        assert_eq!(
            std::mem::discriminant(&error),
            std::mem::discriminant(&expected)
        );
        assert!(matches!(
            session.poll_control().await,
            Err(Error::Disconnected)
        ));
        assert_eq!(
            fixture
                .commands
                .lock()
                .unwrap()
                .iter()
                .filter(|b| command(b) == DISPLAY)
                .count(),
            count
        );
    }
}

#[tokio::test]
async fn native_cancellation_during_a_display_does_not_wait_for_its_reply() {
    let mut fixture = Fixture::start(Mode::BlockFrame);
    let (cancel, rx) = watch::channel(false);
    let mut session = fixture.connect(rx).await.unwrap();
    let pixels = vec![0; protocol::PIXELS];
    let raster = Raster::presenter(&pixels).unwrap();
    let result = {
        let operation = session.publish_pair([raster; 2]);
        let began = fixture.frame_began.take().unwrap();
        let stop = async {
            began.await.unwrap();
            cancel.send_replace(true);
        };
        tokio::time::timeout(Duration::from_millis(500), async {
            tokio::join!(operation, stop).0
        })
        .await
    };
    fixture.release.take().unwrap().send(()).ok();
    assert!(matches!(result.unwrap(), Err(Error::Cancelled)));
    assert!(matches!(
        session.poll_control().await,
        Err(Error::Disconnected)
    ));
    assert!(Remote::open(&fixture.instance_note).is_err());
}
