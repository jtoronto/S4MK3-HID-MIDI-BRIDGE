use super::Error;

pub const PIXELS: usize = 320 * 240;
pub const DISPLAY_BYTES: usize = 20 + 20 + PIXELS * 2 + 8;
pub const MAX_MESSAGE: usize = 65_536;
pub const MAX_CONNECT_REPLY: usize = 4 + 4 + 241 + 4 + 241 + 4;
const TRUE: u32 = 0x7472_7565;
const PRODUCT: u32 = 0x1720;

pub fn words(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn word(message: &[u8], offset: usize) -> Result<u32, Error> {
    let bytes = message
        .get(offset..offset + 4)
        .ok_or(Error::Malformed("truncated word"))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

pub fn accepted(message: &[u8]) -> Result<(), Error> {
    if message.len() < 4 || message.len() > MAX_MESSAGE {
        return Err(Error::Malformed("reply length"));
    }
    if word(message, 0)? != TRUE {
        return Err(Error::Rejected);
    }
    Ok(())
}

pub fn serial(raw: &[u8]) -> Result<&str, Error> {
    if raw.is_empty() || raw.len() > 128 || !raw.iter().all(u8::is_ascii_alphanumeric) {
        return Err(Error::Identity("unsupported serial spelling"));
    }
    std::str::from_utf8(raw).map_err(|_| Error::Malformed("serial encoding"))
}

fn text(raw: &[u8]) -> Result<&str, Error> {
    let Some((&0, body)) = raw.split_last() else {
        return Err(Error::Malformed("missing text terminator"));
    };
    std::str::from_utf8(body).map_err(|_| Error::Malformed("text encoding"))
}

fn port(raw: &[u8]) -> Result<String, Error> {
    let name = text(raw)?;
    // The service returns a LOCAL basename, never a path. Reject embedded NUL,
    // remote paths and traversal before passing anything to ClientOptions.
    if name.is_empty()
        || name.len() > 240
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        || name.contains("..")
    {
        return Err(Error::Malformed("invalid local pipe basename"));
    }
    Ok(name.to_owned())
}

pub struct Ports {
    pub request: String,
    pub notification: String,
}

pub fn ports(message: &[u8]) -> Result<Ports, Error> {
    accepted(message)?;
    if message.len() > MAX_CONNECT_REPLY {
        return Err(Error::Malformed("connect reply size"));
    }
    let request_len =
        usize::try_from(word(message, 4)?).map_err(|_| Error::Malformed("port length"))?;
    if !(2..=241).contains(&request_len) {
        return Err(Error::Malformed("request port length"));
    }
    let end = 8 + request_len;
    let request = port(
        message
            .get(8..end)
            .ok_or(Error::Malformed("request port truncated"))?,
    )?;
    let note_len =
        usize::try_from(word(message, end)?).map_err(|_| Error::Malformed("port length"))?;
    if !(2..=241).contains(&note_len) {
        return Err(Error::Malformed("notification port length"));
    }
    let note_end = end + 4 + note_len;
    let notification = port(
        message
            .get(end + 4..note_end)
            .ok_or(Error::Malformed("notification port truncated"))?,
    )?;
    // Native replies have an optional opaque word after both names. Captures
    // contain both zero and "WTra"; its value is not a connection status.
    // Keep the observed length boundary without interpreting those bytes.
    match message.get(note_end..) {
        Some([] | [_, _, _, _]) => {}
        _ => return Err(Error::Malformed("connect reply footer")),
    }
    if request == notification {
        return Err(Error::Malformed("aliased ports"));
    }
    Ok(Ports {
        request,
        notification,
    })
}

pub fn device_connect() -> Vec<u8> {
    words(&[0x0344_7500, PRODUCT, 0x4e69_4d32, 0x7072_6d79, 0])
}

pub fn instance_connect(value: &str) -> Result<Vec<u8>, Error> {
    serial(value.as_bytes())?;
    let len = u32::try_from(value.len() + 1).map_err(|_| Error::Malformed("serial length"))?;
    let mut result = words(&[0x0344_4900, PRODUCT, 0x4e69_4d32, 0x7072_6d79, len]);
    result.extend_from_slice(value.as_bytes());
    result.push(0);
    Ok(result)
}

pub fn ack(name: &str) -> Result<Vec<u8>, Error> {
    let len = u32::try_from(name.len()).map_err(|_| Error::Malformed("port length"))?;
    let mut result = words(&[0x0340_4300, TRUE, 0, len]);
    result.extend_from_slice(name.as_bytes());
    result.push(0);
    Ok(result)
}

pub enum Notification<'a> {
    Control(bool),
    Device { present: bool, serial: &'a str },
    Other,
}

pub fn notification(message: &[u8]) -> Result<Notification<'_>, Error> {
    if message.len() > MAX_MESSAGE {
        return Err(Error::Malformed("notification too large"));
    }
    match word(message, 0)? {
        0x0343_4e00 => {
            if message.len() != 8 {
                return Err(Error::Malformed("control length"));
            }
            match word(message, 4)? {
                TRUE => Ok(Notification::Control(true)),
                0 => Ok(Notification::Control(false)),
                _ => Err(Error::Malformed("unknown control state")),
            }
        }
        kind @ (0x0344_4e2b | 0x0344_4e2d) => {
            if word(message, 8)? != PRODUCT {
                return Err(Error::Identity("service product"));
            }
            let len = usize::try_from(word(message, 12)?)
                .map_err(|_| Error::Malformed("serial length"))?;
            if !(2..=129).contains(&len) || message.len() != 16 + len {
                return Err(Error::Malformed("device notification length"));
            }
            let value = text(&message[16..])?;
            serial(value.as_bytes())?;
            Ok(Notification::Device {
                present: kind == 0x0344_4e2b,
                serial: value,
            })
        }
        // A non-control event cannot establish or revoke control. It must still
        // contain aligned words; unknown/truncated control messages fail above.
        _ if message.len().is_multiple_of(4) => Ok(Notification::Other),
        _ => Err(Error::Malformed("unaligned notification")),
    }
}

pub fn setup() -> Vec<Vec<u8>> {
    let project = b"S4Bridge\0";
    let mut name = words(&[0x0349_734e, 0x7000_1006, 0xf6b2_4000, 9]);
    name.extend_from_slice(project);
    // Only configuration writes are needed for display ownership. Unused
    // metadata queries (Keys/RKey, serial, MGD, FGC) have device-dependent
    // replies and must not leave responses on the frame request pipe.
    vec![
        name,
        words(&[0x0343_4300, 0x7374_7274]),
        words(&[0x0349_7354, 0]),
        words(&[0x0349_7374, TRUE]),
        words(&[0x0344_5200, 2]),
        words(&[0x0344_4100, 3, 0, 0x04e2_97b0]),
    ]
}

#[derive(Clone, Copy)]
pub enum PixelOrder {
    Rgb565,
    Bgr565,
}

impl PixelOrder {
    const fn encode(self, rgb: u16) -> [u8; 2] {
        let value = match self {
            Self::Rgb565 => rgb,
            Self::Bgr565 => ((rgb & 0x001f) << 11) | (rgb & 0x07e0) | ((rgb & 0xf800) >> 11),
        };
        value.to_be_bytes()
    }
}

pub fn display(index: u8, pixels: &[u16], order: PixelOrder, output: &mut Vec<u8>) {
    output.clear();
    output.extend_from_slice(&words(&[0x0364_7344, u32::from(index), 0, 0x0140_00f0, 0]));
    output.extend_from_slice(&[
        0x84, 0, index, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x40, 0, 0xf0,
    ]);
    let pairs: Vec<_> = pixels.chunks_exact(2).collect();
    let mut i = 0;
    while i < pairs.len() {
        let start = i;
        while i < pairs.len() && pairs[i] == pairs[start] {
            i += 1;
        }
        let run = i - start;
        if run >= 4 {
            let count = run.to_be_bytes();
            output.extend_from_slice(&[
                1,
                count[count.len() - 3],
                count[count.len() - 2],
                count[count.len() - 1],
            ]);
            for &rgb in pairs[start] {
                output.extend_from_slice(&order.encode(rgb));
            }
        } else {
            while i < pairs.len() {
                if i + 4 <= pairs.len() && (1..4).all(|n| pairs[i] == pairs[i + n]) {
                    break;
                }
                i += 1;
            }
            let count = (i - start).to_be_bytes();
            output.extend_from_slice(&[
                0,
                count[count.len() - 3],
                count[count.len() - 2],
                count[count.len() - 1],
            ]);
            for pair in &pairs[start..i] {
                for &rgb in *pair {
                    output.extend_from_slice(&order.encode(rgb));
                }
            }
        }
    }
    output.extend_from_slice(&[3, 0, 0, 0, 0x40, 0, 0, 0]);
    let len = (output.len() - 20).to_le_bytes();
    output[16..20].copy_from_slice(&len[..4]);
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
