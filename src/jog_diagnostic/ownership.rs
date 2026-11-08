//! Refuse known competing controllers before acquiring a HID collection.

use std::process::Command;

use anyhow::{Context as _, ensure};

#[cfg(any(windows, test))]
#[path = "ownership/process.rs"]
mod process;

pub fn check() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        return windows_check();
    }
    #[cfg(not(windows))]
    {
        let output = Command::new("/bin/ps")
            .args(["-axo", "pid=,comm="])
            .output()
            .context("inspect competing controller processes")?;
        ensure!(output.status.success(), "process inspection failed");
        reject_competitors(std::str::from_utf8(&output.stdout)?, std::process::id())
    }
}

#[cfg(windows)]
fn windows_check() -> anyhow::Result<()> {
    use std::process::Stdio;
    use std::time::Duration;

    #[derive(serde::Deserialize)]
    struct Process {
        #[serde(rename = "Id")]
        id: u32,
        #[serde(rename = "ProcessName")]
        name: String,
    }
    let system = std::env::var_os("SystemRoot").context("SystemRoot missing")?;
    let executable =
        std::path::PathBuf::from(system).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut child = Command::new(executable).args([
        "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
        "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); ConvertTo-Json -Compress -InputObject @(Get-Process | Select-Object Id,ProcessName)"
    ]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().context("start Windows controller process inventory")?;
    let output = process::capture(&mut child, Duration::from_secs(5))
        .context("inspect Windows controller processes")?;
    let processes: Vec<Process> =
        serde_json::from_slice(&output.stdout).context("parse Windows process inventory")?;
    let lines = processes
        .iter()
        .map(|process| format!("{} {}", process.id, process.name))
        .collect::<Vec<_>>()
        .join("\n");
    reject_competitors(&lines, std::process::id())
}

fn reject_competitors(processes: &str, own_pid: u32) -> anyhow::Result<()> {
    for line in processes.lines().filter(|line| !line.trim().is_empty()) {
        let line = line.trim();
        let (pid, executable) = line
            .split_once(char::is_whitespace)
            .context("unexpected process listing")?;
        let pid = pid.parse::<u32>().context("invalid process ID")?;
        let name = executable
            .trim()
            .rsplit(['/', '\\'])
            .next()
            .context("missing executable name")?;
        let name = name.to_lowercase();
        let name = name.strip_suffix(".exe").unwrap_or(&name);
        ensure!(
            pid == own_pid
                || !(name == "s4mk3-hid-midi-bridge"
                    || name == "s4mk3-jog-diagnostic"
                    || name == "s4mk3-screen-diagnostic"
                    || name.starts_with("traktor")
                    || name == "mixxx"),
            "competing controller process {pid}: {executable}; stop it before diagnostic capture/trial"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_each_controller_but_allows_own_process_and_idle_app() -> anyhow::Result<()> {
        // Given this diagnostic, idle menu-bar app, and Djay using audio only.
        let safe = "1 /x/s4mk3-jog-diagnostic\n2 /x/S4Bridge\n3 /x/djay Pro";
        // When ownership is checked, then only this process is exempt.
        reject_competitors(safe, 1)?;
        for name in [
            "s4mk3-hid-midi-bridge",
            "s4mk3-jog-diagnostic",
            "s4mk3-screen-diagnostic",
            "Traktor",
            "Traktor Pro 4",
            "mixxx",
        ] {
            // Given a separate control owner, when checked, then opening is refused.
            assert!(reject_competitors(&format!("2 /x/{name}"), 1).is_err());
        }
        Ok(())
    }

    #[test]
    fn malformed_process_data_fails_closed() {
        // Given unreadable process data, when parsed, then ownership is not assumed.
        assert!(reject_competitors("not-a-pid /x/anything", 1).is_err());
    }
}
