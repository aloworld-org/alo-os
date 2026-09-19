//! What was open is a list of applications and where each was, kept in the
//! person's folder, restored only if they asked — and it carries nothing about
//! what was in them.
//!
//! Two halves, and both are needed. The first walks a real file on a real disk:
//! a person turns the setting on, logs out, signs in, and finds their
//! applications and places back. The second **reads this crate's own shipped
//! source** for every word a leak would arrive as — a title, a document, a URL,
//! a window's identifier — because a list that carries none of them today is one
//! sentence in a header away from carrying one tomorrow, and a header is not a
//! check.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_appearance::DisplayId;
use alo_dividing::Place;
use alo_leaving::keeping::{self, THE_FILE};
use alo_leaving::restoring::{self, AtSignIn};
use alo_leaving::{Changes, MayEnd, Open, Restoring, Settings, Split, WasOpen};

/// A folder under the temporary directory that is this test's alone, emptied.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-leaving-kept-{what}"));
    if folder.exists() {
        fs::remove_dir_all(&folder).unwrap();
    }
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// The laptop's own screen.
fn the_laptop() -> DisplayId {
    DisplayId::named("eDP-1 Built-in display").unwrap()
}

/// The screen beside it.
fn the_screen() -> DisplayId {
    DisplayId::named("DP-1 Dell U2720Q").unwrap()
}

/// Mail down the left of the laptop, the editor on the screen beside it, and a
/// browser in the bottom-right quarter of that screen.
fn a_desk() -> WasOpen {
    WasOpen::nothing()
        .and(Open::of("org.example.Mail", the_laptop(), Split::Of(Place::LeftHalf)).unwrap())
        .and(Open::of("org.example.Editor", the_screen(), Split::TheWholeScreen).unwrap())
        .and(
            Open::of(
                "org.example.Browser",
                the_screen(),
                Split::Of(Place::BottomRightQuarter),
            )
            .unwrap(),
        )
}

/// A log-out that asked every application and got a yes from each.
fn everything_closed(at: &Path, was_open: &WasOpen) {
    let may_end = ready_to_end();
    keeping::at_sign_out(at, &may_end, was_open).unwrap();
}

/// A `MayEnd` from the one road a test can take to one: a log-out on an open
/// seat where nothing was open, which needs no compositor.
fn ready_to_end() -> MayEnd {
    struct NothingToAsk;
    impl alo_leaving::TheApplications for NothingToAsk {
        fn asked_to_close(
            &mut self,
            _what: &alo_applications::Application,
        ) -> alo_leaving::Closing {
            panic!("nothing was open and something was asked")
        }
    }
    let mut accounts = alo_accounts::Accounts::none().unwrap();
    accounts
        .created("anna", 1000, "a password nobody knows")
        .unwrap();
    let signed_in = accounts
        .signs_in("anna", "a password nobody knows")
        .unwrap();
    let session = alo_accounts::Session::opened(signed_in, 1000).unwrap();
    let seat = alo_locking::Seat::<String>::opened(session);
    let alo_leaving::LoggingOut::Ready(may_end) =
        alo_leaving::logging_out::asked(&seat, &WasOpen::nothing(), &mut NothingToAsk)
    else {
        panic!("a session with nothing open was not ready to end")
    };
    may_end
}

