//! Process QA for managed NI mode; this module is never shipped.

use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context as _, ensure};
use clap::Parser as _;
use crossbeam_channel::{Receiver, unbounded};
use serde_json::Value;

use crate::bridge_runtime::{
    self, MotorSettings,
    controller::{Decoder, Handle},
};
use crate::screens::ni_service::{
    self,
    macos_fixture::{self as ni, Fixture, Mode},
};
use crate::{Args, full, lifecycle, tension};

const CHILD: &str = "ni_process_tests::offline_managed_child";
const BOOT: &str = "S4_TEST_PRIVATE_NI_BOOTSTRAP";
const WAIT: Duration = Duration::from_secs(8);
const LIVE: u32 = 173;
const LAUNCH: [&str; 6] = [
    "s4mk3-hid-midi-bridge",
    "--midi",
    "full",
    "--ni-decks",
    "--managed-stdin",
    "--until-stopped",
];

pub fn active() -> bool {
    ni_service::offline_active()
}

#[test]
fn offline_managed_child() -> anyhow::Result<()> {
    let Some(name) = std::env::var_os(BOOT) else {
        return Ok(());
    };
    ni_service::select_offline_bootstrap(
        name.into_string()
            .map_err(|_| anyhow::anyhow!("invalid fixture bootstrap"))?,
    )?;
    crate::run_cli(Args::try_parse_from(LAUNCH)?)
}

pub fn open(
    args: &Args,
    config: Option<tension::Config>,
    jog: Option<full::jog::Config>,
    session: Option<&mut lifecycle::Session>,
) -> anyhow::Result<()> {
    use encdr::device::{led_builder::LedBuilder, loader::DescriptorRegistry};

    ensure!(args.screens.ni_decks, "fixture admits only NI deck output");
    let session = session.context("fixture requires managed session")?;
    let mut registry = DescriptorRegistry::new();
    registry.load_builtins()?;
    let descriptor = crate::full_control_descriptor(
        registry
            .find(crate::S4_VENDOR_ID, crate::S4_PRODUCT_ID)
            .context("S4 fixture descriptor")?,
    )?;
    let interface = descriptor
        .interface_by_id("control")
        .context("fixture control interface")?;
    let buttons = descriptor.leds.first().context("fixture button layout")?;
    let mut leds = LedBuilder::new(buttons, interface);
    crate::ni_runtime::bind(session, Some(ni::SERIAL))?;
    session.milestone(|ready| ready.hid = true)?;
    bridge_runtime::run(
        args,
        &Handle::Offline,
        Handle::Offline,
        Decoder::Offline,
        &mut leds,
        MotorSettings {
            tension: config.context("fixture full-profile settings")?,
            jog: jog.context("fixture jog settings")?,
        },
        session,
    )
}

struct Process {
    child: Child,
    lines: Receiver<String>,
    stderr: Option<std::thread::JoinHandle<std::io::Result<String>>>,
    transcript: Vec<String>,
}

impl Process {
    fn start(bootstrap: &str) -> anyhow::Result<Self> {
        let child = Command::new(std::env::current_exe()?)
            .args(["--exact", CHILD, "--nocapture"])
            .env_clear()
            .env(BOOT, bootstrap)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let (tx, lines) = unbounded();
        let mut process = Self {
            child,
            lines,
            stderr: None,
            transcript: Vec::new(),
        };
        let stdout = process.child.stdout.take().context("child stdout")?;
        let stderr = process.child.stderr.take().context("child stderr")?;
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        process.stderr = Some(std::thread::spawn(move || std::io::read_to_string(stderr)));
        Ok(process)
    }

    fn expect(&mut self, predicate: impl Fn(&str) -> bool) -> anyhow::Result<String> {
        let limit = Instant::now() + WAIT;
        loop {
            let line = self.lines.recv_deadline(limit).with_context(|| {
                format!("missing child stdout receipt; saw {:?}", self.transcript)
            })?;
            let matched = predicate(&line);
            self.transcript.push(line.clone());
            if matched {
                return Ok(line);
            }
        }
    }

    fn send(&mut self, command: &str) -> anyhow::Result<()> {
        writeln!(
            self.child.stdin.as_mut().context("managed stdin")?,
            "{command}"
        )?;
        Ok(())
    }

    fn finish(&mut self, success: bool) -> anyhow::Result<String> {
        let limit = Instant::now() + WAIT;
        loop {
            match self.lines.recv_deadline(limit) {
                Ok(line) => self.transcript.push(line),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                Err(error) => return Err(error).context("wait for child stdout EOF"),
            }
        }
        let status = self.child.wait()?;
        let stderr = self
            .stderr
            .take()
            .context("child stderr reader")?
            .join()
            .map_err(|_| anyhow::anyhow!("stderr reader panicked"))??;
        assert_eq!(
            status.success(),
            success,
            "stdout={:?}; stderr={stderr}",
            self.transcript
        );
        assert!(!stderr.contains("S4_FIXTURE_FORBIDDEN"), "{stderr}");
        Ok(stderr)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return;
        }
        if let Err(error) = self.child.kill() {
            eprintln!("could not terminate offline fixture child: {error}");
        }
        if let Err(error) = self.child.wait() {
            eprintln!("could not reap offline fixture child: {error}");
        }
    }
}

fn state(line: &str) -> Option<Value> {
    serde_json::from_str(line.strip_prefix("S4_BRIDGE_STATUS ")?).ok()
}

