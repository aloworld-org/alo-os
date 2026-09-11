//! The question this crate's own list cannot ask: whether what it says
//! survives being put beside everybody else's.
//!
//! `crate::words`' tests say the list is well formed. This says it is
//! **collected** — that the one vocabulary a translation is checked against
//! holds these four keys, so a translator's line for a sign-in that did not
//! finish is read as a line rather than as a mistake.
//!
//! It is worth its own file because the failure it catches is silent:
//! `crates/alo-overlay` declared nine strings that `alo-saying` collected none
//! of, and every sentence the overlay had would have reached a real shell as a
//! missing key. `crates/alo-collected` turned that into arithmetic over the
//! workspace; this is the same question asked from this crate's own side, where
//! whoever adds a word to `words.rs` is standing.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_saying::everything_this_machine_can_say;

/// **Every sentence this crate can say is in the machine's one vocabulary.**
#[test]
fn every_sentence_this_crate_can_say_is_in_the_machines_vocabulary() {
    let Ok(vocabulary) = everything_this_machine_can_say() else {
        panic!("alo OS's own words do not collect");
    };

    for word in alo_sessiond::EVERY_WORD {
        let key = word.key();
        assert!(
            vocabulary.phrase(&key).is_some(),
            "{key} is declared here and collected nowhere"
        );
    }
}

/// **And the English in the vocabulary is this crate's English**, rather than
/// a key that somebody else happened to declare first. A collision would leave
/// a person reading somebody else's sentence about their own sign-in.
#[test]
fn the_sentence_collected_is_the_sentence_this_crate_wrote() {
    let Ok(vocabulary) = everything_this_machine_can_say() else {
        panic!("alo OS's own words do not collect");
    };

    for word in alo_sessiond::EVERY_WORD {
        let key = word.key();
        let Some(phrase) = vocabulary.phrase(&key) else {
            panic!("{key} is collected nowhere");
        };
        assert_eq!(phrase.source().as_written(), word.says(), "{key}");
    }
}
