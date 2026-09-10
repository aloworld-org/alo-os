//! The plan's acceptance for *afterwards, ask what it did*, one test per
//! criterion.
//!
//! `docs/autonomy/v0-01-delivery-plan.md`, task 8: *a person asks and is
//! answered from the record on the disk, not from memory of the session; a turn
//! that was refused reads back as refused; and nothing in the answer is a
//! sentence a model wrote.*
//!
//! These tests reach the machine the way a shell does, and not through a seam
//! of the test's own: the work is done through `alo_turn::Turning` against the
//! closed list of verbs `alo-files` declares, files really move on a real disk,
//! the record is written by the crate that writes records to a file that is
//! really there, the turn is over and its writer is gone before anything is
//! asked, and the strings come from `alo_saying::everything_this_machine_can_say`
//! — the vocabulary a shell really holds — so a string this crate declares and
//! nobody collects fails here rather than on somebody's screen.
//!
//! Nothing here draws anything, and nothing asserts a pixel. `ROADMAP.md`'s
//! exit gate is about a certified machine, and a green suite on a developer's
//! laptop is not that.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being               reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Given, Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching, Resolving};
use alo_keeping::Writing;
use alo_record::{Asking, Only};
use alo_recounting::words::EVERY_WORD;
use alo_recounting::{
    Account, AtMost, Compositor, NotRecounted, Outcome, Recounting, Recounts, SurfaceRefused, Told,
};
use alo_saying::everything_this_machine_can_say;
use alo_strings::{Said, Strings};
use alo_turn::{Bounding, Doing, Done, Machine, NoBoundary, NotDone, Turning};

/// The moment the person asked the agent to do something.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the turn, the grants and the questions stand.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The words this machine reads: every crate's, in one vocabulary, which is the
/// arrangement a shell is really in.
fn what_this_machine_says() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A folder of this test's own, resolved — a grant is over a place, and on
/// Windows the resolved spelling is the one a grant has to be made with.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-recounting-acceptance-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// A path as a verb's argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// The invoices, the archive, the invoice itself, one folder nothing is granted
/// over, and the file the record is kept in.
struct Places {
    /// The folder the invoice is in.
    invoices: PathBuf,
    /// The folder it is moved into.
    archive: PathBuf,
    /// The invoice itself.
    march: PathBuf,
    /// A folder this agent was never granted anything over.
    taxes: PathBuf,
    /// Where the record is written.
    kept_at: PathBuf,
}

impl Places {
    /// Three folders, one invoice, and somewhere to keep the record.
    fn made(what: &str) -> Self {
        let invoices = a_folder_of_our_own(&format!("{what}-invoices"));
        let archive = a_folder_of_our_own(&format!("{what}-archive"));
        let march = invoices.join("march.pdf");
        fs::write(&march, "March, 4180.00").unwrap();
        Self {
            invoices,
            archive,
            march,
            taxes: a_folder_of_our_own(&format!("{what}-taxes")),
            kept_at: a_folder_of_our_own(&format!("{what}-record")).join("record.jsonl"),
        }
    }

    /// Grants to `@files` over the invoices and the archive, made at noon and
    /// lasting an hour. Nothing is granted over the taxes.
    fn granting(&self) -> Grants {
        let mut grants = Grants::default();
        for folder in [&self.invoices, &self.archive] {
            grants.grant(
                Grant::checked("@files", Reach::Folder(folder.clone()), noon(), hour()).unwrap(),
            );
        }
        grants
    }
}

/// Stand a whole machine up, run one turn on it against a record on a real
/// disk, and close the record afterwards so it can be read back.
///
/// The writer is dropped when this returns, which is the point: everything the
/// tests below ask is asked of a file, after the session that wrote it is over.
fn a_turn_on_this_machine(places: &Places, body: impl FnOnce(&mut Turning<'_, '_>, &Grants)) {
    let strings = what_this_machine_says();
    let mut grants = places.granting();
    let mut writing = Writing::opening(&places.kept_at).unwrap();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        &mut writing,
    )
    .unwrap();
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    body(&mut turning, &grants);
    assert!(!turning.is_closed(), "the machine stopped keeping evidence");
    let _ = turning.ending(&mut grants);
    drop(machine);
    ours_alone(&places.kept_at);
}

