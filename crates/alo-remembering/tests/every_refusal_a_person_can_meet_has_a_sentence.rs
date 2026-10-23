//! **Every refusal the level road can produce has a sentence, and the ones it
//! cannot do not.**
//!
//! `crate::words` argues the split: the grants, the pairings and the machine
//! names are read by `alo-agentd` at start into a service log, where English is
//! correct; the level a person chose for running code is read by a **surface**,
//! and a surface that cannot say why shows `{why:?}` or nothing.
//!
//! That argument is only as good as the list of which arms each road produces,
//! and **that list is the thing that will go stale**. An arm that gains a road
//! gains a reader, and nothing about the code says so. So this walks the level
//! road for real — a file that is a link, a file somebody else could write, a
//! file saying no level, a file saying a word that is not a level — and asserts
//! each one answers with a sentence.
//!
//! It cannot walk the arms it says are unworded, because the level road cannot
//! produce them; those are asserted to answer `None` by construction instead,
//! which is the honest limit of this file and is stated rather than hidden.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_remembering::{NotRemembered, declare_into, the_level_kept, the_level_remembered};
use alo_strings::{Strings, Vocabulary};
use std::os::unix::fs::PermissionsExt as _;

/// A `Strings` holding this crate's words and nothing else.
fn words() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).expect("this crate's list declares");
    Strings::of(vocabulary)
}

/// A folder of this test's own, removed by the operating system afterwards.
fn a_folder(called: &str) -> std::path::PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-remembering-words-{}-{called}",
        std::process::id()
    ));
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// Every refusal reachable by reading the level file says something, and what
/// it says names the file and reads as a sentence rather than as a key.
#[test]
fn every_refusal_the_level_road_can_produce_says_something() {
    let strings = words();
    let folder = a_folder("reachable");

    // A link, which is refused however good the file it points at.
    let real = folder.join("how-code-runs.toml");
    the_level_kept(&real, alo_capability::HowCodeRuns::AskingEachTime).unwrap();
    let link = folder.join("linked.toml");
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&real, &link).unwrap();

    // A file others could write, which on this file is an escalation.
    let loose = folder.join("loose.toml");
    the_level_kept(&loose, alo_capability::HowCodeRuns::AtFullTrust).unwrap();
    std::fs::set_permissions(&loose, std::fs::Permissions::from_mode(0o646)).unwrap();

    // A file that is there and says no level.
    let silent = folder.join("silent.toml");
    std::fs::write(&silent, "format = 1\n").unwrap();
    std::fs::set_permissions(&silent, std::fs::Permissions::from_mode(0o600)).unwrap();

    // A word that is not a level.
    let nearly = folder.join("nearly.toml");
    std::fs::write(&nearly, "format = 1\nchose = \"full-trust\"\n").unwrap();
    std::fs::set_permissions(&nearly, std::fs::Permissions::from_mode(0o600)).unwrap();

    // Not this file at all.
    let garbage = folder.join("garbage.toml");
    std::fs::write(&garbage, "this is not toml [\n").unwrap();
    std::fs::set_permissions(&garbage, std::fs::Permissions::from_mode(0o600)).unwrap();

    // From a newer alo OS.
    let later = folder.join("later.toml");
    std::fs::write(&later, "format = 99\nchose = \"invented-later\"\n").unwrap();
    std::fs::set_permissions(&later, std::fs::Permissions::from_mode(0o600)).unwrap();

    // A folder that is not there, which is the write refusal.
    let nowhere = folder.join("not-made").join("how-code-runs.toml");

    let mut seen = 0;
    for (what, why) in [
        (
            "a link",
            the_level_remembered(&link).expect_err("a link is refused"),
        ),
        (
            "others could write it",
            the_level_remembered(&loose).expect_err("a loose file is refused"),
        ),
        (
            "it says no level",
            the_level_remembered(&silent).expect_err("a silent file is refused"),
        ),
        (
            "not a level",
            the_level_remembered(&nearly).expect_err("a wrong word is refused"),
        ),
        (
            "not the shape",
            the_level_remembered(&garbage).expect_err("garbage is refused"),
        ),
        (
            "a newer alo OS",
            the_level_remembered(&later).expect_err("a newer format is refused"),
        ),
        (
            "could not be written",
            the_level_kept(&nowhere, alo_capability::HowCodeRuns::InASealedBox)
                .expect_err("a missing folder is refused"),
        ),
    ] {
        let said = why
            .said(&strings)
            .unwrap_or_else(|| panic!("`{what}` is reachable from a surface and says nothing"));
        let text = said.text();
        assert!(
            !text.contains("remembering."),
            "`{what}` read as {text:?}, which is a key and not a sentence — the word is not \
             declared"
        );
        assert!(
            !text.contains('{'),
            "`{what}` read as {text:?}, which has a hole nothing filled"
        );
        seen += 1;
    }
    assert_eq!(seen, 7, "the walk visited {seen} refusals rather than 7");
}

/// **A person who has not chosen is told nothing**, because there is nothing to
/// tell them: no file is `Ok(None)` and not a refusal at all.
///
/// The one arm of the believing refusals deliberately left unworded, and the
/// reason is that a sentence for it would be a machine telling somebody off for
/// never having opened a screen.
#[test]
fn a_person_who_has_not_chosen_is_not_told_off() {
    let folder = a_folder("unchosen");
    let nothing = the_level_remembered(&folder.join("how-code-runs.toml")).unwrap();
    assert_eq!(nothing, None);

    // And if it ever becomes a refusal rather than an absence, it must still
    // not be worded as one without this test being changed deliberately.
    let strings = words();
    let not_there = NotRemembered::NotThere {
        at: folder.join("how-code-runs.toml"),
    };
    assert!(
        not_there.said(&strings).is_none(),
        "`nothing has been chosen` was given a sentence, which makes it read as a failure"
    );
}

/// **The grants road's arms say nothing, and that is the argument, not an
/// oversight.**
///
/// Asserted by construction because the level road cannot produce them: these
/// come from `crate::read` over a hand-edited grants file, which `alo-agentd`
/// reads at start into a log. A sentence for one would be a string a translator
/// was handed for nothing.
#[test]
fn the_arms_only_a_service_log_reads_are_not_worded() {
    let strings = words();
    for (what, why) in [
        (
            "not the shape",
            NotRemembered::NotTheShape {
                why: "a parser's words".to_owned(),
            },
        ),
        (
            "another format",
            NotRemembered::AnotherFormat { format: 99 },
        ),
        (
            "granted to nobody",
            NotRemembered::GrantedToNobody { handle: 1 },
        ),
        ("granted to two", NotRemembered::GrantedToTwo { handle: 1 }),
        (
            "reaches nothing",
            NotRemembered::ReachesNothing { handle: 1 },
        ),
        (
            "reaches two things",
            NotRemembered::ReachesTwoThings { handle: 1 },
        ),
        ("not a moment", NotRemembered::NotAMoment { handle: 1 }),
    ] {
        assert!(
            why.said(&strings).is_none(),
            "`{what}` was given a sentence; it is read by a service log, where English is correct"
        );
    }
}
