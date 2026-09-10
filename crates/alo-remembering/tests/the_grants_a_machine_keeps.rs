//! What a machine keeps between one sign-in and the next, measured on a real
//! disk from a real pick.
//!
//! The unit tests beside `crate::keeping` and `crate::written` ask this crate's
//! own questions. This file asks the plan's: a grant **a person made** survives
//! a restart and is honoured afterwards, a revoked one does not come back, an
//! expired one is gone when the list is read, and a file somebody else could
//! write is refused in words.
//!
//! So the grant here is not constructed. It is
//! `alo_picking::Granting::of(alo_picking::Chosen)` over a folder walked to
//! with `alo_picking::OnThisDisk` — the one act on this machine that makes a
//! grant — because a test that built its own grant would be measuring this
//! crate against itself.
//!
//! Unix only: the file has an owner and a mode, and both are what half of this
//! is about.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grantee, Grants};
use alo_picking::{Chosen, Granting, OnThisDisk, Picker};
use alo_remembering::{NotRemembered, kept, remembered};

/// The agent this machine's person grants to.
const HERS: &str = "@files";

/// The moment this test calls noon.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grant made here lasts.
fn an_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A folder of this test's own, with an invoice in it, emptied of any earlier
/// run.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-remembering-{what}-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(at.join("Invoices")).unwrap();
    std::fs::write(at.join("Invoices").join("march.pdf"), b"an invoice").unwrap();
    at
}

/// A person, standing in their invoices folder, picking it.
fn a_folder_picked(home: &Path) -> Chosen {
    let mut picker = Picker::standing_in(home, &OnThisDisk).unwrap();
    picker.go_into("Invoices", &OnThisDisk).unwrap();
    picker.pick().unwrap()
}

/// That pick, as the grant it means.
fn a_grant_made_in(home: &Path) -> Grants {
    let mut grants = Grants::default();
    let made = Granting::to(HERS, an_hour())
        .of(&a_folder_picked(home), &mut grants, noon())
        .unwrap();
    assert!(made.is_some(), "the pick made no grant");
    grants
}

/// What the agent would ask for, inside the folder that was picked.
fn the_invoice_in(home: &Path) -> Ask {
    Ask::path(home.join("Invoices").join("march.pdf"))
}

/// **A grant a person made survives a restart and is honoured afterwards.**
///
/// Two processes, as far as the disk is concerned: one that had the grant and
/// wrote it down, and one that has nothing but the file. The second permits
/// what the first did, over the folder that was really picked, and it still
/// ends when it was always going to end.
#[test]
fn a_grant_a_person_made_survives_a_restart_and_is_honoured_afterwards() {
    let home = a_folder_of_our_own("survives");
    let at = home.join("grants.toml");

    kept(&at, &a_grant_made_in(&home), noon()).unwrap();

    let after_a_restart = remembered(&at, noon()).unwrap();
    assert!(after_a_restart.permits(&Grantee::named(HERS), &the_invoice_in(&home), noon()));
    assert!(!after_a_restart.permits(
        &Grantee::named(HERS),
        &the_invoice_in(&home),
        noon() + an_hour()
    ));
    // And it is that person's grant to that agent, not a grant to whoever asks.
    assert!(!after_a_restart.permits(&Grantee::named("@mail"), &the_invoice_in(&home), noon()));
}

/// **A revoked grant does not come back.**
///
/// Revoked, kept, and read again by a machine that has only the file: the
/// question is not whether the list in memory forgot it but whether the disk
/// did, because a grant that returns at the next sign-in is a revocation that
/// did not happen.
#[test]
fn a_revoked_grant_does_not_come_back() {
    let home = a_folder_of_our_own("revoked");
    let at = home.join("grants.toml");
    let mut grants = a_grant_made_in(&home);
    kept(&at, &grants, noon()).unwrap();

    let held = grants.active_at(noon()).next().unwrap().id;
    assert!(grants.revoke(held));
    kept(&at, &grants, noon()).unwrap();

    let after_a_restart = remembered(&at, noon()).unwrap();
    assert!(after_a_restart.is_empty(), "a revoked grant came back");
    assert!(!after_a_restart.permits(&Grantee::named(HERS), &the_invoice_in(&home), noon()));
}

/// **An expired grant is gone when the list is read**, rather than read and
/// then filtered.
///
/// The list that comes back does not hold it at all — `len` is zero, not one
/// with a filter in front of it — because a promise kept by whoever remembers
/// to call the filter is a promise one forgetful caller breaks.
#[test]
fn an_expired_grant_is_gone_when_the_list_is_read() {
    let home = a_folder_of_our_own("expired");
    let at = home.join("grants.toml");
    kept(&at, &a_grant_made_in(&home), noon()).unwrap();

    let an_hour_later = remembered(&at, noon() + an_hour()).unwrap();
    assert_eq!(an_hour_later.len(), 0, "an expired grant was on the list");
    assert_eq!(an_hour_later.active_at(noon() + an_hour()).count(), 0);
    assert!(!an_hour_later.permits(
        &Grantee::named(HERS),
        &the_invoice_in(&home),
        noon() + an_hour()
    ));

    // And the same file, read a minute after it was written, still holds it:
    // what makes the grant gone is the time, not the reading.
    let a_minute_later = remembered(&at, noon() + Duration::from_secs(60)).unwrap();
    assert_eq!(a_minute_later.len(), 1);
}

/// **A file somebody else could write is refused, in words** — and nothing is
/// read out of it.
///
/// Whoever can write this file says what this machine's agent may reach, so it
/// is believed under the accounts store's rules or not at all. The sentence
/// names the file and the mode, because whoever reads it has to go and change
/// one of them.
#[test]
fn a_file_somebody_else_could_write_is_refused_in_words() {
    let home = a_folder_of_our_own("writable");
    let at = home.join("grants.toml");
    kept(&at, &a_grant_made_in(&home), noon()).unwrap();
    assert!(remembered(&at, noon()).is_ok(), "ours was not believed");

    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o666)).unwrap();
    let refused = remembered(&at, noon()).unwrap_err();

    assert!(
        matches!(refused, NotRemembered::WritableByOthers { mode: 0o666, .. }),
        "{refused}"
    );
    let said = refused.to_string();
    assert!(said.contains("grants.toml"), "{said}");
    assert!(said.contains("666"), "{said}");
    assert!(
        said.contains("what this machine's agent may reach"),
        "{said}"
    );

    // A symbolic link is the other half of the same rule: a name somebody can
    // point at a file they own.
    let link = home.join("linked.toml");
    std::os::unix::fs::symlink(&at, &link).unwrap();
    assert!(matches!(
        remembered(&link, noon()),
        Err(NotRemembered::ALink { .. })
    ));
}
