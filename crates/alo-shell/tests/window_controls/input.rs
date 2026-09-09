//! Real-client ownership and refusal checks for native control transactions.
use super::{Retire, mapped, maximize};
use crate::Fixture;
use alo_shell::{
    WindowControlPressError, WindowControlRelease as Release, WindowControlReleaseError,
    WindowControlSnapshotError, WindowMaximizeError,
};
use alo_shortcuts::Action;
use smithay::{
    backend::input::{ButtonState, KeyState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn press(
    f: &Fixture,
    root: &WlSurface,
    x: f64,
) -> std::result::Result<bool, WindowControlPressError> {
    let root = root.clone();
    f.backend(move |s| s.press_window_control(&root, (120, 48), (3, 4), (x, 5.0)))
}
fn release(f: &Fixture, x: f64) -> std::result::Result<Release, WindowControlReleaseError> {
    f.backend(move |s| s.release_window_control((120, 48), (3, 4), (x, 5.0)))
}

#[test]
fn window_controls_input_close_is_exactly_once_and_preserves_client_input() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    f.focus_surface(root.clone())?;
    f.backend(|s| s.pointer_motion(2.0, 2.0, 1))?;
    app.sync();
    assert!(press(&f, &root, 76.0)?);
    assert!(f.key(30, KeyState::Pressed)?);
    // A duplicate press on a different control must not replace close ownership.
    assert!(press(&f, &root, 4.0)?);
    assert_eq!(release(&f, 76.0)?, Release::Executed(Action::CloseWindow));
    assert_eq!(release(&f, 76.0)?, Release::Unowned);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    other.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(other.events.close_requests, 0);
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(other.events.keyboard.keys.is_empty());
    assert!(app.events.pointer.buttons.is_empty());
    assert!(other.events.pointer.buttons.is_empty());
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))?);
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(f.backend(|s| s.mapped_surfaces().count()), 2);
    Ok(())
}

#[test]
fn window_controls_input_disabled_hit_cannot_arm_when_output_arrives() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    assert!(press(&f, &root, 40.0)?);
    f.render((120, 48), false, 1)?;
    assert!(press(&f, &root, 40.0)?);
    assert_eq!(release(&f, 40.0)?, Release::Cancelled);
    app.sync();
    assert_eq!(app.events.sizes, [(0, 0)]);
    assert!(press(&f, &root, 40.0)?);
    assert_eq!(
        release(&f, 40.0)?,
        Release::Executed(Action::MaximiseWindow)
    );
    app.sync();
    assert_eq!(app.events.maximized.last(), Some(&true));
    assert!(press(&f, &root, 40.0)?);
    assert_eq!(
        release(&f, 40.0)?,
        Release::Executed(Action::MaximiseWindow)
    );
    app.sync();
    assert_eq!(app.events.maximized.last(), Some(&false));
    assert!(press(&f, &root, 4.0)?);
    assert_eq!(release(&f, 4.0)?, Release::Executed(Action::MinimiseWindow));
    assert_eq!(f.backend(|s| s.minimized_surfaces().count()), 1);
    Ok(())
}

#[test]
fn window_controls_input_live_policy_refuses_and_changed_intent_cancels() -> Result {
    let f = Fixture::new();
    let _app = mapped(&f);
    let root = f.root();
    f.render((120, 48), false, 1)?;
    assert!(press(&f, &root, 40.0)?);
    f.backend(|s| s.retire_output(&mut Retire))?;
    assert!(matches!(
        release(&f, 40.0),
        Err(WindowControlReleaseError::Maximize(
            WindowMaximizeError::OutputUnavailable
        ))
    ));
    assert_eq!(release(&f, 40.0)?, Release::Unowned);
    f.render((120, 48), false, 2)?;
    assert!(press(&f, &root, 40.0)?);
    maximize(&f, &root, true)?;
    assert_eq!(release(&f, 40.0)?, Release::Cancelled);
    Ok(())
}

