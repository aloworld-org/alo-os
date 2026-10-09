//! Desktop entries written to real directories, read back as a list.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// Write these entries into a directory of their own, and return it.
///
/// **Named from the contents, not from a count.** The first version keyed the
/// directory on the number of entries and the first filename, so the two calls
/// in `the_persons_own_directory_wins` — one entry each, both
/// `browser.desktop` — produced the **same** directory and the second
/// overwrote the first. The test then failed for a reason that had nothing to
/// do with the code it was about.
fn a_place_holding(entries: &[(&str, &str)]) -> std::path::PathBuf {
    let mut fingerprint: u64 = 1469598103934665603;
    for (named, text) in entries {
        for byte in named.bytes().chain(text.bytes()) {
            fingerprint ^= u64::from(byte);
            fingerprint = fingerprint.wrapping_mul(1099511628211);
        }
    }
    let at = std::env::temp_dir().join(format!(
        "alo-installed-{}-{fingerprint:x}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for (named, text) in entries {
        let path = at.join(named);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, text).unwrap();
    }
    at
}

const A_BROWSER: &str = "[Desktop Entry]\nType=Application\nName=Browser\nExec=browser %u\n";

/// **An entry on the machine is an application in the list.**
#[test]
fn what_is_declared_is_what_is_held() {
    let at = a_place_holding(&[("browser.desktop", A_BROWSER)]);
    let installed = read(std::slice::from_ref(&at));
    assert_eq!(
        installed.how_many(),
        1,
        "one entry did not make one application"
    );
    assert!(
        installed.has("browser"),
        "the identifier is not the file's own stem"
    );
    let _ = std::fs::remove_dir_all(at);
}

/// **`NoDisplay` and `Hidden` keep an entry out, and `Type` other than
/// `Application` does too.**
///
/// Three refusals rather than one test each, because what matters is that the
/// list a person sees is the list the entries asked to be in.
#[test]
fn what_asks_not_to_be_listed_is_not_listed() {
    let at = a_place_holding(&[
        ("shown.desktop", A_BROWSER),
        (
            "helper.desktop",
            "[Desktop Entry]\nType=Application\nName=Helper\nNoDisplay=true\n",
        ),
        (
            "masked.desktop",
            "[Desktop Entry]\nType=Application\nName=Masked\nHidden=true\n",
        ),
        (
            "a-link.desktop",
            "[Desktop Entry]\nType=Link\nName=Somewhere\nURL=https://example.invalid\n",
        ),
    ]);
    let installed = read(std::slice::from_ref(&at));
    assert_eq!(
        installed.how_many(),
        1,
        "something that asked not to be listed was listed: {:?}",
        installed
            .all()
            .map(Application::identifier)
            .collect::<Vec<_>>()
    );
    assert!(installed.has("shown"));
    let _ = std::fs::remove_dir_all(at);
}

/// **An earlier directory wins**, which is what lets a person override the
/// system's copy of an application.
///
/// The reverse would silently ignore their own entry, which is the failure
/// this ordering exists to prevent.
#[test]
fn the_persons_own_directory_wins() {
    let theirs = a_place_holding(&[(
        "browser.desktop",
        "[Desktop Entry]\nType=Application\nName=Their Browser\n",
    )]);
    let system = a_place_holding(&[(
        "browser.desktop",
        "[Desktop Entry]\nType=Application\nName=System Browser\n",
    )]);
    let installed = read(&[theirs.clone(), system.clone()]);
    assert_eq!(
        installed.how_many(),
        1,
        "one identifier became two applications"
    );
    let held = installed.knows("browser").unwrap();
    assert!(
        format!("{held:?}").contains("Their Browser"),
        "the system's entry won over the person's own: {held:?}"
    );
    let _ = std::fs::remove_dir_all(theirs);
    let _ = std::fs::remove_dir_all(system);
}

/// **An entry in a subdirectory is identified with a hyphen**, which is the
/// specification's desktop file ID and not a convenience.
#[test]
fn a_subdirectory_becomes_part_of_the_identifier() {
    let at = a_place_holding(&[("kde4/konsole.desktop", A_BROWSER)]);
    let installed = read(std::slice::from_ref(&at));
    assert!(
        installed.has("kde4-konsole"),
        "a nested entry was not identified by its path: {:?}",
        installed
            .all()
            .map(Application::identifier)
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_dir_all(at);
}

/// **A `Name` belonging to an action is not the application's name.**
///
/// A desktop file carries one group per action, each with its own `Name`.
/// Reading the first `Name` anywhere would call an application after one of
/// its right-click items.
#[test]
fn an_actions_name_is_not_the_applications() {
    let at = a_place_holding(&[(
        "player.desktop",
        "[Desktop Entry]\nType=Application\nName=Player\n\n[Desktop Action Next]\nName=Next Track\n",
    )]);
    let installed = read(std::slice::from_ref(&at));
    let held = installed.knows("player").unwrap();
    assert!(
        format!("{held:?}").contains("Player") && !format!("{held:?}").contains("Next Track"),
        "an action's name reached the application: {held:?}"
    );
    let _ = std::fs::remove_dir_all(at);
}

/// **A directory that is not there is not an error**, because a machine with
/// nothing of its own installed is ordinary.
#[test]
fn a_place_that_does_not_exist_is_empty_rather_than_a_failure() {
    let installed = read(&[std::path::PathBuf::from(
        "/this/does/not/exist/applications",
    )]);
    assert!(installed.is_empty());
}

/// **An empty variable means unset**, which the specification says and which a
/// shell produces by exporting nothing.
///
/// Treating `""` as a directory would read entries out of the process's
/// working directory, which is how a launcher ends up listing whatever
/// happened to be beside the binary.
#[test]
fn an_empty_variable_is_not_a_directory() {
    let places = where_entries_live(
        Some(std::path::Path::new("/home/somebody")),
        Some(std::ffi::OsStr::new("")),
        Some(std::ffi::OsStr::new("")),
    );
    assert!(
        places.iter().all(|place| place.is_absolute()),
        "a relative directory reached the list: {places:?}"
    );
    assert_eq!(
        places.first().unwrap(),
        &std::path::PathBuf::from("/home/somebody/.local/share/applications"),
        "an empty XDG_DATA_HOME did not fall back to the specification's default"
    );
}

/// **The person's own directory is asked first**, which is the whole of why
/// `the_persons_own_directory_wins` can be true.
#[test]
fn the_persons_directory_comes_before_the_systems() {
    let places = where_entries_live(
        Some(std::path::Path::new("/home/somebody")),
        None,
        Some(std::ffi::OsStr::new("/opt/share:/usr/share")),
    );
    assert_eq!(
        places,
        vec![
            std::path::PathBuf::from("/home/somebody/.local/share/applications"),
            std::path::PathBuf::from("/opt/share/applications"),
            std::path::PathBuf::from("/usr/share/applications"),
        ]
    );
}
