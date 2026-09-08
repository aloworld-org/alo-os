//! Real-client size negotiation, lifetime and refusal evidence.
use super::{Application, Fixture};
use alo_shell::WindowSizeError;
use smithay::{
    backend::input::KeyState, reexports::wayland_server::protocol::wl_surface::WlSurface,
};

/// Map one real client before addressing its native root.
fn mapped(fixture: &Fixture) -> Application {
    let mut app = Application::new(fixture);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Serialize the trusted request with server dispatch and expose its wire serial.
fn size(
    f: &Fixture,
    root: &WlSurface,
    dimensions: (i32, i32),
) -> Result<Option<u32>, WindowSizeError> {
    let root = root.clone();
    f.backend(move |s| {
        s.request_window_size(&root, dimensions)
            .map(|serial| serial.map(u32::from))
    })
}

#[test]
fn window_size_negotiates_once_preserving_input_activation_and_client_choice() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    assert!(f.focus_surface(root.clone()).is_ok());
    app.sync();
    let before = other.events.sizes.len();
    let serial = size(&f, &root, (80, 60));
    assert!(matches!(serial, Ok(Some(_))));
    app.sync();
    other.sync();
    assert_eq!(serial, Ok(app.events.serial));
    assert_eq!(app.events.sizes.last(), Some(&(80, 60)));
    assert_eq!(app.events.activation.last(), Some(&true));
    assert_eq!(other.events.sizes.len(), before);
    let count = app.events.sizes.len();
    assert_eq!(size(&f, &root, (80, 60)), Ok(None));
    // Until the client responds, its old buffer and keyboard remain usable.
    assert_eq!(f.render((320, 200), false, 1).ok(), Some(2));
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(30, KeyState::Released).ok(), Some(true));
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    if let Ok(Some(serial)) = serial {
        app.xdg.ack_configure(serial);
    }
    // A normal (non-maximized) client is allowed to choose its own 16x16 size.
    app.attach();
    app.sync();
    assert_eq!(size(&f, &root, (80, 60)), Ok(None));
    assert_eq!(f.render((320, 200), false, 2).ok(), Some(2));
    assert!(matches!(size(&f, &root, (96, 72)), Ok(Some(_))));
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(96, 72)));
}

#[test]
fn window_size_refuses_invalid_and_committed_limits_without_mutation() {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    for dimensions in [(0, 1), (1, 0), (-1, 1), (1, i32::MIN)] {
        assert_eq!(
            size(&f, &root, dimensions),
            Err(WindowSizeError::InvalidSize)
        );
    }
    app.toplevel.set_min_size(20, 30);
    app.toplevel.set_max_size(100, 90);
    app.sync();
    // Requests in the client queue are not committed window constraints.
    assert!(matches!(size(&f, &root, (10, 10)), Ok(Some(_))));
    app.surface.commit();
    app.sync();
    let count = app.events.sizes.len();
    for dimensions in [(19, 30), (20, 29), (101, 30), (20, 91)] {
        assert_eq!(
            size(&f, &root, dimensions),
            Err(WindowSizeError::ClientLimits)
        );
    }
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert!(matches!(size(&f, &root, (20, 30)), Ok(Some(_))));
    assert!(matches!(size(&f, &root, (100, 90)), Ok(Some(_))));
    app.toplevel.set_min_size(0, 0);
    app.toplevel.set_max_size(0, 0);
    app.surface.commit();
    app.sync();
    // Protocol-positive maxima only queue a suggestion, never allocate memory.
    assert!(matches!(size(&f, &root, (i32::MAX, i32::MAX)), Ok(Some(_))));
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(i32::MAX, i32::MAX)));
}

#[test]
fn window_size_acknowledged_buffer_changes_hit_geometry_only_on_commit() {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut app = mapped(&f);
    let root = f.root();
    let serial = size(&f, &root, (32, 24));
    assert!(matches!(serial, Ok(Some(_))));
    app.sync();
    assert_eq!(serial, Ok(app.events.serial));
    assert!(f.backend(|s| s.pointer_motion(24.0, 20.0, 1)).is_ok());
    app.sync();
    assert!(app.events.pointer.enters.is_empty());
    if let Ok(Some(serial)) = serial {
        app.xdg.ack_configure(serial);
    }
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(24.0, 20.0, 2)).is_ok());
    app.sync();
    assert!(app.events.pointer.enters.is_empty());
    app.attach_resized();
    app.sync();
    assert_eq!(f.render((64, 48), false, 3).ok(), Some(1));
    assert!(f.backend(|s| s.pointer_motion(24.0, 20.0, 4)).is_ok());
    app.sync();
    assert_eq!(app.events.pointer.enters.len(), 1);
    assert!(f.backend(|s| s.pointer_motion(32.0, 24.0, 5)).is_ok());
    app.sync();
    assert_eq!(app.events.pointer.leaves, 1);
    assert_eq!(size(&f, &root, (32, 24)), Ok(None));
}

#[test]
fn window_size_refuses_foreign_children_popups_unmapped_and_dead_roots() {
    let f = Fixture::new();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    let foreign = Fixture::new();
    let mut outsider = mapped(&foreign);
    assert_eq!(
        size(&f, &foreign.root(), (80, 60)),
        Err(WindowSizeError::Unmapped)
    );
    let (_child, _role) = app.child((2, 2));
    app.surface.commit();
    app.sync();
    let parent = root.clone();
    assert_eq!(
        f.backend(move |s| {
            smithay::wayland::compositor::get_children(&parent)
                .into_iter()
                .find(|child| child != &parent)
                .map(|child| s.request_window_size(&child, (80, 60)))
        }),
        Some(Err(WindowSizeError::Unmapped))
    );
    let (popup, xdg, _popup_role) = app.popup(true, 1);
    popup.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&popup);
    app.sync();
    assert_eq!(
        f.backend(|s| {
            s.popup_surfaces().first().map(|p| {
                let surface = p.surface.clone();
                s.request_window_size(&surface, (80, 60))
            })
        }),
        Some(Err(WindowSizeError::Unmapped))
    );
    app.sync();
    assert_eq!(app.events.sizes, [(0, 0)]);
    outsider.sync();
    assert_eq!(outsider.events.sizes, [(0, 0)]);
    assert!(matches!(size(&f, &root, (80, 60)), Ok(Some(_))));
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert_eq!(size(&f, &root, (80, 60)), Err(WindowSizeError::Unmapped));
    app.configure();
    assert_eq!(size(&f, &root, (80, 60)), Err(WindowSizeError::Unmapped));
    app.attach();
    app.sync();
    // Remapping resets the server's requested size, so it must be sent anew.
    assert!(matches!(size(&f, &root, (80, 60)), Ok(Some(_))));
    drop(app);
    f.wait_for((0, 0));
    assert_eq!(size(&f, &root, (80, 60)), Err(WindowSizeError::Unmapped));
}
