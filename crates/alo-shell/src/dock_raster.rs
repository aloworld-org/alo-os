//! The dock on one display: a floating bar, centred, above the bottom edge.
//!
//! # Whose decisions these are
//!
//! How thick it is and how wide it needs to be are `alo-dock`'s answers for this
//! display's own size, the person's text size and how many icons there are. That
//! it is along the **bottom** is [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md),
//! not a value asked for per display. This file turns those answers into pixels
//! and decides none of them. Asked once per display, so a laptop and the screen
//! beside it each get the dock laid out for their own size — and a dock that
//! gives its names way on the small one keeps them on the large one.
//!
//! # A bar, not a band
//!
//! It drew a band across the whole width, flush to the bottom edge, from before
//! ADR 0076 until now. **The designs show a bar**: as wide as what it holds,
//! centred, with room underneath it and either side.
//!
//! That is not decoration. A band is part of the screen's frame; a bar is a
//! thing lying on the canvas, which is what the Dock is — the canvas moves under
//! it and it does not move with the canvas. The gap beneath it is
//! `alo_dock::measures::FLOATING_ABOVE_THE_EDGE`.
//!
//! **What it holds is passed in.** This file does not count icons: how many
//! there are is `alo_dock::Holding`'s answer and how many *fit* is
//! `alo_dock::fit`'s, so a bar drawn here cannot disagree with the list the Dock
//! decided to show.
//!
//! # When it will not fit
//!
//! A bar wider than the screen is clamped to the screen's width less its
//! margins, and what does not fit went into the overflow before it ever reached
//! this file. Clamping is the last resort rather than the mechanism: shrinking
//! icons to fit more is how a dock becomes unusable exactly when somebody has
//! the most open, which `alo_dock::fit` refuses on the way in.
//!
//! # The status area is not drawn here any more
//!
//! This file drew a segment at the far end of the band, set off by a rule in ink,
//! and called it the status area. ADR 0076 took the status area off the Dock: a
//! clock is not something a person opens or brings into focus, so it is not the
//! Dock's, and *where it goes instead* is handed to the shell's own plan rather
//! than answered here.
//!
//! **The egress indicator was not left floating with it.** It still sits where it
//! sat, and `crate::egress_status_place` now works its corner out from the screen
//! and the dock's thickness directly — which is what that file always did, minus
//! one indirection through a type the Dock no longer has. Nothing about where a
//! person looks for *nothing has left this machine* changed in this record.
//!
//! # Furniture, not authority
//!
//! Nothing here grants, approves or revokes, and nothing here measures: the
//! dock is a band of the person's colours with the accent along its inside
//! edge. `tests/desktop_source.rs` reads these files to hold that.

use alo_dock::measures::{FLOATING_ABOVE_THE_EDGE, MARGIN};
use alo_dock::{Dock, Layout, Room, Screen};
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;
use crate::desktop_look::DesktopLook;
use crate::painted::Solid;

/// The largest display side, in pixels, the dock is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The dock for one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// `alo-dock`'s layout for this display.
    pub(crate) layout: Layout,
    /// The whole band, along the bottom.
    pub(crate) band: Rectangle<i32, Physical>,
    /// The accent along the band's inside — that is, top — edge.
    pub(crate) accent: Rectangle<i32, Physical>,
    /// Flat shapes, in painting order.
    pub(crate) solids: Vec<Solid>,
}

