//! The plan's acceptance for *the account a person asks for is the one their
//! machine kept*, one test per criterion.
//!
//! `docs/autonomy/v0-01-lane-b-plan.md`, task 6: *a record whose entries and
//! whose beginning disagree — an entry older than the moment the head says the
//! record starts at, or moments that run backwards — is reported as* this
//! record is not what it says it is*, in words, alongside everything that could
//! be read, as an unreadable line already is and never as a refusal of the
//! whole file; the disagreement is carried into the account rather than being a
//! flag on a struct a surface may forget to draw; a record that was
//! legitimately shortened is **not** reported, which is the case this is
//! easiest to get wrong; and the daemon's own writing and shortening still
//! produce a record this check is silent about, measured by writing one with
//! `alo_keeping::Writing` and reading it back.*
//!
//! Every rule task 5 put on the record file is about its *place* — its owner,
//! its mode, its not being a link — and all three are satisfied by a record
//! written whole by whoever already owns the file. So every forgery below
//! starts from a record the daemon really wrote, and replaces only its first
//! line: a believable file whose beginning and entries tell two different
//! stories, read back through the same `Recounting` a surface holds, off a
//! description on a disk exactly as a shell reads one.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::Grantee;
use alo_keeping::{Keeping, Writing};
use alo_record::{Asking, Entry};
use alo_recounting::{Account, AtMost, Recounting};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;

/// The moment this machine's afternoon starts at.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A day, as records are shortened by.
fn day() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/// The words this machine reads: every crate's, in one vocabulary, which is
/// the arrangement a shell is really in — so a sentence `alo-keeping` declares
/// and nobody collects fails here rather than on somebody's screen.
fn what_this_machine_says() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A machine stood up: a description that says where the record is, and the
/// record itself, written by the daemon's own writer at these moments.
struct AMachine {
    /// What the machine says about itself.
    description: PathBuf,
    /// Where that description says the record is.
    record: PathBuf,
}

impl AMachine {
    /// A machine whose daemon wrote one entry at each of these moments, in
    /// this order — which is the only order a daemon writes in.
    fn that_wrote(what: &str, moments: &[SystemTime]) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let folder = std::env::temp_dir().join(format!(
            "alo-recounting-says-it-is-{}-{what}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap();
        let record = folder.join("record.jsonl");
        let description = folder.join("agentd.toml");
        fs::write(
            &description,
            format!(
                "format = 2\n\n\
                 [logins]\nperson = 1000\nagent = 1001\ngroup = 1002\n\n\
                 [agent]\nname = \"@files\"\nturn-seconds = 900\nproposal-seconds = 120\n\n\
                 [record]\npath = \"{}\"\nkeeping = \"forever\"\n",
                record.display()
            ),
        )
        .unwrap();
        let mut writing = Writing::opening(&record).unwrap();
        for at in moments {
            writing
                .keep(&Entry::turned_away(
                    "tidy_everything",
                    "there is no verb called tidy_everything",
                    &Grantee::named("@files"),
                    *at,
                ))
                .unwrap();
        }
        drop(writing);
        let machine = Self {
            description,
            record,
        };
        machine.ours_alone();
        machine
    }

    /// The record's first line replaced by a head claiming the record was
    /// shortened to this moment. Nothing else in the file changes: this is the
    /// forgery the check exists for, a file every place-rule still believes.
    fn now_claiming_it_starts_at(&self, since: SystemTime) {
        let text = fs::read_to_string(&self.record).unwrap();
        let entries = text.split_once('\n').unwrap().1;
        let secs = since
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        fs::write(
            &self.record,
            format!(
                "{{\"format\":1,\"since\":{{\"secs_since_epoch\":{secs},\
                 \"nanos_since_epoch\":0}},\"under\":{{\"for-days\":30}}}}\n{entries}"
            ),
        )
        .unwrap();
        self.ours_alone();
    }

    /// The record left readable and writable by nobody but its owner, so what
    /// these tests measure is the check rather than the umask they ran under.
    #[cfg(unix)]
    fn ours_alone(&self) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&self.record, fs::Permissions::from_mode(0o600)).unwrap();
    }

    /// The same, on a machine with no such question to be asked.
    #[cfg(not(unix))]
    fn ours_alone(&self) {}

    /// Everything this machine's record answers, read as a person's side reads
    /// it: off the file the description names, believed first.
    fn everything(&self) -> Account {
        Recounting::described_at(&self.description)
            .unwrap()
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap()
    }
}

/// What the account's sentences read as, for the tests about the words.
fn sentences_of(account: &Account) -> Vec<String> {
    let strings = what_this_machine_says();
    let said = account.said(&strings);
    for one in &said {
        assert!(!one.is_a_bug(), "{one:?}");
    }
    said.iter().map(|one| one.text().to_owned()).collect()
}

