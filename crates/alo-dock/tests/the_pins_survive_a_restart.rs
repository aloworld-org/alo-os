//! **What a person did to their Dock, through a real file and back.**
//!
//! `docs/features.md` [v0.01]: *after that the person owns the list: additions,
//! removals, reordering and a list they deliberately emptied all survive a
//! restart and an update, and an empty list is never refilled with the defaults
//! on the grounds that it is empty.*
//!
//! `crates/alo-dock/tests/a_fresh_session_pins_the_defaults_and_keeps_them.rs`
//! holds that rule against a **stand-in** store, which is what let the rule be
//! settled before anything could write. This file is the other half: the same
//! clauses against `alo_dock::keeping`, a real path, and a file read back off
//! the disk.
//!
//! # The three states, and why a file is the only place they can be told apart
//!
//! `None`, `Some(vec![])` and `Some(list)` are three facts in memory and the
//! question is whether they are still three after a write and a read. **The
//! middle one is where it goes wrong**: a `skip_serializing_if` that skipped an
//! empty list would write the same file for *nobody has chosen* and *I emptied
//! it*, and the person who most clearly did not want a Dock would be handed the
//! defaults on their next sign-in. No test of `Changes` alone can see that,
//! because the collapse happens in the bytes.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dock::changes::Setting;
use alo_dock::keeping::{self, THE_FILE};
use alo_dock::{ADockEntry, ANativeAction, Changes};

/// An application entry.
fn app(named: &str) -> ADockEntry {
    ADockEntry::Application(
        alo_dock::AppId::named(named).expect("a plain identifier is an application"),
    )
}

/// The Settings entry.
fn settings() -> ADockEntry {
    ADockEntry::Native(ANativeAction::Settings)
}

/// A private folder with a `dock.toml` path in it.
fn a_folder() -> (tempfile::TempDir, std::path::PathBuf) {
    let folder = tempfile::tempdir().expect("a private directory");
    let at = folder.path().join(THE_FILE);
    (folder, at)
}

/// Write these changes and read them back, as a sign-out and a sign-in would.
fn through_a_file(at: &std::path::Path, changes: &Changes) -> Changes {
    keeping::keep(at, changes).expect("the changes are written");
    keeping::read(at).expect("the file this test just wrote reads back")
}

/// **A person's arrangement survives, in their order, with both kinds in it.**
///
/// The order is asserted rather than the membership: a file that read back the
/// same entries in another order would be a Dock a person has to re-arrange
/// every morning.
#[test]
fn an_arrangement_of_both_kinds_survives_a_restart() {
    let (_folder, at) = a_folder();
    let theirs = vec![
        settings(),
        app("org.mozilla.firefox"),
        app("org.kde.dolphin"),
    ];
    let mut changes = Changes::untouched();
    changes.set_pins(theirs.clone());

    let back = through_a_file(&at, &changes);

    assert_eq!(
        back.pins(),
        Some(theirs.as_slice()),
        "the Dock a person arranged came back different"
    );
}

/// **A Dock somebody deliberately emptied comes back empty, and is not the same
/// file as a Dock nobody has chosen.**
///
/// The clause the whole `Option` exists for, asserted in the one place it can
/// fail: the bytes. Both halves are needed — that the value survives, and that
/// the two files differ — because a reader that answered `Some(vec![])` for a
/// file with no `pins` key at all would pass the first and break every machine
/// that has never been touched.
#[test]
fn a_deliberately_empty_dock_is_not_a_dock_nobody_has_chosen() {
    let (_folder, emptied_at) = a_folder();
    let (_untouched_folder, untouched_at) = a_folder();

    let mut emptied = Changes::untouched();
    emptied.set_pins(Vec::new());
    let back = through_a_file(&emptied_at, &emptied);
    assert_eq!(
        back.pins(),
        Some([].as_slice()),
        "a Dock the person emptied read back as a Dock nobody has chosen, so the defaults would \
         refill it"
    );

    let untouched = through_a_file(&untouched_at, &Changes::untouched());
    assert_eq!(
        untouched.pins(),
        None,
        "a machine nobody has touched read back as a Dock somebody emptied, so it would never \
         get its defaults"
    );

    let emptied_text = std::fs::read_to_string(&emptied_at).expect("the emptied file is there");
    let untouched_text =
        std::fs::read_to_string(&untouched_at).expect("the untouched file is there");
    assert_ne!(
        emptied_text, untouched_text,
        "the two states wrote the same file, so nothing downstream can tell them apart"
    );
    assert!(
        emptied_text.contains("pins"),
        "an emptied Dock wrote no pins key at all: {emptied_text}"
    );
    assert!(
        !untouched_text.contains("pins"),
        "a machine nobody has touched wrote a pins key: {untouched_text}"
    );
}

/// **A removal sticks.** The clause a person notices first: they take the
/// browser off, and the next morning it is not back.
#[test]
fn a_removal_survives_a_restart() {
    let (_folder, at) = a_folder();
    let mut changes = Changes::untouched();
    changes.set_pins(vec![
        app("org.kde.dolphin"),
        app("org.mozilla.firefox"),
        settings(),
    ]);
    through_a_file(&at, &changes);

    // They remove the browser.
    let mut now = keeping::read(&at).expect("what was written reads");
    let left: Vec<ADockEntry> = now
        .pins()
        .expect("the pins were set")
        .iter()
        .filter(|entry| {
            entry
                .application()
                .is_none_or(|a| a.name() != "org.mozilla.firefox")
        })
        .cloned()
        .collect();
    now.set_pins(left);

    let back = through_a_file(&at, &now);

    assert_eq!(
        back.pins(),
        Some([app("org.kde.dolphin"), settings()].as_slice()),
        "an application the person removed came back, or something else left with it"
    );
}

