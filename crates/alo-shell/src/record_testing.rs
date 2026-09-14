//! The record the record window's tests are written against: a real one, on a
//! real disk, written by a real turn.
//!
//! A turn on the approval surface's machine (`crate::approval_testing`) writes
//! into an `alo_keeping::Writing` rather than into memory: one move approved
//! and carried out, one declined, one refused before anybody was asked, and one
//! verb the machine does not offer turned away. Beside them, the machine's
//! other kinds of entry are kept the way the daemon keeps them — a question
//! answered here, one that left for a provider, one a rule held back, and alo
//! OS's own errand, which names no agent.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_appearance::{Scheme, TextScale};
use alo_capability::{Given, Grantee};
use alo_egress::{Destination, EgressPolicy, Errand, Indicator, Leaving, OnItsOwn, Why};
use alo_keeping::Writing;
use alo_models::Region;
use alo_record::Entry;
use alo_recounting::Recounting;
use alo_strings::Direction;

use crate::RecordLook;
use crate::approval_testing::{archiving, hour, keeping_in};
pub(crate) use crate::approval_testing::{noon, words};

/// The ordinary light look, read left to right.
pub(crate) fn light() -> RecordLook {
    RecordLook {
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
        reading: Direction::LeftToRight,
    }
}

/// A record kept in a folder of the test's own, removed when it is dropped.
pub(crate) struct Kept {
    /// The folder, held so it outlives the test.
    _held: tempfile::TempDir,
    /// Where the record is.
    pub(crate) at: PathBuf,
}

impl Kept {
    /// The record, asked the way the window asks it.
    pub(crate) fn recounting(&self) -> Recounting {
        Recounting::kept_at(&self.at)
    }

    /// A machine description beside the record, naming it as
    /// `docs/contracts/machine-description.md` has it — the way a person's side
    /// of the machine finds its record without anybody handing it a path.
    pub(crate) fn described(&self) -> PathBuf {
        let description = self.at.with_file_name("agentd.toml");
        std::fs::write(
            &description,
            format!(
                "format = 2\n\n\
                 [logins]\nperson = 1000\nagent = 1001\ngroup = 1002\n\n\
                 [agent]\nname = \"@files\"\nturn-seconds = 900\nproposal-seconds = 120\n\n\
                 [record]\npath = \"{}\"\nkeeping = \"forever\"\n",
                self.at.display()
            ),
        )
        .unwrap();
        description
    }

    /// Keep one more entry, as the daemon would while the window is open.
    pub(crate) fn keeping(&self, entry: &Entry) {
        let mut writing = Writing::opening(&self.at).unwrap();
        writing.keep(entry).unwrap();
        drop(writing);
        ours_alone(&self.at);
    }
}

/// A record with nothing in it yet.
pub(crate) fn an_empty_record() -> Kept {
    let held = tempfile::tempdir().unwrap();
    let at = held.path().join("record.jsonl");
    drop(Writing::opening(&at).unwrap());
    ours_alone(&at);
    Kept { _held: held, at }
}

/// An afternoon on this machine, in the order it happened: a move carried out,
/// a move declined, a move refused before anybody was asked, a verb turned
/// away, a question answered here, a question that left, one held back, and a
/// model fetched by alo OS on its own.
pub(crate) fn an_afternoon_kept() -> Kept {
    let kept = an_empty_record();
    let mut writing = Writing::opening(&kept.at).unwrap();
    keeping_in(&mut writing, |turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        turning.approving(march, grants, noon()).unwrap();
        let april = archiving(turning, grants, places, "april");
        turning.declining(april, noon()).unwrap();
        let elsewhere = places.archive.with_file_name("nobody-granted-this");
        std::fs::create_dir_all(&elsewhere).unwrap();
        assert!(
            turning
                .proposing(
                    "move_file",
                    &[
                        (
                            "file",
                            Given::text(places.invoice("may").to_string_lossy().into_owned())
                        ),
                        (
                            "into",
                            Given::text(elsewhere.to_string_lossy().into_owned())
                        ),
                    ],
                    grants,
                    hour(),
                    noon(),
                )
                .is_err()
        );
        assert!(
            turning
                .proposing("tidy_everything", &[], grants, hour(), noon())
                .is_err()
        );
    });
    let mail = Grantee::named("@mail");
    let strings = words();
    let a_provider = Destination::provider("alo", Region::Declared("the EU".to_owned())).unwrap();
    writing
        .keep(&Entry::answered_here(&mail, noon() + minutes(1)))
        .unwrap();
    let departing = Indicator::default()
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::because(&mail, Why::Asking, a_provider.clone()),
            noon() + minutes(2),
        )
        .unwrap();
    writing.keep(&Entry::left(&departing)).unwrap();
    let held_back = Indicator::default()
        .beginning(
            &EgressPolicy::NothingLeaves,
            Leaving::because(&mail, Why::Asking, a_provider),
            noon() + minutes(3),
        )
        .unwrap_err();
    writing
        .keep(&Entry::held_back(&held_back, &strings, noon() + minutes(3)))
        .unwrap();
    let underway = Indicator::default().beginning_on_its_own(
        OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::at("models.alo.example").unwrap(),
        ),
        noon() + minutes(4),
    );
    writing.keep(&Entry::left_on_its_own(&underway)).unwrap();
    drop(writing);
    ours_alone(&kept.at);
    kept
}

/// A record of `how_many` verbs turned away, a minute apart — long enough that
/// no window holds all of it.
pub(crate) fn a_long_record(how_many: u64) -> Kept {
    let kept = an_empty_record();
    let mut writing = Writing::opening(&kept.at).unwrap();
    for which in 0..how_many {
        writing
            .keep(&Entry::turned_away(
                "tidy_everything",
                "there is no verb called tidy_everything",
                &Grantee::named("@files"),
                noon() + minutes(which),
            ))
            .unwrap();
    }
    drop(writing);
    ours_alone(&kept.at);
    kept
}

/// `how_many` minutes.
fn minutes(how_many: u64) -> Duration {
    Duration::from_secs(60 * how_many)
}

/// A file left readable and writable by nobody but its owner, which is a record
/// this machine will believe.
pub(crate) fn ours_alone(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}
