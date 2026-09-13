//! *Search your own files* is asked the way everything else is asked — and
//! is reachable without asking anybody.
//!
//! The plan's acceptance for task 5, on this crate's half: *a read verb,
//! declared in the shape `alo-files` declares its own, so no approval is
//! asked and the record's entry has no approval to name; evaluated against a
//! grant naming the directory it reads, refused outside it, and recorded as
//! having run; and the same answer reachable by a caller with no agent and
//! no grant at all.*
//!
//! The real verb, the real grants, the real resolver and the real record,
//! walked the way a daemon would walk them, over a folder this test builds
//! and indexes on the disk it is running on.
//!
//! | The acceptance | The test |
//! |---|---|
//! | a read under its grant: no approval, answered inside the turn, recorded with no approval to name | [`a_search_under_its_grant_answers_inside_the_turn_and_is_recorded_with_no_approval`] |
//! | refused outside its grant, before the index is asked, and the refusal is recorded | [`a_folder_outside_the_grant_is_refused_before_the_index_is_asked_and_the_refusal_is_recorded`] |
//! | a grant revoked after the authorisation, or expired, still stops it | [`a_grant_taken_away_or_run_out_stops_the_search`] |
//! | an index of another folder does not answer for the granted one | [`an_index_of_another_folder_does_not_answer_for_the_granted_one`] |
//! | a call that is not one never reaches the grants or the index | [`a_call_that_is_not_one_never_reaches_the_grants_or_the_index`] |
//! | the same answer, with no agent and no grant | [`a_person_with_no_agent_and_no_grant_gets_the_same_answer`] |
//! | only the search verb is this crate's to answer | [`only_the_search_verb_is_this_crates_to_answer`] |

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{
    Authorised, Call, CallError, Given, Grant, Grantee, Grants, NotAuthorised, Reach, Refused,
};
use alo_files::{OnThisMachine, Resolving, Touching};
use alo_finding::{Index, NotAnswered, NotIndexed, Query, Searched, finding_verbs, finding_words};
use alo_record::{Asking, Entry, Only, Record};
use alo_strings::Strings;

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants here last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent everything here is granted to.
fn agent() -> Grantee {
    Grantee::named("@finding")
}

