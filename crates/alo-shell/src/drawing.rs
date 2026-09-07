//! Fallible surface-tree import; upstream's convenience helper logs import errors.

use crate::RenderError;
use smithay::{
    backend::renderer::{
        element::{Element, Kind, surface::WaylandSurfaceRenderElement},
        gles::GlesRenderer,
        utils::RendererSurfaceStateUserData,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Point, Rectangle},
    wayland::compositor::{TraversalAction, with_surface_tree_downward},
};

/// Imported elements and their surfaces, in front-to-back tree order.
pub(crate) struct Drawing {
    /// Textures retained until after submission.
    pub(crate) elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>>,
    /// Only surfaces intersecting the framebuffer receive callbacks.
    pub(crate) surfaces: Vec<WlSurface>,
}

/// Import a tree at an explicit origin (popup geometry or cursor hotspot).
pub(crate) fn import_at(
    renderer: &mut GlesRenderer,
    roots: &[WlSurface],
    bounds: Rectangle<i32, Physical>,
    origin: Point<f64, Physical>,
) -> Result<Drawing, RenderError> {
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
                        *location + view.offset.to_f64().to_physical(1.0),
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
                let location = *location + view.offset.to_f64().to_physical(1.0);
                // Clip before Smithay converts geometry to i32. Arbitrary cursor
                // hotspots and child offsets must not overflow rectangle endpoints.
                if !Rectangle::new(location, view.dst.to_f64().to_physical(1.0))
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
                    Ok(Some(element)) if element.geometry(1.0.into()).overlaps(bounds) => {
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
