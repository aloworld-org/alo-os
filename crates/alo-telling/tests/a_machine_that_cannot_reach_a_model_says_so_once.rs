//! A machine that cannot reach a model says so once, where it happened, and
//! continues.
//!
//! The plan's acceptance for this task, one test each, against the vocabulary a
//! real machine holds rather than against this crate's own list — because the
//! failure that actually happened one floor down was `alo-overlay` declaring
//! nine strings that nothing collected, which every test inside that crate
//! passed.
//!
//! | The acceptance | The test |
//! |---|---|
//! | the same unavailability told once is told once | [`the_same_unavailability_told_once_is_told_once`] |
//! | telling it again takes the source, the reason or the person asking again | [`telling_it_again_takes_something_changing`] |
//! | a different reason is not suppressed | [`a_different_reason_is_a_different_telling`] |
//! | nothing is told until a turn actually failed | [`nothing_is_told_until_a_turn_actually_failed`] |
//! | every string is in the vocabulary `alo-saying` collects | [`every_string_is_in_the_vocabulary_this_machine_collects`] |
//! | it never asks anybody to buy anything and never chooses another source | [`nothing_asks_anybody_to_buy_anything_and_nothing_chooses_another_source`] |

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_answering::{Answering, Failed, WentWrong};
use alo_models::{InferenceSource, Region, SourcePolicy};
use alo_strings::Strings;
use alo_telling::{Tell, Telling, WhoAsked};

/// A provider somebody pays for, which has said where it runs.
fn a_provider() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "alo".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// A question put at this place, which went wrong this way, on a machine that
/// also has these places and forbids none of them.
fn failing(source: InferenceSource, why: WentWrong, others: &[InferenceSource]) -> Failed {
    Answering::chosen(source, &SourcePolicy::Anywhere)
        .unwrap()
        .did_not_answer(why, others, &SourcePolicy::Anywhere)
        .unwrap()
}

/// Everything a real machine can say, which is what a shell holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// **The same unavailability told once is told once.**
///
/// A provider account empties in the morning. The person is told. Everything
/// the machine does by itself for the rest of the day says nothing at all —
/// which is the difference between an operating system and the greyed-out panel
/// ADR 0009 rejected.
#[test]
fn the_same_unavailability_told_once_is_told_once() {
    let mut telling = Telling::nothing_said_yet();

    let first = telling.about(
        failing(a_provider(), WentWrong::RanOut, &[]),
        WhoAsked::ThePerson,
    );
    assert!(first.was_said());

    let mut said_again = 0;
    for _ in 0..100 {
        if telling
            .about(
                failing(a_provider(), WentWrong::RanOut, &[]),
                WhoAsked::TheMachine,
            )
            .was_said()
        {
            said_again += 1;
        }
    }
    assert_eq!(said_again, 0, "the machine said it again unasked");
    assert_eq!(telling.how_many_it_remembers(), 1);
}

/// **Telling it again takes something changing** — the source, the reason, or
/// the person asking again themselves. All three, in one session, each after a
/// run of silence that shows the suppression was really in force.
#[test]
fn telling_it_again_takes_something_changing() {
    let mut telling = Telling::nothing_said_yet();
    let told = |telling: &mut Telling, source, why, who| {
        telling.about(failing(source, why, &[]), who).was_said()
    };

    assert!(told(
        &mut telling,
        a_provider(),
        WentWrong::RanOut,
        WhoAsked::ThePerson
    ));
    assert!(!told(
        &mut telling,
        a_provider(),
        WentWrong::RanOut,
        WhoAsked::TheMachine
    ));

    // The source changed: a different half of somebody's arrangements.
    assert!(told(
        &mut telling,
        InferenceSource::ThisMachine,
        WentWrong::RanOut,
        WhoAsked::TheMachine
    ));

    // The reason changed.
    assert!(told(
        &mut telling,
        a_provider(),
        WentWrong::KeyNotAccepted,
        WhoAsked::TheMachine
    ));
    assert!(!told(
        &mut telling,
        a_provider(),
        WentWrong::KeyNotAccepted,
        WhoAsked::TheMachine
    ));

    // And nothing changed at all, but the person asked again themselves. They
    // are answered: withholding it would be the key that silently does nothing.
    assert!(told(
        &mut telling,
        a_provider(),
        WentWrong::RanOut,
        WhoAsked::ThePerson
    ));
}

/// **A different reason is a different telling and is not suppressed**, because
/// a machine that swallowed the second failure would hide the one that
/// mattered.
///
/// All eight of `alo_answering::WentWrong`'s reasons at one place, none of them
/// suppressed by any of the others, and each one saying a line of its own.
#[test]
fn a_different_reason_is_a_different_telling() {
    let strings = what_this_machine_can_say();
    let mut telling = Telling::nothing_said_yet();
    let mut what_they_read = Vec::new();

    for why in [
        WentWrong::NothingAnswered,
        WentWrong::TookTooLong,
        WentWrong::NothingUsable,
        WentWrong::NoModelThere,
        WentWrong::KeyNotAccepted,
        WentWrong::SentSomewhereElse,
        WentWrong::RanOut,
        WentWrong::HavingTrouble(503),
        // Two statuses from one service are two reasons, because the number is
        // inside the sentence a person reads.
        WentWrong::HavingTrouble(500),
    ] {
        let tell = telling.about(failing(a_provider(), why, &[]), WhoAsked::TheMachine);
        let told = tell
            .to_say()
            .unwrap_or_else(|| panic!("{why:?} was swallowed"));
        what_they_read.push(told.what_happened(&strings).into_text());
    }

    what_they_read.sort_unstable();
    let how_many = what_they_read.len();
    what_they_read.dedup();
    assert_eq!(
        what_they_read.len(),
        how_many,
        "two different failures read identically"
    );
    assert_eq!(telling.how_many_it_remembers(), how_many);
}

