//! The switch on the Terminal page that lets alo use a terminal: open one,
//! read what it shows and press its buttons — never type into it.
//!
//! [ADR 0111](../../../docs/decisions/0111-the-person-chooses-whether-their-agent-may-use-the-terminal.md):
//! *off until the person turns it on, the sentence that grants it says what it
//! costs, it is revocable in one action.* The owner, 2026-11-17, of alo
//! reading a terminal's text: *"Both yes, within grants."*
//!
//! # Kept beside the level, read at every question
//!
//! The answer is `alo-uses-a-terminal` in `/var/lib/alo/how-code-runs.toml`
//! (`docs/contracts/how-code-runs-file.md`), under that file's rules about who
//! may have written it, so an agent cannot let itself in. `alo-agentd` reads it
//! at every read, proposal and approval; the sheet alo asks for an application
//! on reads it at the press. **One press turns it either way**, and the switch
//! moves only once the file says so.
//!
//! # Off, and a file that would not read is off
//!
//! Off is what a person who never touched it has, and what `alo-agentd` does
//! over a file it cannot read — so off is the true thing to draw. Pressing the
//! switch over such a file is refused rather than written, because writing it
//! would throw away a level the person may have chosen.

use std::path::{Path, PathBuf};

use alo_capability::ATerminalForTheAgent;

use crate::settings_kept::SettingsKept;

/// The switch, as Settings found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AloUsesATerminalHere {
    /// What the file says, not at all where it says nothing or would not read.
    is: ATerminalForTheAgent,
    /// Where it is kept.
    at: PathBuf,
}

impl AloUsesATerminalHere {
    /// What the switch is at sign-in.
    pub(crate) fn at_sign_in(at: &Path) -> Self {
        Self {
            is: alo_remembering::alo_uses_a_terminal_remembered(at).unwrap_or_default(),
            at: at.to_owned(),
        }
    }

    /// What the switch says.
    pub(crate) const fn is(&self) -> ATerminalForTheAgent {
        self.is
    }

    /// One press: the other answer, written first and drawn second.
    pub(crate) fn pressed(&mut self) -> SettingsKept {
        let next = match self.is {
            ATerminalForTheAgent::NotAtAll => ATerminalForTheAgent::MayOpenOne,
            ATerminalForTheAgent::MayOpenOne => ATerminalForTheAgent::NotAtAll,
        };
        match alo_remembering::alo_uses_a_terminal_kept(&self.at, next) {
            Ok(()) => {
                self.is = next;
                SettingsKept::Written
            }
            Err(_) => SettingsKept::NotWritten,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;

    use super::*;
    use alo_capability::HowCodeRuns;

    /// A private directory, and the file path inside it.
    fn somewhere(what: &str) -> (tempfile::TempDir, PathBuf) {
        let at = tempfile::tempdir().expect("a temporary directory");
        let file = at.path().join(format!("how-code-runs-{what}.toml"));
        (at, file)
    }

    /// **Off until the person switches it on.**
    #[test]
    fn a_machine_nobody_has_touched_has_the_switch_off() {
        let (_at, file) = somewhere("untouched");
        assert_eq!(
            AloUsesATerminalHere::at_sign_in(&file).is(),
            ATerminalForTheAgent::NotAtAll
        );
    }

    /// **On in one press, off again in one press**, each read back by a second
    /// reader so this cannot pass on the value in hand — and the level for
    /// running code is kept through both.
    #[test]
    fn one_press_on_and_one_press_off_keeping_the_level() {
        let (_at, file) = somewhere("pressed");
        alo_remembering::the_level_kept(&file, HowCodeRuns::AskingEachTime)
            .expect("a level is kept");
        let mut here = AloUsesATerminalHere::at_sign_in(&file);
        assert_eq!(here.pressed(), SettingsKept::Written);
        assert_eq!(
            AloUsesATerminalHere::at_sign_in(&file).is(),
            ATerminalForTheAgent::MayOpenOne
        );
        assert_eq!(here.pressed(), SettingsKept::Written);
        assert_eq!(
            AloUsesATerminalHere::at_sign_in(&file).is(),
            ATerminalForTheAgent::NotAtAll,
            "one press from on did not turn it off"
        );
        assert_eq!(
            alo_remembering::the_level_remembered(&file).expect("it reads"),
            Some(HowCodeRuns::AskingEachTime),
            "the switch forgot the level for running code"
        );
    }

    /// **A file that would not read is drawn off, and the switch is not
    /// written over it.**
    #[test]
    fn over_a_file_that_would_not_read_the_switch_is_off_and_not_written() {
        let (_at, file) = somewhere("garbage");
        std::fs::write(&file, "not the file [").expect("written");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).expect("mode");
        let mut here = AloUsesATerminalHere::at_sign_in(&file);
        assert_eq!(here.is(), ATerminalForTheAgent::NotAtAll);
        assert_eq!(here.pressed(), SettingsKept::NotWritten);
        assert_eq!(
            here.is(),
            ATerminalForTheAgent::NotAtAll,
            "the switch moved"
        );
        assert_eq!(
            std::fs::read_to_string(&file).expect("it reads"),
            "not the file ["
        );
    }
}
