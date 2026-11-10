//! The choice on the Folders alo can use page of **where alo works in an
//! application**: in its own copy, or in the person's own window
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q6, and the owner's answer A of 2026-11-10).
//!
//! # Kept beside the level, read at every ask
//!
//! The answer is `in-applications` in `/var/lib/alo/how-code-runs.toml`
//! (`docs/contracts/how-code-runs-file.md`); `alo_remembering` writes it whole
//! and keeps the level and the terminal's answer as they were. The door that
//! lets alo work in an application reads the same file each time it is asked
//! (`crate::alo_at_work`), so a choice made here is what the next ask meets.
//!
//! **The tick moves only once the file says so**, for the level's reason: a
//! tick that moved first would tell somebody alo works only in its own copy
//! when it may work in their window, which on this setting is the whole
//! difference.
//!
//! # alo's own copy, not *not set*
//!
//! As with the terminal's switch, there is no third state: the promise is that
//! alo works in its own copy **until the person chooses otherwise**, so a person
//! who never touched it has that, and it is the true thing to tick. A file that
//! would not read is drawn as alo's own copy too, because that is what the door
//! then does — and choosing over such a file is refused rather than written,
//! because writing it would throw away a level the person may have chosen.

use std::path::{Path, PathBuf};

use alo_capability::AloInApplications;

use crate::settings_kept::SettingsKept;

/// The choice, as Settings found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AloInApplicationsHere {
    /// What the file says, alo's own copy where it says nothing or would not
    /// read.
    is: AloInApplications,
    /// Where it is kept.
    at: PathBuf,
}

impl AloInApplicationsHere {
    /// What the choice is at sign-in.
    pub(crate) fn at_sign_in(at: &Path) -> Self {
        Self {
            is: alo_remembering::alo_in_applications_remembered(at).unwrap_or_default(),
            at: at.to_owned(),
        }
    }

    /// Which answer is chosen.
    pub(crate) const fn is(&self) -> AloInApplications {
        self.is
    }

    /// One press on one of the two: written first and drawn second.
    pub(crate) fn picked(&mut self, answer: AloInApplications) -> SettingsKept {
        match alo_remembering::alo_in_applications_kept(&self.at, answer) {
            Ok(()) => {
                self.is = answer;
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

    /// **alo's own copy until the person chooses otherwise.**
    #[test]
    fn a_machine_nobody_has_touched_has_alo_in_its_own_copy() {
        let (_at, file) = somewhere("untouched");
        assert_eq!(
            AloInApplicationsHere::at_sign_in(&file).is(),
            AloInApplications::InItsOwnCopy
        );
    }

    /// **Either answer in one press**, each read back by a second reader so
    /// this cannot pass on the value in hand — and the level is kept.
    #[test]
    fn each_answer_is_written_and_the_level_kept() {
        let (_at, file) = somewhere("picked");
        alo_remembering::the_level_kept(&file, HowCodeRuns::AskingEachTime)
            .expect("a level is kept");
        let mut here = AloInApplicationsHere::at_sign_in(&file);
        for answer in [
            AloInApplications::InTheirWindow,
            AloInApplications::InItsOwnCopy,
        ] {
            assert_eq!(here.picked(answer), SettingsKept::Written);
            assert_eq!(AloInApplicationsHere::at_sign_in(&file).is(), answer);
        }
        assert_eq!(
            alo_remembering::the_level_remembered(&file).expect("it reads"),
            Some(HowCodeRuns::AskingEachTime),
            "the choice forgot the level for running code"
        );
    }

    /// **A file that would not read is drawn as alo's own copy, and nothing
    /// is written over it.**
    #[test]
    fn over_a_file_that_would_not_read_nothing_is_written() {
        let (_at, file) = somewhere("garbage");
        std::fs::write(&file, "not the file [").expect("written");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).expect("mode");
        let mut here = AloInApplicationsHere::at_sign_in(&file);
        assert_eq!(here.is(), AloInApplications::InItsOwnCopy);
        assert_eq!(
            here.picked(AloInApplications::InTheirWindow),
            SettingsKept::NotWritten
        );
        assert_eq!(here.is(), AloInApplications::InItsOwnCopy, "the tick moved");
        assert_eq!(
            std::fs::read_to_string(&file).expect("it reads"),
            "not the file ["
        );
    }
}
