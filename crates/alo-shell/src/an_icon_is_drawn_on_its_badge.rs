//! **An application's own icon, composited onto the Dock's surface** where its
//! badge is.
//!
//! [ADR 0101](../../../docs/decisions/0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md),
//! the owner: *real applications retain their actual application icons … first
//! letters are a missing-icon fallback, not the finished Dock.*
//!
//! `crate::dock_surface` already draws a circular badge for each application
//! and its own documentation says what is supposed to happen next: each badge
//! is *centred in its 48 × 48 target — **so a real icon replaces one without
//! the slot moving or resizing***. This is that replacement.
//!
//! # Onto the finished buffer, not into the distance field
//!
//! `dock_surface` builds its panel with one per-pixel pass over signed
//! distance functions — a circle, a rounded bar, a well — because every one of
//! those is partial coverage at its edge and no number of rectangles makes one.
//! A photograph is not a distance function. Sampling one inside that loop would
//! put an image decoder's concerns in the middle of the file that draws the
//! shell's only rounded corners, and law 4 asks for the split at the moment it
//! is discovered rather than after.
//!
//! So the panel is finished first and this runs over the result: ordinary
//! source-over compositing, one rectangle at a time, which is what a picture
//! with an alpha channel needs and all it needs.
//!
//! # Why the badge stays underneath
//!
//! The badge is not erased before the icon lands on it. Most application icons
//! are opaque squares and cover it completely; the ones that are not — a
//! rounded icon, one with a transparent border — are **meant** to show the
//! badge behind them, because that circle is what gives every slot on the Dock
//! the same silhouette. Clearing it first would make those icons float.

use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;

/// Draw each picture onto this surface at the rectangle given for it.
///
/// Each rectangle is in the same output pixels as `onto.area`, and a picture is
/// scaled to it. Anything that would fall outside the surface is clipped
/// rather than refused: a Dock narrower than its contents is a layout question
/// and this is not the file that answers it.
pub(crate) fn pictures_onto(
    onto: &mut Washed,
    pictures: &[(Rectangle<i32, Physical>, &alo_dock::artwork::APicture)],
) {
    let (surface_wide, surface_tall) = (onto.area.size.w, onto.area.size.h);
    if surface_wide <= 0 || surface_tall <= 0 {
        return;
    }
    for (at, picture) in pictures {
        if at.size.w <= 0 || at.size.h <= 0 {
            continue;
        }
        let Ok(wide) = u32::try_from(at.size.w) else {
            continue;
        };
        let Ok(tall) = u32::try_from(at.size.h) else {
            continue;
        };
        let scaled = picture.scaled_to(wide, tall);
        // Where the box sits inside the surface's own buffer, which starts at
        // `onto.area.loc` and not at the origin of the display.
        let (offset_x, offset_y) = (at.loc.x - onto.area.loc.x, at.loc.y - onto.area.loc.y);
        for row in 0..at.size.h {
            let y = offset_y + row;
            if y < 0 || y >= surface_tall {
                continue;
            }
            for column in 0..at.size.w {
                let x = offset_x + column;
                if x < 0 || x >= surface_wide {
                    continue;
                }
                let from = (row as usize * at.size.w as usize + column as usize) * 4;
                let onto_at = (y as usize * surface_wide as usize + x as usize) * 4;
                let Some(source) = scaled.pixels().get(from..from + 4) else {
                    continue;
                };
                let Some(target) = onto.pixels.get_mut(onto_at..onto_at + 4) else {
                    continue;
                };
                over(target, source);
            }
        }
    }
}

/// One pixel of `source` drawn over one pixel of `target`, in place.
///
/// **Source-over, on straight (not premultiplied) colour**, which is what both
/// buffers hold: `Washed` is four bytes of red, green, blue then alpha, and so
/// is `alo_dock::artwork::APicture`. The result's colour is the two mixed by
/// how much of each is there, and its opacity is what you get looking through
/// one at the other.
///
/// A fully transparent source leaves the target exactly as it was, which is
/// what keeps the badge visible around a rounded icon.
fn over(target: &mut [u8], source: &[u8]) {
    // **Four bytes at a time and never an index**, which the lints ask for and
    // which is right in a compositor besides: a slice that was not four long
    // would panic inside a frame, and every caller's bounds here are arithmetic
    // on two rectangles. A pixel that is not there is simply not drawn.
    let (Ok(from), Ok(onto)) = (<[u8; 4]>::try_from(source), <[u8; 4]>::try_from(&*target)) else {
        return;
    };
    let [source_red, source_green, source_blue, source_opacity] = from;
    let [target_red, target_green, target_blue, target_opacity] = onto;
    let source_alpha = u32::from(source_opacity);
    if source_alpha == 0 {
        return;
    }
    if source_alpha == 255 {
        target.copy_from_slice(source);
        return;
    }
    // How much of the target shows through: all of it, less what the source
    // covers.
    let kept = u32::from(target_opacity) * (255 - source_alpha) / 255;
    let total = source_alpha + kept;
    let mixed = |source_channel: u8, target_channel: u8| {
        (u32::from(source_channel) * source_alpha + u32::from(target_channel) * kept)
            .checked_div(total)
            .map_or(0, |each| u8::try_from(each).unwrap_or(u8::MAX))
    };
    target.copy_from_slice(&[
        mixed(source_red, target_red),
        mixed(source_green, target_green),
        mixed(source_blue, target_blue),
        u8::try_from(total).unwrap_or(u8::MAX),
    ]);
}

#[cfg(test)]
#[path = "an_icon_is_drawn_on_its_badge_tests.rs"]
mod tests;
