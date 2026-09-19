//! What a person is told, before and after — the acceptance of task 5 of
//! `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, one criterion at a
//! time.
//!
//! | Criterion | Test |
//! |---|---|
//! | every sentence this plan's crates can say is in the vocabulary, with a translator's note | [`every_sentence_these_crates_can_say_is_in_the_vocabulary_with_a_note`] |
//! | a walk from *an update exists* to *it is applied* to *it is rolled back* is the exact sequence a person meets | [`the_walk_from_an_update_existing_to_going_back_is_the_sequence_in_the_table`] |
//! | the walk is the table, and a sentence that changes without it fails | [`the_walk_from_an_update_existing_to_going_back_is_the_sequence_in_the_table`], [`the_table_and_the_vocabulary_say_the_same_thing`] |
//! | *this cannot be undone* names **why**, every time | [`a_refusal_to_put_something_back_always_says_why`] |
//! | no sentence names the machinery | [`no_sentence_a_person_reads_names_the_machinery`] |
//! | the refusals on the same road are said too, and each in its own words | [`every_refusal_on_the_road_is_said_and_no_two_read_alike`] |
//!
//! # Why this file is here and not in `alo-keeping-up`
//!
//! Because *before and after* needs both halves. The **before** — an update is
//! ready, this will not interrupt you, going back is offered — is
//! `alo-keeping-up`'s vocabulary. The **after** — *this machine started on an
//! updated version of its system* — is `alo-recounting`'s, read off an entry
//! `alo-record` keeps. `alo-updating` is the one crate that already has the
//! record and the decisions beside each other, which is the same reason
//! `src/putting_back.rs` lives here, and a walk that stopped at the moment the
//! person approved something would be half the promise.
//!
//! # The table is here, and the report copies it
//!
//! `THE_WALK` is the living copy: change a sentence and this fails until the
//! table changes with it. The report published with this task prints the same
//! sixteen rows verbatim, as a dated record of what a person met on the day it
//! was written. The other direction — a test that read the report and held the
//! code to it — was rejected: `docs/autonomy/SHARED_MAIN.md` says a published
//! report is never edited by anybody but its author, so binding the build to
//! one would make the next person to reword a sentence break a rule to fix a
//! test.
//!
//! # What this file does not do
//!
//! It re-decides nothing. Every sentence here is one that already existed
//! before this test was written, and the task's own constraint is that if a
//! sentence is true and reads badly the sentence changes, and if it reads well
//! and is not true it changes the other way. What is added is the proof that
//! the sequence is a sequence, that nothing in it is missing, and that every
//! refusal in it says why.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{Approvals, Given, Grant, Grantee, Grants, Proposal, Reach, Verbs};
use alo_declared::every_verb_this_machine_ships;
use alo_egress::{Destination, Indicator};
use alo_keeping_up::{
    AnUndo, Bracket, CannotGoBack, Change, Changed, Deployments, Digest, GoingBack, HowFarBack,
    NotAWindow, NotRunningABuild, NotStaged, NotUndoable, Offered, Returning, Running, Since,
    Source, Staging, Standing, THE_RULE, WhatWasDone, WhatWasKept, WhenItApplies, WhenItWasDone,
    a_check_at, words,
};
use alo_record::Entry;
use alo_recounting::{Outcome, Told};
use alo_saying::everything_this_machine_can_say;
use alo_strings::{Filling, Key, Said, Strings, Word};
use alo_updating::{NotAnswered, NotApplied, NotGoneBack, NotRead, NotRecorded};

/// One step of the walk: what has just happened, the string it is said with,
/// and the sentence a person reads.
struct Step {
    /// What has just happened, in the words the report's table uses.
    when: &'static str,
    /// The key the sentence is declared under.
    key: &'static str,
    /// The sentence, exactly as a person reads it in English.
    sentence: &'static str,
}

