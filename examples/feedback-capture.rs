//! Capture Djay's real output with paired virtual ports, without opening HID.

use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::ensure;
use clap::Parser;
use midir::os::unix::{VirtualInput as _, VirtualOutput as _};
use midir::{MidiInput, MidiOutput};
use tracing::{info, warn};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "S4 MK3 MIDI Full")]
    port: String,
    #[arg(long, default_value = "300")]
    seconds: NonZeroU64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(false)
        .try_init()
        .map_err(|error| anyhow::anyhow!("initialize capture logging: {error}"))?;
    let (sender, receiver) = crossbeam_channel::bounded(4096);
    let overflow = Arc::new(AtomicBool::new(false));
    let callback_overflow = Arc::clone(&overflow);
    let _destination = MidiInput::new("S4 feedback capture")?
        .create_virtual(
            &args.port,
            move |timestamp, bytes, ()| {
                if let Err(error) = sender.try_send((timestamp, bytes.to_vec())) {
                    callback_overflow.store(true, Ordering::Relaxed);
                    warn!(%error, "feedback capture could not retain a packet");
                }
            },
            (),
        )
        .map_err(|error| anyhow::anyhow!("create capture destination: {error}"))?;
    let _source = MidiOutput::new("S4 feedback capture")?
        .create_virtual(&args.port)
        .map_err(|error| anyhow::anyhow!("create capture source: {error}"))?;
    info!(
        port = args.port,
        "FEEDBACK_CAPTURE_READY: no HID interface opened"
    );
    let started = Instant::now();
    let duration = Duration::from_secs(args.seconds.get());
    let mut count = 0_u64;
    while let Some(remaining) = duration.checked_sub(started.elapsed()) {
        match receiver.recv_timeout(remaining) {
            Ok((timestamp, bytes)) => {
                info!(timestamp, bytes = %format_args!("{bytes:02x?}"), "DJAY_FEEDBACK");
                count = count.saturating_add(1);
            }
            Err(
                crossbeam_channel::RecvTimeoutError::Timeout
                | crossbeam_channel::RecvTimeoutError::Disconnected,
            ) => break,
        }
    }
    ensure!(
        !overflow.load(Ordering::Relaxed),
        "feedback capture lost packets; this capture is incomplete"
    );
    info!(messages = count, "FEEDBACK_CAPTURE_ENDED");
    Ok(())
}
