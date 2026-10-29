//! How one turn of the loop ended.
//!
//! Five ends, and every one of them is a thing that happened rather than an
//! error: a change waiting for a person is the **success** this loop is for,
//! and running out of rounds is a fact about the model rather than a fault in
//! the machine.
//!
//! # A change waiting is the end, not a step
//!
//! `CLAUDE.md`: *reads answer, changes wait*. So a turn reads as far as it needs
//! to and then proposes once, and the proposal is where the loop stops —
//! because what happens next is a person reading a sentence, which is not
//! something an agent does. The approval arrives on the **other** door, which
//! this crate cannot reach.
//!
//! **The loop stops; the turn does not.** This said until 2026-10-26 that the
//! proposal was not something this crate may wait for, and the process exited
//! on it — which ended the turn with the question inside it, so nothing was
//! ever waiting by the time the person's shell asked. The loop still ends at
//! the proposal; [`crate::holding_the_turn`] keeps the turn open after it until
//! the person has answered, for as long as [`WhatHappened::AChangeIsWaiting`]'s
//! `stands_for` and no longer.
//!
//! A turn that proposed twice would be a turn asking a person two questions
//! about one request, and the second would arrive with the first unanswered. So
//! the first proposal ends it.

use alo_instructing::AStep;

/// How a turn ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatHappened {
    /// A change is waiting for the person, under this number, on that sentence.
    ///
    /// **The end this loop exists to reach.** Everything before it was reading.
    AChangeIsWaiting {
        /// The number the person approves or declines by.
        number: u64,
        /// The sentence they will read, already in their own language.
        sentence: String,
        /// Every read this turn made on the way, oldest first.
        after: Vec<AStep>,
        /// How long the daemon said the question stands, and [`None`] when it
        /// did not say.
        ///
        /// **The daemon's figure, not one chosen here** — `proposal-seconds` in
        /// the machine's description. It bounds how long the turn is held open
        /// for a person who never answers; a person who answers ends the hold
        /// at once.
        stands_for: Option<std::time::Duration>,
    },
    /// The request needed no change, and this is the answer to it.
    ///
    /// A model that read what it needed and then answered in words rather than
    /// proposing: *"where is that file?"* is finished by the answer and has
    /// nothing to approve.
    ItWasAnsweredInWords {
        /// What the model said, in its own words and never inside one of ours.
        said: String,
        /// Every read this turn made on the way, oldest first.
        after: Vec<AStep>,
    },
    /// The machine refused something, and this is what it said.
    ///
    /// The sentence is the refusing crate's own, rendered in the person's
    /// language by the daemon. Nothing here rewords it.
    ItWasRefused {
        /// What to tell the person.
        said: String,
        /// How far it had got, oldest first.
        after: Vec<AStep>,
    },
    /// **The person stopped it** (ADR 0139): the shell shut its side of the
    /// request, and the loop asked for nothing more. Not a refusal and not a
    /// fault; what was done before it is in `after`.
    ItWasStopped {
        /// What was asked and answered before the Stop arrived.
        after: Vec<AStep>,
    },
    /// The turn used every round it had without finishing.
    ///
    /// Not a refusal and not a fault: the model kept asking for reads and never
    /// proposed or answered. What a person is told about it is a surface's to
    /// decide, which is why there is no sentence here.
    ItRanOutOfRounds {
        /// Everything it did ask for, oldest first.
        after: Vec<AStep>,
    },
    /// The machine could not be asked at all.
    ///
    /// The door, not the turn: no daemon, or something on the socket that is
    /// not this protocol. English, because its reader is fixing the machine.
    TheMachineCouldNotBeAsked {
        /// What went wrong, in English.
        why: String,
        /// How far it had got, oldest first.
        after: Vec<AStep>,
    },
}

impl WhatHappened {
    /// Every read this turn made, however it ended.
    ///
    /// One accessor rather than five matches at every call site, and the reason
    /// it exists at all: **a turn that got three steps in and then failed did
    /// those three things**, and a surface that showed nothing because the end
    /// was a failure would be hiding work the record already has.
    #[must_use]
    pub fn after(&self) -> &[AStep] {
        match self {
            Self::AChangeIsWaiting { after, .. }
            | Self::ItWasAnsweredInWords { after, .. }
            | Self::ItWasRefused { after, .. }
            | Self::ItRanOutOfRounds { after }
            | Self::ItWasStopped { after }
            | Self::TheMachineCouldNotBeAsked { after, .. } => after,
        }
    }

    /// Whether this is the end a request that changes something reaches.
    #[must_use]
    pub const fn is_waiting_for_the_person(&self) -> bool {
        matches!(self, Self::AChangeIsWaiting { .. })
    }
}
