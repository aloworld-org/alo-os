//! One firing of the timer: read the folder, ask each person's window what is
//! outside it, remove exactly that, and write down which turns lost their undo.
//!
//! Everything it needs is handed to it — the disk, the remover, the moment and
//! the record — so that the whole of what the unit does can be walked on a
//! machine that has no subvolumes and holds no capability. What the program in
//! `src/bin/` adds is only the machine's own paths.
//!
//! # The order, and why it is this one
//!
//! 1. **Is this a machine that keeps anything?** A disk without subvolumes has
//!    nothing to remove, and [`Swept::NotOnThisMachine`] is an answer rather
//!    than a failure — ADR 0045's sixth term, and a timer that failed at every
//!    firing on a healthy machine is one its administrator learns to ignore.
//! 2. **Everything the window no longer reaches goes**, whatever the disk says.
//!    That is the person's own setting being obeyed
//!    ([`crate::keeping::at_sign_in`]), and it is the half that keeps a roomy
//!    machine from holding every snapshot for ever.
//! 3. **Then, only while the disk is below [`crate::THE_FLOOR`], the oldest of
//!    what is left goes**, one at a time, asking the filesystem again after
//!    each. `crate::deciding` has why the stopping cannot be decided in
//!    advance.
//! 4. **What went is written down**, named rather than counted, in the words the
//!    person approved at the time — one entry for each of the two reasons, so
//!    that a person reading the record is told which it was.
//!
//! # Nothing is recorded that did not happen
//!
//! A removal the machine refused — no capability, a snapshot already gone, a
//! disk that would not answer — leaves that turn out of the entry and says so
//! to the journal. The next firing finds it again. A record that said an undo
//! was gone while its snapshot sat on the disk would be the one line in this
//! file a person could not check, and it is the line a machine with a
//! misconfigured unit would write every hour.

use std::path::Path;
use std::time::SystemTime;

use alo_record::{Entry, Forgone, WhyLetGo};

use crate::deciding::WhatGoes;
use crate::found::{Everyone, Whose};
use crate::keeping::{THE_FILE, at_sign_in};
use crate::removing::Removing;
use crate::the_disk::{AskingTheDisk, WhatItIs};
use crate::writing_it_down::WritingItDown;

/// What one firing of the timer did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Swept {
    /// This machine keeps nothing an undo could put back, so there was nothing
    /// to let go of. ADR 0045's sixth term, and the honest answer on every
    /// machine installed before the filesystem was decided.
    NotOnThisMachine,
    /// It read the folder and did what follows.
    Done(WhatItDid),
}

/// What a firing that ran actually did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WhatItDid {
    /// How many kept turns went because the window no longer reached them.
    outside_the_window: usize,
    /// How many went because the disk was short of room.
    for_room: usize,
    /// Everything it would not or could not do, in English, for the journal:
    /// what it stepped over, what a removal refused, and a person's settings
    /// file that did not read.
    said: Vec<String>,
}

impl WhatItDid {
    /// How many kept turns went because the window no longer reached them.
    #[must_use]
    pub const fn outside_the_window(&self) -> usize {
        self.outside_the_window
    }

    /// How many went because the disk was short of room.
    #[must_use]
    pub const fn for_room(&self) -> usize {
        self.for_room
    }

    /// Whether anything at all went.
    #[must_use]
    pub const fn anything_went(&self) -> bool {
        self.outside_the_window > 0 || self.for_room > 0
    }

    /// Everything for the journal.
    #[must_use]
    pub fn said(&self) -> &[String] {
        &self.said
    }
}

/// One firing of the timer, over the folder at `folder`.
///
/// `now` is handed in rather than read, for `alo-keeping-up`'s reason: a crate
/// that consulted a clock of its own would be a second opinion about time, and
/// the moment the whole of one firing is measured against should be one moment
/// rather than however many the walk took.
pub fn sweep(
    folder: &Path,
    disk: &dyn AskingTheDisk,
    remover: &dyn Removing,
    record: &mut dyn WritingItDown,
    now: SystemTime,
) -> Swept {
    let mut did = WhatItDid::default();
    match disk.what_it_is(folder) {
        Ok(WhatItIs::OneThatKeeps) => {}
        Ok(WhatItIs::NotOneThatKeeps) => return Swept::NotOnThisMachine,
        Err(why) => {
            // A machine that cannot say what its own filesystem is removes
            // nothing: the alternative is removing a person's history on a
            // guess.
            did.said
                .push(format!("{} was not asked about: {why}", folder.display()));
            return Swept::Done(did);
        }
    }

    let everyone = Everyone::under(folder);
    did.said.extend(everyone.stepped_over().iter().cloned());
    for whose in everyone.whose() {
        one_person(whose, disk, remover, record, now, &mut did);
    }
    Swept::Done(did)
}

