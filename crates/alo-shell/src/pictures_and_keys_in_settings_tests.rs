#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use super::*;
use alo_capability::HowCodeRuns;

/// A game: an application that draws its own window and describes nothing.
const A_GAME: &str = "net.supertuxkart.SuperTuxKart";

/// A private directory, and the file path inside it.
fn somewhere(what: &str) -> (tempfile::TempDir, PathBuf) {
    let at = tempfile::tempdir().expect("a temporary directory");
    let file = at.path().join(format!("how-code-runs-{what}.toml"));
    (at, file)
}

/// **Off in every application on a machine nobody has touched.**
#[test]
fn a_machine_nobody_has_touched_has_every_switch_off() {
    let (_at, file) = somewhere("untouched");
    let here = PicturesAndKeysHere::at_sign_in(&file);
    assert!(!here.is_on_in(A_GAME));
    assert!(!here.is_on_in("org.mozilla.firefox"));
}

/// **One press turns it on in that application alone and a second turns it
/// off**, each read back by a second reader so this cannot pass on the value
/// in hand — and the level is kept.
#[test]
fn a_press_turns_one_application_on_and_a_second_turns_it_off() {
    let (_at, file) = somewhere("pressed");
    alo_remembering::the_level_kept(&file, HowCodeRuns::AskingEachTime).expect("a level is kept");
    let mut here = PicturesAndKeysHere::at_sign_in(&file);
    assert_eq!(here.pressed(A_GAME), SettingsKept::Written);
    assert!(here.is_on_in(A_GAME));
    let again = PicturesAndKeysHere::at_sign_in(&file);
    assert!(again.is_on_in(A_GAME));
    assert!(!again.is_on_in("org.mozilla.firefox"));
    assert_eq!(here.pressed(A_GAME), SettingsKept::Written);
    assert!(!here.is_on_in(A_GAME));
    assert!(!PicturesAndKeysHere::at_sign_in(&file).is_on_in(A_GAME));
    assert_eq!(
        alo_remembering::the_level_remembered(&file).expect("it reads"),
        Some(HowCodeRuns::AskingEachTime),
        "the switch forgot the level for running code"
    );
}

/// **A file that would not read is drawn off, and nothing is written over
/// it.**
#[test]
fn over_a_file_that_would_not_read_nothing_is_written() {
    let (_at, file) = somewhere("garbage");
    std::fs::write(&file, "not the file [").expect("written");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).expect("mode");
    let mut here = PicturesAndKeysHere::at_sign_in(&file);
    assert!(!here.is_on_in(A_GAME));
    assert_eq!(here.pressed(A_GAME), SettingsKept::NotWritten);
    assert!(!here.is_on_in(A_GAME), "the switch moved");
    assert_eq!(
        std::fs::read_to_string(&file).expect("it reads"),
        "not the file ["
    );
}

/// **A file another login could have written is drawn off**, even naming the
/// game: an agent cannot switch this on for itself.
#[test]
fn a_file_others_could_write_is_drawn_off() {
    let (_at, file) = somewhere("others");
    alo_remembering::pictures_and_keys_switched(&file, A_GAME, true).expect("switched");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o646)).expect("mode");
    assert!(!PicturesAndKeysHere::at_sign_in(&file).is_on_in(A_GAME));
}
