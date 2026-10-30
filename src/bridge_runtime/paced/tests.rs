use super::*;
use std::cell::{Cell, RefCell};

mod backspin;
mod playback;
mod recovery;
mod tt_gesture;
mod watchdog;

fn config(value: i32) -> anyhow::Result<s4mk3_hid_midi_bridge::tension::Config> {
    Ok(serde_json::from_str(&format!(r#"{{"tension":{value}}}"#))?)
}

fn frame(host_us: u64, counts: [u32; 2], touch: u8) -> Frame {
    let mut buffers = [[0; 128]; 3];
    buffers[0][0] = 1;
    buffers[0][17] = touch;
    buffers[1][0] = 3;
    for (offset, value) in [(8, host_us.to_le_bytes()), (36, host_us.to_le_bytes())] {
        buffers[1][offset..offset + 4].copy_from_slice(&value[..4]);
    }
    for (offset, count) in [(12, counts[0]), (40, counts[1])] {
        buffers[1][offset..offset + 4].copy_from_slice(&count.to_le_bytes());
    }
    Frame {
        host_us,
        buffers,
        lengths: [23, 60, 0],
    }
}

fn active_engine(setting: i32) -> anyhow::Result<Engine> {
    let mut engine = Engine::new(config(setting)?);
    for index in 0_u32..8 {
        let at = u64::from(index) * 2000 + 1;
        let frame = frame(at, [index * 10, index * 10], 0x30);
        engine.observe(frame.report(0), frame.report(1), at, at)?;
        engine.tick(at)?;
    }
    Ok(engine)
}

#[test]
fn screens_require_input_and_a_successful_native_feedback_tick() -> anyhow::Result<()> {
    for input in [false, true] {
        for fail_led in [false, true] {
            // Given initialized output, with or without a queued input snapshot.
            let mut engine = Engine::new(config(-25)?);
            let (mut reader, frames, _done) = Reader::channels();
            if input {
                frames.send(frame(2001, [0, 0], 0))?;
            }
            let (stop_tx, stop) = crossbeam_channel::bounded(1);
            let events = RefCell::new(Vec::new());
            let mut output = Output::new(|action| {
                match action {
                    Action::Frame(..) => events.borrow_mut().push("input"),
                    Action::Feedback(_) => {
                        events.borrow_mut().push("feedback");
                        if fail_led {
                            anyhow::bail!("native LED write failed");
                        }
                        if !input {
                            stop_tx.send(())?;
                        }
                    }
                    Action::Screens => {
                        events.borrow_mut().push("screens");
                        stop_tx.send(())?;
                    }
                    Action::Initialize | Action::Motor(_) | Action::Playback(..) => {}
                }
                Ok(())
            })?;
            output.initialize()?;
            // When the real scheduler services its first output tick.
            let result = run(
                &mut engine,
                &mut reader,
                &stop,
                None,
                || Ok(2001),
                &mut output,
            );
            output.finish()?;
            // Then screen publication follows successful output and never precedes input.
            assert_eq!(result.is_err(), fail_led);
            let mut expected = Vec::new();
            if input {
                expected.push("input");
            }
            expected.push("feedback");
            if input && !fail_led {
                expected.push("screens");
            }
            assert_eq!(*events.borrow(), expected);
        }
    }
    Ok(())
}

#[test]
fn queued_one_wheel_release_precedes_overdue_motor_and_feedback() -> anyhow::Result<()> {
    // Given both wheels already assisting, and a queued left-only release.
    let mut engine = active_engine(-100)?;
    let (mut reader, frames, _done) = Reader::channels();
    frames.send(frame(16001, [80, 80], 0x20))?;
    let (stop_tx, stop) = crossbeam_channel::bounded(1);
    let actions = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => actions.borrow_mut().push((0, report)),
            Action::Frame(..) => actions.borrow_mut().push((1, neutral_report())),
            Action::Feedback(_) => {
                actions.borrow_mut().push((2, neutral_report()));
                stop_tx.send(())?;
            }
            Action::Initialize | Action::Screens | Action::Playback(..) => {}
        }
        Ok(())
    })?;
    // When the actual production scheduler handles the overdue slot.
    run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(16001),
        &mut output,
    )?;
    output.finish()?;
    // Then immediate partial neutral precedes MIDI dispatch and the tick;
    // the right force survives, and Stop ends with full neutral.
    let actions = actions.borrow();
    assert_eq!(actions[1].0, 0);
    assert_eq!(&actions[1].1[4..6], &[0, 0]);
    assert_ne!(&actions[1].1[9..11], &[0, 0]);
    assert_eq!(actions[2].0, 1);
    assert_eq!(actions[3].0, 0);
    assert_eq!(actions[4].0, 2);
    assert_eq!(actions.last(), Some(&(0, neutral_report())));
    Ok(())
}

