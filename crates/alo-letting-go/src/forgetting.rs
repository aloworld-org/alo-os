//! One act a person asked for, whole: take what was asked, work out whose it
//! is, remove everything that person's machine was keeping for them, and write
//! down which turns can no longer be put back.
//!
//! [`crate::sweep`] is the machine's own housekeeping — a window and a disk,
//! firing off a timer. This is the other half of
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s point
//! 5: *forgetting it is **one act**.* The two share the remover, the record and
//! the folder, and they share nothing else — this one is started by a person and
//! obeys nobody's window.
//!
//! # The order, and why it is this one
//!
//! 1. **Take every asking, first and whatever happens next.**
//!    [`crate::asking::taken`] has both reasons: an approval that survives being
//!    acted on can be acted on again, and an asking left behind starts the unit
//!    again.
//! 2. **Is this a machine that keeps anything?** ADR 0045's sixth term, and the
//!    same answer [`crate::Swept::NotOnThisMachine`] gives — *not yet on this
//!    machine*, rather than a machine that reports having forgotten something it
//!    never had.
//! 3. **Whose act is it?** The owner of the settings folder that person's own
//!    session wrote down, matched against the user the filesystem says left the
//!    asking. A person nobody asked for is walked past untouched.
//! 4. **Everything goes**, whatever the window says and whatever the disk says.
//!    That is what the person approved: not the oldest, not the expired — all of
//!    it, in one act, because a machine that made them reclaim their disk a turn
//!    at a time would be keeping it by attrition.
//! 5. **What went is written down**, named rather than counted, in the words the
//!    person approved at the time, under a reason of its own so that a person
//!    reading their record can tell *you asked for this* from *your machine
//!    tidied up*.
//!
//! # Nothing is recorded that did not happen
//!
//! [`crate::sweep`]'s rule exactly, and for the same reason: a removal the
//! machine refused leaves that turn out of the entry and says so to the journal.
//! A record saying an undo is gone while its snapshot sits on the disk is the
//! one line in this file a person could not check.
//!
//! # The window is not touched, and could not be
//!
//! Forgetting is **not** a window of nought. `alo_keeping_up::HowFarBack`
//! refuses one by name, and this act never asks it anything: what a person
//! approved is everything, once, and their window is exactly what it was
//! afterwards — so the next changing turn is kept again.

use std::path::Path;
use std::time::SystemTime;

use alo_record::{Entry, Forgone, WhyLetGo};

use crate::asking;
use crate::found::{Everyone, Whose};
use crate::removing::Removing;
use crate::the_disk::{AskingTheDisk, WhatItIs};
use crate::whose::WhoOwns;
use crate::writing_it_down::WritingItDown;

/// What one asking did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Forgotten {
    /// This machine keeps nothing an undo could put back, so there was nothing
    /// to forget — ADR 0045's sixth term, said rather than pretended.
    NotOnThisMachine,
    /// It read what was asked and did what follows.
    Done(WhatWasForgotten),
}

/// What an act that ran actually did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WhatWasForgotten {
    /// How many people's machines forgot what they were keeping.
    people: usize,
    /// How many kept turns went altogether.
    turns: usize,
    /// Everything it would not or could not do, in English, for the journal.
    said: Vec<String>,
}

impl WhatWasForgotten {
    /// How many people's machines forgot what they were keeping — nought when
    /// nobody asked, or when nobody who asked has anything kept.
    #[must_use]
    pub const fn people(&self) -> usize {
        self.people
    }

    /// How many kept turns went altogether.
    #[must_use]
    pub const fn turns(&self) -> usize {
        self.turns
    }

    /// Whether anything at all went.
    #[must_use]
    pub const fn anything_went(&self) -> bool {
        self.turns > 0
    }

    /// Everything for the journal.
    #[must_use]
    pub fn said(&self) -> &[String] {
        &self.said
    }
}