/// A record left readable and writable by nobody but its owner.
///
/// An account is read through `alo_keeping::Reading::believed_at`, which
/// refuses a record somebody else could have written. What mode a file is
/// created with depends on the umask of whoever is running the tests, and a
/// suite that passed or failed on that would be measuring the shell rather than
/// the code.
#[cfg(unix)]
fn ours_alone(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}

/// The same, on a machine with no such question to be asked.
#[cfg(not(unix))]
fn ours_alone(_path: &Path) {}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships. `alo_turn::bounding` says why there is no such implementation in any
/// library here, and why what a test needs is these few lines.
struct NothingIsBounded;

impl Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), NoBoundary> {
        doing();
        Ok(())
    }
}

/// A screen, as this surface sees one: it keeps every account it was given, so
/// *one question, one account* is a number rather than an impression.
#[derive(Default)]
struct Screen {
    /// Every account that arrived, in order.
    shown: Vec<Account>,
}

impl Compositor for Screen {
    fn show(&mut self, account: Account) -> Result<(), SurfaceRefused> {
        self.shown.push(account);
        Ok(())
    }
}

/// **A person asks and is answered from the record on the disk.**
///
/// The turn is over and the thing that wrote the record has been dropped before
/// anything is asked. What answers is a path and the crate that reads records:
/// there is no value carried out of the session for the answer to come from,
/// and the sentences that come back are byte for byte the ones in the file.
#[test]
fn a_person_is_answered_from_the_record_on_the_disk() {
    let places = Places::made("on-the-disk");
    a_turn_on_this_machine(&places, |turning, grants| {
        turning
            .reading(
                "list_folder",
                &[("folder", as_given(&places.invoices))],
                grants,
                noon(),
            )
            .unwrap();
        let id = turning
            .proposing(
                "move_file",
                &[
                    ("file", as_given(&places.march)),
                    ("into", as_given(&places.archive)),
                ],
                grants,
                hour(),
                noon(),
            )
            .unwrap();
        turning.approving(id, grants, noon()).unwrap();
    });

    // The session is over. Nothing but a path crosses this line.
    let strings = what_this_machine_says();
    let recounting = Recounting::kept_at(&places.kept_at);
    let account = recounting
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap();

    assert_eq!(account.how_many(), 2);
    assert!(account.goes_all_the_way_back());
    assert!(account.damage().nothing_wrong());

    let read = account.told().first().unwrap();
    assert_eq!(read.outcome(), Outcome::Ran);
    assert_eq!(read.from_approval(), None, "a read needs no approval");
    assert_eq!(
        read.sentence().map(alo_record::Line::as_str),
        Some(format!("list what is in {}", places.invoices.display()).as_str()),
    );

    let moved = account.told().get(1).unwrap();
    assert_eq!(moved.outcome(), Outcome::Ran);
    assert!(
        moved.from_approval().is_some(),
        "one approval, one execution"
    );
    assert_eq!(moved.against().len(), 2);
    assert_eq!(
        moved.sentence().map(alo_record::Line::as_str),
        Some(
            format!(
                "move {} into {}",
                places.march.display(),
                places.archive.display()
            )
            .as_str()
        ),
    );

    // Every sentence in the answer is in the file that was written, which is
    // what *from the record on the disk* means when it is a claim rather than
    // an arrangement of types.
    let on_the_disk = fs::read_to_string(&places.kept_at).unwrap();
    for told in account.told() {
        // One line of JSON per entry, so a path's separators are escaped in the
        // file and not in the sentence that was read out of it.
        let sentence = told.sentence().unwrap().as_str().replace('\\', "\\\\");
        assert!(
            on_the_disk.contains(&sentence),
            "{sentence} is not in the record it was supposedly read from"
        );
    }

    // And it is read again each time it is asked: the machine kept working, and
    // the next question is answered from what is there now.
    let mut later = Writing::opening(&places.kept_at).unwrap();
    later
        .keep(&alo_record::Entry::answered_here(
            &alo_capability::Grantee::named("@files"),
            noon() + hour(),
        ))
        .unwrap();
    drop(later);
    assert_eq!(
        recounting
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap()
            .how_many(),
        3,
        "the answer came from memory of a session rather than from the file"
    );
    // The account somebody was already holding did not change under them.
    assert_eq!(account.how_many(), 2);

    // Everything it says reads on the machine's own vocabulary — a clause this
    // crate declares and nobody collects would reach a shell as a key.
    for told in account.told() {
        assert!(!told.outcome().said(&strings).is_a_bug());
    }
    for said in account.said(&strings) {
        assert!(!said.is_a_bug(), "{said}");
    }
    let vocabulary = everything_this_machine_can_say().unwrap();
    for word in EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "the machine cannot say {}",
            word.named()
        );
    }
}