/// **The walk**, from an update existing to the machine being back where it
/// was: every sentence a person meets, in the order they meet it.
///
/// Sixteen, and two of them are refusals, because a person who chooses the
/// same thing twice is an ordinary person rather than an error. The last of
/// each half is read *afterwards*, out of the machine's own history, which is
/// the half of this task that the offer and the approval do not cover.
const THE_WALK: [Step; 16] = [
    Step {
        when: "They check, and the version offered is the one running",
        key: "keeping-up.up-to-date",
        sentence: "This machine is up to date",
    },
    Step {
        when: "They check again later, and a different version is offered",
        key: "keeping-up.ready",
        sentence: "An update is ready. It will apply when you choose, and nothing you are doing \
                   will be interrupted until then",
    },
    Step {
        when: "Beside it, the first of the three promises",
        key: "keeping-up.never.restarts",
        sentence: "An update never restarts this machine. It applies when you restart",
    },
    Step {
        when: "the second",
        key: "keeping-up.never.closes-an-application",
        sentence: "An update never closes an application you have open",
    },
    Step {
        when: "the third",
        key: "keeping-up.never.interrupts",
        sentence: "An update never interrupts what you are doing",
    },
    Step {
        when: "The first of the two choices they have",
        key: "keeping-up.when.at-the-next-restart",
        sentence: "Apply it the next time I restart",
    },
    Step {
        when: "the other",
        key: "keeping-up.when.now",
        sentence: "Restart now and apply it, which closes the applications that are open",
    },
    Step {
        when: "They choose the next restart, and the machine prepares it",
        key: "keeping-up.waiting-for-the-restart",
        sentence: "The update will apply the next time you restart. Your files and settings stay \
                   as they are",
    },
    Step {
        when: "They choose the same update a second time",
        key: "keeping-up.already-waiting",
        sentence: "This update is already waiting for your next restart",
    },
    Step {
        when: "They restart, and afterwards their machine's history says",
        key: "recounting.outcome.updated",
        sentence: "this machine started on an updated version of its system, after the person \
                   restarted it",
    },
    Step {
        when: "Going back to the version before is offered",
        key: "keeping-up.going-back.offered",
        sentence: "This machine can go back to the version of its system it ran before its last \
                   update. Your files and your own settings stay as they are. Accounts, passwords \
                   and settings for the whole machine that changed since that update go back to \
                   how they were",
    },
    Step {
        when: "The first of the two choices they have",
        key: "keeping-up.going-back.at-the-next-restart",
        sentence: "Go back the next time I restart",
    },
    Step {
        when: "the other",
        key: "keeping-up.going-back.now",
        sentence: "Restart now and go back, which closes the applications that are open",
    },
    Step {
        when: "They choose the next restart, and the machine sets it",
        key: "keeping-up.going-back.waiting-for-the-restart",
        sentence: "This machine will go back to the version it ran before the next time you \
                   restart. Your files stay as they are",
    },
    Step {
        when: "They ask to go back a second time",
        key: "keeping-up.going-back.already-waiting",
        sentence: "This machine will already go back to the version it ran before the next time \
                   you restart",
    },
    Step {
        when: "They restart, and afterwards their machine's history says",
        key: "recounting.outcome.rolled-back",
        sentence: "this machine went back to the version of its system it ran before, as the \
                   person asked, after they restarted it",
    },
];

/// Every word from the machinery, which no sentence on this road may say.
///
/// The same list `alo-keeping-up`'s own `words.rs` holds itself to, applied
/// here to the four clauses a person reads afterwards as well — `docs/features.md`:
/// *a person never learns the name of anything we rented*.
const THE_MACHINERY: [&str; 16] = [
    "bootc",
    "ostree",
    "deployment",
    "digest",
    "image",
    "registry",
    "container",
    "sha256",
    "reboot",
    "rollback",
    "roll back",
    "rolled back",
    "snapshot",
    "subvolume",
    "btrfs",
    "filesystem",
];

/// A moment, handed to whatever needs one: nothing in either crate reads a
/// clock.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

/// The machine's whole vocabulary, in English, which is the only place any of
/// these sentences comes from.
fn the_machine() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A whole build name, from one repeated pair.
fn build(pair: &str) -> Digest {
    Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
}

/// Where this machine's updates come from.
fn the_source() -> Source {
    Source::named("ghcr.io/aloworld-org/alo-os").unwrap()
}

