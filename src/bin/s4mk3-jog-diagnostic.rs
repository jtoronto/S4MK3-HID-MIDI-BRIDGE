//! Standalone tension diagnostic; never part of the normal MIDI bridge.

#[path = "../input.rs"]
mod input;
#[path = "../jog_diagnostic/mod.rs"]
mod jog_diagnostic;
#[path = "../platform.rs"]
mod platform;

use clap::Parser as _;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_ansi(false)
        .try_init()
        .map_err(|error| anyhow::anyhow!("initialize logging: {error}"))?;
    jog_diagnostic::run(jog_diagnostic::Args::parse())
}
