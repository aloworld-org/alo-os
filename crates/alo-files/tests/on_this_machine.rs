//! The file half against a real filesystem.
//!
//! Everything in the crate's own tests decides against a filesystem written
//! down in the test, which is how the refusals can be checked on every platform
//! the tests run on. This file is the other half of that bargain: the same
//! journey, from a declared verb to a token an executor could open a file with,
//! against a folder that really exists on the machine running the tests.
//!
//! It is not the hardware verification `CLAUDE.md` asks for — that is a
//! certified machine, and this is whatever the tests were run on. What it does
//! prove is that [`OnThisMachine`] and the grant check agree about real paths,
//! which two things written apart from each other would otherwise be trusted to
//! do.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Authorised, Given, Grant, Grantee, Grants, Reach, Verbs};
use alo_files::{OnThisMachine, Real, Resolving, Touching, file_verbs};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants in these tests last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the tests grant things to.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// A folder of this test's own, resolved, so that what is granted and what is
/// asked about are spelled the way this machine spells them.
///
/// That is not a test convenience. A grant is over a place, so a person picking
/// a folder grants the *real* one — on Windows a resolved path carries a
/// `\\?\` prefix that the path it was typed from does not, and a grant made
/// over the unresolved spelling would match nothing. `docs/quirks.md` says so.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-files-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// Everything `@files` has been granted, over one real folder.
fn granting(folder: &Path) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(folder.to_path_buf()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants
}

/// The file verbs, on a list.
fn verbs() -> Verbs {
    file_verbs().unwrap()
}

/// A path as a verb's argument would arrive: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// The ordinary day: a granted folder that really is where it says it is, read
/// from a declared verb all the way to something an executor could open.
#[test]
fn a_granted_folder_that_is_really_there_may_be_touched() {
    let root = a_folder_of_our_own("ordinary");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();
    fs::write(invoices.join("march.pdf"), b"an invoice").unwrap();

    let grants = granting(&invoices);
    let call = verbs()
        .call("list_folder", &[("folder", as_given(&invoices))])
        .unwrap();
    let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine).unwrap();

    assert_eq!(
        touching.real("folder").map(Real::as_path),
        Some(fs::canonicalize(&invoices).unwrap().as_path())
    );
    assert_eq!(touching.verb(), "list_folder");

    let _ = fs::remove_dir_all(&root);
}

/// A file that is not there is refused, and the refusal says what to do about
/// it rather than reporting an error number.
#[test]
fn a_file_that_is_not_there_is_refused() {
    let root = a_folder_of_our_own("missing");
    let invoices = root.join("Invoices");
    fs::create_dir_all(&invoices).unwrap();

    let grants = granting(&invoices);
    let call = verbs()
        .call(
            "read_file",
            &[("file", as_given(&invoices.join("april.pdf")))],
        )
        .unwrap();
    let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
    let refused = Touching::of(authorised, &grants, &OnThisMachine).unwrap_err();

    assert!(
        refused.to_string().contains("there is nothing at"),
        "{refused}"
    );
    assert_eq!(refused.call(), &call);

    let _ = fs::remove_dir_all(&root);
}

/// A folder outside every grant is refused without the machine being asked
/// about it, whether it is there or not.
#[test]
fn a_folder_nobody_granted_is_refused_whether_or_not_it_is_there() {
    let root = a_folder_of_our_own("ungranted");
    let invoices = root.join("Invoices");
    let taxes = root.join("Taxes");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&taxes).unwrap();

    let grants = granting(&invoices);
    let call = verbs()
        .call("list_folder", &[("folder", as_given(&taxes))])
        .unwrap();
    // It never becomes an authorisation at all: the deciding crate refuses it
    // lexically, and nothing here is reached.
    let refused = Authorised::read(&call, &files(), &grants, noon()).unwrap_err();
    assert!(
        refused.to_string().contains("has not been granted"),
        "{refused}"
    );

    let _ = fs::remove_dir_all(&root);
}

/// **The escape, for real.** A link inside a granted folder pointing outside it
/// is refused, against a filesystem rather than against a description of one.
///
/// Unix only: creating a symbolic link on Windows needs a privilege a
/// developer's account may not have, and a test that quietly skips itself is a
/// test that stops being run. The same refusal is asserted on every platform in
/// the crate's own tests.
#[cfg(unix)]
#[test]
fn a_link_out_of_a_granted_folder_is_refused_on_a_real_filesystem() {
    let root = a_folder_of_our_own("escape");
    let invoices = root.join("Invoices");
    let elsewhere = root.join("Elsewhere");
    fs::create_dir_all(&invoices).unwrap();
    fs::create_dir_all(&elsewhere).unwrap();
    let secret = elsewhere.join("secret.txt");
    fs::write(&secret, b"not an invoice").unwrap();
    let link = invoices.join("march.pdf");
    std::os::unix::fs::symlink(&secret, &link).unwrap();

    let grants = granting(&invoices);
    let call = verbs()
        .call("read_file", &[("file", as_given(&link))])
        .unwrap();

    // Lexically it is inside the granted folder, and it is authorised.
    let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();

    let refused = Touching::of(authorised, &grants, &OnThisMachine).unwrap_err();
    assert!(refused.to_string().contains("really leads to"), "{refused}");
    assert!(refused.to_string().contains("secret.txt"), "{refused}");
    assert_eq!(refused.call(), &call);

    let _ = fs::remove_dir_all(&root);
}
