use std::os::unix::fs::PermissionsExt;

use alo_capability::{AloInApplications, AloInTheTerminal, HowCodeRuns};

use super::*;
use crate::testing::a_folder_of_our_own;
use crate::{
    alo_in_applications_kept, alo_in_applications_remembered, alo_in_the_terminal_kept,
    alo_in_the_terminal_remembered, the_level_kept, the_level_remembered,
};

/// A game: an application that draws its own window and describes nothing.
const A_GAME: &str = "net.supertuxkart.SuperTuxKart";

/// **A machine arrives with pictures and keys on in no application**: no
/// file, and a file holding only a level, both answer none.
#[test]
fn a_machine_arrives_with_pictures_and_keys_nowhere() {
    let folder = a_folder_of_our_own("pictures-arrive");
    let at = folder.join("how-code-runs.toml");
    assert_eq!(
        pictures_and_keys_remembered(&at).unwrap(),
        PicturesAndKeys::default()
    );
    the_level_kept(&at, HowCodeRuns::AtFullTrust).unwrap();
    assert!(!pictures_and_keys_remembered(&at).unwrap().allows(A_GAME));
}

/// **Switched on in one application, it is read back on in that one alone**,
/// and switched off it reads off — with a file still there that says so, not a
/// file that says nothing.
#[test]
fn switched_on_and_off_in_one_application_is_read_back_from_the_disk() {
    let folder = a_folder_of_our_own("pictures-switched");
    let at = folder.join("how-code-runs.toml");
    let on = pictures_and_keys_switched(&at, A_GAME, true).unwrap();
    assert!(on.allows(A_GAME));
    let read = pictures_and_keys_remembered(&at).unwrap();
    assert!(read.allows(A_GAME));
    assert!(!read.allows("org.mozilla.firefox"));
    let off = pictures_and_keys_switched(&at, A_GAME, false).unwrap();
    assert!(!off.allows(A_GAME));
    assert!(
        !pictures_and_keys_remembered(&at).unwrap().allows(A_GAME),
        "switched off, it was read back on"
    );
    assert_eq!(
        the_level_remembered(&at).unwrap(),
        None,
        "a level was invented"
    );
}

/// **A terminal is never switched on**, even asked for by name.
#[test]
fn a_terminal_is_not_switched_on() {
    let folder = a_folder_of_our_own("pictures-terminal");
    let at = folder.join("how-code-runs.toml");
    let after = pictures_and_keys_switched(&at, "app.devsuite.Ptyxis", true).unwrap();
    assert!(!after.allows("app.devsuite.Ptyxis"));
    assert!(
        !pictures_and_keys_remembered(&at)
            .unwrap()
            .allows("app.devsuite.Ptyxis")
    );
}

/// **Every other answer in the file survives a switch, and a switch survives
/// every other answer being written.**
#[test]
fn every_answer_survives_pictures_being_switched_and_the_other_way_round() {
    let folder = a_folder_of_our_own("pictures-kept");
    let at = folder.join("how-code-runs.toml");
    the_level_kept(&at, HowCodeRuns::AskingEachTime).unwrap();
    alo_in_the_terminal_kept(&at, AloInTheTerminal::ActsWithoutAsking).unwrap();
    alo_in_applications_kept(&at, AloInApplications::InTheirWindow).unwrap();
    pictures_and_keys_switched(&at, A_GAME, true).unwrap();
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
        AloInApplications::InTheirWindow
    );
    the_level_kept(&at, HowCodeRuns::AtFullTrust).unwrap();
    alo_in_the_terminal_kept(&at, AloInTheTerminal::AsksFirst).unwrap();
    alo_in_applications_kept(&at, AloInApplications::InItsOwnCopy).unwrap();
    assert!(
        pictures_and_keys_remembered(&at).unwrap().allows(A_GAME),
        "writing another answer forgot pictures and keys"
    );
}

/// **A second application switched on keeps the first**: the switch reads the
/// file at the press rather than writing back a list it held.
#[test]
fn switching_a_second_application_keeps_the_first() {
    let folder = a_folder_of_our_own("pictures-two");
    let at = folder.join("how-code-runs.toml");
    pictures_and_keys_switched(&at, A_GAME, true).unwrap();
    pictures_and_keys_switched(&at, "org.mozilla.firefox", true).unwrap();
    let read = pictures_and_keys_remembered(&at).unwrap();
    assert!(read.allows(A_GAME) && read.allows("org.mozilla.firefox"));
}

