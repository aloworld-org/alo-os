//! Where things are on the plane, in signed logical units.
//!
//! **Nothing here is a pixel and nothing here knows about a camera.** A plane
//! coordinate is where a frame is on the surface a person moves; what that
//! becomes on a screen is [`crate::camera`]'s answer and is asked in one
//! direction only.
//!
//! Signed, because panning is what a plane is for: a person who drags the
//! surface to the right puts frames at negative coordinates, and a plane with a
//! wall at zero would be a plane with a corner. Whole units, because two frames
//! that touch should touch, and a fraction of a unit is a seam waiting to appear
//! at some zoom.

use crate::place::Place;

/// The furthest from the origin a frame may sit, in logical units.
///
/// **Not a wall a person can reach.** A million units either way is a plane
/// about twenty metres across at the size a window is drawn, which nobody pans
/// to by accident; what it buys is that every sum of two coordinates, and every
/// span, stays inside an `i32` so the arithmetic here cannot overflow. Task 8 —
/// *a frame is never lost* — is about a person's reach and is a different
/// question from this one.
pub const FURTHEST: i32 = 1_000_000;

/// One point on the plane, in logical units from the plane's origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct At {
    /// Units right of the origin; negative is left of it.
    pub x: i32,
    /// Units below the origin; negative is above it.
    pub y: i32,
}

impl At {
    /// This point, if it is on the plane at all.
    ///
    /// # Errors
    /// [`None`] for a coordinate past [`FURTHEST`] on either axis. Refused
    /// rather than clamped: a frame silently moved to a wall is a frame a
    /// person put somewhere and found somewhere else.
    #[must_use]
    pub fn checked(x: i32, y: i32) -> Option<Self> {
        (x.abs() <= FURTHEST && y.abs() <= FURTHEST).then_some(Self { x, y })
    }

    /// The origin.
    #[must_use]
    pub const fn origin() -> Self {
        Self { x: 0, y: 0 }
    }

    /// This point moved by these units, or [`None`] where that leaves the plane.
    #[must_use]
    pub fn moved_by(self, x: i32, y: i32) -> Option<Self> {
        Self::checked(self.x.checked_add(x)?, self.y.checked_add(y)?)
    }
}

/// How big something is, in logical units.
///
/// Never zero along either axis: a frame with no width is not a frame, and a
/// viewport with no height is a display nobody has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
    /// Units across.
    width: u32,
    /// Units down.
    height: u32,
}

impl Size {
    /// This size, if it is one.
    ///
    /// # Errors
    /// [`None`] for zero along either axis, or for a side past [`FURTHEST`].
    #[must_use]
    pub fn checked(width: u32, height: u32) -> Option<Self> {
        let furthest = FURTHEST.unsigned_abs();
        (width > 0 && height > 0 && width <= furthest && height <= furthest)
            .then_some(Self { width, height })
    }

    /// Units across.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Units down.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }
}

/// How far a set of frames reaches, which is what *Show all* has to fit.
///
/// **Computed from the frames rather than declared.** The plan's task 6 says so
/// outright: *the extent `Show all` fits is the frames' own, computed from them.
/// A plane with declared bounds that the frames can sit outside is how the design
/// file's own fit would have been wrong.* So there is no stored extent here to
/// drift from what is actually open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The top-left corner of everything.
    pub from: At,
    /// The bottom-right corner of everything.
    pub to: At,
}

/// One application's frame on the plane.
///
/// This is a rectangle, an identity and **which surface it is on**, and
/// deliberately nothing else. What is *inside* it is the application's, which is
/// the constraint task 2 states: the application is never told about the canvas —
/// it is told its size and gets its events, as it would on any compositor. It is
/// not told which Place it is on either, for the same reason.
///
/// # Why the Place is a field and not an argument to the functions that need one
///
/// `the-canvas-and-its-places.md` task 3: **a frame belongs to exactly one Place
/// at every moment.** There is no *between* — a frame in flight has a Place, and
/// it is the one it started on until it has the one it ends on. A field is how
/// that is said in a type: there is no way to hold a `Frame` that is on no
/// surface, and no way to ask where one is without being told which plane the
/// answer is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    /// Which frame this is, as the compositor numbers its windows.
    id: u64,
    /// Which surface it is on.
    place: Place,
    /// Its top-left corner on that surface.
    at: At,
    /// How big it is, in plane units.
    size: Size,
}

impl Frame {
    /// A frame of this size, at this point, on this Place.
    #[must_use]
    pub const fn of(id: u64, place: Place, at: At, size: Size) -> Self {
        Self {
            id,
            place,
            at,
            size,
        }
    }

    /// Which frame this is.
    #[must_use]
    pub const fn id(self) -> u64 {
        self.id
    }

    /// Which surface it is on.
    #[must_use]
    pub const fn place(self) -> Place {
        self.place
    }

    /// The same frame, on another Place.
    ///
    /// The whole of what moving a window between surfaces does to its geometry:
    /// **nothing.** Task 3's *the work goes with it* is this — a frame keeps its
    /// point and its size and changes which plane those are on, so a window
    /// dragged into another Place is the same window and not a copy of one.
    #[must_use]
    pub const fn now_on(mut self, place: Place) -> Self {
        self.place = place;
        self
    }

    /// Its top-left corner.
    #[must_use]
    pub const fn at(self) -> At {
        self.at
    }

    /// How big it is.
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    /// The corner opposite [`Self::at`], or [`None`] where it leaves the plane.
    #[must_use]
    pub fn opposite(self) -> Option<At> {
        self.at.moved_by(
            i32::try_from(self.size.width()).ok()?,
            i32::try_from(self.size.height()).ok()?,
        )
    }
}

/// How far the frames **on this Place** reach together, or [`None`] where there
/// are none there.
///
/// The answer *Show all* fits, and the reason it is a function of the frames
/// rather than a field on anything.
///
/// # Why it takes a Place rather than a slice somebody has already filtered
///
/// Until Places existed this folded every frame it was given into one extent,
/// which was right when there was one surface and became wrong in silence the
/// moment there were two: *Show all* would have zoomed out far enough to fit a
/// window on a surface the person is not looking at, and the window they wanted
/// would have been a speck. Nothing would have failed — the arithmetic is the
/// same arithmetic — which is why the Place is a parameter here rather than a
/// filter at the call site that one caller could forget.
#[must_use]
pub fn reached_by(frames: &[Frame], place: Place) -> Option<Span> {
    let mut frames = frames.iter().filter(|frame| frame.place() == place);
    let first = frames.next()?;
    let mut span = Span {
        from: first.at(),
        to: first.opposite()?,
    };
    for frame in frames {
        let opposite = frame.opposite()?;
        span.from = At {
            x: span.from.x.min(frame.at().x),
            y: span.from.y.min(frame.at().y),
        };
        span.to = At {
            x: span.to.x.max(opposite.x),
            y: span.to.y.max(opposite.y),
        };
    }
    Some(span)
}

#[cfg(test)]
#[path = "plane_tests.rs"]
mod tests;
