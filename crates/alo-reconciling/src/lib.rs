//! Every v0.01 promise, against the evidence this repository can actually run.
//!
//! `docs/features.md` is the definition of what alo OS is — the scope gate, and
//! the sentences a customer would read. `docs/autonomy/v0-01-evidence.md` is the
//! other half of that document and is new here: **for every `[v0.01]` line, the
//! test or the report that shows it, and what is still owed.** This crate is
//! what makes the second file true of the first rather than a list somebody
//! wrote once.
//!
//! # Why a crate and not a document somebody reads
//!
//! Because the audit that this task exists to replace was done seven times by
//! reading. `ROADMAP.md` records the result: **six v0.01 promises with no item
//! and no line, found one at a time, and twice the audit believed it had found
//! the last.** A promise is not missed because somebody is careless; it is
//! missed because a hundred-line document is read in a different order every
//! time and the eye stops where the last reading stopped.
//!
//! So the reconciliation is arithmetic. Every promise is read out of the
//! definition, every promise must be named exactly once in the ledger, and every
//! ledger entry must name a promise that is still in the definition. A promise
//! added to `docs/features.md` and not reconciled fails the gate in the change
//! that adds it, which is the only moment anybody has the knowledge to reconcile
//! it.
//!
//! # What counts as evidence, and what it deliberately is not
//!
//! Two things, and no third: **a test in this repository**, named by its path
//! and holding at least one `#[test]`, or **a task report** under
//! `docs/autonomy/updates/`. A path that is neither is refused rather than
//! believed, and a test that is not there is refused rather than assumed to have
//! moved.
//!
//! This crate does not judge whether a test *proves* its promise — nothing
//! mechanical can, and pretending otherwise would be the audit lying in a new
//! way. What it removes is the failure that actually happened: a promise nobody
//! reconciled at all, and a pointer at a file that has since gone.
//!
//! **An empty answer is refused too.** An entry that names no evidence must say
//! what is owed, in a sentence long enough to be an answer — see
//! [`owed::AN_ANSWER`]. A promise with neither is this audit's finding, and
//! naming it is the point of the exercise rather than a failure of it.
//!
//! # It says nothing to a person
//!
//! Nothing here reaches a screen. The reader of a [`Finding`] is whoever is
//! reconciling the release, in the repository, with the two documents open — so
//! the findings keep their English and their `Display`, exactly as
//! `alo_saying::NotCollected` does for the same reason. There are no strings to
//! externalise here, and `alo-saying` does not collect this crate.
//!
//! # It reads no disk
//!
//! [`reconcile`] is handed the two documents' text and a way to read a file by
//! its repository-relative path. The test in `tests/` is what puts the real
//! repository behind it. That is what lets every refusal below be shown
//! happening against a fixture: **an audit that has never been seen to refuse
//! anything passes on the day it stops looking, in exactly the same colour.**

pub mod entry;
pub mod evidence;
pub mod finding;
pub mod ledger;
pub mod owed;
pub mod promise;
pub mod reconciling;

pub use entry::Entry;
pub use evidence::Evidence;
pub use finding::Finding;
pub use ledger::entries_in;
pub use promise::{Promise, promises_in};
pub use reconciling::{Reconciled, reconcile};
