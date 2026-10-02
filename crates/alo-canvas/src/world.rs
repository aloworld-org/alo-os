//! Every Place a person has, seen at once.
//!
//! `docs/autonomy/the-canvas-and-its-places.md` task 2, and the level
//! `docs/decisions/0065` has described since it was written —
//! `World → Place → Object` — which no type in this repository has ever had.
//!
//! # Why this adds no second way to navigate
//!
//! The task's constraint is the whole of its design: **if the World needs a
//! switcher, this task has failed rather than found a requirement.** So there is
//! no switcher here, no mode, no new binding, and — this is the part worth
//! reading — **no new arithmetic.**
//!
//! A Place seen in the World is a **tile**: a point and a size on a plane. That
//! is a [`Frame`](crate::Frame), which this crate already has, and fitting a set
//! of them into a viewport is [`reached_by`](crate::plane::reached_by) followed by
//! [`Camera::showing`], which this crate already does for the frames on one
//! Place. **The World fits its Places exactly as a Place fits its frames**, and a
//! person who has learned to pan and zoom one has learned the other without being
//! told.
//!
//! # Where the boundary is, and why no existing number moved
//!
//! [`Zoom::FURTHEST_OUT`] is *the furthest out a person may look* **within a
//! Place**, and `one_step_out` at that rung answers [`None`] — a step with nowhere
//! to go. The World is what that step reaches. Nothing above it changes meaning,
//! no constant moved, and `Show all` still fits one Place's frames.
//!
//! That matters more than it reads. The obvious implementation lowers
//! `FURTHEST_OUT` so the World is just a further-out plane, and it is wrong twice:
//! it would multiply `as_far_as_show_all_reaches` by five — a test asserts that
//! extent to the pixel — and it would claim that **Places share one coordinate
//! space**, which they cannot. A Place is an *endless* surface. Two endless
//! surfaces do not sit side by side on a plane; `(4200, 0)` exists on both. The
//! World is a level above, not a wider view of the same level, and the tile a
//! Place occupies in it is a *representation* of that Place rather than its
//! extent.

use crate::{At, Camera, Frame, Place, Size, Span, plane};

/// The notional surface the World's own tiles are laid out on.
///
/// **A tile is not on the Place it stands for**, and conflating those was the
/// first version of this file: `plane::reached_by` folds the frames *on one
/// Place*, so tiles each carrying their own Place would have folded one tile and
/// answered that the World reaches as far as whichever Place came first. Silently,
/// with correct arithmetic, about the wrong set.
///
/// So every tile sits on **one** surface — this one — and the Place it represents
/// is carried as its [`Frame::id`], which is the Place's own number. One tile, one
/// identity, nothing to keep in step.
const THE_WORLDS_OWN_SURFACE: Place = Place::FIRST;

/// What a person is looking at: one Place, or all of them.
///
/// **Returned rather than stored**, because it is a consequence of the zoom and
/// not a mode anybody sets. A field would be a second thing that can disagree
/// with the camera, which is the fault `one-plane-two-vocabularies` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Showing {
    /// Every Place at once. Reached by stepping out from [`Zoom::FURTHEST_OUT`].
    TheWorld,
    /// One Place, filling the screen, with its own frames on it.
    OnePlace(Place),
}

/// Every Place a person has, and where each sits when they are all seen.
///
/// The order Places were made in is not the order they are laid out in, and this
/// type holds the layout rather than deriving it: where a Place sits in the World
/// is a thing a person may come to rely on, so it is data rather than a function
/// of a `BTreeMap`'s ordering.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct World {
    /// One tile per Place, in the order they are laid out.
    tiles: Vec<Frame>,
}

impl World {
    /// A World with no Places in it, which is a machine nobody has used yet.
    #[must_use]
    pub const fn new() -> Self {
        Self { tiles: Vec::new() }
    }

    /// This World with a Place laid out at this point, at this size.
    ///
    /// The tile carries the Place's own identity as its [`Frame::id`] — not a
    /// second numbering — so a tile and the Place it stands for cannot drift
    /// apart. `id` is the Place's number and `place()` is the Place itself; both
    /// answer the same question and neither can be set without the other.
    #[must_use]
    pub fn with(mut self, place: Place, at: At, size: Size) -> Self {
        self.tiles
            .push(Frame::of(place.number(), THE_WORLDS_OWN_SURFACE, at, size));
        self
    }

    /// How many Places this World holds.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.tiles.len()
    }

    /// Every Place in it, in the order they are laid out.
    ///
    /// Read back through [`Place::numbered`] rather than stored twice — the tile's
    /// id *is* the Place's number, and zero is not a Place, so a tile that could
    /// not name one is not one this World put there.
    pub fn each(&self) -> impl Iterator<Item = Place> + '_ {
        self.tiles
            .iter()
            .filter_map(|tile| Place::numbered(tile.id()))
    }

    /// How far the Places reach together, which is what seeing them all must fit.
    ///
    /// [`None`] for a World with no Places. **The same function a Place uses for
    /// its frames**, which is the whole of why this is not a second navigation
    /// model: `plane::reached_by` is given the tiles and asked for their span, and
    /// it does not know or care that these stand for Places.
    #[must_use]
    pub fn reached_by(&self) -> Option<Span> {
        plane::reached_by(&self.tiles, THE_WORLDS_OWN_SURFACE)
    }

    /// The Place whose tile holds this point, or [`None`] between them.
    ///
    /// How zooming in from the World chooses a Place: the one under the pointer,
    /// which is the same answer a press on a frame gets from
    /// [`Camera::inside`]. Front-most last, so a later tile wins an overlap the
    /// way a later frame does.
    #[must_use]
    pub fn the_place_at(&self, point: At) -> Option<Place> {
        self.tiles
            .iter()
            .rev()
            .find(|tile| Camera::inside(**tile, point).is_some())
            .and_then(|tile| Place::numbered(tile.id()))
    }

    /// A camera that shows every Place at once in a viewport this size.
    ///
    /// [`None`] with no Places, or where they are spread further than a canvas
    /// can zoom out to hold — refused rather than shown as a view that moved
    /// somewhere arbitrary, which is what `Camera::showing` already promises for
    /// the frames on one Place.
    #[must_use]
    pub fn showing(&self, viewport: Size) -> Option<Camera> {
        Camera::showing(self.reached_by()?, viewport)
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;