#[test]
fn window_controls_input_cancellation_and_changed_hit_never_click_through() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    for x in [35.0, -1.0, 120.0, f64::NAN, f64::INFINITY] {
        assert!(!press(&f, &root, x)?);
        assert_eq!(release(&f, 76.0)?, Release::Unowned);
    }
    for x in [4.0, 35.0, -1.0, f64::NAN] {
        assert!(press(&f, &root, 76.0)?);
        assert_eq!(release(&f, x)?, Release::Cancelled);
    }
    assert!(press(&f, &root, 76.0)?);
    assert_eq!(
        f.backend(|s| s.release_window_control((120, 48), (4, 4), (76.0, 5.0)))?,
        Release::Cancelled
    );
    assert!(press(&f, &root, 76.0)?);
    f.backend(|s| {
        s.cancel_window_control();
        s.cancel_window_control();
    });
    assert!(press(&f, &root, 76.0)?);
    assert_eq!(release(&f, 76.0)?, Release::Cancelled);
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    assert!(press(&f, &root, 76.0)?);
    assert_eq!(release(&f, 76.0)?, Release::Executed(Action::CloseWindow));
    Ok(())
}

#[test]
fn window_controls_input_mapping_and_visibility_retirement_are_permanent() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    assert!(press(&f, &root, 76.0)?);
    let target = root.clone();
    f.backend(move |s| {
        s.set_window_minimized(&target, true)?;
        s.set_window_minimized(&target, false)
    })?; // Hide/reveal entirely between dispatches.
    assert_eq!(release(&f, 76.0)?, Release::Cancelled);
    assert!(press(&f, &root, 76.0)?);
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    app.configure();
    app.attach();
    app.sync();
    assert_eq!(f.root(), root);
    assert_eq!(release(&f, 76.0)?, Release::Cancelled);
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    assert!(press(&f, &root, 76.0)?);
    drop(app);
    f.wait_for((0, 0));
    let mut replacement = mapped(&f);
    assert_eq!(release(&f, 76.0)?, Release::Cancelled);
    replacement.sync();
    assert_eq!(replacement.events.close_requests, 0);
    assert!(matches!(
        press(&f, &root, 76.0),
        Err(WindowControlPressError::Snapshot(
            WindowControlSnapshotError::Unmapped
        ))
    ));
    Ok(())
}

#[test]
fn window_controls_input_foreign_targets_and_client_grabs_refuse_without_stealing() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    let foreign = Fixture::new();
    let _other = mapped(&foreign);
    let other_root = foreign.root();
    assert!(matches!(
        press(&f, &other_root, 76.0),
        Err(WindowControlPressError::Snapshot(
            WindowControlSnapshotError::Unmapped
        ))
    ));
    assert_eq!(release(&f, 76.0)?, Release::Unowned);
    f.backend(|s| s.pointer_motion(2.0, 2.0, 1))?;
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    assert!(matches!(
        press(&f, &root, 76.0),
        Err(WindowControlPressError::Busy)
    ));
    assert_eq!(release(&f, 76.0)?, Release::Unowned);
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))?);
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn window_controls_input_popup_started_after_press_refuses_live_maximize() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    let root = f.root();
    f.render((120, 48), false, 1)?;
    f.focus_surface(root.clone())?;
    assert!(press(&f, &root, 40.0)?);
    assert!(f.key(30, KeyState::Pressed)?);
    app.sync();
    let (surface, xdg, role) = app.popup(true, 1);
    app.grab_popup(&role, app.events.keyboard.key_serial);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    assert!(matches!(
        release(&f, 40.0),
        Err(WindowControlReleaseError::Maximize(
            WindowMaximizeError::Busy
        ))
    ));
    assert_eq!(release(&f, 40.0)?, Release::Unowned);
    assert!(matches!(
        press(&f, &root, 76.0),
        Err(WindowControlPressError::Busy)
    ));
    // Popup focus transfer consumes its initiating key; the control refusal
    // must neither revive that key nor interfere with subsequent ordinary input.
    assert!(!f.key(30, KeyState::Released)?);
    assert!(f.key(48, KeyState::Pressed)?);
    assert!(f.key(48, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.popups.done, 0);
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(app.events.maximized.last(), Some(&false));
    Ok(())
}
