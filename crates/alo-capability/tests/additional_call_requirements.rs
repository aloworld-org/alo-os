//! A trusted adapter may add reach to check, never remove existing requirements.
//!
//! The extra application is metadata from a checked adapter, not an argument
//! invented by the model. Existing proposal, approval and record provenance
//! must therefore see the additional ask alongside every original path.

#![expect(
    clippy::unwrap_used,
    reason = "an unexpected fixture refusal is a test failure"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Arg, Ask, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal,
    ProposalError, Reach, Requires, Takes, Value, Verb,
};
use alo_strings::Word;

/// A fixed test application unrelated to Studio's implementation.
const APP: &str = "org.example.PermissionTest";

/// Fixed time makes expiry arithmetic rather than a sleep.
fn now() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// Every fixture grant and question lasts one hour.
fn hour() -> Duration {
    Duration::from_secs(3600)
}

/// The principal whose permission is checked by the public API.
fn agent() -> Grantee {
    Grantee::named("@requirements")
}

/// A valid call already requires two distinct original paths.
fn original(effect: Effect) -> Call {
    let verb = Verb::checked(
        "testing.two_paths",
        Word::saying("testing.requirements.purpose", "use a file and a folder"),
        effect,
        vec![
            Arg::taking(
                "file",
                Word::saying("testing.requirements.file", "the file"),
                Takes::Path,
            ),
            Arg::taking(
                "into",
                Word::saying("testing.requirements.into", "the folder"),
                Takes::Path,
            ),
        ],
        Requires::grants_over(["file", "into"]),
        Word::saying("testing.requirements.sentence", "use {file} and {into}"),
    )
    .unwrap();
    Call::of(
        &verb,
        &[
            ("file", Given::text("/home/test/Notes/one.txt")),
            ("into", Given::text("/home/test/Archive")),
        ],
    )
    .unwrap()
}

/// Grant precisely one reach, retaining the actual returned identity.
fn granting(grants: &mut Grants, reach: Reach) -> alo_capability::GrantId {
    grants.grant(Grant::checked("@requirements", reach, now(), hour()).unwrap())
}

/// The two existing folder grants, in the same order as the original asks.
fn folders(grants: &mut Grants) -> Vec<alo_capability::GrantId> {
    vec![
        granting(grants, Reach::Folder(PathBuf::from("/home/test/Notes"))),
        granting(grants, Reach::Folder(PathBuf::from("/home/test/Archive"))),
    ]
}

/// Additional reach cannot alter the validated call the person would approve.
#[test]
fn adding_an_application_preserves_original_paths_values_effect_and_sentence() {
    for effect in [Effect::Read, Effect::Change] {
        let before = original(effect);
        let after = before
            .clone()
            .also_requiring(Ask::Application(APP.to_owned()));
        assert_eq!(
            after.asks(),
            [
                Ask::Path(PathBuf::from("/home/test/Notes/one.txt")),
                Ask::Path(PathBuf::from("/home/test/Archive")),
                Ask::Application(APP.to_owned()),
            ]
        );
        assert_eq!(before.asks().len(), 2, "the original clone was changed");
        assert_eq!(after.verb(), "testing.two_paths");
        assert_eq!(after.effect(), effect);
        assert_eq!(after.values(), before.values());
        assert_eq!(
            after.value("file"),
            Some(&Value::Path(PathBuf::from("/home/test/Notes/one.txt")))
        );
        assert_eq!(after.sentence_key(), before.sentence_key());
        assert_eq!(after.waits_for_approval(), before.waits_for_approval());
    }
}

/// Repeating any existing ask is idempotent and does not reorder the list.
#[test]
fn duplicate_additions_are_stable_and_do_not_replace_other_requirements() {
    let call = original(Effect::Read)
        .also_requiring(Ask::Application(APP.to_owned()))
        .also_requiring(Ask::Application(APP.to_owned()))
        .also_requiring(Ask::Path(PathBuf::from("/home/test/Notes/one.txt")))
        .also_requiring(Ask::Application("org.example.Second".to_owned()));
    assert_eq!(
        call.asks(),
        [
            Ask::Path(PathBuf::from("/home/test/Notes/one.txt")),
            Ask::Path(PathBuf::from("/home/test/Archive")),
            Ask::Application(APP.to_owned()),
            Ask::Application("org.example.Second".to_owned()),
        ]
    );
}

