//! Giving a name, giving it again, and every way it is not given.
//!
//! **The account a test gives a name to is whoever runs it**, because giving a
//! file to somebody else needs `CAP_CHOWN` and a lane runs as a person. Where
//! the test runs as root — as the gate does — root cannot have an account
//! (`alo_accounts` refuses uid 0), so the name is given to account 1000, which
//! root may. The owner is asserted either way: it is the account asked for.

use std::{
    fs,
    os::unix::fs::{MetadataExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
};

use super::given_to;
use crate::failing::NotGiven;

/// A folder of this test's own, named so two tests cannot collide.
fn a_folder(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-names-given-{}-{what}", std::process::id()));
    drop(fs::remove_dir_all(&at));
    fs::create_dir_all(&at).unwrap();
    at
}

/// The account this test may give a file to: itself, or 1000 when it is root.
fn an_account_we_may_give_to() -> u32 {
    let me = rustix::process::geteuid().as_raw();
    if me == 0 { 1000 } else { me }
}

/// A store, kept beside the test, holding one account with this name.
fn a_store(folder: &Path, name: &str, person: u32) -> PathBuf {
    let mut store = alo_accounts::Accounts::none().unwrap();
    store.created(name, person, "a password").unwrap();
    let at = folder.join("accounts.toml");
    alo_accounts::kept(&at, &store).unwrap();
    at
}

/// **The name is given, only the person may read it, and it reads back.**
#[test]
fn a_name_is_given_to_the_person_and_reads_back() {
    let folder = a_folder("given");
    let person = an_account_we_may_give_to();
    let accounts = a_store(&folder, "anna", person);
    let under = folder.join("names");

    let theirs = given_to(&under, &accounts, person).unwrap();

    assert_eq!(theirs, under.join(person.to_string()));
    let seen = fs::metadata(&theirs).unwrap();
    assert_eq!(seen.uid(), person, "the file is not the person's");
    assert_eq!(
        seen.permissions().mode() & 0o777,
        0o600,
        "somebody other than the person could read their name"
    );
    assert_eq!(
        fs::metadata(&under).unwrap().permissions().mode() & 0o777,
        0o711,
        "the folder could be listed, which says who else has a name here"
    );
    assert_eq!(
        alo_accounts::their_name(&under, person).unwrap().as_deref(),
        Some("anna")
    );
    assert!(
        !under.join(format!(".{person}.new")).exists(),
        "the sibling it was written in was left behind"
    );

    drop(fs::remove_dir_all(&folder));
}

/// **A name changed in the store is picked up at the next sign-in**, and the
/// second giving needs nothing the first did not.
#[test]
fn a_changed_name_is_given_at_the_next_sign_in() {
    let folder = a_folder("again");
    let person = an_account_we_may_give_to();
    let under = folder.join("names");

    let accounts = a_store(&folder, "anna", person);
    given_to(&under, &accounts, person).unwrap();

    let accounts = a_store(&folder, "anne", person);
    given_to(&under, &accounts, person).unwrap();

    assert_eq!(
        alo_accounts::their_name(&under, person).unwrap().as_deref(),
        Some("anne")
    );

    drop(fs::remove_dir_all(&folder));
}

/// **A sibling left by a run that died is not a failure.**
#[test]
fn a_sibling_left_behind_is_replaced() {
    let folder = a_folder("left");
    let person = an_account_we_may_give_to();
    let accounts = a_store(&folder, "anna", person);
    let under = folder.join("names");
    fs::create_dir_all(&under).unwrap();
    fs::write(under.join(format!(".{person}.new")), "half a na").unwrap();

    given_to(&under, &accounts, person).unwrap();

    assert_eq!(
        alo_accounts::their_name(&under, person).unwrap().as_deref(),
        Some("anna")
    );
    drop(fs::remove_dir_all(&folder));
}

/// **A number the store has nobody for gives nothing**, and leaves no file.
#[test]
fn a_number_with_no_account_gives_nothing() {
    let folder = a_folder("nobody");
    let person = an_account_we_may_give_to();
    let accounts = a_store(&folder, "anna", person);
    let under = folder.join("names");

    let why = given_to(&under, &accounts, person + 1).unwrap_err();

    assert!(
        matches!(why, NotGiven::NobodyNumbered(n) if n == person + 1),
        "the wrong refusal: {why}"
    );
    assert!(
        !under.join((person + 1).to_string()).exists(),
        "a name file was made for nobody"
    );
    drop(fs::remove_dir_all(&folder));
}

/// **A store that is not there gives nothing**, and says it was the store.
#[test]
fn a_store_that_is_not_there_gives_nothing() {
    let folder = a_folder("no-store");
    let under = folder.join("names");

    let why = given_to(&under, &folder.join("accounts.toml"), 1000).unwrap_err();

    match why {
        NotGiven::TheAccounts(_) => {}
        ref other @ (NotGiven::NotAnAccountNumber(_)
        | NotGiven::NotAnAbsoluteFolder(_)
        | NotGiven::NobodyNumbered(_)
        | NotGiven::TheFile { .. }
        | NotGiven::NotTheirs { .. }) => {
            panic!("a missing store was refused for another reason: {other}")
        }
    }
    assert!(format!("{why}").contains("account store"), "{why}");
    drop(fs::remove_dir_all(&folder));
}

/// **A folder that is not absolute is refused before the store is read.**
#[test]
fn a_folder_that_is_not_absolute_is_refused() {
    let why = given_to(Path::new("names"), Path::new("/nowhere"), 1000).unwrap_err();
    assert!(
        matches!(why, NotGiven::NotAnAbsoluteFolder(_)),
        "the wrong refusal: {why}"
    );
}

/// **A folder that cannot be made says which**, and the store is untouched.
#[test]
fn a_folder_that_cannot_be_made_names_itself() {
    let folder = a_folder("in-the-way");
    let person = an_account_we_may_give_to();
    let accounts = a_store(&folder, "anna", person);
    let in_the_way = folder.join("names");
    fs::write(&in_the_way, b"not a folder").unwrap();

    let why = given_to(&in_the_way, &accounts, person).unwrap_err();

    assert!(
        matches!(why, NotGiven::TheFile { ref at, .. } if at == &in_the_way),
        "the wrong refusal: {why}"
    );
    assert!(format!("{why}").contains("could not be written"), "{why}");
    drop(fs::remove_dir_all(&folder));
}
