//! The plan's acceptance for *a person can be told what their machine did*,
//! one test per criterion.
//!
//! `docs/autonomy/v0-01-lane-b-plan.md`, task 5: *an account of what happened on
//! this machine is read back off the real record file and answered as
//! `alo_recounting::Told` values, oldest first and bounded, with
//! `alo_keeping`'s* this record does not go all the way back *carried into the
//! account rather than dropped; a record file that cannot be believed is refused
//! in words rather than answered as an empty day; a question about a span
//! answers only that span; and nothing an agent can send over the socket reaches
//! any of it.*
//!
//! What is new here is the first half of every one of those sentences: **the
//! record file**. `afterwards_ask_what_it_did.rs` asks a `Recounting` that a
//! test handed a path to, and on a running machine no such caller existed —
//! the record lives at the path `docs/contracts/machine-description.md` names,
//! and nothing on the person's side had ever read that. So these tests start
//! from a description on a disk, exactly as a shell does, and every account
//! below is read off the file that description names.
//!
//! The record is written by `alo_keeping::Writing`, which is what the daemon
//! writes with, and the entries are made by `alo-record`'s own constructors, so
//! nothing here is a shape of file this machine could not really produce. The
//! strings come from `alo_saying::everything_this_machine_can_say` — the
//! vocabulary a shell really holds — so a sentence this crate declares and
//! nobody collects fails here rather than on somebody's screen.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::Grantee;
use alo_keeping::Writing;
// The refusals a record's own file can produce are asked about on Unix only —
// the questions are a file's owner and its mode — so on any other host this name
// is unused and `-D warnings` is right to say so.
#[cfg(unix)]
use alo_keeping::NotKept;
use alo_record::{Asking, Entry};
use alo_recounting::{AtMost, NotRecounted, NotSaid, Outcome, Recounting, Told};
use alo_saying::everything_this_machine_can_say;
use alo_strings::{Said, Strings};

/// The moment this machine's afternoon starts at.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A minute, as these tests space an afternoon out by.
fn minute() -> Duration {
    Duration::from_secs(60)
}

/// The words this machine reads: every crate's, in one vocabulary, which is the
/// arrangement a shell is really in.
fn what_this_machine_says() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A folder of this test's own.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-recounting-told-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// A file left readable and writable by nobody but its owner, so that what a
/// test measures is the code rather than the umask it was run under.
#[cfg(unix)]
fn ours_alone(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}

/// The same, on a machine with no such question to be asked.
#[cfg(not(unix))]
fn ours_alone(_path: &Path) {}

/// A machine stood up: a description that says where the record is, and the
/// record itself, with this much of an afternoon written into it.
struct AMachine {
    /// What the machine says about itself.
    description: PathBuf,
    /// Where that description says the record is.
    record: PathBuf,
}

impl AMachine {
    /// A machine whose record holds this many things the agent was turned away
    /// for, one a minute from noon.
    fn with_an_afternoon_of(what: &str, how_many: u64) -> Self {
        let machine = Self::with_a_record(what);
        let mut writing = Writing::opening(&machine.record).unwrap();
        for which in 0..how_many {
            writing
                .keep(&Entry::turned_away(
                    "tidy_everything",
                    "there is no verb called tidy_everything",
                    &Grantee::named("@files"),
                    noon() + minute() * u32::try_from(which).unwrap(),
                ))
                .unwrap();
        }
        drop(writing);
        ours_alone(&machine.record);
        machine
    }

    /// A machine with a record that has been opened and nothing written to it.
    fn with_a_record(what: &str) -> Self {
        let machine = Self::described(what, None);
        drop(Writing::opening(&machine.record).unwrap());
        ours_alone(&machine.record);
        machine
    }

    /// A machine that says where it keeps its record, and no record there yet.
    ///
    /// The description is written the way
    /// `docs/contracts/machine-description.md` has it, so that a machine these
    /// tests stand up is one `alo-agentd` would recognise.
    fn described(what: &str, saying: Option<&str>) -> Self {
        let folder = a_folder_of_our_own(what);
        let record = folder.join("record.jsonl");
        let description = folder.join("agentd.toml");
        let written = saying.map_or_else(
            || {
                format!(
                    "format = 2\n\n\
                     [logins]\nperson = 1000\nagent = 1001\ngroup = 1002\n\n\
                     [agent]\nname = \"@files\"\nturn-seconds = 900\nproposal-seconds = 120\n\n\
                     [record]\npath = \"{}\"\nkeeping = \"forever\"\n",
                    record.display()
                )
            },
            str::to_owned,
        );
        fs::write(&description, written).unwrap();
        Self {
            description,
            record,
        }
    }

