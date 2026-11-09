//! How a band's three controls are drawn: their marks, their squares, the
//! focus ring, and how far they have faded.
//!
//! Separate from `crate::window_edge` because they are two reasons to change —
//! the design moving a control, and the design redrawing one.
//!
//! # From the design's own artwork
//!
//! Each mark is the SVG the design exported for its 44-unit target
//! (`crates/alo-shell/icons/the-window-controls/`, with where each came from),
//! read by `alo_vector_shapes` and drawn at the display's scale, so a mark is
//! the designer's stroke of 1.4 with round caps rather than a staircase of
//! rectangles.
//!
//! # The looks, measured
//!
//! | | at rest, revealed | hovered | panel open |
//! |---|---|---|---|
//! | Window actions | `bg/cool` square, corner 8, navy mark (`11:164`) | the same | navy square, white mark (`11:42`) |
//! | Minimise | navy mark, no square (`11:45`) | `bg/cool` square | — |
//! | Close | navy mark, no square (`11:49`) | `bg/cool` square, `status/danger` mark | — |
//!
//! **Red only on Close, and only while it is pointed at** — the specification's
//! *restrained red hover*. The keyboard's focus ring is navy, 2 wide, round the
//! whole target with the same corner, so focus is never shown by colour alone.
//! An unfocused window's marks are its look's quieter ink, as its title is.

use std::sync::OnceLock;

use alo_appearance::Role;
use alo_vector_shapes::AShape;
use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;
use crate::window_edge::{OnTheEdge, WindowEdge};
use crate::window_edge_look::WhichLook;

/// A focus ring's width, in logical units.
const A_FOCUS_RING_IS: f32 = 2.0;

/// The corner of every control's square: `11:42` and `11:164`, `rx = 8`.
const A_SQUARES_CORNER: f32 = 8.0;

/// Where a pointer and the keyboard are on a band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pointing {
    /// The control under the pointer, if any.
    pub hovered: Option<OnTheEdge>,
    /// The control the keyboard is on, if any.
    pub focused: Option<OnTheEdge>,
}

/// Everything about a band's controls that is not where they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowTheControlsLook {
    /// Pointer and keyboard.
    pub pointing: Pointing,
    /// Whether this window's panel is open, which is Window actions' open look.
    pub open: bool,
    /// How far they have faded in, in thousandths (`crate::the_controls_fade`).
    pub how_shown: u16,
    /// The window's look: the typing window's or every other's.
    pub look: WhichLook,
}

/// The mark each control is drawn with, read once.
fn the_mark(of: OnTheEdge) -> Option<&'static AShape> {
    /// The three, in [`OnTheEdge`]'s order, each read once or not at all.
    static MARKS: OnceLock<[Option<AShape>; 3]> = OnceLock::new();
    let marks = MARKS.get_or_init(|| {
        [
            AShape::read(include_str!(
                "../icons/the-window-controls/window-actions-mark.svg"
            ))
            .ok(),
            AShape::read(include_str!("../icons/the-window-controls/minimise.svg")).ok(),
            AShape::read(include_str!("../icons/the-window-controls/close.svg")).ok(),
        ]
    });
    let which = match of {
        OnTheEdge::WindowActions => 0,
        OnTheEdge::Minimise => 1,
        OnTheEdge::Close => 2,
    };
    marks.get(which).and_then(Option::as_ref)
}

/// The colours one control is drawn in: its square (if any) and its mark.
#[must_use]
pub(crate) fn its_colours(
    does: OnTheEdge,
    how: &HowTheControlsLook,
) -> (Option<[u8; 3]>, [u8; 3]) {
    let rgb = |role: Role| {
        let it = role.colour();
        [it.red(), it.green(), it.blue()]
    };
    let hovered = how.pointing.hovered == Some(does);
    let ink = how.look.colours().ink;
    match does {
        OnTheEdge::WindowActions if how.open => (Some(rgb(Role::TextPrimary)), rgb(Role::BgSurface)),
        OnTheEdge::WindowActions => (Some(rgb(Role::BgCool)), ink),
        OnTheEdge::Close if hovered => (Some(rgb(Role::BgCool)), rgb(Role::StatusDanger)),
        OnTheEdge::Minimise | OnTheEdge::Close if hovered => (Some(rgb(Role::BgCool)), ink),
        OnTheEdge::Minimise | OnTheEdge::Close => (None, ink),
    }
}

/// Every control on this band, drawn, in the display's pixels.
///
/// **Nothing at all when they have faded right out**, so a concealed band
/// costs no rasters.
#[must_use]
pub(crate) fn the_controls_drawn(
    edge: &WindowEdge,
    how: &HowTheControlsLook,
    scale: u32,
) -> Vec<Washed> {
    if how.how_shown == 0 {
        return Vec::new();
    }
    let fraction = f64::from(scale.max(1)) / 100.0;
    edge.controls
        .iter()
        .filter_map(|control| {
            let target = in_pixels(control.target, scale);
            one_control(target, control.does, how, fraction)
        })
        .collect()
}

