//! Semantic pointer transactions with real private client mappings and input.
use super::{mapped, presentation::present, reader::begin};
use crate::Fixture;
use alo_shell::{
    ReaderKeyCommand as Command, ReaderKeyRoute as Route, ReaderPointerFeedback,
    ReaderPointerHit as Hit, WindowControlReaderPointer,
};
use alo_shortcuts::Action;
use smithay::backend::input::{
    ButtonState::{Pressed, Released},
    KeyState,
};

type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

const LEFT: u32 = 0x110;
const RIGHT: u32 = 0x111;
const NEXT: Option<Hit> = Some(Hit::Command(Command::Next));

#[test]
fn native_reader_pointer_navigation_feedback_and_ordinary_typing() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (320, 180), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| {
        let mut pointer = WindowControlReaderPointer::default();
        for (command, expected, index) in [
            (Command::Previous, Route::Consumed, 0),
            (Command::Next, Route::Changed, 1),
            (Command::Next, Route::Changed, 2),
            (Command::Next, Route::Consumed, 2),
            (Command::Previous, Route::Changed, 1),
        ] {
            let hit = Some(Hit::Command(command));
            let enabled = expected == Route::Changed;
            assert!(pointer.motion(s, Some(&mut reader), hit));
            assert_eq!(
                pointer.feedback(s, Some(&mut reader)),
                ReaderPointerFeedback {
                    hovered: enabled.then_some(command),
                    pressed: None
                }
            );
            assert_eq!(
                pointer.button(s, Some(&mut reader), hit, LEFT, Pressed),
                Route::Consumed
            );
            assert_eq!(
                pointer.feedback(s, Some(&mut reader)).pressed,
                enabled.then_some(command)
            );
            assert_eq!(
                pointer.button(s, Some(&mut reader), hit, LEFT, Pressed),
                Route::Consumed
            );
            assert_eq!(
                pointer.button(s, Some(&mut reader), hit, LEFT, Released),
                expected
            );
            assert_eq!(reader.selected(), index);
            assert_eq!(
                pointer.button(s, Some(&mut reader), hit, LEFT, Released),
                Route::Forward
            );
        }
        let dismiss = Some(Hit::Command(Command::Dismiss));
        assert_eq!(
            pointer.button(s, Some(&mut reader), dismiss, LEFT, Pressed),
            Route::Consumed
        );
        assert_eq!(
            pointer.button(s, Some(&mut reader), dismiss, LEFT, Released),
            Route::Dismissed
        );
        assert_eq!(
            pointer.feedback(s, Some(&mut reader)),
            ReaderPointerFeedback::default()
        );
        assert!(s.read_window_control_page(&mut reader, 1).is_none());
    });
    assert!(f.key(30, KeyState::Pressed)?);
    assert!(f.key(30, KeyState::Released)?);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn native_reader_pointer_disarms_without_release_leaks_or_rearming() -> Result {
    for case in 0..9 {
        let f = Fixture::keyboard();
        f.backend(|s| s.enable_pointer())?;
        let mut app = mapped(&f);
        present(&f, &f.root(), (320, 180), (3, 4))?;
        let mut reader = begin(&f, Action::CloseWindow)?;
        f.backend(move |s| {
            let mut pointer = WindowControlReaderPointer::default();
            assert_eq!(
                pointer.button(s, Some(&mut reader), NEXT, LEFT, Pressed),
                Route::Consumed
            );
            match case {
                0 => pointer.cancel(),
                1 => {
                    assert!(pointer.motion(s, Some(&mut reader), None));
                }
                2 => {
                    assert!(pointer.motion(s, Some(&mut reader), Some(Hit::Content)));
                }
                3 => {
                    assert!(pointer.motion(
                        s,
                        Some(&mut reader),
                        Some(Hit::Command(Command::Dismiss))
                    ));
                }
                4 => {
                    assert_eq!(
                        pointer.button(s, Some(&mut reader), NEXT, RIGHT, Pressed),
                        Route::Consumed
                    );
                    assert_eq!(
                        pointer.button(s, Some(&mut reader), NEXT, RIGHT, Released),
                        Route::Consumed
                    );
                }
                5 => {
                    assert!(s.read_window_control_page(&mut reader, 1).is_some());
                    assert!(s.read_window_control_page(&mut reader, 0).is_some());
                }
                6 => reader.dismiss(),
                7 => s.retire_window_controls(),
                _ => s.clear_input(),
            }
            assert!(pointer.motion(s, Some(&mut reader), NEXT));
            assert_eq!(pointer.feedback(s, Some(&mut reader)).pressed, None);
            assert_eq!(
                pointer.button(s, Some(&mut reader), NEXT, LEFT, Pressed),
                Route::Consumed
            );
            assert_eq!(
                pointer.button(s, Some(&mut reader), NEXT, LEFT, Released),
                Route::Consumed
            );
            assert_eq!(reader.selected(), 0);
            assert_eq!(
                pointer.button(s, None, None, LEFT, Released),
                Route::Forward
            );
            assert!(!pointer.motion(s, None, None));
        });
        app.sync();
        assert_eq!(app.events.close_requests, 0);
    }
    Ok(())
}