/// **A record whose entries deny its own beginning is reported in words,
/// alongside everything that could be read — never as a refusal.**
///
/// The first acceptance. The forged file passes every believing rule — it is
/// where it should be, owned right, closed to everybody else — and the one
/// thing the copy could not forge is the agreement between its first line and
/// its entries. Every entry still comes back beside the sentence, because a
/// reader that refused the whole file would let a forger choose between being
/// believed and being unreadable, and unreadable is the better deal.
#[test]
fn a_record_starting_after_its_own_entries_is_reported_beside_everything_it_holds() {
    let machine = AMachine::that_wrote("forged", &[noon() - day() * 2, noon() - day(), noon()]);
    machine.now_claiming_it_starts_at(noon() - Duration::from_secs(60));

    let account = machine.everything();
    assert_eq!(account.how_many(), 3, "everything readable was answered");
    assert!(!account.is_what_it_says_it_is());
    assert_eq!(account.disagreement().before_it_begins(), [2, 3]);

    let sentences = sentences_of(&account);
    assert!(
        sentences
            .iter()
            .any(|one| one.contains("not what it says it is")),
        "{sentences:?}"
    );
}

/// **The disagreement is carried into the account, not a flag on a struct.**
///
/// The second acceptance. Moments that run backwards this time — an order the
/// daemon never writes in — and the report is a whole sentence inside
/// [`Account::said`], where a surface that draws the account's sentences shows
/// it without having to know the check exists. The flag is still there for a
/// surface that wants it, but the sentence does not depend on anybody asking.
#[test]
fn moments_that_run_backwards_are_a_sentence_in_the_account_itself() {
    let machine = AMachine::that_wrote("backwards", &[noon(), noon() + day(), noon()]);

    let account = machine.everything();
    assert!(!account.is_what_it_says_it_is());
    assert_eq!(account.disagreement().runs_backwards(), [4]);
    assert_eq!(account.how_many(), 3);

    let sentences = sentences_of(&account);
    assert!(
        sentences
            .iter()
            .any(|one| one.contains("not what it says it is") && one.contains("order")),
        "the disagreement is not a sentence in the account: {sentences:?}"
    );
}

/// **A record that was legitimately shortened is not reported** — the case
/// this is easiest to get wrong. A shortening keeps only entries at or after
/// the moment it writes into the head, including the entry exactly at its
/// boundary, so the honest record says *this record does not go all the way
/// back* and nothing about not being what it says it is.
#[test]
fn a_record_legitimately_shortened_is_not_reported() {
    let machine = AMachine::that_wrote(
        "shortened",
        &[
            noon() - day() * 10,
            noon() - day() * 7,
            noon() - day(),
            noon(),
        ],
    );
    let mut writing = Writing::opening(&machine.record).unwrap();
    let pruned = writing
        .prune(Keeping::for_days(7).unwrap(), noon())
        .unwrap();
    drop(writing);
    machine.ours_alone();
    assert!(pruned.anything_removed(), "the shortening really happened");
    assert_eq!(pruned.kept(), 3, "and the boundary entry was kept");

    let account = machine.everything();
    assert!(!account.goes_all_the_way_back());
    assert!(account.is_what_it_says_it_is());

    let sentences = sentences_of(&account);
    assert!(
        sentences
            .iter()
            .any(|one| one.contains("does not go all the way back")),
        "{sentences:?}"
    );
    assert!(
        sentences
            .iter()
            .all(|one| !one.contains("not what it says it is")),
        "an honestly shortened record was reported as a forgery: {sentences:?}"
    );
}

/// **What the daemon writes and shortens, the check is silent about** —
/// measured by writing a record with `alo_keeping::Writing`, shortening it,
/// adding to it afterwards, and reading the account back off the disk. Two
/// entries in one moment are part of the fixture on purpose: a busy second is
/// ordinary, and a check that reported one would report every machine.
#[test]
fn what_this_machine_writes_reads_back_as_what_it_says_it_is() {
    let machine = AMachine::that_wrote(
        "its-own",
        &[noon() - day() * 9, noon() - day() * 3, noon(), noon()],
    );
    assert!(machine.everything().is_what_it_says_it_is());

    let mut writing = Writing::opening(&machine.record).unwrap();
    writing
        .prune(Keeping::for_days(7).unwrap(), noon())
        .unwrap();
    writing
        .keep(&Entry::turned_away(
            "tidy_everything",
            "there is no verb called tidy_everything",
            &Grantee::named("@files"),
            noon() + Duration::from_secs(60),
        ))
        .unwrap();
    drop(writing);
    machine.ours_alone();

    let account = machine.everything();
    assert!(!account.goes_all_the_way_back());
    assert!(account.is_what_it_says_it_is());
    assert!(account.disagreement().before_it_begins().is_empty());
    assert!(account.disagreement().runs_backwards().is_empty());
    assert!(
        sentences_of(&account)
            .iter()
            .all(|one| !one.contains("not what it says it is")),
        "the machine's own record was reported as a forgery"
    );
}

/// The path every forgery above takes is the only door there is: the check is
/// part of reading, so a reader cannot choose to skip it, and there is no
/// second reading that answers without it.
#[test]
fn the_check_is_part_of_reading_rather_than_a_question_a_surface_may_forget() {
    let machine = AMachine::that_wrote("no-second-door", &[noon(), noon() - day()]);
    let account = machine.everything();
    // Nothing here asked about disagreement; the sentence is in the account
    // anyway, which is the whole of the criterion.
    assert!(
        sentences_of(&account)
            .iter()
            .any(|one| one.contains("not what it says it is")),
        "an account was answered without the check having run"
    );
}
