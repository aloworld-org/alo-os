//! Real XDG requests and backend input prove drag authority and cancellation.
use super::{Application, Fixture};
use smithay::{
    backend::input::{
        ButtonState::{Pressed, Released},
        KeyState,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};
use wayland_client::protocol::wl_keyboard;

/// **A press on a subsurface cannot move the frame, and a drag survives an
/// unrelated unmap.**
///
/// This test used to prove the opposite of its first half: a press on a child
/// surface *authorised* a move of its root, by walking up the tree to find it.
/// ADR 0071 removed that road — the name above a frame is what moves it — and a
/// child press is the sharpest case of the rule it replaced it with, because a
/// subsurface is as deep inside an application as a press can land. Renamed
/// rather than deleted: the same sequence now holds the refusal.
///
/// Its second half is unchanged in substance and re-rooted on the gesture that
/// exists: another client unmapping must not disturb a drag in progress.
#[test]
fn a_press_on_a_subsurface_cannot_move_the_frame_and_a_drag_survives_an_unrelated_unmap() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let (_child, _role) = app.child((24, 32));
    app.surface.commit();
    app.sync();
    let mut other = mapped(&f);

    // Deep inside the application, on a surface of its own.
    assert!(f.backend(|s| s.pointer_motion(25.0, 33.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(true),
        "a press on a subsurface was not delivered to the application"
    );
    app.sync();
    let serial = app.events.pointer.button_serial;
    request(&mut app, serial);
    assert!(f.backend(|s| s.pointer_motion(35.0, 43.0, 3)).is_ok());
    assert_eq!(
        origin(&f, &root),
        (0.0, 0.0),
        "a press on a subsurface moved the frame, and everything inside a frame \
         belongs to the application"
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
        Some(true)
    );

    // And the drag that does work is not disturbed by another client leaving.
    take_hold(&f, &mut app);
    other.surface.attach(None, 0, 0);
    other.surface.commit();
    other.sync();
    assert!(f.backend(|s| s.pointer_motion(35.0, 43.0, 5)).is_ok());
    assert_eq!(origin(&f, &root), (31.0, 59.0));
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 6)).ok(),
        Some(false)
    );
}

#[test]
fn window_move_without_a_pointer_is_ignored_without_disconnect() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    request(&mut app, 0);
    assert_eq!(origin(&f, &root), (0.0, 0.0));
    assert_eq!(f.render((80, 80), false, 1).ok(), Some(1));
}

fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f
}

fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

fn origin(f: &Fixture, root: &WlSurface) -> (f64, f64) {
    let root = root.clone();
    f.backend(move |_| {
        let point = alo_shell::window_buffer_origin(&root);
        (point.x, point.y)
    })
}

/// Ask to be moved, the way a client with its own title bar does.
///
/// **Refused since ADR 0071**: the shell owns the name above a frame, so a client
/// holding a serial for a press inside its own content is asking on behalf of
/// something that belongs to it. Kept here because the refusal is worth
/// asserting.
fn request(app: &mut Application, serial: u32) {
    assert!(app.events.keyboard.seat.is_some());
    if let Some(seat) = &app.events.keyboard.seat {
        app.toplevel._move(seat, serial);
    }
    app.sync();
}

/// Take hold of the frame by the name above it, which is the only road now.
///
/// The band sits directly against the frame's own top edge, so a press there is
/// the shell's: it reaches no client, `pointer_button` answers `false`, and the
/// application is told nothing at all. That is the difference from the old road,
/// which began with a press the client received and then had to have balanced by
/// a synthetic release and a leave.
fn take_hold(f: &Fixture, app: &mut Application) {
    assert!(f.backend(|s| s.pointer_motion(4.0, -16.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(false),
        "a press on the name was delivered to a client, and the name is the shell's"
    );
    app.sync();
}

fn press(f: &Fixture, app: &mut Application) -> u32 {
    assert!(f.backend(|s| s.pointer_motion(4.0, 5.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(true)
    );
    app.sync();
    app.events.pointer.button_serial
}

#[test]
fn window_move_tracks_geometry_consumes_pointer_and_preserves_keyboard() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    app.xdg.set_window_geometry(2, 3, 12, 12);
    app.surface.commit();
    app.sync();
    let mut other = mapped(&f);
    assert!(f.focus_surface(root.clone()).is_ok());
    app.sync();
    let sizes = app.events.sizes.clone();
    let order = f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>());
    take_hold(&f, &mut app);
    // **The application hears nothing**, which is what changed with ADR 0071: the
    // old road began with a press the client received and then had to have
    // balanced by a synthetic release and a leave. A press on the name is the
    // shell's, so there is nothing to balance.
    assert!(app.events.pointer.buttons.is_empty());
    assert_eq!(app.events.pointer.leaves, 0);
    let motions = app.events.pointer.motion.len();
    // Anchored at the band press, (4, -16); geometry origin is (2, 3), so the
    // placement is (2, 3) + (pointer - anchor) and the buffer origin is that less
    // the geometry origin again.
    assert!(f.backend(|s| s.pointer_motion(34.0, 25.0, 3)).is_ok());
    assert_eq!(origin(&f, &root), (30.0, 41.0));
    // Negative placement and fractional motion preserve the initial anchor.
    assert!(f.backend(|s| s.pointer_motion(-1.25, -1.25, 4)).is_ok());
    assert_eq!(origin(&f, &root), (-5.0, 15.0));
    assert_eq!(f.key(30, KeyState::Pressed).ok(), Some(true));
    assert_eq!(f.key(30, KeyState::Released).ok(), Some(true));
    app.sync();
    other.sync();
    assert_eq!(
        app.events.keyboard.keys,
        [
            (30, wl_keyboard::KeyState::Pressed),
            (30, wl_keyboard::KeyState::Released)
        ]
    );
    assert!(other.events.keyboard.keys.is_empty());
    assert!(other.events.pointer.buttons.is_empty());
    assert_eq!(app.events.pointer.motion.len(), motions);
    assert_eq!(app.events.sizes, sizes);
    assert_eq!(
        f.backend(|s| s.mapped_surfaces().cloned().collect::<Vec<_>>()),
        order
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 5)).ok(),
        Some(false)
    );
    assert!(f.backend(|s| s.pointer_motion(50.0, 50.0, 6)).is_ok());
    assert_eq!(origin(&f, &root), (-5.0, 15.0));
    // And asking for a move, the way an application with its own title bar does,
    // is refused: the frame stays where the person left it.
    let serial = app.events.pointer.button_serial;
    request(&mut app, serial);
    assert!(f.backend(|s| s.pointer_motion(60.0, 60.0, 7)).is_ok());
    assert_eq!(origin(&f, &root), (-5.0, 15.0));
}

