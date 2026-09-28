//! What a person is looking at: where the viewport sits on the plane, and how
//! far in.
//!
//! **This is the only transform in the canvas, and it goes one way round.** The
//! plane is asked where a point lands on the screen; the viewport is never asked
//! to correct for anything. A dock that subtracted a pan to stay still would be
//! drawn in the plane and compensating, which the plan's constraint names as the
//! wrong layer — and it would pass a test that only checked where the dock ended
//! up.
//!
//! # The zoom is thousandths of an integer
//!
//! A click has to reach the right coordinate inside an application's surface at
//! 40 % and at 250 % (task 2), and *nearly the right pixel* in a drawing program
//! is a wrong pixel. Floating point gives different answers depending on the
//! order the sums were done, so a zoom here is `400` or `2500` and every
//! conversion is integer multiplication with one division.
//!
//! What that costs is honest and small: a screen point does not always name one
//! plane unit, because at 250 % two or three screen pixels sit inside one unit.
//! [`Camera::plane_of`] therefore answers *the unit that contains this pixel*,
//! which is the question a click is actually asking.

use crate::plane::{At, Size};

/// Why a zoom is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotZoomed {
    /// Further out than the canvas goes.
    #[error("a zoom of {thousandths} thousandths is further out than this canvas goes")]
    TooFarOut {
        /// What was asked for.
        thousandths: u32,
    },
    /// Further in than the canvas goes.
    #[error("a zoom of {thousandths} thousandths is further in than this canvas goes")]
    TooFarIn {
        /// What was asked for.
        thousandths: u32,
    },
}

/// How far in a person is looking, in thousandths.
///
/// `1000` is life size. The bounds are the plan's two named zooms with room
/// either side, not a preference: a canvas that can be zoomed to nothing has
/// frames a person cannot find, and one that can be zoomed to a thousand times
/// has arithmetic nobody checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Zoom(u32);

impl Zoom {
    /// Life size.
    pub const LIFE_SIZE: Self = Self(1000);
    /// The furthest out a person may look: a twentieth of life size.
    pub const FURTHEST_OUT: u32 = 50;
    /// The furthest in: ten times life size.
    pub const FURTHEST_IN: u32 = 10_000;

    /// This many thousandths, if that is a zoom.
    ///
    /// # Errors
    /// [`NotZoomed`] outside the bounds above, by name rather than clamped: a
    /// person who asked to go further out and was silently not moved has a
    /// canvas that ignores them.
    pub fn of(thousandths: u32) -> Result<Self, NotZoomed> {
        if thousandths < Self::FURTHEST_OUT {
            return Err(NotZoomed::TooFarOut { thousandths });
        }
        if thousandths > Self::FURTHEST_IN {
            return Err(NotZoomed::TooFarIn { thousandths });
        }
        Ok(Self(thousandths))
    }

    /// How far in, in thousandths.
    #[must_use]
    pub const fn thousandths(self) -> u32 {
        self.0
    }

    /// The zooms a step stands on, from furthest out to furthest in.
    ///
    /// **A ladder rather than a multiplier**, for a reason a person notices
    /// rather than a tidiness one. In integer thousandths a repeated multiply
    /// does not come back: stepping in by a factor and out again lands beside
    /// where it started, a little further every time, and a canvas that cannot
    /// return to life size is one somebody stops trusting. On a ladder, in and
    /// then out is exactly where you were, because it is the same two rungs.
    ///
    /// It is also what *return the canvas to 100 %* needs — `docs/design/the-
    /// shortcuts-and-the-edges.md` gives that its own key — because a stop the
    /// steps do not stand on makes that key a jump rather than a step back.
    ///
    /// The ends are [`Self::FURTHEST_OUT`] and [`Self::FURTHEST_IN`], so stepping
    /// can reach the bounds this type already refuses outside of, and the rungs
    /// are roughly a factor of 1.4 apart — close enough that a step feels like a
    /// step and not a jump, far enough that crossing the range does not take
    /// twenty presses.
    ///
    /// **This is not an acceleration.** Every press moves exactly one rung,
    /// however fast they arrive; the plan's task 5 says the same of panning.
    pub const STOPS: [u32; 17] = [
        Self::FURTHEST_OUT,
        70,
        100,
        150,
        200,
        300,
        400,
        500,
        700,
        1000,
        1500,
        2000,
        3000,
        4000,
        5000,
        7000,
        Self::FURTHEST_IN,
    ];

