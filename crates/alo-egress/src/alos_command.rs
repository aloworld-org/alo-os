//! A command alo is running, and the line a person reads while it sends
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
//! as amended 2026-11-13).
//!
//! The fifth kind of line on the one indicator. Where no verb fits, alo runs a
//! command (ADR 0149), and **what the command sends leaves from the command's
//! own process**, not from `alo-agentd`, so [`Leaving`](crate::Leaving) cannot
//! show it. The owner, 2026-11-13: *"Yes, count alo's commands."* So the
//! command runs in a control group of its own, the kernel counts what that
//! group sends while it runs, and this is the line:
//!
//! - **Counted**: up from the first byte counted until the command ends, and
//!   never at all for a command that sent nothing ([`Counting::Counted`]);
//! - **Uncounted**: on a machine where the count could not be begun, up for the
//!   whole time the command runs, saying so ([`Counting::Uncounted`]) — a line
//!   that stayed dark there would read as *nothing left*, which nobody knows.
//!
//! **It has no destination**, for [`crate::DuringTheirCommand`]'s reason: the
//! counter reads how much, never where. **And no agent's name**: the command is
//! alo's, run because the person chose a level that runs one.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words;

/// The most characters of a program's name a line carries.
///
/// The same bound an application's name is held to, so no line on the
/// indicator is longer than another kind would be.
const LONGEST: usize = 128;

/// What is named when a command's first word cannot be shown at all.
const THE_SHELL: &str = "sh";

/// Which of the two lines it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Counting {
    /// Counted by the kernel, and seen sending.
    Counted,
    /// On a machine where the count could not be begun.
    Uncounted,
}

/// A command alo is running.
///
/// Serialises so the indicator's contents can be told to another process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AlosCommand {
    /// The command's first word, as the person would recognise it.
    program: String,
    /// Which line.
    how: Counting,
}

impl AlosCommand {
    /// A command alo is running whose first word is `program`, shown this way.
    ///
    /// **It cannot be refused**, unlike an application's line: a command that
    /// runs must be shown, so a name that cannot be shown is made showable —
    /// a character that cannot be shown becomes a space, a name too long is cut
    /// at `LONGEST` characters, and a name with nothing left is `sh`, which is
    /// what runs every command.
    #[must_use]
    pub fn of(program: &str, how: Counting) -> Self {
        let showable: String = program
            .chars()
            .map(|one| if one.is_control() { ' ' } else { one })
            .take(LONGEST)
            .collect();
        let program = match showable.trim() {
            "" => THE_SHELL.to_owned(),
            named => named.to_owned(),
        };
        Self { program, how }
    }

    /// The command's first word, as the line names it.
    #[must_use]
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Which line.
    #[must_use]
    pub const fn how(&self) -> Counting {
        self.how
    }

    /// The line a person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        said_of_alos_command(&self.program, self.how, strings)
    }
}

/// **The one sentence for each of the two**, for this machine's line and for a
/// [`crate::Reported`] one alike.
pub(crate) fn said_of_alos_command(program: &str, how: Counting, strings: &Strings) -> Said {
    let word = match how {
        Counting::Counted => words::ALOS_COMMAND_IS_SENDING,
        Counting::Uncounted => words::ALOS_COMMAND_CANNOT_BE_COUNTED,
    };
    strings.say(&word.key(), &Filling::of("program", program))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **Each line names the program and says what is counted.**
    #[test]
    fn each_line_names_the_program_and_says_what_is_counted() {
        let strings = in_english();
        for (how, said) in [
            (
                Counting::Counted,
                "curl, a command alo is running, is sending",
            ),
            (
                Counting::Uncounted,
                "alo is running curl, and this machine cannot count what it sends",
            ),
        ] {
            let line = AlosCommand::of("curl", how);
            assert_eq!(line.said(&strings).text(), said);
            assert_eq!(line.how(), how);
            assert_eq!(line.program(), "curl");
        }
    }

    /// **A name that cannot be shown is made showable rather than refused**,
    /// because a command that runs must be on the indicator whatever it is
    /// called.
    #[test]
    fn a_name_that_cannot_be_shown_is_made_showable_and_never_refused() {
        assert_eq!(
            AlosCommand::of("cu\u{7}rl", Counting::Counted).program(),
            "cu rl"
        );
        assert_eq!(
            AlosCommand::of(" \u{1b} ", Counting::Counted).program(),
            "sh"
        );
        let long = "x".repeat(LONGEST * 2);
        assert_eq!(
            AlosCommand::of(&long, Counting::Uncounted)
                .program()
                .chars()
                .count(),
            LONGEST
        );
    }
}