/// **A file another login could write is refused, even naming the game** — so
/// an agent cannot switch this on for itself.
#[test]
fn a_file_others_could_write_is_refused_for_pictures() {
    let folder = a_folder_of_our_own("pictures-untrusted");
    let at = folder.join("how-code-runs.toml");
    pictures_and_keys_switched(&at, A_GAME, true).unwrap();
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o646)).unwrap();
    let read = pictures_and_keys_remembered(&at);
    assert!(matches!(
        read,
        Err(NotRemembered::WritableByOthers { mode: 0o646, .. })
    ));
    assert!(!PicturesAndKeys::as_read(read).allows(A_GAME));
}

/// **Switching over a file that would not read is refused, and the file is
/// left exactly as it was** — written over, it would lose a level.
#[test]
fn switching_over_a_file_that_would_not_read_is_refused_and_writes_nothing() {
    let folder = a_folder_of_our_own("pictures-unreadable");
    let at = folder.join("how-code-runs.toml");
    let broken = "format = 1\nchose = \"full-trust\"\n";
    std::fs::write(&at, broken).unwrap();
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(pictures_and_keys_switched(&at, A_GAME, true).is_err());
    assert_eq!(std::fs::read_to_string(&at).unwrap(), broken);
}

/// **A file holding only pictures and keys is read by every other reader as
/// their own default**, not refused.
#[test]
fn a_file_holding_only_pictures_reads_as_every_other_default() {
    let folder = a_folder_of_our_own("pictures-only");
    let at = folder.join("how-code-runs.toml");
    pictures_and_keys_switched(&at, A_GAME, true).unwrap();
    assert_eq!(the_level_remembered(&at).unwrap(), None);
    assert_eq!(
        alo_in_the_terminal_remembered(&at).unwrap(),
        AloInTheTerminal::default()
    );
    assert_eq!(
        alo_in_applications_remembered(&at).unwrap(),
        AloInApplications::default()
    );
}

/// **Switching off keeps the other three answers too**, not only switching on.
#[test]
fn switching_off_keeps_every_other_answer() {
    let folder = a_folder_of_our_own("pictures-off-kept");
    let at = folder.join("how-code-runs.toml");
    pictures_and_keys_switched(&at, A_GAME, true).unwrap();
    the_level_kept(&at, HowCodeRuns::InASealedBox).unwrap();
    alo_in_the_terminal_kept(&at, AloInTheTerminal::ActsWithoutAsking).unwrap();
    alo_in_applications_kept(&at, AloInApplications::InTheirWindow).unwrap();
    pictures_and_keys_switched(&at, A_GAME, false).unwrap();
    assert_eq!(
        the_level_remembered(&at).unwrap(),
        Some(HowCodeRuns::InASealedBox)
    );
    assert_eq!(
        alo_in_the_terminal_remembered(&at).unwrap(),
        AloInTheTerminal::ActsWithoutAsking
    );
    assert_eq!(
        alo_in_applications_remembered(&at).unwrap(),
        AloInApplications::InTheirWindow
    );
}

/// **Switching over a file another login could write is refused, and its
/// bytes are left as they were** — believing it to write over it would carry
/// that login's answers into a file that is then the person's.
#[test]
fn switching_over_a_file_others_could_write_is_refused() {
    let folder = a_folder_of_our_own("pictures-switch-untrusted");
    let at = folder.join("how-code-runs.toml");
    the_level_kept(&at, HowCodeRuns::AtFullTrust).unwrap();
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o646)).unwrap();
    let before = std::fs::read(&at).unwrap();
    assert!(pictures_and_keys_switched(&at, A_GAME, true).is_err());
    assert_eq!(std::fs::read(&at).unwrap(), before);
}

/// **Every terminal, asked for by name, leaves a file that still reads** with
/// the level beside it — never one the reader then refuses whole.
#[test]
fn every_terminal_asked_for_leaves_a_file_that_reads() {
    let folder = a_folder_of_our_own("pictures-every-terminal");
    let at = folder.join("how-code-runs.toml");
    the_level_kept(&at, HowCodeRuns::AskingEachTime).unwrap();
    for terminal in alo_capability::A_PERSONS_OWN {
        let after = pictures_and_keys_switched(&at, terminal, true).unwrap();
        assert!(!after.allows(terminal));
        let read = pictures_and_keys_remembered(&at).unwrap();
        assert!(!read.allows(terminal), "{terminal} was written");
    }
    assert_eq!(
        the_level_remembered(&at).unwrap(),
        Some(HowCodeRuns::AskingEachTime)
    );
}