/// One of the person's files, really moved by an agent: the machine's own
/// `move_file`, one grant, one approval redeemed once, and the sentence they
/// were shown.
///
/// A fixture verb of this test's own would let the two clauses read back
/// afterwards agree with a machine that does not exist.
fn a_file_an_agent_moved(strings: &Strings) -> Entry {
    let a_day = Duration::from_secs(60 * 60 * 24);
    let mut grants = Grants::default();
    for folder in ["/home/anna/Invoices", "/home/anna/Archive"] {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from(folder)),
                noon(),
                a_day,
            )
            .expect("a grant over a folder"),
        );
    }
    let verbs: Verbs = every_verb_this_machine_ships().expect("the verbs this machine ships");
    let moving = verbs
        .call(
            "move_file",
            &[
                ("file", Given::text("/home/anna/Invoices/march.pdf")),
                ("into", Given::text("/home/anna/Archive")),
            ],
        )
        .expect("the machine's own move_file takes a file and a folder");
    let mut approvals = Approvals::default();
    let id = approvals.propose(
        Proposal::checked(&moving, &Grantee::named("@files"), &grants, noon(), a_day)
            .expect("a move inside the grant"),
    );
    let approved = approvals.approve(id, noon()).expect("one approval");
    let running = approved
        .redeem(&grants, noon())
        .expect("one approval redeemed once");
    Entry::ran(&running, strings)
}

/// A build offered, heard the only way one can be: during a check that was on
/// the indicator while it happened.
fn offered(digest: Digest) -> Offered {
    let mut indicator = Indicator::default();
    let underway = indicator.beginning_on_its_own(
        a_check_at(Destination::at("updates.alo.example").unwrap()),
        noon(),
    );
    let offered = Offered::heard(&underway, digest).unwrap();
    assert!(indicator.ended_on_its_own(underway));
    offered
}

/// One word, said by this machine with nothing put into it.
fn plainly(word: Word, strings: &Strings) -> Said {
    strings.say(&word.key(), &Filling::nothing())
}

/// The walk itself, driven through the types that decide each step — never a
/// list of constants read back to itself.
fn walk(strings: &Strings) -> Vec<Said> {
    let mut met = Vec::new();
    let first = build("aa");
    let second = build("bb");
    let source = the_source();
    let running = Running::reported(first.clone());

    // Nothing to do.
    met.push(Standing::between(&running, &offered(first.clone())).said(strings));

    // A different version is offered, which is the whole of what an update is.
    let standing = Standing::between(&running, &offered(second.clone()));
    met.push(standing.said(strings));
    let Standing::Ready(ready) = standing else {
        panic!("a different version offered was not an update: {standing:?}");
    };

    // The promise it is shown beside, clause by clause.
    met.extend(THE_RULE.said(strings));

    // The two choices, and there is no third.
    for when in WhenItApplies::EVERY {
        met.push(plainly(when.word(), strings));
    }

    // They choose the next restart, and the machine prepares it.
    let before_it_waits = Deployments::reported(Some(first.clone()), None, None);
    let staging = Staging::of(
        &ready,
        &before_it_waits,
        &source,
        WhenItApplies::AtTheNextRestart,
    )
    .expect("an update on a machine running what the offer was found against");
    met.push(staging.said(strings));

    // They choose it again: one approval is one execution, and they are told
    // so rather than the same update being prepared twice.
    let waiting = Deployments::reported(Some(first.clone()), Some(second.clone()), None);
    let again = Staging::of(&ready, &waiting, &source, WhenItApplies::AtTheNextRestart)
        .expect_err("the same update prepared twice");
    assert_eq!(again, NotStaged::AlreadyWaiting);
    met.push(again.said(strings));

    // They restart. The first start on the new version is what the record is
    // told about, and this is the clause they read back afterwards.
    let after_the_update = Deployments::reported(Some(second.clone()), None, Some(first.clone()));
    let since = Since::between(Some(&first), None, &after_the_update)
        .expect("a machine whose booted version the base named");
    let Since::Updated { from, to } = &since else {
        panic!("starting on a different version was not an update: {since:?}");
    };
    let updated = Entry::updated(from.as_str(), to.as_str(), noon());
    let told = Told::of(&updated);
    assert_eq!(told.outcome(), Outcome::Updated);
    met.push(told.outcome().said(strings));

    // Going back to what they had.
    let changed = Changed::between(first.clone(), second.clone());
    let offer = GoingBack::offered(&after_the_update, Some(&changed))
        .expect("going back after an update that kept the version before");
    met.push(offer.said(strings));

    // Its own two choices, in going back's own words rather than an update's.
    for when in WhenItApplies::EVERY {
        met.push(plainly(GoingBack::when_word(when), strings));
    }

    // They choose the next restart, and the machine sets it.
    let returning = Returning::of(&offer, &after_the_update, WhenItApplies::AtTheNextRestart)
        .expect("going back on the machine the offer was made about");
    met.push(returning.said(strings));

    // They ask again, and are told it is already set.
    let already = GoingBack::offered(
        &after_the_update.clone().going_back_at_the_next_restart(),
        Some(&changed),
    )
    .expect_err("going back offered twice");
    assert_eq!(already, CannotGoBack::AlreadyGoingBack);
    met.push(already.said(strings));

    // They restart, and the machine is where it was.
    let after_going_back = Deployments::reported(Some(first.clone()), None, Some(second.clone()));
    let since = Since::between(Some(&second), Some(&first), &after_going_back)
        .expect("a machine whose booted version the base named");
    let Since::RolledBack { from, to } = &since else {
        panic!("starting on the version they chose was not a return: {since:?}");
    };
    let rolled_back = Entry::rolled_back(from.as_str(), to.as_str(), noon());
    let told = Told::of(&rolled_back);
    assert_eq!(told.outcome(), Outcome::RolledBack);
    met.push(told.outcome().said(strings));

    met
}

