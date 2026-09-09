//! The exact nested pump adapter with real private clients.
use super::{
    mapped,
    presentation::present,
    reader::begin,
    reader_frame::{Target, draw},
};
use crate::Fixture;
use alo_shell::{NestedControlInput, NestedPointerEvent as Event, ReaderKeyRoute as Route};
use alo_shortcuts::Action;
use smithay::{
    backend::input::{
        Axis, ButtonState,
        KeyState::{Pressed, Released},
    },
    input::pointer::AxisFrame,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn nested_reader_keys_navigate_repeat_refuse_and_keep_typing() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = NestedControlInput::default();
        let mut target = Target::default();
        draw(s, &mut target, &mut reader, input.reader_pointer(), 260)?;
        for state in [Pressed, Released] {
            assert_eq!(
                input.reader_key(s, Some(&mut reader), true, (30, state, 1))?,
                Route::Forward
            );
        }
        for state in [Pressed, Pressed, Released] {
            let expected = if state == Released {
                Route::Changed
            } else {
                Route::Consumed
            };
            assert_eq!(
                input.reader_key(s, Some(&mut reader), true, (109, state, 2))?,
                expected
            );
        }
        assert_eq!(reader.selected(), 1);
        draw(s, &mut target, &mut reader, input.reader_pointer(), 260)?;
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (104, Pressed, 3))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (104, Released, 4))?,
            Route::Changed
        );
        assert_eq!(reader.selected(), 0);
        draw(s, &mut target, &mut reader, input.reader_pointer(), 260)?;
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (104, Pressed, 5))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (104, Released, 6))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (1, Pressed, 7))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (1, Released, 8))?,
            Route::Dismissed
        );
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn nested_reader_pointer_uses_parent_position_and_drains_after_removal() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = NestedControlInput::default();
        let mut target = Target::default();
        draw(s, &mut target, &mut reader, input.reader_pointer(), 260)?;
        assert_eq!(
            input.route_reader(
                s,
                Some(&mut reader),
                true,
                Some(Event::Motion {
                    x: 259.5,
                    y: 104.0,
                    time: 1
                })
            )?,
            Route::Consumed
        );
        assert_eq!(input.position(), Some((259.5, 104.0)));
        assert_eq!(
            input.route_reader(
                s,
                Some(&mut reader),
                true,
                Some(Event::Axis(AxisFrame::new(2).value(Axis::Vertical, 7.0)))
            )?,
            Route::Consumed
        );
        assert_eq!(
            input.route_reader(
                s,
                Some(&mut reader),
                true,
                Some(Event::Button {
                    code: 0x110,
                    state: ButtonState::Pressed,
                    time: 3
                })
            )?,
            Route::Consumed
        );
        assert_eq!(
            input.route_reader(
                s,
                Some(&mut reader),
                true,
                Some(Event::Button {
                    code: 0x110,
                    state: ButtonState::Released,
                    time: 4
                })
            )?,
            Route::Changed
        );
        assert_eq!(reader.selected(), 1);
        draw(s, &mut target, &mut reader, input.reader_pointer(), 260)?;
        assert_eq!(
            input.route_reader(
                s,
                Some(&mut reader),
                true,
                Some(Event::Button {
                    code: 0x110,
                    state: ButtonState::Pressed,
                    time: 5
                })
            )?,
            Route::Consumed
        );
        input.route_reader(s, None, true, None)?;
        assert_eq!(
            input.route_reader(
                s,
                None,
                true,
                Some(Event::Button {
                    code: 0x110,
                    state: ButtonState::Released,
                    time: 6
                })
            )?,
            Route::Consumed
        );
        assert_eq!(reader.selected(), 1);
        input.route_reader(
            s,
            None,
            true,
            Some(Event::Motion {
                x: 10.0,
                y: 10.0,
                time: 7,
            }),
        )?;
        for state in [ButtonState::Pressed, ButtonState::Released] {
            input.route_reader(
                s,
                None,
                true,
                Some(Event::Button {
                    code: 0x111,
                    state,
                    time: 8,
                }),
            )?;
        }
        input.route_reader(
            s,
            None,
            true,
            Some(Event::Axis(AxisFrame::new(9).value(Axis::Vertical, 3.0))),
        )?;
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.pointer.axes.len(), 1);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn nested_reader_loss_and_malformed_events_cancel_without_losing_releases() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = NestedControlInput::default();
        let mut target = Target::default();
        draw(s, &mut target, &mut reader, input.reader_pointer(), 260)?;
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (109, Pressed, 1))?,
            Route::Consumed
        );
        input.route_reader(s, Some(&mut reader), false, None)?;
        assert_eq!(input.position(), None);
        assert_eq!(
            input.reader_key(s, None, false, (109, Released, 2))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_key(s, None, false, (30, Pressed, 3))?,
            Route::Forward
        );
        assert_eq!(reader.selected(), 0);
        assert!(!s.window_control_reader_presented(&mut reader));
        // Malformed events refuse even without publication and clear actual position.
        for event in [
            Event::Motion {
                x: f64::NAN,
                y: 0.0,
                time: 4,
            },
            Event::Button {
                code: 0,
                state: ButtonState::Pressed,
                time: 5,
            },
            Event::Axis(AxisFrame::new(6).value(Axis::Vertical, f64::INFINITY)),
        ] {
            assert!(input.route_reader(s, None, true, Some(event)).is_err());
            assert_eq!(input.position(), None);
        }
        Ok(())
    })?;
    app.sync();
    assert!(app.events.keyboard.keys.is_empty());
    assert!(app.events.pointer.buttons.is_empty());
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn nested_reader_client_owned_keys_refuse_and_backend_errors_drain() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = NestedControlInput::default();
        let mut target = Target::default();
        // A key pressed before publication remains the client's key on release.
        assert_eq!(
            input.reader_key(s, None, true, (109, Pressed, 1))?,
            Route::Forward
        );
        draw(s, &mut target, &mut reader, input.reader_pointer(), 260)?;
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (109, Pressed, 2))?,
            Route::Forward
        );
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (109, Released, 3))?,
            Route::Forward
        );
        assert_eq!(reader.selected(), 0);
        assert_eq!(
            input.reader_key(s, Some(&mut reader), true, (109, Pressed, 4))?,
            Route::Consumed
        );
        // This is the same cancellation called on pump translation/dispatch failure.
        assert!(
            input
                .reader_key(s, Some(&mut reader), true, (0, Pressed, 5))
                .is_err()
        );
        assert_eq!(
            input.reader_key(s, None, true, (109, Released, 6))?,
            Route::Consumed
        );
        assert_eq!(reader.selected(), 0);
        assert!(!s.window_control_reader_presented(&mut reader));
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
