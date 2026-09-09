//! Trusted key transactions against real private Wayland client lifetimes.
use super::{mapped, presentation::present, reader::begin};
use crate::Fixture;
use alo_shell::{ReaderKeyCommand as Command, ReaderKeyRoute as Route, WindowControlReaderKeys};
use alo_shortcuts::Action;
use smithay::backend::input::KeyState::{Pressed, Released};

type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn native_reader_keys_execute_once_preserve_boundaries_and_ordinary_typing() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (320, 180), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| {
        let mut keys = WindowControlReaderKeys::default();
        assert_eq!(
            keys.route(s, Some(&mut reader), 30, Pressed, None),
            Route::Forward
        );
        assert_eq!(
            keys.route(s, Some(&mut reader), 30, Released, None),
            Route::Forward
        );
        assert_eq!(
            keys.route(s, Some(&mut reader), 900, Pressed, Some(Command::Next)),
            Route::Forward
        );
        for (command, expected, index) in [
            (Command::Previous, Route::Consumed, 0),
            (Command::Next, Route::Changed, 1),
            (Command::Next, Route::Changed, 2),
            (Command::Next, Route::Consumed, 2),
            (Command::Previous, Route::Changed, 1),
        ] {
            assert_eq!(
                keys.route(s, Some(&mut reader), 106, Pressed, Some(command)),
                Route::Consumed
            );
            assert_eq!(
                keys.route(s, Some(&mut reader), 106, Pressed, Some(Command::Dismiss)),
                Route::Consumed
            );
            assert_eq!(
                keys.route(s, Some(&mut reader), 106, Released, None),
                expected
            );
            assert_eq!(reader.selected(), index);
            assert_eq!(
                keys.route(s, Some(&mut reader), 106, Released, None),
                Route::Forward
            );
        }
        assert_eq!(
            keys.route(s, Some(&mut reader), 106, Pressed, Some(Command::Next)),
            Route::Consumed
        );
        assert_eq!(
            keys.route(s, Some(&mut reader), 1, Pressed, Some(Command::Dismiss)),
            Route::Consumed
        );
        assert_eq!(
            keys.route(s, Some(&mut reader), 106, Released, None),
            Route::Changed
        );
        assert_eq!(
            keys.route(s, Some(&mut reader), 1, Released, None),
            Route::Consumed
        );
        assert!(s.read_window_control_page(&mut reader, 2).is_some());
        assert_eq!(
            keys.route(s, Some(&mut reader), 1, Pressed, Some(Command::Dismiss)),
            Route::Consumed
        );
        assert_eq!(
            keys.route(s, Some(&mut reader), 1, Released, Some(Command::Next)),
            Route::Dismissed
        );
        assert!(s.read_window_control_page(&mut reader, 2).is_none());
    });
    assert!(f.key(30, Pressed)?);
    assert!(f.key(30, Released)?);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn native_reader_keys_cancel_and_drain_after_lifetime_or_page_changes() -> Result {
    for case in 0..6 {
        let f = Fixture::keyboard();
        let mut app = mapped(&f);
        let root = f.root();
        present(&f, &root, (320, 180), (3, 4))?;
        let mut reader = begin(&f, Action::CloseWindow)?;
        f.backend(move |s| {
            let mut keys = WindowControlReaderKeys::default();
            assert_eq!(
                keys.route(s, Some(&mut reader), 106, Pressed, Some(Command::Next)),
                Route::Consumed
            );
            match case {
                0 => keys.cancel(),
                1 => reader.dismiss(),
                2 => s.retire_window_controls(),
                3 => s.clear_input(),
                4 => {
                    assert!(s.read_window_control_page(&mut reader, 1).is_some());
                    assert!(s.read_window_control_page(&mut reader, 0).is_some());
                }
                _ => keys.cancel(),
            }
            assert_eq!(
                keys.route(s, Some(&mut reader), 106, Pressed, Some(Command::Next)),
                Route::Consumed
            );
            let active = if case == 5 { None } else { Some(&mut reader) };
            assert_eq!(keys.route(s, active, 106, Released, None), Route::Consumed);
            assert_eq!(reader.selected(), 0);
            assert_eq!(keys.route(s, None, 106, Released, None), Route::Forward);
        });
        app.sync();
        assert_eq!(app.events.close_requests, 0);
    }
    Ok(())
}

#[test]
fn native_reader_keys_refuse_replacement_and_foreign_readers() -> Result {
    let f = Fixture::keyboard();
    let _app = mapped(&f);
    present(&f, &f.root(), (320, 180), (3, 4))?;
    let mut original = begin(&f, Action::CloseWindow)?;
    let mut replacement = begin(&f, Action::CloseWindow)?;
    let mut keys = f.backend(move |s| {
        let mut keys = WindowControlReaderKeys::default();
        assert_eq!(
            keys.route(s, Some(&mut original), 106, Pressed, Some(Command::Next)),
            Route::Consumed
        );
        assert_eq!(
            keys.route(s, Some(&mut replacement), 106, Released, None),
            Route::Consumed
        );
        assert_eq!(replacement.selected(), 0);
        assert!(s.read_window_control_page(&mut replacement, 0).is_some());
        assert_eq!(
            keys.route(s, Some(&mut replacement), 106, Pressed, Some(Command::Next)),
            Route::Consumed
        );
        (keys, replacement)
    });
    let foreign = Fixture::keyboard();
    let _foreign_app = mapped(&foreign);
    present(&foreign, &foreign.root(), (320, 180), (3, 4))?;
    foreign.backend(move |s| {
        assert_eq!(
            keys.0.route(s, Some(&mut keys.1), 106, Released, None),
            Route::Consumed
        );
        assert_eq!(keys.1.selected(), 0);
        assert!(s.read_window_control_page(&mut keys.1, 0).is_none());
        assert_eq!(
            keys.0
                .route(s, Some(&mut keys.1), 106, Pressed, Some(Command::Next)),
            Route::Forward
        );
    });
    Ok(())
}

#[test]
fn native_reader_keys_never_steal_a_client_press_or_acquire_without_a_seat() -> Result {
    for keyboard in [false, true] {
        let f = if keyboard {
            Fixture::keyboard()
        } else {
            Fixture::new()
        };
        let mut app = mapped(&f);
        let root = f.root();
        present(&f, &root, (320, 180), (3, 4))?;
        let mut reader = begin(&f, Action::CloseWindow)?;
        if keyboard {
            f.focus_surface(root)?;
            assert!(f.key(106, Pressed)?);
        }
        f.backend(move |s| {
            let mut keys = WindowControlReaderKeys::default();
            for code in [0, 106, 768, u32::MAX] {
                assert_eq!(
                    keys.route(s, Some(&mut reader), code, Pressed, Some(Command::Next)),
                    Route::Forward
                );
                assert_eq!(
                    keys.route(s, Some(&mut reader), code, Released, None),
                    Route::Forward
                );
            }
            assert_eq!(reader.selected(), 0);
        });
        if keyboard {
            assert!(f.key(106, Released)?);
            app.sync();
            assert_eq!(app.events.keyboard.keys.len(), 2);
            assert_eq!(app.events.keyboard.leaves, 0);
        }
        app.sync();
        assert_eq!(app.events.close_requests, 0);
    }
    Ok(())
}
