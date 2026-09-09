//! Real clients exercise the single host presentation's target and focus lifetime.
use super::{
    mapped, maximize,
    routing::{CLOSE, DOWN, UP},
};
use crate::Fixture;
use alo_shell::{
    InputError, PaintedWindowControls, WindowControlPointerEvent as Event,
    WindowControlRelease as Release, WindowControlRoute as Route, WindowControlSnapshotError,
};
use alo_shortcuts::Action;
use smithay::{
    backend::input::{ButtonState, KeyState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn present(
    f: &Fixture,
    root: &WlSurface,
    viewport: (i32, i32),
    origin: (i32, i32),
) -> std::result::Result<(), WindowControlSnapshotError> {
    let root = root.clone();
    f.backend(move |s| {
        s.present_window_controls(Some(PaintedWindowControls {
            surface: &root,
            viewport,
            origin,
        }))
    })
}

fn publish(f: &Fixture, root: &WlSurface) -> Result {
    Ok(present(f, root, (120, 48), (3, 4))?)
}

fn focus(f: &Fixture) -> bool {
    f.backend(|s| s.focus_window_control(Some(Action::MaximiseWindow)))
}

fn focused(f: &Fixture) -> Result<bool> {
    Ok(
        f.backend(|s| s.presented_window_control_label(None, (100, 40)))?
            .is_some(),
    )
}

fn route(f: &Fixture, event: Event) -> Result<Route> {
    Ok(f.backend(move |s| s.route_presented_window_control_pointer(CLOSE, event, 1))?)
}

#[test]
fn window_controls_presentation_refresh_disabled_focus_and_exactly_once_input() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    app.sync();
    let sizes = app.events.sizes.clone();
    publish(&f, &root)?;
    assert!(focus(&f));
    publish(&f, &root)?;
    assert!(focused(&f)?);
    let label = f
        .backend(|s| s.presented_window_control_label(None, (100, 40)))?
        .ok_or("missing disabled focus")?;
    assert_eq!(label.control.action(), Action::MaximiseWindow);
    assert!(!label.control.enabled());
    let view = f
        .backend(|s| s.presented_window_controls(Some(CLOSE)))
        .ok_or("missing current feedback")?;
    assert_eq!(view.surface(), &root);
    assert_eq!(
        view.layout().controls().map(|control| control.feedback()),
        [
            alo_shell::WindowControlFeedback::Idle,
            alo_shell::WindowControlFeedback::Idle,
            alo_shell::WindowControlFeedback::Hovered,
        ]
    );
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    assert_eq!(route(&f, DOWN)?, Route::Consumed);
    assert!(!focused(&f)?);
    assert!(!focus(&f));
    publish(&f, &root)?; // An unchanged frame must not cancel a held press.
    assert_eq!(
        route(&f, UP)?,
        Route::Released(Release::Executed(Action::CloseWindow))
    );
    assert_eq!(route(&f, UP)?, Route::Client(false));
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert!(app.events.pointer.buttons.is_empty());
    assert_eq!(app.events.sizes, sizes);
    // Focus requests outside visible strip geometry clear the previous name.
    assert!(focus(&f));
    assert!(!f.backend(|s| s.focus_window_control(None)));
    assert!(!focused(&f)?);
    let hover = f
        .backend(|s| s.presented_window_control_label(Some(CLOSE), (100, 40)))?
        .ok_or("missing fresh hover")?;
    assert_eq!(hover.control.action(), Action::CloseWindow);
    assert!(!focused(&f)?); // Hover was not retained.
    assert!(matches!(
        f.backend(|s| s.presented_window_control_label(Some(CLOSE), (8, 40))),
        Err(alo_shell::WindowControlLabelError::Geometry)
    ));
    assert!(focus(&f));
    assert!(!f.backend(|s| s.focus_window_control(Some(Action::Launcher))));
    assert!(!focused(&f)?);
    present(&f, &root, (50, 48), (3, 4))?;
    assert!(!f.backend(|s| s.focus_window_control(Some(Action::CloseWindow))));
    assert!(!focused(&f)?);
    Ok(())
}

#[test]
fn window_controls_presentation_replacement_relayout_and_failure_retire_authority() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    let replacement = f
        .backend({
            let root = root.clone();
            move |s| {
                s.mapped_surfaces()
                    .find(|surface| **surface != root)
                    .cloned()
            }
        })
        .ok_or("second root missing")?;
    let foreign = Fixture::new();
    let _foreign_app = mapped(&foreign);
    for case in 0..6 {
        publish(&f, &root)?;
        assert!(focus(&f));
        // First prove focus retirement without a pointer event dismissing it.
        let change = |f: &Fixture| -> Result {
            match case {
                0 => publish(f, &replacement)?,
                1 => present(f, &root, (119, 48), (3, 4))?,
                2 => present(f, &root, (120, 48), (4, 4))?,
                3 => assert!(present(f, &root, (0, 48), (3, 4)).is_err()),
                4 => assert!(matches!(
                    present(f, &foreign.root(), (120, 48), (3, 4)),
                    Err(WindowControlSnapshotError::Unmapped)
                )),
                _ => f.backend(|s| s.present_window_controls(None))?,
            }
            Ok(())
        };
        change(&f)?;
        assert!(!focused(&f)?);
        publish(&f, &root)?;
        assert_eq!(route(&f, DOWN)?, Route::Consumed);
        change(&f)?;
        publish(&f, &root)?; // Returning cannot revive the retired gesture.
        assert_eq!(route(&f, UP)?, Route::Released(Release::Cancelled));
    }
    app.sync();
    other.sync();
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(other.events.close_requests, 0);
    Ok(())
}