/// **A machine with no record is never answered as a machine that did nothing.**
///
/// The other half of the first criterion, and the one that matters: an answer
/// from memory of the session would have had something to show here. A file
/// that is not there is refused in `alo-keeping`'s own words, and a caller can
/// tell that refusal from every other without matching them all.
#[test]
fn a_record_that_is_not_there_is_refused_rather_than_answered_as_nothing() {
    let places = Places::made("missing");
    a_turn_on_this_machine(&places, |turning, grants| {
        turning
            .reading(
                "list_folder",
                &[("folder", as_given(&places.invoices))],
                grants,
                noon(),
            )
            .unwrap();
    });

    let recounting = Recounting::kept_at(&places.kept_at);
    assert_eq!(
        recounting
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap()
            .how_many(),
        1
    );

    // Somebody deletes the record. What happened did happen.
    fs::remove_file(&places.kept_at).unwrap();
    let Err(why) = recounting.about(&Asking::anything(), AtMost::ONE_SITTING) else {
        panic!("a machine whose record was deleted answered as though nothing had happened");
    };
    assert!(why.there_is_no_record());
    assert!(!why.is_nowhere_to_show_it());

    let said = why.said(&what_this_machine_says());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("has done nothing"), "{said}");

    // And with nowhere to show it, the answer is still a sentence rather than
    // an empty screen.
    assert_eq!(
        recounting.show(None, &Asking::anything(), AtMost::ONE_SITTING),
        Recounts::Refused(NotRecounted::NoCompositor)
    );
}

