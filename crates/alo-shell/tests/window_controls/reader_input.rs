//! Ordered backend input with real clients and submitted reader publications.
use super::{
    mapped,
    presentation::present,
    reader::begin,
    reader_frame::{Target, draw},
};
use crate::Fixture;
use alo_shell::{
    ReaderKeyCommand as Command, ReaderKeyRoute as Route, WindowControlPointerEvent as Pointer,
    WindowControlReaderInput,
};
use alo_shortcuts::Action;
use smithay::backend::input::{
    ButtonState,
    KeyState::{Pressed, Released},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn reader_input_failed_frame_cancels_keys_and_pointer_grabs_refuse() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = WindowControlReaderInput::default();
        let mut target = Target::default();
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        s.pointer_motion(10.0, 10.0, 90)?;
        assert!(s.pointer_button(0x110, ButtonState::Pressed, 91)?);
        let position = Some((259.5, 104.0));
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                position,
                Pointer::Button(0x110, ButtonState::Pressed)
            ),
            Route::Forward
        );
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                position,
                Pointer::Button(0x110, ButtonState::Released)
            ),
            Route::Forward
        );
        assert!(s.pointer_button(0x110, ButtonState::Released, 92)?);
        // Querying during a client grab permanently invalidates this reader.
        // A new published reader is required for the independent frame refusal.
        assert!(s.read_window_control_page(&mut reader, 0).is_none());
        Ok(())
    })?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = WindowControlReaderInput::default();
        let mut target = Target::default();
        let position = Some((259.5, 104.0));
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Consumed
        );
        assert!(draw(s, &mut target, &mut reader, input.frame_pointer(), 300).is_err());
        assert_eq!(
            input.key(s, Some(&mut reader), true, 106, Released, None),
            Route::Consumed
        );
        assert_eq!(reader.selected(), 0);
        assert!(!input.synchronize(s, Some(&mut reader), true));
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                position,
                Pointer::Button(0x110, ButtonState::Pressed)
            ),
            Route::Forward
        );
        assert!(s.read_window_control_page(&mut reader, 0).is_none());
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn reader_input_requires_publication_and_routes_navigation_once() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = WindowControlReaderInput::default();
        let mut target = Target::default();
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Forward
        );
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Consumed
        );
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Consumed
        );
        assert_eq!(
            input.key(s, Some(&mut reader), true, 106, Released, None),
            Route::Changed
        );
        assert_eq!(reader.selected(), 1);
        assert_eq!(
            input.key(s, Some(&mut reader), true, 106, Released, None),
            Route::Forward
        );
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Forward
        );
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        assert_eq!(
            input.key(s, Some(&mut reader), true, 30, Pressed, None),
            Route::Forward
        );
        assert!(s.keyboard_key(30, Pressed, 100)?);
        assert_eq!(
            input.key(s, Some(&mut reader), true, 30, Released, None),
            Route::Forward
        );
        assert!(s.keyboard_key(30, Released, 101)?);
        // An actual published hit navigates back; caller supplies no semantic hit.
        let position = Some((259.5, 75.0));
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                position,
                Pointer::Button(0x110, ButtonState::Pressed)
            ),
            Route::Consumed
        );
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                position,
                Pointer::Button(0x110, ButtonState::Released)
            ),
            Route::Changed
        );
        assert_eq!(reader.selected(), 0);
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.keyboard.leaves, 0);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}

#[test]
fn reader_input_changed_geometry_loss_and_removal_drain_without_execution() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let _app = mapped(&f);
    let root = f.root();
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = WindowControlReaderInput::default();
        let mut target = Target::default();
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Consumed
        );
        // No event observes the intermediate geometry. Returning to the original
        // rectangles must still invalidate the publication captured at press.
        draw(s, &mut target, &mut reader, input.frame_pointer(), 259)?;
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        assert_eq!(
            input.key(s, Some(&mut reader), true, 106, Released, None),
            Route::Consumed
        );
        assert_eq!(reader.selected(), 0);
        let position = Some((259.5, 104.0));
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                position,
                Pointer::Button(0x110, ButtonState::Pressed)
            ),
            Route::Consumed
        );
        assert!(!input.synchronize(s, Some(&mut reader), false));
        assert_eq!(
            input.pointer(
                s,
                None,
                false,
                None,
                Pointer::Button(0x110, ButtonState::Released)
            ),
            Route::Consumed
        );
        assert_eq!(reader.selected(), 0);
        assert!(!input.synchronize(s, Some(&mut reader), true));
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Consumed
        );
        s.render(&mut target, 110)?;
        assert_eq!(
            input.key(s, None, false, 106, Released, None),
            Route::Consumed
        );
        assert_eq!(
            input.key(s, None, false, 106, Released, None),
            Route::Forward
        );
        assert!(!input.synchronize(s, Some(&mut reader), true));
        assert_eq!(reader.selected(), 0);
        Ok(())
    })
}

#[test]
fn reader_input_competing_devices_cancel_and_client_owned_keys_refuse() -> Result {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root.clone())?;
    present(&f, &root, (640, 480), (3, 4))?;
    let mut reader = begin(&f, Action::CloseWindow)?;
    f.backend(move |s| -> Result {
        let mut input = WindowControlReaderInput::default();
        let mut target = Target::default();
        draw(s, &mut target, &mut reader, input.frame_pointer(), 260)?;
        assert!(s.keyboard_key(106, Pressed, 90)?);
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Forward
        );
        assert!(s.keyboard_key(106, Released, 91)?);
        let position = Some((259.5, 104.0));
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                position,
                Pointer::Button(0x110, ButtonState::Pressed)
            ),
            Route::Consumed
        );
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                106,
                Pressed,
                Some(Command::Next)
            ),
            Route::Consumed
        );
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                position,
                Pointer::Button(0x110, ButtonState::Released)
            ),
            Route::Consumed
        );
        // Competing pointer press outside the reader cancels the keyboard too.
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                Some((500.0, 470.0)),
                Pointer::Button(0x110, ButtonState::Pressed)
            ),
            Route::Forward
        );
        assert_eq!(
            input.key(s, Some(&mut reader), true, 106, Released, None),
            Route::Consumed
        );
        assert_eq!(reader.selected(), 0);
        assert_eq!(
            input.pointer(
                s,
                Some(&mut reader),
                true,
                Some((f64::NAN, 104.0)),
                Pointer::Motion
            ),
            Route::Forward
        );
        assert_eq!(
            input.pointer(s, Some(&mut reader), true, position, Pointer::Motion),
            Route::Consumed
        );
        assert_eq!(
            input.key(
                s,
                Some(&mut reader),
                true,
                1,
                Pressed,
                Some(Command::Dismiss)
            ),
            Route::Consumed
        );
        assert_eq!(
            input.key(s, Some(&mut reader), true, 1, Released, None),
            Route::Dismissed
        );
        assert!(s.read_window_control_page(&mut reader, 0).is_none());
        Ok(())
    })?;
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), 2);
    assert_eq!(app.events.close_requests, 0);
    Ok(())
}
