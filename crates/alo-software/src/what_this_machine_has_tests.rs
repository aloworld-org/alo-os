//! What a reading of this machine takes, what it refuses, and the one
//! distinction the promise turns on.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected value is the failure being reported"
)]

use super::{WhatThisMachineHas, what_this_machine_has};
use crate::source::{Configured, SourceName};

use crate::tool::{Failed, Tool};
use alo_applications::Application;

/// A tool that answers with what a test put in it.
///
/// **`installed` and `open` answer differently on purpose.** They are the two
/// methods that return a list of identifiers, and a reading that asked for the
/// running applications instead of the installed ones would produce a shorter
/// list that looks entirely plausible. Giving them different contents is what
/// makes the wiring checkable — a fixture whose seams all answer alike cannot
/// catch a reading that took the wrong one.
struct AToolThatSays {
    installed: Result<Vec<String>, Failed>,
    open: Vec<String>,
}

impl AToolThatSays {
    fn listing(lines: &[&str]) -> Self {
        Self {
            installed: Ok(lines.iter().map(|line| (*line).to_owned()).collect()),
            open: vec!["org.running.Only".to_owned()],
        }
    }

    fn refusing(why: Failed) -> Self {
        Self {
            installed: Err(why),
            open: Vec::new(),
        }
    }
}

impl Tool for AToolThatSays {
    fn installed(&self) -> Result<Vec<String>, Failed> {
        self.installed.clone()
    }

    fn open(&self) -> Result<Vec<String>, Failed> {
        Ok(self.open.clone())
    }

    // The rest are not this reading's business, and answer so rather than
    // panicking: a fixture that panics on an unused method tells a reader the
    // method was reached, which is a different fault from the one being tested.
    fn sources(&self) -> Result<Vec<Configured>, Failed> {
        Err(Failed::NotOffered)
    }

    fn install(&self, _: &SourceName, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotOffered)
    }

    fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
        Err(Failed::NotOffered)
    }

    fn update(&self, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotOffered)
    }

    fn remove(&self, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotOffered)
    }
}

/// **A failed scan yields no list at all**, which is the clause this whole file
/// exists to make true.
///
/// `docs/features.md` [v0.01]: *the pins are written once, and only after
/// application discovery has succeeded — **an unfinished scan and a failed one
/// are not evidence that an application is absent**.*
///
/// So the type must make the mistake unrepresentable rather than discouraged.
/// There is no `WhatThisMachineHas` to be had from a failure, so a caller
/// cannot read a machine that would not answer as a machine with nothing on
/// it. `absent is stronger than defaulted`: a default can be failed to
/// override, an absence cannot be overridden wrongly.
#[test]
fn a_failed_scan_hands_back_no_list_for_anybody_to_misread() {
    for why in [
        Failed::NotOffered,
        Failed::NotInstalled,
        Failed::DidNotAnswer {
            said: "the tool was not there".to_owned(),
        },
    ] {
        let read = what_this_machine_has(&AToolThatSays::refusing(why.clone()));
        match read {
            Err(said) => assert_eq!(said, why, "the tool's own reason is not carried through"),
            Ok(found) => panic!(
                "a failed scan produced a list of {} — a caller could read this \
                 machine as having nothing installed",
                found.how_many()
            ),
        }
    }
}

/// **And a scan that succeeded and found nothing is a different fact.**
///
/// Asserted beside the failure deliberately: both end in *no applications*, and
/// the two must not be one value. A machine really can have none — before
/// `alo-shipping` has run, which is exactly when a fresh Dock is first drawn.
#[test]
fn a_scan_that_found_nothing_is_not_a_scan_that_failed() {
    let empty = what_this_machine_has(&AToolThatSays::listing(&[]))
        .expect("a tool that answered with no lines has not failed");
    assert_eq!(empty.how_many(), 0);
    assert!(empty.installed().is_empty());
    assert!(
        empty.is_whole(),
        "a machine with nothing installed, read cleanly, is a whole reading"
    );

    let failed = what_this_machine_has(&AToolThatSays::refusing(Failed::NotOffered));
    assert!(failed.is_err(), "the premise: the other one fails");
}