#[test]
fn window_controls_presentation_unobserved_hide_remap_death_and_intent_retire() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    publish(&f, &root)?;
    assert!(focus(&f));
    let target = root.clone();
    f.backend(move |s| {
        s.set_window_minimized(&target, true)?;
        s.set_window_minimized(&target, false)
    })?;
    assert!(!focused(&f)?); // No host observation between hide and reveal.
    assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
    publish(&f, &root)?;
    assert!(focus(&f));
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    app.configure();
    app.attach();
    app.sync();
    // Republishing the same protocol identity also detects the new mapping.
    publish(&f, &root)?;
    assert!(!focused(&f)?);
    assert!(focus(&f));
    f.render((120, 48), false, 1)?;
    maximize(&f, &root, true)?;
    assert!(!focused(&f)?);
    assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
    publish(&f, &root)?;
    assert!(focus(&f));
    drop(app);
    f.wait_for((0, 0));
    assert!(!focused(&f)?);
    assert!(!focus(&f));
    Ok(())
}

#[test]
fn window_controls_presentation_backend_loss_retires_even_without_pointer() -> Result {
    for with_pointer in [false, true] {
        let f = Fixture::keyboard();
        if with_pointer {
            f.backend(|s| s.enable_pointer())?;
        }
        let mut app = mapped(&f);
        let root = f.root();
        for path in 0..4 {
            for held in [false, true] {
                publish(&f, &root)?;
                assert!(focus(&f));
                if held {
                    assert_eq!(route(&f, DOWN)?, Route::Consumed);
                }
                let result = f.backend(move |s| match path {
                    0 => s.pointer_leave(),
                    1 => s.nested_pointer(false, None),
                    2 => s.direct_pointer(false, (0, 0), None),
                    _ => {
                        s.clear_input();
                        Ok(())
                    }
                });
                if with_pointer || path == 3 {
                    result?;
                } else {
                    assert!(matches!(result, Err(InputError::PointerUnavailable)));
                }
                assert!(!focused(&f)?);
                assert!(f.backend(|s| s.presented_window_controls(None)).is_none());
                if held {
                    assert_eq!(route(&f, UP)?, Route::Released(Release::Cancelled));
                }
            }
        }
        app.sync();
        assert_eq!(app.events.close_requests, 0);
        assert!(app.events.pointer.buttons.is_empty());
    }
    Ok(())
}

#[test]
fn window_controls_presentation_client_ownership_dismisses_focus_without_return() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    publish(&f, &f.root())?;
    assert!(focus(&f));
    f.backend(|s| s.pointer_motion(2.0, 2.0, 1))?;
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    assert!(!focused(&f)?);
    assert!(!focus(&f));
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))?);
    assert!(!focused(&f)?);
    assert!(focus(&f));
    // Current pointer motion dismisses native focus; it is not cached hover.
    assert_eq!(route(&f, Event::Motion)?, Route::Client(true));
    assert!(!focused(&f)?);
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