/// **Nothing is told at all until a turn actually failed**, so there is no
/// reminder anybody can be followed around by.
///
/// A session that runs all day without a question failing has nothing to show
/// and nothing remembered — and the only door into a telling needs an
/// `alo_answering::Failed`, which only an attempt that did not answer produces.
#[test]
fn nothing_is_told_until_a_turn_actually_failed() {
    let telling = Telling::nothing_said_yet();
    assert_eq!(telling.how_many_it_remembers(), 0);

    // And a failure that has been told about leaves nothing behind to show a
    // second time: the suppressed answer carries no telling at all.
    let mut telling = Telling::nothing_said_yet();
    assert!(
        telling
            .about(
                failing(InferenceSource::ThisMachine, WentWrong::NoModelThere, &[]),
                WhoAsked::ThePerson
            )
            .was_said()
    );
    let again = telling.about(
        failing(InferenceSource::ThisMachine, WentWrong::NoModelThere, &[]),
        WhoAsked::TheMachine,
    );
    assert_eq!(again, Tell::SaidAlready);
    assert!(again.to_say().is_none());
}

/// **Every string this crate says is in the vocabulary `alo-saying` collects**,
/// and so is every string the lines around it come from.
///
/// The question this crate's own tests cannot ask: whether what it declares
/// survives being put beside everybody else's, and whether it is collected at
/// all. A telling read against the machine's real vocabulary is four sentences;
/// one that is not collected is a key where a sentence belongs, at the moment a
/// person's machine has already failed them.
#[test]
fn every_string_is_in_the_vocabulary_this_machine_collects() {
    let strings = what_this_machine_can_say();
    let mut telling = Telling::nothing_said_yet();
    let tell = telling.about(
        failing(a_provider(), WentWrong::RanOut, &[]),
        WhoAsked::ThePerson,
    );
    let told = tell.to_say().expect("nothing has said this yet");

    for line in told.lines(&strings) {
        assert!(!line.is_a_bug(), "{line} is a key rather than a sentence");
        assert!(!line.text().is_empty());
    }

    let lines = told.lines(&strings);
    assert_eq!(lines[0].text(), "The agent cannot answer right now");
    assert_eq!(
        lines[1].text(),
        "nothing was answered by alo, in the EU — the account there has run out, so nothing will \
         be answered until it is paid for, and nothing else about this machine has changed"
    );
    assert_eq!(
        lines[2].text(),
        "nothing was sent anywhere, and nothing will be unless you say so"
    );
    assert_eq!(
        lines[3].text(),
        "You can carry on. Nothing else on this machine depends on the agent, and nothing here is \
         waiting for you to do anything about this"
    );

    // And the two this crate owns are declared once, by this crate, under its
    // own area.
    for word in alo_telling::EVERY_WORD {
        assert_eq!(word.key().area(), "telling");
    }
}

/// **It never asks anybody to buy anything and it never chooses another
/// source** — the constraint the plan sets, in the one case where breaking it
/// would be most tempting and most profitable.
///
/// An account that has run out, with two other places this machine could ask.
/// Nothing in the telling sells anything, and the doors onwards are exactly the
/// doors a broken runtime would have opened: offered, unranked, unchosen, and
/// opened only by a person.
#[test]
fn nothing_asks_anybody_to_buy_anything_and_nothing_chooses_another_source() {
    let strings = what_this_machine_can_say();
    let places = [
        InferenceSource::ThisMachine,
        InferenceSource::PairedMachine {
            machine: "the studio workstation".to_owned(),
        },
    ];

    let mut telling = Telling::nothing_said_yet();
    let ran_out = telling.about(
        failing(a_provider(), WentWrong::RanOut, &places),
        WhoAsked::ThePerson,
    );
    let ran_out = ran_out.to_say().expect("nothing has said this yet");

    // Neither line this crate owns mentions money, an account, or anywhere to
    // get more of either. The line that does is `alo-answering`'s and says what
    // is so rather than what to do about it.
    for line in [
        ran_out.the_agent_cannot_answer(&strings).into_text(),
        ran_out.carry_on(&strings).into_text(),
    ] {
        let read = line.to_ascii_lowercase();
        for selling in ["buy", "credit", "top up", "upgrade", "subscri", "pay"] {
            assert!(!read.contains(selling), "\"{line}\" says \"{selling}\"");
        }
    }

    // Two places are offered and none is chosen: an `Answering` exists nowhere
    // in this program until a person answers one.
    assert_eq!(ran_out.elsewhere().offers().len(), 2);

    // And running out opens no door that a runtime which was simply not running
    // would not have opened.
    let mut otherwise = Telling::nothing_said_yet();
    let broken = otherwise.about(
        failing(a_provider(), WentWrong::NothingAnswered, &places),
        WhoAsked::ThePerson,
    );
    let broken = broken.to_say().expect("nothing has said this yet");
    assert_eq!(ran_out.elsewhere(), broken.elsewhere());
}
