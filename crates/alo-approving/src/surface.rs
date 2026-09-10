//! The compositor's half: what it is given, and how it may refuse.
//!
//! The plan calls this task *one approval, and the sentence a person approves*,
//! and the sentence has to reach a person's eyes. This file fixes the one
//! question between this crate and whatever owns the screen: **may this be put
//! in front of somebody, and here is exactly what it says.**
//!
//! # The request is the question
//!
//! `alo-overlay`'s summoning seam has an empty `SurfaceRequest` beside it,
//! because *summon the agent* has nothing to carry. This one carries an
//! [`Asked`], for the same reason `alo_indicator::Compositor` carries a picture:
//! a compositor cannot be asked to put a change to somebody and then be left to
//! decide what the change is. It is handed the sentence, whose question it is
//! and how long is left, all from the moment the question was really waiting.
//!
//! So there is nothing here a compositor could draw that was not a proposal the
//! capability model checked — not because it promises not to, but because
//! nothing else is ever passed to it.
//!
//! # What a compositor is not asked to decide
//!
//! Not how the two answers are drawn, not where the surface sits, not whether
//! it is modal, and **not what happens when it is answered**. An answer comes
//! back through [`crate::Approving`], which is the only road to
//! `alo_turn::Turning` — a compositor that could execute what it drew would be
//! a second executor with no record behind it.
//!
//! # Refusing is a fact about the machine
//!
//! [`SurfaceRefused`] has one variant today and it is a fact about the machine
//! rather than about the question: a question cannot be malformed, because only
//! a change that is really waiting can become one. It is an enum rather than a
//! unit struct because the next fact — a screen that went away mid-frame, a
//! session being torn down — is a variant added additively, and
//! [`crate::NotAsked::said`] is where each meets its sentence.

use crate::asked::Asked;

/// Why the compositor could not put a change to the person.
///
/// Each maps to a sentence in [`crate::words`] through
/// [`crate::NotAsked::said`], because a change that could not be put to
/// anybody, and that nobody was told about, is an agent that appears to have
/// been ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SurfaceRefused {
    /// The compositor is running with no screen to put a surface on — an
    /// offscreen or headless session, or a machine whose display went away.
    NothingToShowOn,
}

/// The compositor, as this surface sees it: something that can put one change
/// in front of a person, or say why it cannot.
///
/// Implemented by whatever owns the screen — `alo-shell`, when it wires this
/// seam up — and by nothing else. The answer is synchronous because the two
/// live in the same process: the shell owns the surfaces, so *can this be
/// shown* is a question it answers without waiting on anybody. **Whether the
/// person answers is not this call**: it hands the question over and returns.
pub trait Compositor {
    /// Put exactly this change to the person: the sentence, whose it is, and
    /// how long is left to answer.
    ///
    /// Taken by value because a question put to somebody is one moment. A
    /// compositor that kept one and put it up again later would be asking about
    /// a machine that has moved on — and on this surface that is the difference
    /// between a change somebody agreed to and a change they agreed to an hour
    /// ago about a file that is no longer there.
    ///
    /// # Errors
    /// [`SurfaceRefused`] when the compositor has nowhere to put it.
    fn ask(&mut self, asked: Asked) -> Result<(), SurfaceRefused>;
}
