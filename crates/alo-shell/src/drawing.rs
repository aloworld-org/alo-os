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

/// Preserve subsurface offsets and stacking, refusing any failed import.
pub(crate) fn import(
    renderer: &mut GlesRenderer,
    roots: &[WlSurface],
    bounds: Rectangle<i32, Physical>,
) -> Result<Drawing, RenderError> {
    let mut drawing = Drawing {
        elements: Vec::new(),
        surfaces: Vec::new(),
    };
    let mut failure = None;
    for root in roots {
        with_surface_tree_downward(
            root,
            Point::<i32, Physical>::from((0, 0)),
            |_, states, location| {
                let view = states
                    .data_map
                    .get::<RendererSurfaceStateUserData>()
                    .and_then(|state| state.lock().ok().and_then(|state| state.view()));
                match view {
                    Some(view) => TraversalAction::DoChildren(
                        *location + view.offset.to_physical_precise_round(1.0),
                    ),
                    None => TraversalAction::SkipChildren,
                }
            },
            |surface, states, location| {
                if failure.is_some() {
                    return;
                }
                let offset = states
                    .data_map
                    .get::<RendererSurfaceStateUserData>()
                    .and_then(|state| state.lock().ok().and_then(|state| state.view()))
                    .map(|view| view.offset.to_physical_precise_round(1.0))
                    .unwrap_or_default();
                match WaylandSurfaceRenderElement::from_surface(
                    renderer,
                    surface,
                    states,
                    (*location + offset).to_f64(),
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
