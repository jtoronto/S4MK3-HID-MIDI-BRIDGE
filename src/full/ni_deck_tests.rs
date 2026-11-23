use super::{Deck, Translator, feedback::FeedbackState, leds::LedConfig};

#[test]
fn ni_output_tracks_confirmed_feedback_for_the_deck_on_each_physical_side() -> anyhow::Result<()> {
    let colors = LedConfig::default();
    let mut translator = Translator {
        decks: [Deck::C, Deck::D],
        motor_enabled: [true, true],
        ..Translator::default()
    };
    let mut feedback = FeedbackState::default();
    let mut midi = Vec::new();
    let mut desired = |translator: &mut Translator, feedback: &FeedbackState, fresh| {
        midi.clear();
        translator.sync_front_panel(feedback, &mut midi)?;
        Ok::<_, anyhow::Error>(translator.ni_deck_output(feedback, &colors, fresh))
    };

    let unknown = desired(&mut translator, &feedback, true)?;
    assert!(
        unknown
            .iter()
            .all(|side| !side.playing && side.ring.is_none())
    );
    feedback.loaded = [Some(true); 4];
    feedback.internal = [Some(true); 4];
    feedback.playing = [Some(true), Some(true), Some(false), Some(true)];
    let right = desired(&mut translator, &feedback, true)?;
    assert_eq!(right.map(|side| side.playing), [false, true]);
    assert_eq!(
        right.map(|side| side.ring),
        [2, 3].map(|deck| {
            Some(colors.deck_colors[deck].base() + colors.palette_active_intensity)
        })
    );

    feedback.playing[2] = Some(true);
    assert_eq!(
        desired(&mut translator, &feedback, true)?.map(|side| side.playing),
        [true, true]
    );
    feedback.thru[3] = Some(true);
    let left = desired(&mut translator, &feedback, true)?;
    assert_eq!(left.map(|side| side.playing), [true, false]);
    assert_eq!(left[1].ring, None);
    assert_eq!(
        desired(&mut translator, &feedback, false)?,
        [s4mk3_hid_midi_bridge::screens::DeckOutput::default(); 2]
    );
    Ok(())
}

#[test]
fn jog_output_requires_selected_deck_playback_and_fresh_input() -> anyhow::Result<()> {
    let colors = LedConfig::default();
    let mut translator = Translator {
        decks: [Deck::C, Deck::D],
        ..Translator::default()
    };
    let mut feedback = FeedbackState::default();
    feedback.loaded = [Some(true); 4];
    feedback.internal = [Some(true); 4];
    feedback.playing = [Some(false), Some(false), Some(true), Some(true)];
    translator.sync_front_panel(&feedback, &mut Vec::new())?;
    let output = translator.ni_deck_output(&feedback, &colors, true);
    assert_eq!(output.map(|side| side.turntable), [false, false]);
    assert_eq!(output.map(|side| side.playing), [true, true]);
    feedback.playing[2] = None;
    feedback.loaded[3] = None;
    translator.sync_front_panel(&feedback, &mut Vec::new())?;
    assert_eq!(
        translator
            .ni_deck_output(&feedback, &colors, true)
            .map(|side| side.playing),
        [false, false]
    );
    assert_eq!(
        translator.ni_deck_output(&feedback, &colors, false),
        [s4mk3_hid_midi_bridge::screens::DeckOutput::default(); 2]
    );
    Ok(())
}
