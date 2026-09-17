//! A session with no folder says so in Settings, and writes nothing anywhere.
//!
//! `docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 7. A
//! login with no home directory has nowhere to keep a person's settings, and
//! that is right — but a person who moves the dock in that session and finds it
//! back at the next sign-in, with nothing that told them, has been forgotten
//! silently. So [`alo_choosing::the_persons_folder`] answers the folder or a
//! [`NoFolder`], and the refusal is a sentence in the machine's vocabulary.
//!
//! One test per way the folder is missing, each against the vocabulary
//! `alo-saying` really collects, and each held to writing nothing: not under
//! the directory the test runs in (where a relative variable followed would
//! land), not under the system's temporary directory (where a fallback would
//! land), and not in a home that exists but was not named.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use alo_choosing::{
    HomeWas, NoFolder, SESSION_NO_FOLDER, THE_FOLDER, the_persons_folder, where_the_folder_is,
};
use alo_strings::Strings;

/// Everything the machine can say, which is what a session really holds.
fn everything_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// Every place a session with no folder could have made one up, and whether
/// each is there now.
fn the_places_nothing_may_appear(relative: &[&str]) -> Vec<(PathBuf, bool)> {
    let mut places: Vec<PathBuf> = relative
        .iter()
        .flat_map(|named| {
            [
                Path::new(named).join(THE_FOLDER),
                Path::new(named).join(".config").join(THE_FOLDER),
            ]
        })
        .collect();
    places.push(std::env::temp_dir().join(THE_FOLDER));
    places.push(std::env::temp_dir().join(".config").join(THE_FOLDER));
    places
        .into_iter()
        .map(|place| {
            let there = place.exists();
            (place, there)
        })
        .collect()
}

/// The refusal a session with this environment answers, checked the way every
/// way the folder is missing is: no folder where `where_the_folder_is` has
/// none, the sentence whole and naming no path, and nothing appearing anywhere
/// a made-up folder would have gone.
fn refused(config_home: Option<&str>, home: Option<&str>) -> NoFolder {
    let config_home = config_home.map(OsStr::new);
    let home_said = home.map(OsStr::new);
    let relative: Vec<&str> = [config_home, home_said]
        .into_iter()
        .flatten()
        .filter_map(OsStr::to_str)
        .filter(|named| !named.is_empty())
        .collect();
    let before = the_places_nothing_may_appear(&relative);

    assert_eq!(where_the_folder_is(config_home, home_said), None);
    let refused = the_persons_folder(config_home, home_said)
        .expect_err("a session with no usable home answered a folder");

    let said = refused.said(&everything_this_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
    assert_eq!(said.text(), SESSION_NO_FOLDER.says());
    assert!(said.text().contains("home directory"), "{said}");
    assert!(
        said.text().contains("nothing you change now will be kept"),
        "{said}"
    );
    assert!(
        !said.text().contains('/'),
        "the sentence names a path: {said}"
    );

    assert_eq!(
        the_places_nothing_may_appear(&relative),
        before,
        "a session with no folder made one somewhere"
    );
    refused
}

/// **No `$HOME` at all.**
#[test]
fn a_session_with_no_home_says_its_changes_will_not_be_kept() {
    assert_eq!(refused(None, None).home(), HomeWas::Unset);
}

/// **A relative `$XDG_CONFIG_HOME` and no `$HOME`** — the configuration
/// directory is not followed from wherever the session started.
#[test]
fn a_relative_configuration_directory_and_no_home_says_its_changes_will_not_be_kept() {
    assert_eq!(refused(Some("config"), None).home(), HomeWas::Unset);
    assert_eq!(refused(Some(""), None).home(), HomeWas::Unset);
}

/// **A relative `$HOME`**, and an empty one, which is not a home either.
#[test]
fn a_relative_home_says_its_changes_will_not_be_kept() {
    assert_eq!(refused(None, Some("ada")).home(), HomeWas::NotAbsolute);
    assert_eq!(
        refused(Some("config"), Some("ada")).home(),
        HomeWas::NotAbsolute
    );
    assert_eq!(refused(None, Some("")).home(), HomeWas::NotAbsolute);
}

/// **A session with a home is not refused**, and a keeper's path is only ever
/// inside the folder that session names — the legitimate road beside the three
/// refused ones.
#[test]
fn a_session_with_a_home_is_handed_paths_inside_its_own_folder() {
    let home = Some(OsStr::new("/home/ada"));
    let folder = the_persons_folder(None, home).unwrap();
    assert_eq!(
        Some(folder.folder().to_owned()),
        where_the_folder_is(None, home)
    );
    let dock = folder.path_of(alo_dock::keeping::THE_FILE);
    assert_eq!(dock.parent(), Some(folder.folder()));
    assert_eq!(dock.file_name(), Some(OsStr::new("dock.toml")));

    let configured = the_persons_folder(Some(OsStr::new("/srv/ada/config")), None).unwrap();
    assert!(configured.folder().starts_with("/srv/ada/config"));
}