/// **A native entry is removed and stays removed**, which is the owner's
/// *native describes the implementation; it does not mean permanent* carried
/// all the way to a file.
///
/// Its own test rather than a line in the one above, because this is the clause
/// somebody would be most tempted to "fix" by making Settings unconditional —
/// and law 5 forbids that **whatever the reason given for it**.
#[test]
fn a_native_entry_the_person_removed_stays_removed() {
    let (_folder, at) = a_folder();
    let mut changes = Changes::untouched();
    changes.set_pins(vec![app("org.kde.dolphin"), settings()]);
    through_a_file(&at, &changes);

    let mut now = keeping::read(&at).expect("what was written reads");
    now.set_pins(vec![app("org.kde.dolphin")]);

    let back = through_a_file(&at, &now);

    assert_eq!(
        back.pins(),
        Some([app("org.kde.dolphin")].as_slice()),
        "the Settings entry came back after the person took it off"
    );
}

/// **Putting the pins back is not emptying them.**
///
/// `Setting::WhatIsPinned` through `forget` returns the Dock to *nobody has
/// chosen*, so the defaults apply again — which is what *put it back as alo OS
/// ships it* means. `set_pins(vec![])` is the opposite and the test above holds
/// it. The two share a type and must never share a value.
#[test]
fn forgetting_the_pins_is_not_the_same_as_emptying_them() {
    let (_folder, at) = a_folder();
    let mut changes = Changes::untouched();
    changes.set_pins(vec![app("org.kde.dolphin")]);
    through_a_file(&at, &changes);

    let mut now = keeping::read(&at).expect("what was written reads");
    assert!(
        now.forget(Setting::WhatIsPinned),
        "there was nothing to forget, so this test asserts nothing"
    );

    let back = through_a_file(&at, &now);

    assert_eq!(
        back.pins(),
        None,
        "forgetting the pins emptied them instead, so the defaults would never apply again"
    );
    assert!(
        !now.forget(Setting::WhatIsPinned),
        "and only once, as for every other setting"
    );
}

/// **A hand-written file goes through the same validation as everything else.**
///
/// The contract's own reason for TOML is that *this file is typed by a person
/// as often as it is written by a panel*. So the refusals are asserted against
/// text somebody could type: a pin naming both kinds, a pin naming neither, and
/// an application whose name is not one.
///
/// **Refused whole, not in part** — `alo-kept`'s rule — so a file with one bad
/// pin leaves the Dock as the release ships it rather than with the pins that
/// happened to parse.
#[test]
fn a_hand_written_pin_that_is_not_one_refuses_the_file() {
    for (what, text) in [
        (
            "both kinds",
            "format = 1\n[[pins]]\napplication = \"org.kde.dolphin\"\nnative = \"settings\"\n",
        ),
        ("neither kind", "format = 1\n[[pins]]\n"),
        (
            "a name that is not one",
            "format = 1\n[[pins]]\napplication = \"   \"\n",
        ),
        (
            "an action this release does not have",
            "format = 1\n[[pins]]\nnative = \"reboot-the-machine\"\n",
        ),
    ] {
        let (_folder, at) = a_folder();
        std::fs::write(&at, text).expect("the folder is writable");

        let refused = keeping::read(&at);

        let why = refused.expect_err(&format!("{what}: a pin that is not one was read as a Dock"));
        // **Which refusal, not that one happened.** The three hand-written
        // faults all arrive as `NotItsShape`, and that is what lets the
        // person's sentence be `dock.kept.not-understood` in their own
        // language — `alo_dock::unkept` maps the variant with `{ .. }` and
        // **drops** the English the deserialiser produced. Asserting the
        // variant is what holds that: a refusal that arrived as some other
        // variant would be said with some other sentence, and a reader of this
        // test would not find out.
        assert!(
            matches!(why.why(), alo_kept::Unread::NotItsShape { .. }),
            "{what}: refused as {:?}, so the sentence a person is told is not the one this \
             file's exemption argument rests on",
            why.why()
        );
        // And the file is left for the person to mend rather than written over.
        assert_eq!(
            std::fs::read_to_string(&at).expect("the file is still there"),
            text,
            "{what}: the file was rewritten under the person"
        );
    }
}

/// **A file a person typed by hand, with both kinds in it, reads.**
///
/// The control for the test above: every refusal there must be about the pin
/// and not about the shape, and the only way to say so is to show the shape
/// working. Without this, a reader that refused **every** `[[pins]]` table
/// would pass all four cases above.
#[test]
fn a_hand_written_file_with_both_kinds_reads() {
    let (_folder, at) = a_folder();
    std::fs::write(
        &at,
        "format = 1\n\n[[pins]]\napplication = \"org.kde.dolphin\"\n\n[[pins]]\nnative = \
         \"settings\"\n",
    )
    .expect("the folder is writable");

    let read = keeping::read(&at).expect("a file a person could type reads");

    assert_eq!(
        read.pins(),
        Some([app("org.kde.dolphin"), settings()].as_slice()),
        "a hand-written arrangement did not read as what it says"
    );
}
