//! **A change without the OK at full access, and nowhere else** (ADR 0149) —
//! the yes, and every refusal around it.

use super::*;
use crate::grant::Grantee;
use crate::test_calls::{
    archiving_march, files, granting, granting_both, hour, listing_invoices, noon,
};

/// **At full access a change is authorised with no approval**, and says so: no
/// approval, carried out without the OK, allowed by full access where no grant
/// covered it.
#[test]
fn at_full_access_a_change_is_authorised_with_no_approval() {
    let at = Grants::default().at_full_access_for(&files());
    let authorised = Authorised::at_full_access(&archiving_march(), &files(), &at, noon()).unwrap();
    assert_eq!(authorised.call(), &archiving_march());
    assert_eq!(authorised.from_approval(), None);
    assert!(authorised.without_the_ok());
    assert!(authorised.by_full_access());
    assert!(authorised.against().is_empty());
    assert_eq!(authorised.under(), &files());
    assert_eq!(authorised.at(), noon());

    // A grant that covers it is still named, as at every other door.
    let at = granting_both().at_full_access_for(&files());
    let authorised = Authorised::at_full_access(&archiving_march(), &files(), &at, noon()).unwrap();
    assert!(authorised.without_the_ok());
    assert!(!authorised.by_full_access());
    assert_eq!(authorised.against().len(), 2);
}

/// **On a list without full access a change waits**, granted or not — the read
/// door's own refusal, carrying the call for the record. This is *ask me each
/// time*, the sealed box, nothing chosen and a file that would not read, which
/// the service all hands over as the list unchanged.
#[test]
fn without_full_access_a_change_waits_whatever_is_granted() {
    for grants in [
        Grants::default(),
        granting(&["/home/anna/Invoices"]),
        granting_both(),
    ] {
        let refused =
            Authorised::at_full_access(&archiving_march(), &files(), &grants, noon()).unwrap_err();
        assert_eq!(
            refused.why(),
            &NotAuthorised::ChangeWaits {
                verb: "move_file".to_owned()
            }
        );
        assert_eq!(refused.call(), &archiving_march());
    }
}

/// **Full access for one agent is not full access for another**: a list made
/// for `@files` carries nothing out for a paired machine's principal, and an
/// application's grantee is never given it.
#[test]
fn full_access_for_one_agent_carries_nothing_out_for_another() {
    let at = granting_both().at_full_access_for(&files());
    let someone_else = Grantee::named("@studio");
    assert!(matches!(
        Authorised::at_full_access(&archiving_march(), &someone_else, &at, noon())
            .unwrap_err()
            .why(),
        NotAuthorised::ChangeWaits { .. }
    ));
}

/// **A read offered here is a read**: authorised as the read door authorises
/// it, with nothing marked as carried out without the OK.
#[test]
fn a_read_offered_here_is_only_a_read() {
    let at = Grants::default().at_full_access_for(&files());
    let authorised =
        Authorised::at_full_access(&listing_invoices(), &files(), &at, noon()).unwrap();
    assert!(!authorised.without_the_ok());
    assert_eq!(authorised.from_approval(), None);
    // And with no full access and no grant, refused as a read is.
    let refused =
        Authorised::at_full_access(&listing_invoices(), &files(), &Grants::default(), noon())
            .unwrap_err();
    assert!(matches!(refused.why(), NotAuthorised::NotGranted(_)));
}

/// **An approval's authority is never marked as without the OK** — the two
/// roads stay apart in the record.
#[test]
fn a_change_that_came_through_an_approval_is_not_without_the_ok() {
    let grants = granting_both();
    let mut approvals = crate::approvals::Approvals::default();
    let id = approvals.propose(
        crate::proposal::Proposal::checked(&archiving_march(), &files(), &grants, noon(), hour())
            .unwrap(),
    );
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    assert_eq!(authorised.from_approval(), Some(id));
    assert!(!authorised.without_the_ok());
}
