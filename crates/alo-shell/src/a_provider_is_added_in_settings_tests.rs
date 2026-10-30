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
fn a_paste_asks_for_the_selection_and_does_not_pretend_it_arrived() {
    // **Not ItChanged.** Nothing is in the field yet and may never be: the
    // selection belongs to another client, which writes when it gets round to
    // it and may not write at all. A surface that drew on this press would
    // draw an unchanged field and call it a paste.
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Paste),
        Some(WhatFillingInDid::AskedForThePaste)
    );
    assert_eq!(
        server
            .a_provider_is_being_filled_in()
            .expect("a form")
            .what_is_in(WhichField::Name),
        "",
        "and the field is untouched until something comes back"
    );
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Nothing),
        Some(WhatFillingInDid::Nothing),
        "the control: a press that really means nothing answers differently"
    );
}

#[test]
fn nothing_arriving_changes_nothing_and_says_so() {
    // **What a frame loop is told almost every time it asks**, sixty times a
    // second. A surface that redrew on this would redraw on every frame.
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    assert!(!server.that_paste_arrived());
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

#[test]
fn what_a_client_wrote_lands_in_the_field_that_has_the_keyboard() {
    // **The claim the whole piece exists for**: a person copies a key
    // somewhere else, presses Control+V, and it is in the field. Driven
    // through the clipboard directly rather than through a seat, because a
    // seat needs a real client to own a selection -- what is under test here
    // is that what comes back reaches the right field.
    use std::io::Write as _;

    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    assert_eq!(server.the_field_being_filled_in(), Some(WhichField::Key));

    let (reading, writing) = std::io::pipe().expect("a pipe");
    server
        .clipboard
        .reading(std::os::unix::io::OwnedFd::from(reading));
    let mut writing = writing;
    writing
        .write_all(b"sk-proj-a-very-long-key-nobody-would-type\n")
        .expect("the client writes");
    drop(writing);

    // The client writes when it gets round to it, so this waits rather than
    // asserting on the next line -- which is the whole reason the read is on a
    // thread and not in the frame loop.
    let until = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while !server.that_paste_arrived() && std::time::Instant::now() < until {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let form = server.a_provider_is_being_filled_in().expect("a form");
    assert_eq!(
        form.what_is_in(WhichField::Key),
        "sk-proj-a-very-long-key-nobody-would-type",
        "it landed in the key field, with the newline a web page put on the end gone"
    );
    // The controls at the other end: the two fields they tabbed past are still
    // empty, so what arrived went where the keyboard was and not everywhere.
    assert_eq!(form.what_is_in(WhichField::Name), "");
    assert_eq!(form.what_is_in(WhichField::Address), "");
}

#[test]
fn a_paste_that_arrives_after_the_form_closed_is_dropped() {
    // A person asked, gave up, and the client answered afterwards. Keeping it
    // for a later form would put a key they had abandoned into the next one
    // they opened.
    use std::io::Write as _;

    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    let (reading, writing) = std::io::pipe().expect("a pipe");
    server
        .clipboard
        .reading(std::os::unix::io::OwnedFd::from(reading));
    let mut writing = writing;
    writing.write_all(b"sk-abandoned").expect("written");
    drop(writing);

    server.a_key_fills_in_the_provider(FillingInKey::GiveUp);
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert!(!server.that_paste_arrived());

    // And the next form a person opens is empty.
    server.a_provider_is_added();
    assert_eq!(
        server
            .a_provider_is_being_filled_in()
            .expect("a form")
            .what_is_in(WhichField::Name),
        ""
    );
}

/// Fill the form in the way a person adding OpenAI would.
fn filled_in(server: &mut crate::Server) {
    server.a_provider_is_added();
    types(server, "OpenAI");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(server, "https://api.openai.com");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(server, "sk-not-a-real-key");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(server, "gpt-4o-mini");
}

#[test]
fn pressing_enter_puts_the_provider_aside_for_the_loop_to_save() {
    let (_directory, mut server) = a_server();
    filled_in(&mut server);

    assert!(
        server.take_the_provider_to_save().is_none(),
        "the control: nothing is waiting until they press Enter"
    );
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Save),
        Some(WhatFillingInDid::SaveIt)
    );

    let saving = server
        .take_the_provider_to_save()
        .expect("a provider is waiting to be saved");
    assert_eq!(saving.provider.name, "OpenAI");
    assert_eq!(saving.model, "gpt-4o-mini");
    assert!(
        server.take_the_provider_to_save().is_none(),
        "taken rather than read, so one Enter saves once however many frames pass"
    );
}