/// The words this machine reads: this crate's beside the file half's and the
/// capability model's, which is the arrangement a shell has.
fn in_english() -> Strings {
    let mut vocabulary = finding_words().unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this test's own, **resolved**, so that what is granted, what
/// is asked about and what is indexed are spelled the way this machine
/// spells them — a grant is over a real place, and on Windows a resolved
/// path carries a prefix the path it was typed from does not.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-finding-verb-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// A few known files: two whose names hold *march*, one that does not.
fn a_library(root: &Path) {
    fs::create_dir_all(root.join("2026")).unwrap();
    fs::write(root.join("notes.txt"), b"Dear Anna, the contract.").unwrap();
    fs::write(root.join("march-notes.txt"), b"March, as it was.").unwrap();
    fs::write(
        root.join("2026").join("march.pdf"),
        b"%PDF-1.4\n1 0 obj << /Type /Catalog >> endobj\n%%EOF\n",
    )
    .unwrap();
}

/// A path, as a call arrives with it.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// Grants to the agent over these folders, made at noon and lasting an hour.
fn granting(folders: &[&Path]) -> Grants {
    let mut grants = Grants::default();
    for folder in folders {
        grants.grant(
            Grant::checked(
                agent().as_str(),
                Reach::Folder(folder.to_path_buf()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

/// A call of the search verb over this folder, for this part of a name.
fn searching(folder: &Path, named: &str) -> Result<Call, CallError> {
    finding_verbs().unwrap().call(
        "search_files",
        &[("folder", as_given(folder)), ("named", Given::text(named))],
    )
}

/// What a refusal says on this machine.
fn refusal(refused: &Refused) -> String {
    refused.said(&in_english()).into_text()
}

/// The paths below the folder of what a search found, in the index's order.
fn below(searched: &Searched<'_>) -> Vec<String> {
    searched
        .answer()
        .unwrap()
        .found
        .iter()
        .map(|entry| entry.below.clone())
        .collect()
}

/// **The ordinary road, all of it.** A read is authorised with no proposal
/// and no approval, the folder is made real and asked about again, the
/// index answers, and the record keeps all four of ADR 0001 §7's answers —
/// with *from which approval* answered by none, because none was needed.
#[test]
fn a_search_under_its_grant_answers_inside_the_turn_and_is_recorded_with_no_approval() {
    let strings = in_english();
    let documents = a_folder_of_our_own("granted");
    a_library(&documents);
    let index = Index::of(&documents).unwrap();
    let grants = granting(&[&documents]);

    let call = searching(&documents, "march").unwrap();
    assert!(!call.waits_for_approval(), "a search is a read");

    // The read door: no proposal, no approval, the grants asked now.
    let authorised = Authorised::read(&call, &agent(), &grants, noon()).unwrap();
    assert!(authorised.from_approval().is_none());
    assert_eq!(authorised.against().len(), 1);

    // Where the folder really leads, and whether the grants cover that.
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
    assert_eq!(touching.real("folder").unwrap().as_path(), documents);

    // The index answers, from itself.
    let searched = Searched::of(touching, &index);
    assert_eq!(below(&searched), ["march-notes.txt", "2026/march.pdf"]);
    let answer = searched.answer().unwrap();
    assert!(answer.not_searched.is_nothing());
    assert_eq!(answer.not_searched.outside, documents);

    // And what ran is written down, with no approval to name.
    let (authorised, outcome) = searched.into_parts();
    assert!(outcome.is_ok());
    let mut record = Record::default();
    record.keep(Entry::ran(&authorised, &strings));
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), None);
    assert_eq!(entry.happened().against().len(), 1);
    let what = entry.happened().what().unwrap();
    assert!(what.verb().is("search_files"));
    assert!(
        what.sentence().as_str().starts_with("search the index of "),
        "{}",
        what.sentence().as_str()
    );
    assert!(
        what.sentence()
            .as_str()
            .ends_with(" for files whose name contains march")
    );

    let _ = fs::remove_dir_all(&documents);
}

/// **A folder nobody granted is refused before the index is asked**, by the
/// grants and in their words, and the refusal is written down beside every
/// other — so *the agent tried and was stopped* is a sentence the record can
/// say about a search.
#[test]
fn a_folder_outside_the_grant_is_refused_before_the_index_is_asked_and_the_refusal_is_recorded() {
    let strings = in_english();
    let documents = a_folder_of_our_own("granted-not-this");
    let pictures = a_folder_of_our_own("not-granted");
    a_library(&pictures);
    let grants = granting(&[&documents]);

    let call = searching(&pictures, "march").unwrap();
    let refused = Authorised::read(&call, &agent(), &grants, noon()).unwrap_err();
    assert!(matches!(refused.why(), NotAuthorised::NotGranted(_)));
    assert!(
        refusal(&refused).contains("has not been granted"),
        "{}",
        refusal(&refused)
    );
    assert!(
        refusal(&refused).contains(&pictures.to_string_lossy().into_owned()),
        "{}",
        refusal(&refused)
    );
    assert_eq!(refused.call(), &call, "the record needs what was refused");

    let mut record = Record::default();
    record.keep(Entry::refused(&refused, &agent(), &strings, noon()));
    let refusals_only = Asking::anything().only(Only::Refusals);
    let stopped: Vec<_> = record.answering(&refusals_only).collect();
    assert_eq!(stopped.len(), 1);
    let entry = stopped.first().unwrap();
    assert!(entry.happened().was_stopped());
    assert!(!entry.happened().ran());
    assert!(
        entry
            .happened()
            .why_stopped()
            .unwrap()
            .as_str()
            .contains("has not been granted")
    );

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&pictures);
}

/// The grants are asked again at the door to the index, so a grant taken
/// away between the authorisation and the search still stops it — and a
/// grant that has run out authorises nothing in the first place.
#[test]
fn a_grant_taken_away_or_run_out_stops_the_search() {
    let strings = in_english();
    let documents = a_folder_of_our_own("revoked");
    a_library(&documents);
    let mut grants = granting(&[&documents]);
    let call = searching(&documents, "march").unwrap();

    let authorised = Authorised::read(&call, &agent(), &grants, noon()).unwrap();
    assert_eq!(grants.revoke_everything_for(&agent()), 1);
    let refused = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap_err();
    assert!(
        refusal(&refused).contains("has not been granted"),
        "{}",
        refusal(&refused)
    );

    let grants = granting(&[&documents]);
    let expired = Authorised::read(&call, &agent(), &grants, noon() + hour()).unwrap_err();
    assert!(
        refusal(&expired).contains("has expired"),
        "{}",
        refusal(&expired)
    );

    let _ = fs::remove_dir_all(&documents);
}

/// **An index of another folder does not answer for the granted one.** The
/// call was permitted and attempted, so the authorisation comes back and is
/// recorded as having run; what comes back beside it says the index was
/// somebody else's.
#[test]
fn an_index_of_another_folder_does_not_answer_for_the_granted_one() {
    let strings = in_english();
    let documents = a_folder_of_our_own("granted-wrong-index");
    let pictures = a_folder_of_our_own("indexed-instead");
    a_library(&documents);
    a_library(&pictures);
    let of_pictures = Index::of(&pictures).unwrap();
    let grants = granting(&[&documents]);

    let call = searching(&documents, "march").unwrap();
    let authorised = Authorised::read(&call, &agent(), &grants, noon()).unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
    let searched = Searched::of(touching, &of_pictures);
    assert!(searched.answer().is_none());
    match searched.not_answered().unwrap() {
        NotAnswered::Indexed(NotIndexed::NotTheSame { asked, indexed }) => {
            assert_eq!(asked, &documents);
            assert_eq!(indexed, &pictures);
        }
        other => panic!("{other:?}"),
    }
    let said = searched.not_answered().unwrap().said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");

    let (authorised, outcome) = searched.into_parts();
    assert!(outcome.is_err());
    let mut record = Record::default();
    record.keep(Entry::ran(&authorised, &strings));
    assert!(record.everything().next().unwrap().happened().ran());

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&pictures);
}

/// A call that does not survive the door never becomes a call, so nothing
/// asks the grants and nothing asks the index: a relative folder, an empty
/// name, a name that is a path, a name longer than any name.
#[test]
fn a_call_that_is_not_one_never_reaches_the_grants_or_the_index() {
    let documents = a_folder_of_our_own("door");
    for (folder, named) in [
        (Path::new("Documents"), "march"),
        (documents.as_path(), "  "),
        (documents.as_path(), "2026/march.pdf"),
        (documents.as_path(), ".."),
    ] {
        let err = searching(folder, named).unwrap_err();
        assert!(matches!(err, CallError::Argument(_)), "{named}: {err:?}");
    }
    let long = "m".repeat(alo_finding::A_NAME + 1);
    assert!(matches!(
        searching(&documents, &long).unwrap_err(),
        CallError::Argument(_)
    ));
    let _ = fs::remove_dir_all(&documents);
}

/// **A person searching their own files is not an agent and is not asking
/// anybody.** The index and the query are all it takes — no grantee, no
/// grants, no authorisation — and the answer is the same list the verb gave,
/// because it is the same function.
#[test]
fn a_person_with_no_agent_and_no_grant_gets_the_same_answer() {
    let strings = in_english();
    let documents = a_folder_of_our_own("by-hand");
    a_library(&documents);
    let index = Index::of(&documents).unwrap();

    // Nobody: no grants were made, and nothing asks for any.
    let by_hand = index.answer(&Query::named("march")).unwrap();
    let by_hand_below: Vec<&str> = by_hand
        .found
        .iter()
        .map(|entry| entry.below.as_str())
        .collect();
    assert_eq!(by_hand_below, ["march-notes.txt", "2026/march.pdf"]);

    // The agent, under a grant, through the verb.
    let grants = granting(&[&documents]);
    let call = searching(&documents, "march").unwrap();
    let authorised = Authorised::read(&call, &agent(), &grants, noon()).unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
    let searched = Searched::of(touching, &index);
    assert_eq!(below(&searched), by_hand_below);

    // And the shape says the same thing: the search takes an index and a
    // query, and there is no argument through which a person and an agent
    // could be told apart.
    let answer: for<'a> fn(
        &'a Index,
        &Query,
    ) -> Result<alo_finding::Answer<'a>, alo_finding::NotAsked> = Index::answer;
    let _ = answer;

    let _ = fs::remove_dir_all(&documents);
}

/// A permitted call of some other verb handed to this door is not answered
/// by it — the index searches, and anything else is somewhere else's to do.
#[test]
fn only_the_search_verb_is_this_crates_to_answer() {
    let strings = in_english();
    let documents = a_folder_of_our_own("not-ours");
    a_library(&documents);
    let index = Index::of(&documents).unwrap();
    let grants = granting(&[&documents]);

    let listing = alo_files::file_verbs()
        .unwrap()
        .call("list_folder", &[("folder", as_given(&documents))])
        .unwrap();
    let authorised = Authorised::read(&listing, &agent(), &grants, noon()).unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
    let searched = Searched::of(touching, &index);
    match searched.not_answered().unwrap() {
        NotAnswered::NotThisCrates { verb } => assert_eq!(verb, "list_folder"),
        other => panic!("{other:?}"),
    }
    assert!(
        searched
            .not_answered()
            .unwrap()
            .said(&strings)
            .text()
            .contains("list_folder")
    );

    let _ = fs::remove_dir_all(&documents);
}
