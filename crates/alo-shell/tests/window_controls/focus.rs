//! Real clients exercise native traversal without acquiring application keys.
use super::{
    mapped,
    presentation::present,
    reader_selection::{chrome, style, words},
};
use crate::Fixture;
use alo_shell::{
    NestedControlInput, NestedReaderSession, ReaderKeyRoute, WindowControlFocus as Focus,
    WindowControlLabels,
};
use alo_shortcuts::Action;
use smithay::backend::input::{
    ButtonState,
    KeyState::{Pressed, Released},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn navigate(f: &Fixture, direction: Focus) -> Option<Action> {
    f.backend(move |s| s.navigate_window_control(direction))
}

#[test]
fn control_focus_traverses_disabled_names_wraps_and_preserves_client_typing() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    app.sync();
    let sizes = app.events.sizes.clone();
    present(&f, &root, (120, 48), (3, 4))?;
    for (direction, expected) in [
        (Focus::Next, Action::MinimiseWindow),
        (Focus::Next, Action::MaximiseWindow),
        (Focus::Next, Action::CloseWindow),
        (Focus::Next, Action::MinimiseWindow),
        (Focus::Previous, Action::CloseWindow),
        (Focus::Previous, Action::MaximiseWindow),
        (Focus::First, Action::MinimiseWindow),
        (Focus::Last, Action::CloseWindow),
    ] {
        assert_eq!(navigate(&f, direction), Some(expected));
        let label = f
            .backend(|s| s.presented_window_control_label(None, (100, 40)))?
            .ok_or("selected label")?;
        assert_eq!(label.control.action(), expected);
        if expected == Action::MaximiseWindow {
            assert!(!label.control.enabled());
        }
        present(&f, &root, (120, 48), (3, 4))?;
    }
    assert!(f.key(30, Pressed)?);
    assert!(f.key(30, Released)?);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(app.events.sizes, sizes);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn control_focus_skips_full_clipping_keeps_partial_and_handles_empty_strip() -> Result {
    let f = Fixture::new();
    let _app = mapped(&f);
    let root = f.root();
    assert_eq!(navigate(&f, Focus::Next), None);
    present(&f, &root, (50, 48), (3, 4))?;
    assert_eq!(navigate(&f, Focus::Previous), Some(Action::MaximiseWindow));
    assert_eq!(navigate(&f, Focus::Next), Some(Action::MinimiseWindow));
    assert_eq!(navigate(&f, Focus::Last), Some(Action::MaximiseWindow));
    present(&f, &root, (50, 48), (-70, 4))?;
    for direction in [Focus::Next, Focus::Previous, Focus::First, Focus::Last] {
        assert_eq!(navigate(&f, direction), Some(Action::CloseWindow));
    }
    present(&f, &root, (50, 48), (3, 48))?;
    for direction in [Focus::Next, Focus::Previous, Focus::First, Focus::Last] {
        assert_eq!(navigate(&f, direction), None);
        assert!(
            f.backend(|s| s.presented_window_control_label(None, (100, 40)))?
                .is_none()
        );
    }
    Ok(())
}

#[test]
fn control_focus_refuses_competition_and_stale_mapping_without_resurrection() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    present(&f, &root, (120, 48), (3, 4))?;
    assert_eq!(navigate(&f, Focus::Last), Some(Action::CloseWindow));
    f.backend(|s| s.pointer_motion(2.0, 2.0, 1))?;
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
    assert_eq!(navigate(&f, Focus::Next), None);
    assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 3))?);
    assert_eq!(navigate(&f, Focus::Next), Some(Action::MinimiseWindow));
    let target = root.clone();
    assert!(f.backend(move |s| s.press_window_control(&target, (120, 48), (3, 4), (76.0, 5.0)))?);
    assert_eq!(navigate(&f, Focus::Last), None);
    f.backend(|s| s.cancel_window_control());
    assert_eq!(navigate(&f, Focus::Next), None); // Cancelled release is still owned.
    f.backend(|s| s.release_window_control((120, 48), (3, 4), (76.0, 5.0)))?;
    assert_eq!(navigate(&f, Focus::Last), Some(Action::CloseWindow));
    let target = root.clone();
    f.backend(move |s| {
        s.set_window_minimized(&target, true)?;
        s.set_window_minimized(&target, false)
    })?;
    assert_eq!(navigate(&f, Focus::Next), None);
    assert_eq!(navigate(&f, Focus::Previous), None);
    present(&f, &root, (120, 48), (3, 4))?;
    assert_eq!(navigate(&f, Focus::Next), Some(Action::MinimiseWindow));
    app.surface.attach(None, 0, 0);
    app.surface.commit();
    app.sync();
    app.configure();
    app.attach();
    app.sync();
    assert_eq!(navigate(&f, Focus::Next), None);
    present(&f, &root, (120, 48), (3, 4))?;
    assert_eq!(navigate(&f, Focus::Previous), Some(Action::CloseWindow));
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    drop(app);
    f.wait_for((0, 0));
    assert_eq!(navigate(&f, Focus::First), None);
    Ok(())
}

#[test]
fn control_focus_traversal_opens_selected_name_and_cancels_pending_single_item_wrap() -> Result {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (-70, 4))?;
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
            s.navigate_window_control(Focus::Next),
            Some(Action::CloseWindow)
        );
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Pressed, 1))?,
            ReaderKeyRoute::Consumed
        );
        // Traversal back to the very same action is a new explicit selection.
        assert_eq!(
            s.navigate_window_control(Focus::Next),
            Some(Action::CloseWindow)
        );
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Released, 2))?,
            ReaderKeyRoute::Consumed
        );
        assert!(session.reader.is_none());
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Pressed, 3))?,
            ReaderKeyRoute::Consumed
        );
        assert_eq!(
            input.reader_session_key(s, &mut session, true, (59, Released, 4))?,
            ReaderKeyRoute::Changed
        );
        let reader = session
            .reader
            .as_mut()
            .ok_or("reader opened after traversal")?;
        let page = s.read_window_control_page(reader, 0).ok_or("live page")?;
        assert_eq!(page.total, 3);
        Ok(())
    })?;
    app.sync();
    assert!(app.events.keyboard.keys.is_empty());
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
