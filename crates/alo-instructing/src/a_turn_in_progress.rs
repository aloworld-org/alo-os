//! What a model is shown on the **second** request of a turn, and every one
//! after it.
//!
//! [`crate::shown_to_a_turn`] shows a model the instructions, the verbs and one
//! request. That is the whole of what a turn could show until now, and it is
//! why every grade in `data/catalogue.toml` is a grade at **one step**: ask,
//! answer, done. Real work is not one step. *Tidy my invoices* is a folder
//! listed, what came back read, and only then a move proposed — and a model
//! that cannot be told what came back cannot take the second step.
//!
//! This is the telling. Nothing else here is new.
//!
//! # The instructions do not change, and that is the point
//!
//! [`Instructions`] is a closed set of two texts, each with a digest, and
//! `Instructions::SHOWN_TO_A_TURN` names the one a turn shows. **A turn shown
//! any other text is a turn no grade is about.** So a continued turn is shown
//! the *same* instructions and the *same* verbs; what grows is the request half,
//! which was always the part that differs from one turn to the next.
//!
//! Had the continuation been written into the instructions instead, every grade
//! ever earned would have stopped describing what a turn shows, silently, and
//! `Instructions::SHOWN_TO_A_TURN`'s own sentence would have become false.
//!
//! **What this does not do is make those grades cover more.** A model that
//! drove ten single-step exercises reliably has been measured at one step and
//! nowhere else. Whether it can finish a job of five is a different
//! measurement, which this file makes possible and does not perform.
//!
//! # A step is the exact two lines, not a description of them
//!
//! [`AStep`] holds the line the model wrote and the line the daemon answered
//! with, as they went over the socket. Three reasons, and the third is the one
//! that matters:
//!
//! 1. The model already reads and writes that dialect — the instructions teach
//!    it, in literal JSON.
//! 2. This crate composes no prose about an answer, so there is no second
//!    wording of a refusal to keep true.
//! 3. **What the model is told it asked for cannot drift from what was really
//!    asked.** A shape holding `alo_protocol::FromAnAgent` would be re-rendered
//!    here, and a re-rendering can differ from what was sent — which is the
//!    failure this repository keeps meeting under other names.
//!
//! # Every step that was shortened says so
//!
//! `alo-files`' rule, met a third time. A read can answer with a whole file, and
//! a turn of five steps carrying five whole files would be a prompt no model can
//! hold. So a line longer than [`AStep::THE_MOST_OF_AN_ANSWER`] is cut and
//! **says that it was cut**, in the text the model reads, because an answer
//! silently shortened reads exactly like a complete one — to a model as much as
//! to a person.

use alo_capability::Verbs;

use crate::instructions::Instructions;
use crate::shown::shown_to_a_model_after;
use crate::the_conversation_so_far::TheConversationSoFar;
use crate::what_it_may_reach::WhatItMayReach;

/// One request a turn has already made, and what came back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AStep {
    /// The line the model wrote, as it was sent.
    asked: String,
    /// The line the daemon answered with, as it arrived.
    came_back: String,
}

impl AStep {
    /// How much of one answer a model is shown, in bytes.
    ///
    /// A bound rather than a budget: it is per answer, so a turn of five steps
    /// shows at most five of these. The number is deliberately generous for a
    /// listing and ungenerous for a file, because a listing is what a next step
    /// is usually decided from and a file's contents rarely are.
    pub const THE_MOST_OF_AN_ANSWER: usize = 2000;

    /// What was asked and what came back, as the two lines really were.
    #[must_use]
    pub fn of(asked: &str, came_back: &str) -> Self {
        Self {
            asked: asked.trim().to_owned(),
            came_back: came_back.trim().to_owned(),
        }
    }

    /// **What the daemon answered, for whoever is reading a log.**
    ///
    /// A turn that ends in `ItRanOutOfRounds` printed one sentence about eight
    /// steps and nothing about any of them — measured on a booted machine on
    /// 2026-10-24, where a real question reached a real provider and the only
    /// thing the machine said was *it got nowhere in 8 step(s)*. The reason
    /// each round failed was built, pushed onto the steps, and then reduced to
    /// a count.
    ///
    /// **The model's own line is deliberately not exposed.** `asked` carries
    /// the person's words and whatever a verb read for them; this side carries
    /// what the machine said back, which for a round that failed to parse is
    /// *not a request: …* and the error — a diagnosis, not their content.
    #[must_use]
    pub fn what_came_back(&self) -> &str {
        &self.came_back
    }

