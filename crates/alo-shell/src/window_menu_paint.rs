//! What the window menu draws, so that a person can see it.
//!
//! `crate::window_menu` laid the panel out and `crate::window_menu_keys` walks
//! it, and **until this file nothing painted it**. A menu with a hit test, a
//! keyboard road and a screen-reader reading, that no one can see, is not a
//! menu. It is the same fault the edge had before `window_edge_paint`: every
//! half present except the one a person meets.
//!
//! # The same shape as the edge's painter, on purpose
//!
//! [`Solid`](crate::Solid) is `(rectangle, colour)` and a caller fills them
//! back to front. One painter would have been wrong — the edge is a strip of
//! 44 × 44 targets and the menu is a panel of 248 × 48 rows — and one *shape*
//! is right, because the thing that consumes them is the same renderer.
//!
//! # The five colours are the five the project already has
//!
//! The design's own note says so: *existing semantic colours; same geometry*.
//! So nothing here asks `alo-appearance` for a role it does not have.
//!
//! | what | role | light |
//! |---|---|---|
//! | the panel | `bg/surface` in a `border/default` frame | `#FFFFFF` on `#E7EBEF` |
//! | the row under the pointer | `bg/cool` | `#EEF2F4` |
//! | a row that will act | `text/primary` | `#102A43` |
//! | a row this window will not allow | `text/muted` | `#596B78` |
//! | the keyboard's ring | `text/primary`, two units | `#102A43` |
//!
//! **An unavailable row is drawn muted rather than hidden**, which is the
//! same decision `crate::window_menu::MenuRow::available` records for the
//! reading: a row that vanished would move every row below it between one
//! window and the next.
//!
//! # What this does not draw
//!
//! **Text.** The title and each row's words are `Inked`, which needs the font
//! system, and `crate::window_edge_picture` is where that is done for the
//! edge. This draws the shapes; the picture that carries both is
//! `crate::window_menu_picture`'s job and it calls this.

use alo_appearance::Role;
use smithay::utils::{Logical, Rectangle};

use crate::window_edge_paint::Solid;
use crate::window_menu::{MenuRow, WhatARowDoes, WindowMenu};

/// How thick the panel's border is, one logical unit, as the edge's is.
const THE_BORDER_IS: i32 = 1;

/// How thick the ring on the row with the keyboard is.
///
/// Two, for `crate::window_edge_paint::A_FOCUS_RING_IS`' reason and measured
/// from the same place: a focus indicator the design draws at two units.
const A_FOCUS_RING_IS: i32 = 2;

/// What a person is doing to the menu, which is all that changes its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PointingAtTheMenu {
    /// The row the pointer is over, if it is over one.
    pub hovered: Option<WhatARowDoes>,
    /// The row the keyboard is on, if it is on one.
    pub focused: Option<WhatARowDoes>,
}

/// Everything to fill for this menu, back to front.
///
/// The panel's border, its surface, then each row: its hover fill, its ink,
/// and any focus ring. A caller fills them in order and gets the design.
///
/// **The rows come from the menu that was laid out.** Nothing here computes a
/// row's position, for `crate::window_menu_reading`'s reason in the other
/// direction: two files deciding where a row is would be two answers, and the
/// one that is drawn and the one that is pressed would come apart.
#[must_use]
pub fn solids(menu: &WindowMenu, pointing: PointingAtTheMenu) -> Vec<Solid> {
    let mut solids = vec![(menu.panel, Role::BorderDefault.colour())];
    if let Some(inside) = inset(menu.panel, THE_BORDER_IS) {
        solids.push((inside, Role::BgSurface.colour()));
    }

    for row in &menu.rows {
        solids.extend(a_row(*row, pointing));
    }
    solids
}

/// One row: what is filled behind it, and the mark that says what it does.
fn a_row(row: MenuRow, pointing: PointingAtTheMenu) -> Vec<Solid> {
    let mut solids = Vec::new();

    // **Only a row that can be used is highlighted.** Filling behind a row
    // that will not act would say *this one* to a pointer and then do
    // nothing, which is the shape of fault the edge's painter avoids by
    // highlighting the hovered control and no other.
    if pointing.hovered == Some(row.does) && row.available {
        solids.push((row.row, Role::BgCool.colour()));
    }

    // **Muted where the window will not allow it**, which is the drawn half
    // of what a reader is told as `State::ReadOnly`. The two must agree: a row
    // spoken as unavailable and drawn as ordinary would be one surface saying
    // two things.
    let ink = if row.available {
        Role::TextPrimary
    } else {
        Role::TextMuted
    };
    solids.extend(the_mark_of(row, ink));

    if pointing.focused == Some(row.does) {
        // The whole row, as the edge's ring is the whole target: the thing a
        // person is on is the thing that is ringed.
        solids.extend(outline_of(row.row, Role::TextPrimary));
    }
    solids
}

/// The mark drawn in a row, standing for the words until text is drawn.
///
/// **A bar and not a glyph.** The design's rows carry words, and words need
/// the font system this file deliberately does not have — see the header. A
/// bar at the row's own leading edge is honest about that: it marks where the
/// words go and claims to be nothing else. `crate::window_menu_picture` is
/// what replaces it with the words.
fn the_mark_of(row: MenuRow, ink: Role) -> Vec<Solid> {
    // Inset so the mark sits where the words will, not against the border.
    let inset_by = 16;
    let tall = 2;
    let wide = (row.row.size.w - inset_by * 2).max(0);
    if wide == 0 || row.row.size.h <= tall {
        return Vec::new();
    }
    vec![(
        Rectangle::new(
            (
                row.row.loc.x + inset_by,
                row.row.loc.y + (row.row.size.h - tall) / 2,
            )
                .into(),
            (wide, tall).into(),
        ),
        ink.colour(),
    )]
}

/// A ring around a rectangle, at the focus ring's own thickness.
fn outline_of(around: Rectangle<i32, Logical>, ink: Role) -> Vec<Solid> {
    let colour = ink.colour();
    let thick = A_FOCUS_RING_IS;
    let (x, y, w, h) = (around.loc.x, around.loc.y, around.size.w, around.size.h);
    vec![
        (Rectangle::new((x, y).into(), (w, thick).into()), colour),
        (
            Rectangle::new((x, y + h - thick).into(), (w, thick).into()),
            colour,
        ),
        (Rectangle::new((x, y).into(), (thick, h).into()), colour),
        (
            Rectangle::new((x + w - thick, y).into(), (thick, h).into()),
            colour,
        ),
    ]
}

/// A rectangle shrunk by `by` on every side, or [`None`] if nothing is left.
fn inset(of: Rectangle<i32, Logical>, by: i32) -> Option<Rectangle<i32, Logical>> {
    let (w, h) = (of.size.w - by * 2, of.size.h - by * 2);
    (w > 0 && h > 0).then(|| Rectangle::new((of.loc.x + by, of.loc.y + by).into(), (w, h).into()))
}

#[cfg(test)]
#[path = "window_menu_paint_tests.rs"]
mod tests;
