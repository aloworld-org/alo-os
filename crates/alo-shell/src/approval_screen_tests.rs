//! The approval surface, answered and left alone on a real turn.
//!
//! Every change here is proposed through `alo_turn::Turning` against the file
//! verbs, answered through the screen, and carried out on a real disk — so
//! *one answer, one change* is a file that moved once, and *silence answers
//! nothing* is a file that did not move and a record with nothing in it.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use super::*;
use crate::approval_testing::{
    ANoSpaceLeftDisk, archiving, archiving_for, hour, keeping_in, noon, on_a_machine, words,
};
use alo_approving::SurfaceRefused;
use alo_capability::AnswerError;
use alo_turn::NotDone;
use std::time::Duration;

/// The question on the screen, or a failure naming what was there instead.
fn question(screen: &ApprovalScreen) -> (Asked, Option<ApprovalAnswer>) {
    match screen.shows() {
        ApprovalShows::Question { asked, selected } => (asked.clone(), selected),
        other => panic!("no question is on the screen: {other:?}"),
    }
}

/// **Nothing is preselected, and Enter on a fresh question answers nothing.**
/// A question goes up with neither answer selected, so the Enter held down
/// from the last dialogue, or pressed for something else, carries nothing out
/// and declines nothing.
#[test]
fn nothing_is_preselected_and_enter_on_a_fresh_question_answers_nothing() {
    let (record, places) = on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        assert_eq!(
            screen.arrived(turning, march, noon()),
            ApprovalOutcome::default()
        );
        let (asked, selected) = question(&screen);
        assert_eq!(asked.id(), march);
        assert_eq!(selected, None, "an answer was preselected");

        for _ in 0..3 {
            assert_eq!(
                screen.pressed(ApprovalKey::Choose, turning, grants, noon()),
                ApprovalOutcome::default(),
                "Enter with nothing selected gave an answer"
            );
        }
        assert_eq!(question(&screen), (asked, None));
        assert!(turning.proposed(march).is_some());
    });
    assert!(places.invoice("march").exists());
    assert!(
        record.is_empty(),
        "an unanswered question left evidence of an answer"
    );
}

/// **A surface left alone answers nothing.** Frames are drawn, time passes
/// right up to the moment the question lapses, keys that mean nothing arrive —
/// and the question is still waiting, nothing is selected, the file has not
/// moved and the record holds no answer. When it does lapse, it lapses in
/// `alo-capability`, which carries nothing out either.
#[test]
fn a_surface_left_alone_answers_nothing() {
    let (record, places) = on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());

        let minute = Duration::from_secs(60);
        for minutes in 0..60 {
            let now = noon() + minute * minutes;
            assert_eq!(
                screen.looked_again(turning, now),
                ApprovalOutcome::default()
            );
            assert_eq!(
                screen.pressed(ApprovalKey::Nothing, turning, grants, now),
                ApprovalOutcome::default()
            );
            let (asked, selected) = question(&screen);
            assert_eq!(asked.id(), march);
            assert_eq!(selected, None);
        }
        assert!(turning.proposed(march).is_some(), "silence answered it");

        // Left alone past its hour, it comes down with the reason it went —
        // and still nothing was carried out.
        let late = noon() + hour();
        assert_eq!(
            screen.looked_again(turning, late),
            ApprovalOutcome::default()
        );
        let ApprovalShows::Refusal(said) = screen.shows() else {
            panic!("a lapsed question stayed up: {:?}", screen.shows());
        };
        assert!(said.text().contains("ask again"), "{said}");
        assert!(!said.is_a_bug(), "{said}");
    });
    assert!(places.invoice("march").exists(), "silence moved a file");
    assert!(!places.archived("march").exists());
    assert!(record.is_empty(), "silence left evidence of an answer");
}