    /// The record this machine says it keeps, as a person's side of it finds
    /// one.
    fn asked(&self) -> Recounting {
        Recounting::described_at(&self.description).unwrap()
    }
}

/// What a sentence reads as, for the tests that are about the words in it.
fn said(said: &[Said]) -> String {
    said.iter()
        .map(|one| one.text().to_owned())
        .collect::<Vec<_>>()
        .join(" / ")
}

/// **An account of this machine's own afternoon, read off the record the
/// machine says it keeps.**
///
/// The first acceptance, whole: the path comes from the description rather than
/// from the test, the answer is `Told` values in the order they happened, and
/// it is bounded — with how many answered altogether beside it, so a bound is
/// never mistaken for a machine that did nothing else.
#[test]
fn what_this_machine_did_is_read_off_the_record_it_says_it_keeps() {
    let machine = AMachine::with_an_afternoon_of("an-afternoon", 40);
    let recounting = machine.asked();
    assert_eq!(
        recounting.where_it_is(),
        machine.record.as_path(),
        "the account was read from somewhere other than where this machine says its record is"
    );

    let account = recounting
        .about(&Asking::anything(), AtMost::entries(10).unwrap())
        .unwrap();

    // Bounded, and what is kept is the end of the day rather than the start.
    assert_eq!(account.how_many(), 10);
    assert_eq!(account.how_many_answered(), 40);
    assert_eq!(account.how_many_in_the_record(), 40);
    assert!(!account.is_all_that_answered());
    assert_eq!(
        account.told().last().map(Told::at),
        Some(noon() + minute() * 39)
    );

    // Oldest first, which is the order they happened in.
    let moments: Vec<SystemTime> = account.told().iter().map(Told::at).collect();
    let mut sorted = moments.clone();
    sorted.sort_unstable();
    assert_eq!(moments, sorted);
    assert_eq!(
        account.told().first().map(Told::at),
        Some(noon() + minute() * 30)
    );

    // Every line is an entry this machine really wrote, read back as one.
    for told in account.told() {
        assert_eq!(told.outcome(), Outcome::NeverBecameACall);
        assert!(
            told.asked_for()
                .is_some_and(|asked| asked.is("tidy_everything"))
        );
    }

    // And the person is told that this is not all of it, in words a shell
    // really holds — beside the record's own account of whether it goes all the
    // way back.
    let strings = what_this_machine_says();
    let sentences = account.said(&strings);
    assert!(
        sentences.iter().all(|one| !one.is_a_bug()),
        "{}",
        said(&sentences)
    );
    assert!(
        sentences
            .iter()
            .any(|one| one.text().contains("most recent part")),
        "{}",
        said(&sentences)
    );
    assert!(
        sentences
            .iter()
            .any(|one| one.text().contains("nothing has been removed")),
        "{}",
        said(&sentences)
    );
    assert!(account.goes_all_the_way_back());
}

/// **A record that does not go all the way back says so in the account.**
///
/// The other half of the first acceptance, and the reason it is named in the
/// plan: `alo-keeping` already knows the record was shortened, and an account
/// that dropped that would answer *the agent did nothing in March* for a
/// machine whose March was removed a fortnight ago.
#[test]
fn a_record_that_was_shortened_says_so_in_the_account_rather_than_being_read_as_a_quiet_day() {
    let machine = AMachine::described("shortened", None);
    fs::write(
        &machine.record,
        "{\"format\":1,\"since\":{\"secs_since_epoch\":1760000000,\
         \"nanos_since_epoch\":0},\"under\":{\"for-days\":30}}\n",
    )
    .unwrap();
    ours_alone(&machine.record);

    let account = machine
        .asked()
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap();
    assert!(account.is_empty());
    assert!(!account.goes_all_the_way_back());
    assert_eq!(account.begins(), Some(noon()));

    let sentences = account.said(&what_this_machine_says());
    assert_eq!(sentences.len(), 2, "{}", said(&sentences));
    assert!(
        said(&sentences).contains("nothing in this machine's record"),
        "{}",
        said(&sentences)
    );
    assert!(
        said(&sentences).contains("does not go all the way back"),
        "{}",
        said(&sentences)
    );
}