    /// The answer as the model is shown it, cut if it is long and saying so.
    ///
    /// Cut on a character boundary, because cutting a UTF-8 string by bytes
    /// alone panics on the byte that is halfway through a character — and a
    /// file with an accent in it is not an unusual file.
    fn the_answer_shown(&self) -> String {
        if self.came_back.len() <= Self::THE_MOST_OF_AN_ANSWER {
            return self.came_back.clone();
        }
        let mut at = Self::THE_MOST_OF_AN_ANSWER;
        while at > 0 && !self.came_back.is_char_boundary(at) {
            at -= 1;
        }
        let shown = self.came_back.get(..at).unwrap_or("");
        format!("{shown}\n(this answer was longer than that and was cut here)")
    }
}

/// A turn that has already asked for something.
///
/// Holds the person's request and every step since, in the order they happened.
/// It borrows rather than owns, because the loop that drives a turn holds both
/// and this is read once per round.
#[derive(Debug, Clone, Copy)]
pub struct ATurnInProgress<'a> {
    /// What the person asked for, in their own words.
    request: &'a str,
    /// What has been asked and answered so far, oldest first.
    steps: &'a [AStep],
    /// The folders this turn may reach, which the model is told about.
    ///
    /// **Carried on the turn rather than passed beside it**, so a continued
    /// round cannot be shown a different set from the first: a model told
    /// about a folder on round one and not on round two would start guessing
    /// again, which is the behaviour `crate::what_it_may_reach` exists to end.
    may_reach: &'a WhatItMayReach,
    /// What was said in the task window before this request, which is nothing
    /// for a first request (ADR 0137).
    ///
    /// On the turn rather than beside it for `may_reach`'s reason: every round
    /// of one turn is shown the same earlier conversation.
    earlier: &'a TheConversationSoFar,
}

/// The conversation before a first request, which is none.
static NOTHING_EARLIER: TheConversationSoFar = TheConversationSoFar::nothing();

impl<'a> ATurnInProgress<'a> {
    /// A turn on this request, after these steps, able to reach these folders.
    #[must_use]
    pub const fn of(request: &'a str, steps: &'a [AStep], may_reach: &'a WhatItMayReach) -> Self {
        Self {
            request,
            steps,
            may_reach,
            earlier: &NOTHING_EARLIER,
        }
    }

    /// **The same turn, following this earlier conversation** — a follow-up a
    /// person typed in a task window.
    #[must_use]
    pub const fn after(self, earlier: &'a TheConversationSoFar) -> Self {
        Self { earlier, ..self }
    }

    /// Whether anything has been asked yet.
    ///
    /// A turn with no steps is what [`crate::shown_to_a_turn`] already shows,
    /// and [`shown_to_a_turn_in_progress`] answers identically for one — which
    /// is asserted below rather than left to be assumed, because two texts for
    /// the first round would make a grade earned through one door a statement
    /// about the other.
    #[must_use]
    pub const fn has_begun(&self) -> bool {
        !self.steps.is_empty()
    }
}

/// **The whole text a model is shown on a continued turn**: the same
/// instructions, the same verbs, the request, and then what has happened.
///
/// For a turn with no steps this is exactly [`crate::shown_to_a_turn`]'s text,
/// to the byte.
#[must_use]
pub fn shown_to_a_turn_in_progress(verbs: &Verbs, turn: ATurnInProgress<'_>) -> String {
    shown_in_progress(Instructions::SHOWN_TO_A_TURN, verbs, turn)
}