/// One person's kept turns, under their own window.
fn one_person(
    whose: &Whose,
    disk: &dyn AskingTheDisk,
    remover: &dyn Removing,
    record: &mut dyn WritingItDown,
    now: SystemTime,
    did: &mut WhatItDid,
) {
    let (settings, not_read) = at_sign_in(&whose.settings().join(THE_FILE));
    if let Some(refused) = not_read {
        // Said, and the shipped window is used — which keeps rather than
        // removes, so a file somebody typed wrong can only cost a person
        // snapshots they would have lost anyway.
        did.said.push(format!(
            "{} did not read, so this machine keeps what alo OS ships: {:?}",
            refused.at().display(),
            refused.why()
        ));
    }

    let goes = WhatGoes::decided(settings.window, &whose.days_ago(now));

    let outside = took(whose, goes.outside_the_window(), remover, did);
    did.outside_the_window += outside.len();
    write_down(record, WhyLetGo::OutsideTheWindow, &outside, now, did);

    let mut for_room = Vec::new();
    for at in goes.then_for_room() {
        match disk.below_the_floor(whose.at()) {
            Ok(true) => {}
            Ok(false) => break,
            Err(why) => {
                did.said.push(format!(
                    "{} was not asked how much room it has, so nothing went for room: {why}",
                    whose.at().display()
                ));
                break;
            }
        }
        for_room.extend(took(whose, std::slice::from_ref(at), remover, did));
    }
    did.for_room += for_room.len();
    write_down(record, WhyLetGo::TheDiskNeededTheRoom, &for_room, now, did);
}

/// Remove these kept turns of this person's, and answer with the ones that
/// really went.
fn took(
    whose: &Whose,
    which: &[usize],
    remover: &dyn Removing,
    did: &mut WhatItDid,
) -> Vec<Forgone> {
    let mut went = Vec::new();
    for at in which {
        let Some(found) = whose.kept().get(*at) else {
            continue;
        };
        match remover.remove(found.at()) {
            Ok(()) => {
                if let Some(forgone) = Forgone::of(found.turn().taken(), found.turn().did()) {
                    went.push(forgone);
                }
            }
            Err(why) => did.said.push(why.to_string()),
        }
    }
    went
}

