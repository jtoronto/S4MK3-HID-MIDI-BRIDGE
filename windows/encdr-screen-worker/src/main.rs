//! Production Windows screen worker. The supervisor owns containment and deadlines.
use anyhow::{Context, bail};
use std::io;
use std::process::ExitCode;

#[cfg(any(windows, test))]
mod frames;
mod protocol;
#[cfg(any(windows, test))]
mod session;

fn serial(args: impl IntoIterator<Item = String>) -> anyhow::Result<String> {
    let mut args = args.into_iter();
    match (args.next().as_deref(), args.next(), args.next()) {
        (Some("--serial"), Some(serial), None)
            if !serial.is_empty()
                && serial.len() <= 64
                && serial.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
        {
            Ok(serial)
        }
        _ => bail!("usage: s4-encdr-screen-worker --serial SERIAL (1-64 ASCII alphanumeric)"),
    }
}

fn run() -> anyhow::Result<()> {
    let serial = serial(std::env::args().skip(1))?;
    #[cfg(windows)]
    {
        tracing_subscriber::fmt()
            .with_writer(io::stderr)
            .with_ansi(false)
            .try_init()
            .map_err(|error| anyhow::anyhow!("initialize stderr logging: {error}"))?;
        session::run(io::stdin(), &mut io::stdout().lock(), || {
            session::NativeScreens::connect(&serial)
        })
    }
    #[cfg(not(windows))]
    {
        // A host build is useful for offline tests; it must never connect to
        // the macOS service through Encdr's platform-specific implementation.
        drop(serial);
        protocol::arm(&mut io::stdin().lock())?;
        bail!("NI screen service worker requires Windows");
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if let Err(write_error) =
                protocol::error(&mut io::stdout().lock(), &error).context("report worker failure")
            {
                eprintln!("{write_error:#}");
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests;