#[test]
fn managed_cli_acquires_updates_and_releases_private_service() -> anyhow::Result<()> {
    let fixture = Fixture::start(Mode::Normal);
    let mut child = Process::start(&fixture.bootstrap)?;
    child.expect(|line| state(line).is_some_and(|status| status["state"] == "starting"))?;
    let ready =
        child.expect(|line| state(line).is_some_and(|status| status["state"] == "ready"))?;
    let ready = state(&ready).context("managed ready JSON")?;
    for key in ["hid", "midi_input", "midi_output", "controller_initialized"] {
        assert_eq!(ready["readiness"][key], true);
    }
    assert_instance_ready(&fixture)?;
    child.send(&format!("ni_jog_tension {LIVE}"))?;
    let receipt = format!("S4_NI_JOG_TENSION submitted {LIVE}");
    child.expect(|line| line == receipt)?;
    fixture.await_scalar(&ni::words(&[0x0356_7375, 0x4a4f_4754, LIVE]), WAIT)?;
    child.send("S4_BRIDGE_CONTROL 42 STOP")?;
    let stop =
        child.expect(|line| state(line).is_some_and(|status| status["state"] == "stopping"))?;
    assert_eq!(
        state(&stop).context("managed stop JSON")?["request_id"],
        "42"
    );
    fixture.await_scalar(&ni::words(&[0x036a_7354, 1, 0x7472_7565]), WAIT)?;
    child.finish(true)?;
    assert_released(&fixture)?;
    Ok(())
}

fn assert_instance_ready(fixture: &Fixture) -> anyhow::Result<()> {
    let commands = fixture
        .commands
        .lock()
        .map_err(|_| anyhow::anyhow!("fixture commands poisoned"))?;
    assert!(commands.contains(&ni::words(&[0x0343_4300, 0x7374_7274])));
    assert!(commands.contains(&ni::words(&[0x0356_7375, 0x4a4f_4754, 107])));
    let frames: Vec<_> = commands
        .iter()
        .filter(|bytes| ni::command(bytes) == ni::DISPLAY)
        .map(|bytes| -> anyhow::Result<_> {
            assert!(bytes.len() > 44);
            let declared = u32::from_le_bytes(bytes[16..20].try_into()?);
            assert_eq!(usize::try_from(declared)?, bytes.len() - 20);
            Ok((bytes[4], bytes[22]))
        })
        .collect::<anyhow::Result<_>>()?;
    drop(commands);
    assert_eq!(frames, [(0, 0), (1, 1)]);
    Ok(())
}

fn assert_released(fixture: &Fixture) -> anyhow::Result<()> {
    let commands = fixture
        .commands
        .lock()
        .map_err(|_| anyhow::anyhow!("fixture commands poisoned"))?;
    let live = commands
        .iter()
        .position(|bytes| *bytes == ni::words(&[0x0356_7375, 0x4a4f_4754, LIVE]))
        .context("private service did not receive live tension")?;
    let tail = &commands[live + 1..];
    assert_eq!(tail.first(), Some(&ni::words(&[0x036a_736d, 0, 0])));
    assert_eq!(tail.get(1), Some(&ni::words(&[0x036a_736d, 1, 0])));
    for side in [0, 1] {
        for command in [0x036a_734c, 0x036a_7343] {
            assert!(tail.contains(&ni::words(&[command, side, 0])));
        }
    }
    drop(commands);
    assert!(s4_ni_message_port::Remote::open(&fixture.device_note).is_err());
    assert!(s4_ni_message_port::Remote::open(&fixture.instance_note).is_err());
    Ok(())
}

#[test]
fn managed_cli_fails_closed_without_private_service() -> anyhow::Result<()> {
    let fixture = Fixture::start(Mode::Normal);
    let unavailable = fixture.bootstrap.clone();
    drop(fixture);
    assert!(s4_ni_message_port::Remote::open(&unavailable).is_err());
    let mut child = Process::start(&unavailable)?;
    let failed =
        child.expect(|line| state(line).is_some_and(|status| status["state"] == "failed"))?;
    let status = state(&failed).context("managed failure JSON")?;
    let error = status["error"]["message"]
        .as_str()
        .context("managed failure message")?;
    assert!(error.contains("NI message port unavailable"), "{error}");
    assert_eq!(status["readiness"]["controller_initialized"], false);
    child.finish(false)?;
    assert!(
        child
            .transcript
            .iter()
            .any(|line| line == "S4_SCREEN_STATUS 0 failed")
    );
    assert!(
        !child
            .transcript
            .iter()
            .filter_map(|line| state(line))
            .any(|status| status["state"] == "ready")
    );
    Ok(())
}

#[test]
fn managed_cli_rejects_readiness_without_right_screen_ack() -> anyhow::Result<()> {
    let fixture = Fixture::start(Mode::RejectSecond);
    let mut child = Process::start(&fixture.bootstrap)?;
    let failed =
        child.expect(|line| state(line).is_some_and(|status| status["state"] == "failed"))?;
    let status = state(&failed).context("managed failure JSON")?;
    let message = status["error"]["message"]
        .as_str()
        .context("managed failure message")?;
    assert!(
        message.contains("publish first shared NI screen pair"),
        "{message}"
    );
    assert_eq!(status["readiness"]["controller_initialized"], false);
    child.finish(false)?;
    assert!(
        !child
            .transcript
            .iter()
            .filter_map(|line| state(line))
            .any(|status| status["state"] == "ready")
    );
    Ok(())
}
