//! The forwarding, and the one thing this crate decides: which `SettingsKept`
//! each of the terminal's three outcomes becomes.
//!
//! **What the states offer and what the write does are measured in
//! `alo-terminal`**, where they live. Repeating them here would be two tests of
//! one thing, and the one that broke would be the one nobody read.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use alo_strings::Strings;

use super::TheShellIntegration;
use crate::settings_kept::SettingsKept;

/// The vocabulary a machine is really built from, so a missing word fails here.
fn strings() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say()
            .expect("the vocabulary every machine is built from"),
    )
}

/// **A row is offered where there is a shell and a folder, and not otherwise.**
#[test]
fn a_row_is_offered_only_where_there_is_something_to_add() {
    let folder = tempfile::tempdir().expect("a directory of our own");

    let offered = TheShellIntegration::at_sign_in(Some("/bin/bash"), Some(folder.path()));
    assert!(offered.offers_a_row());
    assert_eq!(offered.row(&strings()).len(), 3);

    for (shell, at) in [
        (Some("/usr/bin/zsh"), Some(folder.path())),
        (Some("/bin/bash"), None),
        (None, Some(folder.path())),
    ] {
        let none = TheShellIntegration::at_sign_in(shell, at);
        assert!(!none.offers_a_row(), "{shell:?} {at:?}");
        assert!(none.row(&strings()).is_empty(), "{shell:?} {at:?}");
    }
}

/// **Each of the terminal\'s three outcomes becomes the right `SettingsKept`.**
///
/// This mapping is the only decision this file makes, and it is the one a
/// person feels: `Written` draws the section as changed, `NowhereToKeep` does
/// not.
#[test]
fn the_terminals_outcomes_become_this_windows() {
    let folder = tempfile::tempdir().expect("a directory of our own");

    let mut offered = TheShellIntegration::at_sign_in(Some("bash"), Some(folder.path()));
    assert_eq!(offered.add(&strings()), SettingsKept::Written);
    assert!(offered.is_already_there());
    assert_eq!(
        offered.add(&strings()),
        SettingsKept::Written,
        "pressing again is still written, not a failure"
    );

    let mut nothing = TheShellIntegration::at_sign_in(None, None);
    assert_eq!(nothing.add(&strings()), SettingsKept::NowhereToKeep);
    assert!(!nothing.is_already_there());
}

/// **The section says why there is no row, in the terminal\'s words.**
#[test]
fn the_section_forwards_the_sentence_that_says_why() {
    let said = TheShellIntegration::at_sign_in(Some("/usr/bin/fish"), None).said(&strings());
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(
        said.first().expect("the one sentence").contains("fish"),
        "the shell is named back to the person: {said:?}"
    );
}
