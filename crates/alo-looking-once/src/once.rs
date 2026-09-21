//! The whole act, once: ask the base what is running, ask the place what it
//! offers, write the departure down, keep the answer.
//!
//! **One call, one check.** [`AtAStart::look_once`] is a function that asks and
//! answers. There is nothing in this file that could call it again — no loop,
//! no timer, no thread and no retry — and the test that reads this crate's
//! source is what keeps it that way.
//!
//! # In the order it happens, and the order is the argument
//!
//! 1. **What this machine is running**, read at the moment it is wanted out of
//!    what the base has already written down (`alo_updating::WrittenDown`). A
//!    machine that will not say what it is running has nothing to compare an
//!    offer against, so there is no question to ask and nothing leaves.
//!
//!    **Read rather than asked, because the base refuses to answer the
//!    person.** Until 2026-09-21 this asked `bootc status`, and on a real bootc
//!    machine that command answers uid 1000 with *This command must be executed
//!    as the root user* — so this unit, which runs as the person and holds
//!    nothing, failed at every boot and kept nothing
//!    (`docs/autonomy/updates/the-base-answers-only-root.md`). The answer was
//!    not to make this root, which is what this crate's own comments and
//!    ADR 0018 spend their length refusing: the kernel's own command line names
//!    the deployment that booted and the base's `.origin` file beside it names
//!    the build, both world-readable, and no program is run to read either.
//! 2. **The record is opened**, before anything is asked. A machine that cannot
//!    write down what it is about to do does not do it.
//! 3. **The check**, on the indicator for the whole of it, written into the
//!    record before it comes off, on every road including every refusal.
//! 4. **What to say**, through `alo_looking::SaidOnce`, so that a machine with
//!    no way out at all is told about once rather than at every asking.
//! 5. **The answer is kept**, where a surface reads it back without asking
//!    again and putting a second line on somebody's indicator.
//!
//! A refusal at any step leaves the machine exactly as it was. A check asks a
//! question; nothing here changes anything about this machine but the two files
//! it writes.
//!
//! # `SaidOnce` across a start, honestly
//!
//! A `SaidOnce` remembers for as long as whatever holds it lasts, and this
//! program lasts one check — so within a start it decides what it always
//! decided, and it is handed in rather than made here so that the caller is what
//! holds it and a test can ask twice.
//!
//! **It is deliberately not written to a disk.** That crate's own reason: a
//! machine that remembered across restarts would stay silent about a network
//! that was down last week. What keeps a person from meeting the same line at
//! every start is not a memory of this program's — it is that a refused check
//! keeps nothing, so no surface has anything new to put in front of them. The
//! line each start leaves is in the journal, for whoever administers the
//! machine.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_egress::Indicator;
use alo_keeping::Writing;
use alo_looking::{Because, Found, Kept, Place, SaidOnce, Say, ThePlace, look};
use alo_updating::WrittenDown;

use crate::record::IntoTheRecord;
use crate::refusing::DidNotLook;

/// Where the answer a check keeps lives on an alo OS machine.
///
/// `alo_looking::THE_ANSWER`, named again here only so that a reader of this
/// crate can see both files a check writes in one place. It is that constant
/// rather than a second spelling of the path.
pub const THE_ANSWER: &str = alo_looking::THE_ANSWER;

/// Where the record of what this machine checked lives.
///
/// **Beside the person's record rather than inside it.** `alo_keeping::Writing`
/// has exactly one writer per file, because two processes appending to one
/// interleave, and `alo-agentd` is the person's record's writer. `alo-brokerd`
/// met this first and answered it the same way. A check at a start cannot be
/// ordered against a sign-in tightly enough to promise the two never overlap,
/// and a record that might interleave is not evidence of anything.
pub const THE_RECORD: &str = "/var/lib/alo/checking-for-updates.jsonl";

/// The one look this machine takes on its way up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtAStart {
    /// Where the answer is kept.
    kept: Kept,
    /// Where the departure is written down.
    record: PathBuf,
}

impl AtAStart {
    /// The two files an alo OS machine keeps this in.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self {
            kept: Kept::on_this_machine(),
            record: PathBuf::from(THE_RECORD),
        }
    }

    /// The same two files, under this folder — a test's.
    ///
    /// **The folder is not made.** Where these go is the image's decision
    /// (`usr/lib/tmpfiles.d/alo.conf`), and a program that made directories on
    /// the way to a path it was handed would turn a typo into a second record
    /// nobody is reading — which is `alo_keeping::Writing`'s own rule.
    #[must_use]
    pub fn in_folder(folder: &Path) -> Self {
        Self {
            kept: Kept::in_folder(folder),
            record: folder.join("checking-for-updates.jsonl"),
        }
    }

    /// Where the answer is kept.
    #[must_use]
    pub fn answer(&self) -> &Kept {
        &self.kept
    }

    /// Where the departure is written down.
    #[must_use]
    pub fn record(&self) -> &Path {
        &self.record
    }

    /// Look once, because this machine started.
    ///
    /// The only place in this repository that names that occasion on a booted
    /// machine.
    ///
    /// # Errors
    /// [`DidNotLook`], on each of which nothing is kept and the machine is
    /// exactly as it was.
    pub fn look_once(
        &self,
        machine: &WrittenDown,
        place: &Place,
        asking: &impl ThePlace,
        said: &mut SaidOnce,
        now: SystemTime,
    ) -> Result<Found, DidNotLook> {
        let running = machine.running().map_err(DidNotLook::TheBaseWouldNotSay)?;
        let mut writing = Writing::opening(&self.record).map_err(DidNotLook::NoRecordToWriteIn)?;

        let mut indicator = Indicator::default();
        let mut into = IntoTheRecord::writing_into(&mut writing);
        let answered = look(
            &mut indicator,
            &mut into,
            now,
            place,
            &running,
            Because::ThisMachineStarted,
            asking,
        );
        if let Some(why) = into.what_went_wrong() {
            return Err(DidNotLook::NotWrittenDown(why));
        }

        let found = match said.what_to_say(answered) {
            Ok(found) => found,
            Err(Say::This(refusal)) => return Err(DidNotLook::NothingFoundOut(refusal)),
            Err(Say::SaidAlready) => return Err(DidNotLook::SaidAlready),
        };
        self.kept.keep(&found).map_err(DidNotLook::NotKept)?;
        Ok(found)
    }
}
