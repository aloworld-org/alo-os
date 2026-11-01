//! What the external window edge is drawn out of.
//!
//! `crate::window_edge` says **where** everything is; this says **what colour**
//! and **what shape**, and nothing else. Two files because they are two
//! reasons to change: the design moving a control is the first, the design
//! recolouring one is the second.
//!
//! # Every colour is a role, and no new one was added
//!
//! The contract asks to *reuse the file's existing variables*, and all five it
//! names already exist as roles resolving to the measured hexes:
//!
//! | the design says | the role | and it is |
//! |---|---|---|
//! | `bg/surface` | [`Role::BgSurface`] | `#FFFFFF` |
//! | `border/default` | [`Role::BorderDefault`] | `#E7EBEF` |
//! | `text/primary` | [`Role::TextPrimary`] | `#102A43` |
//! | `text/muted` | [`Role::TextMuted`] | `#596B78` |
//! | `bg/cool` | [`Role::BgCool`] | `#EEF2F4` |
//! | `status/danger` | [`Role::StatusDanger`] | `#B42318` |
//!
//! **No teal.** The contract rules it out for ordinary window controls, and
//! `the-canvas-as-a-workspace.md` §6 reserves it for alo acting.
//!
//! # What the contract forbids, and how this is shaped to obey
//!
//! > Transparent button backgrounds at rest. Subtle highlight only on the
//! > hovered control. No permanent square button tiles, bouncing or hover
//! > enlargement.
//!
//! So a control at rest contributes **nothing** — not a tile, not a border,
//! not a background. Only its artwork is drawn. The one thing that ever
//! appears behind a control is the hovered highlight, and that is the 32 × 28
//! pill `crate::window_edge` measured, never the 44 × 44 target.
//!
//! Nothing here grows, moves or animates a control. Hover changes two things
//! and both are colour: a highlight appears, and Close's artwork turns
//! `status/danger`.
//!
//! # What is not here
//!
//! **The title's text.** Drawing a string needs the font machinery
//! `crate::window_control_label` owns, and a glyph run is a different
//! responsibility from a rectangle. [`WindowEdge::title`](crate::WindowEdge::title) is
//! where it goes and a caller with fonts puts it there.
//!
//! **The strip itself.** Since 2026-11-01 its top corners are rounded at 10
//! (`554:25`), which a rectangle cannot be, so `crate::the_title_strip_drawn`
//! draws it as a raster and this draws what sits on it.

use alo_appearance::{Colour, Role};
use smithay::utils::{Logical, Rectangle};

use crate::window_edge::{EdgeControl, OnTheEdge, WindowEdge};
use crate::window_edge_look::WhichLook;

/// One rectangle to fill, and what to fill it with.
pub type Solid = (Rectangle<i32, Logical>, Colour);


/// How thick the keyboard-focus ring is.
///
/// **Two, and not the strip's one-unit line.** Measured from `414:27609` on
/// 2026-10-12: at the ring's middle row the rendered pixels read
/// `#ffffff #102a43 #102a43 #ffffff` — the ink exactly, twice, with no
/// partial values either side, so it is two units and not one spread over
/// two by antialiasing. The strip's own border is a different line and
/// stays at one; sharing a constant would have made a design change to
/// either silently move the other.
const A_FOCUS_RING_IS: i32 = 2;

/// What a person is doing to the edge, which is all that changes its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pointing {
    /// The control the pointer is over, if it is over one.
    pub hovered: Option<OnTheEdge>,
    /// The control keyboard focus is on, if any. Drawn with a navy outline.
    pub focused: Option<OnTheEdge>,
}

/// Everything to fill for this edge, in the order it is drawn.
///
/// Back to front: each control's highlight, then its artwork, then any focus
/// outline. A caller fills them in order and gets the design.
///
/// **The strip is not here any more.** Its top corners are rounded at 10 since
/// the owner approved `554:56` on 2026-11-01, and a rounded corner is partial
/// coverage a rectangle cannot be, so it is a raster:
/// `crate::the_title_strip_drawn`. What is here is drawn over it.
///
/// `look` is which window this is — the one the person's typing goes to, or
/// another — and decides the controls' ink (`crate::window_edge_look`).
#[must_use]
pub fn solids(edge: &WindowEdge, pointing: Pointing, look: WhichLook) -> Vec<Solid> {
    let mut solids = Vec::new();

    // The grip: quiet at rest and a mark when revealed, and **`text/muted`
    // either way**. The contract says *navy human controls* and the grip is
    // not one of them — measured from `398:26244`, which resolves
    // `text/muted` and not `text/primary`.
    solids.extend(grip_of(edge));

    let [red, green, blue] = look.colours().ink;
    let ink_of_the_look = Colour::of(red, green, blue);
    for control in &edge.controls {
        if pointing.hovered == Some(control.does) {
            solids.push((control.highlight, Role::BgCool.colour()));
        }
        let ink = if pointing.hovered == Some(control.does) && control.does == OnTheEdge::Close {
            // **Only Close, and only hovered.** Measured: `status/danger`
            // resolves on the Close-hover variant and on no other.
            Role::StatusDanger.colour()
        } else {
            // **Navy on the focused window, muted on every other**, by the
            // board `557:456`: *title, icon and controls in text/muted — still
            // readable, still 44 × 44.*
            ink_of_the_look
        };
        // **Move draws no artwork of its own**, because `grip_of` above already
        // drew its mark: the design's *Move window / 14px glyph* is the same
        // 14 × 14 the grip occupies. Two draws of one rectangle would be two
        // places deciding what that control looks like.
        if control.does != OnTheEdge::Move {
            solids.extend(artwork_of(control, ink));
        }
        if pointing.focused == Some(control.does) {
            // **The target, not the highlight**, which this rang until
            // 2026-10-12. `control.highlight` is 32 × 28 at `(+6, +14)` and
            // is right for hover — it is byte-for-byte the rectangle
            // `State=Menu open` draws. The design rings the whole 44 × 44
            // interaction envelope, so the real ring stands 12 units above
            // the visible strip and the drawn one sat inside it. Measured
            // from `414:27609`; `MANIFEST.md` carries the reading and why
            // `get_metadata` could not show it.
            solids.extend(outline_of(control.target, Role::TextPrimary));
        }
    }
    solids
}

