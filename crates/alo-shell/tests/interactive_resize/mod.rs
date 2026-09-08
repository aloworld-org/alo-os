//! Real wire resize transactions, including refusal and commit ordering.
use super::{Application, Fixture};
use smithay::{
    backend::input::{
        ButtonState::{Pressed, Released},
        KeyState,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};
use wayland_protocols::xdg::shell::client::xdg_toplevel::ResizeEdge;

/// Enable the shared native keyboard/pointer path.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f
}
/// Map a real buffered application.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}
/// Send the wire request using the client's seat.
fn request(app: &mut Application, serial: u32, edge: ResizeEdge) {
    assert!(app.events.keyboard.seat.is_some());
    if let Some(seat) = &app.events.keyboard.seat {
        app.toplevel.resize(seat, serial, edge);
    }
    app.sync();
}
/// Obtain a real delivered press serial.
fn press(f: &Fixture, app: &mut Application) -> u32 {
    assert!(f.backend(|s| s.pointer_motion(4.0, 5.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(true)
    );
    app.sync();
    app.events.pointer.button_serial
}
/// Observe scene origin without mutating the scene.
fn origin(f: &Fixture, root: &WlSurface) -> (f64, f64) {
    let root = root.clone();
    f.backend(move |_| {
        let p = alo_shell::window_buffer_origin(&root);
        (p.x, p.y)
    })
}
/// Explicit acknowledgement; roundtrip alone must never apply placement.
fn ack(app: &mut Application) {
    assert!(app.events.serial.is_some());
    if let Some(serial) = app.events.serial {
        app.xdg.ack_configure(serial);
    }
    app.sync();
}

#[test]
fn interactive_resize_anchors_only_committed_responses_and_retires_final_commit() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let serial = press(&f, &mut app);
    request(&mut app, serial, ResizeEdge::TopLeft);
    assert_eq!(app.events.resizing.last(), Some(&true));
    assert!(f.backend(|s| s.pointer_motion(-20.0, -11.0, 3)).is_ok());
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(40, 32)));
    assert_eq!(origin(&f, &root), (0.0, 0.0));
    ack(&mut app);
    assert_eq!(origin(&f, &root), (0.0, 0.0));
    // Client chooses a size different from the suggestion.
    app.attach_resized();
    app.sync();
    assert_eq!(origin(&f, &root), (-16.0, -8.0));
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
        Some(false)
    );
    app.sync();
    assert_eq!(app.events.resizing.last(), Some(&false));
    let count = app.events.sizes.len();
    assert!(f.backend(|s| s.pointer_motion(50.0, 50.0, 5)).is_ok());
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    // An older acknowledged response after release still anchors, but does
    // not retire the final transaction.
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (0.0, 0.0));
    ack(&mut app);
    app.attach_resized();
    app.sync();
    assert_eq!(origin(&f, &root), (-16.0, -8.0));
    // Subsequent spontaneous resizing must not reuse the completed anchor.
    app.attach();
    app.sync();
    assert_eq!(origin(&f, &root), (-16.0, -8.0));
}

#[test]
fn interactive_resize_refuses_serial_target_edge_release_and_missing_pointer() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    let serial = press(&f, &mut app);
    let count = app.events.sizes.len();
    let other_count = other.events.sizes.len();
    request(&mut app, serial.wrapping_add(1000), ResizeEdge::Right);
    request(&mut other, serial, ResizeEdge::Right);
    request(&mut app, serial, ResizeEdge::None);
    if let Some(seat) = &app.events.keyboard.seat {
        use wayland_client::{Proxy, WEnum};
        assert!(
            app.toplevel
                .send_request(
                    wayland_protocols::xdg::shell::client::xdg_toplevel::Request::Resize {
                        seat: seat.clone(),
                        serial,
                        edges: WEnum::Unknown(3),
                    }
                )
                .is_ok()
        );
    }
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(other.events.sizes.len(), other_count);
    assert_eq!(app.events.pointer.leaves, 0);
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 3)).ok(),
        Some(true)
    );
    app.sync();
    let release = app.events.pointer.button_serial;
    request(&mut app, serial, ResizeEdge::Right);
    request(&mut app, release, ResizeEdge::Right);
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(origin(&f, &root), (0.0, 0.0));
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    request(&mut app, serial, ResizeEdge::Right);
    assert_eq!(app.events.sizes.len(), count);
    let no_pointer = Fixture::keyboard();
    let mut app = mapped(&no_pointer);
    let count = app.events.sizes.len();
    request(&mut app, 0, ResizeEdge::Right);
    assert_eq!(app.events.sizes.len(), count);
}

