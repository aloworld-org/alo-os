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
//!
//! # And whether alo in the terminal acts without the OK
//!
//! *Added 2026-11-06.* The same file keeps a second answer — `in-the-terminal`,
//! `crate::the_level` has its shape — and so the same trust. **This is the
//! reason an agent cannot switch it on for itself**: the agent is its own login
//! (ADR 0001 §5), the folder is the person's alone, and a file anybody else
//! could have written is refused here rather than believed, so the one reader
//! of this answer — `alo-ask`, run by the person — is told nothing it could
//! act on.
//!
//! **Each writer keeps the other's answer.** Choosing a level does not switch
//! the terminal off, and switching the terminal does not unchoose a level. The
//! one place they differ is a file that would not read: a level chosen over it
//! is written as before, and loses a terminal answer nobody could read anyway —
//! the switch then reads off, which is the direction that takes nothing from
//! the person. The terminal's switch is **refused** over such a file instead,
//! because writing it would throw away a level the person may well have chosen.

use std::path::Path;

use alo_capability::{AloInApplications, AloInTheTerminal, HowCodeRuns};

use crate::believing::{read_believed, replaced_whole};
use crate::refusing::NotRemembered;
use crate::the_level::WhatTheFileSays;

/// Where a machine keeps the level its person chose for running code.
pub const THE_LEVEL: &str = "/var/lib/alo/how-code-runs.toml";

