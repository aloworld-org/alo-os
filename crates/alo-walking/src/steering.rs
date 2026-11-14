//! **Steering the pointer onto a place by looking**: put it there, find it on
//! the screen, and put it again by what it missed by, until it is there.
//!
//! The walk puts the pointer at an absolute place (`crate::pointer_moves`), but
//! the place a guest draws it at is the guest's own mapping of qemu's range
//! onto its display, and a pixel or two off is its to decide. So each round
//! reads the pointer back off the screen, and the next round aims as far the
//! other way as the last one missed.
//!
//! **Lost is its own outcome.** A pointer that cannot be found after a move is
//! not *probably where it was asked to go*: a check that went on from there
//! would be checking a place the pointer may never have reached.

use std::time::Duration;

use crate::frame::Frame;
use crate::machine::{Machine, MachineError};

/// How close counts as there, in pixels on each axis.
pub const CLOSE: usize = 4;

/// The most rounds a steer takes.
pub const ROUNDS: u32 = 8;

/// The wait after a move before the screen is read: the compositor draws the
/// pointer on its next frame.
pub const BEFORE_LOOKING: Duration = Duration::from_millis(700);

/// How a steer ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Steered {
    /// The pointer is within [`CLOSE`] of the place.
    There {
        /// Where it is.
        at: (usize, usize),
        /// How many rounds it took.
        rounds: u32,
    },
    /// After a move, the pointer could not be found.
    Lost {
        /// Where it was last seen.
        last: (usize, usize),
        /// In which round.
        rounds: u32,
    },
    /// Every round was taken and it is still not there.
    NotThere {
        /// Where it is.
        at: (usize, usize),
    },
}

/// Steer the pointer from `from` onto `to`, finding it after each move with
/// `find`, which is given the picture and where the pointer should now be.
///
/// # Errors
///
/// When the machine cannot be moved or photographed.
pub fn steer(
    machine: &mut impl Machine,
    from: (usize, usize),
    to: (usize, usize),
    find: impl Fn(&Frame, (usize, usize)) -> Option<(usize, usize)>,
) -> Result<Steered, MachineError> {
    let mut at = from;
    let mut aim = to;
    for round in 0..ROUNDS {
        if at.0.abs_diff(to.0) <= CLOSE && at.1.abs_diff(to.1) <= CLOSE {
            return Ok(Steered::There { at, rounds: round });
        }
        put(machine, aim)?;
        let frame = machine.frame()?;
        let Some(now) = find(&frame, to) else {
            return Ok(Steered::Lost {
                last: at,
                rounds: round + 1,
            });
        };
        aim = (corrected(aim.0, to.0, now.0), corrected(aim.1, to.1, now.1));
        at = now;
    }
    if at.0.abs_diff(to.0) <= CLOSE && at.1.abs_diff(to.1) <= CLOSE {
        return Ok(Steered::There { at, rounds: ROUNDS });
    }
    Ok(Steered::NotThere { at })
}

/// Put the pointer at `at`, and give the compositor time to draw it there.
///
/// # Errors
///
/// When the machine did not take it.
pub fn put(machine: &mut impl Machine, at: (usize, usize)) -> Result<(), MachineError> {
    machine.point_at(at)?;
    machine.wait(BEFORE_LOOKING);
    Ok(())
}

/// The next aim on one axis: the last one, moved by what it missed `to` by.
fn corrected(aim: usize, to: usize, landed: usize) -> usize {
    let next = signed(aim) + signed(to) - signed(landed);
    usize::try_from(next.max(0)).unwrap_or(0)
}

/// A position as a signed number, to subtract.
fn signed(it: usize) -> i64 {
    i64::try_from(it).unwrap_or(i64::MAX)
}

#[cfg(test)]
#[path = "steering_tests.rs"]
mod tests;