#[test]
fn stop_wins_over_queued_force_without_dispatching_midi() -> anyhow::Result<()> {
    // Given active assistance, a pending frame, and Stop already signaled.
    let mut engine = active_engine(-100)?;
    let (mut reader, frames, _done) = Reader::channels();
    frames.send(frame(16001, [80, 80], 0x30))?;
    let (stop_tx, stop) = crossbeam_channel::bounded(1);
    stop_tx.send(())?;
    let writes = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => writes.borrow_mut().push(report),
            Action::Frame(..)
            | Action::Feedback(_)
            | Action::Initialize
            | Action::Screens
            | Action::Playback(..) => {
                anyhow::bail!("Stop must win before service dispatch")
            }
        }
        Ok(())
    })?;
    // When the actual scheduler sees both.
    run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(16001),
        &mut output,
    )?;
    output.finish()?;
    // Then there are only startup and final neutral writes.
    assert_eq!(*writes.borrow(), vec![neutral_report(); 2]);
    Ok(())
}

#[test]
fn runtime_faults_clear_force_before_returning() -> anyhow::Result<()> {
    // Given a reader error, future queued frame, or MIDI/LED/native write failure.
    // Freshness gaps have separate nonfatal regressions in the watchdog module.
    for fault in [0, 1, 3, 4, 5] {
        let mut engine = active_engine(-100)?;
        let (mut reader, frames, done) = Reader::channels();
        let (_stop_tx, stop) = crossbeam_channel::bounded(1);
        let now = 16001;
        if fault == 0 {
            done.send(Err(anyhow::anyhow!("native input failure")))?;
        } else if fault == 1 {
            frames.send(frame(70002, [80, 80], 0x30))?;
        } else if fault == 3 {
            frames.send(frame(16001, [80, 80], 0x30))?;
        }
        let writes = RefCell::new(Vec::new());
        let mut output = Output::new(|action| {
            match action {
                Action::Motor(report) => {
                    writes.borrow_mut().push(report);
                    if fault == 4 && report != neutral_report() {
                        anyhow::bail!("native output failure");
                    }
                }
                Action::Frame(..) if fault == 3 => anyhow::bail!("MIDI service failure"),
                Action::Feedback(_) if fault == 5 => anyhow::bail!("LED service failure"),
                Action::Initialize
                | Action::Frame(..)
                | Action::Feedback(_)
                | Action::Screens
                | Action::Playback(..) => {}
            }
            Ok(())
        })?;
        // When the production loop encounters the fault, its owner drops the guard.
        let result = run(
            &mut engine,
            &mut reader,
            &stop,
            None,
            || Ok(now),
            &mut output,
        );
        drop(output);
        // Then the error is preserved and the final attempted output is neutral.
        assert!(result.is_err(), "fault {fault}");
        assert_eq!(writes.borrow().last(), Some(&neutral_report()));
    }
    Ok(())
}

#[test]
fn analog_interrupt_reaches_service_unchanged_and_duration_stops_output() -> anyhow::Result<()> {
    // Given the original full-profile analog interrupt alongside fresh snapshots.
    let mut engine = Engine::new(config(100)?);
    let (mut reader, frames, _done) = Reader::channels();
    let mut sample = frame(1, [0, 0], 0);
    sample.lengths[2] = 79;
    sample.buffers[2][0] = 2;
    sample.buffers[2][3..5].copy_from_slice(&4095_u16.to_le_bytes());
    frames.send(sample.clone())?;
    let (_stop_tx, stop) = crossbeam_channel::bounded(1);
    let clock = Cell::new(1);
    let seen = RefCell::new(Vec::new());
    let writes = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Frame(frame, _) => {
                seen.borrow_mut().push(frame.report(2).to_vec());
                clock.set(10000);
            }
            Action::Motor(report) => writes.borrow_mut().push(report),
            Action::Initialize | Action::Feedback(_) | Action::Screens | Action::Playback(..) => {}
        }
        Ok(())
    })?;
    // When the actual loop dispatches the sample and reaches its bounded deadline.
    run(
        &mut engine,
        &mut reader,
        &stop,
        Some(10000),
        || Ok(clock.get()),
        &mut output,
    )?;
    output.finish()?;
    // Then the fader report is not replaced by the saved tension selection,
    // and no overdue motor tick occurs past the deadline.
    assert_eq!(*seen.borrow(), vec![sample.report(2).to_vec()]);
    assert_eq!(*writes.borrow(), vec![neutral_report(); 2]);
    Ok(())
}

#[test]
fn failed_startup_or_initialization_still_attempts_neutral() {
    // Given output failure during startup, before or during ring initialization.
    for fail_startup in [true, false] {
        let writes = RefCell::new(Vec::new());
        let calls = Cell::new(0);
        // When initialization returns an error.
        let result = (|| {
            let mut output = Output::new(|action| {
                match action {
                    Action::Motor(report) => {
                        writes.borrow_mut().push(report);
                        calls.set(calls.get() + 1);
                        if fail_startup && calls.get() == 1 {
                            anyhow::bail!("startup write failed");
                        }
                    }
                    Action::Initialize => anyhow::bail!("ring initialization failed"),
                    Action::Frame(..)
                    | Action::Feedback(_)
                    | Action::Screens
                    | Action::Playback(..) => {}
                }
                Ok(())
            })?;
            output.initialize()
        })();
        // Then the guard exists early enough to attempt cleanup in both cases.
        assert!(result.is_err());
        assert_eq!(*writes.borrow(), vec![neutral_report(); 2]);
    }
}
