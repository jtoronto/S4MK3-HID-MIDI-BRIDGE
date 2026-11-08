use crate::{
    frames::Encoder,
    protocol::{self, PAIR_BYTES, Record, SCREEN_BYTES},
    session::{self, Screens},
};
use crossbeam_channel::{Receiver, Sender, bounded};
use std::{
    cell::Cell,
    io::{self, Cursor, Read, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

fn header(tag: u8, length: u32) -> Vec<u8> {
    let mut bytes = vec![tag];
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes
}

fn wire(pairs: &[Vec<u8>], stop: bool) -> Cursor<Vec<u8>> {
    let mut bytes = vec![0xa5];
    for pair in pairs {
        bytes.extend(header(1, u32::try_from(pair.len()).unwrap()));
        bytes.extend(pair);
    }
    if stop {
        bytes.extend(header(2, 0));
    }
    Cursor::new(bytes)
}

// Independent decoder of the documented NI color stream. A full frame may not
// contain skip commands or depend on previous device RAM contents.
fn decoded(bytes: &[u8], display: u8) -> Vec<u16> {
    assert_eq!(
        &bytes[..16],
        &[
            0x84, 0, display, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x40, 0, 0xf0
        ]
    );
    let mut input = Cursor::new(&bytes[16..]);
    let mut pixels = Vec::new();
    loop {
        let mut command = [0; 4];
        input.read_exact(&mut command).unwrap();
        let [op, a, b, c] = command;
        let count = usize::try_from(u32::from_be_bytes([0, a, b, c])).unwrap();
        match op {
            0 => {
                let mut data = vec![0; count * 4];
                input.read_exact(&mut data).unwrap();
                pixels.extend(
                    data.chunks_exact(2)
                        .map(|p| u16::from_be_bytes([p[0], p[1]])),
                );
            }
            1 => {
                let mut pair = [0; 4];
                input.read_exact(&mut pair).unwrap();
                for _ in 0..count {
                    pixels.extend([
                        u16::from_be_bytes([pair[0], pair[1]]),
                        u16::from_be_bytes([pair[2], pair[3]]),
                    ]);
                }
            }
            3 => {
                assert_eq!(count, 0);
                let mut footer = [0; 4];
                input.read_exact(&mut footer).unwrap();
                assert_eq!(footer, [0x40, 0, 0, 0]);
                assert_eq!(usize::try_from(input.position()).unwrap(), bytes.len() - 16);
                assert_eq!(pixels.len(), 320 * 240);
                return pixels;
            }
            _ => panic!("not a self-contained full frame: opcode {op}"),
        }
    }
}

#[test]
fn logical_primaries_match_validated_upstream_diagnostic_streams() {
    let encoder = Encoder::new().unwrap();
    for color in [0xf800_u16, 0x001f, 0x07e0, 0xffff, 0] {
        let pair = color.to_le_bytes().repeat(PAIR_BYTES / 2);
        let frames = encoder.pair(&pair).unwrap();
        for (display, frame) in [0, 1].into_iter().zip(frames) {
            let mut expected = vec![
                0x84, 0, display, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x40, 0, 0xf0, 1, 0, 0x96, 0,
            ];
            expected.extend(color.to_be_bytes().repeat(2));
            expected.extend([3, 0, 0, 0, 0x40, 0, 0, 0]);
            assert_eq!(frame, expected);
        }
    }
}

#[test]
fn all_rgb565_values_and_asymmetric_positions_survive_without_dithering() {
    // Both screens cover every possible RGB565 word, including non-extreme
    // channels. Unequal neighboring pixels and rows expose swaps and flips.
    let left: Vec<u16> = (0..320 * 240)
        .map(|i| u16::try_from(i % 65_536).unwrap())
        .collect();
    let right: Vec<u16> = left.iter().map(|p| p.rotate_left(7) ^ 0x69a5).collect();
    let pixels: Vec<u8> = left
        .iter()
        .chain(&right)
        .flat_map(|p| p.to_le_bytes())
        .collect();
    let frames = Encoder::new().unwrap().pair(&pixels).unwrap();
    assert_eq!(decoded(&frames[0], 0), left);
    assert_eq!(decoded(&frames[1], 1), right);
}

#[test]
fn repeated_and_single_pixel_changes_always_encode_independent_full_frames() {
    let encoder = Encoder::new().unwrap();
    let initial = 0x6b4d_u16.to_le_bytes().repeat(PAIR_BYTES / 2);
    let first = encoder.pair(&initial).unwrap();
    let repeated = encoder.pair(&initial).unwrap();
    assert_eq!(repeated, first);
    let mut changed = initial;
    changed[SCREEN_BYTES + 2..SCREEN_BYTES + 4].copy_from_slice(&0xa536_u16.to_le_bytes());
    let frames = encoder.pair(&changed).unwrap();
    assert_eq!(frames[0], first[0]);
    let mut expected = vec![0x6b4d; 320 * 240];
    expected[1] = 0xa536;
    assert_eq!(decoded(&frames[1], 1), expected);
}

struct Fragmented(Cursor<Vec<u8>>);
impl Read for Fragmented {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let size = output.len().min(3);
        self.0.read(&mut output[..size])
    }
}

#[test]
fn fragmented_pipe_reads_preserve_frame_boundaries_and_little_endian_length() {
    let pair: Vec<u8> = (0..PAIR_BYTES)
        .map(|i| u8::try_from(i % 251).unwrap())
        .collect();
    let mut input = Fragmented(wire(std::slice::from_ref(&pair), true));
    protocol::arm(&mut input).unwrap();
    let Record::Frame(actual) = protocol::record(&mut input).unwrap() else {
        panic!("expected frame");
    };
    assert_eq!(actual, pair);
    assert!(matches!(
        protocol::record(&mut input).unwrap(),
        Record::Stop
    ));
}

#[test]
fn malformed_tags_lengths_truncated_headers_and_payloads_fail() {
    let mut invalid = vec![
        header(1, 0),
        header(1, 307_199),
        header(1, 307_201),
        header(1, u32::MAX),
        header(2, 1),
        vec![1, 0, 4, 0xb0, 0], // 307200 encoded BE instead of LE.
    ];
    for tag in (0..=255).filter(|tag| *tag != 1 && *tag != 2) {
        invalid.push(header(tag, 0));
    }
    for size in 0..5 {
        invalid.push(header(1, 307_200)[..size].to_vec());
    }
    for size in [0, 1, SCREEN_BYTES, PAIR_BYTES - 1] {
        let mut truncated = header(1, 307_200);
        truncated.resize(5 + size, 0x43);
        invalid.push(truncated);
    }
    for bytes in invalid {
        assert!(protocol::record(&mut bytes.as_slice()).is_err());
    }
}

#[derive(Default)]
enum SendEffect {
    #[default]
    Accept,
    Fail(u8),
    LoseControl(u8),
}

type CapturedFrames = Arc<Mutex<Vec<(u8, Vec<u8>)>>>;

struct TestScreens {
    control: Receiver<bool>,
    changes: Option<Sender<bool>>,
    frames: CapturedFrames,
    dropped: Arc<AtomicBool>,
    effect: SendEffect,
}

impl TestScreens {
    fn granted() -> Self {
        let (sender, control) = bounded(4);
        sender.send(true).unwrap();
        Self {
            control,
            changes: Some(sender),
            frames: Arc::default(),
            dropped: Arc::default(),
            effect: SendEffect::Accept,
        }
    }
}

impl Drop for TestScreens {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

impl Screens for TestScreens {
    fn control(&self) -> &Receiver<bool> {
        &self.control
    }

    fn send_display(&self, display: u8, commands: &[u8]) -> anyhow::Result<()> {
        self.frames
            .lock()
            .unwrap()
            .push((display, commands.to_vec()));
        match self.effect {
            SendEffect::Fail(target) if target == display => anyhow::bail!("send refused"),
            SendEffect::LoseControl(target) if target == display => {
                self.changes.as_ref().unwrap().send(false).unwrap();
                Ok(())
            }
            SendEffect::Accept | SendEffect::Fail(_) | SendEffect::LoseControl(_) => Ok(()),
        }
    }
}

#[test]
fn every_non_arm_byte_and_eof_prevent_service_construction() {
    let mut inputs = vec![vec![]];
    inputs.extend((0..=255).filter(|b| *b != 0xa5).map(|b| vec![b]));
    for input in inputs {
        let connected = Cell::new(false);
        let mut output = Vec::new();
        let result = session::run(Cursor::new(input), &mut output, || {
            connected.set(true);
            Ok(TestScreens::granted())
        });
        assert!(result.is_err());
        assert!(!connected.get());
        assert!(output.is_empty());
    }
}

#[test]
fn no_ready_or_frame_send_without_actual_control_grant() {
    let mut screens = TestScreens::granted();
    screens.control.recv().unwrap(); // Remove grant; only a denial then EOF.
    screens.changes.as_ref().unwrap().send(false).unwrap();
    screens.changes = None;
    let frames = Arc::clone(&screens.frames);
    let mut output = Vec::new();
    let result = session::run(wire(&[], true), &mut output, || Ok(screens));
    assert!(result.unwrap_err().to_string().contains("before grant"));
    assert!(output.is_empty());
    assert!(frames.lock().unwrap().is_empty());
}

#[test]
fn ack_requires_both_sends_and_stop_drops_session_without_clearing() {
    let screens = TestScreens::granted();
    let frames = Arc::clone(&screens.frames);
    let dropped = Arc::clone(&screens.dropped);
    let pair = [
        0xf800_u16.to_le_bytes().repeat(SCREEN_BYTES / 2),
        0x001f_u16.to_le_bytes().repeat(SCREEN_BYTES / 2),
    ]
    .concat();
    let mut output = Vec::new();
    session::run(wire(&[pair], true), &mut output, || Ok(screens)).unwrap();
    assert_eq!(output, b"READY\nFRAME_OK\nSTOPPED\n");
    let frames = frames.lock().unwrap();
    assert_eq!(frames.len(), 2);
    assert_eq!(frames.iter().map(|f| f.0).collect::<Vec<_>>(), [0, 1]);
    assert_eq!(decoded(&frames[0].1, 0), vec![0xf800; 320 * 240]);
    assert_eq!(decoded(&frames[1].1, 1), vec![0x001f; 320 * 240]);
    drop(frames);
    assert!(dropped.load(Ordering::SeqCst));
}

#[test]
fn failure_or_control_loss_during_either_send_never_acknowledges_pair() {
    for effect in [
        SendEffect::Fail(0),
        SendEffect::Fail(1),
        SendEffect::LoseControl(0),
        SendEffect::LoseControl(1),
    ] {
        let mut screens = TestScreens::granted();
        screens.effect = effect;
        let mut output = Vec::new();
        let result = session::run(wire(&[vec![0; PAIR_BYTES]], true), &mut output, || {
            Ok(screens)
        });
        assert!(result.is_err());
        assert_eq!(output, b"READY\n");
    }
}

#[test]
fn eof_after_arm_or_after_complete_pair_is_a_session_error() {
    for pairs in [vec![], vec![vec![0; PAIR_BYTES]]] {
        let screens = TestScreens::granted();
        let mut output = Vec::new();
        let result = session::run(wire(&pairs, false), &mut output, || Ok(screens));
        assert!(result.unwrap_err().to_string().contains("record header"));
        let expected = if pairs.is_empty() {
            "READY\n"
        } else {
            "READY\nFRAME_OK\n"
        };
        assert_eq!(output, expected.as_bytes());
    }
}

#[test]
fn queued_control_loss_or_closed_notifications_prevent_ready() {
    for closed in [false, true] {
        let mut screens = TestScreens::granted();
        if closed {
            screens.changes = None;
        } else {
            screens.changes.as_ref().unwrap().send(false).unwrap();
        }
        let mut output = Vec::new();
        assert!(session::run(wire(&[], true), &mut output, || Ok(screens)).is_err());
        assert!(output.is_empty());
    }
}

struct ObservedOutput(Sender<Vec<u8>>);
impl Write for ObservedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.send(bytes.to_vec()).map_err(io::Error::other)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// Holds stdin without fixed sleeps while a real control loss arrives.
struct HeldInput(Receiver<()>);
impl Read for HeldInput {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        self.0.recv().map_err(io::Error::other)?;
        Ok(0)
    }
}

#[test]
fn idle_control_loss_ends_session_without_waiting_for_stdin() {
    let screens = TestScreens::granted();
    let changes = screens.changes.as_ref().unwrap().clone();
    let (release_input, blocked_input) = bounded(0);
    let (writes, observed) = bounded(10);
    let (done, result) = bounded(1);
    let thread = std::thread::spawn(move || {
        let input = Cursor::new(vec![0xa5]).chain(HeldInput(blocked_input));
        let outcome = session::run(input, &mut ObservedOutput(writes), || Ok(screens));
        done.send(outcome).unwrap();
    });
    let deadline = std::time::Duration::from_secs(5);
    let mut ready = Vec::new();
    while !ready.ends_with(b"\n") {
        ready.extend(observed.recv_timeout(deadline).unwrap());
    }
    assert_eq!(ready, b"READY\n");
    changes.send(false).unwrap();
    assert!(
        result
            .recv_timeout(deadline)
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("control lost")
    );
    drop(release_input); // Releases the isolated reader before this fixture ends.
    thread.join().unwrap();
}

#[test]
fn error_response_is_exactly_one_ascii_line() {
    let error = anyhow::anyhow!("first\nREADY\r\u{1b}\u{2603}last");
    let mut output = Vec::new();
    protocol::error(&mut output, &error).unwrap();
    assert_eq!(output, b"ERROR first READY   last\n");
}

#[test]
fn serial_cli_requires_exact_flag_and_valid_nonempty_serial() {
    for args in [
        vec![],
        vec!["--serial"],
        vec!["--serial", ""],
        vec!["--serial", "S4-123"],
        vec!["--serial", "serial\nREADY"],
        vec!["--serial", "1234", "--extra"],
        vec!["--other", "1234"],
    ] {
        assert!(crate::serial(args.into_iter().map(str::to_owned)).is_err());
    }
    assert!(crate::serial(["--serial".into(), "a".repeat(65)]).is_err());
    assert_eq!(
        crate::serial(["--serial".into(), "18F73BA2".into()]).unwrap(),
        "18F73BA2"
    );
}