#[test]
fn interactive_resize_preserves_keyboard_and_clamps_live_limits_without_drift() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    assert!(f.focus_surface(root.clone()).is_ok());
    app.sync();
    let serial = press(&f, &mut app);
    request(&mut app, serial, ResizeEdge::BottomRight);
    let motions = app.events.pointer.motion.len();
    app.toplevel.set_min_size(20, 18);
    app.toplevel.set_max_size(24, 22);
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(104.0, 105.0, 3)).is_ok());
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(116, 116)));
    app.surface.commit();
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(104.0, 105.0, 4)).is_ok());
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(24, 22)));
    let count = app.events.sizes.len();
    assert!(f.backend(|s| s.pointer_motion(104.0, 105.0, 5)).is_ok());
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    for x in [f64::NAN, f64::INFINITY, 1_000_005.0] {
        assert!(f.backend(move |s| s.pointer_motion(x, 5.0, 6)).is_err());
    }
    assert!(f.backend(|s| s.pointer_motion(-104.0, -105.0, 7)).is_ok());
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(20, 18)));
    assert_eq!(app.events.pointer.motion.len(), motions);
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(30, KeyState::Released).ok(), Some(true));
    app.sync();
    other.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(other.events.keyboard.keys.is_empty());
    assert!(other.events.pointer.buttons.is_empty());
    assert_eq!(app.events.activation.last(), Some(&true));
    assert_eq!(origin(&f, &root), (0.0, 0.0));
}

#[test]
fn interactive_resize_cancels_leave_unmap_disconnect_and_impossible_live_limits() {
    for cancel in 0..4 {
        let f = fixture();
        let mut app = mapped(&f);
        let root = f.root();
        let serial = press(&f, &mut app);
        request(&mut app, serial, ResizeEdge::Left);
        match cancel {
            0 => {
                assert!(f.backend(|s| s.pointer_leave()).is_ok());
                app.sync();
                assert_eq!(app.events.resizing.last(), Some(&false));
            }
            1 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
                app.sync();
                app.configure();
                app.attach();
                app.sync();
                assert_eq!(app.events.resizing.last(), Some(&false));
            }
            2 => {
                drop(app);
                f.wait_for((0, 0));
                assert!(f.backend(|s| s.pointer_motion(10.0, 10.0, 3)).is_ok());
                assert_eq!(
                    f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
                    Some(false)
                );
                continue;
            }
            _ => {
                app.toplevel.set_min_size(0, 20);
                app.surface.commit();
                app.sync();
                assert!(f.backend(|s| s.pointer_motion(10.0, 10.0, 3)).is_ok());
                app.sync();
                assert_eq!(app.events.resizing.last(), Some(&false));
            }
        }
        if cancel != 1 {
            ack(&mut app);
        }
        app.attach_resized();
        app.sync();
        assert_eq!(origin(&f, &root), (0.0, 0.0));
        let count = app.events.sizes.len();
        assert!(f.backend(|s| s.pointer_motion(14.0, 15.0, 5)).is_ok());
        assert_eq!(
            f.backend(|s| s.pointer_button(0x110, Released, 6)).ok(),
            Some(false)
        );
        request(&mut app, serial, ResizeEdge::Left);
        assert_eq!(app.events.sizes.len(), count);
    }
}

