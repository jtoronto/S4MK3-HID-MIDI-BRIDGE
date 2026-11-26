//! These tests require the actual supervisor, but never select an NI worker.

use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::process::Child;

use super::*;

fn launch(mode: &str) -> anyhow::Result<(Fixture, Command)> {
    let executable = std::env::var_os("S4_SCREEN_SUPERVISOR_FIXTURE").context(
        "required S4_SCREEN_SUPERVISOR_FIXTURE must name the real supervisor executable",
    )?;
    let directory = std::env::temp_dir().join(format!(
        "s4-native-screen-{}-{}-{mode}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&directory)?;
    let fixture = Fixture(directory);
    let mut command = Command::new(executable);
    command
        .arg("--self-test")
        .arg(mode)
        .arg("--worker-pid-file")
        .arg(fixture.0.join("worker.pid"));
    Ok((fixture, command))
}

fn powershell() -> anyhow::Result<PathBuf> {
    Ok(
        std::path::PathBuf::from(std::env::var_os("SystemRoot").context("Windows SystemRoot")?)
            .join("System32/WindowsPowerShell/v1.0/powershell.exe"),
    )
}

async fn connect(
    mode: &str,
    cancellation: Cancellation,
) -> anyhow::Result<(Supervisor, Fixture, Child)> {
    let (fixture, command) = launch(mode)?;
    let supervisor = tokio::task::spawn_blocking(move || {
        Supervisor::spawn(
            command,
            cancellation,
            Limits {
                frame: Duration::from_millis(250),
                stop: Duration::from_millis(250),
                ..Limits::default()
            },
        )
    })
    .await??;
    // The fake worker atomically publishes its PID before READY. Reading after
    // the real READY is event synchronization, not file polling or a sleep.
    let worker: u32 = std::fs::read_to_string(fixture.0.join("worker.pid"))?
        .trim()
        .parse()?;
    let script = format!(
        "$ErrorActionPreference='Stop'; \
         $p=[System.Diagnostics.Process]::GetProcessById({worker}); \
         [void]$p.Handle; [Console]::WriteLine('WATCHING'); \
         if(-not $p.WaitForExit(5000)){{exit 3}}; exit 0"
    );
    let mut observer = Command::new(powershell()?)
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()?;
    let mut output = BufReader::new(observer.stdout.take().context("worker observer stdout")?);
    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(3), output.read_line(&mut line)).await??;
    anyhow::ensure!(line.trim() == "WATCHING", "worker observer did not arm");
    Ok((supervisor, fixture, observer))
}

async fn worker_exited(observer: &mut Child) -> anyhow::Result<()> {
    let status = tokio::time::timeout(Duration::from_secs(6), observer.wait()).await??;
    anyhow::ensure!(
        status.success(),
        "owned fake worker survived supervisor release"
    );
    Ok(())
}

#[tokio::test]
async fn actual_supervisor_setup_failure_bounds_runtime_drop_and_kills_worker() -> anyhow::Result<()>
{
    let (fixture, command) = launch("connect-stall")?;
    let (_cancel, cancellation) = watch::channel(false);
    let connecting = tokio::task::spawn_blocking(move || {
        Supervisor::spawn(
            command,
            cancellation,
            Limits {
                setup: Duration::from_secs(2),
                ..Limits::default()
            },
        )
    });
    assert!(
        tokio::time::timeout(Duration::from_secs(4), connecting)
            .await??
            .is_err_and(|error| error.is::<Unconfirmed>())
    );
    // A missing marker is a failure: it would not exercise post-ARM setup.
    let worker: u32 = std::fs::read_to_string(fixture.0.join("worker.pid"))?
        .trim()
        .parse()?;
    let script = format!(
        "try {{ $p=[System.Diagnostics.Process]::GetProcessById({worker}); \
         if(-not $p.WaitForExit(1000)){{exit 3}} }} \
         catch [System.ArgumentException] {{ exit 0 }}; exit 0"
    );
    let result = tokio::time::timeout(
        Duration::from_secs(3),
        Command::new(powershell()?)
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .kill_on_drop(true)
            .output(),
    )
    .await??;
    anyhow::ensure!(
        result.status.success(),
        "setup failure left its worker alive"
    );
    Ok(())
}

#[tokio::test]
async fn actual_supervisor_acknowledges_pair_and_clean_stop_with_nested_job() -> anyhow::Result<()>
{
    for mode in ["normal", "flood"] {
        let (_cancel, cancellation) = watch::channel(false);
        let (mut supervisor, _fixture, mut observer) = connect(mode, cancellation).await?;
        let closed = tokio::task::spawn_blocking(move || {
            supervisor.publish(&frames())?;
            supervisor.close()
        });
        tokio::time::timeout(Duration::from_secs(3), closed).await???;
        worker_exited(&mut observer).await?;
    }
    Ok(())
}

#[tokio::test]
async fn actual_supervisor_stalls_kill_outer_and_nested_jobs_unconfirmed() -> anyhow::Result<()> {
    for mode in ["send-stall", "blocked-write", "stop-stall"] {
        let (_cancel, cancellation) = watch::channel(false);
        let (mut supervisor, _fixture, mut observer) = connect(mode, cancellation).await?;
        let closed = tokio::task::spawn_blocking(move || {
            match mode {
                "send-stall" | "blocked-write" => {
                    assert!(supervisor.publish(&frames()).is_err());
                }
                "stop-stall" => supervisor.publish(&frames())?,
                _ => unreachable!(),
            }
            let result = supervisor.close();
            drop(supervisor);
            result
        });
        assert!(
            tokio::time::timeout(Duration::from_secs(3), closed)
                .await??
                .is_err_and(|error| error.is::<Unconfirmed>())
        );
        worker_exited(&mut observer).await?;
    }
    Ok(())
}