/// **A person who asked for it signs in to the applications and places they
/// left** — every application, the screen each was on, and the split each had.
#[test]
fn a_person_who_asked_for_it_signs_in_to_what_they_left() {
    let at = a_folder_of_our_own("what-they-left").join(THE_FILE);
    let mut changes = Changes::untouched();
    changes.set_reopen(true);
    keeping::keep(&at, &changes).unwrap();

    everything_closed(&at, &a_desk());

    let AtSignIn {
        settings,
        restoring,
        refused,
    } = restoring::at_sign_in(&at);
    assert_eq!(refused, None);
    assert!(settings.reopen);
    assert_eq!(restoring, Restoring::These(a_desk()));

    let back = restoring.what_was_open();
    let each: Vec<(&str, &str, &'static str)> = back
        .windows()
        .iter()
        .map(|window| {
            (
                window.what().identifier(),
                window.on().name(),
                window.split().named(),
            )
        })
        .collect();
    assert_eq!(
        each,
        vec![
            ("org.example.Mail", "eDP-1 Built-in display", "left-half"),
            ("org.example.Editor", "DP-1 Dell U2720Q", "the-whole-screen"),
            (
                "org.example.Browser",
                "DP-1 Dell U2720Q",
                "bottom-right-quarter"
            ),
        ]
    );
}

/// **A person who did not ask for it has nothing written down at all.** Not an
/// empty list, not a file with a list in it that nobody reads: no file.
#[test]
fn a_person_who_did_not_ask_has_nothing_written_down() {
    let at = a_folder_of_our_own("did-not-ask").join(THE_FILE);
    everything_closed(&at, &a_desk());
    assert!(!at.exists(), "a list was kept for somebody who did not ask");
    assert_eq!(
        restoring::at_sign_in(&at).restoring,
        Restoring::NothingIsReopened
    );
}

/// **The file holds identifiers and places, and a person can read it.** What is
/// on the disk is exactly three keys per window — and the whole file is shown
/// here, so a fourth key arriving in it is this test failing.
#[test]
fn the_file_holds_identifiers_and_places_and_nothing_else() {
    let at = a_folder_of_our_own("on-the-disk").join(THE_FILE);
    let mut changes = Changes::untouched();
    changes.set_reopen(true);
    keeping::keep(&at, &changes).unwrap();
    everything_closed(&at, &a_desk());

    let written = fs::read_to_string(&at).unwrap();
    assert_eq!(
        written,
        "format = 1\n\nreopen = true\n\n\
         [[was-open]]\napplication = \"org.example.Mail\"\non = \"eDP-1 Built-in display\"\nsplit \
         = \"left-half\"\n\n\
         [[was-open]]\napplication = \"org.example.Editor\"\non = \"DP-1 Dell U2720Q\"\nsplit = \
         \"the-whole-screen\"\n\n\
         [[was-open]]\napplication = \"org.example.Browser\"\non = \"DP-1 Dell U2720Q\"\nsplit = \
         \"bottom-right-quarter\"\n",
        "{written}"
    );
}

/// **A window with a title, a document or an address in it refuses the whole
/// file.** There is no key for any of them, and a file that has one is not read
/// past — so a later release that started writing one, or a person who added
/// one by hand, is a refusal naming the file rather than a diary nobody noticed.
#[test]
fn a_window_with_anything_else_in_it_refuses_the_whole_file() {
    for (what, line) in [
        ("a-title", "title = \"Invoices — March\""),
        ("a-document", "document = \"/home/anna/invoices/march.odt\""),
        ("an-address", "url = \"https://example.org/invoices\""),
        ("a-window", "window = 41"),
    ] {
        let at = a_folder_of_our_own(what).join(THE_FILE);
        fs::write(
            &at,
            format!(
                "format = 1\nreopen = true\n\n[[was-open]]\napplication = \
                 \"org.example.Editor\"\non = \"eDP-1 Built-in display\"\nsplit = \
                 \"the-whole-screen\"\n{line}\n"
            ),
        )
        .unwrap();
        let signing_in = restoring::at_sign_in(&at);
        assert!(signing_in.refused.is_some(), "{what} was read: {line}");
        assert_eq!(signing_in.settings, Settings::shipped(), "{what}");
        assert_eq!(signing_in.restoring, Restoring::NothingIsReopened, "{what}");
    }
}

/// Every source file this crate ships, with its test modules cut off.
///
/// `src/testing.rs` is `cfg(test)` whole and is left out; everything else is
/// split at its `#[cfg(test)]`, so what is read is the code that reaches a
/// machine.
fn the_shipped_source() -> Vec<(String, String)> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    for entry in fs::read_dir(&src).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "testing.rs" {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap();
        let shipped = text.split("#[cfg(test)]").next().unwrap_or("").to_owned();
        found.push((name, shipped));
    }
    assert!(found.len() > 8, "this crate's source was not found");
    found
}

/// The lines of a file that are code rather than what somebody wrote about it.
///
/// A header that says *no title is ever written down* would otherwise be a
/// failure of the check below, which would be a check nobody could keep. The
/// crate's `html_root_url` goes with them: it is where this crate's own
/// documentation is published, and it is not a window.
fn the_code_of(text: &str) -> String {
    text.lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//") && !line.starts_with("#![doc("))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// **Nothing in this crate's shipped code names a thing that was in a window.**
/// No title, no document, no address, no subject, no window identifier — in a
/// field, a variable or a key.
///
/// `src/words.rs` is left out, and deliberately: what this crate *says* about
/// documents is the opposite of a leak — the note beside the setting has to tell
/// a translator that no document, page or message is written down. What matters
/// there is that no sentence has a **gap** one could be filled into, which is
/// `words.rs`'s own
/// `nothing_but_an_identifier_a_path_a_line_or_a_key_is_filled_in`.
#[test]
fn nothing_in_this_crate_names_what_was_in_a_window() {
    let leaks = [
        "title",
        "Title",
        "document",
        "Document",
        "url",
        "Url",
        "URL",
        "address",
        "subject",
        "contents",
        "WindowId",
        "window_id",
        "Window",
    ];
    for (name, shipped) in the_shipped_source() {
        if name == "words.rs" {
            continue;
        }
        let code = the_code_of(&shipped);
        for leak in leaks {
            assert!(
                !code.contains(leak),
                "src/{name} names `{leak}` in its code: what was open is applications and places"
            );
        }
    }
}

/// **The only file this crate writes is `leaving.toml`.** No application's own
/// session file is read or written — this crate names no other file at all, and
/// the rule it keeps its one file by is `alo-kept`'s.
#[test]
fn the_only_file_this_crate_names_is_its_own() {
    assert_eq!(THE_FILE, "leaving.toml");
    for (name, shipped) in the_shipped_source() {
        let code = the_code_of(&shipped);
        for (at, _) in code.match_indices(".toml") {
            let from = code.get(..at).map_or(0, |before| {
                before.rfind('"').map_or(0, |quote| quote.saturating_add(1))
            });
            let named = code.get(from..at.saturating_add(5)).unwrap_or_default();
            assert_eq!(
                named, THE_FILE,
                "src/{name} names the file `{named}`, and this crate keeps one"
            );
        }
    }
}
