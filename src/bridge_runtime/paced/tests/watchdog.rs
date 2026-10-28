use super::*;

#[test]
fn freshness_gap_neutralizes_tension_without_stopping_midi_or_feedback() -> anyhow::Result<()> {
    // Given active assistance and a clock-controlled gap in snapshot delivery.
    let mut engine = active_engine(-100)?;
    let (mut reader, frames, _done) = Reader::channels();
    let (stop_tx, stop) = crossbeam_channel::bounded(1);
    let clock = Cell::new(16001);
    let ticks = Cell::new(0);
    let writes = RefCell::new(Vec::new());
    let received = RefCell::new(Vec::new());
    let resumed = frame(72002, [80, 80], 0x30);
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => writes.borrow_mut().push(report),
            Action::Frame(frame, _) => received.borrow_mut().push(frame.report(0).to_vec()),
            Action::Feedback(_) => {
                ticks.set(ticks.get() + 1);
                match ticks.get() {
                    1 => clock.set(70002),
                    2 => {
                        frames.send(resumed.clone())?;
                        clock.set(72002);
                    }
                    3 => stop_tx.send(())?,
                    _ => anyhow::bail!("unexpected extra output tick"),
                }
            }
            Action::Initialize | Action::Screens | Action::Playback(..) => {}
        }
        Ok(())
    })?;
    // When the production scheduler crosses 50 ms and fresh input then returns.
    run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(clock.get()),
        &mut output,
    )?;
    output.finish()?;
    // Then force is cleared, but fresh controls and feedback are still serviced.
    let writes = writes.borrow();
    assert_ne!(writes[1], neutral_report());
    assert!(writes[2..].iter().all(|report| *report == neutral_report()));
    assert_eq!(*received.borrow(), vec![resumed.report(0).to_vec()]);
    assert_eq!(ticks.get(), 3);
    Ok(())
}

#[test]
fn fresh_queued_motion_cannot_hide_timeout_or_reuse_prior_force() -> anyhow::Result<()> {
    for setting in [-100, 0, 100] {
        // Given a fresh queued snapshot after the old history's deadline,
        // for assistance, neutral, and braking.
        let mut engine = active_engine(setting)?;
        let (mut reader, frames, _done) = Reader::channels();
        frames.send(frame(64001, [80, 80], 0x30))?;
        let (stop_tx, stop) = crossbeam_channel::bounded(1);
        let clock = Cell::new(64001);
        let ticks = Cell::new(0_u32);
        let writes = RefCell::new(Vec::new());
        let received = Cell::new(0);
        let mut output = Output::new(|action| {
            match action {
                Action::Motor(report) => writes.borrow_mut().push((clock.get(), report)),
                Action::Frame(..) => {
                    if received.get() < 2 {
                        assert_eq!(
                            writes.borrow().last().map(|(_, report)| *report),
                            Some(neutral_report())
                        );
                    }
                    received.set(received.get() + 1);
                }
                Action::Feedback(_) => {
                    ticks.set(ticks.get() + 1);
                    if ticks.get() == 32 {
                        stop_tx.send(())?;
                    } else {
                        let at = clock.get() + 2000;
                        frames.send(frame(at, [80 + ticks.get() * 10; 2], 0x30))?;
                        clock.set(at);
                    }
                }
                Action::Initialize | Action::Screens | Action::Playback(..) => {}
            }
            Ok(())
        })?;
        // When time reaches the deadline, followed by sustained fresh motion.
        run(
            &mut engine,
            &mut reader,
            &stop,
            None,
            || Ok(clock.get()),
            &mut output,
        )?;
        output.finish()?;
        // Then old history and the first new baseline cannot produce force.
        // New movement starts at 68001 and must qualify for 6 ms before force.
        let writes = writes.borrow();
        assert!(
            writes
                .iter()
                .filter(|(at, _)| *at < 74001)
                .all(|(_, report)| *report == neutral_report())
        );
        assert_eq!(
            writes.iter().any(|(_, report)| *report != neutral_report()),
            setting != 0
        );
        assert_eq!(received.get(), 32);
        assert_eq!(ticks.get(), 32);
    }
    Ok(())
}

#[test]
fn failed_timeout_neutral_still_exits_with_an_error() -> anyhow::Result<()> {
    // Given active assistance whose neutral write will fail after the gap.
    let mut engine = active_engine(-100)?;
    let (mut reader, _frames, _done) = Reader::channels();
    let (_stop_tx, stop) = crossbeam_channel::bounded(1);
    let clock = Cell::new(16001);
    let active = Cell::new(false);
    let writes = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => {
                writes.borrow_mut().push(report);
                if active.get() && report == neutral_report() {
                    anyhow::bail!("native neutral write failed");
                }
                active.set(report != neutral_report());
            }
            Action::Feedback(_) => clock.set(70002),
            Action::Frame(..) | Action::Initialize | Action::Screens | Action::Playback(..) => {}
        }
        Ok(())
    })?;
    // When the watchdog tries to clear force, then the error is not swallowed
    // and the guard still attempts final neutral.
    let result = run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(clock.get()),
        &mut output,
    );
    drop(output);
    assert!(result.is_err());
    assert_ne!(writes.borrow()[1], neutral_report());
    assert_eq!(writes.borrow().last(), Some(&neutral_report()));
    Ok(())
}
