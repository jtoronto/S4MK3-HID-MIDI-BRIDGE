//! Observe the real virtual source without requiring a third-party MIDI monitor.

use std::num::NonZeroU64;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use clap::Parser;
use midir::MidiInput;
use tracing::{info, warn};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "S4 MK3 MIDI")]
    port: String,
    #[arg(long, default_value = "60")]
    seconds: NonZeroU64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(false)
        .try_init()
        .map_err(|error| anyhow::anyhow!("initialize monitor logging: {error}"))?;
    let input = MidiInput::new("S4 MIDI monitor")?;
    let mut selected = None;
    for port in input.ports() {
        if input.port_name(&port)? == args.port {
            selected = Some(port);
            break;
        }
    }
    let port = selected.context("MIDI source absent; start the probe with --midi first")?;
    let (sender, receiver) = crossbeam_channel::unbounded();
    let _connection = input
        .connect(
            &port,
            "S4 MIDI monitor connection",
            move |_, bytes, ()| {
                if let Err(error) = sender.send(bytes.to_vec()) {
                    warn!(%error, "MIDI monitor receiver closed");
                }
            },
            (),
        )
        .map_err(|error| anyhow::anyhow!("connect MIDI monitor: {error}"))?;
    info!(port = args.port, "MIDI_MONITOR_READY");
    let started = Instant::now();
    let duration = Duration::from_secs(args.seconds.get());
    let mut count = 0_u64;
    while let Some(remaining) = duration.checked_sub(started.elapsed()) {
        match receiver.recv_timeout(remaining) {
            Ok(bytes) => {
                info!(bytes = %format_args!("{bytes:02x?}"), "MIDI_RECEIVED");
                count = count.saturating_add(1);
            }
            Err(
                crossbeam_channel::RecvTimeoutError::Timeout
                | crossbeam_channel::RecvTimeoutError::Disconnected,
            ) => break,
        }
    }
    info!(messages = count, "MIDI_MONITOR_ENDED");
    Ok(())
}
