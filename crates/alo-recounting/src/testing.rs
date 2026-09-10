//! The record, the strings and the afternoon this crate's own tests are written
//! against.
//!
//! Every file here has the same two questions to answer — *what does this say
//! on a machine with no translations* and *what does it say when somebody has
//! translated it* — and answering them from one fixture is what stops the files
//! inventing vocabularies that resemble the real one. The real one is
//! [`crate::recounting_words`], and both of these are built from it.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # The entries are real entries
//!
//! Every one of them is made the way the machine makes one: a call validated
//! against a verb, grants that really permit it or really do not, an approval
//! redeemed, a departure an indicator really showed. There is deliberately no
//! fixture here for *an entry that was never written down* — there is no way to
//! write one, and a fixture that could conjure one would be testing something
//! else.
//!
//! # It is not only this crate's words
//!
//! An account quotes the record, and the record's own sentences are worded by
//! whoever decided them: `alo-capability` for a refusal, `alo-egress` and
//! `alo-models` for where something went, and the verb's own strings for the
//! sentence a person was shown. A fixture holding only this crate's list would
//! answer all of those with a key in guillemets and the tests would still pass,
//! which is a fixture proving the tests rather than the code.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Arg, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Reach,
    Requires, Takes, Verb,
};
use alo_egress::{
    Departing, Destination, EgressPolicy, Errand, Indicator, Leaving, NotPermitted, OnItsOwn,
    Underway, Why,
};
use alo_keeping::Writing;
use alo_models::Region;
use alo_record::Entry;
use alo_strings::{Language, Strings, Translation, Vocabulary, Word};

/// A fixed moment, so that everything about time here is arithmetic.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the questions in these tests last.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent that moves files about.
pub(crate) fn files() -> Grantee {
    Grantee::named("@files")
}

/// The agent that asks questions, so a query by agent has something to
/// separate.
pub(crate) fn mail() -> Grantee {
    Grantee::named("@mail")
}

/// What the change these tests follow does.
const MOVING: Word = Word::saying(
    "testing.verb.move-file.purpose",
    "move a file into a folder",
);
/// **The sentence a person approves before that file is moved**, which is the
/// one the record keeps and the one an account reads back.
const MOVING_SENTENCE: Word =
    Word::saying("testing.verb.move-file.sentence", "move {file} into {into}");
/// What it moves.
const MOVING_FILE: Word = Word::saying("testing.verb.move-file.argument.file", "the file to move");
/// Where it moves it.
const MOVING_INTO: Word = Word::saying(
    "testing.verb.move-file.argument.into",
    "the folder it goes into",
);
/// What the read these tests follow does.
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

/// Everything the two fixture verbs can say.
///
/// A verb is declared from words and the record renders its sentence, so a
/// fixture that declared a verb without declaring its strings would write the
/// key into the record where the sentence belongs — and every test about a
/// sentence would be a test about a key.
const THE_VERBS_WORDS: [Word; 7] = [
    MOVING,
    MOVING_SENTENCE,
    MOVING_FILE,
    MOVING_INTO,
    LISTING,
    LISTING_SENTENCE,
    LISTING_FOLDER,
];

/// Everything this crate says, and everything it quotes.
fn its_own_and_what_it_quotes() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    crate::words::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    for word in THE_VERBS_WORDS {
        vocabulary.says(word.phrase().unwrap()).unwrap();
    }
    vocabulary
}

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own_and_what_it_quotes())
}

/// The same, with some of these words translated into German and German
/// preferred — German because it is the language the rest of this repository
/// tests translation with.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = its_own_and_what_it_quotes();
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
fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// A change: moving one file into one folder, so two things must be granted.
fn move_file() -> Verb {
    Verb::checked(
        "move_file",
        MOVING,
        Effect::Change,
        vec![
            Arg::taking("file", MOVING_FILE, Takes::Path),
            Arg::taking("into", MOVING_INTO, Takes::Path),
        ],
        Requires::grants_over(["file", "into"]),
        MOVING_SENTENCE,
    )
    .unwrap()
}

