//! **A publication through a whole turn**: at full access it runs once with no
//! snapshot; at *ask me each time* nothing runs before the one OK, and the OK
//! runs it once; the sealed box, a missing grant, a rule that says nothing
//! may leave, nobody signed in and an address that cannot exist all publish
//! nothing — and every run and every refusal is written down.

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

use alo_capability::{AnswerError, Given, Grants};
use alo_egress::{EgressPolicy, Indicator};
use alo_files::OnThisMachine;
use alo_publishing::NotPublished;
use alo_record::Record;
use alo_strings::Said;

use crate::a_change_at_full_access::Changed;
use crate::machine::{Machine, WhatPublishingNeeds};
use crate::refusing::NotDone;
use crate::testing::{
    APublisher, NothingIsBounded, a_folder_of_our_own, as_given, files, hour, in_english, noon,
    offering,
};
use crate::turning::Turning;

/// A folder with a page in it.
fn a_site(what: &str) -> PathBuf {
    let folder = a_folder_of_our_own(what);
    fs::write(folder.join("index.html"), "<h1>Hello</h1>").unwrap();
    folder
}

/// What publishing `folder` to `account`'s site is asked for with.
fn publishing(folder: &Path, account: &str) -> Vec<(&'static str, Given)> {
    vec![
        ("folder", as_given(folder)),
        ("account", Given::text(account)),
    ]
}

/// A turn on a machine that publishes through `publisher` under `rule`, whose
/// record is `record`, with the grants `grants` gives back for the folder.
fn on_a_machine<T>(
    folder: &Path,
    publisher: &APublisher,
    rule: EgressPolicy,
    record: &mut Record,
    doing: impl FnOnce(&mut Turning<'_, '_>) -> T,
) -> T {
    let strings = in_english();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let answered = {
        let mut machine = Machine::carrying_out_file_and_publishing_verbs(
            &strings,
            &OnThisMachine,
            WhatPublishingNeeds {
                runs: publisher,
                through: alo_publishing::Through::on_github_com(),
                rule,
            },
            &mut bounding,
            &mut indicator,
            record,
        )
        .unwrap();
        let mut grants = Grants::default();
        let mut turning = Turning::beginning(
            offering(&folder.join("index.html")),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        doing(&mut turning)
    };
    assert!(
        indicator.is_quiet(),
        "the line on the indicator was left up after the publication"
    );
    answered
}

/// A `before` that counts how often a snapshot was asked for.
fn counting(asked: &Cell<usize>) -> impl FnMut(&Said) -> Option<String> + '_ {
    move |_| {
        asked.set(asked.get() + 1);
        Some("1793502334-812-1".to_owned())
    }
}

/// **At full access it is published at once, exactly once, with no snapshot
/// asked for** — nothing on this computer changes, and a place kept beside it
/// would put *Put back* on a thing it cannot recall — and the record says it
/// ran at full access with nothing kept before it.
#[test]
fn at_full_access_it_is_published_once_and_no_snapshot_is_taken() {
    let folder = a_site("full-access");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    let asked = Cell::new(0);
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            let grants = Grants::default().at_full_access_for(&files());
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "anna"),
                &grants,
                hour(),
                &mut counting(&asked),
                noon(),
            )
        },
    );
    let Ok(Changed::Done { did, sentence }) = &changed else {
        panic!("it was not published at full access: {changed:?}");
    };
    assert_eq!(
        did.published().map(alo_publishing::Published::address),
        Some("https://anna.github.io/")
    );
    assert!(
        sentence.text().contains("Put back can't recall it"),
        "{sentence:?}"
    );
    assert_eq!(publisher.runs(), 1, "one publication was not one run");
    assert_eq!(asked.get(), 0, "a snapshot was taken before a publication");
    let written: Vec<_> = record.everything().collect();
    assert_eq!(written.len(), 1, "{written:?}");
    let entry = written.first().unwrap();
    assert!(entry.happened().ran(), "{entry:?}");
    assert!(entry.was_at_full_access(), "{entry:?}");
    assert!(
        entry.kept_before_it().is_none(),
        "Put back is offered beside it: {entry:?}"
    );
    let ran = publisher.ran.lock().unwrap();
    assert!(
        ran.first()
            .is_some_and(|line| line.contains("anna/anna.github.io")),
        "{ran:?}"
    );
}