/// The same, under a named set of instructions.
///
/// **Taken rather than assumed, because a grade is about a set.** A harness
/// measuring whether `Instructions::SeveralSteps` helps a model take a second
/// step has to be able to put the same turn under both, and
/// [`shown_to_a_turn_in_progress`] names the one a turn really shows so that a
/// surface with no opinion cannot pick a set by accident.
#[must_use]
pub fn shown_in_progress(
    instructions: Instructions,
    verbs: &Verbs,
    turn: ATurnInProgress<'_>,
) -> String {
    let mut text = shown_to_a_model_after(
        instructions,
        verbs,
        turn.may_reach,
        turn.earlier,
        turn.request,
    );
    if !turn.has_begun() {
        return text;
    }
    text.push_str("\n\nYou have already asked for this, and this came back:\n");
    for step in turn.steps {
        text.push_str("\nYou asked: ");
        text.push_str(&step.asked);
        text.push_str("\nThe answer: ");
        text.push_str(&step.the_answer_shown());
        text.push('\n');
    }
    // Last, for `shown.rs`'s reason: nothing after the instruction can be
    // mistaken for part of what came back.
    text.push('\n');
    text.push_str(WHAT_TO_SEND_NEXT);
    text
}

/// **The last thing a model reads on a continued turn**, and the three ways it
/// may go on.
///
/// Until 2026-10-27 this offered two: *the next line of JSON, or the propose
/// line that finishes the request*. With the reply held to a shape that had no
/// answer in it, a request that needed no change had no way to end, and the one
/// way out this told a model about was a proposal. On booted machines the same
/// request ended in proposals to install Krita, GIMP and Inkscape (ADR 0133).
///
/// **No grade is about this text.** It is shown only once a turn has taken a
/// step, and every grade is earned on a first round, which
/// [`shown_in_progress`] returns before reaching here.
pub const WHAT_TO_SEND_NEXT: &str = "\
Send the next line: a read line if you still need something, a propose line if \
the request asks for anything to be done, or an answer line if the request only \
asks to be told something and what came back tells it.";

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::{AStep, ATurnInProgress, shown_to_a_turn_in_progress};
    use crate::shown::shown_to_a_turn;
    use crate::testing::the_verbs;
    use crate::what_it_may_reach::WhatItMayReach;

    /// **A turn that has asked nothing is shown what it was shown before**, to
    /// the byte. The grade every model holds was earned through
    /// `shown_to_a_turn`, so a first round that differed here would make this
    /// file a change to what a turn shows rather than an addition to it.
    #[test]
    fn the_first_round_is_the_text_it_always_was() {
        let verbs = the_verbs();
        let nothing: [AStep; 0] = [];
        assert_eq!(
            shown_to_a_turn_in_progress(
                &verbs,
                ATurnInProgress::of("tidy my invoices", &nothing, &WhatItMayReach::nothing())
            ),
            shown_to_a_turn(&verbs, &WhatItMayReach::nothing(), "tidy my invoices")
        );
    }

    /// And a second round carries the first, as the two lines really were.
    #[test]
    fn a_second_round_shows_what_was_asked_and_what_came_back() {
        let verbs = the_verbs();
        let asked = r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[]}}}"#;
        let came_back = r#"{"format":1,"tells":{"did":{"listed":{"things":[]}}}}"#;
        let steps = [AStep::of(asked, came_back)];
        let shown = shown_to_a_turn_in_progress(
            &verbs,
            ATurnInProgress::of("tidy", &steps, &WhatItMayReach::nothing()),
        );

        assert!(shown.contains(asked), "the line it wrote is not shown back");
        assert!(shown.contains(came_back), "the answer is not shown");
        assert!(
            shown.starts_with(&shown_to_a_turn(&verbs, &WhatItMayReach::nothing(), "tidy")),
            "a continued turn no longer begins with what a turn shows"
        );
        // The request still comes before what happened, and the instruction to
        // answer comes last.
        let request_at = shown.find("The request: tidy").unwrap();
        let answer_at = shown.find(came_back).unwrap();
        let next_at = shown.find(super::WHAT_TO_SEND_NEXT).unwrap();
        assert!(
            request_at < answer_at,
            "the request moved after the answers"
        );
        assert!(answer_at < next_at, "the instruction is not last");
        assert!(shown.ends_with(super::WHAT_TO_SEND_NEXT));
    }

    /// **A continued turn is told it may answer**, and is not told that a
    /// proposal is what finishes every request — the wording that left a
    /// question with no way to end.
    #[test]
    fn a_continued_turn_may_end_in_an_answer_and_is_not_steered_to_a_proposal() {
        assert!(super::WHAT_TO_SEND_NEXT.contains("an answer line"));
        assert!(!super::WHAT_TO_SEND_NEXT.contains("propose line that finishes the request"));
    }

    /// **A long answer is cut and says it was cut.** The number is asserted
    /// rather than the behaviour alone, because a bound nobody can see the edge
    /// of is a bound somebody will widen by accident.
    #[test]
    fn a_long_answer_is_cut_and_says_so() {
        let verbs = the_verbs();
        let long = "x".repeat(AStep::THE_MOST_OF_AN_ANSWER + 500);
        let steps = [AStep::of("{}", &long)];
        let shown = shown_to_a_turn_in_progress(
            &verbs,
            ATurnInProgress::of("read it", &steps, &WhatItMayReach::nothing()),
        );
        assert!(
            shown.contains("was longer than that and was cut here"),
            "a shortened answer did not say so"
        );
        assert!(
            !shown.contains(&"x".repeat(AStep::THE_MOST_OF_AN_ANSWER + 1)),
            "the whole long answer was shown anyway"
        );
        // And one that just fits is not cut.
        let fits = "y".repeat(AStep::THE_MOST_OF_AN_ANSWER);
        let steps = [AStep::of("{}", &fits)];
        let shown = shown_to_a_turn_in_progress(
            &verbs,
            ATurnInProgress::of("read it", &steps, &WhatItMayReach::nothing()),
        );
        assert!(shown.contains(&fits), "an answer at the bound was cut");
        assert!(!shown.contains("was cut here"));
    }

    /// **Cutting does not panic on a character that straddles the bound.** A
    /// file with an accent in it is an ordinary file, and `&s[..n]` on the wrong
    /// byte is a panic inside a daemon.
    #[test]
    fn a_long_answer_full_of_accents_is_cut_without_panicking() {
        let verbs = the_verbs();
        // Each 'é' is two bytes, so the bound lands mid-character for half of
        // the offsets this could have chosen.
        let long = "é".repeat(AStep::THE_MOST_OF_AN_ANSWER);
        let steps = [AStep::of("{}", &long)];
        let shown = shown_to_a_turn_in_progress(
            &verbs,
            ATurnInProgress::of("read it", &steps, &WhatItMayReach::nothing()),
        );
        assert!(shown.contains("was cut here"));
    }

    /// **Every round of a follow-up is shown the earlier conversation**, before
    /// the request, and the steps of this turn still come after it.
    #[test]
    fn a_follow_up_shows_the_earlier_conversation_on_every_round() {
        use crate::the_conversation_so_far::{AnEarlierTurn, TheConversationSoFar};

        let verbs = the_verbs();
        let earlier = TheConversationSoFar::of(vec![
            AnEarlierTurn::ThePersonSaid("list the invoices".to_owned()),
            AnEarlierTurn::AloAnswered("march.pdf and april.pdf".to_owned()),
        ]);
        let nothing: [AStep; 0] = [];
        let steps = [AStep::of("asked-first", "answered-first")];
        for steps in [&nothing[..], &steps[..]] {
            let shown = shown_to_a_turn_in_progress(
                &verbs,
                ATurnInProgress::of("and the April one too", steps, &WhatItMayReach::nothing())
                    .after(&earlier),
            );
            let earlier_at = shown.find("alo answered: march.pdf and april.pdf").unwrap();
            let request_at = shown.find("The request: and the April one too").unwrap();
            assert!(earlier_at < request_at, "{shown}");
            if let Some(step_at) = shown.find("asked-first") {
                assert!(request_at < step_at, "{shown}");
            }
        }
    }

    /// Several steps are shown oldest first, which is the order they happened
    /// and the only order a model can read as a sequence.
    #[test]
    fn the_steps_are_shown_in_the_order_they_happened() {
        let verbs = the_verbs();
        let steps = [
            AStep::of("asked-first", "answered-first"),
            AStep::of("asked-second", "answered-second"),
        ];
        let shown = shown_to_a_turn_in_progress(
            &verbs,
            ATurnInProgress::of("a job", &steps, &WhatItMayReach::nothing()),
        );
        let first = shown.find("asked-first").unwrap();
        let second = shown.find("asked-second").unwrap();
        assert!(first < second, "the steps are shown newest first");
    }
}