#[test]
fn interactive_resize_accepts_subsurface_press_and_consumes_all_buttons() {
    let f = fixture();
    let mut app = mapped(&f);
    let (_child, _role) = app.child((24, 32));
    app.surface.commit();
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(25.0, 33.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(true)
    );
    app.sync();
    let serial = app.events.pointer.button_serial;
    request(&mut app, serial, ResizeEdge::BottomRight);
    assert_eq!(app.events.resizing.last(), Some(&true));
    let count = app.events.sizes.len();
    request(&mut app, serial, ResizeEdge::Left);
    if let Some(seat) = &app.events.keyboard.seat {
        app.toplevel._move(seat, serial);
    }
    app.sync();
    assert_eq!(app.events.sizes.len(), count);
    let mut other = mapped(&f);
    other.surface.attach(None, 0, 0);
    other.surface.commit();
    other.sync();
    assert_eq!(
        f.backend(|s| s.pointer_button(0x111, Pressed, 3)).ok(),
        Some(false)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
        Some(false)
    );
    assert!(f.backend(|s| s.pointer_motion(35.0, 43.0, 5)).is_ok());
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(50, 58)));
    assert_eq!(app.events.resizing.last(), Some(&true));
    assert_eq!(
        f.backend(|s| s.pointer_button(0x111, Released, 6)).ok(),
        Some(false)
    );
    app.sync();
    assert_eq!(app.events.resizing.last(), Some(&false));
}

#[test]
fn interactive_resize_all_edges_follow_actual_geometry_and_unacked_commit_is_not_a_response() {
    use ResizeEdge::*;
    for (edge, expected, origin_expected) in [
        (Top, (16, 8), (0.0, -8.0)),
        (Bottom, (16, 24), (0.0, 0.0)),
        (Left, (8, 16), (-16.0, 0.0)),
        (Right, (24, 16), (0.0, 0.0)),
        (TopLeft, (8, 8), (-16.0, -8.0)),
        (TopRight, (24, 8), (0.0, -8.0)),
        (BottomLeft, (8, 24), (-16.0, 0.0)),
        (BottomRight, (24, 24), (0.0, 0.0)),
    ] {
        let f = fixture();
        let mut app = mapped(&f);
        let root = f.root();
        let serial = press(&f, &mut app);
        request(&mut app, serial, edge);
        assert!(f.backend(|s| s.pointer_motion(12.0, 13.0, 3)).is_ok());
        app.sync();
        assert_eq!(app.events.sizes.last(), Some(&expected));
        app.attach_resized();
        app.sync();
        assert_eq!(origin(&f, &root), (0.0, 0.0));
        ack(&mut app);
        assert_eq!(origin(&f, &root), (0.0, 0.0));
        app.surface.commit();
        app.sync();
        assert_eq!(origin(&f, &root), origin_expected);
    }
}

#[test]
fn interactive_resize_release_refreshes_limits_and_refuses_impossible_initial_geometry() {
    let f = fixture();
    let mut app = mapped(&f);
    app.toplevel.set_min_size(0, 20);
    app.surface.commit();
    app.sync();
    let serial = press(&f, &mut app);
    let count = app.events.sizes.len();
    request(&mut app, serial, ResizeEdge::Left);
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(app.events.pointer.leaves, 0);
    // The refused operation did not consume authority; a valid axis can use it.
    request(&mut app, serial, ResizeEdge::BottomRight);
    assert_eq!(app.events.resizing.last(), Some(&true));
    app.toplevel.set_min_size(24, 28);
    app.surface.commit();
    app.sync();
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
        Some(false)
    );
    app.sync();
    assert_eq!(app.events.sizes.last(), Some(&(24, 28)));
    assert_eq!(app.events.resizing.last(), Some(&false));
}
