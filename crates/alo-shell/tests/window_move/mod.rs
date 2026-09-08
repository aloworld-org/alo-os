//! Real XDG requests and backend input prove drag authority and cancellation.
use super::{Application, Fixture};
use smithay::{
    backend::input::{
        ButtonState::{Pressed, Released},
        KeyState,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};
use wayland_client::protocol::{wl_keyboard, wl_pointer};

#[test]
fn window_move_accepts_a_child_press_and_survives_unrelated_unmap() {
    let f = fixture();
    let mut app = mapped(&f);
    let root = f.root();
    let (_child, _role) = app.child((24, 32));
    app.surface.commit();
    app.sync();
    let mut other = mapped(&f);
    assert!(f.backend(|s| s.pointer_motion(25.0, 33.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(true)
    );
    app.sync();
    let serial = app.events.pointer.button_serial;
    request(&mut app, serial);
    other.surface.attach(None, 0, 0);
    other.surface.commit();
    other.sync();
    assert!(f.backend(|s| s.pointer_motion(35.0, 43.0, 3)).is_ok());
    assert_eq!(origin(&f, &root), (10.0, 10.0));
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 4)).ok(),
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

fn request(app: &mut Application, serial: u32) {
    assert!(app.events.keyboard.seat.is_some());
    if let Some(seat) = &app.events.keyboard.seat {
        app.toplevel._move(seat, serial);
    }
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
    let serial = press(&f, &mut app);
    request(&mut app, serial);
    assert_eq!(
        app.events.pointer.buttons,
        [
            (0x110, wl_pointer::ButtonState::Pressed),
            (0x110, wl_pointer::ButtonState::Released)
        ]
    );
    assert_eq!(app.events.pointer.leaves, 1);
    let motions = app.events.pointer.motion.len();
    assert!(f.backend(|s| s.pointer_motion(34.0, 25.0, 3)).is_ok());
    assert_eq!(origin(&f, &root), (30.0, 20.0));
    // Negative placement and fractional motion preserve the initial anchor.
    assert!(f.backend(|s| s.pointer_motion(-1.25, -1.25, 4)).is_ok());
    assert_eq!(origin(&f, &root), (-5.0, -6.0));
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
    assert_eq!(origin(&f, &root), (-5.0, -6.0));
    request(&mut app, serial);
    assert!(f.backend(|s| s.pointer_motion(60.0, 60.0, 7)).is_ok());
    assert_eq!(origin(&f, &root), (-5.0, -6.0));
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
    let serial = press(&f, &mut app);
    request(&mut app, serial);
    for x in [f64::NAN, f64::INFINITY, 1_000_005.0] {
        assert!(f.backend(move |s| s.pointer_motion(x, 5.0, 3)).is_err());
        assert_eq!(origin(&f, &root), (0.0, 0.0));
    }
    // Duplicate requests cannot re-anchor an active drag.
    request(&mut app, serial);
    assert_eq!(
        f.backend(|s| s.pointer_button(0x111, Pressed, 4)).ok(),
        Some(false)
    );
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Released, 5)).ok(),
        Some(false)
    );
    assert!(f.backend(|s| s.pointer_motion(14.0, 15.0, 6)).is_ok());
    assert_eq!(origin(&f, &root), (10.0, 10.0));
    assert_eq!(
        f.backend(|s| s.pointer_button(0x111, Released, 7)).ok(),
        Some(false)
    );
    assert!(f.backend(|s| s.pointer_motion(24.0, 25.0, 8)).is_ok());
    assert_eq!(origin(&f, &root), (10.0, 10.0));
}

#[test]
fn window_move_cancels_on_leave_unmap_and_disconnect() {
    for cancel in 0..3 {
        let f = fixture();
        let mut app = mapped(&f);
        let root = f.root();
        let serial = press(&f, &mut app);
        request(&mut app, serial);
        assert!(f.backend(|s| s.pointer_motion(14.0, 15.0, 3)).is_ok());
        assert_eq!(origin(&f, &root), (10.0, 10.0));
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
                    (10.0, 10.0)
                } else {
                    (0.0, 0.0)
                }
            );
        }
        assert!(f.render((80, 80), false, 6).is_ok());
    }
}
