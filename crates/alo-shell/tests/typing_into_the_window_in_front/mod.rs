//! **Typing into the window in front, as key presses, to that one client**
//! (ADR 0153), against real clients on a real display.
//!
//! What is asserted is what each client was **sent**, read off its own
//! connection: the layouts, the keys and the modifiers. So *nothing reaches
//! a window that is not in front* and *the other client is told nothing* are
//! about the wire, not about this crate's own bookkeeping.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or a short list is the failure being reported"
)]

use super::support::{Application, Fixture};
use alo_acting::{ToType, Typed};
use alo_applications::Application as Named;
use alo_shell::typing_into_the_window_in_front::MayTypeInto;
use wayland_client::protocol::wl_keyboard::KeyState;

/// The application the window in front belongs to.
const EDITOR: &str = "org.example.Editor";

/// Answers the same for every process.
struct Answers(bool);

impl MayTypeInto for Answers {
    fn may_type_into(&mut self, _: &str, _: u32) -> bool {
        self.0
    }
}

fn mapped(f: &Fixture, app_id: &str) -> Application {
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    app
}

fn text(into: &str, bytes: usize) -> ToType {
    ToType::Text {
        application: Named::identified(into).expect("an identifier"),
        bytes,
    }
}

fn typed(f: &Fixture, asked: ToType, what: &str, may: bool) -> Typed {
    let what = what.to_owned();
    f.backend(move |s| s.typed_into_the_window_in_front(&asked, &what, &mut Answers(may)))
}

/// The keys a client was sent, pressed and let go, after what it had before.
fn presses_after(app: &Application, before: usize) -> Vec<(u32, KeyState)> {
    app.events
        .keyboard
        .keys
        .get(before..)
        .unwrap_or_default()
        .to_vec()
}

#[test]
fn a_text_is_typed_into_the_window_in_front_and_its_layout_is_put_back() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f, EDITOR);
    f.focus(Some(0)).expect("the window takes the keyboard");
    app.sync();
    let (keys, layouts) = (
        app.events.keyboard.keys.len(),
        app.events.keyboard.keymaps.len(),
    );
    let theirs = app
        .events
        .keyboard
        .keymaps
        .last()
        .cloned()
        .expect("a layout");

    let said = "fn é\n\t日";
    assert_eq!(
        typed(&f, text(EDITOR, said.len()), said, true),
        Typed::Typed
    );
    app.sync();

    let sent = presses_after(&app, keys);
    // Every character pressed and let go, in order, nothing else.
    assert_eq!(sent.len(), said.chars().count() * 2);
    assert!(sent.chunks(2).all(|pair| pair[0].1 == KeyState::Pressed
        && pair[1].1 == KeyState::Released
        && pair[0].0 == pair[1].0));
    // One layout of alo's own, then the person's again — the same text.
    let received = app
        .events
        .keyboard
        .keymaps
        .get(layouts..)
        .unwrap_or_default();
    assert_eq!(received.len(), 2);
    assert!(received[0].contains("U65E5") || received[0].contains("kanji"));
    assert_eq!(received[1], theirs);
}

#[test]
fn nothing_is_sent_when_no_window_has_the_keyboard() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f, EDITOR);
    let (keys, layouts) = (
        app.events.keyboard.keys.len(),
        app.events.keyboard.keymaps.len(),
    );
    assert_eq!(typed(&f, text(EDITOR, 3), "abc", true), Typed::NotInFront);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), keys);
    assert_eq!(app.events.keyboard.keymaps.len(), layouts);
}

#[test]
fn nothing_is_sent_when_the_window_in_front_is_another_applications() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f, "org.example.Payroll");
    f.focus(Some(0)).expect("the window takes the keyboard");
    app.sync();
    let (keys, layouts) = (
        app.events.keyboard.keys.len(),
        app.events.keyboard.keymaps.len(),
    );
    assert_eq!(typed(&f, text(EDITOR, 3), "abc", true), Typed::NotInFront);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), keys);
    assert_eq!(app.events.keyboard.keymaps.len(), layouts);
}

#[test]
fn nothing_is_sent_to_a_window_that_may_not_be_typed_into() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f, EDITOR);
    f.focus(Some(0)).expect("the window takes the keyboard");
    app.sync();
    let (keys, layouts) = (
        app.events.keyboard.keys.len(),
        app.events.keyboard.keymaps.len(),
    );
    assert_eq!(typed(&f, text(EDITOR, 3), "abc", false), Typed::NotAlosCopy);
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), keys);
    assert_eq!(app.events.keyboard.keymaps.len(), layouts);
}

#[test]
fn a_control_character_stops_the_typing_before_its_first_key() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f, EDITOR);
    f.focus(Some(0)).expect("the window takes the keyboard");
    app.sync();
    let keys = app.events.keyboard.keys.len();
    assert_eq!(
        typed(&f, text(EDITOR, 4), "ab\u{7}c", true),
        Typed::CannotType('\u{7}')
    );
    app.sync();
    assert_eq!(app.events.keyboard.keys.len(), keys);
}

#[test]
fn a_display_with_no_keyboard_says_so() {
    let f = Fixture::new();
    let _app = mapped(&f, EDITOR);
    assert_eq!(typed(&f, text(EDITOR, 3), "abc", true), Typed::NoKeyboard);
}

#[test]
fn the_other_client_is_told_nothing() {
    let f = Fixture::keyboard();
    let mut other = mapped(&f, "org.example.Payroll");
    let mut app = mapped(&f, EDITOR);
    f.focus(Some(1)).expect("the editor takes the keyboard");
    app.sync();
    other.sync();
    let (keys, layouts) = (
        other.events.keyboard.keys.len(),
        other.events.keyboard.keymaps.len(),
    );
    assert_eq!(typed(&f, text(EDITOR, 2), "hi", true), Typed::Typed);
    app.sync();
    other.sync();
    assert_eq!(other.events.keyboard.keys.len(), keys);
    assert_eq!(other.events.keyboard.keymaps.len(), layouts);
    assert_eq!(app.events.keyboard.keys.len() % 2, 0);
}

#[test]
fn saving_holds_control_around_one_key() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f, EDITOR);
    f.focus(Some(0)).expect("the window takes the keyboard");
    app.sync();
    let (keys, mods) = (
        app.events.keyboard.keys.len(),
        app.events.keyboard.modifiers.len(),
    );
    let save = ToType::Save {
        application: Named::identified(EDITOR).expect("an identifier"),
    };
    assert_eq!(typed(&f, save, "", true), Typed::AskedToSave);
    app.sync();
    // Control down, the key down and up, Control up.
    assert_eq!(presses_after(&app, keys).len(), 4);
    let sent = app
        .events
        .keyboard
        .modifiers
        .get(mods..)
        .unwrap_or_default();
    assert!(sent.iter().any(|mask| *mask != 0), "{sent:?}");
    assert_eq!(sent.last(), Some(&0));
}