#[test]
fn native_reader_pointer_content_buttons_replacement_and_foreign_refusals() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let _app = mapped(&f);
    present(&f, &f.root(), (320, 180), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    let mut replacement = begin(&f, Action::CloseWindow)?;
    let (mut pointer, mut reader) = f.backend(move |s| {
        let mut pointer = WindowControlReaderPointer::default();
        for button in LEFT..=0x117 {
            for hit in [Some(Hit::Content), Some(Hit::Command(Command::Previous))] {
                assert_eq!(
                    pointer.button(s, Some(&mut reader), hit, button, Pressed),
                    Route::Consumed
                );
                assert_eq!(
                    pointer.button(s, Some(&mut reader), hit, button, Released),
                    Route::Consumed
                );
                assert_eq!(reader.selected(), 0);
            }
        }
        for button in RIGHT..=0x117 {
            assert_eq!(
                pointer.button(s, Some(&mut reader), NEXT, button, Pressed),
                Route::Consumed
            );
            assert_eq!(
                pointer.button(s, Some(&mut reader), NEXT, button, Released),
                Route::Consumed
            );
            assert_eq!(reader.selected(), 0);
        }
        assert_eq!(
            pointer.button(s, Some(&mut reader), NEXT, LEFT, Pressed),
            Route::Consumed
        );
        assert_eq!(
            pointer.feedback(s, Some(&mut replacement)),
            ReaderPointerFeedback::default()
        );
        assert_eq!(
            pointer.button(s, Some(&mut replacement), NEXT, LEFT, Released),
            Route::Consumed
        );
        assert_eq!(replacement.selected(), 0);
        assert_eq!(
            pointer.button(s, Some(&mut replacement), NEXT, LEFT, Pressed),
            Route::Consumed
        );
        (pointer, replacement)
    });
    let foreign = Fixture::keyboard();
    foreign.backend(|s| s.enable_pointer())?;
    let _foreign_app = mapped(&foreign);
    present(&foreign, &foreign.root(), (320, 180), (3, 4))?;
    foreign.backend(move |s| {
        assert_eq!(
            pointer.button(s, Some(&mut reader), NEXT, LEFT, Released),
            Route::Consumed
        );
        assert_eq!(
            pointer.button(s, Some(&mut reader), NEXT, LEFT, Pressed),
            Route::Forward
        );
        assert_eq!(reader.selected(), 0);
        assert_eq!(
            pointer.feedback(s, Some(&mut reader)),
            ReaderPointerFeedback::default()
        );
    });
    Ok(())
}

