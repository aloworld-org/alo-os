//! **A fresh session resolves what is installed into the agreed order, and a
//! restart keeps it.**
//!
//! The owner asked for this verification by name on 2026-10-10: *verify that a
//! fresh session resolves the installed applications into the agreed pin order
//! and preserves it after restarting.*
//!
//! # What this can verify, and what it cannot
//!
//! Two of the four owners in `docs/contracts/person-settings.md` have not built
//! their half. **Discovery** — reading what is installed off a Linux machine —
//! is the desktop integration lane's, and nothing in this repository produces an
//! `alo_dock::pinning::Discovery` yet. **Persistence** is the owner of
//! `alo-kept`'s, and `dock.toml` has no key for pins.
//!
//! So this cannot start a process, and saying it verifies *a restart* in the
//! sense of a machine rebooting would be the fault this repository calls
//! *declared place versus actual place*. What it does verify is **the whole of
//! the Dock's half, across the seam, in the order a session meets it**: discovery
//! answers, the pins resolve, the caller is told to write them down, a later
//! session reads them back, and nothing re-derives.
//!
//! The store below is a stand-in **shaped by the contract rather than by
//! convenience**: it holds *whether the pins have ever been set* apart from
//! *what they are*, because that is the distinction the contract requires of
//! whatever really stores them, and a stand-in that collapsed the two would make
//! this test pass over the fault it exists to catch.
//!
//! # Why an integration test and not another unit test
//!
//! `src/pinning.rs` holds each clause on its own. This holds the **sequence**:
//! the bug it is built against is not any one answer being wrong, it is two right
//! answers composed in the wrong order — initialising before discovery, or
//! re-deriving after a restart. A unit test of either end sees neither.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None here is the failure being reported, and \
              `expect` names what was wanted. `panic` and `unwrap_used` are deliberately not \
              expected: this file reaches for neither, and an unfulfilled expectation is itself \
              a clippy error, which is how an allowance that has stopped being needed is found"
)]

use alo_dock::pinning::{ARole, Discovery, ThePins, WhatToPin, WhyNotYet};
use alo_dock::window::AppId;

/// An application identifier.
fn app(named: &str) -> AppId {
    AppId::named(named).expect("a named application")
}

/// The names of these pins, in order.
fn names(pins: &[AppId]) -> Vec<String> {
    pins.iter().map(|app| app.name().to_owned()).collect()
}

/// What this machine has, **in an order nothing asked for** — which is the
/// point: a discovery that happened to answer in the agreed order would make
/// the ordering assertions below pass without the ordering working.
fn what_is_installed() -> Vec<(ARole, AppId)> {
    vec![
        (ARole::Settings, app("alo.Settings")),
        (ARole::Apps, app("alo.Apps")),
        (ARole::Browser, app("org.mozilla.firefox")),
        (ARole::Files, app("org.gnome.Nautilus")),
    ]
}

/// The order the owner gave: Files, Browser, Apps, Settings.
const THE_AGREED_ORDER: [&str; 4] = [
    "org.gnome.Nautilus",
    "org.mozilla.firefox",
    "alo.Apps",
    "alo.Settings",
];

/// Where the pins are kept between sessions.
///
/// **Shaped by `docs/contracts/person-settings.md`, not by convenience.** It
/// holds whether the pins have ever been set **apart from** what they are,
/// because that is what the contract requires of whatever really stores them. A
/// stand-in that held only a `Vec` would make every test below pass while the
/// product refilled an emptied Dock on the next boot.
#[derive(Debug, Default, Clone)]
struct TheStore {
    /// `None` until something has been written: never set.
    kept: Option<Vec<AppId>>,
    /// How many times anything has been written, so a test can assert that a
    /// later session wrote **nothing**.
    writes: usize,
}

impl TheStore {
    /// What a session reads at sign-in.
    fn at_sign_in(&self) -> ThePins {
        match &self.kept {
            None => ThePins::NeverSet,
            Some(pins) => ThePins::Chosen(pins.clone()),
        }
    }