/// **One answer carries one change out, and a second answer to the same
/// question is refused by `alo-approving`, not by this screen.** The second
/// press reaches `Approving` and comes back as its own
/// `NotAnswered::NothingToAnswer`; the file moved once and the record holds one
/// execution.
#[test]
fn one_answer_carries_one_change_and_a_second_is_refused_there_rather_than_here() {
    let (record, places) = on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());

        let first = screen.answer(ApprovalAnswer::Approve, turning, grants, noon());
        assert!(
            matches!(first.answered, Some(Answered::Carried(_))),
            "{first:?}"
        );
        assert_eq!(screen.shows(), ApprovalShows::Nothing);

        let second = screen.answer(ApprovalAnswer::Approve, turning, grants, noon());
        assert_eq!(
            second.answered,
            Some(Answered::Refused(NotAnswered::NothingToAnswer)),
            "a second answer was not refused by alo-approving"
        );
        let third = screen.answer(ApprovalAnswer::No, turning, grants, noon());
        assert_eq!(
            third.answered,
            Some(Answered::Refused(NotAnswered::NothingToAnswer))
        );
        // A stale answer has nothing to show about it.
        assert_eq!(screen.shows(), ApprovalShows::Nothing);
    });
    assert!(!places.invoice("march").exists());
    assert!(places.archived("march").exists());
    assert_eq!(record.len(), 1, "one approval was not one execution");
}

/// **Tab, then Enter, answers** — and the keys choose between exactly two
/// answers, *no* first in reading order, with Shift+Tab going the other way.
#[test]
fn tab_selects_between_exactly_two_answers_and_enter_gives_the_selected_one() {
    let (record, places) = on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());

        let mut seen = Vec::new();
        for _ in 0..4 {
            screen.pressed(ApprovalKey::NextAnswer, turning, grants, noon());
            seen.push(question(&screen).1.unwrap());
        }
        use ApprovalAnswer::{Approve, No};
        assert_eq!(seen, [No, Approve, No, Approve]);

        let mut backwards = ApprovalScreen::on_an_output();
        let april = archiving(turning, grants, places, "april");
        backwards.arrived(turning, april, noon());
        backwards.pressed(ApprovalKey::PreviousAnswer, turning, grants, noon());
        assert_eq!(question(&backwards).1, Some(Approve));
        backwards.pressed(ApprovalKey::PreviousAnswer, turning, grants, noon());
        assert_eq!(question(&backwards).1, Some(No));

        // `screen` has Approve selected.
        let outcome = screen.pressed(ApprovalKey::Choose, turning, grants, noon());
        assert!(matches!(outcome.answered, Some(Answered::Carried(_))));
    });
    assert!(places.archived("march").exists());
    assert!(places.invoice("april").exists());
    assert_eq!(record.len(), 1);
}

/// **No is the whole answer.** Declining carries nothing out, puts no sentence
/// on the screen afterwards and asks for nothing — and the record says the
/// person declined, with no reason in it because none was asked for.
#[test]
fn declining_carries_nothing_out_and_says_nothing_about_why() {
    let (record, places) = on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        screen.pressed(ApprovalKey::NextAnswer, turning, grants, noon());
        assert_eq!(question(&screen).1, Some(ApprovalAnswer::No));

        let outcome = screen.pressed(ApprovalKey::Choose, turning, grants, noon());
        assert_eq!(outcome.answered, Some(Answered::Declined));
        assert!(outcome.nowhere_to_show.is_empty());
        assert_eq!(
            screen.shows(),
            ApprovalShows::Nothing,
            "something was said after a no"
        );
        assert!(!screen.is_open());
        assert!(turning.proposed(march).is_none());
    });
    assert!(places.invoice("march").exists());
    assert!(!places.archived("march").exists());
    assert_eq!(record.len(), 1, "saying no left no evidence");
}

/// **A proposal that arrives while another is open waits behind it**, and is
/// not put up over the question the person is reading. When that question is
/// answered the next goes up — with nothing selected, so the Enter that
/// answered the first cannot answer the second.
#[test]
fn a_proposal_that_arrives_while_another_is_open_waits_behind_it() {
    let (record, places) = on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let april = archiving(turning, grants, places, "april");
        let may = archiving(turning, grants, places, "may");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        screen.arrived(turning, april, noon());
        screen.arrived(turning, may, noon());

        let (asked, _) = question(&screen);
        assert_eq!(asked.id(), march, "a later proposal replaced the open one");
        assert_eq!(screen.waiting_behind(), 2);
        assert!(turning.proposed(april).is_some());

        screen.pressed(ApprovalKey::PreviousAnswer, turning, grants, noon());
        let outcome = screen.pressed(ApprovalKey::Choose, turning, grants, noon());
        assert!(matches!(outcome.answered, Some(Answered::Carried(_))));

        let (asked, selected) = question(&screen);
        assert_eq!(
            asked.id(),
            april,
            "the next question is not the next to arrive"
        );
        assert_eq!(selected, None, "the next question went up preselected");
        assert_eq!(
            screen.pressed(ApprovalKey::Choose, turning, grants, noon()),
            ApprovalOutcome::default(),
            "the Enter that answered one question answered the next"
        );
        assert_eq!(screen.waiting_behind(), 1);

        let outcome = screen.answer(ApprovalAnswer::No, turning, grants, noon());
        assert_eq!(outcome.answered, Some(Answered::Declined));
        assert_eq!(question(&screen).0.id(), may);
        assert_eq!(screen.waiting_behind(), 0);
    });
    assert!(places.archived("march").exists());
    assert!(places.invoice("april").exists());
    assert!(places.invoice("may").exists());
    assert_eq!(record.len(), 2);
}

