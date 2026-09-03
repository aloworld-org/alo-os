//! The whole journey, from a question that was not answered to what left this
//! machine — or, in the case this crate exists for, to nothing having left.
//!
//! The crate's own tests take one decision at a time. This is the other half:
//! `alo-answering` beside `alo-egress` and `alo-record`, which is the
//! arrangement a daemon has, asking the question a security reviewer asks —
//! *when the local model failed, what left, and what does the record say?*
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! asked a model anything: there is no daemon, no runtime answering and no
//! socket. What is asserted is that the types cannot be assembled into a
//! machine that falls back, and that when a person says yes the departure is an
//! ordinary one.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_answering::{Answering, WentWrong, answering_words};
use alo_capability::Grantee;
use alo_egress::{DestinationError, EgressPolicy, Indicator, Leaving, Why};
use alo_models::{InferenceSource, Region, SourcePolicy};
use alo_record::{Only, Record};
use alo_strings::Strings;

/// A fixed moment, so an indicator and a record are arithmetic rather than a
/// wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The agent whose question this is.
fn mail() -> Grantee {
    Grantee::named("@mail")
}

/// A provider somebody added, which has said where it runs.
fn alo() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "alo".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// This crate's words beside the ones its sentences borrow.
fn strings() -> Strings {
    let mut vocabulary = answering_words().unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// **The promise, as a test.** A local model fails, a provider is set up and
/// permitted — and nothing leaves, nothing is shown on the indicator, and the
/// record has nothing in it. The offer is sitting there unanswered, which is
/// the whole difference between this system and one that falls back.
#[test]
fn a_local_model_that_failed_sends_nothing_and_leaves_no_record_of_anything() {
    let indicator = Indicator::default();
    let record = Record::default();

    let failed = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
        .unwrap()
        .did_not_answer(WentWrong::NothingUsable, &[alo()], &SourcePolicy::Anywhere)
        .unwrap();

    // There is somewhere to ask, and a person has not been asked yet.
    assert_eq!(failed.elsewhere().offers().len(), 1);

    // Nothing is on the indicator, because nothing is leaving.
    assert!(indicator.is_quiet());
    assert_eq!(indicator.showing().len(), 0);

    // And there is nothing to write down: a question answered nowhere is not
    // something an agent did and not something that left.
    assert!(record.is_empty());
    let anything = alo_record::Asking::anything();
    assert_eq!(record.answering(&anything).count(), 0);

    // The place that failed cannot be turned into a departure at all — a
    // question that was to be answered here has nowhere to go.
    assert_eq!(
        Leaving::asking(&mail(), failed.source()).unwrap_err(),
        DestinationError::NothingLeaves
    );

    // And the person is told, in as many words, that nothing happened instead.
    assert_eq!(
        failed.nothing_was_sent(&strings()).text(),
        "nothing was sent anywhere, and nothing will be unless you say so"
    );
}

/// **And when a person says yes, it is an ordinary departure.** Taking an offer
/// does not open a private road: the question goes through the indicator like
/// everything else an agent causes, and afterwards it is one line in the record
/// naming where it went.
#[test]
fn an_offer_a_person_took_leaves_through_the_indicator_and_into_the_record() {
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let strings = strings();

    let failed = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
        .unwrap()
        .did_not_answer(
            WentWrong::NothingAnswered,
            &[alo()],
            &SourcePolicy::Anywhere,
        )
        .unwrap();

    // What the person reads before they answer: where it would go, and that it
    // is worth one question.
    let offer = failed.elsewhere().offers().first().cloned().unwrap();
    assert_eq!(
        offer.said(&strings).text(),
        "have this question answered by alo, in the EU instead, just this once — the question \
         would leave this machine and the building"
    );

    // They say yes. That is worth one attempt, at that place.
    let elsewhere = failed.take(&offer).unwrap();
    assert_eq!(elsewhere.source(), &alo());
    assert!(elsewhere.causes_egress());

    // Which is a departure like any other: the indicator decides and shows in
    // one call, and it is the only thing that makes the token to open a
    // connection with.
    let leaving = Leaving::asking(&mail(), elsewhere.source()).unwrap();
    assert_eq!(leaving.why(), Why::Asking);
    let departing = indicator
        .beginning(&EgressPolicy::Anywhere, leaving, noon())
        .unwrap();
    assert_eq!(
        indicator
            .showing()
            .first()
            .map(|shown| shown.said(&strings).into_text()),
        Some("@mail is asking a question of alo, in the EU".to_owned())
    );

    // And afterwards it is in the record, once, as egress.
    record.keep(alo_record::Entry::left(&departing));
    indicator.ended(departing);
    assert!(indicator.is_quiet());

    let only_egress = alo_record::Asking::anything().only(Only::Egress);
    let egress: Vec<_> = record.answering(&only_egress).collect();
    assert_eq!(egress.len(), 1);
    assert_eq!(record.len(), 1);

    // The record says where it went and nothing about what was asked.
    let written = format!("{:?}", egress.first());
    assert!(written.contains("alo"), "{written}");
}

/// **A machine whose rule closed the door offers nothing, and says which rule.**
/// Both doors are shut, and the second one matters most: a person cannot reach
/// a provider by choosing it directly either, so the offer being absent is not
/// something a shell could work around by asking again.
#[test]
fn a_machine_that_keeps_questions_in_the_building_offers_nothing_and_says_why() {
    let strings = strings();
    let failed = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::InTheBuilding)
        .unwrap()
        .did_not_answer(
            WentWrong::NoModelThere,
            &[alo()],
            &SourcePolicy::InTheBuilding,
        )
        .unwrap();

    assert!(failed.elsewhere().is_nowhere());
    assert!(!failed.elsewhere().nothing_else());

    let lines = failed.elsewhere().lines(&strings);
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines.first().map(|line| line.text()),
        Some(
            "this machine is set to keep questions in the building, and by alo, in the EU would \
             send this one outside it"
        )
    );

    // The other door is shut too, and by the same rule in the same words.
    let refused = Answering::chosen(alo(), &SourcePolicy::InTheBuilding).unwrap_err();
    assert_eq!(
        refused.said(&strings).text(),
        lines
            .first()
            .map(alo_strings::Said::text)
            .unwrap_or_default()
    );
}

/// **The corridor is a departure too**, and the offer says so before somebody
/// approves it — *it only went down the corridor* is exactly the exception that
/// would erode law 1.
#[test]
fn asking_the_machine_in_the_next_room_is_still_something_leaving() {
    let strings = strings();
    let studio = InferenceSource::PairedMachine {
        machine: "the studio workstation".to_owned(),
    };
    let failed = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::InTheBuilding)
        .unwrap()
        .did_not_answer(
            WentWrong::TookTooLong,
            std::slice::from_ref(&studio),
            &SourcePolicy::InTheBuilding,
        )
        .unwrap();

    let offer = failed.elsewhere().offers().first().cloned().unwrap();
    assert!(offer.causes_egress());
    assert!(
        offer
            .said(&strings)
            .text()
            .contains("would leave this machine and stay on your network"),
        "{}",
        offer.said(&strings)
    );

    let elsewhere = failed.take(&offer).unwrap();
    let mut indicator = Indicator::default();
    let departing = indicator
        .beginning(
            &EgressPolicy::InTheBuilding,
            Leaving::asking(&mail(), elsewhere.source()).unwrap(),
            noon(),
        )
        .unwrap();
    assert_eq!(indicator.showing().len(), 1);
    assert_eq!(departing.why(), Why::Asking);
}