/// Everything kept at this path, believed and read; `Ok(None)` for no file.
fn what_is_kept(at: &Path) -> Result<Option<WhatTheFileSays>, NotRemembered> {
    match read_believed(at) {
        Ok(text) => match crate::the_level::read(&text) {
            Ok(says) => Ok(Some(says)),
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

/// What the file is to say, written whole to this path.
fn kept(at: &Path, says: WhatTheFileSays) -> Result<(), NotRemembered> {
    let text = crate::the_level::written(says).map_err(|why| NotRemembered::NotTheLevel {
        at: at.to_owned(),
        why,
    })?;
    replaced_whole(at, &text)
}

/// The level kept at this path, believed and read.
///
/// `Ok(None)` is **a person who has not chosen**, which is not a failure and
/// not the default; see this module's table. A file holding only the
/// terminal's answer is that too.
///
/// # Errors
///
/// [`NotRemembered::ALink`] for a symbolic link;
/// [`NotRemembered::SomebodyElses`] and [`NotRemembered::WritableByOthers`] for
/// a file somebody else could have written — which here would be an escalation
/// to full trust; [`NotRemembered::NotRead`] for a file that would not read;
/// and [`NotRemembered::NotTheLevel`] for everything `the_level`'s own reader
/// refuses about the text, including a file that says nothing at all.
pub fn the_level_remembered(at: &Path) -> Result<Option<HowCodeRuns>, NotRemembered> {
    Ok(what_is_kept(at)?.and_then(|says| says.chose))
}

/// This level, written whole to this path, keeping the terminal's answer and
/// where alo works in applications.
///
/// # Errors
///
/// [`NotRemembered::NotWritten`] naming what the machine said — including a
/// folder that is not there, which is refused rather than made — and
/// [`NotRemembered::NotTheLevel`] for a level that cannot be written down.
pub fn the_level_kept(at: &Path, chose: HowCodeRuns) -> Result<(), NotRemembered> {
    // A file that would not read is written over, as it always was; see this
    // module's header for why that loses nothing the person could be shown.
    let before = what_is_kept(at).ok().flatten().unwrap_or_default();
    kept(
        at,
        WhatTheFileSays {
            chose: Some(chose),
            ..before
        },
    )
}

/// Whether alo in the terminal acts without the OK, as kept at this path.
///
/// **No file, and a file that says nothing about the terminal, are the switch
/// as a machine arrives: off.** Unlike the level, there is no *has not chosen*
/// to keep apart from it — the switch is offered off, and off is what a person
/// who never touched it has.
///
/// # Errors
///
/// Every refusal [`the_level_remembered`] makes, for the same file. **A reader
/// that meets one must ask first**: that is the only answer that cannot be a
/// file somebody else wrote speaking for the person.
pub fn alo_in_the_terminal_remembered(at: &Path) -> Result<AloInTheTerminal, NotRemembered> {
    Ok(what_is_kept(at)?
        .and_then(|says| says.in_the_terminal)
        .unwrap_or_default())
}

/// Whether alo in the terminal acts without the OK, written whole to this path,
/// keeping the level and where alo works in applications.
///
/// # Errors
///
/// Every refusal [`the_level_remembered`] makes, when the file is there and
/// would not read — **written over, it would lose a level the person chose** —
/// and [`NotRemembered::NotWritten`] for a file that could not be written.
pub fn alo_in_the_terminal_kept(at: &Path, answer: AloInTheTerminal) -> Result<(), NotRemembered> {
    let before = what_is_kept(at)?.unwrap_or_default();
    kept(
        at,
        WhatTheFileSays {
            in_the_terminal: Some(answer),
            ..before
        },
    )
}

/// Where alo works in an application, as kept at this path (ADR 0143 Q6).
///
/// **No file, and a file that says nothing about applications, are alo's own
/// copy** — how a machine arrives, and the answer that watches nothing of the
/// person's.
///
/// # Errors
///
/// Every refusal [`the_level_remembered`] makes, for the same file. **A reader
/// that meets one must answer alo's own copy**: that is the only answer that
/// cannot be a file somebody else wrote speaking for the person.
pub fn alo_in_applications_remembered(at: &Path) -> Result<AloInApplications, NotRemembered> {
    Ok(what_is_kept(at)?
        .and_then(|says| says.in_applications)
        .unwrap_or_default())
}

/// Where alo works in an application, written whole to this path, keeping the
/// level and the terminal's answer.
///
/// # Errors
///
/// Every refusal [`the_level_remembered`] makes, when the file is there and
/// would not read — **written over, it would lose a level the person chose** —
/// and [`NotRemembered::NotWritten`] for a file that could not be written.
pub fn alo_in_applications_kept(at: &Path, answer: AloInApplications) -> Result<(), NotRemembered> {
    let before = what_is_kept(at)?.unwrap_or_default();
    kept(
        at,
        WhatTheFileSays {
            in_applications: Some(answer),
            ..before
        },
    )
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

    /// **Off until the person switches it on.** No file at all is the switch
    /// as a machine arrives, and so is a file that says only a level.
    #[test]
    fn alo_in_the_terminal_is_off_until_switched_on() {
        let folder = a_folder_of_our_own("terminal-off");
        let at = folder.join("how-code-runs.toml");
        assert_eq!(
            alo_in_the_terminal_remembered(&at).unwrap(),
            AloInTheTerminal::AsksFirst,
            "a machine with nothing written down acts without asking"
        );
        the_level_kept(&at, HowCodeRuns::AtFullTrust).unwrap();
        assert_eq!(
            alo_in_the_terminal_remembered(&at).unwrap(),
            AloInTheTerminal::AsksFirst,
            "full trust for running code switched the terminal on, which nobody chose"
        );
    }

    /// **Each writer keeps the other's answer**, and a file holding only the
    /// terminal's answer is a person who has not chosen a level.
    #[test]
    fn the_terminal_and_the_level_are_two_answers_in_one_file() {
        let folder = a_folder_of_our_own("terminal-and-level");
        let at = folder.join("how-code-runs.toml");
        alo_in_the_terminal_kept(&at, AloInTheTerminal::ActsWithoutAsking).unwrap();
        assert_eq!(
            the_level_remembered(&at).unwrap(),
            None,
            "switching the terminal on chose a level for running code"
        );
        the_level_kept(&at, HowCodeRuns::AskingEachTime).unwrap();
        assert_eq!(
            alo_in_the_terminal_remembered(&at).unwrap(),
            AloInTheTerminal::ActsWithoutAsking,
            "choosing a level switched the terminal off"
        );
        alo_in_the_terminal_kept(&at, AloInTheTerminal::AsksFirst).unwrap();
        assert_eq!(
            the_level_remembered(&at).unwrap(),
            Some(HowCodeRuns::AskingEachTime),
            "switching the terminal off forgot the level"
        );
        assert_eq!(
            alo_in_the_terminal_remembered(&at).unwrap(),
            AloInTheTerminal::AsksFirst
        );
    }

    /// **An agent cannot switch it on for itself by writing the file.** The
    /// agent is another login, so a file it could have written is one others
    /// can write — and that is refused, never read as *act without asking*.
    #[test]
    fn a_file_others_could_write_saying_act_without_asking_is_refused() {
        let folder = a_folder_of_our_own("terminal-untrusted");
        let at = folder.join("how-code-runs.toml");
        alo_in_the_terminal_kept(&at, AloInTheTerminal::ActsWithoutAsking).unwrap();
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o646)).unwrap();
        assert!(matches!(
            alo_in_the_terminal_remembered(&at),
            Err(NotRemembered::WritableByOthers { mode: 0o646, .. })
        ));
    }

    /// **The switch is not written over a file that would not read**, because
    /// that would throw away a level the person may have chosen — and the file
    /// is left exactly as it was.
    #[test]
    fn the_switch_is_refused_over_a_file_that_would_not_read() {
        let folder = a_folder_of_our_own("terminal-over-garbage");
        let at = folder.join("how-code-runs.toml");
        std::fs::write(&at, "this is not the file [").unwrap();
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(OURS_ALONE)).unwrap();
        assert!(matches!(
            alo_in_the_terminal_kept(&at, AloInTheTerminal::ActsWithoutAsking),
            Err(NotRemembered::NotTheLevel { .. })
        ));
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "this is not the file [",
            "the file that would not read was written over"
        );
        assert!(
            alo_in_the_terminal_remembered(&at).is_err(),
            "a file that would not read was read as an answer"
        );
    }

    /// **alo works in its own copy until the person chooses otherwise**: no
    /// file, and a file that says only a level, both answer it.
    #[test]
    fn alo_in_applications_is_its_own_copy_until_chosen() {
        let folder = a_folder_of_our_own("applications-default");
        let at = folder.join("how-code-runs.toml");
        assert_eq!(
            alo_in_applications_remembered(&at).unwrap(),
            AloInApplications::InItsOwnCopy
        );
        the_level_kept(&at, HowCodeRuns::AtFullTrust).unwrap();
        assert_eq!(
            alo_in_applications_remembered(&at).unwrap(),
            AloInApplications::InItsOwnCopy
        );
    }

    /// **Each of the three answers is kept when another is written** — the
    /// level, the terminal and applications — each read back from the disk.
    #[test]
    fn every_answer_survives_the_others_being_written() {
        let folder = a_folder_of_our_own("applications-kept");
        let at = folder.join("how-code-runs.toml");
        alo_in_applications_kept(&at, AloInApplications::InTheirWindow).unwrap();
        assert_eq!(
            alo_in_applications_remembered(&at).unwrap(),
            AloInApplications::InTheirWindow
        );
        assert_eq!(the_level_remembered(&at).unwrap(), None, "a level was invented");
        the_level_kept(&at, HowCodeRuns::AskingEachTime).unwrap();
        alo_in_the_terminal_kept(&at, AloInTheTerminal::ActsWithoutAsking).unwrap();
        assert_eq!(
            alo_in_applications_remembered(&at).unwrap(),
            AloInApplications::InTheirWindow,
            "writing the level or the terminal forgot where alo works"
        );
        alo_in_applications_kept(&at, AloInApplications::InItsOwnCopy).unwrap();
        assert_eq!(
            the_level_remembered(&at).unwrap(),
            Some(HowCodeRuns::AskingEachTime)
        );
        assert_eq!(
            alo_in_the_terminal_remembered(&at).unwrap(),
            AloInTheTerminal::ActsWithoutAsking
        );
        assert_eq!(
            alo_in_applications_remembered(&at).unwrap(),
            AloInApplications::InItsOwnCopy
        );
    }

    /// **A file another login could write is refused, even saying the
    /// person's own window** — so an agent cannot turn this up for itself.
    #[test]
    fn a_file_others_could_write_is_refused_for_applications() {
        let folder = a_folder_of_our_own("applications-untrusted");
        let at = folder.join("how-code-runs.toml");
        alo_in_applications_kept(&at, AloInApplications::InTheirWindow).unwrap();
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o646)).unwrap();
        assert!(matches!(
            alo_in_applications_remembered(&at),
            Err(NotRemembered::WritableByOthers { mode: 0o646, .. })
        ));
    }
}
