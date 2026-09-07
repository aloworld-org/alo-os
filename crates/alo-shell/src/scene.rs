//! Shared desktop stacking and XDG buffer origins for drawing and hit testing.

use crate::Popup;
use smithay::{
    backend::renderer::utils::RendererSurfaceStateUserData,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, Rectangle},
    wayland::{
        compositor::{TraversalAction, with_states, with_surface_tree_downward},
        shell::xdg::SurfaceCachedState,
    },
};

impl Popup {
    /// Buffer origin at scale one, with the parent buffer at (0,0).
    ///
    /// XDG placement aligns window geometries, excluding client-side shadows.
    /// Read committed geometry on every scene snapshot so geometry changes apply
    /// atomically with the buffer. Floating point prevents client i32 additions
    /// from overflowing before the renderer clips to the output.
    pub fn location(&self) -> Point<f64, Logical> {
        geometry_origin(&self.parent) + self.geometry.loc.to_f64() - geometry_origin(&self.surface)
    }
}

/// Front-to-back roots: each parent's newest popup precedes that parent's tree.
pub(crate) fn trees(
    roots: &[WlSurface],
    popups: &[Popup],
) -> Vec<(WlSurface, Point<f64, Logical>)> {
    let mut trees = Vec::new();
    for root in roots {
        for popup in popups.iter().rev().filter(|popup| popup.parent == *root) {
            trees.push((popup.surface.clone(), popup.location()));
        }
        trees.push((root.clone(), (0.0, 0.0).into()));
    }
    trees
}

/// Protocol geometry is clamped to the surface tree, as required by XDG shell.
fn geometry_origin(surface: &WlSurface) -> Point<f64, Logical> {
    let bounds = tree_bounds(surface);
    with_states(surface, |states| {
        states
            .cached_state
            .get::<SurfaceCachedState>()
            .current()
            .geometry
            .and_then(|geometry| geometry.to_f64().intersection(bounds))
            .unwrap_or(bounds)
            .loc
    })
}

/// Accumulate before integer conversion: a client child can sit at i32::MAX.
fn tree_bounds(surface: &WlSurface) -> Rectangle<f64, Logical> {
    let mut bounds = Rectangle::default();
    with_surface_tree_downward(
        surface,
        Point::<f64, Logical>::default(),
        |_, states, origin| {
            let view = states
                .data_map
                .get::<RendererSurfaceStateUserData>()
                .and_then(|data| data.lock().ok().and_then(|data| data.view()));
            if let Some(view) = view {
                let location = *origin + view.offset.to_f64();
                bounds = bounds.merge(Rectangle::new(location, view.dst.to_f64()));
                TraversalAction::DoChildren(location)
            } else {
                TraversalAction::SkipChildren
            }
        },
        |_, _, _| {},
        |_, _, _| true,
    );
    bounds
}
