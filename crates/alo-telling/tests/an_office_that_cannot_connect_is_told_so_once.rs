//! *The whole of it works with no internet at all* — and what a person in
//! that office is told about their machine's connectedness is true, and said
//! once.
//!
//! The measurement itself is `alo-asking`'s, inside a network namespace with
//! no route out of it. This file is the half that is `alo-telling`'s: given
//! the failure the kernel produces there, what reaches a person, and how
//! often — against the vocabulary a real machine holds, because the word is
//! new and a word nothing collects is a key on somebody's screen.
//!
//! | The acceptance | The test |
//! |---|---|
//! | what a person is told is true: it says there is no way there, not that the provider failed, not that the machine is offline | [`what_a_person_is_told_about_having_no_way_there_is_true`] |
//! | and it is said once | [`having_no_way_there_is_said_once`] |
//! | the studio down the corridor is offered, and nothing here takes it | [`the_machine_down_the_corridor_is_offered_and_never_chosen`] |
//! | the same fact about a different place is a different telling | [`no_way_to_a_different_place_is_a_different_telling`] |

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_answering::{Answering, Failed, WentWrong};
use alo_models::{InferenceSource, Region, SourcePolicy};
use alo_strings::Strings;
use alo_telling::{Tell, Telling, WhoAsked};

/// A provider somebody set up before the office lost its connection.
fn a_provider() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// A second provider, for the telling that is about a different place.
fn another_provider() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "alo".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// The machine down the corridor, paired, with the GPU in it.
fn the_studio() -> InferenceSource {
    InferenceSource::PairedMachine {
        machine: "the studio machine".to_owned(),
    }
}

/// A question put at this place, refused by the kernel for want of a route,
/// on a machine that also has these places and forbids none of them.
fn no_way_to(source: InferenceSource, others: &[InferenceSource]) -> Failed {
    Answering::chosen(source, &SourcePolicy::Anywhere)
        .unwrap()
        .did_not_answer(WentWrong::NoWayThere, others, &SourcePolicy::Anywhere)
        .unwrap()
}

/// Everything a real machine can say, which is what a shell holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// **What a person is told is true.** The line about what happened says
/// there is no way there from this network and that the question was not
/// sent — not that the provider did not answer, which it was never asked, and
/// not that the machine is offline, which the machine cannot know.
#[test]
fn what_a_person_is_told_about_having_no_way_there_is_true() {
    let strings = what_this_machine_can_say();
    let mut telling = Telling::nothing_said_yet();
    let tell = telling.about(no_way_to(a_provider(), &[]), WhoAsked::ThePerson);
    let told = tell.to_say().expect("nobody has been told this yet");

    let [heading, what_happened, nothing_was_sent, carry_on] = told.lines(&strings);
    assert_eq!(heading.text(), "The agent cannot answer right now");
    assert_eq!(
        what_happened.text(),
        "nothing was answered by Mistral, in the EU — there is no way there from the network \
         this machine is on, so the question was not sent"
    );
    assert_eq!(
        nothing_was_sent.text(),
        "nothing was sent anywhere, and nothing will be unless you say so"
    );
    assert!(
        carry_on.text().starts_with("You can carry on."),
        "{carry_on}"
    );

    for line in [&heading, &what_happened, &nothing_was_sent, &carry_on] {
        assert!(!line.is_a_bug(), "a key reached the person: {line}");
        for claim in ["offline", "internet", "disconnected", "did not answer"] {
            assert!(
                !line.text().to_ascii_lowercase().contains(claim),
                "the machine claimed something that is not so: {line}"
            );
        }
    }
}

/// **And it is said once.** The office has no route out all day; the machine
/// retries by itself and says nothing; the person asking again is answered,
/// because an answer to a question is not a reminder.
#[test]
fn having_no_way_there_is_said_once() {
    let mut telling = Telling::nothing_said_yet();
    assert!(
        telling
            .about(no_way_to(a_provider(), &[]), WhoAsked::ThePerson)
            .was_said()
    );
    for _ in 0..20 {
        assert_eq!(
            telling.about(no_way_to(a_provider(), &[]), WhoAsked::TheMachine),
            Tell::SaidAlready,
            "the machine nagged about having no way out"
        );
    }
    assert!(
        telling
            .about(no_way_to(a_provider(), &[]), WhoAsked::ThePerson)
            .was_said(),
        "the person asked again and was not answered"
    );
    assert_eq!(telling.how_many_it_remembers(), 1);
}

/// **The studio is offered, and nothing here takes it.** An office with no
/// internet has working AI because a person can send the question down the
/// corridor — and *can* is the word: the offer is theirs to answer, and a
/// telling that took it for them would be the silent fallback at the moment
/// it is most tempting.
#[test]
fn the_machine_down_the_corridor_is_offered_and_never_chosen() {
    let mut telling = Telling::nothing_said_yet();
    let Tell::Say(told) = telling.about(
        no_way_to(a_provider(), &[the_studio()]),
        WhoAsked::ThePerson,
    ) else {
        unreachable!("a fresh session has said nothing to anybody")
    };

    assert_eq!(told.elsewhere().offers().len(), 1);
    let offer = told.elsewhere().offers().first().cloned().unwrap();
    assert_eq!(offer.source(), &the_studio());

    // Taking it is still a person's act, and it is worth one question.
    let answering = told.take(&offer).unwrap();
    assert_eq!(answering.source(), &the_studio());
}

/// **The same fact about a different place is a different telling.** Having
/// no way to one provider and no way to another are two sentences naming two
/// places, and a person who added a second provider is told about it — the
/// rule this crate keeps for every reason, kept for this one.
#[test]
fn no_way_to_a_different_place_is_a_different_telling() {
    let mut telling = Telling::nothing_said_yet();
    assert!(
        telling
            .about(no_way_to(a_provider(), &[]), WhoAsked::ThePerson)
            .was_said()
    );
    assert!(
        telling
            .about(no_way_to(another_provider(), &[]), WhoAsked::TheMachine)
            .was_said(),
        "a different place was suppressed as the same telling"
    );
    assert_eq!(telling.how_many_it_remembers(), 2);
}
