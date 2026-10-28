use super::*;

#[test]
fn recovery_skips_snapshots_that_aged_during_the_neutral_write() -> anyhow::Result<()> {
    // Given an input gap followed by a delayed neutral write and queued input.
    let mut engine = active_engine(-100)?;
    let (mut reader, frames, _done) = Reader::channels();
    let (stop_tx, stop) = crossbeam_channel::bounded(1);
    let clock = Cell::new(16001);
    let ticks = Cell::new(0);
    let writes = RefCell::new(Vec::new());
    let received = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => {
                writes.borrow_mut().push((clock.get(), report));
                if clock.get() == 70002 {
                    clock.set(140_003);
                    frames.send(frame(72002, [1000, 2000], 0x30))?;
                    frames.send(frame(140_003, [5000, 6000], 0x30))?;
                }
            }
            Action::Frame(frame, _) => received.borrow_mut().push(frame.host_us),
            Action::Feedback(_) => {
                ticks.set(ticks.get() + 1);
                if ticks.get() == 1 {
                    clock.set(70002);
                } else {
                    stop_tx.send(())?;
                }
            }
            Action::Initialize | Action::Screens | Action::Playback(..) => {}
        }
        Ok(())
    })?;
    // When queued snapshots are drained after that write returns.
    run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(clock.get()),
        &mut output,
    )?;
    output.finish()?;
    // Then a stale candidate cannot abort recovery or revive old force.
    assert_eq!(*received.borrow(), vec![72002, 140_003]);
    assert!(
        writes
            .borrow()
            .iter()
            .filter(|(at, _)| *at >= 70002)
            .all(|(_, report)| *report == neutral_report())
    );
    Ok(())
}

#[test]
fn repeated_gaps_require_new_movement_and_keep_each_wheels_touch_gate() -> anyhow::Result<()> {
    // Given active assistance, a stale queued frame at the first gap,
    // then two independent recovery sequences with counter jumps.
    let script = [
        (16001, None),
        (70002, Some(frame(16002, [80, 80], 0x30))),
        (72002, Some(frame(72002, [1000, 2000], 0x30))),
        (74002, Some(frame(74002, [1010, 2000], 0x30))),
        (76002, Some(frame(76002, [1020, 2000], 0x30))),
        (78002, Some(frame(78002, [1030, 2000], 0x30))),
        (80002, Some(frame(80002, [1040, 2000], 0x30))),
        (140_003, None),
        (142_003, Some(frame(142_003, [5000, 5000], 0))),
        (144_003, Some(frame(144_003, [5010, 5010], 0))),
        (146_003, Some(frame(146_003, [5020, 5020], 0))),
        (148_003, Some(frame(148_003, [5030, 5030], 0))),
        (150_003, Some(frame(150_003, [5040, 5040], 0))),
        (152_003, Some(frame(152_003, [5050, 5050], 0x10))),
        (154_003, Some(frame(154_003, [5060, 5060], 0x10))),
        (156_003, Some(frame(156_003, [5070, 5070], 0x10))),
        (158_003, Some(frame(158_003, [5080, 5080], 0x10))),
    ];
    let mut engine = active_engine(-100)?;
    let (mut reader, frames, _done) = Reader::channels();
    let (stop_tx, stop) = crossbeam_channel::bounded(1);
    let clock = Cell::new(script[0].0);
    let next = Cell::new(0);
    let received = Cell::new(0);
    let writes = RefCell::new(Vec::new());
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => writes.borrow_mut().push((clock.get(), report)),
            Action::Frame(..) => received.set(received.get() + 1),
            Action::Feedback(_) => {
                next.set(next.get() + 1);
                if let Some((at, sample)) = script.get(next.get()) {
                    clock.set(*at);
                    if let Some(sample) = sample {
                        frames.send(sample.clone())?;
                    }
                } else {
                    stop_tx.send(())?;
                }
            }
            Action::Initialize | Action::Screens | Action::Playback(..) => {}
        }
        Ok(())
    })?;
    // When the production scheduler receives the scripted gaps and movement.
    run(
        &mut engine,
        &mut reader,
        &stop,
        None,
        || Ok(clock.get()),
        &mut output,
    )?;
    output.finish()?;
    // Then each gap clears force through the new baseline and qualification,
    // with no assistance on the stationary or untouched right wheel.
    let writes = writes.borrow();
    for (start, qualified) in [(70002, 80002), (140_003, 158_003)] {
        assert!(
            writes
                .iter()
                .filter(|(at, _)| *at >= start && *at < qualified)
                .all(|(_, report)| *report == neutral_report())
        );
        assert!(
            writes
                .iter()
                .any(|(at, report)| *at == qualified && report[4..6] != [0, 0])
        );
    }
    assert!(
        writes
            .iter()
            .filter(|(at, _)| *at >= 70002)
            .all(|(_, report)| report[9..11] == [0, 0])
    );
    assert_eq!(
        received.get(),
        script.iter().filter(|(_, frame)| frame.is_some()).count()
    );
    assert_eq!(next.get(), script.len());
    Ok(())
}
