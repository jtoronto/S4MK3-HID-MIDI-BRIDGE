//! Full input translation, isolated from the verified minimal MIDI profile.

pub mod catalog;
pub mod feedback;
pub mod leds;
pub mod mapping;
pub mod output;

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context as _, bail, ensure};
use encdr::Event;
use midir::os::unix::VirtualOutput as _;
use midir::{MidiOutput, MidiOutputConnection};
use num_traits::ToPrimitive as _;
use tracing::{debug, info, warn};

use catalog::{Binding, Kind, PadMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Deck {
    A,
    B,
    C,
    D,
}

impl Deck {
    const fn channel(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Control {
    Button(&'static str, bool),
    Slider(&'static str, f32),
    Encoder(&'static str, i32),
    Jog(&'static str, f64),
}

impl Control {
    fn from_event(event: &Event) -> Option<Self> {
        match event {
            Event::Button { name, pressed, .. } => Some(Self::Button(name, *pressed)),
            Event::Touch { name, touched, .. } => Some(Self::Button(name, *touched)),
            Event::Slider { name, value, .. } => Some(Self::Slider(name, *value)),
            Event::Encoder { name, delta, .. } => Some(Self::Encoder(name, *delta)),
            Event::EncoderFine { name, delta, .. } => Some(Self::Jog(name, f64::from(*delta))),
            Event::DeviceConnected { .. }
            | Event::DeviceDisconnected { .. }
            | Event::Grid { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Address {
    channel: u8,
    number: u8,
}

impl Address {
    const fn press(self) -> [u8; 3] {
        [0x90 | self.channel, self.number, 127]
    }

    const fn release(self) -> [u8; 3] {
        [0x80 | self.channel, self.number, 0]
    }
}

#[derive(Clone, Copy, Debug)]
struct WheelOwner {
    deck: Deck,
    action: &'static str,
}

struct Translator {
    decks: [Deck; 2],
    shifted: [bool; 2],
    pads: [PadMode; 2],
    fx: [Deck; 2],
    quick: Deck,
    vinyl: [bool; 2],
    grid: [bool; 2],
    wheels: [Option<WheelOwner>; 2],
    residuals: [f64; 2],
    held: HashMap<&'static str, Vec<Address>>,
    switches: [Option<u8>; 5],
    delivered_switches: [Option<u8>; 5],
    front_panel_ready: bool,
    led_held: [bool; 11],
    tempos: [Option<f32>; 2],
    move_selecting: [bool; 4],
}

impl Default for Translator {
    fn default() -> Self {
        Self {
            decks: [Deck::A, Deck::B],
            shifted: [false; 2],
            pads: [PadMode::Hotcue; 2],
            fx: [Deck::A, Deck::B],
            quick: Deck::A,
            vinyl: [true; 2],
            grid: [false; 2],
            wheels: [None; 2],
            residuals: [0.0; 2],
            held: HashMap::new(),
            switches: [None; 5],
            delivered_switches: [None; 5],
            front_panel_ready: false,
            led_held: [false; 11],
            tempos: [None; 2],
            move_selecting: [false; 4],
        }
    }
}

pub struct FullBridge {
    feedback_input: feedback::FeedbackInput,
    feedback: feedback::FeedbackState,
    connection: MidiOutputConnection,
    translator: Translator,
    controls: Vec<Control>,
    messages: Vec<[u8; 3]>,
}

impl FullBridge {
    pub fn new() -> anyhow::Result<Self> {
        // Djay can return initial state as soon as the source appears.
        let feedback_input = feedback::FeedbackInput::new(mapping::PORT_NAME)?;
        let connection = MidiOutput::new("S4 MK3 full bridge")?
            .create_virtual(mapping::PORT_NAME)
            .map_err(|error| anyhow::anyhow!("create full MIDI source: {error}"))?;
        info!(port = mapping::PORT_NAME, "MIDI_FULL_READY");
        Ok(Self {
            feedback_input,
            feedback: feedback::FeedbackState::default(),
            connection,
            translator: Translator::default(),
            controls: Vec::with_capacity(160),
            messages: Vec::with_capacity(160),
        })
    }

    pub fn drain_feedback(&mut self, now: Duration) -> anyhow::Result<(usize, usize)> {
        let received = self.feedback_input.drain(&mut self.feedback, now)?;
        self.messages.clear();
        self.translator
            .sync_front_panel(&self.feedback, &mut self.messages)?;
        let sent = self.send_messages()?;
        Ok((received, sent))
    }

    pub const fn feedback_state(&self) -> &feedback::FeedbackState {
        &self.feedback
    }

    pub fn local_led_state(&self) -> leds::LocalState {
        self.translator.local_led_state()
    }

    pub fn send_report(
        &mut self,
        events: &[Event],
        buttons: Option<&[u8; 22]>,
    ) -> anyhow::Result<usize> {
        self.controls.clear();
        self.controls
            .extend(events.iter().filter_map(Control::from_event));
        self.messages.clear();
        self.translator
            .translate(&self.controls, buttons, &mut self.messages)?;
        self.send_messages()
    }

    fn send_messages(&mut self) -> anyhow::Result<usize> {
        for message in &self.messages {
            self.connection
                .send(message)
                .context("send full-profile MIDI")?;
        }
        if !self.messages.is_empty() {
            debug!(messages = self.messages.len(), "MIDI_FULL_SENT");
        }
        Ok(self.messages.len())
    }
}

impl Drop for FullBridge {
    fn drop(&mut self) {
        self.messages.clear();
        self.translator.release_all(&mut self.messages);
        for message in &self.messages {
            if let Err(error) = self.connection.send(message) {
                warn!(%error, "could not release full-profile MIDI note");
            }
        }
    }
}

impl Translator {
    fn local_led_state(&self) -> leds::LocalState {
        leds::LocalState {
            decks: self.decks.map(|deck| usize::from(deck.channel())),
            shifted: self.shifted,
            pads: self.pads,
            fx: self.fx.map(|deck| usize::from(deck.channel())),
            quick: usize::from(self.quick.channel()),
            vinyl: self.vinyl,
            grid: self.grid,
            held: self.led_held,
            tempo: self.tempos,
            move_selecting: self.move_selecting,
        }
    }

    fn observe_led_inputs(&mut self, controls: &[Control]) {
        const MOMENTARY: [&str; 11] = [
            "left_record_mode",
            "left_library_view",
            "left_library_playlist",
            "left_library_star",
            "left_library_play",
            "right_record_mode",
            "right_library_view",
            "right_library_playlist",
            "right_library_star",
            "right_library_play",
            "mixer_fx_filter",
        ];
        for control in controls {
            match *control {
                Control::Button(name, held) => {
                    if let Some(index) = MOMENTARY.iter().position(|expected| *expected == name) {
                        self.led_held[index] = held;
                    }
                }
                Control::Slider("left_tempo_fader", value) => self.tempos[0] = Some(value),
                Control::Slider("right_tempo_fader", value) => self.tempos[1] = Some(value),
                Control::Slider(_, _) | Control::Encoder(_, _) | Control::Jog(_, _) => {}
            }
        }
    }

    fn translate(
        &mut self,
        controls: &[Control],
        buttons: Option<&[u8; 22]>,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<()> {
        self.observe_led_inputs(controls);
        // All releases precede selectors; selectors precede new presses/movement.
        // Thus descriptor item order cannot choose a simultaneous Shift+Play route.
        for control in controls {
            if let Control::Button(name, false) = control {
                self.release(name, output);
            }
        }
        self.selectors(controls, output);
        self.move_selection(controls);
        if let Some(payload) = buttons {
            self.front_panel(payload, output)?;
        }
        for control in controls {
            match *control {
                Control::Button(name, true) => self.press(name, output)?,
                Control::Button(_, false) => {}
                Control::Slider(name, value) => {
                    let routes = self.routes(name)?;
                    if routes.is_empty() {
                        continue;
                    }
                    ensure!(
                        (0.0..=1.0).contains(&value),
                        "invalid full-profile {name}: {value}"
                    );
                    let value = (value * 127.0)
                        .round()
                        .to_u8()
                        .context("quantize MIDI CC")?;
                    for (binding, channel) in routes {
                        if binding.kind == Kind::Absolute {
                            output.push([0xb0 | channel, binding.number, value]);
                        }
                    }
                }
                Control::Encoder(name, delta) => {
                    self.encoder(name, delta, output)?;
                }
                Control::Jog(name, delta) => self.jog(name, delta, output)?,
            }
        }
        Ok(())
    }

    fn move_selection(&mut self, controls: &[Control]) {
        for (side, name) in [
            (0, "left_move_encoder_press"),
            (1, "right_move_encoder_press"),
        ] {
            if pressed(controls, name) && !self.held.contains_key(name) {
                let deck = usize::from(self.decks[side].channel());
                self.move_selecting[deck] = !self.move_selecting[deck];
                // Own the physical press, even though it emits no MIDI note.
                self.held.insert(name, Vec::new());
                info!(
                    deck = deck + 1,
                    selecting = self.move_selecting[deck],
                    "MOVE_SIZE_SELECT"
                );
            }
        }
    }

    fn encoder(&self, name: &str, delta: i32, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        if let Some((side, "move_encoder")) = side_name(name)
            && self.shifted[side]
        {
            let action = if delta < 0 {
                "move_one_backward"
            } else {
                "move_one_forward"
            };
            let binding = catalog::DECK_BINDINGS
                .iter()
                .find(|binding| binding.input == action)
                .context("one-beat MOVE binding missing")?;
            for _ in 0..delta.unsigned_abs() {
                pulse(
                    Address {
                        channel: channel(*binding, self.decks[side], true),
                        number: binding.number,
                    },
                    output,
                );
            }
            return Ok(());
        }
        for (binding, channel) in self.routes(name)? {
            if binding.kind == Kind::Relative {
                relative(
                    Address {
                        channel,
                        number: binding.number,
                    },
                    delta,
                    output,
                )?;
            }
        }
        Ok(())
    }

    fn selectors(&mut self, controls: &[Control], output: &mut Vec<[u8; 3]>) {
        for (side, prefix) in [(0, "left"), (1, "right")] {
            let old_shift = self.shifted[side];
            let old_grid = self.grid[side];
            for control in controls {
                if let Control::Button(name, pressed) = *control {
                    if name
                        == if side == 0 {
                            "left_shift"
                        } else {
                            "right_shift"
                        }
                    {
                        self.shifted[side] = pressed;
                    }
                    if name == if side == 0 { "left_grid" } else { "right_grid" } {
                        self.grid[side] = pressed;
                    }
                }
            }
            if old_grid != self.grid[side]
                || (old_shift != self.shifted[side] && self.wheels[side].is_none())
            {
                self.residuals[side] = 0.0;
            }
            for (name, deck) in [
                ("left_deck_switch_a", Deck::A),
                ("left_deck_switch_c", Deck::C),
                ("right_deck_switch_b", Deck::B),
                ("right_deck_switch_d", Deck::D),
            ] {
                if name.starts_with(prefix) && pressed(controls, name) {
                    self.decks[side] = deck;
                    if self.wheels[side].is_none() {
                        self.residuals[side] = 0.0;
                    }
                    let binding = catalog::deck_selection_binding(deck.channel().saturating_add(1));
                    if let Some(binding) = binding {
                        pulse(
                            Address {
                                channel: 5,
                                number: binding.number,
                            },
                            output,
                        );
                    }
                }
            }
            for (suffix, mode) in [
                ("hotcue_mode", PadMode::Hotcue),
                ("samples_mode", PadMode::Samples),
                ("stems_mode", PadMode::Stems),
                ("mute_mode", PadMode::Stems),
            ] {
                if controls.iter().any(|input| {
                    matches!(input, Control::Button(name, true)
                    if name.strip_prefix(prefix).and_then(|s| s.strip_prefix('_')) == Some(suffix))
                }) {
                    self.pads[side] = mode;
                }
            }
            if pressed(
                controls,
                if side == 0 {
                    "left_jog_mode"
                } else {
                    "right_jog_mode"
                },
            ) {
                self.vinyl[side] = false;
            }
            if pressed(
                controls,
                if side == 0 {
                    "left_turntable_mode"
                } else {
                    "right_turntable_mode"
                },
            ) {
                self.vinyl[side] = true;
            }
            self.wheel_touch(side, controls);
        }
        self.fx_targets(controls);
    }

    fn wheel_touch(&mut self, side: usize, controls: &[Control]) {
        let touch = if side == 0 {
            "left_wheel_touch"
        } else {
            "right_wheel_touch"
        };
        for input in controls {
            if let Control::Button(name, touched) = *input
                && name == touch
            {
                self.residuals[side] = 0.0;
                self.wheels[side] = touched.then_some(WheelOwner {
                    deck: self.decks[side],
                    action: if self.shifted[side] {
                        "jog_seek"
                    } else if self.vinyl[side] {
                        "jog_scratch"
                    } else {
                        "jog_bend"
                    },
                });
            }
        }
    }

    fn fx_targets(&mut self, controls: &[Control]) {
        for (deck, fx1, fx2, quick) in [
            (
                Deck::A,
                "mixer_ch1_fx1",
                "mixer_ch1_fx2",
                "mixer_ch1_fx_select",
            ),
            (
                Deck::B,
                "mixer_ch2_fx1",
                "mixer_ch2_fx2",
                "mixer_ch2_fx_select",
            ),
            (
                Deck::C,
                "mixer_ch3_fx1",
                "mixer_ch3_fx2",
                "mixer_ch3_fx_select",
            ),
            (
                Deck::D,
                "mixer_ch4_fx1",
                "mixer_ch4_fx2",
                "mixer_ch4_fx_select",
            ),
        ] {
            if pressed(controls, fx1) {
                self.fx[0] = deck;
            }
            if pressed(controls, fx2) {
                self.fx[1] = deck;
            }
            if pressed(controls, quick) {
                self.quick = deck;
            }
        }
    }

    fn routes(&self, name: &str) -> anyhow::Result<Vec<(Binding, u8)>> {
        let mut routes = Vec::new();
        if let Some((side, suffix)) = side_name(name) {
            let deck = self.decks[side];
            let suffix = if suffix == "move_encoder"
                && !self.shifted[side]
                && self.move_selecting[usize::from(deck.channel())]
            {
                "move_size"
            } else {
                suffix
            };
            if let Some(index) = suffix.strip_prefix("pad_") {
                let index = index
                    .parse::<u8>()?
                    .checked_sub(1)
                    .context("pad numbering")?;
                let binding = catalog::pad_binding(self.pads[side], index)
                    .context("pad index outside 1-8")?;
                routes.push((binding, channel(binding, deck, self.shifted[side])));
            } else {
                let fx = if side == 0 {
                    catalog::FX_LEFT_BINDINGS
                } else {
                    catalog::FX_RIGHT_BINDINGS
                };
                for binding in fx.iter().filter(|binding| binding.input == suffix) {
                    routes.push((
                        *binding,
                        channel(*binding, self.fx[side], self.shifted[side]),
                    ));
                }
                for binding in catalog::DECK_BINDINGS
                    .iter()
                    .filter(|binding| binding.input == suffix)
                {
                    if suffix != "wheel_touch" || self.vinyl[side] || self.shifted[side] {
                        routes.push((*binding, channel(*binding, deck, self.shifted[side])));
                    }
                }
            }
        } else if let Some((strip, suffix)) = name
            .strip_prefix("fx1_")
            .map(|s| (0, s))
            .or_else(|| name.strip_prefix("fx2_").map(|s| (1, s)))
        {
            let bindings = if strip == 0 {
                catalog::FX_LEFT_BINDINGS
            } else {
                catalog::FX_RIGHT_BINDINGS
            };
            for binding in bindings.iter().filter(|binding| binding.input == suffix) {
                routes.push((
                    *binding,
                    channel(*binding, self.fx[strip], self.shifted[strip]),
                ));
            }
        } else {
            for binding in catalog::MIXER_BINDINGS
                .iter()
                .filter(|binding| binding.input == name)
            {
                routes.push((*binding, 4));
            }
            for binding in catalog::QUICK_BINDINGS
                .iter()
                .filter(|binding| binding.input == name)
            {
                routes.push((*binding, self.quick.channel()));
            }
            if name == "mixer_quantize" {
                for binding in catalog::DECK_BINDINGS
                    .iter()
                    .filter(|binding| binding.input == "quantize")
                {
                    for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
                        routes.push((*binding, deck.channel()));
                    }
                }
            }
        }
        Ok(routes)
    }

    fn press(&mut self, name: &'static str, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        if self.held.contains_key(name) {
            return Ok(());
        }
        let addresses: Vec<_> = self
            .routes(name)?
            .into_iter()
            .filter(|(binding, _)| binding.kind == Kind::Note)
            .map(|(binding, channel)| Address {
                channel,
                number: binding.number,
            })
            .collect();
        for address in &addresses {
            if !self.held.values().flatten().any(|owned| owned == address) {
                output.push(address.press());
            }
        }
        if !addresses.is_empty() {
            self.held.insert(name, addresses);
        }
        Ok(())
    }

    fn release(&mut self, name: &str, output: &mut Vec<[u8; 3]>) {
        if let Some(addresses) = self.held.remove(name) {
            for address in addresses {
                if !self.held.values().flatten().any(|owned| *owned == address) {
                    output.push(address.release());
                }
            }
        }
    }

    fn release_all(&mut self, output: &mut Vec<[u8; 3]>) {
        let mut addresses: Vec<_> = self
            .held
            .drain()
            .flat_map(|(_, addresses)| addresses)
            .collect();
        addresses.sort_unstable_by_key(|address| (address.channel, address.number));
        addresses.dedup();
        output.extend(addresses.into_iter().map(Address::release));
    }

    fn jog(&mut self, name: &str, delta: f64, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        let (side, _) = side_name(name).context("unknown jog side")?;
        ensure!(delta.is_finite(), "non-finite jog movement");
        let owner = self.wheels[side].unwrap_or(WheelOwner {
            deck: self.decks[side],
            action: if self.shifted[side] {
                "jog_seek"
            } else {
                "jog_bend"
            },
        });
        // Encdr divides integer S4 counter deltas by 1000 into f32.
        // Recover those integers before truncation can defer a count to a
        // later report (or cancel it when the direction reverses).
        let counts = (delta * 1000.0).round();
        let scale = if self.grid[side] { 0.125 } else { 1.0 };
        let accumulated = counts.mul_add(scale, self.residuals[side]);
        let steps = accumulated
            .trunc()
            .to_i32()
            .context("jog delta outside MIDI translation range")?;
        self.residuals[side] = accumulated - f64::from(steps);
        if self.grid[side] {
            let binding = catalog::DECK_BINDINGS
                .iter()
                .find(|binding| binding.input == if steps < 0 { "grid_left" } else { "grid_right" })
                .context("grid binding missing")?;
            for _ in 0..steps.unsigned_abs() {
                pulse(
                    Address {
                        channel: owner.deck.channel(),
                        number: binding.number,
                    },
                    output,
                );
            }
        } else {
            let binding = catalog::DECK_BINDINGS
                .iter()
                .find(|binding| binding.input == owner.action)
                .context("jog binding missing")?;
            relative(
                Address {
                    channel: owner.deck.channel(),
                    number: binding.number,
                },
                steps,
                output,
            )?;
        }
        Ok(())
    }

    fn front_panel(&mut self, payload: &[u8; 22], output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        for (deck, shift) in [(Deck::A, 4), (Deck::B, 2), (Deck::C, 6), (Deck::D, 0)] {
            let value = (payload[17] >> shift) & 3;
            ensure!(value <= 2, "invalid crossfader assignment state {value}");
            let index = usize::from(deck.channel());
            self.switches[index] = Some(value);
        }
        let curve = payload[18] & 3;
        match curve {
            0..=2 => {}
            _ => bail!("invalid crossfader curve state {curve}"),
        }
        self.switches[4] = Some(curve);
        self.emit_front_panel(output)
    }

    fn sync_front_panel(
        &mut self,
        feedback: &feedback::FeedbackState,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<()> {
        if !self.front_panel_ready && feedback.playback_received {
            self.front_panel_ready = true;
            info!(
                "CROSSFADER_FEEDBACK_RECEIVED: applying physical switches once; restart bridge after mapping reconnect"
            );
        }
        self.emit_front_panel(output)
    }

    fn emit_front_panel(&mut self, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        if !self.front_panel_ready {
            return Ok(());
        }
        for index in 0..5 {
            if let Some(value) = self.switches[index]
                && self.delivered_switches[index] != Some(value)
            {
                if index < 4 {
                    let deck = u8::try_from(index)?.saturating_add(1);
                    let number = catalog::assignment_binding(deck, value)
                        .context("crossfader assignment binding missing")?
                        .number;
                    pulse(Address { channel: 4, number }, output);
                } else {
                    let value = [127, 64, 0][usize::from(value)];
                    output.push([0xb4, catalog::CURVE_BINDING.number, value]);
                }
                self.delivered_switches[index] = Some(value);
            }
        }
        Ok(())
    }
}

const fn channel(binding: Binding, deck: Deck, shifted: bool) -> u8 {
    if shifted && binding.shifted.is_some() {
        deck.channel().saturating_add(8)
    } else {
        deck.channel()
    }
}

fn side_name(name: &str) -> Option<(usize, &str)> {
    name.strip_prefix("left_")
        .map(|name| (0, name))
        .or_else(|| name.strip_prefix("right_").map(|name| (1, name)))
}

fn pressed(controls: &[Control], name: &str) -> bool {
    controls
        .iter()
        .any(|control| matches!(control, Control::Button(input, true) if *input == name))
}

fn pulse(address: Address, output: &mut Vec<[u8; 3]>) {
    output.push(address.press());
    output.push(address.release());
}

fn relative(address: Address, mut steps: i32, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
    while steps != 0 {
        let chunk = steps.clamp(-63, 63);
        let value = u8::try_from(64_i32.saturating_add(chunk))?;
        output.push([0xb0 | address.channel, address.number, value]);
        steps = steps.saturating_sub(chunk);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(state: &mut Translator, controls: &[Control]) -> anyhow::Result<Vec<[u8; 3]>> {
        let mut output = Vec::new();
        state.translate(controls, None, &mut output)?;
        Ok(output)
    }

    #[test]
    fn physical_routes_exist_in_the_generated_mapping_on_every_layer() -> anyhow::Result<()> {
        // Given actual physical names from the pinned, corrected descriptor.
        use encdr::core::descriptor::InputItemDesc;
        use encdr::device::loader::DescriptorRegistry;
        let mut registry = DescriptorRegistry::new();
        registry.load_builtins()?;
        let original = registry
            .find(crate::S4_VENDOR_ID, crate::S4_PRODUCT_ID)
            .context("S4 descriptor missing")?;
        let descriptor = crate::full_control_descriptor(original)?;
        let mapping = mapping::generate()?;
        let controls = mapping
            .as_dictionary()
            .and_then(|root| root.get("controls"))
            .and_then(plist::Value::as_array)
            .context("mapping controls missing")?;
        // When each deck, modifier and pad layer resolves these physical names.
        for (left, right) in [(Deck::A, Deck::B), (Deck::C, Deck::D)] {
            for shifted in [false, true] {
                for mode in [PadMode::Hotcue, PadMode::Samples, PadMode::Stems] {
                    let state = Translator {
                        decks: [left, right],
                        fx: [left, right],
                        quick: left,
                        shifted: [shifted; 2],
                        pads: [mode; 2],
                        ..Translator::default()
                    };
                    for item in descriptor
                        .input_packets
                        .iter()
                        .flat_map(|packet| &packet.items)
                    {
                        let name = match item {
                            InputItemDesc::Button(item) => &item.name,
                            InputItemDesc::Slider(item) => &item.name,
                            InputItemDesc::Encoder(item) => &item.name,
                            InputItemDesc::EncoderFine(item) => &item.name,
                            InputItemDesc::Touch(item) => &item.name,
                        };
                        for (binding, channel) in state.routes(name)? {
                            let kind = if binding.kind == Kind::Note { 1 } else { 3 };
                            // Then the generator contains the exact runtime address.
                            assert!(
                                controls.iter().any(|control| {
                                    let Some(row) = control.as_dictionary() else {
                                        return false;
                                    };
                                    row.get("midiChannel")
                                        .and_then(plist::Value::as_unsigned_integer)
                                        == Some(u64::from(channel))
                                        && row
                                            .get("midiData")
                                            .and_then(plist::Value::as_unsigned_integer)
                                            == Some(u64::from(binding.number))
                                        && row
                                            .get("midiMessageType")
                                            .and_then(plist::Value::as_unsigned_integer)
                                            == Some(kind)
                                }),
                                "unmapped runtime address for {name}"
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn captured_encoders_route_rotation_and_press_to_the_physical_knob() -> anyhow::Result<()> {
        use encdr::core::descriptor::InputItemDesc;
        use encdr::device::encoder::EncoderState;
        use encdr::device::loader::DescriptorRegistry;

        // Given independently captured physical fields, not descriptor names.
        for shifted in [false, true] {
            for (byte, increment, press_byte, mask, cc, note, deck) in [
                (19, 1, 6, 0x04, 2, 5, 0),   // Left MOVE
                (19, 16, 6, 0x20, 1, 6, 0),  // Left LOOP
                (20, 16, 15, 0x20, 2, 5, 1), // Right MOVE
                (21, 1, 15, 0x04, 1, 6, 1),  // Right LOOP
            ] {
                let mut registry = DescriptorRegistry::new();
                registry.load_builtins()?;
                let original = registry
                    .find(crate::S4_VENDOR_ID, crate::S4_PRODUCT_ID)
                    .context("S4 descriptor missing")?;
                let descriptor = crate::full_control_descriptor(original)?;
                let names = registry.intern_descriptor_names(&descriptor);
                let encoder = descriptor
                    .input_packets
                    .iter()
                    .flat_map(|packet| &packet.items)
                    .find_map(|item| match item {
                        InputItemDesc::Encoder(item)
                            if item.byte == byte
                                && item.bit_offset == if increment == 1 { 0 } else { 4 } =>
                        {
                            Some(item)
                        }
                        _ => None,
                    })
                    .context("captured encoder field missing")?;
                let encoder_name = *names.get(&encoder.name).context("encoder name missing")?;
                let press = descriptor
                    .input_packets
                    .iter()
                    .flat_map(|packet| &packet.items)
                    .find_map(|item| match item {
                        InputItemDesc::Button(item)
                            if item.byte == press_byte && item.mask.0 == u16::from(mask) =>
                        {
                            Some(item)
                        }
                        _ => None,
                    })
                    .context("captured press field missing")?;
                let press_name = *names.get(&press.name).context("press name missing")?;
                let mut state = Translator::default();
                let mut payload = [0_u8; 22];
                if shifted {
                    let shift = if deck == 0 {
                        "left_shift"
                    } else {
                        "right_shift"
                    };
                    state.translate(&[Control::Button(shift, true)], None, &mut Vec::new())?;
                }
                let mut decoder = EncoderState::default();
                assert_eq!(
                    decoder.update_wrap16((payload[byte] >> encoder.bit_offset) & 15),
                    None
                );

                // When the knob moves forward/backward and is pressed/released.
                let mut output = Vec::new();
                for action in 0..4 {
                    match action {
                        0 => payload[byte] = payload[byte].wrapping_add(increment),
                        1 => payload[byte] = payload[byte].wrapping_sub(increment),
                        2 => payload[press_byte] |= mask,
                        _ => payload[press_byte] &= !mask,
                    }
                    let control = if action < 2 {
                        Control::Encoder(
                            encoder_name,
                            decoder
                                .update_wrap16((payload[byte] >> encoder.bit_offset) & 15)
                                .context("captured rotation produced no delta")?,
                        )
                    } else {
                        Control::Button(press_name, payload[press_byte] & mask != 0)
                    };
                    state.translate(&[control], None, &mut output)?;
                }

                // Then the actual MIDI destinations retain physical semantics.
                assert_eq!(
                    output,
                    captured_encoder_midi(cc, note, deck, shifted),
                    "physical field {byte} increment {increment}, shifted={shifted}"
                );
                assert_eq!(state.move_selecting[usize::from(deck)], note == 5);
            }
        }
        Ok(())
    }

    #[test]
    fn normalized_hid_jog_counts_are_not_delayed_by_float_rounding() -> anyhow::Result<()> {
        // Given integer counter deltas normalized to f32 by Encdr.
        let mut state = Translator::default();
        for counts in [5_i16, -5, 9, -9, 63, -63] {
            let delta = f64::from(f32::from(counts) / 1000.0);
            // When slow motion reverses direction.
            let output = run(&mut state, &[Control::Jog("left_jog_wheel", delta)])?;
            // Then every report retains its exact signed counter movement.
            let decoded: i32 = output
                .iter()
                .map(|message| i32::from(message[2]) - 64)
                .sum();
            assert_eq!(decoded, i32::from(counts));
        }
        Ok(())
    }

    #[test]
    fn cue_release_keeps_the_deck_that_owned_its_press() -> anyhow::Result<()> {
        // Given Cue held on deck A.
        let mut state = Translator::default();
        assert_eq!(
            run(&mut state, &[Control::Button("left_cue", true)])?,
            [[0x90, 1, 127]]
        );
        run(
            &mut state,
            &[
                Control::Button("left_deck_switch_c", true),
                Control::Button("left_shift", true),
            ],
        )?;
        // When the physical Cue button is released after changing deck/layer.
        let output = run(&mut state, &[Control::Button("left_cue", false)])?;
        // Then the old deck receives the release, not deck C.
        assert_eq!(output, [[0x80, 1, 0]]);
        Ok(())
    }

    #[test]
    fn simultaneous_shift_and_pad_press_do_not_depend_on_descriptor_order() -> anyhow::Result<()> {
        // Given equivalent HID reports with opposite event ordering.
        for inputs in [
            [
                Control::Button("left_pad_1", true),
                Control::Button("left_shift", true),
            ],
            [
                Control::Button("left_shift", true),
                Control::Button("left_pad_1", true),
            ],
        ] {
            let mut state = Translator::default();
            // When the entire report is translated.
            let output = run(&mut state, &inputs)?;
            // Then the shifted hotcue action always gets the press.
            assert_eq!(output, [[0x98, 40, 127]]);
        }
        Ok(())
    }

    #[test]
    fn held_pad_releases_its_original_mode_before_new_strikes_use_samples() -> anyhow::Result<()> {
        // Given a hotcue pad held while the mode changes.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_pad_2", true)])?;
        run(&mut state, &[Control::Button("left_samples_mode", true)])?;
        // When that pad is released.
        let output = run(&mut state, &[Control::Button("left_pad_2", false)])?;
        // Then only its original hotcue note is released.
        assert_eq!(output, [[0x80, 41, 0]]);
        Ok(())
    }

    #[test]
    fn new_pad_strikes_use_the_selected_pad_mode() -> anyhow::Result<()> {
        // Given Samples mode.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_samples_mode", true)])?;
        // When pad 2 is struck.
        let output = run(&mut state, &[Control::Button("left_pad_2", true)])?;
        // Then its sample binding is emitted.
        assert_eq!(output, [[0x90, 49, 127]]);
        Ok(())
    }

    #[test]
    fn fx_target_changes_do_not_strand_held_effect_buttons() -> anyhow::Result<()> {
        // Given the left FX1 button held for deck A.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_fx_1", true)])?;
        run(&mut state, &[Control::Button("mixer_ch3_fx1", true)])?;
        // When FX1 releases and FX2 presses after selecting deck C.
        let output = run(
            &mut state,
            &[
                Control::Button("left_fx_1", false),
                Control::Button("left_fx_2", true),
            ],
        )?;
        // Then old ownership releases on A, while the new press targets C.
        assert_eq!(output, [[0x80, 33, 0], [0x92, 34, 127]]);
        Ok(())
    }

    #[test]
    fn touched_jog_keeps_its_owner_through_deck_selection() -> anyhow::Result<()> {
        // Given a touched wheel on A followed by selection of C.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_wheel_touch", true)])?;
        run(&mut state, &[Control::Button("left_deck_switch_c", true)])?;
        // When the touched wheel moves.
        let output = run(&mut state, &[Control::Jog("left_jog_wheel", 0.002)])?;
        // Then scratch movement remains on its touch owner A.
        assert_eq!(output, [[0xb0, 5, 66]]);
        Ok(())
    }

    #[test]
    fn fractional_grid_motion_is_preserved_without_timing_assumptions() -> anyhow::Result<()> {
        // Given three reports smaller than the eight-count grid step.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_grid", true)])?;
        run(&mut state, &[Control::Jog("left_jog_wheel", 0.003)])?;
        run(&mut state, &[Control::Jog("left_jog_wheel", 0.003)])?;
        // When the accumulated motion passes one step.
        let output = run(&mut state, &[Control::Jog("left_jog_wheel", 0.003)])?;
        // Then one grid pulse is emitted and the remainder retained.
        assert_eq!(output, [[0x90, 30, 127], [0x80, 30, 0]]);
        assert!((state.residuals[0] - 0.125).abs() < 1e-9);
        Ok(())
    }

    fn captured_encoder_midi(cc: u8, note: u8, deck: u8, shifted: bool) -> Vec<[u8; 3]> {
        let channel = if shifted { deck + 8 } else { deck };
        match (note, shifted) {
            (5, false) => vec![[0xb0 | deck, cc, 65], [0xb0 | deck, cc, 63]],
            (5, true) => vec![
                [0x90 | channel, 5, 127],
                [0x80 | channel, 5, 0],
                [0x90 | channel, 14, 127],
                [0x80 | channel, 14, 0],
            ],
            _ => vec![
                [0xb0 | channel, cc, 65],
                [0xb0 | channel, cc, 63],
                [0x90 | channel, note, 127],
                [0x80 | channel, note, 0],
            ],
        }
    }

    #[test]
    fn move_selection_is_per_deck_and_duplicate_presses_do_not_toggle() -> anyhow::Result<()> {
        // Given A selecting size, including a duplicate press and a deck switch while held.
        let mut state = Translator::default();
        assert!(
            run(
                &mut state,
                &[Control::Button("left_move_encoder_press", true)]
            )?
            .is_empty()
        );
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", true)],
        )?;
        run(&mut state, &[Control::Button("left_deck_switch_c", true)])?;
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", false)],
        )?;
        assert_eq!(
            run(&mut state, &[Control::Encoder("left_move_encoder", 1)])?,
            [[0xb2, 2, 65]]
        );
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", true)],
        )?;
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", false)],
        )?;
        run(&mut state, &[Control::Button("left_deck_switch_a", true)])?;
        // When A's knob turns again, its selection mode is retained without a timeout.
        assert_eq!(
            run(&mut state, &[Control::Encoder("left_move_encoder", -2)])?,
            [[0xb0, 7, 62]]
        );
        assert_eq!(
            state.local_led_state().move_selecting,
            [true, false, true, false]
        );
        // Then finishing selection restores jumping, independently of C.
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", true)],
        )?;
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", false)],
        )?;
        assert_eq!(
            run(&mut state, &[Control::Encoder("left_move_encoder", 1)])?,
            [[0xb0, 2, 65]]
        );
        assert!(state.move_selecting[2]);
        Ok(())
    }

    #[test]
    fn shifted_move_preserves_selection_and_emits_one_beat_per_detent() -> anyhow::Result<()> {
        for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
            for selecting in [false, true] {
                // Given the deck on the left with Shift held, in either local mode.
                let mut state = Translator {
                    decks: [deck, Deck::B],
                    shifted: [true, false],
                    ..Translator::default()
                };
                state.move_selecting[usize::from(deck.channel())] = selecting;
                // When both directions turn through multiple detents.
                for (delta, note) in [(2_i32, 5), (-3, 14), (0, 5)] {
                    let output = run(&mut state, &[Control::Encoder("left_move_encoder", delta)])?;
                    // Then fixed-size pulses preserve every detent, without size-changing CCs.
                    let address = Address {
                        channel: deck.channel() + 8,
                        number: note,
                    };
                    assert_eq!(
                        output,
                        [address.press(), address.release()]
                            .repeat(usize::try_from(delta.unsigned_abs())?)
                    );
                }
                assert_eq!(state.move_selecting[usize::from(deck.channel())], selecting);
            }
        }
        Ok(())
    }

    #[test]
    fn move_selection_precedes_rotation_regardless_of_descriptor_order() -> anyhow::Result<()> {
        // Given a size-mode toggle simultaneous with rotation and a right-side D selector.
        let mut state = Translator::default();
        let output = run(
            &mut state,
            &[
                Control::Encoder("right_move_encoder", 1),
                Control::Button("right_move_encoder_press", true),
                Control::Button("right_deck_switch_d", true),
            ],
        )?;
        // Then the movement changes D's size, never jumps B or D.
        assert!(output.contains(&[0xb3, 7, 65]));
        assert!(!output.iter().any(|m| m[0] & 0xf0 == 0xb0 && m[1] == 2));
        assert_eq!(state.move_selecting, [false, false, false, true]);
        Ok(())
    }

    #[test]
    fn changing_untouched_jog_mode_does_not_leak_old_fractional_motion() -> anyhow::Result<()> {
        // Given an incomplete grid step.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_grid", true)])?;
        run(&mut state, &[Control::Jog("left_jog_wheel", 0.007)])?;
        run(
            &mut state,
            &[
                Control::Button("left_grid", false),
                Control::Button("left_shift", true),
            ],
        )?;
        // When a small movement arrives in seek mode.
        let output = run(&mut state, &[Control::Jog("left_jog_wheel", 0.001)])?;
        // Then only the new count is sent, with no old grid remainder.
        assert_eq!(output, [[0xb0, 6, 65]]);
        assert!(state.residuals[0].abs() < f64::EPSILON);
        Ok(())
    }

    #[test]
    fn relative_encoding_preserves_large_positive_and_negative_deltas() -> anyhow::Result<()> {
        for delta in [130_i32, -130] {
            // Given a large signed encoder delta.
            let mut state = Translator::default();
            // When it is translated.
            let output = run(&mut state, &[Control::Encoder("left_loop_encoder", delta)])?;
            // Then bounded chunks preserve its magnitude and sign.
            let decoded: i32 = output
                .iter()
                .map(|message| i32::from(message[2]) - 64)
                .sum();
            assert_eq!(decoded, delta);
            assert_eq!(output.len(), 3);
        }
        Ok(())
    }

    #[test]
    fn front_panel_assignment_changes_are_atomic() -> anyhow::Result<()> {
        // Given all selectors initially Through and an established snapshot.
        let mut state = Translator {
            front_panel_ready: true,
            ..Translator::default()
        };
        let mut payload = [0_u8; 22];
        payload[17] = 0x55;
        let mut ignored = Vec::new();
        state.translate(&[], Some(&payload), &mut ignored)?;
        payload[17] = 0x95; // Channel 3 -> Left, with all other selectors unchanged.
        // When the next whole snapshot arrives.
        let mut output = Vec::new();
        state.translate(&[], Some(&payload), &mut output)?;
        // Then one Left action fires, with no transient Through/Right action.
        assert_eq!(output, [[0x94, 88, 127], [0x84, 88, 0]]);
        Ok(())
    }

    #[test]
    fn startup_crossfader_waits_for_mapping_feedback_and_uses_latest_snapshot() -> anyhow::Result<()>
    {
        // Given two physical snapshots before Djay's mapping sends playback state.
        let mut state = Translator::default();
        let mut feedback = feedback::FeedbackState::default();
        let mut decoder = feedback::Decoder::default();
        let mut payload = [0_u8; 22];
        payload[17] = 0x55;
        assert!(run_front_snapshot(&mut state, &payload)?.is_empty());
        payload[17] = 0x26; // A left, B through, C right, D left.
        payload[18] = 2;
        assert!(run_front_snapshot(&mut state, &payload)?.is_empty());
        let mut output = Vec::new();
        decoder.feed(&[0x90, 0, 127, 0xb6, 0, 0], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut output)?;
        assert!(output.is_empty()); // Lamps/meters are not the playback gate.
        // When real playback feedback arrives, even paused followed by track-unloaded.
        decoder.feed(&[0xb6, 4, 0, 0x96, 0, 0], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut output)?;
        // Then the latest four assignments and curve emit exactly once.
        assert_eq!(
            output,
            [
                [0x94, 82, 127],
                [0x84, 82, 0],
                [0x94, 84, 127],
                [0x84, 84, 0],
                [0x94, 86, 127],
                [0x84, 86, 0],
                [0x94, 91, 127],
                [0x84, 91, 0],
                [0xb4, 25, 0],
            ]
        );
        output.clear();
        decoder.feed(&[0xb6, 4, 127], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut output)?;
        assert!(output.is_empty()); // Later UI/playback changes cannot replay selectors.
        assert!(run_front_snapshot(&mut state, &payload)?.is_empty());
        Ok(())
    }

    fn run_front_snapshot(
        state: &mut Translator,
        payload: &[u8; 22],
    ) -> anyhow::Result<Vec<[u8; 3]>> {
        let mut output = Vec::new();
        state.translate(&[], Some(payload), &mut output)?;
        Ok(output)
    }

    #[test]
    fn startup_crossfader_handles_both_launch_orders_and_all_selector_positions()
    -> anyhow::Result<()> {
        for feedback_first in [false, true] {
            for (packed, assignments) in [
                (0x00, [0_u8; 4]),
                (0x55, [1; 4]),
                (0xaa, [2; 4]),
                (0x26, [2, 1, 0, 2]),
            ] {
                for (curve, cc) in [(0, 127), (1, 64), (2, 0)] {
                    // Given a fresh bridge with the physical switches already set.
                    let mut state = Translator::default();
                    let mut feedback = feedback::FeedbackState::default();
                    let mut decoder = feedback::Decoder::default();
                    decoder.feed(&[0xb6, 7, 0], |message| {
                        feedback.apply(message, Duration::ZERO);
                    });
                    let mut payload = [0_u8; 22];
                    payload[17] = packed;
                    payload[18] = curve;
                    let mut output = Vec::new();
                    // When playback feedback and the snapshot arrive in either order.
                    if feedback_first {
                        state.sync_front_panel(&feedback, &mut output)?;
                    }
                    state.translate(&[], Some(&payload), &mut output)?;
                    if !feedback_first {
                        state.sync_front_panel(&feedback, &mut output)?;
                    }
                    // Then every assignment and curve is applied once, without moving hardware.
                    let mut expected = Vec::new();
                    for (deck, assignment) in assignments.into_iter().enumerate() {
                        let note = 80 + 3 * u8::try_from(deck)? + assignment;
                        expected.extend([[0x94, note, 127], [0x84, note, 0]]);
                    }
                    expected.push([0xb4, 25, cc]);
                    assert_eq!(output, expected);
                    assert!(run_front_snapshot(&mut state, &payload)?.is_empty());
                }
            }
        }
        Ok(())
    }

    #[test]
    fn shutdown_releases_owned_notes_and_duplicate_edges_do_not_toggle_again() -> anyhow::Result<()>
    {
        // Given a held Play and an ignored duplicate press.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_play", true)])?;
        assert!(run(&mut state, &[Control::Button("left_play", true)])?.is_empty());
        // When shutdown releases ownership.
        let mut output = Vec::new();
        state.release_all(&mut output);
        // Then exactly one release is emitted.
        assert_eq!(output, [[0x80, 0, 0]]);
        assert!(state.held.is_empty());
        Ok(())
    }

    #[test]
    fn reverse_hold_releases_original_deck_after_selector_changes_or_shutdown() -> anyhow::Result<()>
    {
        for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
            for shutdown in [false, true] {
                // Given REV held on each deck with a duplicate input edge.
                let side = usize::from(deck.channel() % 2);
                let name = if side == 0 {
                    "left_reverse"
                } else {
                    "right_reverse"
                };
                let mut state = Translator::default();
                state.decks[side] = deck;
                assert_eq!(
                    run(&mut state, &[Control::Button(name, true)])?,
                    [[0x90 | deck.channel(), 3, 127]]
                );
                assert!(run(&mut state, &[Control::Button(name, true)])?.is_empty());
                run(
                    &mut state,
                    &[
                        Control::Button(
                            if side == 0 {
                                "left_deck_switch_c"
                            } else {
                                "right_deck_switch_d"
                            },
                            true,
                        ),
                        Control::Button(
                            if side == 0 {
                                "left_shift"
                            } else {
                                "right_shift"
                            },
                            true,
                        ),
                    ],
                )?;
                // When the physical button releases or the bridge shuts down.
                let mut output = Vec::new();
                if shutdown {
                    state.release_all(&mut output);
                } else {
                    state.translate(&[Control::Button(name, false)], None, &mut output)?;
                }
                // Then only its original deck receives release, including with Shift now held.
                assert_eq!(output, [[0x80 | deck.channel(), 3, 0]]);
                assert!(!state.held.contains_key(name));
            }
        }
        Ok(())
    }

    #[test]
    fn led_routing_follows_current_selection_while_held_notes_keep_old_ownership()
    -> anyhow::Result<()> {
        // Given Cue held on A.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_cue", true)])?;
        // When C becomes the visible deck.
        run(&mut state, &[Control::Button("left_deck_switch_c", true)])?;
        // Then lights show C while the held Cue still belongs to A.
        assert_eq!(state.local_led_state().decks[0], 2);
        assert_eq!(state.held["left_cue"][0].channel, 0);
        Ok(())
    }

    #[test]
    fn momentary_led_release_survives_shift_and_deck_changes() -> anyhow::Result<()> {
        // Given a held library preview after a deck/modifier change.
        let mut state = Translator::default();
        run(
            &mut state,
            &[
                Control::Button("left_library_play", true),
                Control::Button("left_deck_switch_c", true),
                Control::Button("left_shift", true),
            ],
        )?;
        // When the physical preview button releases.
        run(&mut state, &[Control::Button("left_library_play", false)])?;
        // Then the momentary light is not stranded on the old layer.
        assert!(!state.local_led_state().held[4]);
        Ok(())
    }
}