/// A read: it answers inside the turn and is never proposed.
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

/// The change every test follows: March's invoice into the archive.
fn archiving_march() -> Call {
    Call::of(
        &move_file(),
        &[
            ("file", Given::text("/home/anna/Invoices/march.pdf")),
            ("into", Given::text("/home/anna/Archive")),
        ],
    )
    .unwrap()
}

/// The read every test follows.
fn listing_invoices() -> Call {
    Call::of(
        &list_folder(),
        &[("folder", Given::text("/home/anna/Invoices"))],
    )
    .unwrap()
}

/// Grants to `@files` over these folders, made at noon and lasting a day.
fn granting(reaches: &[&str]) -> Grants {
    let mut grants = Grants::default();
    for reach in reaches {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from(reach)),
                noon(),
                hour() * 24,
            )
            .unwrap(),
        );
    }
    grants
}

/// Everything the archiving change needs granted.
fn granting_both() -> Grants {
    granting(&["/home/anna/Invoices", "/home/anna/Archive"])
}

/// The invoice really archived: proposed, approved once, redeemed and run.
pub(crate) fn archived() -> Entry {
    let grants = granting_both();
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(&archiving_march(), &files(), &grants, noon(), hour()).unwrap());
    let approved = approvals.approve(id, noon()).unwrap();
    let running = approved.redeem(&grants, noon()).unwrap();
    Entry::ran(&running, &in_english())
}

/// A read that ran, which nobody was asked about and which needed no approval.
pub(crate) fn ran_a_read() -> Entry {
    let grants = granting(&["/home/anna/Invoices"]);
    let read = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();
    Entry::ran(&read, &in_english())
}

/// A change the grants already refused, so nobody was ever interrupted.
pub(crate) fn never_asked() -> Entry {
    let half = granting(&["/home/anna/Invoices"]);
    let why = Proposal::checked(&archiving_march(), &files(), &half, noon(), hour())
        .unwrap_err()
        .said(&in_english())
        .into_text();
    Entry::never_asked(&archiving_march(), &files(), &why, &in_english(), noon())
}

/// A change a person was shown and said no to.
pub(crate) fn declined() -> Entry {
    let grants = granting_both();
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(&archiving_march(), &files(), &grants, noon(), hour()).unwrap());
    let declined = approvals.decline(id).unwrap();
    Entry::declined(&declined, &in_english(), noon())
}

/// A call refused at the moment it would have run, because what it reached for
/// was never granted.
pub(crate) fn refused_at_the_moment() -> Entry {
    let refused = Authorised::read(
        &listing_invoices(),
        &files(),
        &granting(&["/home/anna/Taxes"]),
        noon(),
    )
    .unwrap_err();
    Entry::refused(&refused, &files(), &in_english(), noon())
}

/// Something the model asked for that alo OS does not offer, quoted as it
/// arrived.
pub(crate) fn turned_away() -> Entry {
    Entry::turned_away(
        "tidy_everything",
        "there is no verb called tidy_everything",
        &files(),
        noon(),
    )
}

/// A question answered on this machine, which left nothing behind but that it
/// was.
pub(crate) fn answered_here() -> Entry {
    Entry::answered_here(&mail(), noon())
}

/// A question a rule refused before it was put anywhere.
pub(crate) fn never_put_anywhere() -> Entry {
    Entry::never_put_anywhere(
        &mail(),
        "this machine is set to answer questions here and nowhere else",
        noon(),
    )
}

/// A provider that has said where it runs.
fn to_alo() -> Destination {
    Destination::provider("alo", Region::Declared("the EU".to_owned())).unwrap()
}

/// A departure the policy permitted, already on an indicator.
///
/// The indicator is local to the fixture on purpose: a [`Departing`] is the
/// only thing that means *this may leave*, and a test that could conjure one
/// without an indicator having shown it would be testing something else.
fn departing(leaving: Leaving, at: SystemTime) -> Departing {
    Indicator::default()
        .beginning(&EgressPolicy::Anywhere, leaving, at)
        .unwrap()
}

