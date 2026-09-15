//! What an update may never do to the person using the machine.
//!
//! *Updates that never interrupt*, as a value with three clauses: an update
//! **never restarts the machine**, **never closes an application**, and **never
//! interrupts what a person is doing**. [`THE_RULE`] answers for each of them,
//! and each has a test of its own, because a promise tested as a whole can lose
//! a clause without anybody noticing.
//!
//! # The person may do all three, and that is not a loophole
//!
//! A person restarting their own machine closes their applications and applies
//! the update waiting for it. That is not the update restarting the machine; it
//! is the person choosing *when*. So the rule is asked about a [`Cause`] as
//! well as a [`Disturbance`], and the answer turns on the cause alone: the
//! person may, an update may not.
//!
//! # There is no *urgent*
//!
//! Every system that interrupts people for updates did it with a word — urgent,
//! critical, required, security — attached to an update and read by the code
//! that decided to restart. [`Cause`] has two members and neither of them is an
//! update that matters more than another; [`crate::Ready`] carries no priority
//! for one to be read from; and [`TheRule::allows`] takes nothing else. An
//! update that fixes something serious is applied sooner by being *said*
//! plainly to the person, not by taking the choice away from them.

use alo_strings::{Filling, Said, Strings};
use serde::Serialize;

use crate::words::{self, Word};

/// Something that would take a person's machine away from them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Disturbance {
    /// Restarting the machine.
    RestartingTheMachine,
    /// Closing an application that is open.
    ClosingAnApplication,
    /// Interrupting what a person is doing: taking focus, covering their work,
    /// or asking them something they did not start.
    InterruptingThePerson,
}

impl Disturbance {
    /// Every disturbance there is — the three clauses of the rule.
    pub const EVERY: [Self; 3] = [
        Self::RestartingTheMachine,
        Self::ClosingAnApplication,
        Self::InterruptingThePerson,
    ];

    /// The clause of the promise this disturbance is.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::RestartingTheMachine => words::NEVER_RESTARTS,
            Self::ClosingAnApplication => words::NEVER_CLOSES_AN_APPLICATION,
            Self::InterruptingThePerson => words::NEVER_INTERRUPTS,
        }
    }
}

/// Who would be causing a disturbance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cause {
    /// An update, for any reason at all.
    AnUpdate,
    /// The person using the machine, because they chose to.
    ThePerson,
}

/// The rule that an update never disturbs a person.
///
/// A value rather than a function so that it can be named where it is kept —
/// in settings, beside the promise it is — and so that there is exactly one of
/// it: it has no fields, and nothing configures it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheRule {
    /// Private, so the one value is [`THE_RULE`].
    only_one: (),
}

/// The rule.
pub const THE_RULE: TheRule = TheRule { only_one: () };

/// A disturbance an update was about to cause, refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Forbidden {
    /// What was refused.
    disturbance: Disturbance,
}

impl TheRule {
    /// Whether this cause may cause this disturbance.
    ///
    /// # Errors
    /// [`Forbidden`] whenever the cause is an update. There is no argument that
    /// changes that answer, and no second method that does.
    pub fn allows(self, cause: Cause, disturbance: Disturbance) -> Result<(), Forbidden> {
        let Self { only_one: () } = self;
        match cause {
            Cause::ThePerson => Ok(()),
            Cause::AnUpdate => Err(Forbidden { disturbance }),
        }
    }

    /// The promise, clause by clause, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> [Said; 3] {
        Disturbance::EVERY.map(|clause| strings.say(&clause.word().key(), &Filling::nothing()))
    }
}

impl Forbidden {
    /// What was refused.
    #[must_use]
    pub fn disturbance(self) -> Disturbance {
        self.disturbance
    }

    /// The clause of the promise that refused it.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.disturbance.word().key(), &Filling::nothing())
    }
}
