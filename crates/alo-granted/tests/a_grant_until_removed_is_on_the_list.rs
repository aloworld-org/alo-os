//! A grant that runs until the person takes it away is **on the list**, and
//! says so.
//!
//! This is the one test that matters about the length the owner added to
//! ADR 0001 on 2026-10-25. Every other length protects a person by running
//! out; this one does not, so the whole of its protection is that they can
//! find it and take it away. A row is how they find it.
//!
//! It is a separate file from `the_grants_a_person_can_see` because it is a
//! separate claim: that file is about a list of grants that end, including an
//! expired one never shown as live. The risk here is the opposite one — a
//! live grant **not** shown at all — and the two are told apart by nothing
//! except `alo_capability::Until`, which is exactly the distinction worth a
//! file of its own.
//!
//! # Why this could have been missed
//!
//! `Grant::expires_in` answers `None` both for a grant that has run out and
//! for one that never will, and `Seen::of` dropped a row whenever it was
//! `None`. So the grant was in force, the daemon honoured it, and there was
//! no row to revoke it by: a grant with no end and no way back. Nothing in
//! either crate's tests could see it, because both crates were right about
//! the thing they tested.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{ATerminalForTheAgent, Ask, Grant, Grantee, Grants, Reach};
use alo_granted::{Lasts, Listing, Revoked};

/// A fixed moment, so everything about time here is arithmetic.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The agent every grant here is for.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// The folder every grant here is over.
fn invoices() -> Reach {
    Reach::Folder(PathBuf::from("/home/anna/Invoices"))
}

/// A machine holding one grant that lasts until the person removes it.
fn a_machine_with_an_endless_grant() -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked_until_removed(&files(), invoices(), noon(), ATerminalForTheAgent::NotAtAll)
            .unwrap(),
    );
    grants
}

/// **The row exists**, now and a year later — the whole protection this
/// length has.
#[test]
fn a_grant_until_removed_has_a_row_however_long_it_has_been() {
    let grants = a_machine_with_an_endless_grant();
    for later in [
        Duration::ZERO,
        Duration::from_secs(60 * 60),
        Duration::from_secs(365 * 24 * 60 * 60),
    ] {
        let listing = Listing::of(&grants, noon() + later);
        assert!(
            !listing.is_nothing_granted(),
            "nothing granted, {later:?} on"
        );
        let row = listing
            .rows()
            .first()
            .unwrap_or_else(|| panic!("no row, {later:?} on"));
        assert_eq!(row.to(), "@files");
        assert_eq!(row.lasts(), Lasts::UntilRemoved);
    }
}

/// **A control at the other end**: a grant that ends is still dropped once it
/// has. Without this the test above passes on a `Seen::of` that lists
/// everything, which is a different bug wearing the same green tick.
#[test]
fn a_grant_that_ended_is_still_not_on_the_list() {
    let mut grants = Grants::default();
    let hour = Duration::from_secs(60 * 60);
    grants.grant(Grant::checked("@files", invoices(), noon(), hour).unwrap());

    let live = Listing::of(&grants, noon());
    assert_eq!(live.rows().first().unwrap().lasts(), Lasts::For(hour));
    assert!(Listing::of(&grants, noon() + hour).is_nothing_granted());
}

/// **And the row takes it away**, which is the point of its being there: the
/// machine's own revocation, so the daemon's next question is refused.
#[test]
fn the_row_is_how_a_person_takes_an_endless_grant_back() {
    let mut grants = a_machine_with_an_endless_grant();
    let ask = Ask::path("/home/anna/Invoices/march.pdf");
    assert!(grants.permits(&files(), &ask, noon()));

    let row = Listing::of(&grants, noon()).rows().first().unwrap().clone();
    assert_eq!(row.revoke(&mut grants), Revoked::Now);
    assert!(!grants.permits(&files(), &ask, noon()));
    assert!(Listing::of(&grants, noon()).is_nothing_granted());
}