/// **At *ask me each time* nothing runs before the OK; one OK runs it exactly
/// once; the same number again runs nothing** — and the sentence the person
/// is asked carries the warning (ADR 0110 §2, ADR 0143 Q5). **No folder grant
/// is asked for**: the grants here are empty, and the one approval is the
/// whole permission (the owner, 2026-11-13: *"Approval alone is enough"*).
#[test]
fn asking_each_time_one_ok_is_one_publication_and_the_sentence_carries_the_warning() {
    let folder = a_site("asking");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    let asked = Cell::new(0);
    on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            let grants = Grants::default();
            let changed = turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "anna"),
                &grants,
                hour(),
                &mut counting(&asked),
                noon(),
            );
            let Ok(Changed::Waiting(number)) = &changed else {
                panic!("it did not wait for the person: {changed:?}");
            };
            let number = *number;
            assert_eq!(publisher.runs(), 0, "it ran before the OK");
            let strings = in_english();
            let sentence = turning
                .proposed(number)
                .unwrap()
                .proposal
                .sentence(&strings);
            for part in [
                "anna.github.io",
                "to the internet",
                "anybody can read it",
                "Put back can't recall it",
            ] {
                assert!(
                    sentence.text().contains(part),
                    "{part:?} is not in {sentence:?}"
                );
            }
            let did = turning.approving(number, &grants, noon()).unwrap();
            assert!(did.published().is_some(), "{did:?}");
            assert_eq!(publisher.runs(), 1);
            assert!(matches!(
                turning.approving(number, &grants, noon()),
                Err(NotDone::NotAnswered(AnswerError::NothingWaiting { .. }))
            ));
            assert_eq!(publisher.runs(), 1, "one approval ran it twice");
        },
    );
    assert_eq!(asked.get(), 0);
    assert!(
        record.everything().any(|entry| entry.happened().ran()),
        "the publication was not written down"
    );
}

/// **The refusal path: a folder that is not there is refused before anybody
/// is asked** — with no grant to stop it, the check that it is a folder with
/// something in it is what does — written down, and nothing runs.
#[test]
fn a_folder_that_is_not_there_is_refused_and_written_down() {
    let folder = a_site("not-there");
    let gone = folder.join("not-there");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&gone, "anna"),
                &Grants::default(),
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(
        matches!(changed, Err(ref why) if why.was_refused()),
        "{changed:?}"
    );
    assert_eq!(publisher.runs(), 0);
    assert_eq!(record.len(), 1, "the refusal was not written down");
}

/// **The refusal path: an address that cannot exist is refused before
/// anybody is asked**, in a sentence that says why.
#[test]
fn an_account_github_would_not_give_is_refused_before_anybody_is_asked() {
    let folder = a_site("not-an-account");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "anna smith"),
                &Grants::default(),
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    let Err(why) = &changed else {
        panic!("a nonsense address was put to the person: {changed:?}");
    };
    assert!(
        why.said(&in_english())
            .text()
            .contains("is not a name GitHub gives an account"),
        "{why:?}"
    );
    assert_eq!(publisher.runs(), 0);
    assert_eq!(record.len(), 1);
}

/// **The refusal path: nobody signed in publishes nothing, says how to sign
/// in, and is written down with the reason beside it.**
#[test]
fn nobody_signed_in_is_told_how_to_sign_in_and_written_down() {
    let folder = a_site("signed-out");
    let publisher = APublisher::answering(
        Some(72),
        "To get started with GitHub CLI, please run:  gh auth login\n",
    );
    let mut record = Record::default();
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "anna"),
                &Grants::default().at_full_access_for(&files()),
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(
        matches!(
            changed,
            Err(NotDone::CouldNotPublish(NotPublished::NotSignedIn))
        ),
        "{changed:?}"
    );
    let entry = record.everything().next().unwrap();
    assert!(
        entry
            .told()
            .iter()
            .any(|line| line.as_str().contains("gh auth login --web")),
        "the record does not say why: {entry:?}"
    );
}

/// **The refusal path: a rule that says nothing may leave publishes nothing**
/// — refused before the command runs, with nothing on the indicator after.
#[test]
fn a_rule_that_says_nothing_may_leave_publishes_nothing() {
    let folder = a_site("nothing-leaves");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::NothingLeaves,
        &mut record,
        |turning| {
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "anna"),
                &Grants::default().at_full_access_for(&files()),
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(
        matches!(changed, Err(NotDone::NotPermittedToLeave(_))),
        "{changed:?}"
    );
    assert_eq!(
        publisher.runs(),
        0,
        "it ran under a rule that says nothing leaves"
    );
    assert_eq!(record.len(), 1);
}

