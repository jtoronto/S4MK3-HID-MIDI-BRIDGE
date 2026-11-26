//! Live-data quadrants and a reactive physical STEMS legend.

pub use super::info::{Info, TempoPickup};
use super::{Frame, HEIGHT, WIDTH, rgb};

const BACKGROUND: u16 = rgb(10, 12, 18);
const PANEL: u16 = rgb(24, 30, 42);
const BORDER: u16 = rgb(96, 108, 128);
const PRIMARY: u16 = rgb(244, 248, 255);
const SECONDARY: u16 = rgb(164, 178, 202);
const PRESSED: u16 = rgb(252, 176, 36);
const SCALE: usize = 2;
const MARGIN: usize = 8;
const GAP: usize = 4;
const PAD_WIDTH: usize = (WIDTH - 2 * MARGIN - 3 * GAP) / 4;
const PADS_Y: usize = 40;
const PAD_HEIGHT: usize = (HEIGHT - MARGIN - PADS_Y - GAP) / 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadState {
    pub active: Option<bool>,
    pub held: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FxWarning {
    pub deck: u8,
    /// Bits 0-2 are parameter knobs; bit 3 is dry/wet, per physical strip.
    pub mismatch: [u8; 2],
    pub unknown: [u8; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Empty,
    Info(Info),
    Stems {
        shifted: bool,
        pads: [PadState; 8],
        fx: FxWarning,
        backspin: Option<u8>,
    },
    StemFx {
        deck: u8,
        slot: u8,
        layout: Option<usize>,
        routes: [Option<bool>; 5],
        fx: FxWarning,
        backspin: Option<u8>,
    },
}

impl Page {
    #[must_use]
    pub const fn selecting(&self) -> bool {
        match self {
            Self::Info(info) => info.selecting,
            Self::Empty | Self::Stems { .. } | Self::StemFx { .. } => false,
        }
    }
}

pub fn render(page: &Page, frame: &mut Frame) {
    render_phase(page, frame, true);
}

pub fn render_phase(page: &Page, frame: &mut Frame, bright: bool) {
    frame.clear(BACKGROUND);
    let (shifted, pads, fx, backspin) = match page {
        Page::Empty => return,
        Page::Info(info) => return super::info::render(info, frame, bright),
        Page::StemFx {
            deck,
            slot,
            layout,
            routes,
            fx,
            backspin,
        } => {
            let top = render_headers(*fx, *backspin, frame);
            render_stem_fx(frame, top, *deck, *slot, *layout, *routes);
            return;
        }
        Page::Stems {
            shifted,
            pads,
            fx,
            backspin,
        } => (shifted, pads, fx, backspin),
    };
    let top = render_headers(*fx, *backspin, frame);
    let pads_y = PADS_Y + top;
    let pad_height = PAD_HEIGHT - top / 2;
    for (column, name) in ["DRUM", "BASS", "HARM", "VOX"].into_iter().enumerate() {
        let x = MARGIN + column * (PAD_WIDTH + GAP);
        frame.centered_text(x + PAD_WIDTH / 2, 12 + top, name, SCALE, PRIMARY);
    }
    for (index, pad) in pads.iter().enumerate() {
        let x = MARGIN + (index % 4) * (PAD_WIDTH + GAP);
        let y = pads_y + (index / 4) * (pad_height + GAP);
        let (fill, ink, status) = match pad.active {
            Some(true) => (PRIMARY, BACKGROUND, "ON"),
            Some(false) => (PANEL, PRIMARY, "OFF"),
            None => (PANEL, SECONDARY, "?"),
        };
        let border = if pad.held { 3 } else { 1 };
        frame.rect(
            x,
            y,
            PAD_WIDTH,
            pad_height,
            if pad.held { PRESSED } else { BORDER },
        );
        frame.rect(
            x + border,
            y + border,
            PAD_WIDTH - 2 * border,
            pad_height - 2 * border,
            BACKGROUND,
        );
        frame.rect(
            x + border + 1,
            y + border + 1,
            PAD_WIDTH - 2 * (border + 1),
            pad_height - 2 * (border + 1),
            fill,
        );
        let action = if index >= 4 {
            "SOLO"
        } else if *shifted {
            "ONLY"
        } else {
            "MUTE"
        };
        let status_y = y + (pad_height - 20).min(56);
        frame.centered_text(x + PAD_WIDTH / 2, y + 16, action, SCALE, ink);
        frame.centered_text(x + PAD_WIDTH / 2, status_y, status, SCALE, ink);
    }
}

fn render_stem_fx(
    frame: &mut Frame,
    top: usize,
    deck: u8,
    slot: u8,
    layout: Option<usize>,
    routes: [Option<bool>; 5],
) {
    const ROUTING: u16 = rgb(136, 98, 231);
    let title = format!("{} STEM FX", char::from(b'A' + deck));
    frame.centered_text(WIDTH / 2, top + 11, &title, 3, PRIMARY);
    let stems = match layout {
        Some(1) => ["STEM 1", "", "", ""],
        Some(2) => ["INST", "VOX", "", ""],
        Some(3) => ["DRUM", "HARM", "VOX", ""],
        Some(4) => ["DRUM", "BASS", "HARM", "VOX"],
        _ => ["", "", "", ""],
    };
    let rows = top + 54;
    if layout.is_none() {
        frame.centered_text(WIDTH / 2, rows + 20, "WAITING FOR DJAY", SCALE, SECONDARY);
    }
    for (index, label) in stems
        .into_iter()
        .chain(["FX 1", "FX 2", "FX 3", "DECK"])
        .enumerate()
    {
        if label.is_empty() {
            continue;
        }
        let x = MARGIN + (index % 4) * (PAD_WIDTH + GAP);
        let y = rows + (index / 4) * ((HEIGHT - rows - 8) / 2);
        let height = (HEIGHT - rows - 16) / 2;
        let state = match index {
            0..=3 => routes[index + 1],
            4..=6 => Some(index - 4 == usize::from(slot)),
            _ if layout.is_some() => routes[0],
            _ => None,
        };
        let (fill, ink, status) = match state {
            Some(true) if (4..=6).contains(&index) => (ROUTING, BACKGROUND, "EDIT"),
            Some(true) => (PRIMARY, BACKGROUND, "ON"),
            Some(false) if (4..=6).contains(&index) => (PANEL, PRIMARY, ""),
            Some(false) => (PANEL, PRIMARY, "OFF"),
            None => (PANEL, SECONDARY, "?"),
        };
        frame.rect(x, y, PAD_WIDTH, height, BORDER);
        frame.rect(x + 1, y + 1, PAD_WIDTH - 2, height - 2, fill);
        frame.centered_text(x + PAD_WIDTH / 2, y + 7, label, SCALE, ink);
        frame.centered_text(x + PAD_WIDTH / 2, y + height - 17, status, SCALE, ink);
    }
}

pub(super) fn render_headers(fx: FxWarning, backspin: Option<u8>, frame: &mut Frame) -> usize {
    let mut top = 0;
    if let Some(deck) = backspin.filter(|&deck| deck < 4) {
        let label = format!("BACKSPIN {}", char::from(b'A' + deck));
        frame.centered_text(WIDTH / 2, 4 + top, &label, SCALE, PRESSED);
        top += 24;
    }
    top + render_fx_warning(fx, top, frame)
}

fn render_fx_warning(fx: FxWarning, y: usize, frame: &mut Frame) -> usize {
    if fx.mismatch == [0; 2] && fx.unknown == [0; 2] {
        return 0;
    }
    let mut label = format!("{} FX SYNC", char::from(b'A' + fx.deck));
    for side in 0..2 {
        if fx.mismatch[side] | fx.unknown[side] == 0 {
            continue;
        }
        label.push_str(if side == 0 { " L:" } else { " R:" });
        for (knob, name) in ['1', '2', '3', 'W'].into_iter().enumerate() {
            if fx.mismatch[side] & (1 << knob) != 0 {
                label.push(name);
            } else if fx.unknown[side] & (1 << knob) != 0 {
                label.push(name);
                label.push('?');
            }
        }
    }
    frame.centered_text(WIDTH / 2, 4 + y, &label, SCALE, PRESSED);
    24
}

pub fn stopped(frame: &mut Frame) {
    frame.clear(BACKGROUND);
    for (y, text, color) in [
        (68, "BRIDGE", PRIMARY),
        (100, "STOPPED", PRIMARY),
        (148, "START TO", SECONDARY),
        (172, "RESUME", SECONDARY),
    ] {
        frame.centered_text(WIDTH / 2, y, text, SCALE, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(active: Option<bool>) -> Page {
        Page::Stems {
            shifted: false,
            fx: FxWarning::default(),
            backspin: None,
            pads: [PadState {
                active,
                held: false,
            }; 8],
        }
    }

    #[test]
    fn confirmed_activation_inverts_each_tile_without_overwriting_gutters() {
        // Given every pad confirmed active.
        let mut frame = Frame::default();
        // When the full-screen STEMS legend is rendered.
        render(&page(Some(true)), &mut frame);
        // Then all eight tile interiors are inverted within their physical slots.
        for index in 0..8 {
            let x = MARGIN + (index % 4) * (PAD_WIDTH + GAP);
            let y = PADS_Y + (index / 4) * (PAD_HEIGHT + GAP);
            assert_eq!(frame.pixels[(y + 8) * WIDTH + x + 8], PRIMARY);
        }
        for col in 0..3 {
            let x = MARGIN + col * (PAD_WIDTH + GAP) + PAD_WIDTH;
            for row in 0..HEIGHT {
                assert!(
                    frame.pixels[row * WIDTH + x..row * WIDTH + x + GAP]
                        .iter()
                        .all(|&pixel| pixel == BACKGROUND)
                );
            }
        }
    }

    #[test]
    fn unknown_state_is_visibly_different_from_confirmed_off() {
        // Given no feedback versus a real off snapshot.
        let mut unknown = Frame::default();
        let mut off = Frame::default();
        // When both are rendered.
        render(&page(None), &mut unknown);
        render(&page(Some(false)), &mut off);
        // Then the status pixels differ without requiring a color distinction.
        assert_ne!(unknown.pixels, off.pixels);
        assert_eq!(unknown.pixels[(PADS_Y + 8) * WIDTH + MARGIN + 8], PANEL);
        assert_eq!(off.pixels[(PADS_Y + 8) * WIDTH + MARGIN + 8], PANEL);
        let status_pixels: Vec<_> = (PADS_Y + 56..PADS_Y + 72)
            .flat_map(|y| (MARGIN..MARGIN + PAD_WIDTH).map(move |x| y * WIDTH + x))
            .collect();
        assert_ne!(
            status_pixels
                .iter()
                .map(|&i| unknown.pixels[i] == SECONDARY)
                .collect::<Vec<_>>(),
            status_pixels
                .iter()
                .map(|&i| off.pixels[i] == PRIMARY)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn held_outline_does_not_change_confirmed_fill_or_other_tiles() {
        // Given an active pad, with and without its physical press.
        let mut held = [PadState {
            active: Some(true),
            held: false,
        }; 8];
        held[0].held = true;
        let mut normal_frame = Frame::default();
        let mut held_frame = Frame::default();
        // When the physical press is rendered.
        render(&page(Some(true)), &mut normal_frame);
        render(
            &Page::Stems {
                shifted: false,
                pads: held,
                fx: FxWarning::default(),
                backspin: None,
            },
            &mut held_frame,
        );
        // Then only that tile changes and its confirmed interior stays active.
        assert_ne!(normal_frame.pixels, held_frame.pixels);
        assert_eq!(
            held_frame.pixels[(PADS_Y + 8) * WIDTH + MARGIN + 8],
            PRIMARY
        );
        for (index, (&normal, &pressed)) in normal_frame
            .pixels
            .iter()
            .zip(&held_frame.pixels)
            .enumerate()
        {
            if !(MARGIN..MARGIN + PAD_WIDTH).contains(&(index % WIDTH))
                || !(PADS_Y..PADS_Y + PAD_HEIGHT).contains(&(index / WIDTH))
            {
                assert_eq!(normal, pressed);
            }
        }
    }

    #[test]
    fn quiet_page_erases_all_previous_pad_content() {
        // Given a reused STEMS frame.
        let mut frame = Frame::default();
        render(&page(Some(true)), &mut frame);
        // When a quiet page replaces it.
        render(&Page::Empty, &mut frame);
        // Then no old state or legend survives.
        assert!(frame.pixels.iter().all(|&pixel| pixel == BACKGROUND));
    }

    #[test]
    fn stopped_replaces_every_previous_role_pixel() {
        // Given an active frame and a fresh frame.
        let mut reused = Frame::default();
        render(&page(Some(true)), &mut reused);
        let mut fresh = Frame::default();
        // When both receive the stopped page.
        stopped(&mut reused);
        stopped(&mut fresh);
        // Then stopped output is independent of prior roles and visibly populated.
        assert_eq!(reused.pixels, fresh.pixels);
        assert!(reused.pixels.contains(&PRIMARY));
        assert!(!reused.pixels.contains(&PRESSED));
    }

    #[test]
    fn backspin_and_fx_headers_coexist_on_stems_and_clear_to_default_raster() {
        // Given STEMS pages with no headers, backspin only, FX only, and both.
        let pads = [PadState {
            active: Some(true),
            held: false,
        }; 8];
        let fx = FxWarning {
            deck: 0,
            mismatch: [1, 0],
            unknown: [0; 2],
        };
        let render_stems = |fx, backspin| {
            let mut frame = Frame::default();
            render(
                &Page::Stems {
                    shifted: false,
                    pads,
                    fx,
                    backspin,
                },
                &mut frame,
            );
            frame
        };
        let default = render_stems(FxWarning::default(), None);
        let fx_only = render_stems(fx, None);
        let backspin_a = render_stems(FxWarning::default(), Some(0));
        let backspin_c = render_stems(FxWarning::default(), Some(2));
        let both = render_stems(fx, Some(2));
        // Then each deck letter and header combination renders distinct amber header rows
        // while keeping all eight stem tiles populated, and clearing restores exact default pixels.
        assert_ne!(default.pixels, backspin_a.pixels);
        assert_ne!(backspin_a.pixels, backspin_c.pixels);
        assert_ne!(fx_only.pixels, backspin_c.pixels);
        assert_ne!(both.pixels, backspin_c.pixels);
        assert_ne!(both.pixels, fx_only.pixels);
        assert!(!default.pixels[..24 * WIDTH].contains(&PRESSED));
        assert!(backspin_c.pixels[..24 * WIDTH].contains(&PRESSED));
        assert!(!backspin_c.pixels[24 * WIDTH..48 * WIDTH].contains(&PRESSED));
        assert!(both.pixels[..24 * WIDTH].contains(&PRESSED));
        assert!(both.pixels[24 * WIDTH..48 * WIDTH].contains(&PRESSED));
        for index in 0..8 {
            let x = MARGIN + (index % 4) * (PAD_WIDTH + GAP);
            let y = PADS_Y + 48 + (index / 4) * (PAD_HEIGHT - 24 + GAP);
            assert_eq!(both.pixels[(y + 8) * WIDTH + x + 8], PRIMARY);
        }
        let mut cleared = both;
        render(&page(Some(true)), &mut cleared);
        assert_eq!(cleared.pixels, default.pixels);
    }
}
