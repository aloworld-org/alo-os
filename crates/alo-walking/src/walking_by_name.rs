//! **A row of Settings found by its name**: press a key until Settings says its
//! keyboard is on the row or field wanted (`alo_acting::said_in_settings`).
//!
//! Counting rows broke as soon as a row was added above the one counted to:
//! main's walk at 9ced5c40 (run 44851000278) pressed Down five times for *Add a
//! provider* and landed on *alo* (picture 54). The shell now says on the journal,
//! after each key Settings takes, the name of what the keyboard is on, so this
//! presses, waits for the line that press made, and stops at the name wanted —
//! or fails after its presses, saying where the keyboard went instead.

use std::time::Duration;

use super::{READ_THE_LOG_EVERY, Walking, press};
use crate::keys::Chord;
use crate::machine::{Machine, MachineError};
use crate::report::Verdict;
use crate::strokes::Pace;
use crate::the_log;

/// How long a press is given to make its line: a key Settings takes is said at
/// once, so this is the walk's reading of the log with room.
const A_PRESS_IS_SAID_WITHIN: Duration = Duration::from_secs(8);

/// The check's name, for the report and for a check never reached.
pub(super) fn the_check_for(name: &str) -> String {
    format!("the keyboard is on {name:?}")
}

impl Walking {
    /// Press `chord` until Settings says its keyboard is on `name`, at most
    /// `at_most` times.
    pub(super) fn press_until_the_keyboard_is_on(
        &mut self,
        machine: &mut impl Machine,
        chord: &Chord,
        name: &str,
        at_most: usize,
        pace: Pace,
    ) -> Result<(), MachineError> {
        let wanted = alo_acting::said_in_settings::the_keyboard_is_on(name);
        let phrases = [alo_acting::said_in_settings::IN_SETTINGS_THE_KEYBOARD_IS_ON.to_owned()];
        let mut seen = the_log::all_with(&machine.log()?, self.mark, &phrases).len();
        let mut went = Vec::new();
        for presses in 1..=at_most {
            press(machine, chord, pace)?;
            self.did(machine.elapsed(), format!("pressed {}", chord.said()));
            let start = machine.elapsed();
            loop {
                let lines = the_log::all_with(&machine.log()?, self.mark, &phrases);
                if lines.len() > seen {
                    seen = lines.len();
                    let newest = lines.last().cloned().unwrap_or_default();
                    if newest.contains(&wanted) {
                        self.checked(
                            the_check_for(name),
                            Verdict::Passed,
                            format!("after {presses} presses: {newest}"),
                        );
                        return Ok(());
                    }
                    went.push(newest);
                    break;
                }
                if machine.elapsed().saturating_sub(start) >= A_PRESS_IS_SAID_WITHIN {
                    went.push(format!(
                        "press {presses} said nothing within {}s",
                        A_PRESS_IS_SAID_WITHIN.as_secs()
                    ));
                    break;
                }
                machine.wait(READ_THE_LOG_EVERY);
            }
        }
        self.checked(
            the_check_for(name),
            Verdict::Failed,
            format!(
                "after {at_most} presses of {} the keyboard was never on {name:?}; it went to: {}",
                chord.said(),
                went.join(" | ")
            ),
        );
        self.stop_after_failure();
        Ok(())
    }
}
