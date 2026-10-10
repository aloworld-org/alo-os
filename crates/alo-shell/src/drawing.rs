//! Fallible surface-tree import; upstream's convenience helper logs import errors.

use crate::RenderError;
use smithay::{
    backend::renderer::{
        ImportAll, Renderer,
        element::{Element, Kind, surface::WaylandSurfaceRenderElement},
        utils::RendererSurfaceStateUserData,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Point, Rectangle},
    wayland::compositor::{TraversalAction, with_surface_tree_downward},
};

/// Imported elements and their surfaces, in front-to-back tree order.
///
/// **Carries its renderer in its type**, because an imported texture belongs to
/// the renderer that made it and handing one to another is the bug this
/// parameter makes unspellable. `R` is bounded at each use rather than here, so
/// the struct can be named in a signature without repeating the bound.
pub(crate) struct Drawing<R: Renderer> {
    /// Textures retained until after submission.
    pub(crate) elements: Vec<WaylandSurfaceRenderElement<R>>,
    /// Only surfaces intersecting the framebuffer receive callbacks.
    pub(crate) surfaces: Vec<WlSurface>,
}

/// Import a tree at an explicit origin (popup geometry or cursor hotspot).
///
/// `origin` is in screen pixels and `drawn_at` is the scale the surface's own
/// units are drawn at — `crate::scene::drawn_at` for a frame on the plane, and
/// `1.0` for anything in the viewport layer, which a zoom does not resize.
pub(crate) fn import_at<R>(
    renderer: &mut R,
    roots: &[WlSurface],
    bounds: Rectangle<i32, Physical>,
    origin: Point<f64, Physical>,
    drawn_at: f64,
) -> Result<Drawing<R>, RenderError>
where
    R: Renderer + ImportAll,
    R::TextureId: Clone + 'static,
{
    let mut drawing = Drawing {
        elements: Vec::new(),
        surfaces: Vec::new(),
    };
    let mut failure = None;
    for root in roots {
        with_surface_tree_downward(
            root,
            origin,
            |_, states, location| {
                let view = states
                    .data_map
                    .get::<RendererSurfaceStateUserData>()
                    .and_then(|state| state.lock().ok().and_then(|state| state.view()));
                match view {
                    Some(view) => TraversalAction::DoChildren(
                        *location + view.offset.to_f64().to_physical(1.0).upscale(drawn_at),
                    ),
                    None => TraversalAction::SkipChildren,
                }
            },
            |surface, states, location| {
                if failure.is_some() {
                    return;
                }
                let view = states
                    .data_map
                    .get::<RendererSurfaceStateUserData>()
                    .and_then(|state| state.lock().ok().and_then(|state| state.view()));
                let Some(view) = view else {
                    return;
                };
                let location = *location + view.offset.to_f64().to_physical(1.0).upscale(drawn_at);
                // Clip before Smithay converts geometry to i32. Arbitrary cursor
                // hotspots and child offsets must not overflow rectangle endpoints.
                //
                // `location` is already on the screen — `crate::scene::trees` put
                // it there — and only the size is still in the surface's own
                // units, so only the size is scaled. A frame zoomed out from is
                // smaller on the screen than it is in itself, and clipping it at
                // its own size would keep windows that are not visible.
                if !Rectangle::new(
                    location,
                    view.dst.to_f64().to_physical(1.0).upscale(drawn_at),
                )
                .overlaps(bounds.to_f64())
                {
                    return;
                }
                match WaylandSurfaceRenderElement::from_surface(
                    renderer,
                    surface,
                    states,
                    location,
                    1.0,
                    Kind::Unspecified,
                ) {
                    Ok(Some(element)) if element.geometry(drawn_at.into()).overlaps(bounds) => {
                        drawing.elements.push(element);
                        drawing.surfaces.push(surface.clone());
                    }
                    Ok(_) => {}
                    Err(error) => {
                        failure = Some(RenderError::Submission(format!(
                            "import client buffer: {error}"
                        )))
                    }
                }
            },
            |_, _, _| true,
        );
    }
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(drawing)
}
