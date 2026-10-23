//! The level file on the disk: where it is, and who is believed about it.
//!
//! What the level is and how it is written down are `alo_capability` and
//! `crate::the_level`; this is where that text lives, under the rule every file
//! in this crate is held to (`crate::believing`).
//!
//! # `/var/lib/alo/how-code-runs.toml`, beside the grants
//!
//! The same folder as the grants and the pairings, with the same trust: not a
//! symbolic link, owned by root or by the login reading it, writable by nobody
//! else, written `0600` and replaced whole. **This file is the strongest reason
//! that rule exists in this crate.** `crate::keeping` says it of the grants —
//! *whoever can rewrite either says what this machine's agent may do* — and
//! here it is not a grant's worth of machine but all of it: a file somebody
//! else could write into saying `at-full-trust` is an escalation to the agent
//! running code on this machine without asking, which is the thing law 2 and
//! [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
//! are both about.
//!
//! A file of its own rather than a key in `grants.toml`, for the machine
//! names' reason: a grants row is a grant — who, what, until when — and a level
//! is not one. It is a single answer about the whole machine, with its own
//! format number to move when ADR 0064 gains a level, and `grants.toml` does
//! not notice.
//!
//! # Three states, and the reader can tell them apart
//!
//! [`the_level_remembered`] answers `Ok(Some(level))`, `Ok(None)` or `Err`, and
//! that shape is the point of this file:
//!
//! | | |
//! |---|---|
//! | `Ok(Some(level))` | the person chose this |
//! | `Ok(None)` | **no file: the person has not chosen** |
//! | `Err(..)` | something is wrong, and no level was read |
//!
//! A reader that collapsed the second into
//! [`HowCodeRuns::default()`](alo_capability::HowCodeRuns::default) would be
//! showing a person a default as their own decision, which `docs/features.md`
//! rules out in its own words — *the default is that they have not chosen*. A
//! reader that collapsed the **third** into it would be worse: a file somebody
//! tampered with, or a disk that would not read, answering with a level nobody
//! picked. Keeping all three apart is why this does not return a bare
//! `HowCodeRuns`, and `crate::the_level` carries the same argument about the
//! key inside the file.

use std::path::Path;

use alo_capability::HowCodeRuns;

use crate::believing::{read_believed, replaced_whole};
use crate::refusing::NotRemembered;

/// Where a machine keeps the level its person chose for running code.
pub const THE_LEVEL: &str = "/var/lib/alo/how-code-runs.toml";

/// The level kept at this path, believed and read.
///
/// `Ok(None)` is **a person who has not chosen**, which is not a failure and
/// not the default; see this module's table.
///
/// # Errors
///
/// [`NotRemembered::ALink`] for a symbolic link;
/// [`NotRemembered::SomebodyElses`] and [`NotRemembered::WritableByOthers`] for
/// a file somebody else could have written — which here would be an escalation
/// to full trust; [`NotRemembered::NotRead`] for a file that would not read;
/// and [`NotRemembered::NotTheLevel`] for everything `the_level`'s own reader
/// refuses about the text, including a file that says no level at all.
pub fn the_level_remembered(at: &Path) -> Result<Option<HowCodeRuns>, NotRemembered> {
    match read_believed(at) {
        Ok(text) => match crate::the_level::read(&text) {
            Ok(chose) => Ok(Some(chose)),
            // **The path is attached here and nowhere else.** `the_level` reads
            // text and does not know which file it came from; every sentence
            // for this refusal names the file, so the reader supplies it.
            Err(why) => Err(NotRemembered::NotTheLevel {
                at: at.to_owned(),
                why,
            }),
        },
        // **The one arm that is not a failure.** Nothing has been chosen on
        // this machine, which every other refusal is deliberately told apart
        // from.
        Err(NotRemembered::NotThere { .. }) => Ok(None),
        Err(why) => Err(why),
    }
}