/// **Every identifier the tool prints becomes an application.**
#[test]
fn what_the_tool_lists_is_what_the_machine_has() {
    let read = what_this_machine_has(&AToolThatSays::listing(&[
        "org.mozilla.firefox",
        "org.kde.dolphin",
        "org.gnome.TextEditor",
    ]))
    .expect("three identifiers are three applications");
    assert_eq!(read.how_many(), 3);
    for identifier in [
        "org.mozilla.firefox",
        "org.kde.dolphin",
        "org.gnome.TextEditor",
    ] {
        assert!(
            read.installed().has(identifier),
            "{identifier} was listed and is not on the machine's list"
        );
    }
    assert!(read.is_whole(), "{:?} {:?}", read.not_taken(), read.twice());
}

/// **The reading asks what is installed, not what is running.**
///
/// The fixture answers those two differently, so a reading that called `open`
/// would come back with `org.running.Only` and nothing else — a shorter list
/// that looks entirely plausible and would quietly leave applications off a
/// fresh Dock. This is the test that would catch it.
#[test]
fn it_reads_the_installed_list_and_not_the_running_one() {
    let read =
        what_this_machine_has(&AToolThatSays::listing(&["org.kde.ark"])).expect("one identifier");
    assert!(read.installed().has("org.kde.ark"));
    assert!(
        !read.installed().has("org.running.Only"),
        "the reading took the running applications instead of the installed ones"
    );
}

/// **A line that is not an identifier is reported, not dropped.**
///
/// The identifiers come from outside this repository, so the reading has to do
/// something honest with a line it does not understand. `Installed::add`'s own
/// rule is the precedent: it *says which happened rather than swallowing it*.
#[test]
fn a_line_that_is_not_an_identifier_is_named_rather_than_discarded() {
    let read = what_this_machine_has(&AToolThatSays::listing(&[
        "org.kde.dolphin",
        "not an identifier at all",
        "org.kde.ark",
    ]))
    .expect("the scan itself did not fail");
    assert_eq!(read.how_many(), 2, "the two good ones are taken");
    assert_eq!(
        read.not_taken(),
        ["not an identifier at all"],
        "the line this crate could not use is not reported"
    );
    assert!(
        !read.is_whole(),
        "a reading that refused a line is not whole, however many it took"
    );
}

/// **An identifier printed twice keeps the first and says so.**
///
/// `Installed`'s own security argument is why the first wins: *taking the later
/// one would mean something installed after a grant was made could take the
/// identifier that grant is over.* This holds that the duplicate is reported as
/// well, so a packaging fault on a machine is visible rather than silent.
#[test]
fn an_identifier_printed_twice_is_kept_once_and_reported() {
    let read = what_this_machine_has(&AToolThatSays::listing(&[
        "org.kde.dolphin",
        "org.kde.ark",
        "org.kde.dolphin",
    ]))
    .expect("the scan itself did not fail");
    assert_eq!(
        read.how_many(),
        2,
        "the duplicate did not add a second entry"
    );
    assert_eq!(read.twice(), ["org.kde.dolphin"]);
    assert!(
        read.not_taken().is_empty(),
        "a duplicate is not an unreadable line"
    );
    assert!(!read.is_whole());
}

/// **The two kinds of refusal are kept apart**, because they have different
/// owners: a line this crate does not understand is the tool's or the
/// packager's, and an identifier claimed twice is one machine's.
#[test]
fn an_unreadable_line_and_a_duplicate_are_not_one_list() {
    let read = what_this_machine_has(&AToolThatSays::listing(&[
        "org.kde.ark",
        "",
        "org.kde.ark",
        "also not an identifier",
    ]))
    .expect("the scan itself did not fail");
    assert_eq!(read.how_many(), 1);
    assert_eq!(read.twice(), ["org.kde.ark"]);
    assert_eq!(read.not_taken().len(), 2, "{:?}", read.not_taken());
}

/// **`is_whole` is about taking everything, not about finding something.**
///
/// A machine with no applications, read cleanly, is whole. A machine with fifty
/// and one unreadable line is not. Stated as a table because the two readings
/// of that word point in opposite directions and only one is right.
#[test]
fn whole_means_nothing_was_refused_rather_than_something_was_found() {
    let cases: [(&[&str], bool, usize); 4] = [
        (&[], true, 0),
        (&["org.kde.ark"], true, 1),
        (&["org.kde.ark", "bad line"], false, 1),
        (&["org.kde.ark", "org.kde.ark"], false, 1),
    ];
    for (lines, whole, taken) in cases {
        let read: WhatThisMachineHas = what_this_machine_has(&AToolThatSays::listing(lines))
            .expect("the scan itself did not fail");
        assert_eq!(read.is_whole(), whole, "{lines:?}");
        assert_eq!(read.how_many(), taken, "{lines:?}");
    }
}
