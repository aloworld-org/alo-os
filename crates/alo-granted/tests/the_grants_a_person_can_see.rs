//! The whole journey the plan's task 22 names: the grants the machine
//! actually keeps, read as the list a person would see; a revocation made
//! through that list stopping the daemon's own answer immediately, with the
//! verb refused afterwards; an expired grant never shown as live; *nothing
//! granted* a sentence; and every string in the vocabulary `alo-saying`
//! collects.
//!
//! The crate's own tests take each half apart. This is the other half of
//! that bargain: a listing derived from grants that went through
//! `alo_capability::Grants::remembered` — the one road back in from a disk,
//! which is how `alo-remembering` hands a machine its kept list — and a verb
//! built and refused by `alo-capability` itself rather than by a fixture
//! that only looks like one.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{
    Arg, Ask, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Reach, Requires, Takes, Verb,
};
use alo_granted::{Listing, Revoked};
use alo_strings::{Strings, Word};

/// A fixed moment, so that everything about time here is arithmetic.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants these tests read last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent whose grant every test here looks at.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// What the read these tests refuse afterwards does.
const LISTING: Word = Word::saying(
    "testing.verb.list-folder.purpose",
    "list what is in a folder",
);
/// What a person is shown while it happens.
const LISTING_SENTENCE: Word = Word::saying(
    "testing.verb.list-folder.sentence",
    "list what is in {folder}",
);
/// What it lists.
const LISTING_FOLDER: Word = Word::saying(
    "testing.verb.list-folder.argument.folder",
    "the folder to list",
);

/// A read verb, so that *the verb refused afterwards* is a verb and not a
/// stand-in: it answers inside the turn and still needs its grant.
fn list_folder() -> Verb {
    Verb::checked(
        "list_folder",
        LISTING,
        Effect::Read,
        vec![Arg::taking("folder", LISTING_FOLDER, Takes::Path)],
        Requires::grants_over(["folder"]),
        LISTING_SENTENCE,
    )
    .unwrap()
}

/// The call the daemon would answer: list the granted folder.
fn listing_invoices() -> Call {
    Call::of(
        &list_folder(),
        &[("folder", Given::text("/home/anna/Invoices"))],
    )
    .unwrap()
}

