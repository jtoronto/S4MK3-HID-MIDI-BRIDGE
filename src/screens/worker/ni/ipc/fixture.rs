//! Standalone, device-free supervisor peer compiled for the IPC regression tests.

use std::io::{Read as _, Write as _};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut control = TcpStream::connect(&args[1])?;
    control.set_nodelay(true)?;
    let mode = &args[2];
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    // Exceeds anonymous-pipe capacity. READY requires an independent drainer.
    std::io::stderr().lock().write_all(&vec![b'x'; 262_144])?;
    if mode == "setup" {
        control.write_all(&[4])?;
        let mut command = [0];
        control.read_exact(&mut command)?;
        return Ok(());
    }
    output.write_all(b"READY\n")?;
    output.flush()?;
    loop {
        let mut header = [0; 5];
        match input.read_exact(&mut header) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error.into()),
        }
        let length = u32::from_le_bytes(header[1..].try_into()?);
        match header[0] {
            1 if length == 307_200 => {
                if mode == "blocked" {
                    control.write_all(&[3])?;
                    let mut command = [0];
                    control.read_exact(&mut command)?;
                    return Ok(());
                }
                let mut frame = vec![0; 307_200];
                input.read_exact(&mut frame)?;
                control.write_all(&[1])?;
                control.write_all(&frame)?;
                let mut command = [0];
                control.read_exact(&mut command)?;
                match command[0] {
                    b'A' => output.write_all(b"FRAME_OK\n")?,
                    b'W' => output.write_all(b"READY\n")?,
                    b'E' => output.write_all(b"ERROR fixture failure\n")?,
                    b'L' => output.write_all(&vec![b'x'; 1025])?,
                    b'P' => {
                        output.write_all(b"FRAME_OK")?;
                        return Ok(());
                    }
                    _ => return Err("unexpected fixture gate".into()),
                }
                output.flush()?;
            }
            2 if length == 0 => {
                control.write_all(&[2])?;
                if mode == "hung-stop" {
                    let mut command = [0];
                    control.read_exact(&mut command)?;
                    return Ok(());
                }
                output.write_all(b"STOPPED\n")?;
                output.flush()?;
                return Ok(());
            }
            _ => return Err("bridge violated supervisor record contract".into()),
        }
    }
}
