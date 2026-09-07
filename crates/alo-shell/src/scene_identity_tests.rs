//! Real Wayland resource identities retained across injected DRM outcomes.
use super::*;
use crate::scene_scanout::activate;
use scene_scanout_tests::pixels;
use smithay::reexports::wayland_server::{
    Client, DataInit, Dispatch, Display, DisplayHandle, Resource,
    protocol::wl_surface::{self, WlSurface},
};

/// Identity-only protocol fixture: it never dispatches or advertises these objects.
struct IdentityState;
impl Dispatch<WlSurface, ()> for IdentityState {
    fn request(
        _: &mut Self,
        _: &Client,
        surface: &WlSurface,
        _: wl_surface::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        surface.post_error(0u32, "identity fixture does not accept requests");
    }
}

#[test]
fn replacement_changes_surface_identities_only_after_commit_even_when_cleanup_fails()
-> Result<(), Box<dyn std::error::Error>> {
    let display = Display::<IdentityState>::new()?;
    let mut handle = display.handle();
    let (socket, _peer) = std::os::unix::net::UnixStream::pair()?;
    let client = handle.insert_client(socket, std::sync::Arc::new(()))?;
    let old = client.create_resource::<WlSurface, (), IdentityState>(&handle, 1, ())?;
    let new = client.create_resource::<WlSurface, (), IdentityState>(&handle, 1, ())?;
    assert_ne!(old, new);
    let (device, log) = fixture(&[], 0);
    let mut scene = activate(
        (pixels((1280, 720))?, vec![old.clone()]),
        device,
        &scanout_tests::output(),
    )?;
    log.borrow_mut().failures.push("enable");
    assert!(
        scene
            .replace((pixels((1280, 720))?, vec![new.clone()]))
            .is_err()
    );
    assert_eq!(scene.surfaces, [old]);
    log.borrow_mut().failures = vec!["destroy framebuffer"];
    let result = scene.replace((pixels((1280, 720))?, vec![new.clone()]))?;
    assert!(result.retirement_error.is_some());
    assert_eq!(scene.surfaces, [new]);
    log.borrow_mut().failures.clear();
    scene.active.retire()?;
    Ok(())
}