/// One control, drawn into its target.
fn one_control(
    target: Rectangle<i32, Physical>,
    does: OnTheEdge,
    how: &HowTheControlsLook,
    fraction: f64,
) -> Option<Washed> {
    let (Ok(wide), Ok(tall)) = (u32::try_from(target.size.w), u32::try_from(target.size.h)) else {
        return None;
    };
    if wide == 0 || tall == 0 {
        return None;
    }
    let (square, ink) = its_colours(does, how);
    let mark = the_mark(does).map(|shape| shape.coverage(wide, tall));
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a corner and a ring at a display's scale are well inside an f32"
    )]
    let (corner, ring) = (
        (f64::from(A_SQUARES_CORNER) * fraction) as f32,
        (f64::from(A_FOCUS_RING_IS) * fraction).max(1.0) as f32,
    );
    let focused = how.pointing.focused == Some(does);
    let navy = {
        let it = Role::TextPrimary.colour();
        [it.red(), it.green(), it.blue()]
    };
    let shown = f32::from(how.how_shown) / 1000.0;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a control's size in pixels is far inside an f32's exact range"
    )]
    let (w, h) = (wide as f32, tall as f32);
    let mut pixels = Vec::with_capacity(usize::try_from(wide * tall * 4).unwrap_or(0));
    for (index, y) in (0..tall).flat_map(|y| (0..wide).map(move |_| y)).enumerate() {
        let x = u32::try_from(index).unwrap_or(0) % wide;
        #[expect(
            clippy::cast_precision_loss,
            reason = "a pixel's place in a control is far inside an f32's exact range"
        )]
        let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
        let inside = (0.5 - away_from_a_rounded_square(px, py, w, h, corner)).clamp(0.0, 1.0);
        let (mut colour, mut alpha) = match square {
            Some(fill) => (fill, inside),
            None => ([0, 0, 0], 0.0),
        };
        let covered = mark
            .as_ref()
            .and_then(|it| it.get(index))
            .map_or(0.0, |it| f32::from(*it) / 255.0);
        if covered > 0.0 {
            (colour, alpha) = over((colour, alpha), (ink, covered));
        }
        if focused {
            let d = away_from_a_rounded_square(px, py, w, h, corner);
            let on_the_ring = ((0.5 - d).clamp(0.0, 1.0)) * ((d + ring + 0.5).clamp(0.0, 1.0));
            if on_the_ring > 0.0 {
                (colour, alpha) = over((colour, alpha), (navy, on_the_ring));
            }
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to a byte's range before the cast"
        )]
        let byte = ((alpha * shown).clamp(0.0, 1.0) * 255.0).round() as u8;
        pixels.extend_from_slice(&[colour[0], colour[1], colour[2], byte]);
    }
    Some(
        Washed {
            area: target,
            pixels,
        }
        .premultiplied(),
    )
}

/// `top` painted over `under`, both in straight alpha.
fn over(under: ([u8; 3], f32), top: ([u8; 3], f32)) -> ([u8; 3], f32) {
    let (under_colour, under_alpha) = under;
    let (top_colour, top_alpha) = top;
    let alpha = top_alpha + under_alpha * (1.0 - top_alpha);
    if alpha <= 0.0 {
        return ([0, 0, 0], 0.0);
    }
    let mut colour = [0_u8; 3];
    for ((out, a), b) in colour.iter_mut().zip(under_colour).zip(top_colour) {
        let mixed =
            (f32::from(b) * top_alpha + f32::from(a) * under_alpha * (1.0 - top_alpha)) / alpha;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to a byte's range before the cast"
        )]
        let byte = mixed.round().clamp(0.0, 255.0) as u8;
        *out = byte;
    }
    (colour, alpha)
}

/// How far a point is outside a `w × h` square with rounded corners, negative
/// inside — the signed distance every rounded shape in this shell is drawn by.
fn away_from_a_rounded_square(px: f32, py: f32, w: f32, h: f32, corner: f32) -> f32 {
    let corner = corner.min(w / 2.0).min(h / 2.0);
    let (cx, cy) = (w / 2.0, h / 2.0);
    let qx = (px - cx).abs() - (cx - corner);
    let qy = (py - cy).abs() - (cy - corner);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    outside + qx.max(qy).min(0.0) - corner
}

/// A logical rectangle in the display's pixels, scaled once.
fn in_pixels(
    of: Rectangle<i32, smithay::utils::Logical>,
    scale: u32,
) -> Rectangle<i32, Physical> {
    let scaled = |it: i32| {
        i32::try_from(i64::from(it) * i64::from(scale.max(1)) / 100).unwrap_or(i32::MAX)
    };
    Rectangle::new(
        (scaled(of.loc.x), scaled(of.loc.y)).into(),
        (scaled(of.size.w), scaled(of.size.h)).into(),
    )
}

#[cfg(test)]
#[path = "window_edge_paint_tests.rs"]
mod tests;