    /// The next rung in, or [`None`] at the furthest in.
    ///
    /// A zoom between rungs — which is where *Show all* leaves one — moves to the
    /// first rung beyond it rather than snapping backwards.
    #[must_use]
    pub fn one_step_in(self) -> Option<Self> {
        Self::STOPS
            .into_iter()
            .find(|stop| *stop > self.0)
            .and_then(|stop| Self::of(stop).ok())
    }

    /// The next rung out, or [`None`] at the furthest out.
    #[must_use]
    pub fn one_step_out(self) -> Option<Self> {
        Self::STOPS
            .into_iter()
            .rev()
            .find(|stop| *stop < self.0)
            .and_then(|stop| Self::of(stop).ok())
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self::LIFE_SIZE
    }
}

/// What a person is looking at.
///
/// [`Self::at`] is the plane point drawn at the viewport's top-left corner, so
/// panning moves this and nothing else. The viewport's own size is not held here:
/// it is the display's, it changes when a screen does, and a camera that
/// remembered it would be a second answer to how big the screen is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Camera {
    /// The plane point under the viewport's top-left corner.
    at: At,
    /// How far in.
    zoom: Zoom,
}

impl Camera {
    /// Looking at the plane's origin, life size.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The plane point under the viewport's top-left corner.
    #[must_use]
    pub const fn at(self) -> At {
        self.at
    }

    /// How far in.
    #[must_use]
    pub const fn zoom(self) -> Zoom {
        self.zoom
    }

    /// Looking at this point instead, at the same zoom.
    ///
    /// # Errors
    /// [`None`] where that point is off the plane.
    #[must_use]
    pub fn looking_at(self, at: At) -> Option<Self> {
        At::checked(at.x, at.y).map(|at| Self { at, ..self })
    }

    /// Panned by this many **screen** pixels.
    ///
    /// Screen rather than plane, because a person drags a mouse and their hand
    /// moves in pixels: the plan's task 3 says a drag of 100 screen pixels at
    /// half zoom moves a frame 200 plane units, and a pan is the same arithmetic
    /// the other way round.
    ///
    /// # Errors
    /// [`None`] where that leaves the plane.
    #[must_use]
    pub fn panned_by(self, x: i32, y: i32) -> Option<Self> {
        let across = self.units_of(x)?;
        let down = self.units_of(y)?;
        self.at.moved_by(across, down).map(|at| Self { at, ..self })
    }

    /// Zoomed to this, keeping the plane point under `held` where it is.
    ///
    /// **Pointer-centred, which is task 6's acceptance**: the point under the
    /// pointer does not move. That is why this takes the screen point to hold
    /// rather than zooming about a corner — a canvas that zoomed about its origin
    /// throws away whatever a person was looking at.
    ///
    /// # Errors
    /// [`None`] where the resulting camera would leave the plane.
    #[must_use]
    pub fn zoomed_to(self, zoom: Zoom, held: (i32, i32)) -> Option<Self> {
        // Where that pixel is on the plane now, and where the new camera must sit
        // for it to be there still.
        let under = self.plane_of(held)?;
        let after = Self { zoom, ..self };
        let would = after.plane_of(held)?;
        after
            .at
            .moved_by(under.x - would.x, under.y - would.y)
            .map(|at| Self { at, zoom })
    }

    /// Where this plane point lands on the screen.
    ///
    /// The one direction this transform runs. [`None`] where the answer will not
    /// fit a screen coordinate, which is a frame panned absurdly far rather than
    /// anything a person does.
    #[must_use]
    pub fn screen_of(self, at: At) -> Option<(i32, i32)> {
        Some((
            self.pixels_of(at.x - self.at.x)?,
            self.pixels_of(at.y - self.at.y)?,
        ))
    }

    /// Which plane unit contains this screen pixel.
    ///
    /// **A unit rather than a point**, and the rustdoc on this module says why: at
    /// 250 % several pixels sit inside one unit, so this answers the question a
    /// click asks — *what did I press on* — and not a fiction about pixels being
    /// units.
    #[must_use]
    pub fn plane_of(self, screen: (i32, i32)) -> Option<At> {
        self.at
            .moved_by(self.units_of(screen.0)?, self.units_of(screen.1)?)
    }