/// The grip, drawn as the design draws it in each state.
///
/// **Nothing, on an edge that shows the application's icon**: the approved
/// edge (`554:24`) draws the icon where the grip's mark was, inside the Move
/// target, and two marks in one place would be two answers to *what is this*.
/// An edge with no icon to show keeps the mark — `the_grip_mark`.
fn grip_of(edge: &WindowEdge) -> Vec<Solid> {
    let quiet = Role::TextMuted.colour();
    if edge.strip.is_none() {
        // At rest it is the whole mark: a 24 × 2 bar.
        return vec![(edge.grip, quiet)];
    }
    if edge.icon.is_some() {
        return Vec::new();
    }
    the_grip_mark(edge)
}

/// The revealed grip's two bars, for an edge with no icon in its place — an
/// application with its own header, or a shell-drawn window whose application
/// has no icon this machine knows.
#[must_use]
pub(crate) fn the_grip_mark(edge: &WindowEdge) -> Vec<Solid> {
    let quiet = Role::TextMuted.colour();
    // Revealed it is a 14 × 14 mark. Two bars, which is what a grip is: the
    // affordance says *this is the part you pull*, and two lines say it in
    // the space the design gives.
    let mut bars = Vec::new();
    for down in [5, 9] {
        bars.push((
            Rectangle::new(
                (edge.grip.loc.x, edge.grip.loc.y + down).into(),
                (edge.grip.size.w, 2).into(),
            ),
            quiet,
        ));
    }
    bars
}

/// A control's 14 × 14 artwork, as rectangles.
///
/// Drawn from rectangles rather than the design's SVG assets, the way
/// `crate::window_control_paint` already draws its glyphs — this renderer
/// fills rectangles and has no path support, and inventing one for three
/// marks would be a renderer nobody asked for. **The shapes are the design's
/// and the fidelity is this renderer's**, which is a limitation worth naming
/// rather than a choice worth defending.
fn artwork_of(control: &EdgeControl, colour: Colour) -> Vec<Solid> {
    let at = control.artwork.loc;
    let size = control.artwork.size.w;
    let thick = 2;
    let mut marks = Vec::new();
    let mut bar = |x: i32, y: i32, w: i32, h: i32| {
        marks.push((
            Rectangle::new((at.x + x, at.y + y).into(), (w, h).into()),
            colour,
        ));
    };
    match control.does {
        // **Move draws nothing here.** Its mark is the grip, which `grip_of` has
        // already placed, and `solids` skips this call for Move - so the arm
        // is unreachable in practice and is here because a match must be
        // total, an `unreachable!` being a panic in a compositor.
        OnTheEdge::Move => {}
        // A single bar along the bottom.
        OnTheEdge::Minimise => bar(0, size - thick, size, thick),
        // A hollow square.
        OnTheEdge::Maximise => {
            bar(0, 0, size, thick);
            bar(0, size - thick, size, thick);
            bar(0, 0, thick, size);
            bar(size - thick, 0, thick, size);
        }
        // A cross, as a stack of short bars stepping across — this renderer
        // has no diagonal, so a diagonal is drawn one row at a time.
        OnTheEdge::Close => {
            for step in 0..size {
                bar(step, step, thick, thick);
                bar(size - thick - step, step, thick, thick);
            }
        }
        // Three bars: the menu.
        OnTheEdge::Menu => {
            for down in [0, (size - thick) / 2, size - thick] {
                bar(0, down, size, thick);
            }
        }
    }
    marks
}

/// A one-unit outline around a rectangle, for keyboard focus.
///
/// **Navy, and visible.** The contract asks for a *visible navy keyboard-focus
/// outline*, and the focus variant resolves no colour the revealed one does
/// not — so it is `text/primary` rather than a colour of its own.
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


#[cfg(test)]
#[path = "window_edge_paint_tests.rs"]
mod tests;
