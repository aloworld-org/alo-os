//! What should exist, and what making it twice comes to.
//!
//! The owner cannot be asserted as the person: a test runs as whoever runs it,
//! and `chown` to somebody else needs privilege. So what is held here is the
//! **decision** — [`super::what_to_make`], which is two numbers and a path and
//! answers the same on any host — and the **modes**, which a test can set and
//! read back as itself.
//!
//! Whether the owner really changes for a **different** account is a question
//! for a machine with this unit's capability, and it is not asserted here
//! rather than being implied by a green suite. ADR 0119 measured it: with only
//! `cap_chown` in the bounding set, the whole sequence succeeds and the owner
//! afterwards is the account asked for.

use std::{
    fs,
    os::unix::fs::{MetadataExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
};

use super::{A_PERSONS_MODE, THE_FOLDERS_MODE, made_for, what_to_make};
use crate::failing::NotMade;

/// The person's number, on the image this repository ships.
const THE_PERSON: u32 = 1000;

/// A folder of this test's own, named so two tests cannot collide.
fn a_folder(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-undo-place-{}-{what}", std::process::id()));
    drop(fs::remove_dir_all(&at));
    at
}

/// Whoever is running this test, as the machine numbers them.
///
/// **The disk tests below make a place for *this* account and not for account
/// 1000**, because giving a directory to somebody else needs privilege and
/// these run as whoever runs them — on a lane that is a person, in CI it is
/// root. Giving a directory to yourself is permitted either way, so the modes,
/// which are what these tests are about, are measured on every machine.
///
/// Read from a directory this test just made rather than from a syscall, which
/// `std` has no safe spelling for and which would be a dependency for one
/// number.
fn whoever_is_running_this() -> u32 {
    let at = std::env::temp_dir().join(format!("alo-undo-whoami-{}", std::process::id()));
    fs::create_dir_all(&at).unwrap();
    let mine = fs::metadata(&at).unwrap().uid();
    drop(fs::remove_dir_all(&at));
    mine
}

/// **The folder and the person's directory, in that order, with those modes.**
///
/// The order matters and is asserted: the second goes inside the first, so a
/// list that put them the other way round would make a directory and then its
/// parent.
#[test]
fn what_should_exist_is_the_folder_then_the_person() {
    let under = PathBuf::from("/var/lib/alo-undo");
    let [folder, theirs] = what_to_make(&under, THE_PERSON);

    assert_eq!(folder.at, under);
    assert_eq!(folder.mode, 0o711);
    assert_eq!(folder.owner, None, "the folder itself stays root's");

    assert_eq!(theirs.at, under.join("1000"));
    assert_eq!(theirs.mode, 0o700);
    assert_eq!(theirs.owner, Some(THE_PERSON));

    assert!(
        theirs.at.starts_with(&folder.at),
        "the person's directory is not inside the folder, so the order is wrong"
    );
}

/// **The modes are what ADR 0045 term 8 says**, asserted as numbers so a change
/// to either constant has to be deliberate.
#[test]
fn the_modes_are_the_ones_the_decision_names() {
    assert_eq!(THE_FOLDERS_MODE, 0o711, "the folder must be traversable");
    assert_eq!(A_PERSONS_MODE, 0o700, "a person's directory is their own");
    assert_ne!(
        THE_FOLDERS_MODE, 0o700,
        "0700 on the folder is the mode that made taking a bracket impossible, and is what \
         ADR 0045 term 8 exists to correct"
    );
    assert_ne!(
        THE_FOLDERS_MODE & 0o044,
        0o044,
        "the folder must not be listable: that would tell one person who else uses the machine"
    );
}

/// **Making it works, and the modes are on the disk afterwards.**
#[test]
fn making_it_puts_the_modes_on_the_disk() {
    let under = a_folder("made");
    let me = whoever_is_running_this();
    let theirs = made_for(&under, me).unwrap();

    assert_eq!(theirs, under.join(me.to_string()));
    assert!(theirs.is_dir());
    assert_eq!(
        fs::metadata(&under).unwrap().permissions().mode() & 0o777,
        THE_FOLDERS_MODE,
        "the folder's mode is not what was asked for, which `umask` does to a DirBuilder"
    );
    assert_eq!(
        fs::metadata(&theirs).unwrap().permissions().mode() & 0o777,
        A_PERSONS_MODE
    );

    drop(fs::remove_dir_all(&under));
}

/// **Making it twice is not a failure, and puts a widened mode back.**
///
/// Every sign-in after the first finds everything made. This is the case a
/// `DirBuilder`'s own `mode` is silent about — it is ignored for a directory
/// that already exists — so without the explicit set afterwards, a folder
/// somebody widened by hand would stay wide forever.
#[test]
fn signing_in_again_narrows_a_folder_somebody_widened() {
    let under = a_folder("again");
    let me = whoever_is_running_this();
    made_for(&under, me).unwrap();

    // Somebody widens it by hand, which is the case this is about.
    fs::set_permissions(&under, fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(
        fs::metadata(&under).unwrap().permissions().mode() & 0o777,
        0o777,
        "the test did not manage to widen it, so it is not measuring anything"
    );

    made_for(&under, me).unwrap();
    assert_eq!(
        fs::metadata(&under).unwrap().permissions().mode() & 0o777,
        THE_FOLDERS_MODE,
        "a second sign-in left the folder wider than the decision allows"
    );

    drop(fs::remove_dir_all(&under));
}

/// **A folder that cannot be made is said, and says which one.**
///
/// Under a path that is a *file*, so the machine refuses and the refusal is the
/// machine's rather than a rule's.
#[test]
fn a_folder_that_cannot_be_made_names_itself() {
    let in_the_way = a_folder("in-the-way");
    fs::create_dir_all(in_the_way.parent().unwrap()).unwrap();
    fs::write(&in_the_way, b"not a folder").unwrap();

    let why = made_for(&in_the_way, whoever_is_running_this()).unwrap_err();
    match why {
        NotMade::TheFolder { ref at, .. } => assert_eq!(at, &in_the_way),
        // Named rather than caught by a wildcard, so a sixth reason added to
        // `NotMade` makes the compiler ask what this test should say about it.
        ref other @ (NotMade::NotTheirs { .. }
        | NotMade::NotAnAccountNumber(_)
        | NotMade::NotAnAbsoluteFolder(_)) => {
            panic!("a folder that cannot be made was refused for another reason: {other}")
        }
    }
    // And it says what it is in words somebody reading a journal can use.
    assert!(format!("{why}").contains("could not be made"), "{why}");

    drop(fs::remove_file(&in_the_way));
}

/// **A folder that is not an absolute path is refused**, before anything is
/// made.
///
/// A relative one would be resolved against wherever systemd happened to start
/// this, which is not a place anybody chose. Refused first, so a bad argument
/// leaves the machine exactly as it was.
#[test]
fn a_folder_that_is_not_absolute_is_refused() {
    let why = made_for(Path::new("undo"), whoever_is_running_this()).unwrap_err();
    assert!(
        matches!(why, NotMade::NotAnAbsoluteFolder(_)),
        "the wrong refusal: {why}"
    );
    assert!(format!("{why}").contains("absolute"), "{why}");
}

/// **An account number that is not a number is its own refusal**, and says the
/// text it was given.
///
/// Said rather than defaulted: a helper that read `ada` as account 0 would give
/// the machine's own root directory to nobody and report success.
#[test]
fn an_account_number_that_is_not_a_number_is_said() {
    let why = NotMade::NotAnAccountNumber("ada".to_owned());
    let said = format!("{why}");
    assert!(said.contains("ada"), "{said}");
    assert!(said.contains("nobody to give a folder to"), "{said}");
}