    /// Where this plane point lands inside a frame's own surface.
    ///
    /// Task 2's arithmetic: a press at a screen point has to arrive at the right
    /// surface-local coordinate. This is the plane-to-surface half, kept here
    /// because it is the same transform and belongs beside it rather than in a
    /// compositor that would do it slightly differently.
    ///
    /// [`None`] where the point is not inside the frame at all, which is a caller
    /// asking about the wrong frame rather than a coordinate to clamp.
    #[must_use]
    pub fn inside(frame: crate::plane::Frame, at: At) -> Option<(u32, u32)> {
        let across = at.x.checked_sub(frame.at().x)?;
        let down = at.y.checked_sub(frame.at().y)?;
        let (across, down) = (u32::try_from(across).ok()?, u32::try_from(down).ok()?);
        (across < frame.size().width() && down < frame.size().height()).then_some((across, down))
    }

    /// The camera that shows all of this, centred, with nothing clipped.
    ///
    /// *Show all*, and the plan's task 6 constraint is the whole of its design:
    /// **the extent is the frames' own**, arrived at here as a
    /// [`crate::plane::Span`] from [`crate::plane::reached_by`] rather than read
    /// off a declared bound the frames are free to sit outside.
    ///
    /// The zoom is the largest rung-free value that fits — this does not step on
    /// [`Zoom::STOPS`], because a ladder that had to contain the exact fit for
    /// every arrangement of frames would not be a ladder. Stepping afterwards
    /// picks up from the rung beyond wherever this left it.
    ///
    /// # Nothing is clipped, by the arithmetic rather than by a margin
    ///
    /// The integer divisions all floor, and they floor in the safe direction.
    /// `zoom` is `room * 1000 / used`, so `used * zoom <= room * 1000`: the span
    /// fits with a remainder rather than overhanging by one. The centring then
    /// spends only half of what [`Self::sees`] says is actually visible at that
    /// zoom, and `sees` is floored the same way, so the far edge lands at or
    /// inside the viewport's own edge and never a pixel past it.
    ///
    /// # Errors
    /// [`None`] where the frames are spread so far that even
    /// [`Zoom::FURTHEST_OUT`] cannot hold them, which is refused rather than
    /// shown clipped: *Show all* that quietly left a frame off the screen would
    /// be the one promise this function makes, broken silently.
    #[must_use]
    pub fn showing(span: crate::plane::Span, viewport: Size) -> Option<Self> {
        let used = |from: i32, to: i32| u32::try_from(to.checked_sub(from)?).ok();
        let (across, down) = (used(span.from.x, span.to.x)?, used(span.from.y, span.to.y)?);
        // A span with no extent in one direction constrains nothing in it.
        // `Size::checked` refuses a zero side, so no arrangement of real frames
        // reaches this; it is here because a division is not the place to find
        // out, and `Span` is a plain pair of corners anybody may build.
        let fitting = |used: u32, room: u32| -> Option<u32> {
            if used == 0 {
                return Some(Zoom::FURTHEST_IN);
            }
            u32::try_from(u64::from(room).checked_mul(1000)? / u64::from(used)).ok()
        };
        let zoom = fitting(across, viewport.width())?
            .min(fitting(down, viewport.height())?)
            .min(Zoom::FURTHEST_IN);
        let zoom = Zoom::of(zoom).ok()?;
        // Centred: half the room left over, in plane units, on each side.
        let seen = Self {
            at: span.from,
            zoom,
        }
        .sees(viewport)?;
        let spare = |seen: u32, used: u32| i32::try_from(seen.saturating_sub(used) / 2).ok();
        let at = span
            .from
            .moved_by(-spare(seen.width(), across)?, -spare(seen.height(), down)?)?;
        Some(Self { at, zoom })
    }

    /// How big the plane's visible part is, for a viewport of this size.
    ///
    /// Used by *Show all* to work out how far out to go. The viewport's size is
    /// passed in rather than held: see [`Camera`].
    #[must_use]
    pub fn sees(self, viewport: Size) -> Option<Size> {
        Size::checked(
            u32::try_from(self.units_of(i32::try_from(viewport.width()).ok()?)?).ok()?,
            u32::try_from(self.units_of(i32::try_from(viewport.height()).ok()?)?).ok()?,
        )
    }

    /// These plane units as screen pixels, at this zoom.
    fn pixels_of(self, units: i32) -> Option<i32> {
        i32::try_from(
            i64::from(units)
                .checked_mul(i64::from(self.zoom.thousandths()))?
                .checked_div(1000)?,
        )
        .ok()
    }

    /// These screen pixels as plane units, at this zoom.
    fn units_of(self, pixels: i32) -> Option<i32> {
        i32::try_from(
            i64::from(pixels)
                .checked_mul(1000)?
                .checked_div(i64::from(self.zoom.thousandths()))?,
        )
        .ok()
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
