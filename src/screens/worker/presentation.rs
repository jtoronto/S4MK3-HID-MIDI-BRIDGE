//! Diagnostic frame admission is separate from NI ownership and cleanup.

use std::time::{Duration, Instant};

use super::{Frame, page};
use crate::screen_motor_diag::{self as diag, Case, Kind};

pub(in crate::screens::worker) struct Presentation {
    case: Case,
    generation: u64,
    frames: [Frame; 2],
    previous: Option<[(page::Page, bool); 2]>,
    frozen: Option<[page::Page; 2]>,
    bright: bool,
    deadline: Option<Instant>,
    pairs: u64,
}

impl Presentation {
    pub(in crate::screens::worker) fn new(case: Case) -> Self {
        Self {
            case,
            generation: 0,
            frames: [Frame::default(), Frame::default()],
            previous: None,
            frozen: None,
            bright: true,
            deadline: None,
            pairs: 0,
        }
    }

    #[cfg(any(windows, test))]
    pub(super) const fn reset(&mut self, generation: u64) {
        self.generation = generation;
        self.previous = None;
        self.frozen = None;
        self.bright = true;
        self.deadline = None;
        self.pairs = 0;
    }

    pub(in crate::screens::worker) const fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(in crate::screens::worker) fn publish(
        &mut self,
        pages: [page::Page; 2],
        now: Instant,
        publish: impl FnOnce(&[Frame; 2]) -> anyhow::Result<()>,
    ) -> anyhow::Result<bool> {
        if self.case == Case::Acquired || (self.case == Case::Static && self.pairs > 0) {
            return Ok(false);
        }
        let pages = match self.case {
            Case::Static | Case::Updates => *self.frozen.get_or_insert(pages),
            Case::Normal | Case::Off | Case::Acquired => pages,
        };
        if !self.advance_phase(now, &pages) {
            return Ok(false);
        }
        let rendered = pages.map(|page| (page, !page.selecting() || self.bright));
        if self.case != Case::Updates && self.previous == Some(rendered) {
            return Ok(false);
        }
        let render = diag::record(
            Kind::ScreenRenderBegin,
            [self.generation, self.pairs, 0, 0, 0, 0, 0, 0],
            &[],
        );
        for side in 0..2 {
            page::render_phase(&pages[side], &mut self.frames[side], rendered[side].1);
        }
        if self.case == Case::Updates && !self.bright {
            for frame in &mut self.frames {
                frame.pixels[0] ^= u16::MAX;
            }
        }
        let hashes = if diag::enabled() {
            self.frames
                .each_ref()
                .map(|frame| crate::screens::route_fingerprint(&frame.pixels))
        } else {
            [0; 2]
        };
        diag::record(
            Kind::ScreenRenderEnd,
            [
                render,
                self.generation,
                self.pairs,
                hashes[0],
                hashes[1],
                0,
                0,
                0,
            ],
            &[],
        );
        if tracing::enabled!(target: "s4_screen_route_diag", tracing::Level::DEBUG) {
            tracing::debug!(
                target: "s4_screen_route_diag",
                generation = self.generation,
                pages = ?rendered.map(|(page, _)| page),
                bright = ?rendered.map(|(_, bright)| bright),
                hashes = ?self.frames.each_ref().map(|frame| crate::screens::route_fingerprint(&frame.pixels)),
                "SCREEN_ROUTE_RENDER"
            );
        }
        let begin = diag::record(
            Kind::ScreenPublishBegin,
            [
                self.generation,
                self.pairs,
                hashes[0],
                hashes[1],
                0,
                0,
                0,
                0,
            ],
            &[],
        );
        let result = publish(&self.frames);
        diag::record(
            Kind::ScreenPublishEnd,
            [
                begin,
                self.generation,
                u64::from(result.is_ok()),
                self.pairs,
                hashes[0],
                hashes[1],
                0,
                0,
            ],
            &[],
        );
        result?;
        self.pairs = self.pairs.saturating_add(1);
        self.previous = Some(rendered);
        if self.case == Case::Static {
            self.deadline = None;
        }
        Ok(true)
    }

    fn advance_phase(&mut self, now: Instant, pages: &[page::Page; 2]) -> bool {
        if self.case == Case::Updates {
            return update_phase(now, &mut self.deadline, &mut self.bright);
        }
        if pages.iter().any(page::Page::selecting) {
            update_phase(now, &mut self.deadline, &mut self.bright);
        } else {
            self.bright = true;
            self.deadline = None;
        }
        true
    }
}

pub(super) fn update_phase(
    now: Instant,
    deadline: &mut Option<Instant>,
    bright: &mut bool,
) -> bool {
    match *deadline {
        None => {
            *bright = true;
            *deadline = Some(now + Duration::from_millis(500));
            true
        }
        Some(next) if now >= next => {
            *bright = !*bright;
            *deadline = Some(now + Duration::from_millis(500));
            true
        }
        Some(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn static_selection_frame_does_not_leave_an_expiring_flash_deadline() -> anyhow::Result<()> {
        let pages = [page::Page::Info(page::Info {
            selecting: true,
            ..page::Info::default()
        }); 2];
        let mut presentation = Presentation::new(Case::Static);
        assert!(presentation.publish(pages, Instant::now(), |_| Ok(()))?);
        assert_eq!(presentation.deadline(), None);
        assert!(!presentation.publish(pages, Instant::now(), |_| Ok(()))?);
        Ok(())
    }
}
