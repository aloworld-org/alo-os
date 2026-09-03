//! The machine, the folders and the strings this crate's tests are written
//! against.
//!
//! A turn reaches five crates, so its fixture is the arrangement a real machine
//! is in rather than a smaller one: **every crate's words in one vocabulary**,
//! because a refusal met here can have been worded by the capability model, by
//! the file half, by the record or by this crate, and a fixture holding only
//! some of them would make a missing string look like a passing test.
//!
//! The folders are real, for the reason `alo-files`' fixture gives: there is
//! one thing that acts, and abstracting it would be inventing a second answer
//! to *what happened when the machine was asked*.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{
    AnswerError, Authorised, Call, CallError, Given, Grant, Grantee, Grants, ProposalError, Reach,
    Verbs,
};
use alo_context::{Context, Document};
use alo_files::{Failed, OnThisMachine, Resolving, file_verbs, file_words};
use alo_keeping::NotKept;
use alo_strings::{Language, Strings, Translation, Word};

use crate::refusing::NotDone;

/// A fixed moment, so that expiry is arithmetic rather than a wait.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn, a grant and a question stand in these tests.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent these turns belong to.
pub(crate) fn files() -> Grantee {
    Grantee::named("@files")
}

/// A folder of this test's own, resolved.
///
/// Resolved because a grant is over a place: on Windows a resolved path carries
/// a `\\?\` prefix the typed one does not, and a grant made over the other
/// spelling would match nothing. `docs/quirks.md` records it.
pub(crate) fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-turn-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// Grants to `@files` over these folders, made at noon and lasting an hour.
pub(crate) fn granting(folders: &[&Path]) -> Grants {
    let mut grants = Grants::default();
    for folder in folders {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder((*folder).to_path_buf()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

/// A path as a verb's argument arrives: text.
pub(crate) fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// What an invocation offered, with this document open.
pub(crate) fn offering(document: &Path) -> Context {
    Context::at_invocation(noon()).and_document(Document::open(document).unwrap())
}

/// The words this machine reads, with nothing translated.
///
/// **Four crates' lists**, which is the arrangement a shell is really in: one
/// vocabulary, every crate declaring into it under its own area.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything_this_machine_says())
}

/// The same, with the given words translated into German and German preferred.
///
/// German because what is read during a turn is sentences rather than labels,
/// and German moves the verb — so a translation that came out reading like
/// English with the words swapped would not be exercising anything.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = everything_this_machine_says();
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

/// Every word a machine running a turn has loaded.
fn everything_this_machine_says() -> alo_strings::Vocabulary {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    crate::words::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// The six, as a machine offers them.
pub(crate) fn the_six() -> Verbs {
    file_verbs().unwrap()
}

/// A call over a folder, from the verb that lists one.
pub(crate) fn listing(folder: &Path) -> Call {
    the_six()
        .call("list_folder", &[("folder", as_given(folder))])
        .unwrap()
}

/// One example of every way a turn can answer with something other than what
/// was asked for.
///
/// Written out by hand, because the point of the list is that a variant added
/// without a sentence for it is caught — and a list derived from the variants
/// would be derived from the same thing it is checking.
pub(crate) fn everything_that_can_come_back() -> Vec<NotDone> {
    // A real refusal rather than one built by hand, because `Refused` is made
    // by the crate that refuses and this list is about what a person is told.
    let nothing_granted = Grants::default();
    let refused = Authorised::read(
        &listing(Path::new("/home/anna/Invoices")),
        &files(),
        &nothing_granted,
        noon(),
    )
    .err()
    .unwrap();

    vec![
        NotDone::TurnedAway(CallError::NoSuchVerb {
            name: "delete_everything".to_owned(),
        }),
        NotDone::NeverAsked(ProposalError::ReadDoesNotWait {
            verb: "list_folder".to_owned(),
        }),
        NotDone::NotAnswered(AnswerError::NothingWaiting { number: 7 }),
        NotDone::Refused(refused),
        NotDone::MachineCouldNot(Failed::Gone {
            path: "/home/anna/Invoices/march.pdf".to_owned(),
        }),
        NotDone::NotRecorded(NotKept::NotAddedTo {
            path: "/var/lib/alo/record.jsonl".to_owned(),
            why: "no space left on device".to_owned(),
        }),
        NotDone::TurnClosed,
    ]
}
