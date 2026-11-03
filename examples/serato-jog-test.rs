//! Synthetic Serato jog experiment. No HID, audio, screens or motor access.

use std::io::{self, BufRead as _};
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail, ensure};
use clap::Parser;
use crossbeam_channel::{Receiver, Sender};
use midir::os::unix::{VirtualInput as _, VirtualOutput as _};
use midir::{MidiInput, MidiOutput, MidiOutputConnection};
use tracing::{info, warn};

const PORT: &str = "S4 Serato Jog Test";
const RELEASE: [u8; 3] = [0x80, 7, 0];
const TICKS: usize = 128;

#[derive(Parser)]
#[command(about = "Interactive synthetic jog test; opens virtual MIDI only")]
struct Args {
    /// Relative counts per movement packet; restart to change sensitivity.
    #[arg(long, default_value = "8", value_parser = clap::value_parser!(u8).range(1..=63))]
    step: u8,
    /// Verify actual `CoreMIDI` delivery on a separate, temporary test port.
    #[arg(long)]
    self_test: bool,
}

#[derive(Clone, Copy)]
enum Command {
    Play,
    Touch,
    Release,
    Forward,
    Backward,
    Help,
    Quit,
}

impl Command {
    fn parse(line: &str) -> anyhow::Result<Self> {
        match line.trim() {
            "p" => Ok(Self::Play),
            "t" => Ok(Self::Touch),
            "r" => Ok(Self::Release),
            "f" => Ok(Self::Forward),
            "b" => Ok(Self::Backward),
            "?" | "" => Ok(Self::Help),
            "q" => Ok(Self::Quit),
            other => bail!("unknown command {other:?}; enter ? for help"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(false)
        .try_init()
        .map_err(|error| anyhow::anyhow!("initialize probe logging: {error}"))?;
    if args.self_test {
        return loopback(&args);
    }
    let _destination = MidiInput::new(PORT)?
        .create_virtual(
            PORT,
            |timestamp, bytes, ()| {
                info!(timestamp, bytes = %format_args!("{bytes:02x?}"), "SERATO_RETURN");
            },
            (),
        )
        .map_err(|error| anyhow::anyhow!("create return destination: {error}"))?;
    let mut source = MidiOutput::new(PORT)?
        .create_virtual(PORT)
        .map_err(|error| anyhow::anyhow!("create MIDI source: {error}"))?;
    let (sender, receiver) = crossbeam_channel::unbounded();
    let stop = sender.clone();
    ctrlc::set_handler(move || {
        if let Err(error) = stop.send(Command::Quit) {
            warn!(%error, "probe already stopped");
        }
    })
    .context("install Ctrl-C handler")?;
    std::thread::Builder::new()
        .name("serato-jog-commands".into())
        .spawn(move || read_commands(&sender))
        .context("start terminal command reader")?;
    info!(
        port = PORT,
        step = args.step,
        "SERATO_JOG_READY: no MIDI sent until a command"
    );
    help();
    let result = interactive(&receiver, &mut source, &args);
    let cleanup = source.send(&RELEASE).context("release touch on exit");
    result?;
    cleanup?;
    info!("SERATO_JOG_STOPPED: touch released; virtual ports closing");
    Ok(())
}

fn read_commands(sender: &Sender<Command>) {
    for line in io::stdin().lock().lines() {
        let command = line
            .context("read terminal command")
            .and_then(|line| Command::parse(&line));
        match command {
            Ok(command) => {
                if sender.send(command).is_err() {
                    return;
                }
                if matches!(command, Command::Quit) {
                    return;
                }
            }
            Err(error) => warn!(%error, "COMMAND_REJECTED"),
        }
    }
    if let Err(error) = sender.send(Command::Quit) {
        warn!(%error, "probe already stopped at terminal EOF");
    }
}

fn interactive(
    receiver: &Receiver<Command>,
    source: &mut MidiOutputConnection,
    args: &Args,
) -> anyhow::Result<()> {
    for command in receiver {
        match command {
            Command::Quit => break,
            Command::Help => help(),
            Command::Play
            | Command::Touch
            | Command::Release
            | Command::Forward
            | Command::Backward => perform(command, source, args)?,
        }
    }
    Ok(())
}

fn perform(command: Command, source: &mut MidiOutputConnection, args: &Args) -> anyhow::Result<()> {
    let movement = match command {
        Command::Forward => Some(64 + args.step),
        Command::Backward => Some(64 - args.step),
        Command::Play | Command::Touch | Command::Release | Command::Help | Command::Quit => None,
    };
    if let Some(value) = movement {
        for _ in 0..TICKS {
            source.send(&[0xb0, 16, value])?;
            if !args.self_test {
                // This interval defines synthetic wheel speed, not test synchronization.
                std::thread::sleep(Duration::from_millis(8));
            }
        }
        info!(
            packets = TICKS,
            value,
            touched = "unchanged",
            "JOG_BURST_SENT"
        );
        return Ok(());
    }
    let messages: &[[u8; 3]] = match command {
        Command::Play => &[[0x90, 0, 127], [0x80, 0, 0]],
        Command::Touch => &[[0x90, 7, 127]],
        Command::Release => &[RELEASE],
        Command::Forward | Command::Backward | Command::Help | Command::Quit => &[],
    };
    for message in messages {
        source.send(message)?;
        info!(bytes = %format_args!("{message:02x?}"), "MIDI_SENT");
    }
    Ok(())
}

fn help() {
    println!(
        "Commands (type one, then Return):\n\
         p = Play/Pause sanity check\n\
         t = touch ON (held until r or exit)\n\
         r = touch OFF\n\
         f = forward movement for about one second\n\
         b = backward movement for about one second\n\
         ? = help; q = release touch and quit\n\
         Nothing is automatic. Stop playback in Serato before quitting."
    );
}

fn loopback(args: &Args) -> anyhow::Result<()> {
    let name = format!("Serato Jog Loopback {}", std::process::id());
    let mut source = MidiOutput::new(&name)?
        .create_virtual(&name)
        .map_err(|error| anyhow::anyhow!("create loopback source: {error}"))?;
    let input = MidiInput::new(&name)?;
    let mut selected = None;
    for port in input.ports() {
        if input.port_name(&port)? == name {
            selected = Some(port);
            break;
        }
    }
    let port = selected.context("loopback source absent from CoreMIDI")?;
    let (sender, receiver) = crossbeam_channel::unbounded();
    let _monitor = input
        .connect(
            &port,
            &name,
            move |_, bytes, ()| {
                if let Err(error) = sender.send(bytes.to_vec()) {
                    warn!(%error, "loopback receiver closed");
                }
            },
            (),
        )
        .map_err(|error| anyhow::anyhow!("subscribe loopback monitor: {error}"))?;
    // Subscribe first, then exercise the same parser/emitter used interactively.
    for command in ["p", "t", "f", "b", "r"] {
        perform(Command::parse(command)?, &mut source, args)?;
    }
    let mut expected = vec![0x90, 0, 127, 0x80, 0, 0, 0x90, 7, 127];
    for value in [64 + args.step, 64 - args.step] {
        for _ in 0..TICKS {
            expected.extend_from_slice(&[0xb0, 16, value]);
        }
    }
    expected.extend_from_slice(&RELEASE);
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut captured = Vec::with_capacity(expected.len());
    while captured.len() < expected.len() {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .context("CoreMIDI loopback delivery timed out")?;
        captured.extend(
            receiver
                .recv_timeout(remaining)
                .context("receive loopback")?,
        );
    }
    ensure!(
        captured == expected,
        "CoreMIDI bytes differ from the jog test contract"
    );
    info!(
        messages = captured.len() / 3,
        "SERATO_JOG_LOOPBACK_PASS: no HID; separate test port"
    );
    Ok(())
}