/// Every sentence these crates can say, each reached through the one public
/// road to it, with nothing said twice from a list.
///
/// This is what makes *every sentence* a measurement rather than a claim: a
/// string declared and reachable from nothing is caught by comparing what this
/// returns with what `alo-keeping-up` declares.
fn every_sentence_reached(strings: &Strings) -> Vec<Said> {
    let first = build("aa");
    let second = build("bb");
    let changed = Changed::between(first.clone(), second.clone());
    let ready = match Standing::between(&Running::reported(first.clone()), &offered(second.clone()))
    {
        Standing::Ready(ready) => ready,
        Standing::UpToDate => panic!("two different versions were not an update"),
    };
    let with_an_update_waiting = GoingBack::offered(
        &Deployments::reported(Some(second.clone()), Some(build("cc")), Some(first.clone())),
        Some(&changed),
    )
    .expect("going back with an update waiting");

    let mut said = walk(strings);

    // The answer about updates did not make sense.
    said.push(
        Digest::read("not a build")
            .expect_err("nonsense is not a build")
            .said(strings),
    );

    // Which version runs could not be read, on each of the four roads that
    // can meet it.
    said.push(NotRunningABuild.said(strings));
    said.push(NotStaged::NotRunningABuild.said(strings));
    said.push(CannotGoBack::NotRunningABuild.said(strings));
    said.push(NotRecorded::NotRead(NotRead::NotRunningABuild).said(strings));

    // The machine moved on between the update being found and being chosen.
    said.push(
        Staging::of(
            &ready,
            &Deployments::reported(Some(build("cc")), None, None),
            &the_source(),
            WhenItApplies::AtTheNextRestart,
        )
        .expect_err("an update on a machine that moved on")
        .said(strings),
    );

    // The base would not prepare it, and the machine is as it was.
    said.push(
        NotApplied::TheBaseDidNotStageIt(NotAnswered::SaidNo {
            program: "the base".to_owned(),
            code: Some(1),
            said: "no".to_owned(),
        })
        .said(strings),
    );

    // The machine could not write down that it had updated.
    said.push(
        NotRecorded::LastKnownNotKept {
            path: "/var/lib/alo/last-known-build".to_owned(),
            why: "read-only".to_owned(),
        }
        .said(strings),
    );

    // Going back, offered with an update waiting, and every way it is refused.
    said.push(with_an_update_waiting.said(strings));
    said.push(CannotGoBack::NothingBefore.said(strings));
    said.push(CannotGoBack::NoLongerKept { build: first }.said(strings));
    said.push(CannotGoBack::ChangedSinceItWasOffered.said(strings));
    said.push(
        NotGoneBack::TheBaseDidNotSetIt(NotAnswered::NotStarted {
            program: "the base".to_owned(),
            why: "not there".to_owned(),
        })
        .said(strings),
    );
    // Applying an update refused before anything ran, through the road that
    // carries a refusal rather than making one.
    said.push(NotApplied::Refused(NotStaged::AlreadyWaiting).said(strings));

    // Putting back what an agent did: the offer, and every reason there is not
    // one.
    let undo = AnUndo::offered(
        WhatWasDone::ChangedFiles(Change::Renamed),
        &WhatWasKept::EitherSideOfTheTurn(Bracket::comparing(
            vec!["/home/ana/Invoices/March.pdf".to_owned()],
            Vec::new(),
        )),
    )
    .expect("a rename this machine kept both sides of");
    said.push(undo.said(
        &WhenItWasDone::worded("Tuesday at 14:05").expect("a moment with something in it"),
        strings,
    ));
    for why in NotUndoable::EVERY {
        said.push(why.said(strings));
    }
    said.push(WhatWasKept::forgetting(strings));
    for not_a_window in NotAWindow::EVERY {
        said.push(not_a_window.said(strings));
    }
    assert!(HowFarBack::of(0, 50).is_err(), "no days is not a window");

    // Read back afterwards: an undo that happened, and one that could not.
    let a_move = a_file_an_agent_moved(strings);
    let put_back = Entry::undone(&a_move, noon()).expect("an undo of a call that ran");
    assert_eq!(Told::of(&put_back).outcome(), Outcome::PutBack);
    said.push(Told::of(&put_back).outcome().said(strings));
    let not_put_back = Entry::undo_failed(&a_move, "the folder is gone", noon())
        .expect("a failed undo of a call that ran");
    assert_eq!(Told::of(&not_put_back).outcome(), Outcome::NotPutBack);
    said.push(Told::of(&not_put_back).outcome().said(strings));

    said
}