#[test]
fn window_move_refuses_forged_foreign_released_and_unmapped_requests() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    let other_root = f.backend(|s| s.mapped_surfaces().nth(1).cloned());
    let serial = press(&f, &mut app);
    request(&mut app, serial.wrapping_add(1000));
    request(&mut other, serial);
    assert!(f.backend(|s| s.pointer_motion(8.0, 8.0, 3)).is_ok());
    assert_eq!(origin(&f, &root), (0.0, 0.0));
    assert_eq!(
        other_root.as_ref().map(|root| origin(&f, root)),
        Some((0.0, 0.0))
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
        Some(true)
    );
    app.sync();
    let release = app.events.pointer.button_serial;
    request(&mut app, release);
    request(&mut app, serial);
    assert!(f.backend(|s| s.pointer_motion(9.0, 9.0, 5)).is_ok());
    assert_eq!(origin(&f, &root), (0.0, 0.0));
    let serial = press(&f, &mut app);
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    request(&mut app, serial);
    assert!(f.backend(|s| s.pointer_motion(12.0, 12.0, 6)).is_ok());
    assert_eq!(origin(&f, &root), (0.0, 0.0));
}

#[test]
fn window_move_invalid_motion_and_extra_buttons_do_not_lose_ownership() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    take_hold(&f, &mut app);
    for x in [f64::NAN, f64::INFINITY, 1_000_005.0] {
        assert!(f.backend(move |s| s.pointer_motion(x, 5.0, 3)).is_err());
        assert_eq!(origin(&f, &root), (0.0, 0.0));
    }
    // A request cannot re-anchor an active drag, and is refused in any case.
    let serial = app.events.pointer.button_serial;
    request(&mut app, serial);
    assert_eq!(
        f.backend(|s| s.pointer_button(0x111, Pressed, 4)).ok(),
        Some(false)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 5)).ok(),
        Some(false)
    );
    // Anchored at the band press, (4, -16).
    assert!(f.backend(|s| s.pointer_motion(14.0, 15.0, 6)).is_ok());
    assert_eq!(origin(&f, &root), (10.0, 31.0));
    assert_eq!(
        f.backend(|s| s.pointer_button(0x111, Released, 7)).ok(),
        Some(false)
    );
    assert!(f.backend(|s| s.pointer_motion(24.0, 25.0, 8)).is_ok());
    assert_eq!(origin(&f, &root), (10.0, 31.0));
}

#[test]
fn window_move_cancels_on_leave_unmap_and_disconnect() {
    for cancel in 0..3 {
        let f = fixture();
        let mut app = mapped(&f);
        let root = f.root();
        take_hold(&f, &mut app);
        assert!(f.backend(|s| s.pointer_motion(14.0, 15.0, 3)).is_ok());
        assert_eq!(origin(&f, &root), (10.0, 31.0));
        match cancel {
            0 => assert!(f.backend(|s| s.pointer_leave()).is_ok()),
            1 => {
                app.surface.attach(None, 0, 0);
                app.surface.commit();
                app.sync();
                app.configure();
                app.attach();
                app.sync();
            }
            _ => {
                drop(app);
                f.wait_for((0, 0));
            }
        }
        assert!(f.backend(|s| s.pointer_motion(24.0, 25.0, 4)).is_ok());
        assert_eq!(
            f.backend(|s| s.pointer_button(0x110, Released, 5)).ok(),
            Some(false)
        );
        if cancel < 2 {
            assert_eq!(
                origin(&f, &root),
                if cancel == 0 {
                    // Where the cancelled drag left it, anchored at the band
                    // press rather than at a press inside the content.
                    (10.0, 31.0)
                } else {
                    (0.0, 0.0)
                }
            );
        }
        assert!(f.render((80, 80), false, 6).is_ok());
    }
}