/// **A record this machine will not believe is refused in words, and an empty
/// day is a different answer entirely.**
///
/// The second acceptance. Three ways a record file is not this machine's own —
/// a link, a mode anybody could have written through — and each of them is a
/// sentence rather than a screen with nothing on it. The account of a machine
/// that really did nothing is shown beside them, because the whole criterion is
/// that the two must not look the same.
#[cfg(unix)]
#[test]
fn a_record_that_cannot_be_believed_is_refused_in_words_and_never_answered_as_an_empty_day() {
    use std::os::unix::fs::PermissionsExt;

    let strings = what_this_machine_says();

    // A machine that has done nothing: an account, empty, saying so.
    let quiet = AMachine::with_a_record("quiet");
    let nothing = quiet
        .asked()
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap();
    assert!(nothing.is_empty());
    let nothing_said = said(&nothing.said(&strings));
    assert!(
        nothing_said.contains("nothing in this machine's record"),
        "{nothing_said}"
    );

    // A record anybody signed in to this machine could have written.
    let writable = AMachine::with_an_afternoon_of("writable", 3);
    fs::set_permissions(&writable.record, fs::Permissions::from_mode(0o666)).unwrap();
    let refused = writable
        .asked()
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap_err();
    assert!(
        matches!(
            refused,
            NotRecounted::Record(NotKept::WritableByOthers { mode: 0o666, .. })
        ),
        "{refused:?}"
    );
    assert!(!refused.there_is_no_record());
    let refused_said = refused.said(&strings);
    assert!(!refused_said.is_a_bug(), "{refused_said}");
    assert!(
        refused_said.text().contains("could write to it"),
        "{refused_said}"
    );
    assert_ne!(
        refused_said.text(),
        nothing_said,
        "a record nobody could believe reads the same as a machine that did nothing"
    );

    // A record that is really a link to somewhere else.
    let linked = AMachine::described("linked", None);
    let elsewhere = linked.record.with_file_name("somebody-elses.jsonl");
    fs::write(&elsewhere, "{\"format\":1}\n").unwrap();
    std::os::unix::fs::symlink(&elsewhere, &linked.record).unwrap();
    let refused = linked
        .asked()
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap_err();
    assert!(
        matches!(refused, NotRecounted::Record(NotKept::ALink { .. })),
        "{refused:?}"
    );
    assert!(!refused.said(&strings).is_a_bug());

    // And a machine whose record was deleted is still not a quiet machine.
    let deleted = AMachine::with_an_afternoon_of("deleted", 2);
    fs::remove_file(&deleted.record).unwrap();
    let refused = deleted
        .asked()
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap_err();
    assert!(refused.there_is_no_record());
}

/// **A machine that cannot say where its record is refuses in words too**, and
/// says something different again — because the person is being sent to a
/// different part of their machine.
#[test]
fn a_machine_that_cannot_say_where_its_record_is_refuses_rather_than_answering() {
    let strings = what_this_machine_says();
    let nowhere = a_folder_of_our_own("no-description").join("agentd.toml");
    let refused = Recounting::described_at(&nowhere).unwrap_err();
    assert!(
        matches!(
            refused,
            NotRecounted::Nowhere(NotSaid::NoDescription { .. })
        ),
        "{refused:?}"
    );
    assert!(!refused.there_is_no_record());
    assert!(!refused.said(&strings).is_a_bug());
    assert!(
        refused.said(&strings).text().contains("does not say where"),
        "{}",
        refused.said(&strings)
    );

    // A description from an alo OS this one does not know is refused rather
    // than read by guessing, and reads as its own sentence.
    let newer = AMachine::described(
        "newer",
        Some("format = 9\n[record]\npath = \"/var/lib/alo/record\"\n"),
    );
    let refused = Recounting::described_at(&newer.description).unwrap_err();
    assert!(
        matches!(
            refused,
            NotRecounted::Nowhere(NotSaid::NotADescription { .. })
        ),
        "{refused:?}"
    );
    assert!(!refused.said(&strings).is_a_bug());
    assert!(
        refused.said(&strings).text().contains("cannot be read by"),
        "{}",
        refused.said(&strings)
    );
}

