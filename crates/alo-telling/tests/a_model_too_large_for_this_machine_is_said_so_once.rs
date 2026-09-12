//! A model too large for the memory in this laptop is said so plainly, once —
//! and then run anyway.
//!
//! `docs/features.md`'s sentence at v0.5, as tests, against the vocabulary a
//! real machine holds rather than this crate's own list — because a string
//! declared and not collected reaches a person as a key.
//!
//! | The acceptance | The test |
//! |---|---|
//! | weights larger than the machine's memory are told to the person once | [`weights_larger_than_this_machines_memory_are_said_so_once`] |
//! | and then run: nothing here refuses, and weights that fit are nothing to say | [`nothing_here_refuses_and_weights_that_fit_are_nothing_to_say`] |
//! | the person asking again is answered; other weights or another machine are said | [`saying_it_again_takes_the_person_asking_or_something_changing`] |
//! | every line is in the vocabulary `alo-saying` collects, and none nudges | [`every_line_is_in_the_vocabulary_this_machine_collects_and_none_nudges`] |

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::{Weights, costing::GIGABYTE, words};
use alo_strings::Strings;
use alo_telling::{EVERY_WORD, Warn, Warning, WhoAsked};

/// Weights somebody brought, of this size, measured by nobody.
fn weights_of(id: &str, bytes_on_disk: u64) -> Weights {
    Weights::checked(id, bytes_on_disk).unwrap()
}

/// Everything a real machine can say, which is what a shell holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// **Weights larger than this machine's memory are said so once.** A person
/// chooses forty gigabytes of their own weights on a sixteen gigabyte laptop
/// in the morning and is told, in two lines that both say the weights run.
/// Every question for the rest of the day costs them again and says nothing.
#[test]
fn weights_larger_than_this_machines_memory_are_said_so_once() {
    let strings = what_this_machine_can_say();
    let mut warning = Warning::nothing_said_yet();
    let theirs = weights_of("their-own-70b", 40 * GIGABYTE);

    let first = warning.about(&theirs, 16.0, WhoAsked::ThePerson);
    let warned = first.to_say().unwrap();
    let lines = warned.lines(&strings);
    assert!(
        lines[0].text().contains("will still run them"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].text().contains("will run these weights"),
        "{}",
        lines[1]
    );
    for line in &lines {
        assert!(!line.is_a_bug(), "{line}");
    }

    let mut said_again = 0;
    for _ in 0..100 {
        if warning
            .about(&theirs, 16.0, WhoAsked::TheMachine)
            .was_said()
        {
            said_again += 1;
        }
    }
    assert_eq!(said_again, 0);
    assert_eq!(warning.how_many_it_remembers(), 1);
}

/// **Nothing here refuses, and weights that fit are nothing to say.** There is
/// no `Err` to refuse with: the largest weights on the smallest machine are a
/// sentence, and weights that fit are not even that.
#[test]
fn nothing_here_refuses_and_weights_that_fit_are_nothing_to_say() {
    let mut warning = Warning::nothing_said_yet();
    let enormous = weights_of("enormous", 400 * GIGABYTE);
    let small = weights_of("small", 4 * GIGABYTE);

    assert!(
        warning
            .about(&enormous, 0.0, WhoAsked::ThePerson)
            .was_said()
    );
    for who in [WhoAsked::ThePerson, WhoAsked::TheMachine] {
        assert_eq!(warning.about(&small, 16.0, who), Warn::Fits);
    }
    // Nothing was remembered about the weights that fit.
    assert_eq!(warning.how_many_it_remembers(), 1);
}

/// **Saying it again takes the person asking, or something changing** — other
/// weights, or the same weights on a machine with a different amount of
/// memory.
#[test]
fn saying_it_again_takes_the_person_asking_or_something_changing() {
    let mut warning = Warning::nothing_said_yet();
    let theirs = weights_of("their-own-70b", 40 * GIGABYTE);
    assert!(warning.about(&theirs, 16.0, WhoAsked::ThePerson).was_said());
    assert_eq!(
        warning.about(&theirs, 16.0, WhoAsked::TheMachine),
        Warn::SaidAlready
    );

    // The person asks again: answered, not reminded.
    assert!(warning.about(&theirs, 16.0, WhoAsked::ThePerson).was_said());
    // Other weights: said.
    assert!(
        warning
            .about(
                &weights_of("others", 40 * GIGABYTE),
                16.0,
                WhoAsked::TheMachine
            )
            .was_said()
    );
    // The same weights, docked into more memory that still is not enough: said.
    assert!(
        warning
            .about(&theirs, 32.0, WhoAsked::TheMachine)
            .was_said()
    );
    assert_eq!(warning.how_many_it_remembers(), 3);
}

/// **Every line is in the vocabulary this machine collects, and none of them
/// nudges** toward a catalogued model or away from the person's own.
#[test]
fn every_line_is_in_the_vocabulary_this_machine_collects_and_none_nudges() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for word in EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "the machine cannot say {}",
            word.named()
        );
    }

    let mut warning = Warning::nothing_said_yet();
    let warn = warning.about(
        &weights_of("theirs", 40 * GIGABYTE),
        16.0,
        WhoAsked::ThePerson,
    );
    for line in warn.to_say().unwrap().lines(&Strings::of(vocabulary)) {
        assert!(!line.is_a_bug(), "{line}");
        let read = line.text().to_ascii_lowercase();
        for nudge in words::NUDGES {
            assert!(!read.contains(nudge), "{line} says \"{nudge}\"");
        }
        assert!(!read.contains("catalogue"), "{line}");
    }
}
