use std::os::unix::fs::PermissionsExt;

use alo_capability::{AloInApplications, AloInTheTerminal, HowCodeRuns, PicturesAndKeys};

use super::*;
use crate::testing::a_folder_of_our_own;
use crate::the_level::NotTheLevel;
use crate::{
    alo_in_applications_kept, alo_in_applications_remembered, alo_in_the_terminal_kept,
    alo_in_the_terminal_remembered, pictures_and_keys_remembered, pictures_and_keys_switched,
    the_level_kept, the_level_remembered,
};

/// A game: an application that draws its own window and describes nothing.
const A_GAME: &str = "net.supertuxkart.SuperTuxKart";

/// A service the person added.
fn lantern() -> WhoReadsPictures {
    WhoReadsPictures::a_provider("Lantern").unwrap()
}

/// This text written where the level is kept, readable by its owner alone.
fn kept_as(at: &Path, text: &str) {
    std::fs::write(at, text).unwrap();
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o600)).unwrap();
}

/// **A machine arrives with no one reading pictures**: no file, and a file
/// holding only a level, both answer no one.
#[test]
fn a_machine_arrives_with_no_one_reading_pictures() {
    let at = a_folder_of_our_own("readers-arrive").join("how-code-runs.toml");
    assert_eq!(
        who_reads_pictures_remembered(&at).unwrap(),
        WhoReadsPictures::NoOne
    );
    the_level_kept(&at, HowCodeRuns::AtFullTrust).unwrap();
    assert_eq!(
        who_reads_pictures_remembered(&at).unwrap(),
        WhoReadsPictures::NoOne
    );
}

/// **Each choice goes down to the disk and comes back**, a service's name byte
/// for byte, case and inner space and all — and choosing no one again leaves
/// no service's name behind.
#[test]
fn every_reader_goes_down_and_comes_back() {
    let at = a_folder_of_our_own("readers-round").join("how-code-runs.toml");
    for chosen in [
        WhoReadsPictures::ThisComputer,
        WhoReadsPictures::a_provider("LanTern AI").unwrap(),
        WhoReadsPictures::NoOne,
    ] {
        assert_eq!(
            who_reads_pictures_chosen(&at, chosen.clone()).unwrap(),
            chosen
        );
        assert_eq!(who_reads_pictures_remembered(&at).unwrap(), chosen);
    }
    let text = std::fs::read_to_string(&at).unwrap();
    assert!(text.contains("pictures-read-by = \"no-one\""), "{text:?}");
    assert!(
        !text.contains("pictures-read-by-provider"),
        "a service was left behind: {text:?}"
    );
}

/// **A file holding only the reader says something**: it is not refused, and
/// no level is invented from it.
#[test]
fn a_file_holding_only_the_reader_reads() {
    let at = a_folder_of_our_own("readers-only").join("how-code-runs.toml");
    kept_as(&at, "format = 1\npictures-read-by = \"no-one\"\n");
    assert_eq!(
        who_reads_pictures_remembered(&at).unwrap(),
        WhoReadsPictures::NoOne
    );
    assert_eq!(the_level_remembered(&at).unwrap(), None);
    kept_as(
        &at,
        "format = 1\npictures-read-by = \"a-provider\"\npictures-read-by-provider = \"Lantern\"\n",
    );
    assert_eq!(who_reads_pictures_remembered(&at).unwrap(), lantern());
}

/// **Choosing keeps every other answer in the file, and every other writer
/// keeps the reader.**
#[test]
fn every_other_answer_survives_a_reader_chosen_and_the_other_way_round() {
    let at = a_folder_of_our_own("readers-kept").join("how-code-runs.toml");
    the_level_kept(&at, HowCodeRuns::AskingEachTime).unwrap();
    alo_in_the_terminal_kept(&at, AloInTheTerminal::ActsWithoutAsking).unwrap();
    alo_in_applications_kept(&at, AloInApplications::InTheirWindow).unwrap();
    pictures_and_keys_switched(&at, A_GAME, true).unwrap();
    who_reads_pictures_chosen(&at, lantern()).unwrap();
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
    assert!(pictures_and_keys_remembered(&at).unwrap().allows(A_GAME));
    the_level_kept(&at, HowCodeRuns::InASealedBox).unwrap();
    pictures_and_keys_switched(&at, A_GAME, false).unwrap();
    assert_eq!(
        who_reads_pictures_remembered(&at).unwrap(),
        lantern(),
        "another writer forgot the reader"
    );
    assert_eq!(
        pictures_and_keys_remembered(&at).unwrap(),
        PicturesAndKeys::default()
    );
}