/// **A question about a span is answered about that span and nothing else.**
///
/// The third acceptance. The span is `alo-record`'s own — included at the
/// start, not included at the end — so two questions that meet do not both
/// claim the moment between them, and the answer to *what happened this
/// afternoon* is not quietly the answer to *what has ever happened*.
#[test]
fn a_question_about_a_span_is_answered_about_that_span_alone() {
    let machine = AMachine::with_an_afternoon_of("a-span", 60);
    let recounting = machine.asked();

    let first_ten = recounting
        .about(
            &Asking::anything().between(noon(), noon() + minute() * 10),
            AtMost::ONE_SITTING,
        )
        .unwrap();
    assert_eq!(first_ten.how_many(), 10);
    assert_eq!(first_ten.how_many_answered(), 10);
    assert!(first_ten.is_all_that_answered());
    assert!(
        first_ten
            .told()
            .iter()
            .all(|told| told.at() >= noon() && told.at() < noon() + minute() * 10)
    );

    // The whole record is still the whole record: a question narrows what is
    // told and never what the record holds.
    assert_eq!(first_ten.how_many_in_the_record(), 60);

    // The next span meets it and does not overlap it.
    let next_ten = recounting
        .about(
            &Asking::anything().between(noon() + minute() * 10, noon() + minute() * 20),
            AtMost::ONE_SITTING,
        )
        .unwrap();
    assert_eq!(next_ten.how_many(), 10);
    let both: Vec<SystemTime> = first_ten
        .told()
        .iter()
        .chain(next_ten.told())
        .map(Told::at)
        .collect();
    let mut once = both.clone();
    once.sort_unstable();
    once.dedup();
    assert_eq!(once.len(), both.len(), "two spans claimed one moment");

    // And a span nothing happened in answers with nothing, and says so — rather
    // than falling back to everything.
    let quiet = recounting
        .about(
            &Asking::anything().between(noon() + minute() * 100, noon() + minute() * 200),
            AtMost::ONE_SITTING,
        )
        .unwrap();
    assert!(quiet.is_empty());
    assert_eq!(quiet.how_many_answered(), 0);
    assert_eq!(quiet.how_many_in_the_record(), 60);
    assert!(
        said(&quiet.said(&what_this_machine_says())).contains("nothing in this machine's record"),
        "a span with nothing in it answered as though the question had not been asked"
    );
}

/// **Nothing an agent can send over the socket reaches any of this.**
///
/// The fourth acceptance, and it is two absences rather than a check. There is
/// no request for it: `alo-protocol` is the closed list of everything a client
/// can put on the wire, an agent's door reads three of them, and a line asking
/// for a record is not understood on either door. And there is no code path
/// for it either: the service an agent talks to does not depend on this crate
/// at all, so nothing behind either door can make an account to hand back —
/// which is the same shape as `alo-remembering`'s file being unreachable,
/// measured the same way.
#[test]
fn nothing_an_agent_can_send_over_the_socket_reaches_the_record() {
    for asked in [
        "{\"recount\":{}}",
        "{\"told\":{}}",
        "{\"record\":{}}",
        "{\"about\":{\"record\":\"/var/lib/alo/record\"}}",
    ] {
        assert!(
            alo_protocol::FromAnAgent::read(asked).is_err(),
            "an agent asking {asked} was understood"
        );
        assert!(
            alo_protocol::FromAPerson::read(asked).is_err(),
            "{asked} was understood on the person's door"
        );
    }

    // And the service has no way to make an account even if one arrived: it
    // does not depend on this crate.
    let daemon = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("alo-agentd")
        .join("Cargo.toml");
    let manifest = fs::read_to_string(&daemon).unwrap();
    assert!(
        !manifest.contains("alo-recounting"),
        "{} names this crate, so the service an agent talks to can make an account",
        daemon.display()
    );
}
