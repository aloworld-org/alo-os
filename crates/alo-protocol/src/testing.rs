//! The strings this crate's tests are written against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops two files inventing two vocabularies that resemble the real one.
//! The real one is [`crate::protocol_words`], and both of these are built from
//! it.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{Approvals, Given, Grant, Grantee, Grants, Proposal, Reach, Waiting};
use alo_strings::{Language, Strings, Translation, Word};

use crate::words::protocol_words;

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(protocol_words().unwrap())
}

/// The same, with these words translated into German and German preferred.
///
/// German because what this crate says is sentences rather than labels and
/// German moves the verb — so a translation that came out reading like English
/// with the words swapped would not be exercising anything.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = protocol_words().unwrap();
    let mut german = Translation::into_language(german_language());
    for (word, says) in words {
        german = german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german_language()]);
    strings
}

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// A fixed moment, so that a question standing for five minutes is arithmetic
/// rather than a wait.
pub(crate) fn the_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// An hour later, by which time a question that stood for five minutes has
/// stopped standing.
pub(crate) fn an_hour_in() -> SystemTime {
    the_moment() + Duration::from_secs(60 * 60)
}

/// One real change waiting for a person, and the words it was proposed with.
///
/// A real one: the verb is `alo-files`' `rename_file`, the arguments went
/// through `alo_capability::Verbs::call`, and the grants were asked. Nothing
/// here touches a disk — reach is lexical (ADR 0001 §4), which is what lets a
/// proposal be built in a unit test at all.
///
/// The list is handed back with it because `alo_capability::ProposalId` has no
/// public constructor: a change waiting is something an `Approvals` made, and
/// there is no way to compose one, which is the guarantee that makes a number
/// off the wire have to be *found*.
pub(crate) fn a_change_waiting() -> (Approvals, Strings) {
    let folder = PathBuf::from("/home/anna/Invoices");
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(folder.clone()),
            the_moment(),
            Duration::from_secs(60 * 60),
        )
        .unwrap(),
    );

    let verbs = alo_files::file_verbs().unwrap();
    let call = verbs
        .call(
            "rename_file",
            &[
                (
                    "file",
                    Given::text(folder.join("march.pdf").display().to_string()),
                ),
                ("name", Given::text("march-final.pdf")),
            ],
        )
        .unwrap();
    let proposal = Proposal::checked(
        &call,
        &Grantee::named("@files"),
        &grants,
        the_moment(),
        Duration::from_secs(300),
    )
    .unwrap();

    let mut approvals = Approvals::default();
    approvals.propose(proposal);
    (approvals, everything_this_machine_says())
}

/// The one change [`a_change_waiting`] made, whether or not it still stands.
pub(crate) fn the_change(approvals: &Approvals) -> &Waiting {
    approvals
        .waiting_at(the_moment())
        .next()
        .expect("the change this fixture proposed")
}

/// Every word a machine answering these messages has loaded.
///
/// This crate's words and the words of everything it renders a sentence from:
/// a fixture holding only this crate's list would make every sentence that
/// came out of a turn look like a bug in alo OS.
pub(crate) fn everything_this_machine_says() -> Strings {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    crate::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}
