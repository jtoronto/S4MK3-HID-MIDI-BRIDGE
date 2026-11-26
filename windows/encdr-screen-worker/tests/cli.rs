//! Real executable checks that never authorize service registration.
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn executable_rejects_eof_and_invalid_arm_with_one_error_line() {
    for input in [vec![], vec![0], b"READY\n".to_vec()] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_s4-encdr-screen-worker"))
            .args(["--serial", "18F73BA2"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let result = child.wait_with_output().unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.starts_with(b"ERROR "));
        assert!(result.stdout.is_ascii());
        assert!(!result.stdout.strip_suffix(b"\n").unwrap().contains(&b'\n'));
        assert!(result.stderr.is_empty());
    }
}

#[test]
fn executable_reports_cli_errors_without_protocol_noise() {
    let result = Command::new(env!("CARGO_BIN_EXE_s4-encdr-screen-worker"))
        .args(["--serial", ""])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.starts_with(b"ERROR usage: "));
    assert!(!result.stdout.strip_suffix(b"\n").unwrap().contains(&b'\n'));
    assert!(result.stderr.is_empty());
}
