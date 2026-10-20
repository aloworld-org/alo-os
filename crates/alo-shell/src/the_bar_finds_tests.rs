//! That typing finds what a person meant, in the order they meant it, and that
//! an empty field finds nothing.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Strings;

use super::{WhatARowDoes, what_the_bar_finds, where_these_came_from};

/// What a fresh machine has, as `alo_software::what_this_machine_has` prints
/// it: one identifier per line, no names anywhere.
fn installed() -> Vec<String> {
    [
        "org.mozilla.firefox",
        "org.gnome.TextEditor",
        "org.blender.Blender",
        "com.github.firefoxpwa.manager",
        "org.gnome.Calculator",
    ]
    .map(str::to_owned)
    .to_vec()
}

/// The shell's own sentences, in English.
fn english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("the machine's vocabulary"))
}

/// **What a person types finds the application they meant, first.**
///
/// The order is the whole of this test: *fire* finds Firefox before it finds
/// the thing with *firefox* in the middle of its identifier, because a person
/// typing four letters means the obvious one.
#[test]
fn what_was_typed_finds_the_obvious_one_first() {
    let strings = english();
    let found = what_the_bar_finds("fire", &installed(), &strings);

    assert!(
        found.len() >= 2,
        "the premise: this query matches more than one of the fixture's applications"
    );
    assert_eq!(
        found.first().expect("a first result").does,
        WhatARowDoes::OpensAnApplication("org.mozilla.firefox".to_owned()),
        "the application whose own name begins with what was typed is not first: {found:?}"
    );
    assert!(
        found.iter().any(|one| one.does
            == WhatARowDoes::OpensAnApplication("com.github.firefoxpwa.manager".to_owned())),
        "a match further inside the identifier was dropped rather than ranked below: {found:?}"
    );
}

/// **A row says what kind of thing it is**, so an application and a document
/// can be told apart at a glance when the second kind arrives.
#[test]
fn a_row_names_the_application_and_what_it_is() {
    let strings = english();
    let found = what_the_bar_finds("textedit", &installed(), &strings);
    let only = found.first().expect("one result");
    assert_eq!(
        only.shown, "TextEditor  ·  Application",
        "a row does not read the way the design draws it"
    );
}

/// **An identifier is not prettified into a name this product invented.**
#[test]
fn an_application_is_read_as_the_machine_knows_it() {
    let strings = english();
    let found = what_the_bar_finds("calc", &installed(), &strings);
    assert!(
        found
            .first()
            .expect("one result")
            .shown
            .starts_with("Calculator"),
        "the identifier's own last part is not what a person reads"
    );
}

/// **An empty field finds nothing**, so the Bar does not flash a list of every
/// application under somebody's first keystroke.
///
/// Whitespace counts as empty: a person who pressed the chord and then the
/// space bar has typed nothing.
#[test]
fn nothing_typed_finds_nothing() {
    let strings = english();
    assert!(what_the_bar_finds("", &installed(), &strings).is_empty());
    assert!(what_the_bar_finds("   ", &installed(), &strings).is_empty());
    // **The control**: the same fixture with something typed does find.
    assert!(
        !what_the_bar_finds("f", &installed(), &strings).is_empty(),
        "this fixture finds nothing for any query, so the two assertions above mean nothing"
    );
}

/// **Nothing found still says something.**
#[test]
fn a_query_that_finds_nothing_is_still_a_sentence() {
    let strings = english();
    let found = what_the_bar_finds("zzzz", &installed(), &strings);
    assert!(found.is_empty(), "the premise: this query matches nothing");
    let said = where_these_came_from(found.len(), &strings);
    assert!(
        said.contains('0'),
        "a person who found nothing is shown no sentence at all: {said:?}"
    );
    assert!(
        said.starts_with("On this computer"),
        "the line under the field is not the sentence the design draws: {said:?}"
    );
}

/// **One result reads as one**, not as *1 applications*.
#[test]
fn one_result_is_said_in_the_singular() {
    let strings = english();
    assert_eq!(
        where_these_came_from(1, &strings),
        "On this computer  ·  1 application"
    );
    assert_eq!(
        where_these_came_from(3, &strings),
        "On this computer  ·  3 applications"
    );
}

