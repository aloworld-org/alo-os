//! Which surface a frame is on.
//!
//! `docs/autonomy/the-canvas-and-its-places.md` task 1: *a Place is a thing, and
//! the canvas owns what one is.* A **Place is the endless surface** — the plane
//! this crate already has arithmetic for, of which a person may have more than
//! one.
//!
//! # The fifth meaning of a word already spent four times
//!
//! ```text
//! Place   alo-agentd/src/place.rs      the directory a Unix socket goes in
//! Place   alo-looking/src/place.rs     where this machine asks about an update
//! Place   alo-dividing/src/place.rs    a half, a quarter, or a part of a screen
//! APlace  alo-dock/src/places.rs       where an application icon sits on the bar
//! ```
//!
//! None of those four is touched or renamed, and the canvas is the only surface
//! entitled to the fifth: `docs/decisions/0065`'s own diagram is
//! `World → Place → Object`, and this is that middle level. The Dock's type wears
//! an indefinite article for exactly this reason — *a* place on the bar, against
//! *the* Place a window is on.
//!
//! # Why a patch alone was never an answer
//!
//! `(4200, 0)` exists on every surface. *Where is this window* has two halves and
//! has only ever had one, so two windows a person put on two different surfaces
//! have been indistinguishable to every test in this repository that compares
//! rectangles — and there are a dozen of them. A [`Place`] beside the rectangle
//! is the missing half.
//!
//! # Why it is checked, and why zero is not one
//!
//! `alo-arranging` states the rule this type has to satisfy before it is written
//! down: *a position read back without being checked is a position nothing
//! validated*, which is why there is no `Deserialize` for a `Camera` or a `Frame`
//! there and must not be one. An identity read off disk is the same hazard in a
//! quieter form — **serde will invent a number where a process will not** — and
//! `TheFile` carries `#[serde(default)]`, so the number it invents is zero.
//!
//! So zero is refused, and [`Place::numbered`] returns [`Option`] rather than the
//! bare newtype that would have been the obvious thing to write. It costs a
//! `?` at the one boundary that needs it and makes *no Place at all* a value that
//! cannot be constructed by accident.
//!
//! # Why there is no `Default`
//!
//! [`Size`](crate::Size) has none either, and for the neighbouring reason: zero is
//! not a size, and *whichever one* is not an answer to which surface a window is
//! on. A default would let every call site that has not thought about Places
//! compile unchanged and silently mean [`Place::FIRST`], which is the whole fault
//! this task exists to remove. [`Place::FIRST`] is available and says so in its
//! name; what is not available is forgetting to say.

/// Which surface — which endless plane — something is on.
///
/// Opaque and [`Copy`]. The number inside is the compositor's own and means
/// nothing to a person; a Place a person recognises is a name and a wallpaper
/// on top of one of these, which is not this crate's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place(u64);

impl Place {
    /// The Place a machine that has never been used is looking at.
    ///
    /// A person always has at least one surface, because a canvas with no Place
    /// is a machine with nowhere to put a window. Numbered rather than named:
    /// the first Place is not special afterwards, and nothing may come to depend
    /// on it being the one a window falls back to.
    pub const FIRST: Self = Self(1);

    /// This Place, or [`None`] where the number is not one.
    ///
    /// **Zero is refused**, so a number arriving from outside this process — off
    /// a disk, across a socket — cannot mean *a Place* by omission. See this
    /// module's note; the rule is `alo-arranging`'s and predates this type.
    #[must_use]
    pub const fn numbered(number: u64) -> Option<Self> {
        if number == 0 {
            None
        } else {
            Some(Self(number))
        }
    }

    /// Which Place this is.
    #[must_use]
    pub const fn number(self) -> u64 {
        self.0
    }

    /// The Place after this one, or [`None`] at the end of the numbers.
    ///
    /// How a new surface is given an identity: the canvas hands out the next one
    /// rather than a caller choosing, so two Places cannot be made to collide by
    /// two callers picking the same number.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(next) => Some(Self(next)),
            None => None,
        }
    }
}

#[cfg(test)]
#[path = "place_tests.rs"]
mod tests;