/// This level, written whole to this path.
///
/// # Errors
///
/// [`NotRemembered::NotWritten`] naming what the machine said — including a
/// folder that is not there, which is refused rather than made — and
/// [`NotRemembered::NotTheLevel`] for a level that cannot be written down.
pub fn the_level_kept(at: &Path, chose: HowCodeRuns) -> Result<(), NotRemembered> {
    let text = crate::the_level::written(chose).map_err(|why| NotRemembered::NotTheLevel {
        at: at.to_owned(),
        why,
    })?;
    replaced_whole(at, &text)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    use crate::believing::OURS_ALONE;
    use crate::testing::a_folder_of_our_own;

    /// **What was chosen is found again, owner-only.**
    #[test]
    fn a_level_kept_is_found_with_nobody_else_able_to_write_it() {
        let folder = a_folder_of_our_own("level-round-trip");
        let at = folder.join("how-code-runs.toml");
        the_level_kept(&at, HowCodeRuns::AskingEachTime).unwrap();
        let mode = std::fs::metadata(&at).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, OURS_ALONE, "the level went down mode {mode:o}");
        assert_eq!(
            the_level_remembered(&at).unwrap(),
            Some(HowCodeRuns::AskingEachTime)
        );
    }

    /// Every level survives the disk, not only the one the round trip picked.
    #[test]
    fn every_level_survives_the_disk() {
        let folder = a_folder_of_our_own("level-all-three");
        for chose in HowCodeRuns::ALL {
            let at = folder.join(format!("{}.toml", crate::the_level::the_word_for(chose)));
            the_level_kept(&at, chose).unwrap();
            assert_eq!(the_level_remembered(&at).unwrap(), Some(chose));
        }
    }

    /// **No file is a person who has not chosen**, answered as `Ok(None)` and
    /// never as the default.
    ///
    /// This is the assertion the whole module is shaped around: a machine where
    /// nobody has opened the setting must not report *in a sealed box* as
    /// somebody's choice.
    #[test]
    fn no_file_is_a_person_who_has_not_chosen_rather_than_the_default() {
        let folder = a_folder_of_our_own("level-unchosen");
        let nothing = the_level_remembered(&folder.join("how-code-runs.toml")).unwrap();
        assert_eq!(nothing, None);
        assert_ne!(
            nothing,
            Some(HowCodeRuns::default()),
            "a machine nobody has chosen on reported the default as a choice"
        );
    }

    /// **A file somebody else could have written is refused, not read.**
    ///
    /// The escalation this file exists to prevent: a world-writable file saying
    /// `at-full-trust` must not be believed. Asserted as *which* refusal, and
    /// with the level that file holds named, so the test cannot pass by
    /// refusing for some other reason.
    #[test]
    fn a_file_others_could_write_is_refused_even_saying_full_trust() {
        let folder = a_folder_of_our_own("level-untrusted");
        let at = folder.join("how-code-runs.toml");
        the_level_kept(&at, HowCodeRuns::AtFullTrust).unwrap();
        // It reads while it is ours alone.
        assert_eq!(
            the_level_remembered(&at).unwrap(),
            Some(HowCodeRuns::AtFullTrust)
        );
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o646)).unwrap();
        assert!(matches!(
            the_level_remembered(&at),
            Err(NotRemembered::WritableByOthers { mode: 0o646, .. })
        ));
    }

    /// A symbolic link is refused, whatever it points at.
    #[test]
    fn a_link_is_refused() {
        let folder = a_folder_of_our_own("level-link");
        let real = folder.join("how-code-runs.toml");
        the_level_kept(&real, HowCodeRuns::InASealedBox).unwrap();
        let link = folder.join("linked.toml");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert!(matches!(
            the_level_remembered(&link),
            Err(NotRemembered::ALink { .. })
        ));
    }

    /// **A file that is there and says no level is a refusal, not an absence.**
    ///
    /// Told apart from no file at all, because somebody who deleted the line by
    /// hand is owed the difference.
    #[test]
    fn a_file_saying_no_level_is_refused_rather_than_read_as_unchosen() {
        let folder = a_folder_of_our_own("level-says-nothing");
        let at = folder.join("how-code-runs.toml");
        std::fs::write(
            &at,
            format!("format = {}\n", crate::the_level::THE_LEVEL_FORMAT),
        )
        .unwrap();
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(OURS_ALONE)).unwrap();
        assert!(matches!(
            the_level_remembered(&at),
            Err(NotRemembered::NotTheLevel {
                why: crate::the_level::NotTheLevel::SaysNothing,
                ..
            })
        ));
    }

    /// A folder that is not there is refused rather than made.
    #[test]
    fn a_folder_that_is_not_there_is_refused_rather_than_made() {
        let folder = a_folder_of_our_own("level-no-folder");
        let nowhere = folder.join("not-made").join("how-code-runs.toml");
        assert!(matches!(
            the_level_kept(&nowhere, HowCodeRuns::InASealedBox),
            Err(NotRemembered::NotWritten { .. })
        ));
        assert!(
            !folder.join("not-made").exists(),
            "it made the folder it was supposed to refuse"
        );
    }

    /// The path is one literal, in this crate, and names the folder the image
    /// makes.
    #[test]
    fn the_path_is_beside_the_grants() {
        assert_eq!(
            Path::new(THE_LEVEL).parent().unwrap(),
            Path::new(crate::THE_FOLDER),
            "the level is kept somewhere other than beside the grants"
        );
    }
}