    /// One whole session: read, resolve against discovery, write back if owed.
    ///
    /// **The write is conditional on the type and not on this function's
    /// judgement**, which is the thing being verified — a caller cannot decide
    /// to persist a `NotYet` because `should_be_written_down` is false for it.
    fn a_session(&mut self, discovery: &Discovery) -> WhatToPin {
        let answer = self.at_sign_in().what_to_pin(discovery);
        if answer.should_be_written_down() {
            self.kept = Some(answer.to_draw().to_vec());
            self.writes += 1;
        }
        answer
    }
}

/// **A fresh session resolves what is installed into the agreed order, and the
/// next session reads it back unchanged.**
///
/// The owner's sentence, end to end. Two sessions against one store, with the
/// second asserting the thing a restart is for: the same order, and **no write**
/// — because a session that re-derived would undo a person's removals every time
/// they signed in.
#[test]
fn a_fresh_session_pins_the_four_and_the_next_session_keeps_them() {
    let found = Discovery::Completed(what_is_installed());
    let mut store = TheStore::default();

    // The machine a person has just been handed.
    assert!(
        matches!(store.at_sign_in(), ThePins::NeverSet),
        "the premise: nothing has been stored"
    );
    let first = store.a_session(&found);
    assert_eq!(
        names(first.to_draw()),
        THE_AGREED_ORDER,
        "a fresh session did not resolve the installed applications into Files, Browser, Apps, \
         Settings"
    );
    assert!(
        first.should_be_written_down(),
        "the defaults were derived and the caller was not told to store them, so they would be \
         derived again next time"
    );
    assert_eq!(store.writes, 1);

    // Signing out and in again. The same machine, the same discovery.
    let second = store.a_session(&found);
    assert_eq!(
        names(second.to_draw()),
        THE_AGREED_ORDER,
        "the order changed across a restart"
    );
    assert!(
        !second.should_be_written_down(),
        "the second session was told to write again, which means it re-derived the defaults \
         rather than reading what was stored"
    );
    assert_eq!(store.writes, 1, "a second session wrote to the store");
}

/// **What the person did to their Dock survives a restart**, including the
/// things that look like absences: a removal and a reordering.
///
/// This is the clause a re-derivation would quietly undo, and it is the one a
/// person would notice first — they take the browser off, and the next morning
/// it is back.
#[test]
fn a_removal_and_a_reordering_both_survive_a_restart() {
    let found = Discovery::Completed(what_is_installed());
    let mut store = TheStore::default();
    store.a_session(&found);

    // The person removes the browser and moves Settings to the front.
    let theirs = vec![
        app("alo.Settings"),
        app("org.gnome.Nautilus"),
        app("alo.Apps"),
    ];
    store.kept = Some(theirs.clone());
    let writes_before = store.writes;

    for session in 1..=3 {
        let answer = store.a_session(&found);
        assert_eq!(
            names(answer.to_draw()),
            names(&theirs),
            "session {session} did not give the person back the Dock they made"
        );
        assert!(
            !answer.should_be_written_down(),
            "session {session} was told to write, so the defaults ran again"
        );
    }
    assert_eq!(
        store.writes, writes_before,
        "a session wrote over the person's own pins"
    );
}

