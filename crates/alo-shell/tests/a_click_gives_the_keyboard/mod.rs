//! **A click gives the keyboard to the window clicked**, verified on real
//! Wayland client connections.
//!
//! Ordinary focus, settled by the owner on 2026-10-26. Until then a press was
//! delivered to an application and the keyboard stayed wherever it was — so a
//! person who had typed into the Bar and then clicked into their editor went
//! on typing into the Bar. The design already assumed otherwise: the Dock's
//! note says what is announced *after a click — the window that took focus*.
//!
//! Each test carries its control: the application had **no** keyboard before
//! the click, so the click is what gave it one.

use super::support::{Application, Fixture};
use alo_shell::InputError;
use alo_shortcuts::Shortcuts;
use smithay::backend::input::{ButtonState, KeyState};

/// `KEY_LEFTMETA` and `KEY_A`: the shipped chord that opens the Bar.
const SUPER: u32 = 125;
const LETTER_A: u32 = 30;
/// `BTN_LEFT`.
const LEFT: u32 = 0x110;

/// A desktop with a pointer and one real application, mapped and unfocused.
fn a_desktop_with_an_application() -> Result<(Fixture, Application), InputError> {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())?;
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();
    Ok((f, app))
}

/// One click inside the application.
fn click_on_it(f: &Fixture, time: u32) -> Result<(), InputError> {
    f.backend(move |s| s.pointer_motion(4.0, 4.0, time))?;
    f.backend(move |s| s.pointer_button(LEFT, ButtonState::Pressed, time + 1))?;
    f.backend(move |s| s.pointer_button(LEFT, ButtonState::Released, time + 2))?;
    Ok(())
}

/// **A click on an application gives it the keyboard.**
#[test]
fn a_click_on_an_application_gives_it_the_keyboard() -> Result<(), InputError> {
    let (f, mut app) = a_desktop_with_an_application()?;
    app.sync();
    assert!(
        app.events.keyboard.enters.is_empty(),
        "the control: the application had the keyboard before anybody clicked"
    );

    click_on_it(&f, 10)?;
    app.sync();

    assert_eq!(
        app.events.keyboard.enters.len(),
        1,
        "a click on the application did not give it the keyboard"
    );
    Ok(())
}

/// **A click on an application takes the keyboard back from the Bar**, which
/// is the gap that was asked to be closed: typing went on reaching the Bar.
#[test]
fn a_click_on_an_application_takes_the_keyboard_from_the_bar() -> Result<(), InputError> {
    let (f, mut app) = a_desktop_with_an_application()?;
    f.backend(|s| s.the_shortcuts_are(Shortcuts::shipped()));
    for (code, state) in [
        (SUPER, KeyState::Pressed),
        (LETTER_A, KeyState::Pressed),
        (LETTER_A, KeyState::Released),
        (SUPER, KeyState::Released),
    ] {
        f.backend(move |s| s.keyboard_key(code, state, 5))?;
    }
    assert!(
        f.backend(|s| s.the_bar_is_taking_the_keys()),
        "the control: Super+A did not give the Bar the keyboard"
    );

    click_on_it(&f, 10)?;
    app.sync();

    assert!(
        !f.backend(|s| s.the_bar_is_taking_the_keys()),
        "the Bar kept the keyboard after a click on an application"
    );
    assert_eq!(app.events.keyboard.enters.len(), 1);
    Ok(())
}

/// **A second click does not tell the application it lost the keyboard.**
/// Activating a window that already has it would send a leave and an enter for
/// nothing, and an application may act on either.
#[test]
fn a_second_click_does_not_take_the_keyboard_away_and_give_it_back() -> Result<(), InputError> {
    let (f, mut app) = a_desktop_with_an_application()?;

    click_on_it(&f, 10)?;
    click_on_it(&f, 20)?;
    app.sync();

    assert_eq!(app.events.keyboard.enters.len(), 1, "focused twice");
    assert_eq!(app.events.keyboard.leaves, 0, "told it lost the keyboard");
    Ok(())
}
