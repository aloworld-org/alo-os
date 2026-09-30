//! What undo is holding is its own line beside what is filling the disk, in
//! three states and never as a zero.
//!
//! `docs/decisions/0045-what-undoing-rewinds-to.md`'s **fourth accepted term**
//! says *what is filling the disk counts snapshots, by name*. Before this, the
//! answer said nothing about them at all — `alo-measuring` had no mention of a
//! snapshot or an undo anywhere in `src/` — so a person whose disk was full of
//! yesterday's turns was shown a tree that did not account for them and sent
//! hunting for space the machine itself was keeping.
//!
//! What is checked here is the shape of the answer, not a machine: **that the
//! line exists, that it has three states, that none of them is a zero, and
//! that the tree's own numbers did not move because of it.**

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_measuring::{NoUndoHere, Source, UndoIsHolding, WhatUndoIsHolding, measuring_words};
use alo_strings::Strings;

/// English, with nothing translated.
fn in_english() -> Strings {
    Strings::of(measuring_words().unwrap())
}

/// **Every state that is not a number has a sentence a person can read**, and
/// the one that is a number has none, because a number is shown as itself.
///
/// The same shape as `Number`, which is *a number or a sentence, and never a
/// zero standing in for a sentence*.
#[test]
fn every_state_is_a_number_or_a_sentence_and_never_both() {
    let strings = in_english();

    let bytes = UndoIsHolding::Bytes {
        value: 12_884_901_888,
        from: Source::of("/var/home/.snapshots", "undo"),
    };
    assert_eq!(bytes.value(), Some(12_884_901_888));
    assert!(
        bytes.instead(&strings).is_none(),
        "a number is shown as itself and needs no sentence"
    );

    for sentence in [
        UndoIsHolding::NotOnThisMachine,
        UndoIsHolding::NotAnswered {
            why: "the machine did not answer".to_owned(),
        },
    ] {
        assert_eq!(sentence.value(), None, "a sentence is not also a number");
        let said = sentence
            .instead(&strings)
            .expect("a state with no number has a sentence in its place");
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.text().is_empty(), "{said}");
    }
}

/// **A zero is never the answer, and the two that are not numbers are not the
/// same as each other.**
///
/// This is the whole reason the type exists rather than a `u64`. *Cannot keep
/// an undo*, *would not say how much*, and *is keeping nothing* are three
/// different facts about a person's disk, and a number can only carry one of
/// them.
#[test]
fn cannot_hold_one_is_not_the_same_answer_as_holds_nothing() {
    let cannot = UndoIsHolding::NotOnThisMachine;
    let would_not_say = UndoIsHolding::NotAnswered {
        why: "permission denied".to_owned(),
    };
    let holds_nothing = UndoIsHolding::Bytes {
        value: 0,
        from: Source::of("/var/home/.snapshots", "undo"),
    };

    assert_ne!(cannot, would_not_say);
    assert_ne!(cannot, holds_nothing);
    assert_ne!(would_not_say, holds_nothing);

    assert_eq!(cannot.value(), None);
    assert_eq!(would_not_say.value(), None);
    assert_eq!(
        holds_nothing.value(),
        Some(0),
        "a real zero is still readable as a number; what must never happen is \
         the other two being reported as one"
    );

    let strings = in_english();
    let cannot_said = cannot.instead(&strings).unwrap();
    let would_not_said = would_not_say.instead(&strings).unwrap();
    assert_ne!(
        cannot_said.text(),
        would_not_said.text(),
        "two different facts must not read as the same sentence"
    );
}

/// **This machine says it cannot keep an undo**, which is the true answer on
/// every machine this repository has: a person's home is not a subvolume, so
/// there is nothing for a snapshot to be of.
///
/// This is the arm the first reader will see, and it is deliberately the one
/// wired in.
#[test]
fn this_machine_answers_not_on_this_machine() {
    assert_eq!(
        NoUndoHere.what_undo_is_holding(),
        UndoIsHolding::NotOnThisMachine
    );
}

/// **Where the bytes were read travels with them**, the way it does for every
/// other number in this crate, so a person can check the figure against the
/// machine rather than taking it.
#[test]
fn bytes_name_where_they_were_read() {
    let bytes = UndoIsHolding::Bytes {
        value: 1,
        from: Source::of("/var/home/.snapshots", "undo"),
    };
    let from = bytes.from().expect("bytes name their source");
    assert_eq!(from.file(), std::path::Path::new("/var/home/.snapshots"));
    assert_eq!(from.field, "undo");

    assert!(
        UndoIsHolding::NotOnThisMachine.from().is_none(),
        "there is no file to name when the machine keeps no undo"
    );
}

/// **The tree is not touched by any of it.**
///
/// A snapshot's bytes are shared with the live files by copy-on-write, so a
/// node for one would double-count every unchanged file. The line sits beside
/// the tree, and this is the test that says the arithmetic never learned about
/// it: the same folder counted with two different undo answers gives two
/// identical trees.
#[cfg(target_os = "linux")]
#[test]
fn no_size_in_the_tree_moves_because_of_it() {
    use alo_measuring::Holding;

    /// A machine keeping a great deal.
    struct KeepingPlenty;
    impl WhatUndoIsHolding for KeepingPlenty {
        fn what_undo_is_holding(&self) -> UndoIsHolding {
            UndoIsHolding::Bytes {
                value: 999_999_999,
                from: Source::of("/var/home/.snapshots", "undo"),
            }
        }
    }

    let folder =
        std::env::temp_dir().join(format!("alo-undo-line-{}-{}", std::process::id(), line!()));
    std::fs::create_dir_all(folder.join("inside")).unwrap();
    std::fs::write(folder.join("inside").join("one.txt"), b"some bytes").unwrap();

    let keeping_none = Holding::of(&folder, &NoUndoHere).unwrap();
    let keeping_plenty = Holding::of(&folder, &KeepingPlenty).unwrap();

    assert_eq!(
        keeping_none.tree, keeping_plenty.tree,
        "the tree must be identical whatever undo is holding"
    );
    assert_eq!(keeping_none.undo, UndoIsHolding::NotOnThisMachine);
    assert_eq!(keeping_plenty.undo.value(), Some(999_999_999));

    std::fs::remove_dir_all(&folder).unwrap();
}
