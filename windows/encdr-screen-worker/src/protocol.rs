//! The private supervisor pipe: ARM followed by length-prefixed frame pairs.
use anyhow::{Context, bail};
use std::io::{Read, Write};

#[cfg(any(windows, test))]
pub const WIDTH: u16 = 320;
#[cfg(any(windows, test))]
pub const HEIGHT: u16 = 240;
#[cfg(any(windows, test))]
pub const SCREEN_BYTES: usize = 320 * 240 * 2;
#[cfg(any(windows, test))]
pub const PAIR_BYTES: usize = SCREEN_BYTES * 2;
#[cfg(any(windows, test))]
const PAIR_LENGTH: u32 = 307_200;

#[cfg(any(windows, test))]
#[derive(Debug)]
pub enum Record {
    Frame(Vec<u8>),
    Stop,
}

/// No connection-capable object is constructed until this succeeds.
pub fn arm(input: &mut impl Read) -> anyhow::Result<()> {
    let mut byte = [0];
    input
        .read_exact(&mut byte)
        .context("EOF or read failure before ARM")?;
    if byte != [0xa5] {
        bail!("expected ARM byte 0xA5");
    }
    Ok(())
}

#[cfg(any(windows, test))]
pub fn record(input: &mut impl Read) -> anyhow::Result<Record> {
    let mut header = [0; 5];
    input
        .read_exact(&mut header)
        .context("EOF or incomplete record header")?;
    let [tag, a, b, c, d] = header;
    let length = u32::from_le_bytes([a, b, c, d]);
    match (tag, length) {
        (1, PAIR_LENGTH) => {
            let mut pixels = vec![0; PAIR_BYTES];
            input
                .read_exact(&mut pixels)
                .context("incomplete frame pair")?;
            Ok(Record::Frame(pixels))
        }
        (2, 0) => Ok(Record::Stop),
        (1 | 2, _) => bail!("incorrect payload length {length} for tag {tag}"),
        _ => bail!("unknown record tag {tag}"),
    }
}

pub fn emit(output: &mut impl Write, line: &str) -> anyhow::Result<()> {
    writeln!(output, "{line}").context("write protocol response")?;
    output.flush().context("flush protocol response")
}

pub fn error(output: &mut impl Write, error: &anyhow::Error) -> anyhow::Result<()> {
    // Arbitrary service errors must not inject extra responses or non-ASCII bytes.
    let detail: String = format!("{error:#}")
        .chars()
        .map(|c| {
            if c.is_ascii_graphic() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect();
    emit(output, &format!("ERROR {detail}"))
}
