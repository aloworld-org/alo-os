//! That typing finds what a person meant, in the order they meant it, and that
//! an empty field finds nothing.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Strings;

use super::{
    TheirNames, WhatARowDoes, WhatEachIs, nothing_can_answer_a_question, what_the_bar_finds,
    where_these_came_from,
};

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
    let found = what_the_bar_finds("fire", &installed(), &TheirNames::new(), &WhatEachIs::new(), &strings);

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
    let found = what_the_bar_finds("textedit", &installed(), &TheirNames::new(), &WhatEachIs::new(), &strings);
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
    let found = what_the_bar_finds("calc", &installed(), &TheirNames::new(), &WhatEachIs::new(), &strings);
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
    assert!(what_the_bar_finds("", &installed(), &TheirNames::new(), &WhatEachIs::new(), &strings).is_empty());
    assert!(what_the_bar_finds("   ", &installed(), &TheirNames::new(), &WhatEachIs::new(), &strings).is_empty());
    // **The control**: the same fixture with something typed does find.
    assert!(
        !what_the_bar_finds("f", &installed(), &TheirNames::new(), &WhatEachIs::new(), &strings).is_empty(),
        "this fixture finds nothing for any query, so the two assertions above mean nothing"
    );
}

/// **Nothing found still says something.**
#[test]
fn a_query_that_finds_nothing_is_still_a_sentence() {
    let strings = english();
    let found = what_the_bar_finds("zzzz", &installed(), &TheirNames::new(), &WhatEachIs::new(), &strings);
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

/// **A machine with nothing chosen to answer says so, in the place the count
/// goes.**
///
/// The fault this is written for: until 2026-10-22 a person typed a question on
/// such a machine, pressed Enter, and met silence — the request was taken,
/// `a_task_for` answered `None`, and the `if let` that received it had no
/// `else`. They had no way to find out why, and pressing again did the same
/// thing.
#[test]
fn a_machine_with_nothing_chosen_to_answer_says_so() {
    let strings = english();
    let said = nothing_can_answer_a_question(&strings);
    assert!(
        said.starts_with("On this computer"),
        "the line has to be the same shape as the count it replaces, or it reads as a \
         different line appearing rather than this one changing: {said:?}"
    );
    assert!(
        said.contains("answer questions"),
        "the one fact the rows below cannot carry is missing: {said:?}"
    );
}

/// **It is not a key**, which is how a sentence that was never declared reaches
/// a person looking like prose.
///
/// `alo-strings` answers a key it does not hold with the key in guillemets. That
/// is the right behaviour and it is also why this assertion exists: the fault
/// would otherwise be invisible on a screen.
#[test]
fn the_line_is_a_sentence_and_not_an_undeclared_key() {
    let strings = english();
    let said = nothing_can_answer_a_question(&strings);
    assert!(
        !said.starts_with('\u{ab}'),
        "no sentence is declared for this line, so a person is shown its key: {said:?}"
    );
}

/// **The two lines are different**, so a person can tell which they are reading.
///
/// A sentence that came out the same as the count would be a line that changed
/// nothing, and the test above would still pass on it.
#[test]
fn the_two_lines_a_person_can_see_are_not_the_same_line() {
    let strings = english();
    assert_ne!(
        nothing_can_answer_a_question(&strings),
        where_these_came_from(0, &strings),
        "nothing chosen and nothing found are two different facts and must read as two"
    );
}

/// **The name an entry gives is matched and shown**, which is what makes
/// LibreOffice findable at all: its identifiers are `libreoffice26.2-writer`
/// and the like, whose last part after a dot is `2-writer`.
#[test]
fn an_application_is_found_by_the_name_its_entry_gives() {
    let strings = english();
    let installed = vec![
        "libreoffice26.2-writer".to_owned(),
        "libreoffice26.2-calc".to_owned(),
        "org.kde.dolphin".to_owned(),
    ];
    let called: TheirNames = [
        ("libreoffice26.2-writer", "LibreOffice Writer"),
        ("libreoffice26.2-calc", "LibreOffice Calc"),
        ("org.kde.dolphin", "Dolphin"),
    ]
    .into_iter()
    .map(|(identifier, name)| (identifier.to_owned(), name.to_owned()))
    .collect();

    let found = what_the_bar_finds("writer", &installed, &called, &WhatEachIs::new(), &strings);
    assert_eq!(found.len(), 1, "{found:?}");
    let writer = found.first().expect("one result");
    assert_eq!(writer.shown, "LibreOffice Writer  ·  Application");
    assert_eq!(
        writer.does,
        WhatARowDoes::OpensAnApplication("libreoffice26.2-writer".to_owned()),
        "what opens is still the identifier, never the name"
    );
    assert_eq!(
        what_the_bar_finds("libre", &installed, &called, &WhatEachIs::new(), &strings).len(),
        2
    );
    assert_eq!(
        what_the_bar_finds("dolph", &installed, &called, &WhatEachIs::new(), &strings)
            .first()
            .expect("one result")
            .shown,
        "Dolphin  ·  Application"
    );
    // **With nothing said about what they are, a kind finds nothing** — the
    // kinds come from the entries, never from a guess at the name.
    assert!(
        what_the_bar_finds("files", &installed, &called, &WhatEachIs::new(), &strings).is_empty()
    );
}

/// The names and kinds of four applications a fresh machine has, as their
/// entries give them.
fn four_with_their_kinds() -> (Vec<String>, TheirNames, WhatEachIs) {
    let installed = ["org.kde.dolphin", "org.mozilla.firefox", "libreoffice-calc", "filelight"]
        .map(str::to_owned)
        .to_vec();
    let called: TheirNames = [
        ("org.kde.dolphin", "Dolphin"),
        ("org.mozilla.firefox", "Firefox"),
        ("libreoffice-calc", "LibreOffice Calc"),
        ("filelight", "Filelight"),
    ]
    .into_iter()
    .map(|(identifier, name)| (identifier.to_owned(), name.to_owned()))
    .collect();
    let are: WhatEachIs = [
        (
            "org.kde.dolphin",
            &["File Manager", "Gestionnaire de fichiers", "fichiers", "FileManager"][..],
        ),
        ("org.mozilla.firefox", &["Web Browser", "Internet", "WebBrowser"][..]),
        ("libreoffice-calc", &["Spreadsheet", "Office", "Spreadsheet"][..]),
        ("filelight", &["Disk Usage Statistics", "Utility"][..]),
    ]
    .into_iter()
    .map(|(identifier, words)| {
        (
            identifier.to_owned(),
            words.iter().map(|it| (*it).to_owned()).collect(),
        )
    })
    .collect();
    (installed, called, are)
}

/// **The owner's three examples**, in the owner's words of 2026-10-31.
#[test]
fn an_application_is_found_by_what_it_is() {
    let (installed, called, are) = four_with_their_kinds();
    let opens = |typed: &str| -> Vec<String> {
        super::what_matches(typed, &installed, &called, &are)
            .into_iter()
            .cloned()
            .collect()
    };
    assert_eq!(opens("files"), ["org.kde.dolphin"], "a plural finds File Manager");
    assert_eq!(opens("browser"), ["org.mozilla.firefox"]);
    assert_eq!(opens("spreadsheet"), ["libreoffice-calc"]);
    assert_eq!(opens("file man"), ["org.kde.dolphin"], "the start of the phrase");
    assert!(
        opens("rowser").is_empty(),
        "the middle of a word of what it is finds nothing"
    );
}

/// **A translated keyword finds it**, as the reader handed it over in the
/// person's language.
#[test]
fn a_translated_keyword_finds_it() {
    let (installed, called, are) = four_with_their_kinds();
    assert_eq!(
        super::what_matches("fichiers", &installed, &called, &are),
        [&"org.kde.dolphin".to_owned()]
    );
    assert_eq!(
        super::what_matches("gestionnaire", &installed, &called, &are),
        [&"org.kde.dolphin".to_owned()]
    );
}

/// **A name match ranks above a kind match**: *file* is the start of
/// *Filelight*'s name and only of Dolphin's kind, so Filelight is first and
/// Enter opens it.
#[test]
fn a_match_by_name_ranks_above_a_match_by_what_it_is() {
    let (installed, called, are) = four_with_their_kinds();
    assert_eq!(
        super::what_matches("file", &installed, &called, &are),
        [&"filelight".to_owned(), &"org.kde.dolphin".to_owned()]
    );
    let strings = english();
    let rows = what_the_bar_finds("file", &installed, &called, &are, &strings);
    assert_eq!(
        rows.first().map(|row| row.does.clone()),
        Some(WhatARowDoes::OpensAnApplication("filelight".to_owned()))
    );
}
