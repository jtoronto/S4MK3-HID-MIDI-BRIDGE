//! Djay's per-slot Neural Mix routing command and feedback addresses.

use super::{Address, feedback::Message};

pub(super) const FIRST_CHANNEL: u8 = 12;
pub(super) const FIRST_ROUTE: u8 = 32;
pub(super) const ROUTES: usize = 5;
pub(super) const SLOTS: usize = 3;
pub(super) const FIRST_LAYOUT: u8 = 47;
pub(super) const WINDOWS_FIRST_COMMAND: u8 = 80;
pub(super) const WINDOWS_FIRST_FEEDBACK: u8 = 51;
pub(super) const VARIANTS: [(&str, &str); ROUTES] = [
    ("routingDeck", "isDeck"),
    ("routingGenericChannel1", "genericChannel1"),
    ("routingGenericChannel2", "genericChannel2"),
    ("routingGenericChannel3", "genericChannel3"),
    ("routingGenericChannel4", "genericChannel4"),
];
pub(super) const LAYOUTS: [&str; 4] = ["One", "Two", "Three", "Four"];

pub(super) const fn address(deck: u8, slot: u8, route: u8) -> Address {
    Address {
        channel: FIRST_CHANNEL + deck,
        number: FIRST_ROUTE + slot * 5 + route,
    }
}

pub(super) const fn windows_address(deck: u8, slot: u8, route: u8) -> Address {
    Address {
        channel: 8 + deck,
        number: WINDOWS_FIRST_COMMAND + slot * 5 + route,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Feedback {
    pub(super) routes: [[Option<bool>; ROUTES]; SLOTS],
    available: [Option<bool>; 4],
}

impl Feedback {
    pub(super) fn layout(self) -> Option<usize> {
        // Djay publishes availability flags, not an exclusive layout value.
        let count = self
            .available
            .into_iter()
            .flatten()
            .filter(|on| *on)
            .count();
        let expected = std::array::from_fn(|index| Some(index < count));
        (self.available == expected && count > 0).then_some(count)
    }

    pub(super) fn apply(&mut self, message: Message) -> Option<bool> {
        let field = if let Some(index) = message.number.checked_sub(FIRST_LAYOUT) {
            self.available.get_mut(usize::from(index))?
        } else {
            let index = usize::from(message.number.checked_sub(FIRST_ROUTE)?);
            self.routes
                .get_mut(index / ROUTES)?
                .get_mut(index % ROUTES)?
        };
        let value = Some(message.value != 0);
        Some(std::mem::replace(field, value) != value)
    }
}
