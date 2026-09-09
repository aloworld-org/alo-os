//! Native gesture cancellation exercised with real protocol clients.
use super::{mapped, maximize};
use crate::Fixture;
use alo_shell::{InputError, WindowControlRelease as Release};
use alo_shortcuts::Action;
use smithay::backend::input::{ButtonState, KeyState};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn press(f: &Fixture) -> Result {
    let root = f.root();
    assert!(f.backend(move |s| { s.press_window_control(&root, (120, 48), (3, 4), (76.0, 5.0)) })?);
    Ok(())
}

fn motion(f: &Fixture, position: (f64, f64)) -> bool {
    f.backend(move |s| s.window_control_motion((120, 48), (3, 4), position))
}

fn release(f: &Fixture) -> Result<Release> {
    Ok(f.backend(|s| s.release_window_control((120, 48), (3, 4), (76.0, 5.0)))?)
}

#[test]
fn window_controls_motion_inside_executes_and_preserves_ordinary_input() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    f.focus_surface(f.root())?;
    f.backend(|s| s.pointer_motion(2.0, 2.0, 1))?;
    assert!(!motion(&f, (76.0, 5.0)));
    press(&f)?;
    assert!(motion(&f, (100.5, 20.25)));
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    assert_eq!(release(&f)?, Release::Executed(Action::CloseWindow));
    assert!(!motion(&f, (76.0, 5.0)));
    app.sync();
    assert_eq!(app.events.close_requests, 1);
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert!(app.events.pointer.buttons.is_empty());
    // The observation API has not moved ordinary pointer focus or seat position.
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))?);
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    Ok(())
}

#[test]
fn window_controls_motion_out_and_back_never_rearms() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    for position in [
        (4.0, 5.0),   // another control
        (74.0, 5.0),  // transparent gap
        (120.0, 5.0), // outside output
        (76.0, 36.0), // half-open lower edge
        (f64::NAN, 5.0),
        (76.0, f64::INFINITY),
    ] {
        press(&f)?;
        assert!(motion(&f, position));
        assert!(motion(&f, (76.0, 5.0)));
        press(&f)?; // duplicate cannot rearm
        assert_eq!(release(&f)?, Release::Cancelled);
        assert_eq!(release(&f)?, Release::Unowned);
    }
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    press(&f)?;
    assert_eq!(release(&f)?, Release::Executed(Action::CloseWindow));
    Ok(())
}

#[test]
fn window_controls_motion_layout_and_intent_changes_cancel_permanently() -> Result {
    let f = Fixture::new();
    let mut app = mapped(&f);
    for (viewport, origin) in [((119, 48), (3, 4)), ((120, 48), (4, 4)), ((0, 48), (3, 4))] {
        press(&f)?;
        assert!(f.backend(move |s| s.window_control_motion(viewport, origin, (76.0, 5.0))));
        assert!(motion(&f, (76.0, 5.0)));
        assert_eq!(release(&f)?, Release::Cancelled);
    }
    f.render((120, 48), false, 1)?;
    let root = f.root();
    let target = root.clone();
    assert!(f.backend(move |s| s.press_window_control(&target, (120, 48), (3, 4), (40.0, 5.0)))?);
    maximize(&f, &root, true)?;
    assert!(motion(&f, (40.0, 5.0)));
    maximize(&f, &root, false)?;
    assert_eq!(
        f.backend(|s| s.release_window_control((120, 48), (3, 4), (40.0, 5.0)))?,
        Release::Cancelled
    );
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn window_controls_motion_input_loss_cancels_even_without_pointer_capability() -> Result {
    for with_pointer in [false, true] {
        let f = Fixture::keyboard();
        if with_pointer {
            f.backend(|s| s.enable_pointer())?;
        }
        let mut app = mapped(&f);
        for path in 0..4 {
            press(&f)?;
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
            assert!(motion(&f, (76.0, 5.0)));
            press(&f)?;
            assert_eq!(release(&f)?, Release::Cancelled);
            assert_eq!(release(&f)?, Release::Unowned);
        }
        app.sync();
        assert_eq!(app.events.close_requests, 0);
        assert!(app.events.pointer.buttons.is_empty());
        f.focus_surface(f.root())?;
        assert!(f.key(30, KeyState::Pressed)?);
        assert!(f.key(30, KeyState::Released)?);
        app.sync();
        assert_eq!(app.events.keyboard.keys.len(), 2);
        press(&f)?;
        assert_eq!(release(&f)?, Release::Executed(Action::CloseWindow));
    }
    Ok(())
}