/// Path grants alone no longer suffice, and an application grant never replaces them.
#[test]
fn every_original_and_additional_requirement_is_enforced_before_proposing() {
    let call = original(Effect::Change).also_requiring(Ask::Application(APP.to_owned()));
    let mut paths_only = Grants::default();
    let _ = folders(&mut paths_only);
    assert!(matches!(
        Proposal::checked(&call, &agent(), &paths_only, now(), hour()),
        Err(ProposalError::NotGranted(_))
    ));
    let mut application_only = Grants::default();
    granting(&mut application_only, Reach::Application(APP.to_owned()));
    assert!(matches!(
        Proposal::checked(&call, &agent(), &application_only, now(), hour()),
        Err(ProposalError::NotGranted(_))
    ));
    let read = original(Effect::Read).also_requiring(Ask::Application(APP.to_owned()));
    assert!(Authorised::read(&read, &agent(), &paths_only, now()).is_err());
    assert!(Authorised::read(&read, &agent(), &application_only, now()).is_err());
}

/// Read and approved-change authorities retain every real grant identity.
#[test]
fn read_and_approved_change_keep_original_and_application_grant_provenance() {
    let mut grants = Grants::default();
    let mut expected = folders(&mut grants);
    expected.push(granting(&mut grants, Reach::Application(APP.to_owned())));
    let read = original(Effect::Read).also_requiring(Ask::Application(APP.to_owned()));
    let authorised = Authorised::read(&read, &agent(), &grants, now()).unwrap();
    assert_eq!(authorised.against(), expected);
    assert!(!authorised.by_full_access());
    assert_eq!(authorised.from_approval(), None);
    let change = original(Effect::Change).also_requiring(Ask::Application(APP.to_owned()));
    let proposal = Proposal::checked(&change, &agent(), &grants, now(), hour()).unwrap();
    let mut approvals = Approvals::default();
    let number = approvals.propose(proposal);
    let authorised = approvals
        .approve(number, now())
        .unwrap()
        .redeem(&grants, now())
        .unwrap();
    assert_eq!(authorised.against(), expected);
    assert!(!authorised.by_full_access());
    assert_eq!(authorised.from_approval(), Some(number));
    assert!(
        approvals.approve(number, now()).is_err(),
        "one approval was reused"
    );
}

/// A retained approval does not retain an application grant that was withdrawn.
#[test]
fn application_withdrawal_is_checked_again_when_the_approval_is_redeemed() {
    let mut grants = Grants::default();
    let _ = folders(&mut grants);
    let application = granting(&mut grants, Reach::Application(APP.to_owned()));
    let call = original(Effect::Change).also_requiring(Ask::Application(APP.to_owned()));
    let proposal = Proposal::checked(&call, &agent(), &grants, now(), hour()).unwrap();
    let mut approvals = Approvals::default();
    let number = approvals.propose(proposal);
    assert!(grants.revoke(application));
    assert!(
        approvals
            .approve(number, now())
            .unwrap()
            .redeem(&grants, now())
            .is_err()
    );
    assert!(Proposal::checked(&call, &agent(), &grants, now(), hour()).is_err());
}

/// Full access is explicit authority, not a fabricated grant or approval.
#[test]
fn full_access_covers_the_added_application_and_original_paths_without_grants() {
    let stored = Grants::default();
    let full = stored.at_full_access_for(&agent());
    for effect in [Effect::Read, Effect::Change] {
        let call = original(effect).also_requiring(Ask::Application(APP.to_owned()));
        let authorised = Authorised::at_full_access(&call, &agent(), &full, now()).unwrap();
        assert!(authorised.by_full_access());
        assert!(authorised.against().is_empty());
        assert_eq!(authorised.from_approval(), None);
    }
    assert!(stored.is_empty());
}
