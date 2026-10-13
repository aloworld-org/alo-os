//! That every key is a key, that none is said twice, and that the words a
//! person acts on are not interchangeable.

use std::collections::BTreeSet;

use alo_strings::Word;

use super::{ALO_IS_WORKING, EVERY_WORD, STOPPED, STOPPING, WAITING_FOR_YOUR_PERMISSION};

/// **What we ship is held to the rule everybody else is held to.**
///
/// `Word::key` does not check, because a key written in this file cannot arrive
/// from anywhere else; this is the test that makes that true.
#[test]
fn every_key_is_a_key() {
    for word in EVERY_WORD {
        assert_eq!(
            alo_strings::Key::named(word.named()),
            Ok(word.key()),
            "{}",
            word.named()
        );
    }
}

/// A key names one string. Two words sharing one would mean whichever was
/// declared second is a string nobody can reach.
#[test]
fn no_two_words_are_named_the_same() {
    let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();

    assert_eq!(named.len(), EVERY_WORD.len());
}

/// Every one of them is in the area a reader can sort by, which is what lets
/// one vocabulary hold every crate's strings.
#[test]
fn everything_this_crate_says_says_it_is_this_crate() {
    for word in EVERY_WORD {
        assert_eq!(word.key().area(), "tasks", "{}", word.named());
    }
}

/// Every one of them says something, and every one carries the note a
/// translator reads.
#[test]
fn every_word_says_something_and_notes_why() {
    for word in EVERY_WORD {
        assert!(
            !word.says().trim().is_empty(),
            "{} says nothing",
            word.named()
        );
        assert!(
            word.note().is_some_and(|note| note.len() > 40),
            "{} has no note worth reading, and a translator gets the note rather than the \
             window",
            word.named()
        );
    }
}

/// **Working and waiting are not the same sentence.**
///
/// They are the only two words that say whether a person has to do something,
/// read at a glance from across a room. This cannot check a translation, but it
/// can hold the English apart so the note beside them has a subject.
#[test]
fn working_and_waiting_are_not_interchangeable() {
    assert_ne!(ALO_IS_WORKING.says(), WAITING_FOR_YOUR_PERMISSION.says());
    assert_ne!(ALO_IS_WORKING.named(), WAITING_FOR_YOUR_PERMISSION.named());
}

/// **Stopping and stopped are different words**, because the difference between
/// them is the whole of what a Stop must not claim.
///
/// A window saying *Stopped* while the work was still running would be the one
/// thing the owner's own wording forbids: *a Stop that reported success while
/// something was still running would be worse than no Stop.*
#[test]
fn stopping_is_not_stopped() {
    assert_ne!(STOPPING.says(), STOPPED.says());
    assert_ne!(STOPPING.named(), STOPPED.named());
    assert!(
        STOPPING.says().len() > STOPPED.says().len(),
        "the word for work still ending is no longer than the word for work that has ended, \
         which is how the two get confused"
    );
}