/// **Every sentence these crates can say is in the machine's one vocabulary,
/// and every one of them carries the note a translator cannot work without.**
///
/// Both directions. Nothing reachable is undeclared — a key nothing declares
/// answers `Said::is_a_bug`, and a person would be shown the key — and nothing
/// declared is unreachable, which is checked by comparing what the walk above
/// reaches with what `alo-keeping-up` declares.
#[test]
fn every_sentence_these_crates_can_say_is_in_the_vocabulary_with_a_note() {
    let vocabulary = everything_this_machine_can_say().unwrap();
    let strings = Strings::of(vocabulary.clone());

    for word in words::EVERY_WORD {
        let phrase = vocabulary
            .phrase(&word.key())
            .unwrap_or_else(|| panic!("{} is not in the machine's vocabulary", word.named()));
        assert_eq!(
            phrase.source().as_written(),
            word.says(),
            "{}",
            word.named()
        );
        assert!(
            phrase.note().is_some_and(|note| note.len() > 30),
            "{} has no note a translator could work from",
            word.named()
        );
    }

    // The four clauses read afterwards belong to this plan too, and are held
    // to the same two rules.
    for outcome in [
        Outcome::Updated,
        Outcome::RolledBack,
        Outcome::PutBack,
        Outcome::NotPutBack,
    ] {
        let word = outcome.word();
        let phrase = vocabulary
            .phrase(&word.key())
            .unwrap_or_else(|| panic!("{} is not in the machine's vocabulary", word.named()));
        assert!(phrase.note().is_some(), "{}", word.named());
    }

    // Nothing reachable is a bug, and nothing declared is unreachable.
    let mut reached: BTreeSet<String> = BTreeSet::new();
    for said in every_sentence_reached(&strings) {
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said} has a gap left in it");
        reached.insert(said.into_text());
    }
    for word in words::EVERY_WORD {
        if word.named() == words::THE_ONE_WITH_GAPS.named() {
            // The one sentence with anything to fill in reads as the names and
            // the moment put into it, so its English is not what is reached.
            continue;
        }
        assert!(
            reached.contains(word.says()),
            "nothing a person can do reaches {}",
            word.named()
        );
    }
}