/// One act, over the kept-undo folder at `folder` and the askings left in
/// `asking`.
///
/// `now` is handed in rather than read, for [`crate::sweep`]'s reason: one act
/// is measured against one moment rather than however many the walk took.
pub fn forget(
    folder: &Path,
    asking: &Path,
    disk: &dyn AskingTheDisk,
    owner: &dyn WhoOwns,
    remover: &dyn Removing,
    record: &mut dyn WritingItDown,
    now: SystemTime,
) -> Forgotten {
    let asked = asking::taken(asking, owner, now);
    let mut did = WhatWasForgotten {
        said: asked.said().to_vec(),
        ..WhatWasForgotten::default()
    };

    match disk.what_it_is(folder) {
        Ok(WhatItIs::OneThatKeeps) => {}
        Ok(WhatItIs::NotOneThatKeeps) => return Forgotten::NotOnThisMachine,
        Err(why) => {
            // A machine that cannot say what its own filesystem is removes
            // nothing: the alternative is removing a person's history on a
            // guess.
            did.said
                .push(format!("{} was not asked about: {why}", folder.display()));
            return Forgotten::Done(did);
        }
    }
    if asked.nobody() {
        return Forgotten::Done(did);
    }

    let everyone = Everyone::under(folder);
    did.said.extend(everyone.stepped_over().iter().cloned());
    for whose in everyone.whose() {
        let theirs = match owner.of(whose.settings()) {
            Ok(theirs) => theirs,
            Err(why) => {
                // Nothing is forgotten for somebody the machine cannot say the
                // ownership of: whose act this is, is the whole of what decides
                // whose history goes.
                did.said.push(format!(
                    "{} was not asked who owns it, so nothing of theirs was forgotten: {why}",
                    whose.settings().display()
                ));
                continue;
            }
        };
        if !asked.by().contains(&theirs) {
            continue;
        }
        one_person(whose, remover, record, now, &mut did);
    }
    Forgotten::Done(did)
}

