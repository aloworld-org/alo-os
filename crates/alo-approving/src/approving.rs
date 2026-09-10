//! One change in front of a person, and the one answer it gets.
//!
//! [`Asked`] is one moment. This is the thing that lives as long as somebody is
//! looking at the screen: it remembers which change was put to them, carries
//! their answer to the turn that proposed it, and forgets it in the same act.
//!
//! # Three promises, and the middle one is the whole task
//!
//! - **Nothing is ever shown that was not a change the machine checked.**
//!   [`Approving::ask`] takes an `alo_turn::Turning` and a number, and makes the
//!   [`Asked`] itself; there is no way to hand it one.
//! - **One answer, one execution.** Answering takes the question off the screen
//!   before the turn is touched, so a second click finds nothing to answer and
//!   never reaches `alo_turn::Turning::approving` at all. The list underneath
//!   holds the same rule — `alo_capability::Approvals` takes a proposal off in
//!   the act of answering it — so a change is carried out once even if a shell,
//!   a keyboard and a stuck mouse button all answer at the same moment.
//! - **Nowhere to put it is a sentence, not silence.** A change that could not
//!   be put to anybody, and that said nothing about it, is an agent that looks
//!   as though it ignored an instruction.
//!
//! # Why answering forgets, even when the answer failed
//!
//! [`Approving::approve`] and [`Approving::decline`] take the question off the
//! screen whichever way the turn answers. That is deliberate: the question has
//! been answered, and what came back — a grant revoked since it was asked, a
//! disk that could not, a question that stood too long — is a fact about *that*
//! answer rather than a reason to leave the same button under somebody's
//! cursor. The list underneath agrees: `Approvals::approve` removes a lapsed
//! proposal as it refuses it, precisely so that nobody is invited to click it
//! again.
//!
//! What a shell does with the refusal is show it, in the words it arrives with.
//! What it must not do is put the question back up: the agent is still there,
//! and asking it again is the person's act.
//!
//! # It shows one question at a time
//!
//! A person answering two changes at once is a person approving the second
//! without reading it. [`Approving::ask`] therefore replaces whatever was on
//! the screen; the change it replaces is untouched and still waiting, because
//! nothing here answers anything a person did not answer.

use std::time::SystemTime;

use alo_capability::{AnswerError, Grants, ProposalId};
use alo_files::Answer;
use alo_turn::Turning;

use crate::asked::Asked;
use crate::refusing::{NotAnswered, NotAsked};
use crate::surface::Compositor;

/// What one call to [`Approving::ask`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asks {
    /// The change is in front of the person, worded as they read it.
    Asked(Asked),
    /// It could not be put to them, and this is what to say instead — in their
    /// language, through [`NotAsked::said`].
    Refused(NotAsked),
}

/// What one answer did.
#[derive(Debug, PartialEq, Eq)]
pub enum Answered {
    /// The person approved it and the machine did it. What came back is the
    /// verb's own answer, handed on whole.
    Carried(Answer),
    /// The person said no. Nothing happened, nothing was asked about why, and
    /// the record says they declined it.
    Declined,
    /// Nothing was carried out, and this is what to say — in the person's
    /// language, through [`NotAnswered::said`].
    Refused(NotAnswered),
}

/// The change waiting for one person's answer, for as long as they are looking
/// at it.
///
/// One per surface a change can be put on; in v0.01 that is one per machine. It
/// holds no compositor, because whether there is one is a fact about the
/// session that can change while the machine runs — so it is handed in at the
/// moment of the call, exactly as the agent overlay's summoning and the egress
/// indicator take one. It holds no turn either: a turn is borrowed for the
/// length of a call and outlives no answer.
///
/// The worked example is on [`crate`], because standing one of these up means
/// standing a whole turn up and that is where the reader of a first page is.
#[derive(Debug, Default)]
pub struct Approving {
    /// The change that is on the screen, or [`None`] when there is none.
    ///
    /// The number rather than the [`Asked`]: what is on the screen is a picture
    /// of a moment, and what an answer needs is the question's identity. A
    /// surface holding the picture could answer a question with a sentence
    /// somebody read a minute ago.
    asking: Option<ProposalId>,
}

