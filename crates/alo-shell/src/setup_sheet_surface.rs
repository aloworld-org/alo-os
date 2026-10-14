//! The rounded shapes on a first-start sheet, rasterised with their corners.
//!
//! The choice rows are cards and the first action is a filled button, and a
//! flat [`Solid`](crate::painted::Solid) is an axis-aligned rectangle with no
//! corner at all. `crate::sign_in_surface` met this first for the sign-in
//! entry and answered it with a signed distance field antialiased by one
//! `0.5 - d` rule; this is that answer for the sheets, and it reuses that
//! file's `TheColours` so a sheet follows the person's scheme and contrast for
//! the same reason the sign-in screen does.
//!
//! # What this draws
//!
//! The wallpaper — the same artwork `crate::sign_in_wallpaper` already
//! rasterises, because the capture shows the first-start sheets carry the
//! identical five vectors — then a card behind each choice row, and the filled
//! ground behind the first action.
//!
//! **Not drawn: text.** That is `crate::painted_text`'s, inked over this.

use smithay::utils::{Physical, Rectangle, Scale};

use crate::painted::Washed;
use crate::setup_sheet_layout::SetupSheetLayout;
use crate::sign_in_surface::TheColours;

/// A choice card's corner, and the filled action's.
const THE_CORNER_IS: i32 = 14;

/// How thick a card's hairline is.
const THE_HAIRLINE_IS: f32 = 1.0;

/// Rasterise a sheet's shapes for this layout, at this scale.
///
/// [`None`] when the sheet has no area, which is the one case with nothing to
/// draw rather than a refusal worth reporting.
#[must_use]
pub(crate) fn setup_sheet_surface(
    at: &SetupSheetLayout,
    colours: TheColours,
    scale: Scale<f64>,
) -> Option<Washed> {
    let area: Rectangle<i32, Physical> = at.sheet.to_physical_precise_round(scale);
    if area.size.w <= 0 || area.size.h <= 0 {
        return None;
    }
    let (w, h) = (area.size.w, area.size.h);

    // The same wallpaper the sign-in screen draws, on the same ground.
    let ground = crate::sign_in_wallpaper::the_wallpaper(w, h, colours.ground);

    let corner = f64::from(THE_CORNER_IS) * scale.x;
    let hairline = f64::from(THE_HAIRLINE_IS) * scale.x;
    let cards: Vec<Rectangle<i32, Physical>> = at
        .choices
        .iter()
        .map(|it| it.to_physical_precise_round::<f64, i32>(scale))
        .collect();
    // **Only the first action is filled.** The design draws *Continue* and
    // *Save PIN* on a navy ground and *Back* and *Use password only* as an
    // outline, which is how a person tells the way on from the way back
    // without reading either.
    let filled = at
        .actions
        .first()
        .map(|it| it.to_physical_precise_round::<f64, i32>(scale));
    let outlined: Vec<Rectangle<i32, Physical>> = at
        .actions
        .iter()
        .skip(1)
        .map(|it| it.to_physical_precise_round::<f64, i32>(scale))
        .collect();

    let mut pixels = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            let px = f64::from(x) + 0.5;
            let py = f64::from(y) + 0.5;
            let mut colour = ground
                .get(((y * w + x) as usize).min(ground.len().saturating_sub(1)))
                .copied()
                .unwrap_or(colours.ground);

            // A card behind each choice: the surface, then its hairline.
            for card in &cards {
                let outer = crate::sign_in_surface::how_far(px, py, *card, corner);
                let inside = crate::sign_in_surface::coverage(outer);
                if inside > 0.0 {
                    colour = crate::sign_in_surface::mixed(colour, colours.entry, inside);
                    let inner = crate::sign_in_surface::how_far(
                        px,
                        py,
                        crate::sign_in_surface::inset(*card, hairline),
                        (corner - hairline).max(0.0),
                    );
                    let ring = inside - crate::sign_in_surface::coverage(inner);
                    if ring > 0.0 {
                        colour = crate::sign_in_surface::mixed(colour, colours.border, ring);
                    }
                }
            }

            // The way on: filled with the ink itself.
            if let Some(filled) = filled {
                let inside = crate::sign_in_surface::coverage(crate::sign_in_surface::how_far(
                    px, py, filled, corner,
                ));
                if inside > 0.0 {
                    colour = crate::sign_in_surface::mixed(colour, colours.ink, inside);
                }
            }

            // The ways back: the sheet's own ground inside a hairline.
            for action in &outlined {
                let outer = crate::sign_in_surface::how_far(px, py, *action, corner);
                let inside = crate::sign_in_surface::coverage(outer);
                if inside > 0.0 {
                    colour = crate::sign_in_surface::mixed(colour, colours.entry, inside);
                    let inner = crate::sign_in_surface::how_far(
                        px,
                        py,
                        crate::sign_in_surface::inset(*action, hairline),
                        (corner - hairline).max(0.0),
                    );
                    let ring = inside - crate::sign_in_surface::coverage(inner);
                    if ring > 0.0 {
                        colour = crate::sign_in_surface::mixed(colour, colours.border, ring);
                    }
                }
            }

            pixels.extend_from_slice(&[colour[0], colour[1], colour[2], 0xFF]);
        }
    }
    Some(Washed { area, pixels })
}

#[cfg(test)]
#[path = "setup_sheet_surface_tests.rs"]
mod tests;