/// **The walk from an update existing, through it being applied, to it being
/// rolled back is exactly the sequence in the table** — and a sentence that
/// changes without the table changing fails here.
#[test]
fn the_walk_from_an_update_existing_to_going_back_is_the_sequence_in_the_table() {
    let strings = the_machine();
    let met = walk(&strings);
    assert_eq!(
        met.len(),
        THE_WALK.len(),
        "the walk is {} steps and the table has {}",
        met.len(),
        THE_WALK.len()
    );
    for (step, said) in THE_WALK.iter().zip(met.iter()) {
        assert!(!said.is_a_bug(), "{}: {said}", step.when);
        assert_eq!(said.text(), step.sentence, "{}", step.when);
    }

    // No two steps read alike: a person who met the same sentence twice could
    // not tell which of two moments they were in.
    let distinct: BTreeSet<&str> = THE_WALK.iter().map(|step| step.sentence).collect();
    assert_eq!(distinct.len(), THE_WALK.len(), "two steps read the same");
}

/// **The key beside each sentence in the table is the key that says it**, so
/// the table is something a translator can look a line up by rather than a
/// transcript.
#[test]
fn the_table_and_the_vocabulary_say_the_same_thing() {
    let vocabulary = everything_this_machine_can_say().unwrap();
    for step in &THE_WALK {
        let key = Key::named(step.key).unwrap_or_else(|_| panic!("{} is not a key", step.key));
        let phrase = vocabulary
            .phrase(&key)
            .unwrap_or_else(|| panic!("{} is in no crate's list", step.key));
        assert_eq!(phrase.source().as_written(), step.sentence, "{}", step.key);
        assert!(phrase.note().is_some(), "{} has no note", step.key);
    }
}

/// **A refusal to put something back always says why**, and each of the
/// fourteen says a different why.
///
/// The three the plan names by hand are checked by name — it left the machine,
/// it was told to somebody, it was printed — because *it cannot be undone* with
/// no reason attached is the sentence that loses a person's trust at the worst
/// moment there is.
#[test]
fn a_refusal_to_put_something_back_always_says_why() {
    let strings = the_machine();

    let mut whys: BTreeSet<String> = BTreeSet::new();
    for why in NotUndoable::EVERY {
        let said = why.said(&strings);
        assert!(!said.is_a_bug(), "{why:?}: {said}");
        let text = said.text();
        // A reason is a clause, not a word: every one of them says what it is
        // about as well as that it cannot be done.
        assert!(
            text.split_whitespace().count() >= 8,
            "{why:?} is too short to be a reason: {text}"
        );
        assert!(
            !text.to_lowercase().contains("error") && !text.contains("cannot be undone."),
            "{why:?} refuses without saying why: {text}"
        );
        whys.insert(text.to_owned());
    }
    assert_eq!(
        whys.len(),
        NotUndoable::EVERY.len(),
        "two reasons read the same"
    );

    for (why, must_say) in [
        (NotUndoable::ItLeftThisMachine, "left your machine"),
        (NotUndoable::ItWasToldToAModel, "A model was told this"),
        (NotUndoable::ItWasPrintedOnPaper, "printed on paper"),
        (NotUndoable::ItOnlyLooked, "only looked"),
        (
            NotUndoable::NothingKeepsWhatWasThere,
            "does not keep what your files were",
        ),
        (NotUndoable::ChangedSinceItWasDone, "has changed since"),
    ] {
        let said = why.said(&strings);
        assert!(said.text().contains(must_say), "{why:?}: {said}");
    }

    // And what a verb nobody has classified answers, which is the sentence
    // every change added after today falls into.
    let next_year = WhatWasDone::of_a_verb("a_verb_from_next_year", true);
    let refused = next_year
        .could_be_put_back()
        .expect_err("a change verb nobody classified");
    assert_eq!(refused, NotUndoable::NotOneOfTheChangesPutBack);
    assert!(
        refused
            .said(&strings)
            .text()
            .contains("not one of the changes"),
        "a verb nobody classified refuses without saying why"
    );
}

