//! Bounded output-to-parent conversion before upstream popup constraint arithmetic.

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Physical, Point, Rectangle},
    wayland::shell::xdg::PositionerState,
};

/// Use only adjustments the client requested; impossible fits may remain clipped.
pub(crate) fn geometry(
    positioner: PositionerState,
    parent: &WlSurface,
    roots: &[WlSurface],
    popups: &[crate::Popup],
    screens: &[crate::popups::ScreenView],
    cameras: &std::collections::BTreeMap<String, alo_canvas::Camera>,
) -> Option<Rectangle<i32, Logical>> {
    if !safe_positioner(&positioner) {
        return None;
    }
    if screens.is_empty() || positioner.constraint_adjustment.is_empty() {
        return Some(positioner.get_geometry());
    }
    // **Which screen first, then that screen's camera, then the tree.**
    // `trees` needs a camera to say where the parent is *on a screen*, and
    // the camera is the one belonging to the screen the parent is on — which
    // would be circular if it were asked that way. It is not: the parent's
    // place on the **plane** is camera-independent, so the screen is chosen
    // from that and the camera follows. `more-than-one-display-plan.md`
    // task 7.
    let on_the_plane = crate::window_placement::window_buffer_origin(parent)
        + crate::scene::geometry_origin(parent);
    let here = Point::<i32, Physical>::from((
        on_the_plane.x.floor() as i32,
        on_the_plane.y.floor() as i32,
    ));
    let view = the_screen_showing(screens, here)?;
    // Looked up rather than carried, so a pan cannot leave this reading a
    // camera nobody is looking through.
    let camera = cameras
        .get(&view.named)
        .copied()
        .unwrap_or_else(alo_canvas::Camera::new);
    let screen = &view.rect;
    let (_, origin) = crate::scene::trees(roots, popups, camera)
        .into_iter()
        .find(|(surface, _)| surface == parent)?;
    // **The screen this popup's parent is on, not the union of them and not
    // the one that drew last.** `more-than-one-display-plan.md` task 6. Until
    // this, the bound was a single size set by whichever display rendered the
    // most recent frame, so a menu near the inner edge of the left display
    // flipped against the right display's far edge — which is to say it did
    // not flip at all, and opened off the screen it belongs to.
    //
    // Chosen by where the parent is in screen pixels, before the zoom is
    // divided out, because a display's rectangle is in those same units.

    // **The output, in the parent's own units.** A positioner's rectangle is
    // expressed in the parent surface's units and a zoom does not change them —
    // the application is never told about the canvas. `trees` answers in screen
    // pixels, so the screen is divided back rather than the parent multiplied up:
    // a menu constrained against a 1366-pixel output while zoomed out to 40 %
    // has nearly 3,400 of its own units of room, and constraining it against
    // 1,366 would flip menus that fit.
    let zoom = crate::scene::drawn_at(camera);
    let origin = origin.downscale(zoom) + crate::scene::geometry_origin(parent);
    let (left, top, width, height) = the_screen_in_the_parents_units(*screen, origin, zoom);
    // Deep client-controlled chains and extreme surface-tree offsets accumulate
    // in f64. Refuse before narrowing; 16 million leaves ample i32 headroom for
    // upstream flip/slide/resize sums with our one-million positioner operands.
    // The divided extent is bounded here too: at the furthest zoom out an output
    // is twenty times its own size in a parent's units.
    if [origin.x, origin.y, width, height, left, top]
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > 16_000_000.0)
    {
        return None;
    }
    let target = Rectangle::new(
        (left as i32, top as i32).into(),
        (width as i32, height as i32).into(),
    );
    Some(positioner.get_unconstrained_geometry(target))
}

/// The screen a parent at this point is on.
///
/// **A parent on no screen at all is still a parent with a menu to open**, so
/// the first screen is the fallback rather than a refusal — the arrangement's
/// own order, which puts a usable internal panel first, and which is the
/// whole arrangement when there is only one. A refusal here would mean a
/// client that dragged its window a pixel off the desk could not open a menu.
fn the_screen_showing(
    screens: &[crate::popups::ScreenView],
    here: Point<i32, Physical>,
) -> Option<&crate::popups::ScreenView> {
    screens
        .iter()
        .find(|view| view.rect.contains(here))
        .or_else(|| screens.first())
}

/// A screen's rectangle expressed in the parent surface's own units.
///
/// Returned as four numbers rather than a `Rectangle` because they are
/// checked for finiteness before anything is narrowed to `i32`, and a
/// rectangle of `i32` is exactly what must not be built from an unchecked
/// `f64`.
///
/// **The corner is the half task 6 added.** It is zero for a display at the
/// desk's origin — every single-display session, and the main screen of every
/// other — so this changes nothing for them and is the whole of the fix for
/// the rest. Before it, the bound was `(-origin.x, -origin.y)`: the screen
/// assumed to start where the desk does, which is true of one display and of
/// no second one.
fn the_screen_in_the_parents_units(
    screen: Rectangle<i32, Physical>,
    origin: smithay::utils::Point<f64, Logical>,
    zoom: f64,
) -> (f64, f64, f64, f64) {
    (
        f64::from(screen.loc.x) / zoom - origin.x,
        f64::from(screen.loc.y) / zoom - origin.y,
        f64::from(screen.size.w) / zoom,
        f64::from(screen.size.h) / zoom,
    )
}

/// Bound every operand before invoking upstream i32 placement arithmetic.
fn safe_positioner(positioner: &PositionerState) -> bool {
    [
        positioner.rect_size.w,
        positioner.rect_size.h,
        positioner.anchor_rect.loc.x,
        positioner.anchor_rect.loc.y,
        positioner.anchor_rect.size.w,
        positioner.anchor_rect.size.h,
        positioner.offset.x,
        positioner.offset.y,
    ]
    .into_iter()
    .all(|value| (-1_000_000..=1_000_000).contains(&value))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "popup_placement_tests.rs"]
mod tests;
