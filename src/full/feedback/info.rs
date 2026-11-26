//! Numeric Djay state outputs used by screens and diagnostics.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub key_raw: [Option<u8>; 4],
    pub tempo_range: Option<u8>,
    pub tempo_position: [Option<u8>; 4],
    pub jump_revision: [u64; 4],
    jump: [[Option<bool>; 10]; 4],
    active_loop: [[Option<bool>; 22]; 4],
}

pub const JUMP_INTERVALS: [(&str, &str); 10] = [
    ("0125", "1/8"),
    ("025", "1/4"),
    ("05", "1/2"),
    ("1", "1"),
    ("2", "2"),
    ("4", "4"),
    ("8", "8"),
    ("16", "16"),
    ("32", "32"),
    ("64", "64"),
];

pub const LOOP_INTERVALS: [(&str, &str); 22] = [
    ("003125", "1/32"),
    ("00625", "1/16"),
    ("0125", "1/8"),
    ("025", "1/4"),
    ("05", "1/2"),
    ("1", "1"),
    ("2", "2"),
    ("4", "4"),
    ("8", "8"),
    ("16", "16"),
    ("32", "32"),
    ("64", "64"),
    ("128", "128"),
    ("033", "1/3"),
    ("075", "3/4"),
    ("1_5", "1.5"),
    ("3", "3"),
    ("6", "6"),
    ("9", "9"),
    ("12", "12"),
    ("5", "5"),
    ("7", "7"),
];

impl State {
    pub fn apply(&mut self, message: super::Message) -> bool {
        let channel = usize::from(message.status & 0x0f);
        if message.status & 0xf0 != 0xb0 {
            return false;
        }
        if channel < 4 {
            if (80..90).contains(&message.number) {
                self.jump_revision[channel] = self.jump_revision[channel].wrapping_add(1);
                let states = &mut self.jump[channel];
                let index = usize::from(message.number - 80);
                let old = states[index];
                states[index] = Some(message.value != 0);
                return old != states[index];
            }
            if (90..112).contains(&message.number) {
                let states = &mut self.active_loop[channel];
                let index = usize::from(message.number - 90);
                let old = states[index];
                states[index] = Some(message.value != 0);
                return old != states[index];
            }
            if message.number == 112 {
                let old = self.key_raw[channel];
                self.key_raw[channel] = Some(message.value);
                return old != self.key_raw[channel];
            }
            if message.number == crate::full::catalog::TEMPO_POSITION_CC {
                let old = self.tempo_position[channel].replace(message.value);
                return old != Some(message.value);
            }
        }
        if channel == 6 && message.number == 8 {
            let old = self.tempo_range;
            self.tempo_range = match message.value {
                5 => Some(4),
                8 => Some(6),
                10 => Some(8),
                13 => Some(10),
                20 => Some(16),
                32 => Some(25),
                64 => Some(50),
                95 => Some(75),
                _ => None,
            };
            return old != self.tempo_range;
        }
        false
    }

    pub fn jump_size(&self, deck: usize) -> Option<&'static str> {
        Some(JUMP_INTERVALS[self.jump_index(deck)?].1)
    }

    pub fn jump_index(&self, deck: usize) -> Option<usize> {
        let states = self.jump.get(deck)?;
        let mut active = states
            .iter()
            .enumerate()
            .filter(|(_, state)| **state == Some(true));
        let (selected, _) = active.next()?;
        if active.next().is_some() {
            return None;
        }
        Some(selected)
    }

    pub fn active_loop_size(&self, deck: usize) -> Option<&'static str> {
        let states = self.active_loop.get(deck)?;
        let mut active = states
            .iter()
            .enumerate()
            .filter(|(_, state)| **state == Some(true));
        let (selected, _) = active.next()?;
        if active.next().is_some() {
            return None;
        }
        Some(LOOP_INTERVALS[selected].1)
    }

    pub fn clear_deck(&mut self, deck: usize) {
        self.tempo_position[deck] = None;
        // displayKeyIndex is deck-owned: its new key (or zero on eject) can
        // arrive before the song's loadingSuccess=false notification.
        if let Some(states) = self.active_loop.get_mut(deck) {
            states.fill(None);
        }
    }
}