/// The machine's one vocabulary, which is the `Strings` a real shell holds.
fn what_the_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// Grants as a machine keeps them: made, written down, and read back through
/// the one road `alo-remembering` uses — so the list a person sees in these
/// tests is derived from *kept* grants, not from a value that only lived in
/// this process.
fn kept_grants_of_one_folder() -> Grants {
    let mut made = Grants::default();
    made.grant(
        Grant::checked(
            "@files",
            Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    let held = made.active_at(noon()).cloned().collect();
    Grants::remembered(held, made.next_handle().as_u64()).unwrap()
}

/// **The list a person sees is the machine's own kept grants.** A grant made,
/// written down and read back through `Grants::remembered` is one row saying
/// who, what, since when and for how much longer — worded by the machine's
/// one vocabulary — and there is no other door a row can arrive through.
#[test]
fn a_kept_list_read_back_off_a_disk_is_the_list_a_person_sees() {
    let grants = kept_grants_of_one_folder();
    let listing = Listing::of(&grants, noon());
    assert_eq!(listing.rows().len(), 1);

    let row = listing.rows().first().unwrap();
    assert_eq!(row.to(), "@files");
    assert_eq!(row.granted_at(), noon());
    assert_eq!(row.expires_in(), hour());
    assert_eq!(row.id(), grants.active_at(noon()).next().unwrap().id);

    let said = row.said(&what_the_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");
    assert_eq!(
        said.text(),
        "@files can reach /home/anna/Invoices and everything in it"
    );
}

/// **Revoking through the list is the revocation the daemon already
/// enforces.** It takes effect on the daemon's own `permits` immediately —
/// the next question, not the next sign-in — and the verb itself, built and
/// asked through `alo-capability`, is refused afterwards with *never
/// granted*, exactly as if the grant had never been made.
#[test]
fn revoking_through_the_list_stops_the_daemon_immediately_and_the_verb_is_refused() {
    let mut grants = kept_grants_of_one_folder();
    let ask = Ask::path("/home/anna/Invoices/march.pdf");

    // Before: the daemon permits, and the verb runs.
    assert!(grants.permits(&files(), &ask, noon()));
    assert!(Authorised::read(&listing_invoices(), &files(), &grants, noon()).is_ok());

    // The person revokes the row they can see.
    let row = Listing::of(&grants, noon()).rows().first().unwrap().clone();
    let revoked = row.revoke(&mut grants);
    assert_eq!(revoked, Revoked::Now);
    assert!(revoked.took_effect());

    // After, at the same moment: the daemon's own answer is no, and the verb
    // is refused.
    assert!(!grants.permits(&files(), &ask, noon()));
    let refused = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap_err();
    assert!(matches!(
        refused.why(),
        alo_capability::NotAuthorised::NotGranted(alo_capability::NotGranted::Never { .. })
    ));

    // And what the person is told about the revocation says it has already
    // stopped, in the machine's own vocabulary.
    let said = revoked.said(&what_the_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("already stopped"), "{said}");
}

/// **A revocation from a stale list lands on nothing and changes nothing.**
/// The refusal path beside the legitimate one: the grant is gone by the time
/// the row is used, the machine's grants are byte for byte as they were, and
/// the person is told the list was out of date rather than nothing at all.
#[test]
fn a_revocation_from_a_stale_list_changes_nothing_and_says_so() {
    let mut grants = kept_grants_of_one_folder();
    let stale = Listing::of(&grants, noon()).rows().first().unwrap().clone();

    // The grant goes away behind the list's back.
    assert_eq!(stale.revoke(&mut grants), Revoked::Now);

    let before = serde_json::to_string(&grants).unwrap();
    let again = stale.revoke(&mut grants);
    assert_eq!(again, Revoked::AlreadyGone);
    assert!(!again.took_effect());
    assert_eq!(serde_json::to_string(&grants).unwrap(), before);

    let said = again.said(&what_the_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Nothing was changed"), "{said}");
}

/// **An expired grant is never shown as live.** It is still on the machine's
/// stored list — nothing has swept it — the daemon refuses it, and the list a
/// person sees agrees with the daemon rather than with the storage.
#[test]
fn an_expired_grant_is_not_on_the_list_a_person_sees() {
    let grants = kept_grants_of_one_folder();
    let after = noon() + hour();

    assert_eq!(grants.len(), 1, "swept, so this proves nothing");
    assert!(
        !grants.permits(&files(), &Ask::path("/home/anna/Invoices/march.pdf"), after),
        "the daemon still permits this, so the listing is not the thing under test"
    );

    let listing = Listing::of(&grants, after);
    assert!(listing.rows().is_empty());
    assert!(listing.is_nothing_granted());
}

/// **Nothing granted is a sentence rather than an empty list**, on the
/// machine every person starts with — and the sentence comes out of the
/// machine's one vocabulary, not out of this crate's own fixture.
#[test]
fn nothing_granted_is_a_sentence_a_person_reads() {
    let listing = Listing::of(&Grants::default(), noon());
    assert!(listing.is_nothing_granted());

    let said = listing.said(&what_the_machine_can_say()).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().starts_with("Nothing is granted"), "{said}");

    // And a list with a row has no empty sentence to show above it.
    assert!(
        Listing::of(&kept_grants_of_one_folder(), noon())
            .said(&what_the_machine_can_say())
            .is_none()
    );
}

/// **Every string this crate says is in the vocabulary `alo-saying`
/// collects.** `alo-overlay` once declared nine strings nothing collected;
/// this is the test that makes that failure impossible to repeat here.
#[test]
fn everything_this_crate_says_is_collected_by_the_machine() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for word in alo_granted::words::EVERY_WORD {
        let key = word.key();
        assert!(
            vocabulary.phrase(&key).is_some(),
            "{} is not collected: the machine cannot say it",
            word.named()
        );
    }
}
