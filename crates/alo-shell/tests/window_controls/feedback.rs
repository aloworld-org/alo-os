//! Live feedback never becomes transaction authority or leaks client input.
use super::{
    Retire, mapped,
    routing::{CLOSE, DOWN, UP, route},
};
use crate::Fixture;
use alo_shell::{
    WindowControlFeedback as Feedback, WindowControlPointerEvent as Event,
    WindowControlRelease as Release, WindowControlRoute as Route, WindowControlSnapshot,
    WindowControlSnapshotError,
};
use alo_shortcuts::Action;
use smithay::{
    backend::input::{ButtonState, KeyState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn view(
    f: &Fixture,
    root: &WlSurface,
    position: Option<(f64, f64)>,
) -> std::result::Result<WindowControlSnapshot, WindowControlSnapshotError> {
    let root = root.clone();
    f.backend(move |s| s.window_control_feedback(&root, (120, 48), (3, 4), position))
}

fn states(view: &WindowControlSnapshot) -> [Feedback; 3] {
    view.layout().controls().map(|control| control.feedback())
}

#[test]
fn window_controls_feedback_hover_press_release_is_read_only_and_preserves_typing() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    f.backend(|s| s.pointer_motion(2.0, 2.0, 1))?;
    app.sync();
    let motions = app.events.pointer.motion.len();
    let configures = app.events.sizes.clone();
    assert_eq!(
        states(&view(&f, &root, Some(CLOSE))?),
        [Feedback::Idle, Feedback::Idle, Feedback::Hovered]
    );
    route(&f, Some(&root), CLOSE, DOWN)?;
    let held = view(&f, &root, Some(CLOSE))?;
    for _ in 0..3 {
        assert_eq!(
            states(&view(&f, &root, Some(CLOSE))?),
            [Feedback::Idle, Feedback::Idle, Feedback::Pressed]
        );
    }
    // A presentation-only query outside the hit neither cancels nor executes.
    assert_eq!(
        states(&view(&f, &root, Some((4.0, 5.0)))?),
        [Feedback::Idle; 3]
    );
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(app.events.sizes, configures);
    assert_eq!(app.events.pointer.motion.len(), motions);
    assert_eq!(
        route(&f, Some(&root), CLOSE, UP)?,
        Route::Released(Release::Executed(Action::CloseWindow))
    );
    assert_eq!(
        states(&view(&f, &root, Some(CLOSE))?),
        [Feedback::Idle, Feedback::Idle, Feedback::Hovered]
    );
    assert_eq!(
        states(&held),
        [Feedback::Idle, Feedback::Idle, Feedback::Pressed]
    );
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(app.events.pointer.buttons.is_empty());
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    assert_eq!(states(&view(&f, &root, Some(CLOSE))?), [Feedback::Idle; 3]);
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))?);
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    Ok(())
}

#[test]
fn window_controls_feedback_cancelled_disabled_and_unavailable_never_look_armed() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    for position in [
        None,
        Some((f64::NAN, 5.0)),
        Some((120.0, 5.0)),
        Some((35.0, 5.0)),
        Some((40.0, 5.0)),
    ] {
        assert_eq!(states(&view(&f, &root, position)?), [Feedback::Idle; 3]);
    }
    route(&f, Some(&root), (40.0, 5.0), DOWN)?;
    f.render((120, 48), false, 1)?;
    assert_eq!(
        states(&view(&f, &root, Some((40.0, 5.0)))?),
        [Feedback::Idle; 3]
    );
    assert_eq!(
        route(&f, Some(&root), (40.0, 5.0), UP)?,
        Route::Released(Release::Cancelled)
    );
    route(&f, Some(&root), (40.0, 5.0), DOWN)?;
    assert_eq!(
        states(&view(&f, &root, Some((40.0, 5.0)))?),
        [Feedback::Idle, Feedback::Pressed, Feedback::Idle]
    );
    f.backend(|s| s.retire_output(&mut Retire))?;
    assert_eq!(
        states(&view(&f, &root, Some((40.0, 5.0)))?),
        [Feedback::Idle; 3]
    );
    assert!(matches!(
        route(&f, Some(&root), (40.0, 5.0), UP),
        Err(alo_shell::WindowControlRouteError::Release(_))
    ));
    route(&f, Some(&root), CLOSE, DOWN)?;
    route(&f, Some(&root), (4.0, 5.0), Event::Motion)?;
    route(&f, Some(&root), CLOSE, Event::Motion)?;
    assert_eq!(states(&view(&f, &root, Some(CLOSE))?), [Feedback::Idle; 3]);
    route(&f, Some(&root), CLOSE, DOWN)?;
    assert_eq!(
        route(&f, Some(&root), CLOSE, UP)?,
        Route::Released(Release::Cancelled)
    );
    route(&f, Some(&root), CLOSE, DOWN)?;
    f.backend(|s| s.clear_input());
    assert_eq!(states(&view(&f, &root, Some(CLOSE))?), [Feedback::Idle; 3]);
    assert_eq!(
        route(&f, Some(&root), CLOSE, UP)?,
        Route::Released(Release::Cancelled)
    );
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn window_controls_feedback_mapping_target_and_layout_cannot_transfer_pressed_state() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let _other = mapped(&f);
    let original = root.clone();
    let other = f
        .backend(move |s| s.mapped_surfaces().find(|s| **s != original).cloned())
        .ok_or("second root missing")?;
    route(&f, Some(&root), CLOSE, DOWN)?;
    assert_eq!(states(&view(&f, &other, Some(CLOSE))?), [Feedback::Idle; 3]);
    let target = root.clone();
    assert_eq!(
        states(&f.backend(move |s| s.window_control_feedback(
            &target,
            (120, 48),
            (4, 4),
            Some(CLOSE)
        ))?),
        [Feedback::Idle; 3]
    );
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, true))?;
    assert!(matches!(
        view(&f, &root, Some(CLOSE)),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    let target = root.clone();
    f.backend(move |s| s.set_window_minimized(&target, false))?;
    assert_eq!(states(&view(&f, &root, Some(CLOSE))?), [Feedback::Idle; 3]);
    assert_eq!(
        route(&f, Some(&root), CLOSE, UP)?,
        Route::Released(Release::Cancelled)
    );
    route(&f, Some(&root), CLOSE, DOWN)?;
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    assert!(matches!(
        view(&f, &root, Some(CLOSE)),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    app.configure();
    app.attach();
    app.sync();
    assert_eq!(states(&view(&f, &root, Some(CLOSE))?), [Feedback::Idle; 3]);
    assert_eq!(
        route(&f, Some(&root), CLOSE, UP)?,
        Route::Released(Release::Cancelled)
    );
    let foreign = Fixture::new();
    let _foreign_app = mapped(&foreign);
    assert!(matches!(
        view(&f, &foreign.root(), Some(CLOSE)),
        Err(WindowControlSnapshotError::Unmapped)
    ));
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
