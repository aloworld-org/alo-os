//! Real-client input isolation through the combined native/client router.
use super::{Retire, mapped};
use crate::Fixture;
use alo_shell::{
    PaintedWindowControls, WindowControlPointerEvent as Event, WindowControlPressError,
    WindowControlRelease as Release, WindowControlRoute as Route, WindowControlRouteError,
};
use alo_shortcuts::Action;
use smithay::{
    backend::input::{ButtonState, KeyState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) fn route(
    f: &Fixture,
    root: Option<&WlSurface>,
    position: (f64, f64),
    event: Event,
) -> std::result::Result<Route, WindowControlRouteError> {
    let root = root.cloned();
    f.backend(move |s| {
        s.route_window_control_pointer(
            root.as_ref().map(|surface| PaintedWindowControls {
                surface,
                viewport: (120, 48),
                origin: (3, 4),
            }),
            position,
            event,
            1,
        )
    })
}

pub(super) const DOWN: Event = Event::Button(0x110, ButtonState::Pressed);
pub(super) const UP: Event = Event::Button(0x110, ButtonState::Released);
pub(super) const CLOSE: (f64, f64) = (76.0, 5.0);

#[test]
fn window_controls_routing_consumes_native_gesture_and_preserves_client_input() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    assert_eq!(
        route(&f, Some(&root), (2.0, 2.0), Event::Motion)?,
        Route::Client(true)
    );
    app.sync();
    let motions = app.events.pointer.motion.len();
    assert_eq!(route(&f, Some(&root), CLOSE, DOWN)?, Route::Consumed);
    assert_eq!(route(&f, Some(&root), CLOSE, DOWN)?, Route::Consumed);
    assert_eq!(
        route(&f, Some(&root), CLOSE, Event::Motion)?,
        Route::Consumed
    );
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    for state in [ButtonState::Pressed, ButtonState::Released] {
        assert_eq!(
            route(&f, Some(&root), CLOSE, Event::Button(0x111, state))?,
            Route::Client(true)
        );
    }
    assert_eq!(
        route(&f, Some(&root), CLOSE, UP)?,
        Route::Released(Release::Executed(Action::CloseWindow))
    );
    assert_eq!(route(&f, Some(&root), CLOSE, UP)?, Route::Client(false));
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(app.events.pointer.motion.len(), motions);
    assert_eq!(app.events.pointer.buttons.len(), 2); // Only secondary transitions.
    assert_eq!(app.events.keyboard.keys.len(), 2);
    for event in [DOWN, UP] {
        assert_eq!(
            route(&f, Some(&root), (2.0, 2.0), event)?,
            Route::Client(true)
        );
    }
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 4);
    Ok(())
}

#[test]
fn window_controls_routing_replaced_or_removed_target_never_redirects_a_press() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let mut other = mapped(&f);
    let replacement = f
        .backend(move |s| s.mapped_surfaces().find(|s| **s != root).cloned())
        .ok_or("missing other root")?;
    let root = f.root();
    for event in [Event::Motion, DOWN, UP] {
        for target in [None, Some(&replacement)] {
            assert_eq!(route(&f, Some(&root), CLOSE, DOWN)?, Route::Consumed);
            let result = route(&f, target, CLOSE, event)?;
            if matches!(event, Event::Button(_, ButtonState::Released)) {
                assert_eq!(result, Route::Released(Release::Cancelled));
            } else {
                assert_eq!(result, Route::Consumed);
                assert_eq!(
                    route(&f, Some(&root), CLOSE, UP)?,
                    Route::Released(Release::Cancelled)
                );
            }
        }
    }
    app.sync();
    other.sync();
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(other.events.close_requests, 0);
    Ok(())
}

#[test]
fn window_controls_routing_disabled_excursion_and_live_refusal_keep_release_owned() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    let root = f.root();
    let maximize = (40.0, 5.0);
    assert_eq!(route(&f, Some(&root), maximize, DOWN)?, Route::Consumed);
    f.render((120, 48), false, 1)?;
    assert_eq!(
        route(&f, Some(&root), maximize, UP)?,
        Route::Released(Release::Cancelled)
    );
    assert_eq!(route(&f, Some(&root), maximize, DOWN)?, Route::Consumed);
    f.backend(|s| s.retire_output(&mut Retire))?;
    assert!(matches!(
        route(&f, Some(&root), maximize, UP),
        Err(WindowControlRouteError::Release(_))
    ));
    // No pointer seat: a second release reaches the ordinary route and refuses,
    // proving the failed native release already relinquished ownership.
    assert!(matches!(
        route(&f, Some(&root), maximize, UP),
        Err(WindowControlRouteError::Input(_))
    ));
    for outside in [(35.0, 5.0), (f64::NAN, 5.0)] {
        assert_eq!(route(&f, Some(&root), CLOSE, DOWN)?, Route::Consumed);
        assert_eq!(
            route(&f, Some(&root), outside, Event::Motion)?,
            Route::Consumed
        );
        assert_eq!(
            route(&f, Some(&root), CLOSE, Event::Motion)?,
            Route::Consumed
        );
        assert_eq!(
            route(&f, Some(&root), CLOSE, UP)?,
            Route::Released(Release::Cancelled)
        );
    }
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    assert_eq!(app.events.sizes, [(0, 0)]);
    Ok(())
}

#[test]
fn window_controls_routing_client_grab_refusal_preserves_its_matching_release() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    route(&f, None, (2.0, 2.0), Event::Motion)?;
    assert_eq!(route(&f, None, (2.0, 2.0), DOWN)?, Route::Client(true));
    assert!(matches!(
        route(&f, Some(&root), CLOSE, DOWN),
        Err(WindowControlRouteError::Press(
            WindowControlPressError::Busy
        ))
    ));
    assert_eq!(route(&f, Some(&root), CLOSE, UP)?, Route::Client(true));
    assert!(matches!(
        route(
            &f,
            Some(&root),
            CLOSE,
            Event::Button(0, ButtonState::Pressed)
        ),
        Err(WindowControlRouteError::Input(_))
    ));
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