/// **Enter on the Bar opens what it found, and asks alo only when it found
/// nothing.**
///
/// `docs/features.md` promises the Bar *works with no model at all*:
/// application names open applications. Sending `firefox` to a model would
/// answer a different question, and on a machine with no model it would answer
/// none at all.
///
/// # The control is the second half, not a second test
///
/// A person who typed something that is not an application still means the
/// agent, so both roads are asserted on the same session with the same key.
/// Without that, a change that sent **everything** to `asked_to_start` would
/// pass the first assertion.
#[test]
fn enter_opens_what_the_bar_found_and_asks_alo_when_it_found_nothing() {
    use std::os::unix::fs::PermissionsExt as _;

    use crate::Server;
    use crate::composer_keys::ComposerKey;

    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(runtime.path(), "enter-on-the-bar").expect("a display");
    server.what_this_machine_has = installed();
    // **The Bar has to be open**, which is what the chord does — and what the
    // field itself insists on: `TheComposer::pressed` refuses every key for a
    // field that does not have the keyboard.
    let shortcuts = alo_shortcuts::Shortcuts::shipped();
    let chord = shortcuts
        .chord_for(alo_shortcuts::Action::Launcher)
        .expect("the release ships a chord for the Bar");
    assert!(server.a_chord_opens_the_bar(&shortcuts, chord));

    assert_eq!(
        server.how_many_are_waiting_to_start(),
        0,
        "a session nobody has typed in is already starting something"
    );

    for letter in "firefox".chars() {
        server.a_key_reached_the_composer(ComposerKey::Letter(letter));
    }
    server.a_key_reached_the_composer(ComposerKey::Ask);

    assert_eq!(
        server.how_many_are_waiting_to_start(),
        1,
        "Enter on an application the machine has did not ask for it to start"
    );
    assert_eq!(
        server.composer().typed(),
        "",
        "the field kept what was typed after it was acted on, so the next Enter sends it twice"
    );

    // **Opening closes the Bar**, which is what every other system does: the
    // panel goes and the keyboard follows the application that was opened.
    assert!(
        !server.composer().has_the_keyboard(),
        "the Bar kept the keyboard after opening something, so a person typing at the \
         application they just opened types into the Bar instead"
    );

    // **The other road, on the same session**: something no application
    // matches still reaches the agent. The chord again, because the Bar closed.
    assert!(server.a_chord_opens_the_bar(&shortcuts, chord));
    for letter in "what is the capital of Spain".chars() {
        server.a_key_reached_the_composer(ComposerKey::Letter(letter));
    }
    let what = server.a_key_reached_the_composer(ComposerKey::Ask);
    assert!(
        matches!(what, crate::the_composer_a_person_types_in::WhatTheComposerDid::TheyAsked(ref asked)
            if asked.contains("capital")),
        "a question that matches no application was not asked of alo: {what:?}"
    );
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        1,
        "a question that matches no application was sent to be started as one"
    );
}

/// **Enter does nothing when the Bar does not have the keyboard**, even with
/// text left in the field.
///
/// `TheComposer::pressed` refuses every key for a field without the keyboard,
/// and `Server::a_key_reached_the_composer` acts on Enter **before** handing
/// the key to the field — so that guard has to be repeated there or it is not
/// there at all. This is the test that says so.
#[test]
fn enter_does_nothing_when_the_bar_does_not_have_the_keyboard() {
    use std::os::unix::fs::PermissionsExt as _;

    use crate::Server;
    use crate::composer_keys::ComposerKey;

    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(runtime.path(), "enter-with-no-keyboard").expect("a display");
    server.what_this_machine_has = installed();

    let shortcuts = alo_shortcuts::Shortcuts::shipped();
    let chord = shortcuts
        .chord_for(alo_shortcuts::Action::Launcher)
        .expect("the release ships a chord for the Bar");
    assert!(server.a_chord_opens_the_bar(&shortcuts, chord));
    for letter in "firefox".chars() {
        server.a_key_reached_the_composer(ComposerKey::Letter(letter));
    }
    assert_eq!(
        server.composer().typed(),
        "firefox",
        "the premise: the field holds what was typed"
    );

    // The keyboard goes somewhere else, as it would if a window were pressed.
    server.the_composer.it_gave_the_keyboard_up();
    server.a_key_reached_the_composer(ComposerKey::Ask);

    assert_eq!(
        server.how_many_are_waiting_to_start(),
        0,
        "Enter opened an application from a field nobody was looking at"
    );
    // **The control**: with the keyboard back, the same Enter does open it, so
    // the nought above is the guard and not a fixture that never worked.
    server.the_composer.it_took_the_keyboard();
    server.a_key_reached_the_composer(ComposerKey::Ask);
    assert_eq!(
        server.how_many_are_waiting_to_start(),
        1,
        "with the keyboard in the field the same Enter did nothing either, so this test has \
         been measuring a broken fixture"
    );
}