impl Approving {
    /// Where every session starts: no change is in front of anybody.
    #[must_use]
    pub fn nothing_to_answer() -> Self {
        Self { asking: None }
    }

    /// Which change is on the screen, if any.
    ///
    /// What the compositor was last told to put up, not a claim about pixels.
    #[must_use]
    pub fn asking(&self) -> Option<ProposalId> {
        self.asking
    }

    /// Whether anything is waiting for this person's answer on this surface.
    #[must_use]
    pub fn is_asking(&self) -> bool {
        self.asking.is_some()
    }

    /// Put one of the turn's changes in front of the person.
    ///
    /// The question is read off the turn and worded with the machine's own
    /// vocabulary, so nothing a caller assembled can reach a screen. A number
    /// that names nothing, and a question that stood too long, are refused in
    /// `alo-capability`'s own words; with no compositor, or one that refuses,
    /// the answer is a refusal with a sentence in it and nothing is left
    /// believing a question is up.
    pub fn ask(
        &mut self,
        compositor: Option<&mut dyn Compositor>,
        turning: &Turning<'_, '_>,
        id: ProposalId,
        now: SystemTime,
    ) -> Asks {
        let Some(compositor) = compositor else {
            self.asking = None;
            return Asks::Refused(NotAsked::NoCompositor);
        };
        let Some(waiting) = turning.proposed(id) else {
            self.asking = None;
            return Asks::Refused(NotAsked::NotWaiting(AnswerError::NothingWaiting {
                number: id.as_u64(),
            }));
        };
        let Some(asked) = Asked::of(waiting, turning.strings(), now) else {
            self.asking = None;
            return Asks::Refused(NotAsked::NotWaiting(AnswerError::Lapsed {
                call: Box::new(waiting.proposal.call().clone()),
            }));
        };
        match compositor.ask(asked.clone()) {
            Ok(()) => {
                self.asking = Some(id);
                Asks::Asked(asked)
            }
            Err(refused) => {
                self.asking = None;
                Asks::Refused(NotAsked::Surface(refused))
            }
        }
    }

    /// The person approved what is in front of them, so it runs — once.
    ///
    /// The question comes off the screen before the turn is touched, so a
    /// second answer refuses without reaching it. What happens then is
    /// `alo_turn::Turning::approving`'s: the grants are asked again at the
    /// moment of execution, the work runs inside the machine's boundary, and
    /// the record is written before anything comes back here.
    pub fn approve(
        &mut self,
        turning: &mut Turning<'_, '_>,
        grants: &Grants,
        now: SystemTime,
    ) -> Answered {
        let Some(id) = self.asking.take() else {
            return Answered::Refused(NotAnswered::NothingToAnswer);
        };
        match turning.approving(id, grants, now) {
            Ok(answer) => Answered::Carried(answer),
            Err(why) => Answered::Refused(NotAnswered::Turn(why)),
        }
    }

    /// The person said no.
    ///
    /// Nothing is kept about why: *no* is the whole answer, and a system that
    /// recorded a reason would be a system that asked for one. The record still
    /// says they declined it, because a refusal is a thing that happened.
    pub fn decline(&mut self, turning: &mut Turning<'_, '_>, now: SystemTime) -> Answered {
        let Some(id) = self.asking.take() else {
            return Answered::Refused(NotAnswered::NothingToAnswer);
        };
        match turning.declining(id, now) {
            Ok(()) => Answered::Declined,
            Err(why) => Answered::Refused(NotAnswered::Turn(why)),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being               reported"
)]
mod tests {
    use super::*;
    use crate::surface::SurfaceRefused;
    use crate::testing::{Screen, an_invoice, another_invoice, hour, noon, on_a_machine};
    use alo_turn::NotDone;

