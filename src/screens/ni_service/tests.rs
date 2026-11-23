use super::*;
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::mpsc;

#[path = "decks_tests.rs"]
mod decks_tests;
#[cfg(feature = "attended-ni-jog")]
#[path = "left_jog_tests.rs"]
mod left_jog_tests;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
const YES: [u8; 4] = [0x65, 0x75, 0x72, 0x74];
const ACTIVE: [u8; 8] = [0, 0x4e, 0x43, 3, 0x65, 0x75, 0x72, 0x74];
const LOST: [u8; 8] = [0, 0x4e, 0x43, 3, 0, 0, 0, 0];

struct Wire {
    tx: mpsc::Sender<Vec<u8>>,
    rx: mpsc::Receiver<Vec<u8>>,
    owned: Option<Arc<AtomicUsize>>,
    max_write: Option<usize>,
}

impl Drop for Wire {
    fn drop(&mut self) {
        if let Some(count) = &self.owned {
            count.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

impl Pipe for Wire {
    async fn read(&mut self) -> Result<Vec<u8>, Error> {
        self.rx.recv().await.ok_or(Error::Disconnected)
    }
    async fn write(&mut self, message: &[u8]) -> Result<(), Error> {
        let count = self
            .max_write
            .map_or(message.len(), |limit| message.len().min(limit));
        self.tx
            .send(message[..count].to_vec())
            .await
            .map_err(|_| Error::Disconnected)?;
        written(message.len(), count)
    }
}

fn pair() -> (Wire, Wire) {
    let (atx, arx) = mpsc::channel(64);
    let (btx, brx) = mpsc::channel(64);
    (
        Wire {
            tx: atx,
            rx: brx,
            owned: None,
            max_write: None,
        },
        Wire {
            tx: btx,
            rx: arx,
            owned: None,
            max_write: None,
        },
    )
}

struct Bus {
    endpoints: Mutex<HashMap<String, VecDeque<Wire>>>,
    live: Arc<AtomicUsize>,
}

impl Bus {
    fn take(&self, name: &str) -> Result<Wire, Error> {
        let mut pipe = self
            .endpoints
            .lock()
            .map_err(|_| Error::Disconnected)?
            .get_mut(name)
            .and_then(VecDeque::pop_front)
            .ok_or(Error::Unsupported("missing test endpoint"))?;
        self.live.fetch_add(1, Ordering::SeqCst);
        pipe.owned = Some(Arc::clone(&self.live));
        Ok(pipe)
    }
}

impl Connector for Bus {
    type Request = Wire;
    type Notification = Wire;
    async fn open(&self, name: &str) -> Result<Wire, Error> {
        self.take(name)
    }
    fn serve(&self, name: &str) -> Result<Wire, Error> {
        self.take(name)
    }
}

struct Service {
    bootstrap_device: Wire,
    bootstrap_instance: Wire,
    device: Wire,
    device_notes: Wire,
    instance: Wire,
    instance_notes: Wire,
}

fn fixture() -> (Bus, Service) {
    let mut endpoints: HashMap<String, VecDeque<Wire>> = HashMap::new();
    let mut make = |name: &str| {
        let (client, service) = pair();
        endpoints
            .entry(name.to_owned())
            .or_default()
            .push_back(client);
        service
    };
    let service = Service {
        bootstrap_device: make("NIHWMainHandler"),
        bootstrap_instance: make("NIHWMainHandler"),
        device: make("DeviceRequest"),
        device_notes: make("DeviceNotification"),
        instance: make("InstanceRequest"),
        instance_notes: make("InstanceNotification"),
    };
    (
        Bus {
            endpoints: Mutex::new(endpoints),
            live: Arc::new(AtomicUsize::new(0)),
        },
        service,
    )
}

fn connect_reply(request: &str, notification: &str) -> TestResult<Vec<u8>> {
    let mut reply = YES.to_vec();
    reply.extend(u32::try_from(request.len() + 1)?.to_le_bytes());
    reply.extend(request.bytes().chain([0]));
    reply.extend(u32::try_from(notification.len() + 1)?.to_le_bytes());
    reply.extend(notification.bytes().chain([0]));
    Ok(reply)
}

fn device_note(serial: &str, present: bool) -> TestResult<Vec<u8>> {
    let mut message = vec![
        if present { 0x2b } else { 0x2d },
        0x4e,
        0x44,
        3,
        0,
        0,
        0,
        0,
        0x20,
        0x17,
        0,
        0,
    ];
    message.extend(u32::try_from(serial.len() + 1)?.to_le_bytes());
    message.extend(serial.bytes().chain([0]));
    Ok(message)
}

fn identity() -> Result<Identity, Error> {
    let unit = DeviceIdentity {
        vendor: 0x17cc,
        product: 0x1720,
        container: "{container}",
        serial: "S4ABC123",
    };
    Identity::correlate(&[unit], &[unit])
}

impl Service {
    async fn discover(&mut self, reported_serial: &str) -> TestResult {
        assert_eq!(
            self.bootstrap_device.read().await?,
            [
                0, 0x75, 0x44, 3, 0x20, 0x17, 0, 0, 0x32, 0x4d, 0x69, 0x4e, 0x79, 0x6d, 0x72, 0x70,
                0, 0, 0, 0
            ]
        );
        self.bootstrap_device
            .write(&connect_reply("DeviceRequest", "DeviceNotification")?)
            .await?;
        let ack = self.device.read().await?;
        assert_eq!(&ack[..4], [0, 0x43, 0x40, 3]);
        assert_eq!(&ack[16..], b"DeviceNotification\0");
        self.device.write(&YES).await?;
        assert_eq!(self.device.read().await?, [0x43, 0x71, 0x44, 3]);
        self.device_notes
            .write(&device_note(reported_serial, true)?)
            .await?;
        assert!(self.device_notes.read().await?.is_empty());
        Ok(())
    }

    async fn handshake(
        mut self,
        reported_serial: &str,
        control: Option<&[u8]>,
    ) -> TestResult<Self> {
        self.discover(reported_serial).await?;
        // With a mismatched serial, a correct client must not register an instance.
        if reported_serial != "S4ABC123" {
            return Ok(self);
        }
        self.register_instance(control).await?;
        for command in [
            0x0349_734e_u32,
            0x0343_4300,
            0x0349_7354,
            0x0349_7374,
            0x0344_5200,
            0x0344_4100,
        ] {
            let request = self.instance.read().await?;
            assert_eq!(request.get(..4), Some(command.to_le_bytes().as_slice()));
        }
        Ok(self)
    }

    async fn register_instance(&mut self, control: Option<&[u8]>) -> TestResult {
        let instance = self.bootstrap_instance.read().await?;
        assert_eq!(&instance[..8], [0, 0x49, 0x44, 3, 0x20, 0x17, 0, 0]);
        assert_eq!(&instance[16..20], [9, 0, 0, 0]);
        assert_eq!(&instance[20..], b"S4ABC123\0");
        self.bootstrap_instance
            .write(&connect_reply("InstanceRequest", "InstanceNotification")?)
            .await?;
        let ack = self.instance.read().await?;
        assert_eq!(&ack[16..], b"InstanceNotification\0");
        // Exercise callback ACK before the registration reply. A transport that
        // waits for registration reply before serving notifications deadlocks.
        if let Some(message) = control {
            self.instance_notes.write(message).await?;
            assert!(self.instance_notes.read().await?.is_empty());
        }
        self.instance.write(&YES).await?;
        Ok(())
    }
}

#[tokio::test(start_paused = true)]
async fn service_session_gates_control_and_publishes_both_bgr_rasters() -> TestResult {
    let (bus, service) = fixture();
    let (_cancel, rx) = watch::channel(false);
    let (session, peer) = tokio::join!(
        Session::connect(&bus, identity()?, rx),
        service.handshake("S4ABC123", Some(&ACTIVE)),
    );
    let mut session = session?;
    let mut peer = peer?;
    let mut pixels = vec![0; protocol::PIXELS];
    pixels[0] = 0xf800;
    pixels[1] = 0x07e0;
    pixels[protocol::PIXELS - 1] = 0x001f;
    let mut right_pixels = pixels.clone();
    right_pixels[0] = 0x001f;
    let left = Raster::new(&pixels)?;
    let right = Raster::new(&right_pixels)?;
    let (result, accepted) = tokio::join!(session.publish_pair([left, right]), async {
        for index in [0_u8, 1] {
            let message = peer.instance.read().await?;
            assert_eq!(message.len(), 68);
            assert_eq!(
                &message[..20],
                &[
                    0x44, 0x73, 0x64, 3, index, 0, 0, 0, 0, 0, 0, 0, 0xf0, 0, 0x40, 1, 48, 0, 0, 0
                ]
            );
            assert_eq!(
                &message[20..40],
                &[
                    0x84, 0, index, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x40, 0, 0xf0, 0, 0, 0, 1
                ]
            );
            assert_eq!(
                &message[40..44],
                if index == 0 {
                    &[0, 0x1f, 7, 0xe0]
                } else {
                    &[0xf8, 0, 7, 0xe0]
                }
            );
            assert_eq!(
                &message[44..60],
                &[1, 0, 0x95, 0xfe, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0xf8, 0]
            );
            assert_eq!(&message[60..], &[3, 0, 0, 0, 0x40, 0, 0, 0]);
            peer.instance.write(&YES).await?;
        }
        Ok::<(), Error>(())
    },);
    result?;
    accepted?;
    assert_eq!(bus.live.load(Ordering::SeqCst), 4);
    session.close();
    assert_eq!(bus.live.load(Ordering::SeqCst), 0);
    assert!(
        peer.instance.read().await.is_err(),
        "close must send no shutdown image"
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn setup_requires_exact_service_serial_and_positive_control() -> TestResult {
    for (serial, control) in [
        ("OTHER123", Some(ACTIVE.as_slice())),
        ("S4ABC123", None),
        ("S4ABC123", Some(LOST.as_slice())),
    ] {
        let (bus, service) = fixture();
        let (_cancel, rx) = watch::channel(false);
        let (result, peer) = tokio::join!(
            Session::connect(&bus, identity()?, rx),
            service.handshake(serial, control),
        );
        assert!(matches!(result, Err(Error::Timeout)));
        let mut peer = peer?;
        assert_eq!(bus.live.load(Ordering::SeqCst), 0);
        assert!(peer.instance.rx.try_recv().is_err());
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn ready_control_loss_prevents_any_frame_and_closes_owned_resources() -> TestResult {
    for note in [
        LOST.as_slice(),
        &[0, 0x4e, 0x43, 3, 1, 0, 0, 0],
        &ACTIVE[..4],
    ] {
        let (bus, service) = fixture();
        let (_cancel, rx) = watch::channel(false);
        let (session, peer) = tokio::join!(
            Session::connect(&bus, identity()?, rx),
            service.handshake("S4ABC123", Some(&ACTIVE)),
        );
        let mut session = session?;
        let mut peer = peer?;
        peer.instance_notes.write(note).await?;
        let pixels = vec![0; protocol::PIXELS];
        let raster = Raster::new(&pixels)?;
        let result = session.publish_pair([raster, raster]).await;
        if note == LOST {
            assert!(matches!(result, Err(Error::ControlLost)));
        } else {
            assert!(matches!(result, Err(Error::Malformed(_))));
        }
        assert_eq!(bus.live.load(Ordering::SeqCst), 0);
        assert!(peer.instance.read().await.is_err());
        assert!(matches!(
            session.publish_pair([raster, raster]).await,
            Err(Error::Disconnected)
        ));
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn in_flight_loss_rejection_timeout_and_disconnect_stop_second_screen() -> TestResult {
    for outcome in 0..5 {
        let (bus, service) = fixture();
        let (_cancel, rx) = watch::channel(false);
        let (session, peer) = tokio::join!(
            Session::connect(&bus, identity()?, rx),
            service.handshake("S4ABC123", Some(&ACTIVE)),
        );
        let mut session = session?;
        let mut peer = peer?;
        let pixels = vec![0; protocol::PIXELS];
        let raster = Raster::new(&pixels)?;
        let (result, accepted) = tokio::join!(session.publish_pair([raster, raster]), async {
            let frame = peer.instance.read().await?;
            assert_eq!(frame.len(), 52);
            match outcome {
                0 => {
                    peer.instance_notes.write(&LOST).await?;
                    peer.instance.write(&YES).await?;
                }
                1 => peer.instance.write(&[0, 0, 0, 0]).await?,
                2 => {}
                3 => peer.instance.write(&[1, 2]).await?,
                4 => {
                    let (tx, _rx) = mpsc::channel(1);
                    peer.instance.tx = tx;
                }
                _ => unreachable!(),
            }
            Ok::<(), Error>(())
        },);
        accepted?;
        match outcome {
            0 => assert!(matches!(result, Err(Error::ControlLost))),
            1 => assert!(matches!(result, Err(Error::Rejected))),
            2 => assert!(matches!(result, Err(Error::Timeout))),
            3 => assert!(matches!(result, Err(Error::Malformed(_)))),
            4 => assert!(matches!(result, Err(Error::Disconnected))),
            _ => unreachable!(),
        }
        assert_eq!(bus.live.load(Ordering::SeqCst), 0);
        assert!(
            peer.instance.read().await.is_err(),
            "no second screen or shutdown image"
        );
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn unsolicited_late_reply_and_short_write_cannot_acknowledge_a_frame() -> TestResult {
    for short in [false, true] {
        let (bus, service) = fixture();
        let (_cancel, rx) = watch::channel(false);
        let (session, peer) = tokio::join!(
            Session::connect(&bus, identity()?, rx),
            service.handshake("S4ABC123", Some(&ACTIVE)),
        );
        let mut session = session?;
        let mut peer = peer?;
        if short {
            session
                .resources
                .as_mut()
                .ok_or("missing live resources")?
                .instance
                .request
                .max_write = Some(32);
        } else {
            peer.instance.write(&YES).await?;
        }
        let pixels = vec![0; protocol::PIXELS];
        let raster = Raster::new(&pixels)?;
        let result = session.publish_pair([raster, raster]).await;
        if short {
            assert!(matches!(
                result,
                Err(Error::ShortWrite {
                    expected: 52,
                    actual: 32
                })
            ));
            assert_eq!(peer.instance.read().await?.len(), 32);
        } else {
            assert!(matches!(
                result,
                Err(Error::Malformed("unsolicited request reply"))
            ));
        }
        assert_eq!(bus.live.load(Ordering::SeqCst), 0);
        assert!(peer.instance.read().await.is_err());
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn setup_cancellation_closes_bootstrap_without_registering_instance() -> TestResult {
    let (bus, mut service) = fixture();
    let (cancel, rx) = watch::channel(false);
    let (result, interrupted) = tokio::join!(Session::connect(&bus, identity()?, rx), async {
        service.bootstrap_device.read().await?;
        cancel.send(true).map_err(|_| Error::Disconnected)?;
        Ok::<(), Error>(())
    },);
    interrupted?;
    assert!(matches!(result, Err(Error::Cancelled)));
    assert_eq!(bus.live.load(Ordering::SeqCst), 0);
    assert!(service.bootstrap_instance.rx.try_recv().is_err());
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn unsolicited_setup_reply_cannot_acknowledge_first_frame() -> TestResult {
    let (bus, service) = fixture();
    let (_cancel, rx) = watch::channel(false);
    let (session, peer) = tokio::join!(
        Session::connect(&bus, identity()?, rx),
        service.handshake("S4ABC123", Some(&ACTIVE)),
    );
    let mut session = session?;
    let mut peer = peer?;
    peer.instance.write(&YES).await?;
    let pixels = vec![0; protocol::PIXELS];
    let raster = Raster::new(&pixels)?;
    assert!(matches!(
        session.publish_pair([raster, raster]).await,
        Err(Error::Malformed("unsolicited request reply"))
    ));
    assert_eq!(bus.live.load(Ordering::SeqCst), 0);
    assert!(
        peer.instance.read().await.is_err(),
        "a setup reply must not authorize any frame write"
    );
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cancellation_and_dropped_publish_future_release_live_session() -> TestResult {
    for drop_future in [false, true] {
        let (bus, service) = fixture();
        let (cancel, rx) = watch::channel(false);
        let (session, peer) = tokio::join!(
            Session::connect(&bus, identity()?, rx),
            service.handshake("S4ABC123", Some(&ACTIVE)),
        );
        let mut session = session?;
        let mut peer = peer?;
        let pixels = vec![0; protocol::PIXELS];
        let raster = Raster::new(&pixels)?;
        {
            let publish = session.publish_pair([raster, raster]);
            tokio::pin!(publish);
            tokio::select! {
                biased;
                result = &mut publish => {
                    result?;
                    return Err("unexpected publish completion".into());
                }
                message = peer.instance.read() => assert_eq!(message?.len(), 52),
            }
            if !drop_future {
                cancel.send(true)?;
                assert!(matches!(publish.await, Err(Error::Cancelled)));
            }
        }
        assert_eq!(bus.live.load(Ordering::SeqCst), 0);
        assert!(matches!(
            session.publish_pair([raster, raster]).await,
            Err(Error::Disconnected)
        ));
        assert!(peer.instance.read().await.is_err());
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn cancellation_before_setup_opens_no_pipe() -> TestResult {
    for close_sender in [false, true] {
        let (bus, _service) = fixture();
        let (cancel, rx) = watch::channel(!close_sender);
        let _keepalive = if close_sender {
            drop(cancel);
            None
        } else {
            Some(cancel)
        };
        assert!(matches!(
            Session::connect(&bus, identity()?, rx).await,
            Err(Error::Cancelled)
        ));
        assert_eq!(bus.live.load(Ordering::SeqCst), 0);
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn idle_poll_observes_loss_without_publishing_or_reusing_session() -> TestResult {
    let (bus, service) = fixture();
    let (_cancel, rx) = watch::channel(false);
    let (session, peer) = tokio::join!(
        Session::connect(&bus, identity()?, rx),
        service.handshake("S4ABC123", Some(&ACTIVE)),
    );
    let mut session = session?;
    let mut peer = peer?;
    session.poll_control().await?;
    peer.instance_notes.write(&LOST).await?;
    assert!(matches!(
        session.poll_control().await,
        Err(Error::ControlLost)
    ));
    assert_eq!(bus.live.load(Ordering::SeqCst), 0);
    assert!(peer.instance.read().await.is_err());
    assert!(matches!(
        session.poll_control().await,
        Err(Error::Disconnected)
    ));
    Ok(())
}

#[test]
fn identity_rejects_ambiguous_mismatched_and_missing_correlation() {
    let valid = DeviceIdentity {
        vendor: 0x17cc,
        product: 0x1720,
        container: "unit",
        serial: "S4ABC123",
    };
    assert!(Identity::correlate(&[valid], &[valid]).is_ok());
    for (hid, display) in [
        (vec![], vec![valid]),
        (vec![valid, valid], vec![valid]),
        (vec![valid], vec![]),
        (vec![valid], vec![valid, valid]),
        (
            vec![valid],
            vec![DeviceIdentity {
                container: "other",
                ..valid
            }],
        ),
        (
            vec![valid],
            vec![DeviceIdentity {
                serial: "OTHER123",
                ..valid
            }],
        ),
        (
            vec![valid],
            vec![DeviceIdentity {
                serial: "",
                ..valid
            }],
        ),
        (vec![valid], vec![DeviceIdentity { vendor: 0, ..valid }]),
    ] {
        assert!(matches!(
            Identity::correlate(&hid, &display),
            Err(Error::Identity(_))
        ));
    }
}

#[test]
fn bounded_parsers_reject_truncation_lengths_paths_and_invalid_control() -> TestResult {
    let reply = connect_reply("DeviceRequest", "DeviceNotification")?;
    for end in 0..reply.len() {
        assert!(
            protocol::ports(&reply[..end]).is_err(),
            "accepted truncation {end}"
        );
    }
    assert!(protocol::ports(&reply).is_ok());
    for request in [
        "",
        "../escape",
        r"\\remote\pipe\evil",
        "embedded\0nul",
        "µnicode",
    ] {
        assert!(protocol::ports(&connect_reply(request, "Notification")?).is_err());
    }
    for offset in [4, 8 + "DeviceRequest".len() + 1] {
        let mut invalid = reply.clone();
        invalid[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(protocol::ports(&invalid).is_err());
    }
    for note in [
        &ACTIVE[..4],
        &[0, 0x4e, 0x43, 3, 1, 0, 0, 0],
        &[0, 0x4e, 0x43, 3, 0, 0, 0, 0, 0],
    ] {
        assert!(protocol::notification(note).is_err());
    }
    let valid = device_note("S4ABC123", true)?;
    for end in 0..valid.len() {
        assert!(protocol::notification(&valid[..end]).is_err());
    }
    assert!(Raster::new(&[0; 17]).is_err());
    Ok(())
}