/// **No sentence on this road names the machinery** — not the base's tooling,
/// not a deployment, not a build's digest, not the filesystem an undo would
/// rest on.
///
/// `alo-keeping-up` holds its own forty-one to this. What is added here is the
/// four clauses a person reads afterwards, which are another crate's and were
/// held to nothing of the kind: *the system updates; it does not "apply an
/// OSTree transaction"* is a promise about what a person reads, and a person
/// reads their own history.
#[test]
fn no_sentence_a_person_reads_names_the_machinery() {
    let mut every: Vec<Word> = words::EVERY_WORD.to_vec();
    every.extend(
        [
            Outcome::Updated,
            Outcome::RolledBack,
            Outcome::PutBack,
            Outcome::NotPutBack,
        ]
        .map(Outcome::word),
    );

    for word in every {
        let said = word.says().to_lowercase();
        for machinery in THE_MACHINERY {
            assert!(
                !said.contains(machinery),
                "{} says \"{machinery}\"",
                word.named()
            );
        }
    }

    // And the walk itself, sentence by sentence, because the table is what a
    // person actually meets.
    for step in &THE_WALK {
        let sentence = step.sentence.to_lowercase();
        for machinery in THE_MACHINERY {
            assert!(
                !sentence.contains(machinery),
                "{} says \"{machinery}\"",
                step.key
            );
        }
    }
}

/// **Every refusal on the same road is said too, and no two read alike.**
///
/// The walk is the road taken when everything works twice over. This is the
/// other half of it: which version runs could not be read, the machine moved
/// on, the base would not prepare it, there is nothing to go back to, what was
/// there is gone, the machine changed after the offer, and the history could
/// not be written. Each is a sentence rather than a state a person is left to
/// infer, and each says what happened to their machine — which in every case
/// is nothing.
#[test]
fn every_refusal_on_the_road_is_said_and_no_two_read_alike() {
    let strings = the_machine();
    let refusals = [
        plainly(words::ANSWER_NOT_UNDERSTOOD, &strings),
        plainly(words::RUNNING_NOT_KNOWN, &strings),
        plainly(words::CHANGED_SINCE_IT_WAS_FOUND, &strings),
        plainly(words::ALREADY_WAITING, &strings),
        plainly(words::NOT_PREPARED, &strings),
        plainly(words::NOT_WRITTEN_DOWN, &strings),
        plainly(words::NOTHING_TO_GO_BACK_TO, &strings),
        plainly(words::NO_LONGER_KEPT, &strings),
        plainly(words::ALREADY_GOING_BACK, &strings),
        plainly(words::CHANGED_SINCE_GOING_BACK_WAS_OFFERED, &strings),
        plainly(words::GOING_BACK_NOT_PREPARED, &strings),
    ];

    let mut texts: BTreeSet<&str> = BTreeSet::new();
    for said in &refusals {
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        texts.insert(said.text());
    }
    assert_eq!(texts.len(), refusals.len(), "two refusals read the same");

    // The six that follow something the person chose say that their machine is
    // as it was. The five that are said *instead of* an offer do not, because
    // nothing was started for them to say it about.
    for word in [
        words::ANSWER_NOT_UNDERSTOOD,
        words::RUNNING_NOT_KNOWN,
        words::CHANGED_SINCE_IT_WAS_FOUND,
        words::NOT_PREPARED,
        words::GOING_BACK_NOT_PREPARED,
        words::CHANGED_SINCE_GOING_BACK_WAS_OFFERED,
    ] {
        let said = plainly(word, &strings);
        let text = said.text().to_lowercase();
        assert!(
            text.contains("nothing") && (text.contains("changed") || text.contains("has changed")),
            "{} does not say the machine is as it was: {said}",
            word.named()
        );
    }

    // And none of them tells the person to do something the machine could do
    // for them, or hedges about what happened.
    for said in &refusals {
        let text = said.text().to_lowercase();
        for hedge in [
            "probably",
            "might",
            "perhaps",
            "possibly",
            "try again later",
        ] {
            assert!(!text.contains(hedge), "{said} hedges: \"{hedge}\"");
        }
    }
}
