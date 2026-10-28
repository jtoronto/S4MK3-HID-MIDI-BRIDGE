//! Shared descriptor-driven S4 button/encoder parser, without a USB `DeviceId`.

use std::time::Duration;

use anyhow::{Context as _, ensure};
use encdr::{
    core::descriptor::{EncoderEncoding, InputItemDesc},
    device::{encoder::EncoderState, loader::DescriptorRegistry},
};
use s4mk3_hid_midi_bridge::jog::decode::FRESHNESS_US;

use super::Control;

enum Field {
    Button(&'static str, usize, u8, bool),
    Encoder(&'static str, usize, u8, EncoderState),
}

pub(super) struct Parser {
    fields: Vec<Field>,
    previous: [Option<(u32, u32, Duration)>; 2],
    buttons_seen: bool,
}

impl Parser {
    pub(super) fn new() -> anyhow::Result<Self> {
        let mut registry = DescriptorRegistry::new();
        registry.load_builtins()?;
        let descriptor = crate::full_control_descriptor(
            registry
                .find(crate::S4_VENDOR_ID, crate::S4_PRODUCT_ID)
                .context("S4 descriptor")?,
        )?;
        let names = registry.intern_descriptor_names(&descriptor);
        let packet = descriptor
            .input_packets
            .iter()
            .find(|packet| packet.id == "buttons")
            .context("S4 buttons descriptor")?;
        let mut fields = Vec::with_capacity(packet.items.len());
        for item in &packet.items {
            let name = *names.get(item.name()).context("interned S4 control")?;
            match item {
                InputItemDesc::Button(button) => fields.push(Field::Button(
                    name,
                    button.byte,
                    u8::try_from(button.mask.0)?,
                    false,
                )),
                InputItemDesc::Touch(touch) => fields.push(Field::Button(
                    name,
                    touch.byte.context("S4 touch byte")?,
                    u8::try_from(touch.mask.context("S4 touch mask")?.0)?,
                    false,
                )),
                InputItemDesc::Encoder(encoder) => {
                    ensure!(
                        encoder.encoding == EncoderEncoding::Wrap16 && encoder.bits == 4,
                        "S4 encoder encoding"
                    );
                    fields.push(Field::Encoder(
                        name,
                        encoder.byte,
                        encoder.bit_offset,
                        EncoderState::default(),
                    ));
                }
                InputItemDesc::Slider(_) | InputItemDesc::EncoderFine(_) => {
                    anyhow::bail!("unexpected S4 button descriptor item")
                }
            }
        }
        Ok(Self {
            fields,
            previous: [None; 2],
            buttons_seen: false,
        })
    }

    /// Returns false for analog reports, which retain Encdr's existing path.
    pub(super) fn parse(
        &mut self,
        report: &[u8],
        now: Duration,
        output: &mut Vec<Control>,
    ) -> anyhow::Result<bool> {
        let payload = crate::input_payload(report)?.context("empty full input")?;
        match report[0] {
            1 => {
                for field in &mut self.fields {
                    match field {
                        Field::Button(name, byte, mask, previous) => {
                            let held = payload[*byte] & *mask != 0;
                            if held != *previous {
                                let startup_mode = !self.buttons_seen
                                    && matches!(
                                        *name,
                                        "left_turntable_mode"
                                            | "right_turntable_mode"
                                            | "left_jog_mode"
                                            | "right_jog_mode"
                                    );
                                if !startup_mode {
                                    output.push(Control::Button(name, held));
                                }
                                *previous = held;
                            }
                        }
                        Field::Encoder(name, byte, shift, state) => {
                            if let Some(delta) = state.update_wrap16(payload[*byte] >> *shift) {
                                output.push(Control::Encoder(name, delta));
                            }
                        }
                    }
                }
                self.buttons_seen = true;
                Ok(true)
            }
            3 => {
                for (side, offset, name) in [(0, 8, "left_jog_wheel"), (1, 36, "right_jog_wheel")] {
                    let timer = u32::from_le_bytes(report[offset..offset + 4].try_into()?);
                    let counter = u32::from_le_bytes(report[offset + 4..offset + 8].try_into()?);
                    if let Some((old_timer, old_counter, old_time)) = self.previous[side] {
                        if timer == old_timer && counter == old_counter {
                            continue;
                        }
                        let delta =
                            i32::from_le_bytes(counter.wrapping_sub(old_counter).to_le_bytes());
                        if now > old_time
                            && now - old_time < Duration::from_micros(FRESHNESS_US)
                            && u64::from(timer.wrapping_sub(old_timer)) <= FRESHNESS_US * 100
                            && delta.unsigned_abs() <= 512
                            && timer != old_timer
                            && delta != 0
                        {
                            output.push(Control::Jog(name, f64::from(delta) / 1000.0));
                        }
                    }
                    self.previous[side] = Some((timer, counter, now));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_startup_modes_require_release_and_a_new_press() -> anyhow::Result<()> {
        // Given a controller connected with both TT and JOG already held.
        let mut parser = Parser::new()?;
        let mut held = [0; 23];
        held[0] = 1;
        for field in &parser.fields {
            if let Field::Button(name, byte, mask, _) = field
                && matches!(*name, "left_turntable_mode" | "left_jog_mode")
            {
                held[byte + 1] |= mask;
            }
        }
        let mut controls = Vec::new();
        // When the initial snapshot and a duplicate arrive.
        parser.parse(&held, Duration::ZERO, &mut controls)?;
        parser.parse(&held, Duration::from_millis(2), &mut controls)?;
        // Then neither mode is armed until a release and fresh press.
        assert!(controls.is_empty());
        let mut released = [0; 23];
        released[0] = 1;
        parser.parse(&released, Duration::from_millis(4), &mut controls)?;
        controls.clear();
        parser.parse(&held, Duration::from_millis(6), &mut controls)?;
        assert!(
            controls
                .iter()
                .any(|control| matches!(control, Control::Button("left_turntable_mode", true)))
        );
        assert!(
            controls
                .iter()
                .any(|control| matches!(control, Control::Button("left_jog_mode", true)))
        );
        Ok(())
    }

    #[test]
    fn production_descriptor_keeps_verified_move_loop_fields_and_touch() -> anyhow::Result<()> {
        // Given a hardware-free production parser with baseline encoder values.
        let mut parser = Parser::new()?;
        let mut report = [0; 23];
        report[0] = 1;
        let mut controls = Vec::new();
        parser.parse(&report, Duration::ZERO, &mut controls)?;
        controls.clear();
        report[20] = 0x11;
        report[21] = 0x10;
        report[22] = 0x01;
        report[7] = 0x24;
        report[16] = 0x24;
        report[17] = 0x30;
        // When all four physical knobs advance and press alongside top touch.
        parser.parse(&report, Duration::from_millis(2), &mut controls)?;
        // Then the corrected names and clockwise deltas match production routing.
        for name in [
            "left_move_encoder",
            "left_loop_encoder",
            "right_move_encoder",
            "right_loop_encoder",
        ] {
            assert!(
                controls
                    .iter()
                    .any(|control| matches!(control, Control::Encoder(found, 1) if *found == name))
            );
        }
        for name in [
            "left_move_encoder_press",
            "left_loop_encoder_press",
            "right_move_encoder_press",
            "right_loop_encoder_press",
            "left_wheel_touch",
            "right_wheel_touch",
        ] {
            assert!(
                controls.iter().any(
                    |control| matches!(control, Control::Button(found, true) if *found == name)
                )
            );
        }
        Ok(())
    }
}
