//! What the section answers for each of the three states the file keeps apart.
//!
//! The case that matters most is the one in the middle: **a person who has not
//! chosen is not a person at the sealed box.** Every test here was shown to
//! fail against a version that flattened the two.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{HowCodeRunsHere, TheLevelHere};
use crate::settings_kept::SettingsKept;
use alo_capability::HowCodeRuns;

/// A private directory, and the file path inside it.
fn somewhere(what: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let at = tempfile::tempdir().expect("a temporary directory");
    let file = at.path().join(format!("how-code-runs-{what}.toml"));
    (at, file)
}

/// **A machine nobody has opened this on has not chosen the sealed box.**
///
/// The whole reason this section does not use `Keeper`. Nothing is ticked, and
/// nothing is said above the rows, because there is nothing wrong — a person
/// who has not answered a question is not an error.
#[test]
fn no_file_is_a_person_who_has_not_chosen_and_not_a_person_at_the_sealed_box() {
    let (_at, file) = somewhere("absent");
    let here = HowCodeRunsHere::at_sign_in(&file);

    assert_eq!(here.chosen(), None, "nothing is ticked");
    assert_ne!(
        here.chosen(),
        Some(HowCodeRuns::InASealedBox),
        "a person who has not chosen was reported as having chosen the default"
    );
    assert!(
        !matches!(here.found, TheLevelHere::WouldNotRead),
        "a file that is not there was taken for one that would not read"
    );
}

/// **A level a person picked is the level that is ticked**, and it survives
/// being read back from the file rather than from the value in hand.
#[test]
fn a_level_a_person_picked_is_kept_and_read_back() {
    let (_at, file) = somewhere("picked");
    let mut here = HowCodeRunsHere::at_sign_in(&file);
    assert_eq!(
        here.picked(HowCodeRuns::AtFullTrust),
        SettingsKept::Written,
        "a level is kept"
    );
    assert_eq!(here.chosen(), Some(HowCodeRuns::AtFullTrust));

    // Read back from the file by a second reader, so this cannot pass on the
    // value the test handed in.
    let again = HowCodeRunsHere::at_sign_in(&file);
    assert_eq!(again.chosen(), Some(HowCodeRuns::AtFullTrust));
    assert!(!matches!(again.found, TheLevelHere::WouldNotRead));
}

/// **Every one of the three can be picked**, so this cannot pass by handling
/// one and refusing the rest.
#[test]
fn each_of_the_three_levels_can_be_picked() {
    for level in HowCodeRuns::ALL {
        let (_at, file) = somewhere("each");
        let mut here = HowCodeRunsHere::at_sign_in(&file);
        assert_eq!(
            here.picked(level),
            SettingsKept::Written,
            "each level is one a person may pick"
        );
        assert_eq!(
            HowCodeRunsHere::at_sign_in(&file).chosen(),
            Some(level),
            "{level:?} did not come back"
        );
    }
}

/// **A file that will not read is said, and nothing is ticked** — and it is
/// not written over, because the only thing to write would be a level nobody
/// picked.
#[test]
fn a_file_that_will_not_read_is_said_and_not_replaced() {
    let (_at, file) = somewhere("broken");
    std::fs::write(&file, "this is not what that file holds\n").expect("a file is written");
    let before = std::fs::read(&file).expect("the file reads as bytes");

    let here = HowCodeRunsHere::at_sign_in(&file);
    assert!(
        matches!(here.found, TheLevelHere::WouldNotRead),
        "a file that will not read said nothing"
    );
    assert_eq!(here.chosen(), None, "nothing is ticked");
    assert!(
        matches!(here.found, TheLevelHere::WouldNotRead),
        "a file that would not read was taken for one of the other two states"
    );
    assert_eq!(
        std::fs::read(&file).expect("the file is still there"),
        before,
        "the file that did not read was written over"
    );
}

/// **A change that was not written does not move the tick.**
///
/// The refusal is produced rather than simulated: the path is inside a
/// directory that does not exist, so the crate that owns the file refuses it.
///
/// This is the half of *written first, drawn second* that can be tested without
/// a screen. A section that moved its tick and then tried to write would pass
/// every other test here and tell a person their machine runs code at a level
/// it does not — which on this setting is the difference between a sealed box
/// and full trust.
#[test]
fn a_change_that_was_not_written_does_not_move_the_tick() {
    let (at, _file) = somewhere("refused");
    let nowhere = at.path().join("not-a-directory").join("how-code-runs.toml");

    let mut here = HowCodeRunsHere::at_sign_in(&nowhere);
    assert_eq!(
        here.chosen(),
        None,
        "a file that is not there is a person who has not chosen"
    );
    assert!(
        !matches!(here.found, TheLevelHere::WouldNotRead),
        "a file that is not there was taken for one that would not read"
    );

    assert_eq!(
        here.picked(HowCodeRuns::AtFullTrust),
        SettingsKept::NotWritten,
        "a write into a directory that does not exist was reported as done"
    );
    assert_eq!(
        here.chosen(),
        None,
        "a refused change moved the tick to full trust anyway"
    );
}
