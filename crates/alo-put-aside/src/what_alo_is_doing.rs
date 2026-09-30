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
}

impl WhatAloIsDoing {
    /// Whether a surface should draw an agent section.
    ///
    /// The only question worth asking before drawing. Named this way round — *is there
    /// something*, not *is it off* — because the second invites a caller to draw the answer.
    #[must_use]
    pub const fn is_anything(&self) -> bool {
        matches!(self, Self::AtWork(_))
    }

    /// The work, if there is any.
    #[must_use]
    pub const fn at_work(&self) -> Option<&AtWork> {
        match self {
            Self::Nothing => None,
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
}