    /// **A change is put to the person as the sentence the turn renders**, and
    /// the compositor is handed that sentence rather than left to write one.
    #[test]
    fn a_change_is_put_to_the_person_as_the_sentence_the_turn_renders() {
        on_a_machine(|turning, grants, strings, places| {
            let id = an_invoice(turning, grants, places);
            let mut screen = Screen::with_a_screen();
            let mut approving = Approving::nothing_to_answer();
            assert!(!approving.is_asking());

            let Asks::Asked(asked) = approving.ask(Some(&mut screen), turning, id, noon()) else {
                panic!("a change that was waiting was not put to anybody");
            };
            assert_eq!(approving.asking(), Some(id));
            assert_eq!(screen.asked, 1);
            assert_eq!(screen.last.as_ref().unwrap(), &asked);
            assert_eq!(
                asked.sentence(),
                &turning.proposed(id).unwrap().proposal.sentence(strings)
            );
            assert_eq!(asked.lapses_in(), hour());
        });
    }

    /// **One answer, one execution.** The question comes off the screen as it
    /// is answered, so the second click never reaches the turn — and the change
    /// happened once.
    #[test]
    fn one_answer_runs_it_once_and_a_second_answer_runs_nothing() {
        on_a_machine(|turning, grants, _, places| {
            let id = an_invoice(turning, grants, places);
            let mut screen = Screen::with_a_screen();
            let mut approving = Approving::nothing_to_answer();
            assert!(matches!(
                approving.ask(Some(&mut screen), turning, id, noon()),
                Asks::Asked(_)
            ));

            assert!(matches!(
                approving.approve(turning, grants, noon()),
                Answered::Carried(_)
            ));
            assert!(!approving.is_asking());

            assert_eq!(
                approving.approve(turning, grants, noon()),
                Answered::Refused(NotAnswered::NothingToAnswer),
                "a second answer reached the turn"
            );
        });
    }

    /// **A question that stood too long is refused rather than carried out**,
    /// and the refusal is `alo-capability`'s own — the one that quotes the
    /// change so the person can ask for it again.
    #[test]
    fn a_question_that_stood_too_long_is_refused_when_it_is_answered() {
        on_a_machine(|turning, grants, strings, places| {
            let id = an_invoice(turning, grants, places);
            let mut screen = Screen::with_a_screen();
            let mut approving = Approving::nothing_to_answer();
            assert!(matches!(
                approving.ask(Some(&mut screen), turning, id, noon()),
                Asks::Asked(_)
            ));

            let late = noon() + hour();
            let Answered::Refused(why) = approving.approve(turning, grants, late) else {
                panic!("a question that had lapsed was carried out");
            };
            assert!(matches!(
                why,
                NotAnswered::Turn(NotDone::NotAnswered(AnswerError::Lapsed { .. }))
            ));
            let said = why.said(strings);
            assert!(said.text().contains("ask again"), "{said}");
            assert!(!approving.is_asking());
        });
    }

    /// **And it is not put in front of anybody once it has lapsed either**, so
    /// nobody is offered an answer the machine will refuse.
    #[test]
    fn a_question_that_stood_too_long_is_never_put_up() {
        on_a_machine(|turning, grants, strings, places| {
            let id = an_invoice(turning, grants, places);
            let mut screen = Screen::with_a_screen();
            let mut approving = Approving::nothing_to_answer();

            let Asks::Refused(why) = approving.ask(Some(&mut screen), turning, id, noon() + hour())
            else {
                panic!("a question that had lapsed was put to somebody");
            };
            assert!(matches!(
                why,
                NotAsked::NotWaiting(AnswerError::Lapsed { .. })
            ));
            assert!(why.said(strings).text().contains("ask again"));
            assert_eq!(screen.asked, 0, "a lapsed question reached the compositor");
            assert!(!approving.is_asking());
        });
    }

    /// **A number that names nothing is refused in words**, which is what a
    /// surface showing yesterday's list meets.
    #[test]
    fn a_number_that_names_nothing_is_refused_in_words() {
        on_a_machine(|turning, grants, strings, places| {
            let id = an_invoice(turning, grants, places);
            let mut screen = Screen::with_a_screen();
            let mut approving = Approving::nothing_to_answer();
            assert!(matches!(
                approving.ask(Some(&mut screen), turning, id, noon()),
                Asks::Asked(_)
            ));
            assert!(matches!(
                approving.decline(turning, noon()),
                Answered::Declined
            ));

            // Answered and gone: asking for it again finds nothing.
            let Asks::Refused(why) = approving.ask(Some(&mut screen), turning, id, noon()) else {
                panic!("a question nobody is waiting on was put up");
            };
            assert!(matches!(
                why,
                NotAsked::NotWaiting(AnswerError::NothingWaiting { .. })
            ));
            assert!(why.said(strings).text().contains("answered already"));
            assert!(!approving.is_asking());
        });
    }

