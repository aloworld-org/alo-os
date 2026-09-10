//! The surface request the compositor can honour or refuse.
//!
//! This is the compositor's half of the summoning seam: one request type, one
//! answer, and the port a compositor implements to receive them. There is
//! deliberately no rendering in any of it — no size, no position, no colour,
//! no z-order — because what the overlay looks like is the compositor's to
//! decide and does not exist yet. What this file fixes is only the question:
//! *may the agent appear?*
//!
//! # Why the request is empty, and why that is not a placeholder
//!
//! There is exactly one agent overlay on a machine, so a request for it has
//! nothing to say beyond that it was made. The type still exists — rather than
//! a bare method with no argument — because it is the seam's vocabulary:
//! [`crate::Summoning::press`] creates one per summons and nothing else can
//! create one at all, which turns *pressing it asks for the surface exactly
//! once* from a convention into something the compiler holds. Anything the
//! request one day needs to carry — which seat pressed the key, say — is added
//! as a field, additively, which is why the struct is `#[non_exhaustive]`.

/// One request for the agent overlay's surface.
///
/// Created only by [`crate::Summoning::press`] — there is no public
/// constructor, so a compositor holding one of these knows a person pressed
/// the key. It claims nothing about pixels: not where the overlay goes, not
/// how big it is, not what it shows. Honouring it means the compositor takes
/// responsibility for making the overlay appear; how is entirely its own.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SurfaceRequest {}

impl SurfaceRequest {
    /// The one place a request comes from: a press on a closed summoning.
    pub(crate) fn asked() -> Self {
        Self {}
    }
}

/// Why the compositor could not honour a [`SurfaceRequest`].
///
/// Every variant here is a fact about the machine rather than about the
/// request — a request cannot be malformed, because only a press can make one.
/// Each maps to a sentence in [`crate::words`] through
/// [`crate::NotSummoned::said`], because a refusal a person cannot read is a
/// key that silently does nothing, which is the worst outcome available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRefused {
    /// The compositor is running with no screen to put a surface on — an
    /// offscreen or headless session, or a machine whose display went away.
    NothingToShowOn,
}

/// The compositor, as the summoning sees it: something that can honour or
/// refuse a request for the overlay's surface.
///
/// Implemented by whatever owns the screen — `alo-shell`, when it wires this
/// seam up — and by nothing else. The answer is synchronous because the
/// summoning and the compositor live in the same process: the shell routes the
/// chord and owns the surfaces, so *may the agent appear* is a question it can
/// answer without waiting on anybody.
pub trait Compositor {
    /// Honour the request — the overlay will appear — or refuse it with a
    /// reason a sentence exists for.
    ///
    /// The request is taken by value because it is spent either way: a
    /// compositor cannot keep it to honour later, and a refused request is
    /// gone — the next press makes a new one.
    ///
    /// # Errors
    /// [`SurfaceRefused`] when the compositor has nowhere to show the overlay.
    fn honour(&mut self, asked: SurfaceRequest) -> Result<(), SurfaceRefused>;
}
