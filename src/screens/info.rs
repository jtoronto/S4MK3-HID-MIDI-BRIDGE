//! Four-quadrant live-data page, independent of MIDI and USB.

use super::{Frame, HEIGHT, WIDTH, rgb};

const BACKGROUND: u16 = rgb(10, 12, 18);
const PRIMARY: u16 = rgb(244, 248, 255);
const SECONDARY: u16 = rgb(164, 178, 202);
const AMBER: u16 = rgb(252, 176, 36);
const GREEN: u16 = rgb(0, 220, 80);
const PICKUP_RED: u16 = rgb(255, 156, 156);
const BLACK: u16 = 0;
const DIVIDER: u16 = rgb(60, 70, 90);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Info {
    pub jump: Option<&'static str>,
    pub loop_active: Option<bool>,
    pub loop_size: Option<&'static str>,
    pub key: Option<&'static str>,
    pub tempo_range: Option<u8>,
    pub tempo_pickup: Option<TempoPickup>,
    pub selecting: bool,
    pub fx: super::page::FxWarning,
    pub backspin: Option<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoPickup {
    physical: u8,
    target: u8,
}

impl TempoPickup {
    #[must_use]
    pub const fn new(physical: u8, target: u8) -> Option<Self> {
        if physical <= 127 && target <= 127 && physical.abs_diff(target) > 1 {
            Some(Self { physical, target })
        } else {
            None
        }
    }

    fn percent(self) -> u16 {
        (u16::from(self.physical.abs_diff(self.target)) * 100 + 63) / 127
    }

    fn render(self, frame: &mut Frame, x: usize, y: usize, height: usize) {
        let center = x + WIDTH / 4;
        frame.centered_text(center, y + 9, "PICKUP", 2, BLACK);
        let remaining = format!("{}%", self.percent());
        frame.centered_text(center, y + 27, &remaining, 6, BLACK);

        let left = x + 25;
        let right = x + WIDTH / 2 - 25;
        let y = y + height - 30;
        let coordinate = |value| left + (usize::from(value) * (right - left) + 63) / 127;
        let physical = coordinate(self.physical);
        let target = coordinate(self.target);
        frame.rect(x + 6, y, 12, 3, BLACK);
        frame.rect(right + 7, y, 12, 3, BLACK);
        frame.rect(right + 12, y - 5, 3, 13, BLACK);
        frame.rect(left, y, right - left + 1, 3, DIVIDER);
        frame.rect(
            physical.min(target),
            y - 1,
            physical.abs_diff(target) + 1,
            5,
            BLACK,
        );
        frame.rect(target - 1, y - 9, 4, 14, BLACK);
        // Keep the arrow below the rail so even a two-step gap leaves the tick visible.
        for step in 0..=13 {
            let x = if self.physical < self.target {
                physical - step
            } else {
                physical + step
            };
            let half = step * 8 / 13;
            frame.rect(x, y + 16 - half, 1, half * 2 + 1, BLACK);
        }
    }
}

const fn range_label(range: Option<u8>) -> &'static str {
    match range {
        Some(4) => "±4%",
        Some(6) => "±6%",
        Some(8) => "±8%",
        Some(10) => "±10%",
        Some(16) => "±16%",
        Some(25) => "±25%",
        Some(50) => "±50%",
        Some(75) => "±75%",
        Some(_) | None => "?",
    }
}