/// Write down what went, for one reason.
fn write_down(
    record: &mut dyn WritingItDown,
    why: WhyLetGo,
    went: &[Forgone],
    now: SystemTime,
    did: &mut WhatItDid,
) {
    let Some(entry) = Entry::let_go(why, went, now) else {
        return;
    };
    if let Err(said) = record.keep(entry) {
        did.said
            .push(format!("what was let go was not written down: {said}"));
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::changes::Changes;
    use crate::keeping::keep;
    use crate::testing::{
        A_DAY, ADisk, ARecord, ARemover, a_folder, noon, one_kept_turn, one_person as a_person,
    };
    use crate::the_disk::THE_FLOOR;
    use alo_keeping_up::HowFarBack;
    use alo_record::Happened;

    /// What the record says, as the pair a reader asks for.
    fn what_the_record_says(record: &ARecord) -> Vec<(WhyLetGo, Vec<String>)> {
        record
            .kept()
            .iter()
            .filter_map(|entry| match entry.happened() {
                Happened::LetGo { why, turns } => Some((
                    *why,
                    turns
                        .iter()
                        .map(|turn| turn.did().as_str().to_owned())
                        .collect(),
                )),
                _ => None,
            })
            .collect()
    }

    /// **A machine with no subvolumes does nothing, rather than failing every
    /// timer** — ADR 0045's sixth term, and the state of every machine
    /// installed before the filesystem was decided.
    #[test]
    fn a_machine_that_keeps_nothing_does_nothing() {
        let under = a_folder("keeps-nothing").join("undo");
        let ada = a_person(&under, "ada", "/nowhere");
        let kept = one_kept_turn(&ada, "one", noon() - A_DAY * 90, "move March.pdf");

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let swept = sweep(
            &under,
            &ADisk::that_keeps_nothing(),
            &remover,
            &mut record,
            noon(),
        );

        assert_eq!(swept, Swept::NotOnThisMachine);
        assert!(remover.asked().is_empty());
        assert!(record.kept().is_empty());
        assert!(kept.exists());
    }

    /// **What the window no longer reaches goes, and the record names those
    /// turns in the person's own words** — the whole of ADR 0045's first and
    /// second terms, on a machine with room to spare.
    #[test]
    fn what_the_window_no_longer_reaches_goes_and_is_named_in_the_record() {
        let under = a_folder("outside").join("undo");
        let settings = a_folder("outside-settings");
        let ada = a_person(&under, "ada", settings.to_str().unwrap());
        let recent = one_kept_turn(&ada, "recent", noon() - A_DAY, "move March.pdf");
        let old = one_kept_turn(&ada, "old", noon() - A_DAY * 30, "archive Old letters");

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let swept = sweep(&under, &ADisk::with_room(), &remover, &mut record, noon());

        assert_eq!(
            swept,
            Swept::Done(WhatItDid {
                outside_the_window: 1,
                for_room: 0,
                said: Vec::new(),
            })
        );
        assert!(recent.exists(), "a turn inside the window was removed");
        assert!(!old.exists(), "a turn outside the window was kept");
        assert_eq!(
            what_the_record_says(&record),
            [(
                WhyLetGo::OutsideTheWindow,
                vec!["archive Old letters".to_owned()]
            )]
        );
    }

    /// **A wider window keeps what a narrower one would have taken**, on the
    /// same machine, on the same day, with the person's own file as the only
    /// difference.
    #[test]
    fn a_wider_window_keeps_what_the_shipped_one_would_have_taken() {
        for (window, still_there) in [(None, false), (Some(HowFarBack::of(90, 50).unwrap()), true)]
        {
            let under = a_folder("window").join("undo");
            let settings = a_folder("window-settings");
            if let Some(window) = window {
                keep(&settings.join(THE_FILE), &Changes::with_window(window)).unwrap();
            }
            let ada = a_person(&under, "ada", settings.to_str().unwrap());
            let old = one_kept_turn(&ada, "old", noon() - A_DAY * 30, "archive Old letters");

            let remover = ARemover::willing();
            let mut record = ARecord::willing();
            sweep(&under, &ADisk::with_room(), &remover, &mut record, noon());

            assert_eq!(old.exists(), still_there, "{window:?}");
        }
    }

    /// **Under the floor the oldest go first, one at a time, and the machine
    /// stops the moment there is room** — ADR 0045's second term, with the
    /// record saying which turns it cost and why.
    #[test]
    fn under_the_floor_the_oldest_go_first_and_it_stops_when_there_is_room() {
        let under = a_folder("for-room").join("undo");
        let settings = a_folder("for-room-settings");
        let ada = a_person(&under, "ada", settings.to_str().unwrap());
        let oldest = one_kept_turn(&ada, "a", noon() - A_DAY * 3, "archive March");
        let middle = one_kept_turn(&ada, "b", noon() - A_DAY * 2, "move April.pdf");
        let newest = one_kept_turn(&ada, "c", noon() - A_DAY, "rename May.pdf");

        // Two gibibytes short, and each removal gives a gibibyte and a half
        // back: the oldest two go, and after the second there is room, so the
        // newest is never asked for.
        const A_GIBIBYTE: u64 = 1024 * 1024 * 1024;
        let disk = ADisk::short_of_room(THE_FLOOR - 2 * A_GIBIBYTE);
        let remover = ARemover::giving_back(disk.purse(), A_GIBIBYTE + A_GIBIBYTE / 2);
        let mut record = ARecord::willing();
        let swept = sweep(&under, &disk, &remover, &mut record, noon());

        let Swept::Done(did) = swept else {
            unreachable!("the machine keeps")
        };
        assert_eq!(did.outside_the_window(), 0);
        assert_eq!(did.for_room(), 2);
        assert!(!oldest.exists(), "the oldest was kept");
        assert!(!middle.exists(), "the second oldest was kept");
        assert!(
            newest.exists(),
            "the newest went after there was room again"
        );
        assert_eq!(remover.asked().len(), 2);
        assert_eq!(
            what_the_record_says(&record),
            [(
                WhyLetGo::TheDiskNeededTheRoom,
                vec!["archive March".to_owned(), "move April.pdf".to_owned()]
            )]
        );
    }

    /// **A machine with room loses nothing to the disk**, however many
    /// snapshots it is holding.
    #[test]
    fn a_machine_with_room_loses_nothing_to_the_disk() {
        let under = a_folder("with-room").join("undo");
        let settings = a_folder("with-room-settings");
        let ada = a_person(&under, "ada", settings.to_str().unwrap());
        let one = one_kept_turn(&ada, "a", noon() - A_DAY, "archive March");

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let swept = sweep(&under, &ADisk::with_room(), &remover, &mut record, noon());

        assert!(matches!(swept, Swept::Done(did) if !did.anything_went()));
        assert!(one.exists());
        assert!(remover.asked().is_empty());
        assert!(record.kept().is_empty());
    }

    /// **A removal the machine refused is not written down as having
    /// happened.** A machine whose unit holds no capability refuses every one
    /// of them, and an entry saying an undo is gone while its snapshot sits on
    /// the disk is the one line a person could not check.
    #[test]
    fn a_refused_removal_is_said_and_never_recorded() {
        let under = a_folder("refused").join("undo");
        let settings = a_folder("refused-settings");
        let ada = a_person(&under, "ada", settings.to_str().unwrap());
        one_kept_turn(&ada, "old", noon() - A_DAY * 30, "archive Old letters");

        let remover = ARemover::refusing("old");
        let mut record = ARecord::willing();
        let swept = sweep(&under, &ADisk::with_room(), &remover, &mut record, noon());

        assert!(matches!(swept, Swept::Done(ref did) if !did.anything_went()));
        assert!(record.kept().is_empty());
        let Swept::Done(did) = swept else {
            unreachable!("the machine keeps")
        };
        assert_eq!(did.said().len(), 1);
        assert!(did.said()[0].contains("Operation not permitted"));
    }

    /// **A record that will not take the entry is said loudly** — what it
    /// costs is the account, and the unit never carries on quietly about it.
    #[test]
    fn a_record_that_will_not_take_the_entry_is_said() {
        let under = a_folder("no-record").join("undo");
        let settings = a_folder("no-record-settings");
        let ada = a_person(&under, "ada", settings.to_str().unwrap());
        one_kept_turn(&ada, "old", noon() - A_DAY * 30, "archive Old letters");

        let remover = ARemover::willing();
        let mut record = ARecord::refusing();
        let swept = sweep(&under, &ADisk::with_room(), &remover, &mut record, noon());

        let Swept::Done(did) = swept else {
            unreachable!("the machine keeps")
        };
        assert_eq!(did.outside_the_window(), 1);
        assert_eq!(did.said().len(), 1);
        assert!(did.said()[0].contains("not written down"));
    }

    /// **A person whose own settings file will not read keeps what alo OS
    /// ships, and is told** — never a machine that quietly stopped letting go,
    /// and never one that quietly took more than the person asked.
    #[test]
    fn a_settings_file_that_did_not_read_uses_the_shipped_window_and_says_so() {
        let under = a_folder("bad-settings").join("undo");
        let settings = a_folder("bad-settings-folder");
        std::fs::write(settings.join(THE_FILE), "format = 1\nnever-expire = true\n").unwrap();
        let ada = a_person(&under, "ada", settings.to_str().unwrap());
        let inside = one_kept_turn(&ada, "inside", noon() - A_DAY, "move March.pdf");
        let outside = one_kept_turn(&ada, "outside", noon() - A_DAY * 30, "archive Old");

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let swept = sweep(&under, &ADisk::with_room(), &remover, &mut record, noon());

        let Swept::Done(did) = swept else {
            unreachable!("the machine keeps")
        };
        assert_eq!(did.outside_the_window(), 1);
        assert!(inside.exists());
        assert!(!outside.exists());
        assert!(did.said().iter().any(|said| said.contains("did not read")));
    }
}
