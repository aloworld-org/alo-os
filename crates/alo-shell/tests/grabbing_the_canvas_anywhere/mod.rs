//! **`⊞` and a drag take hold of the canvas from anywhere**, a window
//! included — verified on a real Wayland client connection, because the case
//! that matters is what the application under the pointer is told: nothing.
//!
//! The owner's decision of 2026-11-13, `crate::canvas_grab_anywhere` in the
//! shell. The cases that need no client — the Dock and the panel, letting go of
//! `⊞` mid-drag, less motion, two displays — are beside that module.

use super::support::{Application, Fixture};
use alo_shell::InputError;
use smithay::backend::input::{ButtonState, KeyState};

/// `KEY_LEFTMETA`, the `⊞` key.
const SUPER: u32 = 125;
/// `BTN_LEFT`.
const LEFT: u32 = 0x110;

/// A desktop with a pointer and one real application, mapped and unfocused.
fn a_desktop_with_an_application() -> Result<(Fixture, Application), InputError> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();
    Ok((f, app))
}

/// Where this session is looking, as a pair.
fn looking_at(f: &Fixture) -> (i32, i32) {
    f.backend(|s| {
        let at = s.the_camera().at();
        (at.x, at.y)
    })
}

/// Hold `⊞`, press at `from`, and drag by `by`, leaving both held.
fn take_hold_and_drag(f: &Fixture, from: (f64, f64), by: (f64, f64)) -> Result<(), InputError> {
    f.backend(move |s| s.pointer_motion(from.0, from.1, 10))?;
    f.backend(|s| s.keyboard_key(SUPER, KeyState::Pressed, 11))?;
    let told = f.backend(|s| s.pointer_button(LEFT, ButtonState::Pressed, 12))?;
    assert!(
        !told,
        "the press that took hold of the canvas was delivered"
    );
    f.backend(move |s| s.pointer_motion(from.0 + by.0, from.1 + by.1, 13))?;
    Ok(())
}

/// Let go of the button and of `⊞`.
fn let_go(f: &Fixture) -> Result<(), InputError> {
    let told = f.backend(|s| s.pointer_button(LEFT, ButtonState::Released, 14))?;
    assert!(!told, "the release that let go of the canvas was delivered");
    f.backend(|s| s.keyboard_key(SUPER, KeyState::Released, 15))?;
    Ok(())
}

/// **Over a window, `⊞` and a drag pan the canvas, and the window is told
/// nothing**: no press, no movement, no release, and not the keyboard.
///
/// The control is at the end: an ordinary click on the same spot, which the
/// application does hear. Without it, a window that heard nothing because the
/// pointer was not on it would pass the first half.
#[test]
fn over_a_window_super_and_a_drag_pan_and_the_window_hears_nothing() -> Result<(), InputError> {
    let (f, mut app) = a_desktop_with_an_application()?;
    f.backend(|s| s.pointer_motion(4.0, 4.0, 9))?;
    app.sync();
    assert!(
        !app.events.pointer.enters.is_empty(),
        "the control: the pointer is not on the application"
    );
    let buttons = app.events.pointer.buttons.len();
    let movements = app.events.pointer.motion.len();
    let before = looking_at(&f);

    take_hold_and_drag(&f, (4.0, 4.0), (100.0, 30.0))?;
    app.sync();

    assert_eq!(
        looking_at(&f),
        (before.0 - 100, before.1 - 30),
        "a drag of (100, 30) pixels over a window did not pan the canvas by them"
    );
    assert_eq!(
        app.events.pointer.motion.len(),
        movements,
        "the application under the pointer was told about a drag that was the canvas's"
    );

    let_go(&f)?;
    app.sync();
    assert_eq!(
        app.events.pointer.buttons.len(),
        buttons,
        "the application under the pointer was clicked by a press that took hold of the canvas"
    );
    assert!(
        app.events.keyboard.enters.is_empty(),
        "taking hold of the canvas gave the application the keyboard"
    );
    assert!(!f.backend(|s| s.the_canvas_is_held()));

    // The control: the same window, an ordinary click, heard. It moved with
    // the canvas, so it is reached where it is now.
    let now = (4.0 + 100.0, 4.0 + 30.0);
    f.backend(move |s| s.pointer_motion(now.0, now.1, 20))?;
    f.backend(|s| s.pointer_button(LEFT, ButtonState::Pressed, 21))?;
    f.backend(|s| s.pointer_button(LEFT, ButtonState::Released, 22))?;
    app.sync();
    assert_eq!(
        app.events.pointer.buttons.len(),
        buttons + 2,
        "the control: an ordinary click on the window after the pan did not reach it"
    );
    Ok(())
}

/// **Over empty canvas it is the same gesture**, with the same arithmetic, and
/// the application elsewhere hears nothing either.
#[test]
fn over_empty_canvas_super_and_a_drag_pan_the_same() -> Result<(), InputError> {
    let (f, mut app) = a_desktop_with_an_application()?;
    app.sync();
    let buttons = app.events.pointer.buttons.len();
    let before = looking_at(&f);

    take_hold_and_drag(&f, (400.0, 300.0), (100.0, 30.0))?;
    assert_eq!(looking_at(&f), (before.0 - 100, before.1 - 30));
    let_go(&f)?;
    app.sync();

    assert_eq!(app.events.pointer.buttons.len(), buttons);
    assert!(app.events.keyboard.enters.is_empty());
    Ok(())
}
