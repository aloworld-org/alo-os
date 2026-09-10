//! Native F1 transactions against real clients and submitted reader geometry.
use super::{
    mapped,
    presentation::present,
    reader_frame::Target,
    reader_selection::{chrome, style, words},
};
use crate::Fixture;
use alo_shell::{
    NestedControlInput, NestedPointerEvent, NestedReaderSession, ReaderKeyRoute as Route,
    WindowControlLabels, WindowControlReaderFrame,
};
use alo_shortcuts::Action;
use smithay::backend::input::KeyState::{Pressed, Released};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn reader_opening_releases_once_then_submits_navigates_dismisses_and_types() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(|s| -> Result {
        let mut input = NestedControlInput::default();
        let mut slot = None;
        let mut labels = WindowControlLabels::new()?;
        let strings = words()?;
        let mut session = NestedReaderSession {
            reader: &mut slot,
            labels: &mut labels,
            strings: &strings,
            style: style(),
            chrome: chrome(),
        };
        assert!(s.focus_window_control(Some(Action::CloseWindow)));
        for _ in 0..3 {
            assert_eq!(
                input.reader_session_key(s, &mut session, true, (59, Pressed, 1))?,
                Route::Consumed
            );
            assert!(session.reader.is_none());
        }
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Released, 2))?,
            Route::Changed
        );
        let reader = session.reader.as_mut().ok_or("opened")?;
        assert!(!s.window_control_reader_presented(reader));
        s.render_window_control_reader(
            &mut Target::default(),
            reader,
            WindowControlReaderFrame {
                strings: session.strings,
                labels: session.labels,
                chrome: session.chrome,
                pointer: input.reader_pointer(),
            },
            3,
        )?;
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (109, Pressed, 4))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (109, Released, 5))?,
            Route::Changed
        );
        let reader = session.reader.as_mut().ok_or("retained")?;
        assert_eq!(reader.selected(), 1);
        s.render_window_control_reader(
            &mut Target::default(),
            reader,
            WindowControlReaderFrame {
                strings: session.strings,
                labels: session.labels,
                chrome: session.chrome,
                pointer: input.reader_pointer(),
            },
            6,
        )?;
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (1, Pressed, 7))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (1, Released, 8))?,
            Route::Dismissed
        );
        *session.reader = None;
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (30, Pressed, 9))?,
            Route::Forward
        );
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (30, Released, 10))?,
            Route::Forward
        );
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn reader_opening_cancels_focus_roundtrip_republication_competition_and_loss() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(move |s| -> Result {
        let mut input = NestedControlInput::default();
        let mut slot = None;
        let mut labels = WindowControlLabels::new()?;
        let strings = words()?;
        let mut session = NestedReaderSession {
            reader: &mut slot,
            labels: &mut labels,
            strings: &strings,
            style: style(),
            chrome: chrome(),
        };
        for scenario in 0..6 {
            s.present_window_controls(Some(alo_shell::PaintedWindowControls {
                surface: &root,
                viewport: (640, 480),
                origin: (3, 4),
            }))?;
            assert!(s.focus_window_control(Some(Action::CloseWindow)));
            assert_eq!(
                input.reader_session_key(s, &mut session, true, (59, Pressed, 1))?,
                Route::Consumed
            );
            match scenario {
                0 => {
                    s.focus_window_control(None);
                    assert!(s.focus_window_control(Some(Action::CloseWindow)));
                }
                1 => {
                    s.retire_window_controls();
                    s.present_window_controls(Some(alo_shell::PaintedWindowControls {
                        surface: &root,
                        viewport: (640, 480),
                        origin: (3, 4),
                    }))?;
                    assert!(s.focus_window_control(Some(Action::CloseWindow)));
                }
                2 => {
                    input.route_reader(
                        s,
                        None,
                        true,
                        Some(NestedPointerEvent::Motion {
                            x: 75.0,
                            y: 5.0,
                            time: 2,
                        }),
                    )?;
                }
                3 => {
                    input.reader_session_key(s, &mut session, true, (30, Pressed, 2))?;
                    input.reader_session_key(s, &mut session, true, (30, Released, 3))?;
                }
                4 => {
                    input.route_reader(s, None, false, None)?;
                }
                _ => input.cancel(s),
            }
            assert_eq!(
                input.reader_session_key(s, &mut session, true, (59, Pressed, 4))?,
                Route::Consumed
            );
            assert_eq!(
                input.reader_session_key(s, &mut session, true, (59, Released, 5))?,
                Route::Consumed
            );
            assert!(session.reader.is_none());
        }
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn reader_opening_preserves_unselected_f1_client_chords_and_disabled_names() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(|s| -> Result {
        let mut input = NestedControlInput::default();
        let mut slot = None;
        let mut labels = WindowControlLabels::new()?;
        let strings = words()?;
        let mut session = NestedReaderSession {
            reader: &mut slot,
            labels: &mut labels,
            strings: &strings,
            style: style(),
            chrome: chrome(),
        };
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Pressed, 1))?,
            Route::Forward
        );
        assert!(s.focus_window_control(Some(Action::CloseWindow)));
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Pressed, 2))?,
            Route::Forward
        );
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Released, 3))?,
            Route::Forward
        );
        for (code, state) in [(29, Pressed), (59, Pressed), (59, Released), (29, Released)] {
            assert_eq!(
                input.reader_session_key(s, &mut session, true, (code, state, 4))?,
                Route::Forward
            );
        }
        assert!(session.reader.is_none());
        assert!(s.focus_window_control(Some(Action::MaximiseWindow)));
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Pressed, 5))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Released, 6))?,
            Route::Changed
        );
        let page = s
            .read_window_control_page(session.reader.as_mut().ok_or("disabled reader")?, 0)
            .ok_or("page")?;
        assert_eq!(page.said, &Action::MaximiseWindow.said(&strings));
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 6);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn reader_opening_preparation_error_and_pump_mode_change_drain_releases() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    f.backend(move |s| -> Result {
        let mut input = NestedControlInput::default();
        let mut slot = None;
        let mut labels = WindowControlLabels::new()?;
        let strings = words()?;
        let mut session = NestedReaderSession {
            reader: &mut slot,
            labels: &mut labels,
            strings: &strings,
            style: style(),
            chrome: chrome(),
        };
        session.chrome.size = (1, 1);
        assert!(s.focus_window_control(Some(Action::CloseWindow)));
        assert!(
            input
                .reader_session_key(s, &mut session, true, (59, Pressed, 1))
                .is_err()
        );
        assert!(session.reader.is_none());
        assert_eq!(
            input.reader_key(s, None, false, (59, Released, 2))?,
            Route::Consumed
        );
        session.chrome = chrome();
        s.present_window_controls(Some(alo_shell::PaintedWindowControls {
            surface: &root,
            viewport: (640, 480),
            origin: (3, 4),
        }))?;
        assert!(s.focus_window_control(Some(Action::CloseWindow)));
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Pressed, 3))?,
            Route::Consumed
        );
        assert_eq!(
            input.reader_key(s, None, true, (59, Released, 4))?,
            Route::Consumed
        );
        assert!(session.reader.is_none());
        Ok(())
    })?;
    app.sync();
    assert!(app.events.keyboard.keys.is_empty());
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
