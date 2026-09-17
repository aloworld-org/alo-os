//! One lane's gates at a time on one machine.
//!
//! Two supervisors in two checkouts on the same machine are allowed, and are how
//! a machine runs two lanes. Their workers write in parallel without trouble.
//! Their **gates** do not: the whole-workspace build and test run uses every
//! processor it can find, so two at once each take about twice as long. On
//! 2026-09-16 the third PC's two lanes gated together for most of the day. Each
//! gate took longer than the gap between other machines' pushes, so both kept
//! losing the race, and each lost race cost another whole gate. A person pausing
//! one lane by hand was the only thing that got work published.
//!
//! So publishing takes a turn. From the first gate on a task's tree to the push
//! that publishes it, a supervisor holds one machine-wide lock. A second one
//! waits there, saying so in its log, and gates when the first has pushed or
//! stopped. The lock is [`crate::lock::Held`], taken in a directory every
//! checkout on the machine shares, so it has the same property as the checkout
//! lock: a supervisor that is killed releases it, because the operating system
//! does or its process is gone.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::lock;

/// How long a supervisor waiting for its turn sleeps between asking.
///
/// A gate takes tens of minutes, so asking twice a minute costs nothing and
/// loses at most half a minute of a turn.
pub const ASKING_EVERY: Duration = Duration::from_secs(30);

/// The directory every checkout on this machine shares for the turn, under the
/// home directory of whoever runs the supervisors.
const THE_DIRECTORY: &str = ".alo-kernel-loop-gates";

/// The words [`crate::lock::Held::taken`] begins with when somebody else holds
/// the lock, on every platform.
const HELD_BY_ANOTHER: &str = "another loop is running";

/// A turn at the gates, held for as long as this value is.
#[derive(Debug)]
pub struct Turn {
    /// The machine-wide lock, or nothing for a turn nobody needs to take.
    _held: Option<lock::Held>,
}

impl Turn {
    /// A turn that holds nothing, for steps that do not run real gates.
    #[cfg(test)]
    #[must_use]
    pub const fn nobody_elses() -> Self {
        Self { _held: None }
    }
}

/// Where this machine's gate turn is kept.
///
/// # Errors
/// A sentence when the home directory is not known or the directory cannot be
/// made.
pub fn this_machines() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| {
            "neither USERPROFILE nor HOME is set, so there is nowhere every checkout on this \
             machine would find the gate turn"
                .to_owned()
        })?;
    let at = PathBuf::from(home).join(THE_DIRECTORY);
    std::fs::create_dir_all(&at)
        .map_err(|why| format!("{} could not be made: {why}", at.display()))?;
    Ok(at)
}

/// Take the turn in `at`, waiting while another supervisor has it.
///
/// `said` is told once, when waiting begins, so the log says why this
/// supervisor went quiet. `at_most` bounds the number of times it asks; `None`
/// asks until it gets the turn, which is what a real supervisor wants.
///
/// # Errors
/// Whatever the lock said when it was not held by somebody else, and a sentence
/// when `at_most` asks all ran out.
pub fn taken(
    at: &Path,
    every: Duration,
    at_most: Option<u32>,
    said: &mut dyn FnMut(&str),
) -> Result<Turn, String> {
    let mut asked = 0_u32;
    loop {
        match lock::Held::taken(at) {
            Ok(held) => {
                if asked > 0 {
                    said("it is this lane's turn at the gates now");
                }
                return Ok(Turn { _held: Some(held) });
            }
            Err(why) if why.starts_with(HELD_BY_ANOTHER) => {
                if asked == 0 {
                    said(
                        "another lane on this machine is gating or publishing, so this one waits \
                         for its turn rather than gating beside it at half the speed",
                    );
                }
                asked = asked.saturating_add(1);
                if at_most.is_some_and(|most| asked >= most) {
                    return Err(format!(
                        "another lane on this machine held the gates for all {asked} times this \
                         asked"
                    ));
                }
                std::thread::sleep(every);
            }
            Err(why) => return Err(why),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A directory of this test's own, gone when the test is.
    fn a_directory(named: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!(
            "alo-kernel-loop-gate-turn-{named}-{}",
            std::process::id()
        ));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// **A turn nobody holds is taken at once, and nothing is said.**
    #[test]
    fn a_free_turn_is_taken_without_waiting() {
        let at = a_directory("free");
        let mut heard = Vec::new();
        let turn = taken(&at, Duration::from_millis(1), Some(1), &mut |said| {
            heard.push(said.to_owned());
        });
        assert!(turn.is_ok(), "{turn:?}");
        assert!(heard.is_empty(), "{heard:?}");
        drop(turn);
        drop(std::fs::remove_dir_all(&at));
    }

    /// **While one lane holds the turn, another waits and says so once**, and
    /// does not gate.
    #[test]
    fn a_held_turn_makes_the_next_lane_wait_and_say_so() {
        let at = a_directory("held");
        let first = taken(&at, Duration::from_millis(1), Some(1), &mut |_| {}).unwrap();

        let mut heard = Vec::new();
        let second = taken(&at, Duration::from_millis(1), Some(3), &mut |said| {
            heard.push(said.to_owned());
        });
        assert!(
            second.is_err(),
            "a second lane gated beside the first: {second:?}"
        );
        assert_eq!(heard.len(), 1, "{heard:?}");
        assert!(heard.first().unwrap().contains("waits for its turn"));

        drop(first);
        drop(std::fs::remove_dir_all(&at));
    }

    /// **When the first lane lets go, the next one gets its turn.**
    #[test]
    fn a_turn_let_go_is_taken_by_the_next_lane() {
        let at = a_directory("released");
        let first = taken(&at, Duration::from_millis(1), Some(1), &mut |_| {}).unwrap();
        drop(first);

        let next = taken(&at, Duration::from_millis(1), Some(1), &mut |_| {});
        assert!(next.is_ok(), "a released turn was not taken: {next:?}");
        drop(next);
        drop(std::fs::remove_dir_all(&at));
    }
}
