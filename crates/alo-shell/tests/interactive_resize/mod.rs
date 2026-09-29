//! Real resize transactions, including refusal and commit ordering.
//!
//! **These drove the transaction through `xdg_toplevel.resize` until ADR 0071
//! refused that road**, and what they assert was never about the road: a configure
//! sent while the drag is happening, the client's limits refreshed on every
//! motion, the opposite edge anchored, the final commit retired. All of that is
//! `crate::resize_transaction` and none of it changed.
//!
//! So they enter through the shell's own edge band now — `Server::begin_a_resize`,
//! which `crate::canvas_resize` reaches from a press on that band — and go on
//! asserting the same things about the same code.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

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
/// Begin the resize the way a person does: from the shell's own edge band.
///
/// Was the wire request, which ADR 0071 refuses. The shape of the call is kept so
/// each test still reads as *start a resize on this edge*; the serial is unused,
/// because a shell-owned band has no client grab to name.
fn request(f: &Fixture, app: &mut Application, _serial: u32, edge: ResizeEdge) {
    let edge = match edge {
        ResizeEdge::Top => alo_shell::FrameEdge::Top,
        ResizeEdge::Bottom => alo_shell::FrameEdge::Bottom,
        ResizeEdge::Left => alo_shell::FrameEdge::Left,
        ResizeEdge::Right => alo_shell::FrameEdge::Right,
        ResizeEdge::TopLeft => alo_shell::FrameEdge::TopLeft,
        ResizeEdge::TopRight => alo_shell::FrameEdge::TopRight,
        ResizeEdge::BottomLeft => alo_shell::FrameEdge::BottomLeft,
        _ => alo_shell::FrameEdge::BottomRight,
    };
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    let _ = f.backend(move |s| s.begin_a_resize(&frame, edge));
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
    request(&f, &mut app, serial, ResizeEdge::TopLeft);
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
fn a_client_asking_to_be_resized_is_refused_and_the_band_still_resizes() {
    let f = fixture();
    let mut app = mapped(&f);
    let mut other = mapped(&f);
    let serial = press(&f, &mut app);
    let count = app.events.sizes.len();
    let other_count = other.events.sizes.len();

    // Every way a client can ask, including the ones that used to be told apart.
    if let Some(seat) = &app.events.keyboard.seat {
        for edge in [ResizeEdge::Right, ResizeEdge::None, ResizeEdge::TopLeft] {
            app.toplevel.resize(seat, serial, edge);
        }
    }
    app.sync();
    other.sync();
    assert_eq!(
        app.events.sizes.len(),
        count,
        "a client that asked to be resized was resized"
    );
    assert_eq!(
        other.events.sizes.len(),
        other_count,
        "one client asking resized another"
    );
    assert_eq!(
        app.events.pointer.leaves, 0,
        "a refused request still took the pointer away from the client"
    );

    // And the capability the refusal replaces is really there.
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    assert!(
        f.backend(move |s| s.begin_a_resize(&frame, alo_shell::FrameEdge::Right)),
        "the shell's own band could not resize the window the client was refused"
    );
    app.sync();
    assert!(
        app.events.sizes.len() > count,
        "a resize from the band told the application nothing"
    );
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
    request(&f, &mut app, serial, ResizeEdge::BottomRight);
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
        request(&f, &mut app, serial, ResizeEdge::Left);
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
        request(&f, &mut app, serial, ResizeEdge::Left);
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
    request(&f, &mut app, serial, ResizeEdge::BottomRight);
    assert_eq!(app.events.resizing.last(), Some(&true));
    let count = app.events.sizes.len();
    request(&f, &mut app, serial, ResizeEdge::Left);
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
        request(&f, &mut app, serial, edge);
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
    request(&f, &mut app, serial, ResizeEdge::Left);
    assert_eq!(app.events.sizes.len(), count);
    assert_eq!(app.events.pointer.leaves, 0);
    // The refused operation did not consume authority; a valid axis can use it.
    request(&f, &mut app, serial, ResizeEdge::BottomRight);
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