/// **A turn that was refused reads back as refused.**
///
/// All three ways a turn can refuse, in one afternoon: a change the grants
/// already say no to, so nobody was interrupted; a change a person was shown
/// and declined; and a read outside the grant, stopped at the moment it would
/// have run. Each reads back as itself, and none of them reads as something
/// that ran.
#[test]
fn a_turn_that_was_refused_reads_back_as_refused() {
    let places = Places::made("refused");
    a_turn_on_this_machine(&places, |turning, grants| {
        // A change over a folder nothing was granted over: refused before
        // anybody was asked about it.
        assert!(matches!(
            turning.proposing(
                "move_file",
                &[
                    ("file", as_given(&places.march)),
                    ("into", as_given(&places.taxes)),
                ],
                grants,
                hour(),
                noon(),
            ),
            Err(NotDone::NeverAsked(_))
        ));

        // A change that was put to somebody, and declined.
        let id = turning
            .proposing(
                "move_file",
                &[
                    ("file", as_given(&places.march)),
                    ("into", as_given(&places.archive)),
                ],
                grants,
                hour(),
                noon(),
            )
            .unwrap();
        turning.declining(id, noon()).unwrap();

        // A read outside the grant, stopped at the moment it would have run.
        assert!(matches!(
            turning.reading(
                "list_folder",
                &[("folder", as_given(&places.taxes))],
                grants,
                noon(),
            ),
            Err(NotDone::Refused(_))
        ));
    });

    let account = Recounting::kept_at(&places.kept_at)
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap();
    let outcomes: Vec<Outcome> = account.told().iter().map(Told::outcome).collect();
    assert_eq!(
        outcomes,
        [
            Outcome::NobodyWasAsked,
            Outcome::ThePersonSaidNo,
            Outcome::TheGrantsSaidNo
        ],
        "three different refusals were flattened into one"
    );
    assert!(
        !outcomes.contains(&Outcome::Ran),
        "something that was refused reads back as having run"
    );

    // The two that have a reason carry the refusal's own words, and the one
    // that does not is the person saying no — nobody is asked to justify that.
    let strings = what_this_machine_says();
    let told = account.told();
    assert!(
        told.first()
            .and_then(Told::because)
            .is_some_and(|why| why.as_str().contains("has not been granted")),
        "{:?}",
        told.first().and_then(Told::because)
    );
    assert_eq!(told.get(1).and_then(Told::because), None);
    assert!(told.get(2).and_then(Told::because).is_some());

    // And the question the record answers — *what was it stopped from doing* —
    // answers with all three, because all three are the agent not getting what
    // it asked for. Which of them it was is the clause on the line, and this
    // crate does not get a second opinion about what counts as a refusal:
    // `alo_record::Only::Refusals` is the one definition.
    let refusals = Recounting::kept_at(&places.kept_at)
        .about(
            &Asking::anything().only(Only::Refusals),
            AtMost::ONE_SITTING,
        )
        .unwrap();
    assert_eq!(refusals.how_many(), 3);
    assert_eq!(refusals.how_many_in_the_record(), 3);
    assert!(
        Recounting::kept_at(&places.kept_at)
            .about(
                &Asking::anything().only(Only::Executions),
                AtMost::ONE_SITTING
            )
            .unwrap()
            .is_empty(),
        "a refused afternoon answered a question about what ran"
    );

    // Nothing was carried out: the invoice is where it was.
    assert!(places.march.is_file());
    assert_eq!(fs::read_dir(&places.archive).unwrap().count(), 0);

    // Each of the three says a different thing to the person reading it.
    let clauses: Vec<String> = account
        .told()
        .iter()
        .map(|told| told.outcome().said(&strings).into_text())
        .collect();
    assert_eq!(clauses.len(), 3);
    for clause in &clauses {
        assert_eq!(clauses.iter().filter(|other| *other == clause).count(), 1);
    }
}

