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
) -> Option<Rectangle<i32, Logical>> {
    if !safe_positioner(&positioner) {
        return None;
    }
    let Some(size) = output.filter(|_| !positioner.constraint_adjustment.is_empty()) else {
        return Some(positioner.get_geometry());
    };
    let (_, origin) = crate::scene::trees(roots, popups)
        .into_iter()
        .find(|(surface, _)| surface == parent)?;
    let origin = origin + crate::scene::geometry_origin(parent);
    // Deep client-controlled chains and extreme surface-tree offsets accumulate
    // in f64. Refuse before narrowing; 16 million leaves ample i32 headroom for
    // upstream flip/slide/resize sums with our one-million positioner operands.
    if [origin.x, origin.y, f64::from(size.w), f64::from(size.h)]
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > 16_000_000.0)
    {
        return None;
    }
    let target = Rectangle::new(
        (-(origin.x as i32), -(origin.y as i32)).into(),
        (size.w, size.h).into(),
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
