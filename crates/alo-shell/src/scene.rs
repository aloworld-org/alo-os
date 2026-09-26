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

/// Front-to-back trees, descendants above parents and newest siblings first,
/// placed where the canvas's camera puts them.
///
/// **This is the one seam the plane moves at**, and that is why the camera is a
/// parameter here rather than something each caller applies. Three roads read
/// this list — the drawing, the pointer's hit test and popup placement — and the
/// pointer *subtracts* the origin it is given. So panning here moves what is
/// drawn and what can be clicked by the same amount, in the same arithmetic,
/// with nothing to keep in step. A camera applied in the drawing alone would put
/// every window's pixels somewhere its clicks are not.
///
/// The dock, the status area and every other native layer are **not** in this
/// list: they are painted above these trees by `crate::scene_drawing`, from the
/// output's own size. That is the two-layer separation
/// `docs/autonomy/the-smallest-canvas-worth-showing.md` task 1 is about, and the
/// reason nothing in the viewport layer needs to know a camera exists.
///
/// **The zoom is not applied yet.** `crate::drawing` has no scaling path at all,
/// so a scale here would be a number nothing honoured. Task 1's inventory in that
/// plan says so and names it as the expensive half.
pub(crate) fn trees(
    roots: &[WlSurface],
    popups: &[Popup],
    camera: alo_canvas::Camera,
) -> Vec<(WlSurface, Point<f64, Logical>)> {
    let panned = |origin: Point<f64, Logical>| {
        Point::from((
            origin.x - f64::from(camera.at().x),
            origin.y - f64::from(camera.at().y),
        ))
    };
    let mut trees = Vec::new();
    // Expand depth-first without recursive calls on client-controlled chains.
    let mut pending: Vec<_> = roots
        .iter()
        .rev()
        .map(|root| {
            (
                root.clone(),
                panned(crate::window_buffer_origin(root)),
                false,
            )
        })
        .collect();
    while let Some((surface, origin, expanded)) = pending.pop() {
        if expanded {
            trees.push((surface, origin));
            continue;
        }
        pending.push((surface.clone(), origin, true));
        for popup in popups.iter().filter(|popup| popup.parent == surface) {
            pending.push((popup.surface.clone(), origin + popup.location(), false));
        }
    }
    trees
}

/// Protocol geometry is clamped to the surface tree, as required by XDG shell.
pub(crate) fn geometry_origin(surface: &WlSurface) -> Point<f64, Logical> {
    geometry(surface).loc
}

/// Committed window geometry, including its effective surface-tree bounds.
pub(crate) fn geometry(surface: &WlSurface) -> Rectangle<f64, Logical> {
    let bounds = tree_bounds(surface);
    with_states(surface, |states| {
        states
            .cached_state
            .get::<SurfaceCachedState>()
            .current()
            .geometry
            .and_then(|geometry| geometry.to_f64().intersection(bounds))
            .unwrap_or(bounds)
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
