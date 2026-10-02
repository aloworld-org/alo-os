//! Whether there is anything to say about alo in this window, and what.
//!
//! The one place a surface asks *is there an agent section here at all*. Task 7 of
//! `docs/autonomy/putting-a-window-aside.md`.
//!
//! # This type is ADR 0009 in this panel
//!
//! > The agent's surfaces **disappear rather than nag**. The hotkey does nothing, the overlay
//! > does not exist, and Grants, Models and providers are **absent from Settings rather than
//! > present-but-disabled**. A greyed-out feature is an advertisement.
//!
//! So [`WhatAloIsDoing::Nothing`] is not a disabled state and not an empty report. It is the
//! **absence of a report**, and a surface handed it has nothing to draw — no heading, no
//! greyed control, no *AI is off* notice. The panel is then exactly the panel of task 1:
//! previews, a header, a count, three presentations, all fully functional.
//!
//! There is no third variant for *switched off* as distinct from *nothing happening here*,
//! and that is deliberate rather than an omission. Both draw nothing, so a variant
//! distinguishing them would exist only to be branched on — and the first thing somebody
//! would do with a `SwitchedOff` case is write a message for it. **That message is the
//! nagging.** A surface that genuinely needs to know whether this machine has an agent is
//! asking a question about the machine, and it should ask the machine.
//!
//! # The rule above still stands, and [`WhatAloIsDoing::StatusUnavailable`] is not an
//! exception to it
//!
//! A third variant was added on 2026-10-03 and the paragraph above was not weakened, so a
//! reader meeting both deserves the test rather than the precedent. **A variant may exist
//! here only if it cannot reach a person who never handed a window to alo.** One that can is
//! a message about a feature, and that is the nagging, whatever it draws.
//!
//! `StatusUnavailable` passes because the handover is what produces it, and the type is what
//! enforces that: `alo_admitting::Reports::the_reporter_is_gone` answers `Option`, keyed on
//! the windows the person actually handed over, so a window nobody gave to alo yields
//! **`None` — not `Nothing`, and no variant at all.** A machine whose owner has never touched
//! the agent cannot produce this value, which makes it a correction to a claim that person
//! asked for rather than an advertisement for something they declined.
//!
//! `SwitchedOff` fails that test, which is why it is still refused: every machine can reach
//! it, including one whose owner wants nothing to do with an agent.
//!
//! **Phrased as a test rather than as a permission on purpose.** A fourth variant must pass
//! this; citing `StatusUnavailable` as a precedent is not passing it.
//!
//! # Why an enum and not `Option<AtWork>`
//!
//! Because the panel is built for a machine where the agent may not exist at all, and
//! `Option` reads as *not yet*. `None` invites the empty state a `Nothing` refuses: a caller
//! writing `if let Some(..) else { draw_the_placeholder() }` has written idiomatic Rust and
//! broken ADR 0009. The named case is the one whose own documentation says what to draw,
//! which is nothing.
//!
//! The same argument was made for `Travel` in `crate::restoring` and is stronger here,
//! because getting it wrong shows a person an advertisement rather than moving a window.

use crate::alo_at_work::AtWork;

/// Whether alo has anything to report in this window.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum WhatAloIsDoing {
    /// **Nothing to draw at all.** No agent on this machine, or none working here.
    ///
    /// The default, because a window nobody handed to alo is the ordinary window and the
    /// panel must be complete without an agent. A surface reading this draws no heading and
    /// no control — see this module's header for why there is no *switched off* variant to
    /// write a notice for.
    #[default]
    Nothing,
    /// A task is under way, and this is all of it.
    AtWork(AtWork),
    /// **A report was arriving and has stopped, so what is on screen is no
    /// longer known to be true.**
    ///
    /// The owner ruled on 2026-10-02 that a lost connection must become *status
    /// unavailable* and must never remain *working*. Without this variant it
    /// remains working: an [`Self::AtWork`] sits in a preview for as long as the
    /// panel holds it, and nothing in the type can tell a surface that the last
    /// thing it was told is now only the last thing it was told.
    ///
    /// # Why this is not the `SwitchedOff` variant this module refuses
    ///
    /// **Because a person who never handed a window to alo cannot reach it**,
    /// and that is the test this module's header sets. It is not that this case
    /// draws something while `SwitchedOff` draws nothing — drawing something is
    /// true here and is not the safeguard.
    ///
    /// The safeguard is the handover. `alo_admitting::Reports::the_reporter_is_gone`
    /// answers `Option`, keyed on the windows the person handed over, so a
    /// window nobody gave to alo yields `None` — not [`Self::Nothing`], and no
    /// variant at all. So this value exists only where the person already asked
    /// for the work, which makes it a correction to a claim they wanted rather
    /// than an advertisement for a feature they declined.
    ///
    /// What it corrects: a row is already on screen for that window, with a task
    /// name and a progress bar, and the row is making a claim that has stopped
    /// being true.
    ///
    /// [`Self::Nothing`] is the wrong answer for it, and wrong in the expensive
    /// direction: `Nothing` means no agent is working here, so a surface handed
    /// it draws no agent section at all. A dropped connection would silently
    /// erase the work rather than report it, and the person would read that as
    /// the task having finished.
    StatusUnavailable,
}

impl WhatAloIsDoing {
    /// Whether a surface should draw an agent section.
    ///
    /// The only question worth asking before drawing. Named this way round — *is there
    /// something*, not *is it off* — because the second invites a caller to draw the answer.
    #[must_use]
    pub const fn is_anything(&self) -> bool {
        matches!(self, Self::AtWork(_) | Self::StatusUnavailable)
    }

    /// The work, if there is any.
    #[must_use]
    pub const fn at_work(&self) -> Option<&AtWork> {
        match self {
            // Nothing to show, because nothing is happening here.
            Self::Nothing => None,
            // Nothing to show, because what was happening can no longer be
            // confirmed. The opposite reason, and the same answer: a surface
            // must not be handed a report it would draw as current.
            Self::StatusUnavailable => None,
            Self::AtWork(work) => Some(work),
        }
    }

    /// Whether alo has stopped and needs the person.
    ///
    /// Answers `false` when there is nothing here, which is the honest reading: a machine
    /// with no agent is not waiting on anybody. Offered so that a panel can mark the one
    /// preview that needs attention without unwrapping anything.
    #[must_use]
    pub fn requires_you(&self) -> bool {
        self.at_work()
            .is_some_and(|work| work.requires_you().is_waiting())
    }

    /// Whether what is on screen has stopped being known to be true.
    ///
    /// The one branch [`Self::StatusUnavailable`] exists to be taken. A surface
    /// that draws an agent section asks [`Self::is_anything`] first, and then
    /// this: a `true` means draw the section with its status unknown rather
    /// than with the last thing it was told.
    ///
    /// **Not folded into `requires_you`.** An unavailable status is not a
    /// question put to the person — there is nothing for them to answer, and
    /// marking it as needing them would send them to a window to do nothing.
    #[must_use]
    pub const fn status_is_unavailable(&self) -> bool {
        matches!(self, Self::StatusUnavailable)
    }
}