/// **A reader the file does not say cleanly refuses the whole file** — read
/// neither as the service it names nor as no one, so a person is told rather
/// than quietly given a reader.
#[test]
fn a_reader_the_file_does_not_say_cleanly_refuses_the_whole_file() {
    let at = a_folder_of_our_own("readers-refused").join("how-code-runs.toml");
    for text in [
        "format = 1\npictures-read-by = \"no-one\"\npictures-read-by-provider = \"Lantern\"\n",
        "format = 1\npictures-read-by = \"this-computer\"\npictures-read-by-provider = \"Lantern\"\n",
        "format = 1\nchose = \"at-full-trust\"\npictures-read-by-provider = \"Lantern\"\n",
        "format = 1\npictures-read-by = \"a-provider\"\n",
        "format = 1\npictures-read-by = \"a-provider\"\npictures-read-by-provider = \"\"\n",
        "format = 1\npictures-read-by = \"a-provider\"\npictures-read-by-provider = \" Lantern\"\n",
    ] {
        kept_as(&at, text);
        let read = who_reads_pictures_remembered(&at);
        assert!(
            matches!(
                read,
                Err(NotRemembered::NotTheLevel {
                    why: NotTheLevel::NotAPicturesReader { .. },
                    ..
                })
            ),
            "{text:?} was read as {read:?}"
        );
        assert_eq!(WhoReadsPictures::as_read(read), WhoReadsPictures::NoOne);
        assert!(
            the_level_remembered(&at).is_err(),
            "{text:?} was refused for the reader and believed for the level"
        );
    }
    for said in ["No-One", "alos-own", "built-in", "local", "Lantern", ""] {
        kept_as(&at, &format!("format = 1\npictures-read-by = \"{said}\"\n"));
        assert!(
            matches!(
                who_reads_pictures_remembered(&at),
                Err(NotRemembered::NotTheLevel {
                    why: NotTheLevel::NotAPicturesReader { said: s },
                    ..
                }) if s == said
            ),
            "{said:?} was not refused by name"
        );
    }
}

/// **A file another login could write cannot choose a service** — so an agent
/// cannot choose its own reader, least of all one a picture would leave for —
/// and choosing over it writes nothing.
#[test]
fn a_file_others_could_write_reads_as_no_one_and_is_not_chosen_over() {
    let at = a_folder_of_our_own("readers-untrusted").join("how-code-runs.toml");
    who_reads_pictures_chosen(&at, lantern()).unwrap();
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o646)).unwrap();
    let read = who_reads_pictures_remembered(&at);
    assert!(
        matches!(
            read,
            Err(NotRemembered::WritableByOthers { mode: 0o646, .. })
        ),
        "{read:?}"
    );
    assert_eq!(WhoReadsPictures::as_read(read), WhoReadsPictures::NoOne);
    let before = std::fs::read(&at).unwrap();
    assert!(who_reads_pictures_chosen(&at, WhoReadsPictures::ThisComputer).is_err());
    assert_eq!(std::fs::read(&at).unwrap(), before);
}

/// **Choosing over a file that would not read is refused and writes nothing**
/// — written over, it would lose a level the person chose.
#[test]
fn choosing_over_a_file_that_would_not_read_writes_nothing() {
    let at = a_folder_of_our_own("readers-broken").join("how-code-runs.toml");
    let broken = "format = 1\nchose = \"full-trust\"\n";
    kept_as(&at, broken);
    assert!(who_reads_pictures_chosen(&at, lantern()).is_err());
    assert_eq!(std::fs::read_to_string(&at).unwrap(), broken);
}

/// **A service named around the guard is not written**: the file keeps its
/// level and still reads, rather than holding a name the reader then refuses
/// whole.
#[test]
fn a_service_built_around_the_guard_does_not_break_the_file() {
    let at = a_folder_of_our_own("readers-bypass").join("how-code-runs.toml");
    the_level_kept(&at, HowCodeRuns::AskingEachTime).unwrap();
    let odd = WhoReadsPictures::AProvider(" Lantern".to_owned());
    assert!(who_reads_pictures_chosen(&at, odd.clone()).is_err());
    assert_eq!(
        the_level_remembered(&at).unwrap(),
        Some(HowCodeRuns::AskingEachTime),
        "the level was lost"
    );
    assert_eq!(
        who_reads_pictures_remembered(&at).unwrap(),
        WhoReadsPictures::NoOne
    );
}
