//! The switch on the Terminal page that lets what a person asks alo for in the
//! terminal run without the OK.
//!
//! `docs/features.md`, the owner's decision of 2026-11-06: *ADR 0064's third
//! level, full trust, applied to the terminal. **Off until the person switches
//! it on**, offered with what it costs in plain words — alo will send, change
//! and delete what you ask for without showing you first — and off again in one
//! press. The record, Put back and the egress indicator stay on at every
//! setting.*
//!
//! # Kept beside the level, read at every press
//!
//! The answer is `in-the-terminal` in `/var/lib/alo/how-code-runs.toml`, the
//! file the Running code page keeps its level in
//! (`docs/contracts/how-code-runs-file.md`); `alo_remembering` writes it whole
//! and keeps the level as it was. **One press turns it either way**, and the
//! switch moves only once the file says so: a switch that moved first would
//! tell somebody alo asks first when it does not, which on this setting is the
//! whole difference.
//!
//! # Off, not *not set*
//!
//! The Running code page keeps *has not chosen* apart from the sealed box, and
//! this does not keep a third state, deliberately: the promise is a switch that
//! is **off until the person switches it on**, so a person who never touched it
//! has it off, and *Off* is the true thing to draw. A file that would not read
//! is drawn off too, because off is what `alo-ask` then does — and pressing the
//! switch over such a file is refused rather than written, because writing it
//! would throw away a level the person may have chosen.

use std::path::{Path, PathBuf};

use alo_capability::AloInTheTerminal;

use crate::settings_kept::SettingsKept;

/// The switch, as Settings found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AloInTheTerminalHere {
    /// What the file says, off where it says nothing or would not read.
    is: AloInTheTerminal,
    /// Where it is kept.
    at: PathBuf,
}

impl AloInTheTerminalHere {
    /// What the switch is at sign-in.
    pub(crate) fn at_sign_in(at: &Path) -> Self {
        Self {
            is: alo_remembering::alo_in_the_terminal_remembered(at).unwrap_or_default(),
            at: at.to_owned(),
        }
    }

    /// Whether the switch is on.
    pub(crate) const fn is(&self) -> AloInTheTerminal {
        self.is
    }

    /// One press: the other answer, written first and drawn second.
    pub(crate) fn pressed(&mut self) -> SettingsKept {
        let next = self.is.the_other_one();
        match alo_remembering::alo_in_the_terminal_kept(&self.at, next) {
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
            AloInTheTerminalHere::at_sign_in(&file).is(),
            AloInTheTerminal::AsksFirst
        );
    }

    /// **On in one press, off again in one press**, each read back by a second
    /// reader so this cannot pass on the value in hand — and the level for
    /// running code is kept as it was through both.
    #[test]
    fn one_press_on_and_one_press_off_keeping_the_level() {
        let (_at, file) = somewhere("pressed");
        alo_remembering::the_level_kept(&file, HowCodeRuns::AskingEachTime)
            .expect("a level is kept");
        let mut here = AloInTheTerminalHere::at_sign_in(&file);
        assert_eq!(here.pressed(), SettingsKept::Written);
        assert_eq!(
            AloInTheTerminalHere::at_sign_in(&file).is(),
            AloInTheTerminal::ActsWithoutAsking
        );
        assert_eq!(here.pressed(), SettingsKept::Written);
        assert_eq!(
            AloInTheTerminalHere::at_sign_in(&file).is(),
            AloInTheTerminal::AsksFirst,
            "one press from on did not turn it off"
        );
        assert_eq!(
            alo_remembering::the_level_remembered(&file).expect("it reads"),
            Some(HowCodeRuns::AskingEachTime),
            "the switch forgot the level for running code"
        );
    }

    /// **A file that would not read is drawn off, and the switch is not
    /// written over it** — the switch stays where it was drawn.
    #[test]
    fn over_a_file_that_would_not_read_the_switch_is_off_and_not_written() {
        let (_at, file) = somewhere("garbage");
        std::fs::write(&file, "not the file [").expect("written");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).expect("mode");
        let mut here = AloInTheTerminalHere::at_sign_in(&file);
        assert_eq!(here.is(), AloInTheTerminal::AsksFirst);
        assert_eq!(here.pressed(), SettingsKept::NotWritten);
        assert_eq!(here.is(), AloInTheTerminal::AsksFirst, "the switch moved");
        assert_eq!(
            std::fs::read_to_string(&file).expect("it reads"),
            "not the file ["
        );
    }
}
