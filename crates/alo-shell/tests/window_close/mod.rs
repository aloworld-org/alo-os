//! Cooperative close delivery and target refusal over real Wayland sockets.
use super::{Application, Fixture};
use alo_shell::WindowCloseError;
use smithay::{
    backend::input::KeyState, reexports::wayland_server::protocol::wl_surface::WlSurface,
};

/// Map one actual client before choosing a trusted shell target.
fn mapped(fixture: &Fixture) -> Application {
    let mut app = Application::new(fixture);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Serialize a close with dispatch without adding a client-controlled endpoint.
fn close(fixture: &Fixture, surface: &WlSurface) -> Result<(), WindowCloseError> {
    let surface = surface.clone();
    fixture.backend(move |server| server.request_window_close(&surface))
}

#[test]
fn window_close_is_one_request_to_one_client_and_can_be_ignored() {
    let f = Fixture::keyboard();
    let mut selected = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    assert!(f.focus_surface(root.clone()).is_ok());
    selected.sync();
    assert!(close(&f, &root).is_ok());
    selected.sync();
    other.sync();
    assert_eq!(selected.events.close_requests, 1);
    assert_eq!(other.events.close_requests, 0);
    f.wait_for((2, 2));
    assert_eq!(f.render((320, 200), false, 1).ok(), Some(2));
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(30, KeyState::Released).ok(), Some(true));
    selected.sync();
    other.sync();
    assert_eq!(selected.events.keyboard.keys.len(), 2);
    assert_eq!(selected.events.keyboard.leaves, 0);
    assert!(other.events.keyboard.keys.is_empty());
    assert_eq!(selected.events.close_requests, 1);
    // Only another explicit action sends another request.
    assert!(close(&f, &root).is_ok());
    selected.sync();
    assert_eq!(selected.events.close_requests, 2);
    selected.toplevel.destroy();
    selected.xdg.destroy();
    selected.surface.destroy();
    selected.sync();
    f.wait_for((1, 1));
    assert_eq!(close(&f, &root), Err(WindowCloseError::Unmapped));
    other.sync();
    assert_eq!(other.events.close_requests, 0);
}

#[test]
fn window_close_refuses_unmapped_and_disconnected_roots_but_allows_fresh_remap() {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(close(&f, &root), Err(WindowCloseError::Unmapped));
    app.configure();
    assert_eq!(close(&f, &root), Err(WindowCloseError::Unmapped));
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    app.attach();
    app.sync();
    assert!(close(&f, &root).is_ok());
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    drop(app);
    f.wait_for((0, 0));
    assert_eq!(close(&f, &root), Err(WindowCloseError::Unmapped));
    let mut replacement = mapped(&f);
    assert_eq!(close(&f, &root), Err(WindowCloseError::Unmapped));
    replacement.sync();
    assert_eq!(replacement.events.close_requests, 0);
}

#[test]
fn window_close_refuses_foreign_displays_and_popup_roots() {
    let f = Fixture::new();
    f.backend(|server| server.enable_popup_protocol());
    let mut app = mapped(&f);
    let foreign = Fixture::new();
    let mut foreign_app = mapped(&foreign);
    assert_eq!(close(&f, &foreign.root()), Err(WindowCloseError::Unmapped));
    let (surface, xdg, _role) = app.popup(true, 1);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    let result = f.backend(|server| {
        server.popup_surfaces().first().map(|popup| {
            let surface = popup.surface.clone();
            server.request_window_close(&surface)
        })
    });
    assert_eq!(result, Some(Err(WindowCloseError::Unmapped)));
    app.sync();
    foreign_app.sync();
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(app.events.popups.done, 0);
    assert_eq!(foreign_app.events.close_requests, 0);
    assert_eq!(f.backend(|s| s.popup_surfaces().len()), 1);
}
