//! Cold capture preparation completes before the host publishes its endpoint.

use std::io::{self, BufRead, Write};

use anyhow::{Context as _, bail};

use super::{Kind, diag};

pub fn await_endpoint(enabled: bool) -> anyhow::Result<bool> {
    if !enabled {
        return Ok(true);
    }
    exchange(&mut io::stdin().lock(), &mut io::stdout().lock())
}

fn exchange(reader: &mut impl BufRead, writer: &mut impl Write) -> anyhow::Result<bool> {
    diag::record(Kind::Lifecycle, [4, 0, 0, 0, 0, 0, 0, 0], &[]);
    writeln!(writer, "S4_SCREEN_DIAG_PREPARED 1")?;
    writer.flush().context("publish diagnostic preparation")?;
    let mut line = String::new();
    let received = reader
        .read_line(&mut line)
        .context("await diagnostic endpoint publication")?;
    let line = line.trim_end_matches(['\r', '\n']);
    if line == "S4_SCREEN_DIAG_BEGIN 1" {
        diag::record(Kind::Lifecycle, [5, 0, 0, 0, 0, 0, 0, 0], &[]);
        return Ok(true);
    }
    if received == 0 || crate::screens_control::parse_stop(line).is_some() {
        diag::request_stop();
        return Ok(false);
    }
    bail!("invalid diagnostic preparation command")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_consumes_only_the_startup_line_and_retains_managed_commands() -> anyhow::Result<()> {
        let mut reader = io::Cursor::new(b"S4_SCREEN_DIAG_BEGIN 1\nS4_BRIDGE_CONTROL 9 STOP\n");
        let mut output = Vec::new();
        assert!(exchange(&mut reader, &mut output)?);
        assert_eq!(output, b"S4_SCREEN_DIAG_PREPARED 1\n");
        let mut remaining = String::new();
        reader.read_line(&mut remaining)?;
        assert_eq!(remaining, "S4_BRIDGE_CONTROL 9 STOP\n");
        Ok(())
    }

    #[test]
    fn stop_or_closed_stdin_aborts_before_runtime_and_rejects_other_commands() -> anyhow::Result<()>
    {
        for line in ["", "S4_BRIDGE_CONTROL 9 STOP\n"] {
            assert!(!exchange(&mut io::Cursor::new(line), &mut Vec::new())?);
        }
        for line in [
            "\n",
            "S4_SCREEN_DIAG_BEGIN 2\n",
            "S4_SCREEN_DIAG_BEGIN 1 extra\n",
            "S4_BRIDGE_CONTROL +9 STOP\n",
        ] {
            assert!(
                exchange(&mut io::Cursor::new(line), &mut Vec::new()).is_err(),
                "{line:?}"
            );
        }
        Ok(())
    }
}
