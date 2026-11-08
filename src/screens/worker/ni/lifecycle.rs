//! Request generations and rendering are independent of helper process I/O.

use super::*;

fn release<T: Transport>(transport: &mut Option<T>, unconfirmed: &mut bool) -> bool {
    if let Some(mut session) = transport.take() {
        match session.close() {
            Ok(()) => *unconfirmed = false,
            Err(error) => {
                *unconfirmed = true;
                tracing::warn!(%error, "SCREEN_RELEASE_UNCONFIRMED: NI ownership restoration not established");
                return false;
            }
        }
    }
    true
}

fn pending_failure(
    shared: &Shared,
    generation: u64,
    unconfirmed: bool,
) -> anyhow::Result<Option<u64>> {
    if !unconfirmed {
        return Ok(None);
    }
    let mailbox = shared
        .0
        .lock()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    Ok(mailbox
        .latest
        .filter(|request| request.generation != generation)
        .map(|request| request.generation))
}

pub(in crate::screens::worker) fn run<F, T, S>(
    shared: &Shared,
    mut open: F,
    mut enabled: bool,
    status: &S,
) -> anyhow::Result<()>
where
    F: FnMut(Identity, Cancellation) -> anyhow::Result<T>,
    T: Transport,
    S: Fn(u64, State),
{
    let mut transport: Option<T> = None;
    let mut frames = [Frame::default(), Frame::default()];
    let mut current = None;
    let mut previous = None;
    let mut owner = (0, 0);
    let mut active = false;
    let mut bright = true;
    let mut next_flash = None;
    let mut release_unconfirmed = false;
    let mut failed_release = None;
    loop {
        let (request, stopping) = work(
            shared,
            &mut current,
            owner.0,
            transport.is_some(),
            next_flash,
        )?;
        if stopping {
            release(&mut transport, &mut release_unconfirmed);
            return Ok(());
        }
        if let Some(request) = request {
            let released = release(&mut transport, &mut release_unconfirmed)
                && failed_release.take() != Some(request.generation);
            if !released {
                failed_release = pending_failure(shared, request.generation, release_unconfirmed)?;
            }
            previous = None;
            next_flash = None;
            active = false;
            owner = (request.generation, request.id);
            enabled = matches!(request.action, Action::On | Action::Retry) && released;
            let state = if !released || (!enabled && release_unconfirmed) {
                State::Failed
            } else if enabled {
                State::Pending
            } else {
                State::Inactive
            };
            emit(shared, status, owner, state);
        }
        let Some(pages) = current.filter(|_| enabled) else {
            continue;
        };
        let result = (|| -> anyhow::Result<()> {
            if transport.is_none() {
                let Some((identity, cancellation)) = acquire(shared, owner.0)? else {
                    return Ok(());
                };
                transport = Some(open(identity, cancellation)?);
            }
            let transport = transport.as_mut().context("NI screen session absent")?;
            transport.poll_control()?;
            if interrupted(shared, owner.0)? {
                return Ok(());
            }
            let now = Instant::now();
            if pages.iter().any(page::Page::selecting) {
                if next_flash.is_none() {
                    bright = true;
                    next_flash = Some(now + Duration::from_millis(500));
                } else if next_flash.is_some_and(|deadline| now >= deadline) {
                    bright = !bright;
                    next_flash = Some(now + Duration::from_millis(500));
                }
            } else {
                bright = true;
                next_flash = None;
            }
            let rendered = pages.map(|page| (page, !page.selecting() || bright));
            if previous != Some(rendered) {
                for side in 0..2 {
                    page::render_phase(&pages[side], &mut frames[side], rendered[side].1);
                }
                log_render(owner, rendered, &frames);
                transport.publish(&frames)?;
                if interrupted(shared, owner.0)? {
                    return Ok(());
                }
                previous = Some(rendered);
                if !active {
                    active = true;
                    emit(shared, status, owner, State::Active);
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            release_unconfirmed |= error.is::<ipc::Unconfirmed>();
            release(&mut transport, &mut release_unconfirmed);
            failed_release = pending_failure(shared, owner.0, release_unconfirmed)?;
            previous = None;
            next_flash = None;
            active = false;
            enabled = false;
            tracing::warn!(%error, "SCREENS_DISABLED: isolated screen helper failed; controls remain active");
            emit(shared, status, owner, State::Failed);
        }
    }
}
