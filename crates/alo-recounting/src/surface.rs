//! The compositor's half: what it is given, and how it may refuse.
//!
//! The plan calls this task *afterwards, ask what it did*, and the answer has
//! to reach a person's eyes. This file fixes the one question between this
//! crate and whatever owns the screen: **may this be put in front of somebody,
//! and here is exactly what it says.**
//!
//! # The request is the account
//!
//! A compositor cannot be asked to show somebody what their machine has been
//! doing and then be left to decide what that was. It is handed the whole
//! account — every line, in the order it happened, and what the record says
//! about its own completeness — from the moment the file was read.
//!
//! So there is nothing here a compositor could draw that was not written down
//! when it happened: not because it promises not to, but because nothing else
//! is ever passed to it.
//!
//! # What a compositor is not asked to decide
//!
//! Not how a line is laid out, not how a moment is written, not which lines are
//! worth showing, and **not what any of it means**. The clause at the head of
//! each line and the sentence after it are fixed here; the shell chooses type,
//! spacing and order on the screen.
//!
//! Nor is it asked to fetch anything. There is no call back into this crate: an
//! account is one reading of one file, and a surface that wanted a fresher one
//! asks for it again through [`crate::Recounting`], which reads the disk again.
//!
//! # Refusing is a fact about the machine
//!
//! [`SurfaceRefused`] has one variant today and it is a fact about the machine
//! rather than about the account: an account cannot be malformed, because only
//! a record that was read can become one. It is an enum rather than a unit
//! struct because the next fact — a screen that went away mid-frame, a session
//! being torn down — is a variant added additively, and
//! [`crate::NotRecounted::said`] is where each meets its sentence.

use crate::account::Account;

/// Why the compositor could not put the account in front of the person.
///
/// Each maps to a sentence in [`crate::words`] through
/// [`crate::NotRecounted::said`], because somebody who asked what their machine
/// has been doing and was answered with nothing has been told something untrue
/// about their machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SurfaceRefused {
    /// The compositor is running with no screen to put a surface on — an
    /// offscreen or headless session, or a machine whose display went away.
    NothingToShowOn,
}

/// The compositor, as this surface sees it: something that can put one account
/// in front of a person, or say why it cannot.
///
/// Implemented by whatever owns the screen — `alo-shell`, when it wires this
/// seam up — and by nothing else. The answer is synchronous because the two
/// live in the same process: the shell owns the surfaces, so *can this be
/// shown* is a question it answers without waiting on anybody.
pub trait Compositor {
    /// Put exactly this account in front of the person.
    ///
    /// Taken by value because an account is one reading of one file at one
    /// moment. A compositor that kept one and drew it again later would be
    /// showing a person a machine that has moved on — and on this surface that
    /// is the difference between *nothing has happened since* and *nothing has
    /// been read since*.
    ///
    /// # Errors
    /// [`SurfaceRefused`] when the compositor has nowhere to put it.
    fn show(&mut self, account: Account) -> Result<(), SurfaceRefused>;
}