/// An egress this policy refused.
fn not_permitted(policy: &EgressPolicy, leaving: Leaving) -> NotPermitted {
    Indicator::default()
        .beginning(policy, leaving, noon())
        .unwrap_err()
}

/// A question `@mail` really put to a model somewhere else.
pub(crate) fn left() -> Entry {
    Entry::left(&departing(
        Leaving::because(&mail(), Why::Asking, to_alo()),
        noon(),
    ))
}

/// The same question, on a machine set to let nothing leave.
pub(crate) fn held_back() -> Entry {
    Entry::held_back(
        &not_permitted(
            &EgressPolicy::NothingLeaves,
            Leaving::because(&mail(), Why::Asking, to_alo()),
        ),
        &in_english(),
        noon(),
    )
}

/// An errand of alo OS's own: fetching a model, with nobody having asked.
pub(crate) fn fetched_a_model() -> Entry {
    let underway: Underway = Indicator::default().beginning_on_its_own(
        OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::at("models.alo.example").unwrap(),
        ),
        noon() + hour(),
    );
    Entry::left_on_its_own(&underway)
}

/// One of every kind of entry there is, in the order it happened.
///
/// Ten entries and ten outcomes: an afternoon missing one of them would let a
/// kind of entry go unread without any test noticing.
pub(crate) fn an_afternoon() -> Vec<Entry> {
    vec![
        archived(),
        ran_a_read(),
        never_asked(),
        declined(),
        refused_at_the_moment(),
        turned_away(),
        answered_here(),
        never_put_anywhere(),
        left(),
        held_back(),
        fetched_a_model(),
    ]
}

/// A file of this test's own, in the machine's temporary place.
///
/// A record is read off a disk, so the tests that are about reading one are
/// about a real file: a fixture that answered from memory would be testing the
/// thing this crate exists not to do.
pub(crate) fn somewhere_of_our_own(what: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-recounting-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder.join("record.jsonl")
}

/// A record on a real disk, holding these entries.
///
/// Left as a file this machine will believe: the account is read through
/// `alo_keeping::Reading::believed_at`, which refuses a record somebody else
/// could have written, and a fixture whose mode depended on the umask of
/// whoever ran the tests would pass or fail for a reason that has nothing to do
/// with the code.
pub(crate) fn a_record_at(path: &Path, entries: &[Entry]) {
    let mut writing = Writing::opening(path).unwrap();
    for entry in entries {
        writing.keep(entry).unwrap();
    }
    drop(writing);
    ours_alone(path);
}

/// A file left readable and writable by nobody but its owner.
#[cfg(unix)]
pub(crate) fn ours_alone(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

/// The same, where the machine has no such question to be asked.
#[cfg(not(unix))]
pub(crate) fn ours_alone(_path: &Path) {}

/// What a machine says about itself, as far as this crate reads it: a
/// description on a real disk, in this shape, keeping its record there.
///
/// Only the two keys `where_it_is.rs` is answerable for are varied. The rest is
/// written as `docs/contracts/machine-description.md` has it, so that a fixture
/// this reader accepts is one the daemon would recognise.
pub(crate) fn a_description_at(at: &Path, format: u32, record: &str) -> PathBuf {
    std::fs::write(
        at,
        format!(
            "format = {format}\n\n\
             [logins]\nperson = 1000\nagent = 1001\ngroup = 1002\n\n\
             [agent]\nname = \"@files\"\nturn-seconds = 900\nproposal-seconds = 120\n\n\
             [record]\npath = \"{record}\"\nkeeping = \"forever\"\n"
        ),
    )
    .unwrap();
    at.to_path_buf()
}

/// An afternoon long enough to be bounded: this many things that happened, a
/// minute apart, so that *the most recent* is a moment rather than a position.
pub(crate) fn a_long_afternoon(how_many: usize) -> Vec<Entry> {
    (0..how_many)
        .map(|which| {
            Entry::turned_away(
                "tidy_everything",
                "there is no verb called tidy_everything",
                &files(),
                noon() + Duration::from_secs(60 * which as u64),
            )
        })
        .collect()
}
