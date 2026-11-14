//! **Steering the pointer onto a place by looking**: move, find it on the screen,
//! move again by what is left, until it is there.
//!
//! A relative move is accelerated by libinput, and how much depends on how fast
//! the steps arrive, so what was asked is never where the pointer went. Measured
//! by hand on 2026-10-18 under emulation, the same mouse in one session moved
//! 0.63, 1.82 and 0.95 times what it was asked — once, on a lane machine.
//! So each round reads the pointer back off the screen, and the next round asks
//! for what is left, divided by how much the last round was multiplied.
//!
//! **Lost is its own outcome.** A pointer that cannot be found after a move is
//! not *probably where it was asked to go*: a check that went on from there
//! would be checking a place the pointer may never have reached.

use std::time::Duration;

use crate::frame::Frame;
use crate::machine::{Machine, MachineError};
use crate::pointer_moves;

/// How close counts as there, in pixels on each axis.
pub const CLOSE: usize = 4;

/// The most rounds a steer takes.
pub const ROUNDS: u32 = 12;

/// The wait after each step, so the steps arrive as a hand's do.
pub const BETWEEN_STEPS: Duration = Duration::from_millis(30);

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
    // **The first move asks for half**, as though it will be doubled: an
    // overshoot can push the pointer against the screen's edge, where most of
    // it is cut off and it cannot be found, and an undershoot costs one round.
    let mut gain = (2.0_f64, 2.0_f64);
    for round in 0..ROUNDS {
        if at.0.abs_diff(to.0) <= CLOSE && at.1.abs_diff(to.1) <= CLOSE {
            return Ok(Steered::There { at, rounds: round });
        }
        let left = (signed(to.0) - signed(at.0), signed(to.1) - signed(at.1));
        let ask = (asked(left.0, gain.0), asked(left.1, gain.1));
        move_by(machine, ask)?;
        machine.wait(BEFORE_LOOKING);
        let frame = machine.frame()?;
        let Some(now) = find(&frame, to) else {
            return Ok(Steered::Lost {
                last: at,
                rounds: round + 1,
            });
        };
        let moved = (signed(now.0) - signed(at.0), signed(now.1) - signed(at.1));
        gain = (
            learned(gain.0, ask.0, moved.0),
            learned(gain.1, ask.1, moved.1),
        );
        at = now;
    }
    if at.0.abs_diff(to.0) <= CLOSE && at.1.abs_diff(to.1) <= CLOSE {
        return Ok(Steered::There { at, rounds: ROUNDS });
    }
    Ok(Steered::NotThere { at })
}

/// Move by `(dx, dy)` in steps the mouse carries whole.
///
/// # Errors
///
/// When the machine did not take a step.
pub fn move_by(machine: &mut impl Machine, (dx, dy): (i32, i32)) -> Result<(), MachineError> {
    for (x, y) in pointer_moves::in_steps(dx, dy) {
        machine.nudge(x, y)?;
        machine.wait(BETWEEN_STEPS);
    }
    Ok(())
}

/// What to ask for to move `left`, when the last move was multiplied by `gain`.
fn asked(left: i64, gain: f64) -> i32 {
    if left == 0 {
        return 0;
    }
    let ask = (left as f64 / gain).round() as i32;
    if ask == 0 { left.signum() as i32 } else { ask }
}

/// How much a move was multiplied, learned from one round: kept as it was
/// where the round says nothing about it, and never past a quarter or four.
fn learned(gain: f64, ask: i32, moved: i64) -> f64 {
    if ask == 0 || moved == 0 || (moved > 0) != (ask > 0) {
        return gain;
    }
    let ratio = moved as f64 / f64::from(ask);
    ratio.clamp(0.25, 4.0)
}

/// A position as a signed number, to subtract.
fn signed(it: usize) -> i64 {
    i64::try_from(it).unwrap_or(i64::MAX)
}

#[cfg(test)]
#[path = "steering_tests.rs"]
mod tests;