pub(super) fn render(info: &Info, frame: &mut Frame, bright: bool) {
    let top = super::page::render_headers(info.fx, info.backspin, frame);
    let quadrant_height = (HEIGHT - top) / 2;
    let loop_value = match info.loop_active {
        Some(true) => info.loop_size.unwrap_or("?"),
        Some(false) => "OFF",
        None => "?",
    };
    for (index, (label, value)) in [
        ("JUMP", info.jump.unwrap_or("?")),
        ("LOOP", loop_value),
        ("KEY", info.key.unwrap_or("?")),
        ("RANGE", range_label(info.tempo_range)),
    ]
    .into_iter()
    .enumerate()
    {
        let x = (index % 2) * (WIDTH / 2);
        let y = top + (index / 2) * quadrant_height;
        if index == 3
            && let Some(pickup) = info.tempo_pickup
        {
            frame.rect(x, y, WIDTH / 2, quadrant_height, PICKUP_RED);
            pickup.render(frame, x, y, quadrant_height);
            continue;
        }
        let selecting = index == 0 && info.selecting;
        let fill = if index == 1 && info.loop_active == Some(true) {
            GREEN
        } else if selecting && bright {
            AMBER
        } else {
            BACKGROUND
        };
        let ink = if fill == BACKGROUND { PRIMARY } else { BLACK };
        frame.rect(x, y, WIDTH / 2, quadrant_height, fill);
        let label_y = y + match top {
            0 => 12,
            24 => 8,
            _ => 6,
        };
        frame.text(
            x + 12,
            label_y,
            label,
            2,
            if fill == BACKGROUND { SECONDARY } else { BLACK },
        );
        let scale = [6, 5, 4]
            .into_iter()
            .find(|&scale| Frame::text_width(value, scale) <= 136)
            .unwrap_or(4);
        frame.centered_text(
            x + WIDTH / 4,
            y + if top == 0 {
                44
            } else if top == 24 {
                34
            } else {
                28
            },
            value,
            scale,
            if index == 1 && info.loop_active == Some(false) {
                SECONDARY
            } else {
                ink
            },
        );
        if selecting {
            frame.centered_text(
                x + WIDTH / 4,
                y + if top == 0 {
                    100
                } else if top == 24 {
                    88
                } else {
                    76
                },
                "SELECT",
                2,
                if bright { BLACK } else { AMBER },
            );
        }
    }
    frame.rect(WIDTH / 2, top, 1, HEIGHT - top, DIVIDER);
    frame.rect(0, top + quadrant_height, WIDTH, 1, DIVIDER);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(info: Info, bright: bool) -> Frame {
        let mut frame = Frame::default();
        render(&info, &mut frame, bright);
        frame
    }

    #[test]
    fn tempo_rail_marks_the_target_and_points_toward_it_in_either_direction() -> anyhow::Result<()>
    {
        use anyhow::Context as _;
        for (physical, target, arrow_base, arrow_tip, target_x) in
            [(0, 127, 172, 185, 295), (127, 0, 308, 295, 185)]
        {
            let pickup = TempoPickup::new(physical, target).context("opposite endpoints")?;
            assert_eq!(pickup.percent(), 100);
            let info = Info {
                tempo_range: Some(4),
                tempo_pickup: Some(pickup),
                ..Info::default()
            };
            let image = frame(info, true);
            assert!(
                frame(
                    Info {
                        tempo_range: Some(75),
                        ..info
                    },
                    true,
                )
                .pixels
                    == image.pixels,
                "the configured range must not be drawn during pickup"
            );
            assert_eq!(image.pixels[200 * WIDTH + 170], PICKUP_RED);
            assert_eq!(image.pixels[226 * WIDTH + arrow_tip], BLACK);
            assert_eq!(image.pixels[234 * WIDTH + arrow_base], BLACK);
            assert!(
                image.pixels[235 * WIDTH..]
                    .chunks_exact(WIDTH)
                    .all(|row| row[WIDTH / 2 + 1..]
                        .iter()
                        .all(|&pixel| pixel == PICKUP_RED)),
                "leave clear space below the arrow for the bezel"
            );
            assert_eq!(image.pixels[202 * WIDTH + target_x], BLACK);
            assert!(image.pixels[169 * WIDTH + 175..179 * WIDTH].contains(&BLACK));
        }
        assert!(TempoPickup::new(64, 64).is_none());
        assert!(TempoPickup::new(63, 64).is_none());
        assert_eq!(
            TempoPickup::new(62, 127).map(TempoPickup::percent),
            Some(51)
        );
        Ok(())
    }

    #[test]
    fn active_loop_has_green_fill_and_black_ink_only_in_its_quadrant() {
        // Given a confirmed active loop.
        let info = Info {
            loop_active: Some(true),
            loop_size: Some("8"),
            ..Info::default()
        };
        // When drawn at native resolution.
        let actual = frame(info, true);
        // Then green fills its quadrant and its label/value have black pixels.
        assert_eq!(actual.pixels[8 * WIDTH + 168], GREEN);
        assert!(actual.pixels[12 * WIDTH + 172..28 * WIDTH].contains(&BLACK));
        for (index, &pixel) in actual.pixels.iter().enumerate() {
            if pixel == GREEN {
                assert!(index % WIDTH > 160 && index / WIDTH < 120);
            }
        }
    }

    #[test]
    fn inactive_loop_never_shows_a_cached_size() {
        // Given an inactive loop with different old sizes.
        let info = Info {
            loop_active: Some(false),
            loop_size: Some("8"),
            ..Info::default()
        };
        // When rendered with stale size changed or absent.
        let actual = frame(info, true);
        // Then all OFF rasters are identical, while unknown and active differ.
        assert_eq!(
            actual.pixels,
            frame(
                Info {
                    loop_size: None,
                    ..info
                },
                true
            )
            .pixels
        );
        assert_ne!(actual.pixels, frame(Info::default(), true).pixels);
        assert!(!actual.pixels.contains(&GREEN));
    }

    #[test]
    fn selection_phases_preserve_number_geometry_and_other_quadrants() {
        // Given jump-size selection with known data.
        let info = Info {
            jump: Some("16"),
            selecting: true,
            ..Info::default()
        };
        // When both flash phases render.
        let bright = frame(info, true);
        let dim = frame(info, false);
        // Then the same number pixels remain visible; only JUMP changes.
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let index = y * WIDTH + x;
                if x >= 160 || y >= 120 {
                    assert_eq!(bright.pixels[index], dim.pixels[index]);
                } else if (44..92).contains(&y) {
                    assert_eq!(
                        bright.pixels[index] != AMBER,
                        dim.pixels[index] != BACKGROUND
                    );
                }
            }
        }
        assert_ne!(bright.pixels, dim.pixels);
    }

    #[test]
    fn long_values_and_plus_minus_stay_inside_their_quadrants() {
        // Given the longest fractional lengths, key and range.
        let info = Info {
            jump: Some("1/32"),
            loop_active: Some(true),
            loop_size: Some("1/16"),
            key: Some("12B"),
            tempo_range: Some(75),
            ..Info::default()
        };
        // When rendering into a native frame.
        let actual = frame(info, true);
        // Then margins remain clear around every value, and ± is real ink.
        for (x, y) in [(0, 0), (160, 0), (0, 120), (160, 120)] {
            let fill = if x == 160 && y == 0 {
                GREEN
            } else {
                BACKGROUND
            };
            for row in y + 44..y + 92 {
                assert_eq!(actual.pixels[row * WIDTH + x + 4], fill);
                assert_eq!(actual.pixels[row * WIDTH + x + 156], fill);
            }
        }
        assert!(
            (164..196).any(|y| actual.pixels[y * WIDTH + 176..y * WIDTH + 208].contains(&PRIMARY))
        );
    }

    #[test]
    fn backspin_and_fx_headers_reflow_info_quadrants_and_clear_cleanly() {
        // Given populated info fields with each deck owner and optional FX warning.
        let base = Info {
            jump: Some("16"),
            loop_active: Some(true),
            loop_size: Some("8"),
            key: Some("9A"),
            tempo_range: Some(8),
            selecting: true,
            ..Info::default()
        };
        let fx = super::super::page::FxWarning {
            deck: 0,
            mismatch: [1, 2],
            unknown: [0; 2],
        };
        let default = frame(base, true);
        let backspin_decks: [Frame; 4] = std::array::from_fn(|deck| {
            frame(
                Info {
                    backspin: u8::try_from(deck).ok(),
                    ..base
                },
                true,
            )
        });
        let fx_only = frame(Info { fx, ..base }, true);
        let both = frame(
            Info {
                fx,
                backspin: Some(2),
                ..base
            },
            true,
        );
        // Then each deck A-D has a distinct header, both headers coexist in the top 48 rows,
        // all four quadrants remain populated, and clearing backspin restores the default raster.
        for left in 0..4 {
            assert_ne!(default.pixels, backspin_decks[left].pixels);
            assert!(backspin_decks[left].pixels[..24 * WIDTH].contains(&AMBER));
            for right in left + 1..4 {
                assert_ne!(
                    backspin_decks[left].pixels[..24 * WIDTH],
                    backspin_decks[right].pixels[..24 * WIDTH]
                );
            }
        }
        assert_eq!(
            &both.pixels[..24 * WIDTH],
            &backspin_decks[2].pixels[..24 * WIDTH]
        );
        assert!(both.pixels[24 * WIDTH..48 * WIDTH].contains(&AMBER));
        assert_ne!(both.pixels, fx_only.pixels);
        assert_eq!(both.pixels[56 * WIDTH + 8], AMBER);
        assert_eq!(both.pixels[56 * WIDTH + 168], GREEN);
        assert!(both.pixels[152 * WIDTH..236 * WIDTH].contains(&PRIMARY));
        let mut cleared = both;
        render(&base, &mut cleared, true);
        assert_eq!(cleared.pixels, default.pixels);
    }
}
