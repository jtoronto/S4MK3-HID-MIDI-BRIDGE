#[cfg(not(windows))]
fn main() {
    eprintln!("s4-ni-left-jog requires Windows.");
    std::process::exit(2);
}

#[cfg(windows)]
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    use anyhow::{Context as _, ensure};
    use clap::Parser;
    use s4mk3_hid_midi_bridge::screens::ni_service::{Identity, NiService};
    use std::io::{self, IsTerminal as _};
    use tokio::sync::watch;

    #[derive(Parser)]
    #[command(about = "One attended S4 left platter trial with both NI service screens")]
    struct Args {
        /// Exact S4 serial from the unit label.
        #[arg(long)]
        serial: String,
    }

    let identity = Identity::selected_hid(&Args::parse().serial)?;
    let stdin = io::stdin();
    ensure!(stdin.is_terminal(), "use an interactive Windows console");
    eprintln!("Close Traktor, Djay and any bridge. Keep both platters clear.");
    eprintln!("Keep the S4 power switch in reach. Type START when you are watching.");
    let mut response = String::new();
    stdin
        .read_line(&mut response)
        .context("read operator confirmation")?;
    ensure!(response.trim() == "START", "operator did not start the run");

    let (stop_tx, stop_rx) = watch::channel(false);
    ctrlc::set_handler(move || {
        stop_tx.send_replace(true);
    })
    .context("install Ctrl+C stop handler")?;
    let (_session_tx, session_rx) = watch::channel(false);
    let mut connect_stop = stop_rx.clone();
    let mut service = tokio::select! {
        biased;
        result = connect_stop.changed() => {
            result.context("operator stop channel closed during acquisition")?;
            return Err(anyhow::anyhow!("operator cancelled acquisition"));
        }
        result = NiService::connect(identity, session_rx) => result?,
    };
    println!("NI_JOG_CONTROL_ACQUIRED");
    service.run_left_jog_once(stop_rx).await?;
    println!("NI_JOG_SOFTWARE_SEQUENCE_COMPLETE");
    Ok(())
}