#[test]
fn native_reader_pointer_preserves_client_grabs_and_missing_seat_routing() -> Result {
    for seat in [false, true] {
        let f = Fixture::keyboard();
        if seat {
            f.backend(|s| s.enable_pointer())?;
        }
        let mut app = mapped(&f);
        let root = f.root();
        f.focus_surface(root.clone())?;
        present(&f, &root, (320, 180), (3, 4))?;
        let mut reader = begin(&f, Action::CloseWindow)?;
        if seat {
            f.backend(|s| s.pointer_motion(10.0, 10.0, 1))?;
            assert!(f.backend(|s| s.pointer_button(LEFT, Pressed, 2))?);
        }
        f.backend(move |s| {
            let mut pointer = WindowControlReaderPointer::default();
            assert!(!pointer.motion(s, Some(&mut reader), NEXT));
            for button in [0, LEFT, RIGHT, 0x118, u32::MAX] {
                assert_eq!(
                    pointer.button(s, Some(&mut reader), NEXT, button, Pressed),
                    Route::Forward
                );
                assert_eq!(
                    pointer.button(s, Some(&mut reader), NEXT, button, Released),
                    Route::Forward
                );
            }
            assert_eq!(
                pointer.feedback(s, Some(&mut reader)),
                ReaderPointerFeedback::default()
            );
            assert_eq!(reader.selected(), 0);
        });
        if seat {
            assert!(f.backend(|s| s.pointer_button(LEFT, Released, 3))?);
        }
        assert!(f.key(30, KeyState::Pressed)?);
        assert!(f.key(30, KeyState::Released)?);
        app.sync();
        assert_eq!(app.events.pointer.buttons.len(), if seat { 2 } else { 0 });
        assert_eq!(app.events.keyboard.keys.len(), 2);
        assert_eq!(app.events.keyboard.leaves, 0);
        assert_eq!(app.events.close_requests, 0);
    }
    Ok(())
}

#[test]
fn native_reader_pointer_host_routing_excludes_covered_input_and_drains_inactive_buttons() -> Result
{
    use smithay::{
        backend::input::{Axis, AxisSource},
        input::pointer::AxisFrame,
    };
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    present(&f, &f.root(), (320, 180), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(|s| s.pointer_motion(10.0, 10.0, 1))?;
    app.sync();
    let initial_motion = app.events.pointer.motion.len();
    f.backend(move |s| {
        let mut pointer = WindowControlReaderPointer::default();
        // The trusted host uses the component's route to withhold covered events.
        for hit in [Some(Hit::Content), NEXT] {
            let covered = pointer.motion(s, Some(&mut reader), hit);
            assert!(covered);
            if !covered {
                assert!(s.pointer_motion(20.0, 20.0, 2).is_ok());
                assert!(
                    s.pointer_axis(
                        AxisFrame::new(2)
                            .source(AxisSource::Wheel)
                            .value(Axis::Vertical, 3.0)
                    )
                    .is_ok()
                );
            }
        }
        for button in LEFT..=0x117 {
            let route = pointer.button(s, Some(&mut reader), NEXT, button, Pressed);
            assert_eq!(route, Route::Consumed);
            if route == Route::Forward {
                assert!(s.pointer_button(button, Pressed, 3).is_ok());
            }
        }
        pointer.cancel();
        reader.dismiss();
        assert_eq!(pointer.feedback(s, None), ReaderPointerFeedback::default());
        assert!(pointer.motion(s, None, None));
        for button in (LEFT..=0x117).rev() {
            // Inactive repeats remain owned, and cannot recreate execution authority.
            assert_eq!(
                pointer.button(s, None, None, button, Pressed),
                Route::Consumed
            );
            let route = pointer.button(s, None, None, button, Released);
            assert_eq!(route, Route::Consumed);
            if route == Route::Forward {
                assert!(s.pointer_button(button, Released, 4).is_ok());
            }
        }
        assert!(!pointer.motion(s, None, None));
        // Ordinary routing resumes exactly once after the last owned release.
        assert!(s.pointer_motion(12.0, 12.0, 5).is_ok());
        assert!(
            s.pointer_axis(
                AxisFrame::new(5)
                    .source(AxisSource::Wheel)
                    .value(Axis::Vertical, 4.0)
            )
            .is_ok()
        );
        for state in [Pressed, Released] {
            assert_eq!(pointer.button(s, None, None, LEFT, state), Route::Forward);
            assert_eq!(s.pointer_button(LEFT, state, 6).ok(), Some(true));
        }
        assert_eq!(reader.selected(), 0);
    });
    app.sync();
    assert_eq!(app.events.pointer.motion.len(), initial_motion + 1);
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert_eq!(app.events.pointer.axes.len(), 1);
    assert_eq!(
        app.events.pointer.axes.first().map(|axis| axis.1),
        Some(4.0)
    );
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
