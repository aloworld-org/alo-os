//! How far alo has got with the task it was given.
//!
//! Part of task 7 of `docs/autonomy/putting-a-window-aside.md`: a person who put a window
//! away is owed **progress** without having to bring it back.
//!
//! # Hundredths, checked, and never a guess
//!
//! Held as hundredths of the whole, refused outside `0..=100`. A number above a hundred is
//! a surface drawing a bar past its own end, and this repository has already paid for
//! values that were plausible rather than checked — `alo-arranging` rebuilds every restored
//! value through its checked constructor for the same reason.
//!
//! **There is no `Default`.** Zero is a real answer — *it has begun and done nothing* — and
//! a defaulted progress would be indistinguishable from it while meaning *nobody said*. A
//! surface cannot tell those apart, and one of them is a bug in whoever forgot to fill it
//! in.
//!
//! # Why not a fraction of an unknown total
//!
//! Because a task whose total nobody knows is a task with no progress to report, and a
//! surface handed *step 4* with no denominator draws a number that looks like progress and
//! is not. If alo cannot say how far through it is, [`crate::alo_at_work::AtWork`] carries
//! no progress rather than a figure with no meaning.

/// Why a progress could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotProgress {
    /// More than the whole.
    #[error("{0} hundredths is past the end of the task, and a bar cannot be drawn past its own")]
    PastTheEnd(u8),
}

/// How far through, in hundredths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Progress(u8);

impl Progress {
    /// Begun, and nothing done yet. **A real answer**, not an absence.
    pub const JUST_BEGUN: Self = Self(0);
    /// Finished.
    pub const ALL_OF_IT: Self = Self(100);

    /// This many hundredths of the way through.
    ///
    /// # Errors
    ///
    /// [`NotProgress::PastTheEnd`] above a hundred, **by name rather than clamped**: a
    /// clamp would turn a caller's arithmetic mistake into a full bar, which is the most
    /// convincing wrong answer available.
    pub const fn of(hundredths: u8) -> Result<Self, NotProgress> {
        if hundredths > 100 {
            return Err(NotProgress::PastTheEnd(hundredths));
        }
        Ok(Self(hundredths))
    }

    /// How far through, in hundredths.
    #[must_use]
    pub const fn hundredths(self) -> u8 {
        self.0
    }

    /// Whether there is nothing left to do.
    #[must_use]
    pub const fn is_all_of_it(self) -> bool {
        self.0 == 100
    }
}