/// **A first session where discovery has not finished stores nothing, and the
/// session after it initialises properly.**
///
/// The owner's fourth clause, in the order a machine meets it. The fault being
/// forbidden is not that the first session shows an empty Dock — there is
/// nothing to show yet — it is that it **writes** one, after which every later
/// session reads that emptiness back as a choice the person made.
#[test]
fn a_first_session_with_no_discovery_leaves_the_machine_uninitialised() {
    for (why_it_failed, expected) in [
        (Discovery::StillLooking, WhyNotYet::StillLooking),
        (Discovery::Failed, WhyNotYet::ItFailed),
    ] {
        let mut store = TheStore::default();

        let first = store.a_session(&why_it_failed);
        assert_eq!(first, WhatToPin::NotYet(expected), "{why_it_failed:?}");
        assert!(first.to_draw().is_empty(), "{why_it_failed:?}");
        assert_eq!(
            store.writes, 0,
            "{why_it_failed:?}: a session with no answer about what is installed wrote to the \
             store, so this machine is now a person who chose an empty Dock"
        );
        assert!(
            matches!(store.at_sign_in(), ThePins::NeverSet),
            "{why_it_failed:?}: the machine is no longer uninitialised"
        );

        // The scan lands. This session is the fresh one.
        let then = store.a_session(&Discovery::Completed(what_is_installed()));
        assert_eq!(
            names(then.to_draw()),
            THE_AGREED_ORDER,
            "{why_it_failed:?}: the session after a failed scan did not initialise"
        );
        assert_eq!(store.writes, 1, "{why_it_failed:?}");
    }
}

/// **A machine that genuinely has none of the four is initialised and stays
/// initialised**, which is the opposite answer to the test above from the same
/// empty list.
///
/// If these two ever agree, `Discovery` has stopped carrying what it exists to
/// carry and the whole distinction has collapsed.
#[test]
fn a_machine_with_none_of_the_four_is_still_a_machine_that_has_been_asked() {
    let mut store = TheStore::default();
    let found_nothing = Discovery::Completed(Vec::new());

    let first = store.a_session(&found_nothing);
    assert!(first.to_draw().is_empty());
    assert_eq!(
        store.writes, 1,
        "a machine with none of the four was left uninitialised, so it will re-run discovery's \
         defaults on every boot"
    );

    // And a later session where something *is* installed does not pin it: the
    // owner's *an application omitted at first run must not automatically become
    // pinned if installed later.*
    let later = store.a_session(&Discovery::Completed(what_is_installed()));
    assert!(
        later.to_draw().is_empty(),
        "applications installed after the first run pinned themselves"
    );
    assert!(!later.should_be_written_down());
    assert_eq!(store.writes, 1);
}

/// **A discovery failure in a later session does not empty a Dock somebody
/// already has.**
///
/// The case that would bite hardest and the easiest to miss: the person is
/// already set up, and one morning the scan is slow. Their Dock must be their
/// Dock.
#[test]
fn a_later_discovery_failure_does_not_touch_a_dock_that_is_already_set() {
    let mut store = TheStore::default();
    store.a_session(&Discovery::Completed(what_is_installed()));
    let writes_before = store.writes;

    for discovery in [
        Discovery::StillLooking,
        Discovery::Failed,
        Discovery::Completed(Vec::new()),
    ] {
        let answer = store.a_session(&discovery);
        assert_eq!(
            names(answer.to_draw()),
            THE_AGREED_ORDER,
            "{discovery:?} changed a Dock that was already set"
        );
        assert!(!answer.should_be_written_down(), "{discovery:?}");
    }
    assert_eq!(store.writes, writes_before);
}

/// **The order comes from the roles and not from the order discovery answers
/// in**, across a restart as well as on the first run.
///
/// `what_is_installed` is deliberately scrambled; this scrambles it differently
/// and asserts the same Dock. A resolution that passed the list through would
/// give a different Dock to two machines that have exactly the same software.
#[test]
fn two_machines_with_the_same_software_get_the_same_dock() {
    let one = {
        let mut store = TheStore::default();
        store.a_session(&Discovery::Completed(what_is_installed()));
        store
    };
    let other = {
        let mut installed = what_is_installed();
        installed.reverse();
        let mut store = TheStore::default();
        store.a_session(&Discovery::Completed(installed));
        store
    };

    let first = one.kept.expect("the first machine stored its pins");
    let second = other.kept.expect("the second machine stored its pins");
    assert_eq!(names(&first), THE_AGREED_ORDER);
    assert_eq!(
        names(&first),
        names(&second),
        "two machines with the same software got different Docks, so the order is coming from \
         whatever order discovery answered in"
    );
}
