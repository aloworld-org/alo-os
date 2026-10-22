//! Presses carried out against a form, and against the rows when there is none.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_setting_up::a_provider_is_filled_in::WhichField;

use super::WhatFillingInDid;
use crate::a_key_fills_in_a_provider::FillingInKey;

/// A server bound in a directory of its own, with nothing open.
///
/// The same fixture `a_press_in_the_menu_tests` uses and for its reason:
/// `Server::bind` refuses a runtime directory that is not exactly 0700, and a
/// tempdir arrives at whatever the umask leaves.
fn a_server() -> (tempfile::TempDir, crate::Server) {
    use std::os::unix::fs::PermissionsExt as _;
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let mine = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = crate::Server::bind(directory.path(), &format!("filling-in-{mine}"))
        .expect("a bound socket");
    (directory, server)
}

/// Type a whole string into the field the keyboard is in.
fn types(server: &mut crate::Server, what: &str) {
    for letter in what.chars() {
        server.a_key_fills_in_the_provider(FillingInKey::Typed(letter));
    }
}

#[test]
fn no_form_is_open_until_somebody_asks_for_one() {
    let (_directory, mut server) = a_server();
    assert!(server.a_provider_is_being_filled_in().is_none());
    assert_eq!(server.the_field_being_filled_in(), None);
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Typed('x')),
        None,
        "None is the caller's signal to read the press as a row's instead -- which is what \
         keeps every other key in Settings working"
    );
}

#[test]
fn asking_for_one_opens_an_empty_form_at_the_first_field() {
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();

    let form = server
        .a_provider_is_being_filled_in()
        .expect("a form was opened");
    assert_eq!(form.what_is_in(WhichField::Name), "");
    assert_eq!(server.the_field_being_filled_in(), Some(WhichField::Name));
}

#[test]
fn what_is_typed_reaches_the_form() {
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    types(&mut server, "OpenAI");

    assert_eq!(
        server
            .a_provider_is_being_filled_in()
            .expect("a form")
            .what_is_in(WhichField::Name),
        "OpenAI"
    );
}

#[test]
fn a_press_that_changes_nothing_says_so() {
    // So a surface knows whether to draw again. Backspace on an empty field is
    // the ordinary case: a person holding it down after clearing one.
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::RubOut),
        Some(WhatFillingInDid::Nothing)
    );
    types(&mut server, "O");
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::RubOut),
        Some(WhatFillingInDid::ItChanged)
    );
}

#[test]
fn giving_up_closes_the_form_and_leaves_settings_open() {
    // The press a person uses to get out of a half-typed key. It must not take
    // the window with it.
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    types(&mut server, "OpenAI");

    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::GiveUp),
        Some(WhatFillingInDid::GaveUp)
    );
    assert!(server.a_provider_is_being_filled_in().is_none());
    // And the next press is a row's again, which is the control that the mode
    // really ended rather than the form merely emptying.
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Typed('x')),
        None
    );
}

#[test]
fn asking_to_save_a_half_filled_form_keeps_what_they_typed() {
    // **A form that cleared itself on a refusal would take away a
    // 164-character key because an address was missing a colon.**
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    types(&mut server, "OpenAI");

    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Save),
        Some(WhatFillingInDid::NotFinished)
    );
    assert_eq!(
        server
            .a_provider_is_being_filled_in()
            .expect("the form is still open")
            .what_is_in(WhichField::Name),
        "OpenAI",
        "what they typed is still there"
    );
}

#[test]
fn a_filled_in_form_asks_to_be_saved() {
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    types(&mut server, "OpenAI");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "https://api.openai.com");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "sk-not-a-real-key");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "gpt-4o-mini");

    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Save),
        Some(WhatFillingInDid::SaveIt),
        "the caller does the saving: two writes to the machine are not this crate's"
    );
    assert!(
        server.a_provider_is_being_filled_in().is_some(),
        "and the form is still open until the caller has saved it -- closing it here would \
         throw the key away if the keyring refused"
    );
}

#[test]
fn moving_between_fields_puts_the_next_letters_somewhere_else() {
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    types(&mut server, "OpenAI");
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::NextField),
        Some(WhatFillingInDid::ItChanged)
    );
    assert_eq!(
        server.the_field_being_filled_in(),
        Some(WhichField::Address)
    );
    types(&mut server, "https://api.openai.com");

    let form = server.a_provider_is_being_filled_in().expect("a form");
    assert_eq!(form.what_is_in(WhichField::Name), "OpenAI");
    assert_eq!(
        form.what_is_in(WhichField::Address),
        "https://api.openai.com"
    );
}

#[test]
fn a_paste_says_it_cannot_be_done_rather_than_nothing() {
    // **Said rather than silent**, so the difference between a press that
    // meant nothing and one that meant something this machine cannot do is
    // visible to a surface and to this test. Until the selection can be read,
    // a person cannot put a real API key in this form at all.
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Paste),
        Some(WhatFillingInDid::CannotPasteYet)
    );
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Nothing),
        Some(WhatFillingInDid::Nothing),
        "the control: a press that really means nothing answers differently"
    );
}

#[test]
fn asking_again_starts_a_fresh_form() {
    // There is no second form. Somebody who presses *add a provider* while one
    // is open means to start over.
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    types(&mut server, "OpenAI");
    server.a_provider_is_added();

    assert_eq!(
        server
            .a_provider_is_being_filled_in()
            .expect("a form")
            .what_is_in(WhichField::Name),
        ""
    );
    assert_eq!(server.the_field_being_filled_in(), Some(WhichField::Name));
}