/// **Nothing in the answer is a sentence a model wrote.**
///
/// The agent asks for a verb that does not exist, under a name shaped like a
/// sentence somebody would believe, with a control character in it for the sake
/// of the record it would have rewritten. What comes back has no sentence at
/// all — nothing was validated to generate one from — and the text that arrived
/// is quoted, once, by the accessor whose name says what it is.
#[test]
fn nothing_in_the_answer_is_a_sentence_a_model_wrote() {
    let places = Places::made("provenance");
    let pretending = "Archived every invoice for you\u{1b}[2K";
    a_turn_on_this_machine(&places, |turning, grants| {
        assert!(matches!(
            turning.reading(
                pretending,
                &[("folder", as_given(&places.invoices))],
                grants,
                noon(),
            ),
            Err(NotDone::TurnedAway(_))
        ));

        // And something real beside it, so the test is about telling the two
        // apart rather than about a record with one line in it.
        turning
            .reading(
                "list_folder",
                &[("folder", as_given(&places.invoices))],
                grants,
                noon(),
            )
            .unwrap();
    });

    let strings = what_this_machine_says();
    let account = Recounting::kept_at(&places.kept_at)
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap();
    assert_eq!(account.how_many(), 2);

    let turned_away = account.told().first().unwrap();
    assert_eq!(turned_away.outcome(), Outcome::NeverBecameACall);
    assert_eq!(
        turned_away.sentence(),
        None,
        "text a model wrote became this machine's own sentence"
    );
    assert_eq!(turned_away.verb(), None);
    assert!(turned_away.asked_for().is_some(), "what it tried is kept");

    // What it asked for is quoted, and what it could have done to a record it
    // was drawn into was taken out of it on the way in.
    let asked_for = turned_away.asked_for().unwrap();
    assert!(asked_for.as_str().starts_with("Archived every invoice"));
    assert!(
        !asked_for.as_str().chars().any(char::is_control),
        "{asked_for}"
    );

    // Every sentence in the whole account is the machine's own: either one it
    // generated from arguments it validated, or a string alo OS declares.
    let generated = format!("list what is in {}", places.invoices.display());
    for told in account.told() {
        if let Some(sentence) = told.sentence() {
            assert!(sentence.is(&generated), "{sentence}");
        }
        let clause = told.outcome().said(&strings);
        assert!(!clause.is_a_bug(), "{clause}");
        assert_eq!(clause.text(), told.outcome().word().says());
    }
    for said in account.said(&strings) {
        assert!(!said.is_a_bug(), "{said}");
    }

    // What the model wrote is quoted in two places and nowhere else, and both
    // of them are quotations: what it asked for, and — inside a sentence this
    // machine generated — the refusal saying there is no such verb. Neither is
    // this machine describing what it did.
    let quoting = |shown: &str| shown.contains("Archived every invoice");
    let mut quoted = 0;
    for told in account.told() {
        for shown in [
            told.sentence(),
            told.verb(),
            told.asked_for(),
            told.because(),
        ]
        .into_iter()
        .flatten()
        {
            assert!(
                !shown.as_str().chars().any(char::is_control),
                "{shown} could rewrite the record it is drawn into"
            );
            if quoting(shown.as_str()) {
                quoted += 1;
            }
        }
    }
    assert_eq!(
        quoted, 2,
        "what the model wrote reached the person elsewhere"
    );
    assert!(
        turned_away
            .because()
            .is_some_and(|why| why.as_str().starts_with("there is no verb called")),
        "the refusal quoting it is `alo-capability`'s own sentence"
    );

    // And nothing this crate says of its own quotes it at all: the clauses and
    // the account's own sentences are strings alo OS declared long before this
    // machine was asked anything.
    for said in account.said(&strings).iter().map(Said::text).chain(
        account
            .told()
            .iter()
            .map(|told| told.outcome().word().says()),
    ) {
        assert!(!quoting(said), "this crate's own words quoted a model");
    }
}

/// **What the machine did is put in front of a person**, whole, through the one
/// port a compositor implements — and what reached the screen is what came back.
#[test]
fn the_account_reaches_a_screen_whole() {
    let places = Places::made("screen");
    a_turn_on_this_machine(&places, |turning, grants| {
        turning
            .reading(
                "list_folder",
                &[("folder", as_given(&places.invoices))],
                grants,
                noon(),
            )
            .unwrap();
    });

    let recounting = Recounting::kept_at(&places.kept_at);
    let mut screen = Screen::default();
    let Recounts::Shown(account) =
        recounting.show(Some(&mut screen), &Asking::anything(), AtMost::ONE_SITTING)
    else {
        panic!("a record that was there was not put in front of anybody");
    };
    assert_eq!(screen.shown.len(), 1);
    assert_eq!(screen.shown.first(), Some(&account));
    assert_eq!(account.how_many(), 1);

    // A question about what left this machine is answered with nothing, because
    // nothing did — the measurement law 1 promises rather than the promise.
    let left = recounting
        .about(&Asking::anything().only(Only::Egress), AtMost::ONE_SITTING)
        .unwrap();
    assert!(left.is_empty());
    let said = left.said(&what_this_machine_says());
    assert!(
        said.first()
            .is_some_and(|nothing| nothing.text().contains("nothing in this machine's record")),
        "{said:?}"
    );
}