/// Everything one person's machine was keeping for them, forgotten.
fn one_person(
    whose: &Whose,
    remover: &dyn Removing,
    record: &mut dyn WritingItDown,
    now: SystemTime,
    did: &mut WhatWasForgotten,
) {
    let mut went = Vec::new();
    // Oldest first, as the record reads best and as everything else in this
    // crate removes: the kept turns come back newest first.
    for found in whose.kept().iter().rev() {
        match remover.remove(found.at()) {
            Ok(()) => {
                if let Some(forgone) = Forgone::of(found.turn().taken(), found.turn().did()) {
                    went.push(forgone);
                }
            }
            Err(why) => did.said.push(why.to_string()),
        }
    }
    if went.is_empty() {
        return;
    }
    did.people += 1;
    did.turns += went.len();
    let Some(entry) = Entry::let_go(WhyLetGo::ThePersonAskedToForget, &went, now) else {
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
    use crate::asking::ask;
    use crate::testing::{
        A_DAY, ADisk, ARecord, ARemover, AnOwner, a_folder, noon, one_kept_turn,
        one_person as a_person,
    };
    use alo_record::Happened;
    use std::path::PathBuf;

    /// Ada, with three kept turns, and a folder her session leaves an asking
    /// in. Two of her turns are inside any window and one is outside every
    /// one — the act takes no notice of either.
    fn a_machine_ada_keeps_on(what: &str) -> (PathBuf, PathBuf, Vec<PathBuf>) {
        let under = a_folder(what).join("undo");
        let settings = a_folder(&format!("{what}-settings"));
        let ada = a_person(&under, "ada", settings.to_str().unwrap());
        let kept = vec![
            one_kept_turn(&ada, "a", noon() - A_DAY * 30, "archive Old letters"),
            one_kept_turn(&ada, "b", noon() - A_DAY * 2, "move April.pdf"),
            one_kept_turn(&ada, "c", noon(), "rename May.pdf"),
        ];
        (under, a_folder(&format!("{what}-asked")), kept)
    }

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

    /// **One act forgets every turn, not one at a time and not one folder at a
    /// time** — ADR 0045 point 5, which says so by name: a person who wants the
    /// space back wants it back.
    #[test]
    fn one_act_forgets_every_turn_and_not_one_at_a_time() {
        let (under, asked, kept) = a_machine_ada_keeps_on("one-act");
        ask(&asked, noon()).unwrap();

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let forgotten = forget(
            &under,
            &asked,
            &ADisk::with_room(),
            &AnOwner::everything_owned_by(1000),
            &remover,
            &mut record,
            noon(),
        );

        let Forgotten::Done(did) = forgotten else {
            unreachable!("the machine keeps")
        };
        assert_eq!(did.people(), 1);
        assert_eq!(did.turns(), 3);
        assert!(did.said().is_empty(), "{:?}", did.said());
        for one in &kept {
            assert!(!one.exists(), "{} was kept", one.display());
        }
        assert_eq!(record.kept().len(), 1, "one act, one entry");
    }

    /// **The record names the turns in the person's own words, under a reason
    /// of its own** — never a number, and told apart from the window's and the
    /// disk's, because *you asked for this* and *your machine tidied up* are
    /// the one difference a person acts on.
    #[test]
    fn the_record_names_the_turns_in_the_persons_words_and_its_own_reason() {
        let (under, asked, _) = a_machine_ada_keeps_on("named");
        ask(&asked, noon()).unwrap();

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        forget(
            &under,
            &asked,
            &ADisk::with_room(),
            &AnOwner::everything_owned_by(1000),
            &remover,
            &mut record,
            noon(),
        );

        assert_eq!(
            what_the_record_says(&record),
            [(
                WhyLetGo::ThePersonAskedToForget,
                vec![
                    "archive Old letters".to_owned(),
                    "move April.pdf".to_owned(),
                    "rename May.pdf".to_owned(),
                ]
            )]
        );
        assert!(WhyLetGo::ThePersonAskedToForget.is_the_persons_own());
    }

    /// **One approval causes exactly one execution.** The asking is taken before
    /// anything is removed, so the same act run again removes nothing, asks the
    /// remover nothing and writes nothing down.
    #[test]
    fn one_approval_causes_exactly_one_forgetting() {
        let (under, asked, _) = a_machine_ada_keeps_on("once");
        let left = ask(&asked, noon()).unwrap();

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let owner = AnOwner::everything_owned_by(1000);
        let disk = ADisk::with_room();

        forget(&under, &asked, &disk, &owner, &remover, &mut record, noon());
        assert!(!left.exists(), "the approval outlived being acted on");
        let after_one = remover.asked().len();
        assert_eq!(after_one, 3);
        assert_eq!(record.kept().len(), 1);

        let again = forget(&under, &asked, &disk, &owner, &remover, &mut record, noon());
        assert_eq!(
            again,
            Forgotten::Done(WhatWasForgotten::default()),
            "a second execution followed one approval"
        );
        assert_eq!(remover.asked().len(), after_one);
        assert_eq!(record.kept().len(), 1);
    }

    /// **Nobody can ask for somebody else's undo to be forgotten.** Ada asks;
    /// Bo's machine keeps everything it was keeping, and the record names only
    /// Ada's turns. The asking names nobody, so this is not a refusal that had
    /// to be got right — there is nothing in it to aim.
    #[test]
    fn nobody_can_ask_for_somebody_elses_undo_to_be_forgotten() {
        let under = a_folder("not-theirs").join("undo");
        let ada_settings = a_folder("not-theirs-ada");
        let bo_settings = a_folder("not-theirs-bo");
        let ada = a_person(&under, "ada", ada_settings.to_str().unwrap());
        let bo = a_person(&under, "bo", bo_settings.to_str().unwrap());
        let hers = one_kept_turn(&ada, "one", noon(), "move March.pdf");
        let his = one_kept_turn(&bo, "one", noon(), "archive Old letters");

        let asked = a_folder("not-theirs-asked");
        ask(&asked, noon()).unwrap();

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let forgotten = forget(
            &under,
            &asked,
            &ADisk::with_room(),
            // Ada wrote the asking; the settings folders are each their own
            // person's, which is the only thing that decides whose act it is.
            &AnOwner::these(&[
                (ada_settings.as_path(), 1000),
                (bo_settings.as_path(), 1001),
            ])
            .and_everything_else(1000),
            &remover,
            &mut record,
            noon(),
        );

        let Forgotten::Done(did) = forgotten else {
            unreachable!("the machine keeps")
        };
        assert_eq!(did.people(), 1);
        assert_eq!(did.turns(), 1);
        assert!(!hers.exists(), "the person who asked kept their undo");
        assert!(his.exists(), "somebody else's undo was forgotten");
        assert_eq!(
            what_the_record_says(&record),
            [(
                WhyLetGo::ThePersonAskedToForget,
                vec!["move March.pdf".to_owned()]
            )]
        );
    }

    /// **A machine that keeps nothing says so rather than pretending it forgot
    /// something** — ADR 0045's sixth term, and the state of every machine
    /// installed before the filesystem was decided.
    #[test]
    fn a_machine_that_keeps_nothing_says_so_instead_of_pretending() {
        let (under, asked, kept) = a_machine_ada_keeps_on("keeps-nothing");
        let left = ask(&asked, noon()).unwrap();

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let forgotten = forget(
            &under,
            &asked,
            &ADisk::that_keeps_nothing(),
            &AnOwner::everything_owned_by(1000),
            &remover,
            &mut record,
            noon(),
        );

        assert_eq!(forgotten, Forgotten::NotOnThisMachine);
        assert!(remover.asked().is_empty());
        assert!(record.kept().is_empty());
        assert!(kept[0].exists());
        assert!(
            !left.exists(),
            "an approval was left on a machine that could not act on it"
        );
    }

    /// **An asking nobody left forgets nothing**, which is the ordinary state of
    /// a unit somebody started by hand.
    #[test]
    fn an_act_nobody_asked_for_forgets_nothing() {
        let (under, asked, kept) = a_machine_ada_keeps_on("unasked");

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let forgotten = forget(
            &under,
            &asked,
            &ADisk::with_room(),
            &AnOwner::everything_owned_by(1000),
            &remover,
            &mut record,
            noon(),
        );

        assert_eq!(forgotten, Forgotten::Done(WhatWasForgotten::default()));
        assert!(remover.asked().is_empty());
        assert!(kept[0].exists());
    }

    /// **A removal the machine refused is not written down as having
    /// happened** — a machine whose unit holds no capability refuses every one
    /// of them, and an entry saying an undo is gone while its snapshot sits on
    /// the disk is the one line a person could not check.
    #[test]
    fn a_refused_removal_is_said_and_never_recorded() {
        let (under, asked, _) = a_machine_ada_keeps_on("refused");
        ask(&asked, noon()).unwrap();

        let remover = ARemover::refusing_everything();
        let mut record = ARecord::willing();
        let forgotten = forget(
            &under,
            &asked,
            &ADisk::with_room(),
            &AnOwner::everything_owned_by(1000),
            &remover,
            &mut record,
            noon(),
        );

        let Forgotten::Done(did) = forgotten else {
            unreachable!("the machine keeps")
        };
        assert!(!did.anything_went());
        assert_eq!(did.people(), 0);
        assert!(record.kept().is_empty());
        assert_eq!(did.said().len(), 3);
        assert!(
            did.said()
                .iter()
                .all(|said| said.contains("Operation not permitted"))
        );
    }

    /// **Nothing is forgotten for somebody the machine cannot say the ownership
    /// of**, because whose act this is, is the whole of what decides whose
    /// history goes.
    #[test]
    fn a_person_whose_folder_cannot_be_asked_about_keeps_everything() {
        let (under, asked, kept) = a_machine_ada_keeps_on("no-owner");
        ask(&asked, noon()).unwrap();

        let remover = ARemover::willing();
        let mut record = ARecord::willing();
        let forgotten = forget(
            &under,
            &asked,
            &ADisk::with_room(),
            &AnOwner::refusing_folders(),
            &remover,
            &mut record,
            noon(),
        );

        let Forgotten::Done(did) = forgotten else {
            unreachable!("the machine keeps")
        };
        assert!(!did.anything_went());
        assert!(kept[0].exists());
        assert!(
            did.said()
                .iter()
                .any(|said| said.contains("was not asked who owns it"))
        );
    }

    /// **A record that will not take the entry is said loudly** — what it costs
    /// is the account, and the act never carries on quietly about it.
    #[test]
    fn a_record_that_will_not_take_the_entry_is_said() {
        let (under, asked, _) = a_machine_ada_keeps_on("no-record");
        ask(&asked, noon()).unwrap();

        let remover = ARemover::willing();
        let mut record = ARecord::refusing();
        let forgotten = forget(
            &under,
            &asked,
            &ADisk::with_room(),
            &AnOwner::everything_owned_by(1000),
            &remover,
            &mut record,
            noon(),
        );

        let Forgotten::Done(did) = forgotten else {
            unreachable!("the machine keeps")
        };
        assert_eq!(did.turns(), 3);
        assert!(
            did.said()
                .iter()
                .any(|said| said.contains("not written down"))
        );
    }
}
