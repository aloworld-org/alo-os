//! Whether what this crate says survives being put beside everybody else's.
//!
//! `alo_locking::words`' own tests say the list is well formed. This says it
//! is **collected**: the one vocabulary a translation is checked against holds
//! the lock screen's sentence, so it reaches a person at a locked machine in
//! their own language rather than as a key.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_saying::everything_this_machine_can_say;

/// **Every sentence this crate can say is in the machine's one vocabulary**,
/// with this crate's own English.
#[test]
fn every_sentence_this_crate_can_say_is_in_the_machines_vocabulary() {
    let Ok(vocabulary) = everything_this_machine_can_say() else {
        panic!("alo OS's own words do not collect");
    };
    for word in alo_locking::EVERY_WORD {
        let key = word.key();
        let Some(phrase) = vocabulary.phrase(&key) else {
            panic!("{key} is declared here and collected nowhere");
        };
        assert_eq!(phrase.source().as_written(), word.says(), "{key}");
    }
}

/// **No sentence this crate says names anything alo OS rents.** A person told
/// their machine is locked is not asked to learn what locks it.
#[test]
fn no_sentence_this_crate_says_names_anything_we_rent() {
    let Ok(vocabulary) = everything_this_machine_can_say() else {
        panic!("alo OS's own words do not collect");
    };
    let overheard = alo_saying::what_a_person_would_have_to_learn(&vocabulary);
    let ours: Vec<String> = overheard
        .iter()
        .filter(|one| one.key().area() == "locking")
        .map(|one| format!("{one}"))
        .collect();
    assert!(ours.is_empty(), "{ours:?}");
}