#[test]
fn the_form_stays_open_until_the_caller_says_it_was_saved() {
    // **The moment a person most needs their key to still be there.** A
    // keyring that is locked, or a settings file that will not read, both end
    // here -- and they cannot see the key to type it again.
    let (_directory, mut server) = a_server();
    filled_in(&mut server);
    server.a_key_fills_in_the_provider(FillingInKey::Save);
    let _ = server.take_the_provider_to_save();

    assert_eq!(
        server
            .a_provider_is_being_filled_in()
            .expect("the form is still open")
            .what_is_in(WhichField::Key),
        "sk-not-a-real-key",
        "what they pasted is still on the screen"
    );

    server.that_provider_was_saved();
    assert!(
        server.a_provider_is_being_filled_in().is_none(),
        "and only then does the form go"
    );
}

#[test]
fn an_address_that_is_not_one_names_the_field_rather_than_the_form() {
    // Every field is filled and one is wrong, which is a different thing to
    // say to a person than "you have not finished".
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    types(&mut server, "OpenAI");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "api.openai.com");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "sk-not-a-real-key");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "gpt-4o-mini");

    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Save),
        Some(WhatFillingInDid::NotRight(WhichField::Address))
    );
    assert!(
        server.take_the_provider_to_save().is_none(),
        "and nothing was put aside to save"
    );
}

#[test]
fn an_empty_form_is_not_finished_rather_than_wrong() {
    // **The control for the test above.** A surface given one answer for both
    // would tell somebody staring at an empty form that their address is
    // wrong, which is a sentence about a field they have not reached.
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Save),
        Some(WhatFillingInDid::NotFinished)
    );
}

#[test]
fn giving_up_takes_away_what_was_waiting_to_be_saved() {
    // A person who gave up has not asked for it to be written anyway, and a
    // frame loop that found it afterwards would save a provider they had just
    // abandoned.
    let (_directory, mut server) = a_server();
    filled_in(&mut server);
    server.a_key_fills_in_the_provider(FillingInKey::Save);
    server.a_key_fills_in_the_provider(FillingInKey::GiveUp);

    assert!(server.take_the_provider_to_save().is_none());
    assert!(server.a_provider_is_being_filled_in().is_none());
}

#[test]
fn a_service_at_this_machines_address_is_still_saved_when_they_go_ahead() {
    // **The caution takes nothing away** (law 5, owner's decision of
    // 2026-10-29): Settings says alo cannot see where a local service sends a
    // question, and Enter saves it exactly as it saves anything else.
    let (_directory, mut server) = a_server();
    server.a_provider_is_added();
    types(&mut server, "On this machine");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "http://127.0.0.1:11434");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "none");
    server.a_key_fills_in_the_provider(FillingInKey::NextField);
    types(&mut server, "llama3");
    assert!(
        server
            .a_provider_is_being_filled_in()
            .expect("a form")
            .its_address_is_this_machine(),
        "the control: this is the address the caution is shown for"
    );

    assert_eq!(
        server.a_key_fills_in_the_provider(FillingInKey::Save),
        Some(WhatFillingInDid::SaveIt)
    );
    let saving = server
        .take_the_provider_to_save()
        .expect("a provider is waiting to be saved");
    assert_eq!(saving.provider.endpoint, "http://127.0.0.1:11434");
}
