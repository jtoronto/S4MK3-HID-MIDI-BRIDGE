use super::*;

fn backwards(index: u32, released: bool) -> Frame {
    frame(
        u64::from(index) * 2000 + 2001,
        [1_000_000 - index * 2, 2_000_000 - index * 2],
        if released { 0x20 } else { 0x30 },
    )
}

fn released_engine() -> anyhow::Result<Engine> {
    let mut engine = Engine::new(config(-25)?);
    for index in 0..=40 {
        let sample = backwards(index, index > 30);
        engine.observe(
            sample.report(0),
            sample.report(1),
            sample.host_us,
            sample.host_us,
        )?;
        assert!(engine.set_backspin([index >= 8, false]).is_none());
        engine.tick(sample.host_us)?;
    }
    Ok(engine)
}

#[test]
fn actual_pacer_applies_qualification_and_fade_while_midi_frames_keep_arriving()
-> anyhow::Result<()> {
    // Given the real input/output scheduler, with qualification returned by Frame service.
    let mut engine = Engine::new(config(-25)?);
    let (mut reader, frames, _done) = Reader::channels();
    frames.send(backwards(0, false))?;
    let (stop_tx, stop) = crossbeam_channel::bounded(1);
    let index = Cell::new(0_u32);
    let clock = Cell::new(2001);
    let writes = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => writes.borrow_mut().push((clock.get(), report)),
            Action::Frame(sample, assist) => {
                *assist = [index.get() >= 8, false];
                assert_eq!(sample.report(0)[17] & 0x10 != 0, index.get() <= 200);
            }
            Action::Feedback(_) => {
                index.set(index.get() + 1);
                if index.get() > 710 {
                    stop_tx.send(())?;
                } else {
                    let sample = backwards(index.get(), index.get() > 200);
                    clock.set(sample.host_us);
                    frames.send(sample)?;
                }
            }
            Action::Initialize | Action::Screens | Action::Playback(..) => {}
        }
        Ok(())
    })?;
    // When its real output clock services continuous movement across release and deadline.
    run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(clock.get()),
        &mut output,
    )?;
    output.finish()?;
    // Then released assistance is nonzero, bounded and not extended by more qualified frames.
    let writes = writes.borrow();
    let release = 404_001;
    assert!(
        writes
            .iter()
            .any(|(at, report)| *at == release && report[4..6] != [0, 0])
    );
    assert!(
        writes
            .iter()
            .filter(|(at, _)| *at >= release + 1_000_000)
            .all(|(_, report)| report[4..6] == [0, 0])
    );
    assert!(
        writes
            .iter()
            .all(|(_, report)| u16::from_le_bytes([report[4], report[5]]) <= 7000)
    );
    assert_eq!(
        writes.last().map(|(_, report)| *report),
        Some(neutral_report())
    );
    Ok(())
}

#[test]
fn stop_preempts_a_released_assist_pulse_and_pending_input() -> anyhow::Result<()> {
    // Given a real released pulse, queued movement and an already-signalled Stop.
    let mut engine = released_engine()?;
    assert_ne!(&engine.tick(82_001)?[4..6], &[0, 0]);
    let (mut reader, frames, _done) = Reader::channels();
    frames.send(backwards(41, true))?;
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
                anyhow::bail!("Stop must preempt released assistance")
            }
        }
        Ok(())
    })?;
    // When the normal production loop handles shutdown.
    run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(84_001),
        &mut output,
    )?;
    output.finish()?;
    // Then only neutral startup/cleanup reports are written.
    assert_eq!(*writes.borrow(), vec![neutral_report(); 2]);
    Ok(())
}

#[test]
fn watchdog_recovery_cannot_rearm_an_untouched_old_event() -> anyhow::Result<()> {
    // Given an active released pulse whose input has become stale.
    let mut engine = released_engine()?;
    let (mut reader, frames, _done) = Reader::channels();
    let (stop_tx, stop) = crossbeam_channel::bounded(1);
    let clock = Cell::new(140_003);
    let tick = Cell::new(0_u32);
    let writes = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => writes.borrow_mut().push(report),
            Action::Frame(_, assist) => *assist = [true, false],
            Action::Feedback(_) => {
                tick.set(tick.get() + 1);
                if tick.get() > 12 {
                    stop_tx.send(())?;
                } else {
                    clock.set(clock.get() + 2000);
                    let counts = [9000 - tick.get() * 2, 9000];
                    frames.send(frame(clock.get(), counts, 0))?;
                }
            }
            Action::Initialize | Action::Screens | Action::Playback(..) => {}
        }
        Ok(())
    })?;
    // When the watchdog neutralizes, then fresh input returns with old qualification.
    run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(clock.get()),
        &mut output,
    )?;
    output.finish()?;
    // Then neither the old pulse nor the stationary other wheel ever gets force.
    assert!(
        writes
            .borrow()
            .iter()
            .all(|report| *report == neutral_report())
    );
    Ok(())
}

#[test]
fn current_frame_cancellation_writes_partial_neutral_before_any_later_tick() -> anyhow::Result<()> {
    // Given a released pulse and a current input which cancels its software event.
    let mut engine = released_engine()?;
    let writes = RefCell::new(Vec::new());
    let events = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => {
                writes.borrow_mut().push(report);
                events.borrow_mut().push("motor");
            }
            Action::Frame(_, assist) => {
                *assist = [false; 2];
                events.borrow_mut().push("input");
            }
            Action::Playback(..) => {}
            Action::Initialize | Action::Feedback(_) | Action::Screens => {
                anyhow::bail!("No output tick is part of this input-edge check")
            }
        }
        Ok(())
    })?;
    // When the real input-edge handler processes that frame without an output tick.
    output.sample(&mut engine, &backwards(41, true), 84_001)?;
    // Then cancellation itself writes zero on that wheel and preserves the other wheel.
    assert_eq!(*events.borrow(), vec!["motor", "input", "motor"]);
    let cancelled = *writes.borrow().last().context("cancellation report")?;
    assert_eq!(&cancelled[4..6], &[0, 0]);
    assert_ne!(&cancelled[9..11], &[0, 0]);
    output.finish()?;
    Ok(())
}