/// **One proposal is one question**, however many times it is announced: the
/// open one is not queued behind itself, and a kept one is not kept twice.
#[test]
fn the_same_proposal_arriving_twice_is_one_question() {
    on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let april = archiving(turning, grants, places, "april");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        screen.pressed(ApprovalKey::NextAnswer, turning, grants, noon());
        screen.arrived(turning, march, noon());
        screen.arrived(turning, april, noon());
        screen.arrived(turning, april, noon());
        assert_eq!(screen.waiting_behind(), 1);
        assert_eq!(
            question(&screen).1,
            Some(ApprovalAnswer::No),
            "announcing the open question again reset what the person selected"
        );

        screen.answer(ApprovalAnswer::No, turning, grants, noon());
        assert_eq!(question(&screen).0.id(), april);
        screen.answer(ApprovalAnswer::No, turning, grants, noon());
        assert_eq!(screen.shows(), ApprovalShows::Nothing);
    });
}

/// **An answer the machine refused is shown in the refuser's words, and the
/// next question waits until the person has read it.** Here the grants are
/// gone by the moment of execution, which `alo-turn` checks again; the
/// sentence on the screen is that refusal's own, and Enter acknowledges it and
/// puts the next question up.
#[test]
fn a_refused_answer_is_shown_in_the_refusers_words_before_the_next_question() {
    let (_, places) = on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let april = archiving(turning, grants, places, "april");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        screen.arrived(turning, april, noon());

        let revoked = Grants::default();
        let outcome = screen.answer(ApprovalAnswer::Approve, turning, &revoked, noon());
        let Some(Answered::Refused(why)) = &outcome.answered else {
            panic!("an approval with no grant behind it was carried out: {outcome:?}");
        };
        let expected = why.said(turning.strings());
        assert!(!expected.is_a_bug(), "{expected}");
        assert_eq!(screen.shows(), ApprovalShows::Refusal(&expected));

        // Moving between answers means nothing on a refusal, and the second
        // question is not up yet.
        screen.pressed(ApprovalKey::NextAnswer, turning, grants, noon());
        assert_eq!(screen.shows(), ApprovalShows::Refusal(&expected));
        assert_eq!(screen.waiting_behind(), 1);

        assert_eq!(
            screen.pressed(ApprovalKey::Choose, turning, grants, noon()),
            ApprovalOutcome::default()
        );
        let (asked, selected) = question(&screen);
        assert_eq!(asked.id(), april);
        assert_eq!(selected, None);
    });
    assert!(places.invoice("march").exists());
}

/// **A question that lapsed while it waited behind another is never put up.**
/// It becomes `alo-capability`'s sentence for a lapsed question, read before
/// anything else goes up — and the one behind it waits until it has been read.
#[test]
fn a_question_that_lapsed_while_it_waited_is_never_put_up() {
    on_a_machine(|turning, grants, places| {
        let half = Duration::from_secs(30 * 60);
        let march = archiving(turning, grants, places, "march");
        let april = archiving_for(turning, grants, places, "april", half);
        let may = archiving(turning, grants, places, "may");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        screen.arrived(turning, april, noon());
        screen.arrived(turning, may, noon());

        // March is answered after April's half hour has gone.
        let later = noon() + half + Duration::from_secs(60);
        let outcome = screen.answer(ApprovalAnswer::No, turning, grants, later);
        assert_eq!(outcome.answered, Some(Answered::Declined));
        let ApprovalShows::Refusal(said) = screen.shows() else {
            panic!("a lapsed question was put up: {:?}", screen.shows());
        };
        let lapsed = NotAsked::NotWaiting(AnswerError::Lapsed {
            call: Box::new(turning.proposed(april).unwrap().proposal.call().clone()),
        });
        assert_eq!(said, &lapsed.said(turning.strings()));
        assert!(said.text().contains("ask again"), "{said}");
        assert_eq!(screen.waiting_behind(), 1);

        screen.pressed(ApprovalKey::Choose, turning, grants, later);
        assert_eq!(question(&screen).0.id(), may);
    });
}

