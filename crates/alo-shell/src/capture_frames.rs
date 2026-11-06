//! Keep a captured application's frame clock alive when desktop content covers it.

use smithay::{
    backend::renderer::utils::RendererSurfaceStateUserData,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::compositor::{SurfaceAttributes, TraversalAction, with_surface_tree_downward},
};

pub(crate) fn completed(
    root: &WlSurface,
    popups: &[crate::Popup],
    camera: alo_canvas::Camera,
    time: u32,
) {
    for (tree, _) in crate::scene::trees(std::slice::from_ref(root), popups, camera) {
        with_surface_tree_downward(
            &tree,
            (),
            |_, states, _| {
                if states
                    .data_map
                    .get::<RendererSurfaceStateUserData>()
                    .is_some_and(|state| state.lock().is_ok_and(|state| state.view().is_some()))
                {
                    TraversalAction::DoChildren(())
                } else {
                    TraversalAction::SkipChildren
                }
            },
            |_, states, _| {
                if !states
                    .data_map
                    .get::<RendererSurfaceStateUserData>()
                    .is_some_and(|state| state.lock().is_ok_and(|state| state.view().is_some()))
                {
                    return;
                }
                for callback in states
                    .cached_state
                    .get::<SurfaceAttributes>()
                    .current()
                    .frame_callbacks
                    .drain(..)
                {
                    callback.done(time);
                }
            },
            |_, _, _| true,
        );
    }
}
