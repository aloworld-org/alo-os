//! **Every change this machine offers an agent says, on the approval card,
//! what it changes and what it leaves alone** — and nothing it offers can
//! arrive without both.
//!
//! The owner's decision of 2026-11-01. Each sentence is held to what its verb
//! does by a test that runs the verb, in the crate that carries it out:
//!
//! | Verb | Where it is held |
//! |---|---|
//! | `rename_file`, `move_file`, `archive_folder` | `crates/alo-files/tests/what_each_change_says_it_does.rs` |
//! | `open_application`, `focus_application`, `close_application` | `crates/alo-shell/tests/what_an_application_verb_says_it_does` |
//! | `install_application` | `crates/alo-software/tests/installing_updating_and_removing.rs` |
//!
//! This file is the other half: a change added to the list the daemon offers
//! without its two sentences fails here, and a change added with them fails
//! here until it is named in the table above.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

/// The change verbs the table above names.
const HELD: [&str; 7] = [
    "rename_file",
    "move_file",
    "archive_folder",
    "open_application",
    "focus_application",
    "close_application",
    "install_application",
];

#[test]
fn every_change_this_machine_offers_says_what_it_changes_and_what_it_leaves() {
    let verbs = alo_turn::the_verbs_a_whole_machine_offers().unwrap();
    let changes: Vec<&str> = verbs
        .all()
        .filter(|verb| verb.effect().waits_for_approval())
        .map(alo_capability::Verb::name)
        .collect();
    assert!(
        !changes.is_empty(),
        "the control: this machine offers changes"
    );
    for verb in verbs.all() {
        let says = (
            verb.what_it_changes().is_some(),
            verb.what_it_leaves().is_some(),
        );
        if verb.effect().waits_for_approval() {
            assert_eq!(
                says,
                (true, true),
                "{} is offered and does not say what it changes and leaves alone",
                verb.name()
            );
            assert!(
                HELD.contains(&verb.name()),
                "{} says what it does and no test that runs it is named here",
                verb.name()
            );
        } else {
            assert_eq!(
                says,
                (false, false),
                "{} reads and says it changes",
                verb.name()
            );
        }
    }
    let mut offered = changes.clone();
    offered.sort_unstable();
    let mut held = HELD.to_vec();
    held.sort_unstable();
    assert_eq!(
        offered, held,
        "the table names a change this machine does not offer"
    );
}