/// **With nowhere to show a question, every question is refused into the log
/// and none is answered** — `alo-approving`'s refusal, carried whole, and a
/// screen that believes nothing is up.
#[test]
fn with_nowhere_to_show_it_every_question_is_refused_and_none_is_answered() {
    let (record, places) = on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::with_no_output();
        let outcome = screen.arrived(turning, march, noon());
        assert_eq!(outcome.answered, None);
        assert_eq!(
            outcome.nowhere_to_show,
            [NotAsked::Surface(SurfaceRefused::NothingToShowOn)]
        );
        assert!(!screen.is_open());
        assert_eq!(screen.waiting_behind(), 0);

        for key in [ApprovalKey::NextAnswer, ApprovalKey::Choose] {
            screen.pressed(key, turning, grants, noon());
        }
        let stale = screen.answer(ApprovalAnswer::Approve, turning, grants, noon());
        assert_eq!(
            stale.answered,
            Some(Answered::Refused(NotAnswered::NothingToAnswer))
        );
        assert!(
            turning.proposed(march).is_some(),
            "the question was answered"
        );
        for why in &outcome.nowhere_to_show {
            assert!(!why.said(turning.strings()).is_a_bug());
        }
    });
    assert!(places.invoice("march").exists());
    assert!(record.is_empty());
}

/// **A turn that has stopped keeping evidence takes every waiting question
/// with it**: none of them could be carried out, so none is put up after the
/// sentence that says so.
#[test]
fn a_turn_that_stopped_keeping_evidence_takes_every_waiting_question_with_it() {
    let mut disk = ANoSpaceLeftDisk;
    keeping_in(&mut disk, |turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let april = archiving(turning, grants, places, "april");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        screen.arrived(turning, april, noon());

        let outcome = screen.answer(ApprovalAnswer::No, turning, grants, noon());
        let Some(Answered::Refused(why)) = &outcome.answered else {
            panic!("a no the record could not keep was accepted: {outcome:?}");
        };
        assert!(why.is_the_end_of_the_turn(), "{why:?}");
        assert!(turning.is_closed());
        let expected = why.said(turning.strings());
        assert_eq!(screen.shows(), ApprovalShows::Refusal(&expected));
        assert_eq!(screen.waiting_behind(), 0, "questions outlived their turn");

        assert_eq!(
            screen.pressed(ApprovalKey::Choose, turning, grants, noon()),
            ApprovalOutcome::default()
        );
        assert_eq!(
            screen.shows(),
            ApprovalShows::Nothing,
            "a question was put up on a turn that can no longer answer it"
        );
        // And the turn's own word for a closed turn is collected.
        assert!(
            !NotAnswered::Turn(NotDone::TurnClosed)
                .said(&words())
                .is_a_bug()
        );
    });
}

/// **Every sentence the surface can show is in the machine's vocabulary.** The
/// proposal's sentence, both answers and each refusal it can put up are read
/// through the words `alo-saying` collects, and none is a key nobody declared.
#[test]
fn every_sentence_the_surface_can_show_is_in_the_vocabulary() {
    on_a_machine(|turning, grants, places| {
        let strings = turning.strings();
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        let (asked, _) = question(&screen);
        for said in [
            asked.sentence().clone(),
            asked.approve_said(strings),
            asked.no_said(strings),
        ] {
            assert!(!said.is_a_bug(), "{said}");
            assert!(!said.text().is_empty());
        }
        for refusal in [
            NotAsked::NoCompositor.said(strings),
            NotAsked::Surface(SurfaceRefused::NothingToShowOn).said(strings),
            NotAsked::NotWaiting(AnswerError::NothingWaiting { number: 7 }).said(strings),
            NotAnswered::NothingToAnswer.said(strings),
        ] {
            assert!(!refusal.is_a_bug(), "{refusal}");
        }
    });
}
