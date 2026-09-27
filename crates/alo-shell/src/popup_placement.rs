//! Bounded output-to-parent conversion before upstream popup constraint arithmetic.

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Physical, Rectangle, Size},
    wayland::shell::xdg::PositionerState,
};

/// Use only adjustments the client requested; impossible fits may remain clipped.
pub(crate) fn geometry(
    positioner: PositionerState,
    parent: &WlSurface,
    roots: &[WlSurface],
    popups: &[crate::Popup],
    output: Option<Size<i32, Physical>>,
    camera: alo_canvas::Camera,
) -> Option<Rectangle<i32, Logical>> {
    if !safe_positioner(&positioner) {
        return None;
    }
    let Some(size) = output.filter(|_| !positioner.constraint_adjustment.is_empty()) else {
        return Some(positioner.get_geometry());
    };
    let (_, origin) = crate::scene::trees(roots, popups, camera)
        .into_iter()
        .find(|(surface, _)| surface == parent)?;
    // **The output, in the parent's own units.** A positioner's rectangle is
    // expressed in the parent surface's units and a zoom does not change them —
    // the application is never told about the canvas. `trees` answers in screen
    // pixels, so the screen is divided back rather than the parent multiplied up:
    // a menu constrained against a 1366-pixel output while zoomed out to 40 %
    // has nearly 3,400 of its own units of room, and constraining it against
    // 1,366 would flip menus that fit.
    let zoom = crate::scene::drawn_at(camera);
    let origin = origin.downscale(zoom) + crate::scene::geometry_origin(parent);
    let (width, height) = (f64::from(size.w) / zoom, f64::from(size.h) / zoom);
    // Deep client-controlled chains and extreme surface-tree offsets accumulate
    // in f64. Refuse before narrowing; 16 million leaves ample i32 headroom for
    // upstream flip/slide/resize sums with our one-million positioner operands.
    // The divided extent is bounded here too: at the furthest zoom out an output
    // is twenty times its own size in a parent's units.
    if [origin.x, origin.y, width, height]
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > 16_000_000.0)
    {
        return None;
    }
    let target = Rectangle::new(
        (-(origin.x as i32), -(origin.y as i32)).into(),
        (width as i32, height as i32).into(),
    );
    Some(positioner.get_unconstrained_geometry(target))
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
