//! A file dropped on the agent's surface: offered for that question, and a
//! grant of nothing.
//!
//! This is the file the plan's acceptance is about, and the sentence it has to
//! make true is short: **dropping a file onto an agent's surface offers it as
//! context for that turn only and is not a grant.**
//!
//! # Why a drop could so easily have become a grant
//!
//! It is the convenient reading. A person drags an invoice onto the overlay and
//! asks what is in it; the obvious implementation grants the agent that path so
//! it can open it, and the obvious next step is that the grant outlives the
//! question. ADR 0001 §3 forbids it: a grant comes from a deliberate act, and
//! the two acts it names are a folder chosen in a picker and the document
//! offered at invocation. A drag of the wrist is neither, and
//! `docs/autonomy/v0-5-hands-on-the-desktop-plan.md` says it in one line —
//! *no drop to grant, however convenient it looks.*
//!
//! # So nothing is granted, because the file never becomes a path here
//!
//! What lands on the agent's surface is what landed on any other window: the
//! forms the source offered, and the way back to the source. The agent reads
//! **the payload** — the bytes the application hands over for the form it asks
//! for — exactly as a window that was dropped on reads them. No path is
//! extracted, nothing is exported, and nothing anywhere is told that an agent
//! may reach anything it could not reach a moment ago.
//!
//! That is not an argument, it is the crate's dependencies: `alo-capability` is
//! a **dev-dependency** here, so there is no `Grant`, no `Grants` and no `Reach`
//! linked into this crate to make one out of.
//! `tests/a_drop_on_the_agent_is_not_a_grant.rs` reads the manifest, and then
//! does the drop in front of a real grant list and finds it empty.
//!
//! # For that turn only, twice over
//!
//! **It is read once.** [`ForOneTurn::read`] takes `self`, and a `ForOneTurn` is
//! not `Clone`: a second reading of the same drop is not a program that
//! compiles. That is `alo_context::Turn`'s rule, in the same shape, for the same
//! reason.
//!
//! **And it ends with the question.** The moment the turn is over is carried in
//! the value, so a daemon that forgets to throw one away still has an offer that
//! answers nothing — [`crate::NotHanded::TheQuestionIsOver`], rather than a file
//! read into a turn nobody is watching.

use std::time::SystemTime;

use alo_clipboard::{Gives, Kind};

use crate::refusing::NotHanded;

/// What a person dropped onto the agent's surface, for one question.
///
/// Deliberately not `Clone` and deliberately not `Debug`-able into its payload:
/// what it holds is a way back to the application that has the data, and the
/// moment the question ends.
pub struct ForOneTurn {
    /// Every form the source said it can give, in the source's own order.
    forms: Vec<Kind>,
    /// The way back to the window the drag came from.
    source: Box<dyn Gives>,
    /// When the question this was dropped into is over.
    ends: SystemTime,
}

impl std::fmt::Debug for ForOneTurn {
    /// The forms and the moment, and **never the payload** — which is not held
    /// here to print, and would be a person's own document if it were.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ForOneTurn")
            .field("forms", &self.forms)
            .field("ends", &self.ends)
            .finish_non_exhaustive()
    }
}

impl ForOneTurn {
    /// The offer a drop on the agent's surface made. Built by [`crate::Drag`]
    /// and by nothing else.
    pub(crate) fn of(forms: Vec<Kind>, source: Box<dyn Gives>, ends: SystemTime) -> Self {
        Self {
            forms,
            source,
            ends,
        }
    }

    /// Every form what was dropped can be read in, in the order the application
    /// that has it preferred them.
    #[must_use]
    pub fn forms(&self) -> &[Kind] {
        &self.forms
    }

    /// Whether one of the forms on offer is this one.
    #[must_use]
    pub fn offers(&self, form: &Kind) -> bool {
        self.forms.contains(form)
    }

    /// When the question this was dropped into is over.
    #[must_use]
    pub fn ends(&self) -> SystemTime {
        self.ends
    }

    /// Read what was dropped, in this form, during this question.
    ///
    /// Takes `self`: one drop is one reading, which is *for that turn only*
    /// carried by the type rather than by a rule.
    ///
    /// # Errors
    /// [`NotHanded::NotThatForm`] for a form the source never offered — asked
    /// for **without the application being woken up at all**, exactly as
    /// `alo_clipboard::Clipboard::paste` refuses one;
    /// [`NotHanded::TheQuestionIsOver`] once the turn has ended; and
    /// [`NotHanded::DidNotHandItOver`] when the application no longer can.
    pub fn read(mut self, form: &Kind, now: SystemTime) -> Result<Vec<u8>, NotHanded> {
        if now >= self.ends {
            return Err(NotHanded::TheQuestionIsOver);
        }
        if !self.forms.contains(form) {
            return Err(NotHanded::NotThatForm { form: form.clone() });
        }
        self.source.give(form).map_err(NotHanded::DidNotHandItOver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Asked, an_application, at, nothing_to_give};

    /// What the agent reads is what the window that had it gives, once.
    #[test]
    fn what_was_dropped_is_read_in_a_form_that_was_offered() {
        let asked = Asked::nothing_yet();
        let offered = ForOneTurn::of(vec![Kind::text()], an_application(&asked), at(200));

        assert!(offered.offers(&Kind::text()));
        assert_eq!(offered.ends(), at(200));
        assert_eq!(
            offered.read(&Kind::text(), at(100)),
            Ok(b"what was dragged, as text/plain;charset=utf-8".to_vec())
        );
        assert_eq!(asked.how_many(), 1);
    }

    /// **A form nobody offered is refused without the application being asked.**
    /// A compositor that asked and then converted would hand an agent bytes the
    /// window never said it could produce.
    #[test]
    fn a_form_nobody_offered_is_refused_and_nobody_is_woken_up() {
        let asked = Asked::nothing_yet();
        let offered = ForOneTurn::of(vec![Kind::text()], an_application(&asked), at(200));

        assert_eq!(
            offered.read(&Kind::image_png(), at(100)),
            Err(NotHanded::NotThatForm {
                form: Kind::image_png()
            })
        );
        assert_eq!(asked.how_many(), 0);
    }

    /// **The question ends and the offer answers nothing** — and the
    /// application is not asked then either, so a daemon that kept one around
    /// cannot read a person's file into a turn that is over.
    #[test]
    fn once_the_question_is_over_nothing_is_read() {
        let asked = Asked::nothing_yet();
        let offered = ForOneTurn::of(vec![Kind::text()], an_application(&asked), at(200));

        assert_eq!(
            offered.read(&Kind::text(), at(200)),
            Err(NotHanded::TheQuestionIsOver)
        );
        assert_eq!(asked.how_many(), 0);
    }

    /// The window that had it quit between the drop and the reading.
    #[test]
    fn an_application_that_cannot_hand_it_over_says_so() {
        let offered = ForOneTurn::of(vec![Kind::text()], nothing_to_give(), at(200));
        assert!(matches!(
            offered.read(&Kind::text(), at(100)),
            Err(NotHanded::DidNotHandItOver(_))
        ));
    }

    /// What is printed about a drop on the agent is the forms and the moment,
    /// and never a person's own document.
    #[test]
    fn nothing_printed_about_it_is_the_persons_own() {
        let asked = Asked::nothing_yet();
        let offered = ForOneTurn::of(vec![Kind::text()], an_application(&asked), at(200));
        let printed = format!("{offered:?}");
        assert!(printed.contains("text/plain"), "{printed}");
        assert!(!printed.contains("what was dragged"), "{printed}");
    }
}
