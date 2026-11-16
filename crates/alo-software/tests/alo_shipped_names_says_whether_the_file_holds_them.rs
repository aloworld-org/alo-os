//! **`alo-shipped-names` says whether the pinned file holds the packages'
//! names, and never writes it.** The binary itself is run, as the names job
//! runs it: over entries that give the pinned names it agrees, over entries
//! that give another it prints what the file should hold and exits 1, and in
//! neither case does the file change.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use alo_software::{Shipped, ShippedNames};
use tempfile::TempDir;

/// The binary, as cargo built it for this test.
const THE_BINARY: &str = env!("CARGO_BIN_EXE_alo-shipped-names");

/// The committed file.
fn the_file() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("the-shipped-names.txt")
}

/// An `applications` folder with every shipped application's entry, named
/// as the pinned file names it except where `instead` names it otherwise.
fn entries(instead: Option<(&str, &str)>) -> TempDir {
    let names = ShippedNames::pinned().unwrap();
    let folder = TempDir::new().unwrap();
    let applications = folder.path().join("applications");
    fs::create_dir_all(&applications).unwrap();
    for pinned in Shipped::decided().unwrap().every() {
        let identifier = pinned.application().identifier();
        let name = match instead {
            Some((which, other)) if which == identifier => other.to_owned(),
            _ => names.name_of(identifier).unwrap().to_owned(),
        };
        fs::write(
            applications.join(format!("{identifier}.desktop")),
            format!("[Desktop Entry]\nType=Application\nName={name}\nExec=true\n"),
        )
        .unwrap();
    }
    folder
}

fn run(folder: &TempDir) -> Output {
    Command::new(THE_BINARY)
        .arg(folder.path().join("applications"))
        .output()
        .unwrap()
}

#[test]
fn entries_that_give_the_pinned_names_agree_with_the_file() {
    let before = fs::read(the_file()).unwrap();
    let ran = run(&entries(None));
    assert_eq!(ran.status.code(), Some(0), "{ran:?}");
    assert!(String::from_utf8_lossy(&ran.stdout).contains("holds the packages' own names"));
    assert_eq!(fs::read(the_file()).unwrap(), before);
}

#[test]
fn an_entry_that_gives_another_name_fails_and_prints_what_the_file_should_hold() {
    let before = fs::read(the_file()).unwrap();
    let ran = run(&entries(Some(("com.vscodium.codium", "Codium Renamed"))));
    assert_eq!(ran.status.code(), Some(1), "{ran:?}");
    let said = String::from_utf8_lossy(&ran.stdout);
    assert!(
        said.contains("com.vscodium.codium\tCodium Renamed\n"),
        "{said}"
    );
    assert!(said.contains("org.mozilla.firefox\t"), "{said}");
    // **Never written back**: the file is as it was.
    assert_eq!(fs::read(the_file()).unwrap(), before);
}

#[test]
fn it_lists_every_shipped_application_by_its_source_and_identifier() {
    let ran = Command::new(THE_BINARY)
        .arg("--identifiers")
        .output()
        .unwrap();
    assert_eq!(ran.status.code(), Some(0), "{ran:?}");
    let listed: Vec<String> = String::from_utf8_lossy(&ran.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    let expected: Vec<String> = Shipped::decided()
        .unwrap()
        .every()
        .iter()
        .map(|pinned| {
            format!(
                "{} {}",
                pinned.source().as_str(),
                pinned.application().identifier()
            )
        })
        .collect();
    assert_eq!(listed, expected);
}

/// **Asked for nothing, or for something it does not know, it refuses**
/// rather than agreeing with an empty reading.
#[test]
fn nothing_to_read_or_an_unknown_flag_is_refused() {
    for given in [&[][..], &["--write"][..], &["--identifiers", "extra"][..]] {
        let ran = Command::new(THE_BINARY).args(given).output().unwrap();
        assert_eq!(ran.status.code(), Some(2), "{given:?}: {ran:?}");
    }
    // **A flag it does not know is refused beside a folder that would agree**,
    // so it is the flag that is refused and not an empty reading.
    let agreeing = entries(None);
    let ran = Command::new(THE_BINARY)
        .arg("--write")
        .arg(agreeing.path().join("applications"))
        .output()
        .unwrap();
    assert_eq!(ran.status.code(), Some(2), "{ran:?}");
}
