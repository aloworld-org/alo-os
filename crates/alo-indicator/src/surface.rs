//! The compositor's half: what it is given, and how it may refuse.
//!
//! The plan calls this task *the egress indicator, on a screen*, and a screen
//! is the compositor's. This file fixes the one question between the two: **may
//! this be shown, and here is exactly what it says.**
//!
//! # The request is the picture
//!
//! `alo-overlay`'s summoning seam has an empty `SurfaceRequest` beside it,
//! because *summon the agent* has nothing to carry. This one carries a
//! [`Drawn`], and that is the whole design: a compositor cannot be asked to put
//! up an indicator and then be left to decide what it says. It is handed the
//! light and the lines together, from one moment, made by the one constructor
//! that reads the machine's own `alo_egress::Indicator`.
//!
//! So there is nothing here a compositor could draw that was not a departure —
//! not because it promises not to, but because nothing else is ever passed to
//! it.
//!
//! # Refusing is a fact about the machine
//!
//! [`SurfaceRefused`] has one variant today and it is a fact about the machine
//! rather than about the picture: a picture cannot be malformed, because only
//! `alo_egress::Indicator` can make one. It is an enum rather than a unit
//! struct because the next fact — a screen that went away mid-frame, a session
//! being torn down — is a variant added additively, and
//! [`crate::NotShown::said`] is where each meets its sentence.

use crate::drawn::Drawn;

/// Why the compositor could not show what is leaving this machine.
///
/// Each maps to a sentence in [`crate::words`] through
/// [`crate::NotShown::said`], because a machine that cannot show what is
/// leaving it and does not say so is the exact failure law 1 exists to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SurfaceRefused {
    /// The compositor is running with no screen to put a surface on — an
    /// offscreen or headless session, or a machine whose display went away.
    NothingToShowOn,
}

/// The compositor, as the indicator sees it: something that can put what is
/// leaving this machine in front of a person, or say why it cannot.
///
/// Implemented by whatever owns the screen — `alo-shell`, when it wires this
/// seam up — and by nothing else. The answer is synchronous because the two
/// live in the same process: the shell owns the surfaces, so *can this be
/// shown* is a question it answers without waiting on anybody.
pub trait Compositor {
    /// Show exactly this: the light, and the lines under it.
    ///
    /// Taken by value because a picture is one moment and is spent when it is
    /// drawn. A compositor that kept one and drew it again later would be
    /// showing a machine that no longer exists, which on this surface is the
    /// difference between *nothing is leaving* and *nothing was leaving a
    /// minute ago*.
    ///
    /// # Errors
    /// [`SurfaceRefused`] when the compositor has nowhere to show it.
    fn show(&mut self, drawn: Drawn) -> Result<(), SurfaceRefused>;
}