    /// **Nowhere to put it is a refusal, not silence**, and nothing is left
    /// believing a question is on a screen.
    #[test]
    fn nowhere_to_put_it_refuses_in_words_and_believes_nothing() {
        on_a_machine(|turning, grants, strings, places| {
            let id = an_invoice(turning, grants, places);
            let mut approving = Approving::nothing_to_answer();

            // Nothing is drawing a screen at all.
            let refused = approving.ask(None, turning, id, noon());
            assert_eq!(refused, Asks::Refused(NotAsked::NoCompositor));
            assert!(!approving.is_asking());

            // Something is drawing, and there is no display to draw on.
            let mut nowhere = Screen::with_no_screen();
            let refused = approving.ask(Some(&mut nowhere), turning, id, noon());
            assert_eq!(
                refused,
                Asks::Refused(NotAsked::Surface(SurfaceRefused::NothingToShowOn))
            );
            assert_eq!(nowhere.asked, 1);
            assert!(!approving.is_asking());

            // Neither of them answered anything, so the change is still waiting
            // and a screen that arrives can put it up.
            let mut screen = Screen::with_a_screen();
            assert!(matches!(
                approving.ask(Some(&mut screen), turning, id, noon()),
                Asks::Asked(_)
            ));
            assert!(approving.is_asking());
            for said in [
                NotAsked::NoCompositor.said(strings),
                NotAsked::Surface(SurfaceRefused::NothingToShowOn).said(strings),
            ] {
                assert!(!said.is_a_bug(), "{said}");
            }
        });
    }

    /// **Answering with nothing in front of you carries nothing out**, and says
    /// so — on both answers, because a shell that lost its surface can send
    /// either.
    #[test]
    fn answering_with_nothing_in_front_of_you_carries_nothing_out() {
        on_a_machine(|turning, grants, strings, _| {
            let mut approving = Approving::nothing_to_answer();
            assert_eq!(
                approving.approve(turning, grants, noon()),
                Answered::Refused(NotAnswered::NothingToAnswer)
            );
            assert_eq!(
                approving.decline(turning, noon()),
                Answered::Refused(NotAnswered::NothingToAnswer)
            );
            let said = NotAnswered::NothingToAnswer.said(strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.text().contains("Ask the agent"), "{said}");
        });
    }

    /// **A second question replaces the first, and answers nothing.** A person
    /// looking at two changes at once approves the second without reading it;
    /// the one that was replaced is untouched and still waiting.
    #[test]
    fn a_second_question_replaces_the_first_without_answering_it() {
        on_a_machine(|turning, grants, _, places| {
            let march = an_invoice(turning, grants, places);
            let april = another_invoice(turning, grants, places);
            let mut screen = Screen::with_a_screen();
            let mut approving = Approving::nothing_to_answer();

            assert!(matches!(
                approving.ask(Some(&mut screen), turning, march, noon()),
                Asks::Asked(_)
            ));
            assert!(matches!(
                approving.ask(Some(&mut screen), turning, april, noon()),
                Asks::Asked(_)
            ));
            assert_eq!(approving.asking(), Some(april));

            // The first is still waiting for somebody to answer it.
            assert!(turning.proposed(march).is_some());
            assert_eq!(turning.waiting_at(noon()).count(), 2);
        });
    }

    /// A fresh [`Approving`] has nothing in front of anybody, whichever way it
    /// is built.
    #[test]
    fn a_fresh_approving_has_nothing_in_front_of_anybody() {
        assert!(!Approving::default().is_asking());
        assert!(!Approving::nothing_to_answer().is_asking());
        assert_eq!(Approving::nothing_to_answer().asking(), None);
    }
}
