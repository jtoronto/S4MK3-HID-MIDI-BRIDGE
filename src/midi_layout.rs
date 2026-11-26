//! One feedback address partition, shared by the generator and callback ingress.

use crate::full::feedback::Message;
use crate::midi_transport::Backend;

/// Channels are zero based. Unsupported wire addresses never reach the cache.
pub const fn normalize(backend: Backend, mut message: Message) -> Option<Message> {
    match backend {
        Backend::CoreMidi => Some(message),
        Backend::Loopbe | Backend::Bloop => {
            let channel = message.status & 0x0f;
            let family = message.status & 0xf0;
            let number = message.number;
            if let (12..=15, 0xb0, 51..=69) = (channel, family, number) {
                message.number = number - 19;
                return Some(message);
            }
            let canonical = match (channel, family, number) {
                (12..=15, 0x80 | 0x90, 0..=4 | 6 | 8 | 15..=20 | 32..=35 | 40..=63)
                | (12..=15, 0xb0, 21 | 22 | 32..=37 | 80..=113) => channel - 12,
                (7, 0x80 | 0x90, 0..=3) => 4,
                (6, 0x80 | 0x90, 0..=3) | (6, 0xb0, 0..=9) => 6,
                _ => return None,
            };
            message.status = family | canonical;
            if family == 0x80 {
                message.value = 0;
            }
            Some(message)
        }
    }
}

pub const fn feedback_channel(canonical: u8) -> Option<u8> {
    match canonical {
        0..=3 => Some(canonical + 12),
        4 => Some(7),
        6 => Some(6),
        _ => None,
    }
}
