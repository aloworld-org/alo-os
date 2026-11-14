//! **Wait for the screen to change, then to settle**, and say which of the
//! outcomes happened — so *nothing changed* is a sentence, never a silence.
//!
//! The idea is `w26-key.sh`'s, from the walks done by hand under emulation: a
//! picture taken right after a key showed the screen before the key by an
//! unpredictable amount, and on 2026-11-12 that produced one wrong cause. So a
//! picture is taken before the keys, and pictures are taken after them until
//! one differs from the first and the next ones match it. Three outcomes:
//!
//! - **changed and settled** — the key did something and it finished;
//! - **changed, never settled** — something keeps moving;
//! - **unchanged** — the key had no visible effect within the time allowed.
//!
//! A few pixels may differ between two pictures of the same screen — a caret
//! blinking — so a difference up to a stated noise floor counts as none.

use std::time::Duration;

use crate::frame::Frame;
use crate::machine::{Machine, MachineError};

/// How a look waits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settling {
    /// How long between pictures.
    pub every: Duration,
    /// How many pictures in a row must match for the screen to have settled.
    pub matching: u32,
    /// How many pixels may differ between two pictures that count as the same.
    pub noise: usize,
    /// The longest a look waits.
    pub at_most: Duration,
}

impl Settling {
    /// A look under KVM: a picture a second, settled after three that match,
    /// a caret's worth of noise, and a minute at most.
    pub const UNDER_KVM: Self = Self {
        every: Duration::from_secs(1),
        matching: 3,
        noise: 64,
        at_most: Duration::from_secs(60),
    };

    /// The same, waiting at most `at_most`.
    #[must_use]
    pub const fn at_most(self, at_most: Duration) -> Self {
        Self { at_most, ..self }
    }
}

/// What a look saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    /// It changed from the picture before, and stopped changing.
    ChangedAndSettled,
    /// It changed from the picture before, and kept changing.
    ChangedNeverSettled,
    /// It never differed from the picture before.
    Unchanged,
    /// With no picture before to compare, it stopped changing.
    Settled,
    /// With no picture before to compare, it never stopped changing.
    NeverSettled,
}

impl Seen {
    /// The outcome as a report says it.
    #[must_use]
    pub fn said(self) -> &'static str {
        match self {
            Self::ChangedAndSettled => "changed and settled",
            Self::ChangedNeverSettled => "changed and never settled",
            Self::Unchanged => "unchanged",
            Self::Settled => "settled",
            Self::NeverSettled => "never settled",
        }
    }
}

/// A look's outcome, how long it took, and the last picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Looked {
    /// What it saw.
    pub seen: Seen,
    /// How long it took.
    pub took: Duration,
    /// The last picture it took.
    pub last: Frame,
}

/// Look until the screen changes from `before` and settles — or, with no
/// picture before, until it settles.
///
/// # Errors
///
/// When the machine cannot be photographed.
pub fn look(
    machine: &mut impl Machine,
    before: Option<&Frame>,
    rule: Settling,
) -> Result<Looked, MachineError> {
    let start = machine.elapsed();
    let mut last: Option<Frame> = None;
    let mut changed = false;
    let mut matching = 0_u32;
    loop {
        machine.wait(rule.every);
        let now = machine.frame()?;
        let took = machine.elapsed().saturating_sub(start);
        let differs = before.is_none_or(|it| now.differs_from(it) > rule.noise);
        if differs {
            changed = true;
            let same_as_last = last
                .as_ref()
                .is_some_and(|it| now.differs_from(it) <= rule.noise);
            matching = if same_as_last { matching + 1 } else { 0 };
            if matching + 1 >= rule.matching {
                let seen = if before.is_some() {
                    Seen::ChangedAndSettled
                } else {
                    Seen::Settled
                };
                return Ok(Looked {
                    seen,
                    took,
                    last: now,
                });
            }
        }
        last = Some(now);
        if took >= rule.at_most {
            let seen = match (before.is_some(), changed) {
                (true, true) => Seen::ChangedNeverSettled,
                (true, false) => Seen::Unchanged,
                (false, _) => Seen::NeverSettled,
            };
            let Some(last) = last else {
                return Err(MachineError("a look took no picture".to_owned()));
            };
            return Ok(Looked { seen, took, last });
        }
    }
}

#[cfg(test)]
#[path = "settling_tests.rs"]
mod tests;