/// **The refusal path: a level that cannot publish refuses it before anybody
/// is asked, and a waiting one approved after the person moved to that level
/// runs nothing** — each written down, and the approval spent.
#[test]
fn a_level_that_cannot_publish_runs_nothing_now_or_at_the_ok() {
    let folder = a_site("sealed-box");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    let said = in_english().say(
        &alo_publishing::words::THE_SEALED_BOX_CANNOT_PUBLISH.key(),
        &alo_strings::Filling::nothing(),
    );
    on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            let refused = turning.a_change_this_level_cannot_make(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "anna"),
                &said,
                noon(),
            );
            assert_eq!(refused, NotDone::NotAtThisLevel(said.clone()));

            let grants = Grants::default();
            let changed = turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "anna"),
                &grants,
                hour(),
                &mut |_| None,
                noon(),
            );
            let Ok(Changed::Waiting(number)) = &changed else {
                panic!("{changed:?}");
            };
            let number = *number;
            let stopped = turning.a_waiting_change_this_level_cannot_make(number, &said, noon());
            assert!(matches!(stopped, NotDone::Refused(_)), "{stopped:?}");
            assert!(matches!(
                turning.approving(number, &grants, noon()),
                Err(NotDone::NotAnswered(_))
            ));
        },
    );
    assert_eq!(publisher.runs(), 0, "a level that cannot publish published");
    assert_eq!(record.len(), 2, "both refusals were not written down");
}

/// What a model sends when it names the folder alone, as the verb now asks.
fn the_folder_alone(folder: &Path) -> Vec<(&'static str, Given)> {
    vec![("folder", as_given(folder))]
}

/// **No account given: the one `gh` is signed in as.** Found on the walk of
/// 2026-11-13, where a model invented an account and the publication was
/// refused as somebody else's site. Asked each time, the sentence names the
/// signed-in account's address; at full access it is published there.
#[test]
fn with_no_account_given_it_goes_to_the_account_gh_is_signed_in_as() {
    let folder = a_site("no-account-given");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            let changed = turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &the_folder_alone(&folder),
                &Grants::default(),
                hour(),
                &mut |_| None,
                noon(),
            );
            let Ok(Changed::Waiting(number)) = &changed else {
                panic!("it did not wait for the person: {changed:?}");
            };
            let sentence = turning
                .proposed(*number)
                .unwrap()
                .proposal
                .sentence(&in_english());
            assert!(
                sentence.text().contains("anna.github.io"),
                "the sentence does not name the signed-in account's site: {sentence:?}"
            );
        },
    );
    let mut record = Record::default();
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &the_folder_alone(&folder),
                &Grants::default().at_full_access_for(&files()),
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    let Ok(Changed::Done { did, .. }) = &changed else {
        panic!("it was not published at full access: {changed:?}");
    };
    assert_eq!(
        did.published().map(alo_publishing::Published::address),
        Some("https://anna.github.io/")
    );
    assert_eq!(
        publisher.asked_who(),
        2,
        "gh was not asked who is signed in"
    );
    assert_eq!(publisher.runs(), 1, "one publication was not one run");
}

/// **An account given that is the signed-in one, in any case, is published
/// to** — a model that still sends it, as the verb asked until 2026-11-13, is
/// answered as before.
#[test]
fn an_account_given_that_is_the_signed_in_one_is_published_to() {
    let folder = a_site("same-account-given");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "Anna"),
                &Grants::default().at_full_access_for(&files()),
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(matches!(changed, Ok(Changed::Done { .. })), "{changed:?}");
    assert_eq!(publisher.runs(), 1);
}

/// **The refusal path: an account given that is somebody else is refused
/// before anybody is asked**, naming both, written down — and nothing runs.
#[test]
fn an_account_given_that_is_somebody_else_is_refused_before_anybody_is_asked() {
    let folder = a_site("another-account-given");
    let publisher = APublisher::that_publishes();
    let mut record = Record::default();
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &publishing(&folder, "bob"),
                &Grants::default(),
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(
        matches!(
            &changed,
            Err(NotDone::CouldNotPublish(NotPublished::SignedInAsAnother { signed_in, account }))
                if signed_in == "anna" && account == "bob"
        ),
        "{changed:?}"
    );
    assert_eq!(publisher.runs(), 0, "somebody else's site was published to");
    assert_eq!(record.len(), 1, "the refusal was not written down");
}

/// **The refusal path: nobody signed in is refused before anybody is asked,
/// as nobody signed in** — the refusal that raises the sign-in sheet — and
/// written down, with nothing run.
#[test]
fn nobody_signed_in_is_refused_before_anybody_is_asked() {
    let folder = a_site("nobody-signed-in");
    let publisher = APublisher::that_publishes().signed_in_as(None);
    let mut record = Record::default();
    let changed = on_a_machine(
        &folder,
        &publisher,
        EgressPolicy::Anywhere,
        &mut record,
        |turning| {
            turning.changing(
                alo_publishing::PUBLISH_TO_GITHUB_PAGES,
                &the_folder_alone(&folder),
                &Grants::default(),
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(
        matches!(
            changed,
            Err(NotDone::CouldNotPublish(NotPublished::NotSignedIn))
        ),
        "{changed:?}"
    );
    assert_eq!(publisher.runs(), 0);
    assert_eq!(record.len(), 1, "the refusal was not written down");
}