/// Lay `dock` out on a display of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::DesktopScene`] for a display `alo-dock` refuses to lay a dock
/// out on, or one larger than any this dock is laid out for;
/// [`RenderError::AccentRefused`] for a look whose accent is not one a person
/// can choose.
pub(crate) fn picture(
    dock: &Dock,
    look: DesktopLook,
    size: (i32, i32),
    holding: usize,
) -> Result<DockPicture, RenderError> {
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::DesktopScene);
    }
    let screen = Screen::of(
        u32::try_from(width).map_err(|_| RenderError::DesktopScene)?,
        u32::try_from(height).map_err(|_| RenderError::DesktopScene)?,
    )
    .map_err(|_| RenderError::DesktopScene)?;
    let palette = look.palette().map_err(|_| RenderError::AccentRefused)?;
    let measure = look.measure();
    let layout = dock.layout_on(screen, look.scale());
    let thickness = i32::try_from(layout.thickness().as_pixels())
        .map_err(|_| RenderError::DesktopScene)?
        .clamp(1, width.min(height));
    let rule = measure.px(2).min(thickness);

    // As wide as what it holds, clamped to the screen less its margins. The
    // count comes from whoever decided what the Dock shows, so the bar drawn
    // and the list decided cannot disagree.
    let margin = i32::try_from(MARGIN).unwrap_or(i32::MAX);
    let floating = i32::try_from(FLOATING_ABOVE_THE_EDGE).unwrap_or(i32::MAX);
    let wanted = i32::try_from(Room::a_bar_holding(holding).as_pixels())
        .map_err(|_| RenderError::DesktopScene)?;
    let widest = (width - 2 * margin).max(1);
    let bar_width = wanted.clamp(1, widest);

    // Centred across, and lifted clear of the bottom edge.
    let band = Rectangle::new(
        ((width - bar_width) / 2, height - thickness - floating).into(),
        (bar_width, thickness).into(),
    );
    let accent = Rectangle::new(band.loc, (bar_width, rule).into());

    let solids = vec![
        Solid {
            area: band,
            colour: palette.dock,
        },
        Solid {
            area: accent,
            colour: palette.accent,
        },
    ];
    Ok(DockPicture {
        size,
        layout,
        band,
        accent,
        solids,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::desktop_testing::{an_appearance, noon_look};
    use alo_appearance::{Accent, TextScale};
    use alo_strings::Direction;

    /// Whether `inner` lies wholly inside `outer`.
    fn inside(inner: Rectangle<i32, Physical>, outer: Rectangle<i32, Physical>) -> bool {
        outer.intersection(inner) == Some(inner)
    }

    /// **The dock is a bar: centred, clear of the bottom edge, and as wide as
    /// what it holds** — the same bar whichever way the person reads.
    #[test]
    fn the_dock_is_a_centred_bar_clear_of_the_bottom_edge() {
        let size = (1920, 1080);
        let dock = Dock::shipped();
        let mut whichever_way_read: Option<Rectangle<i32, Physical>> = None;
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let look = noon_look(&an_appearance(), reading);
            let drawn = picture(&dock, look, size, 4).unwrap();
            let layout = dock.layout_on(Screen::of(1920, 1080).unwrap(), look.scale());
            assert_eq!(drawn.layout, layout);

            let thick = i32::try_from(layout.thickness().as_pixels()).unwrap();
            let floating = i32::try_from(FLOATING_ABOVE_THE_EDGE).unwrap();

            // Clear of the edge, with room underneath it.
            assert_eq!(
                drawn.band.loc.y + drawn.band.size.h,
                1080 - floating,
                "{reading:?}: it is not floating"
            );
            assert_eq!(drawn.band.size.h, thick, "{reading:?}");

            // As wide as what it holds, and nothing like the whole screen.
            let wanted = i32::try_from(Room::a_bar_holding(4).as_pixels()).unwrap();
            assert_eq!(drawn.band.size.w, wanted, "{reading:?}");
            assert!(drawn.band.size.w < 1920, "{reading:?}: it spans the screen");

            // Centred: the room either side is equal to within a pixel.
            let left = drawn.band.loc.x;
            let right = 1920 - (drawn.band.loc.x + drawn.band.size.w);
            assert!((left - right).abs() <= 1, "{reading:?}: {left} vs {right}");

            assert!(inside(drawn.accent, drawn.band), "{reading:?}");
            assert_eq!(drawn.accent.loc.y, drawn.band.loc.y, "the inside edge");

            match whichever_way_read {
                None => whichever_way_read = Some(drawn.band),
                Some(first) => {
                    assert_eq!(first, drawn.band, "which way a person reads moved the dock");
                }
            }
        }
        assert!(whichever_way_read.is_some(), "neither way round was drawn");
    }

    /// **A bar holding more is wider**, which is what says the width comes from
    /// the contents rather than from the screen.
    #[test]
    fn a_bar_holding_more_is_wider() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let dock = Dock::shipped();
        let mut last = 0;
        for holding in [0_usize, 1, 4, 9] {
            let drawn = picture(&dock, look, (1920, 1080), holding).unwrap();
            assert!(
                drawn.band.size.w > last,
                "holding {holding} was not wider than the one before"
            );
            last = drawn.band.size.w;
        }
    }

    /// **A bar too wide for the screen is clamped rather than drawn off it**,
    /// and what did not fit went into the overflow before it reached this file.
    #[test]
    fn a_bar_wider_than_the_screen_is_clamped_to_it() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        let drawn = picture(&Dock::shipped(), look, (1366, 768), 200).unwrap();
        let margin = i32::try_from(MARGIN).unwrap();

        assert!(drawn.band.size.w <= 1366 - 2 * margin);
        assert!(drawn.band.loc.x >= 0);
        assert!(drawn.band.loc.x + drawn.band.size.w <= 1366);
    }

    /// **Per display.** The same dock on a laptop and on a large screen beside
    /// it is laid out for each display's own size: at a large text size its
    /// names give way on the laptop and are kept on the desk, and the two bands
    /// are as thick as `alo-dock` says for each.
    #[test]
    fn each_display_gets_the_dock_laid_out_for_its_own_size() {
        let mut appearance = an_appearance();
        appearance.set_text(TextScale::percent(300).unwrap());
        let look = noon_look(&appearance, Direction::LeftToRight);
        let dock = Dock::shipped();

        let laptop = picture(&dock, look, (1366, 768), 4).unwrap();
        let desk = picture(&dock, look, (3840, 2160), 4).unwrap();
        let portrait = picture(&dock, look, (1080, 1920), 4).unwrap();
        assert!(!laptop.layout.labels().are_shown());
        assert!(desk.layout.labels().are_shown());
        assert_ne!(laptop.band.size.h, desk.band.size.h);
        for drawn in [&laptop, &desk, &portrait] {
            assert_eq!(
                i64::from(drawn.band.size.h),
                i64::from(drawn.layout.thickness().as_pixels())
            );
            assert_eq!(
                drawn.band.loc.y + drawn.band.size.h,
                drawn.size.1 - i32::try_from(FLOATING_ABOVE_THE_EDGE).unwrap(),
                "it is clear of the bottom edge"
            );
        }
    }

    /// **The accent is `alo-appearance`'s**: the rule along the band's inside
    /// edge is exactly the accent the person chose, for the scheme on screen,
    /// and the band is never deep teal.
    #[test]
    fn the_dock_carries_the_persons_accent() {
        for accent in Accent::ALL {
            let mut appearance = an_appearance();
            appearance.set_accent(accent);
            let look = noon_look(&appearance, Direction::LeftToRight);
            let drawn = picture(&Dock::shipped(), look, (1920, 1080), 4).unwrap();
            let rule = drawn
                .solids
                .iter()
                .find(|solid| solid.area == drawn.accent)
                .unwrap();
            assert_eq!(
                rule.colour,
                crate::desktop_look::rgb(accent.on(look.scheme()))
            );
        }
    }

    /// **A display `alo-dock` refuses is refused**, and so is one larger than any
    /// this dock is laid out for, rather than a dock drawn over somebody's whole
    /// screen.
    #[test]
    fn a_display_too_small_or_too_large_for_a_dock_is_refused() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        for size in [(0, 0), (100, 100), (-5, 800), (20_000, 1080)] {
            assert!(
                matches!(
                    picture(&Dock::shipped(), look, size, 4),
                    Err(RenderError::DesktopScene)
                ),
                "{size:?}"
            );
        }
    }
}
