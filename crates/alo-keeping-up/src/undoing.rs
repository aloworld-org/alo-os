//! Whether what the record says happened can be undone at all, and the
//! sentence a person reads when it cannot.
//!
//! ★ *Undo what the agent did* is the sharpest line in `docs/features.md`'s
//! v0.5, and the half of it that matters most is the half that says **no**. A
//! message that was sent, a question that was put to a model somewhere else, a
//! file handed to another machine are gone beyond recall, and a machine that
//! implied otherwise would be lying at the worst possible moment a person could
//! be lied to. So this file is the closed table from
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) point 1
//! as working code: for each thing the record can say happened, whether there
//! is a road back, and if there is not, **why**, in the words the person reads.
//!
//! # A verb nobody classified answers *never*
//!
//! [`Change`] is the closed list of what can be put back, and it is three
//! verbs long. [`WhatWasDone::of_a_verb`] answers
//! [`WhatWasDone::ChangedSomethingElse`] for every other change verb this
//! machine ships and for every one it gains — so a verb cannot become undoable
//! by being forgotten, which is the only way that table could quietly start
//! lying. `tests/undo_what_the_agent_did.rs` holds every verb
//! `alo-declared` ships against it.
//!
//! # What this file does not know
//!
//! **Whether this machine kept anything.** That a rename *could* be put back
//! is decided here; whether it *can be* on the machine in front of the person
//! is [`crate::putting_back`], because it depends on what the turn left behind
//! rather than on what the verb was. Until the bracket ADR 0045 chose exists on
//! a machine, every one of these three answers *not yet on this machine*, which
//! is true rather than a stub.
//!
//! **What the record is.** Nothing here names `alo_record::Happened`, for the
//! reason [`crate::before`] names no time: this crate decides and another reads
//! the record. `alo-updating` maps one onto the other, in one place, and the
//! mapping is the seam a reviewer checks.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// A change to a person's own files, of the three kinds a road could put back.
///
/// Three, and not *every change verb*: these are the ones ADR 0045's table
/// names, and they share the property that makes them undoable at all — each
/// changes where a file is or what it is called, inside a folder the person
/// granted, and none of them rewrites what is in one. A fourth is added by a
/// change that says so, never by a verb arriving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// A file was given another name.
    Renamed,
    /// A file was moved into another folder.
    Moved,
    /// A folder was made into an archive beside itself.
    Archived,
}

/// What one entry of the record says happened, as far as undoing cares.
///
/// The record's own kinds collapse onto these: what matters to an undo is not
/// which of sixteen things an entry was but whether there is anything left on
/// this machine that could be put back, and if not, which honest sentence says
/// so. Whoever reads the record maps onto this — `alo-updating` — and this
/// crate names no record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatWasDone {
    /// A verb ran that renamed, moved or archived something of the person's.
    ChangedFiles(Change),
    /// A verb ran that only answered a question.
    OnlyLooked,
    /// A verb ran that changed something else about the machine: a printer, a
    /// network, a document converted, a picture of the screen.
    ChangedSomethingElse,
    /// Something left this machine — a question put to a model elsewhere, a
    /// file handed to another machine, anything sent.
    ItLeft,
    /// A model on this machine was told something, for an agent here or for a
    /// paired machine.
    AModelWasTold,
    /// Something was printed on paper.
    ItWasPrinted,
    /// An application was opened, brought forward, arranged or asked to close.
    AnApplication,
    /// An application was installed.
    AnApplicationWasInstalled,
    /// Nothing happened: a refusal, a call that never became one, or a turn
    /// that was never run.
    NothingHappened,
    /// Not an agent's doing: the machine starting on another build, a pairing
    /// kept, a workspace opened, an errand, an undo already done.
    NotAnAgents,
}

/// Why something cannot be undone — each of them a sentence the person reads
/// **instead of** an offer, never after one.
///
/// The first nine are about what was done and hold on every machine there will
/// ever be. The last five are about the machine in front of the person: what it
/// kept, whether it still keeps it, and whether what it would put back is still
/// as the agent left it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotUndoable {
    /// It left this machine, so nothing here has it to put back.
    ItLeftThisMachine,
    /// A model was told it, and nothing untells a model.
    ItWasToldToAModel,
    /// It only looked, so nothing changed that could be put back.
    ItOnlyLooked,
    /// It was printed on paper.
    ItWasPrintedOnPaper,
    /// An application was opened, arranged or closed — not a thing the machine
    /// can put back, and a closed one may have let go of work.
    AnApplicationCannotBePutBack,
    /// An application was installed, and removing one is the person's own act
    /// (ADR 0042) rather than an undo.
    AnApplicationIsYoursToRemove,
    /// A change verb this machine keeps no road back for — every change verb
    /// but the three on [`Change`].
    NotOneOfTheChangesPutBack,
    /// Nothing happened, so there is nothing to put back.
    NothingHappened,
    /// No agent did it, so there is nothing of an agent's to undo.
    NotAnAgents,
    /// This machine keeps nothing of what a person's files were before a turn,
    /// so nothing an agent changed can be put back on it — ADR 0045's *not yet
    /// on this machine*, which is true until the bracket exists here.
    NothingKeepsWhatWasThere,
    /// The machine had no room to keep what the files were before this one
    /// turn, and ran it anyway (ADR 0045, the owner's third term).
    TheTurnWasNotKept,
    /// What was kept either side of the turn has been let go: the window
    /// passed, the disk needed the room, or the person forgot it.
    NoLongerKept,
    /// Something this would put back is no longer as the agent left it, so
    /// putting it back would undo a later change too.
    ChangedSinceItWasDone,
    /// Both sides of the turn are still kept and they are the same: the turn
    /// left nothing behind to put back.
    NothingLeftToPutBack,
}

impl Change {
    /// The verb that renames a file.
    pub const RENAME_FILE: &'static str = "rename_file";

    /// The verb that moves a file into another folder.
    pub const MOVE_FILE: &'static str = "move_file";

    /// The verb that makes an archive of a folder.
    pub const ARCHIVE_FOLDER: &'static str = "archive_folder";

    /// Every change there is a road back for.
    pub const EVERY: [Self; 3] = [Self::Renamed, Self::Moved, Self::Archived];

    /// The verb whose running is this change, by the name the record keeps.
    #[must_use]
    pub fn verb(self) -> &'static str {
        match self {
            Self::Renamed => Self::RENAME_FILE,
            Self::Moved => Self::MOVE_FILE,
            Self::Archived => Self::ARCHIVE_FOLDER,
        }
    }

    /// The change a verb makes, by its name — [`None`] for every verb that is
    /// not one of the three.
    #[must_use]
    pub fn of_a_verb(verb: &str) -> Option<Self> {
        Self::EVERY.into_iter().find(|change| change.verb() == verb)
    }
}

impl WhatWasDone {
    /// The verb that prints.
    const PRINT_DOCUMENT: &'static str = "print_document";

    /// The verb that installs an application.
    const INSTALL_APPLICATION: &'static str = "install_application";

    /// The verbs that open, arrange, front or close an application.
    const ABOUT_AN_APPLICATION: [&'static str; 4] = [
        "open_application",
        "focus_application",
        "close_application",
        "arrange_application",
    ];

    /// What a verb the record says ran did, by its name and by whether the
    /// record says it changed anything.
    ///
    /// For an entry that **ran**, and for no other kind. A call that was
    /// refused changed nothing whatever its verb was
    /// ([`WhatWasDone::NothingHappened`]), and something that left the machine
    /// is [`WhatWasDone::ItLeft`] whatever verb carried it: deciding either
    /// from a name would be reading the wrong column.
    ///
    /// `changed` is the record's own answer — `alo_capability::Effect::Change`
    /// — and it is taken rather than guessed so that **a verb this does not
    /// know still lands somewhere honest**: a new read verb only looked, and a
    /// new change verb is not one of the changes there is a road back for.
    #[must_use]
    pub fn of_a_verb(verb: &str, changed: bool) -> Self {
        if let Some(change) = Change::of_a_verb(verb) {
            return Self::ChangedFiles(change);
        }
        if verb == Self::PRINT_DOCUMENT {
            return Self::ItWasPrinted;
        }
        if verb == Self::INSTALL_APPLICATION {
            return Self::AnApplicationWasInstalled;
        }
        if Self::ABOUT_AN_APPLICATION.contains(&verb) {
            return Self::AnApplication;
        }
        if changed {
            Self::ChangedSomethingElse
        } else {
            Self::OnlyLooked
        }
    }

    /// Whether this is the kind of thing there is any road back for.
    ///
    /// The answer to *could this ever be undone*, which is about what was done
    /// and not about this machine. Whether it can be undone **here** is
    /// [`crate::AnUndo::offered`], which asks this first.
    ///
    /// # Errors
    /// [`NotUndoable`], the reason, which is the deliverable rather than a
    /// consolation for one: a refusal a person cannot read the *why* of is a
    /// system they stop trusting.
    pub fn could_be_put_back(self) -> Result<Change, NotUndoable> {
        match self {
            Self::ChangedFiles(change) => Ok(change),
            Self::OnlyLooked => Err(NotUndoable::ItOnlyLooked),
            Self::ChangedSomethingElse => Err(NotUndoable::NotOneOfTheChangesPutBack),
            Self::ItLeft => Err(NotUndoable::ItLeftThisMachine),
            Self::AModelWasTold => Err(NotUndoable::ItWasToldToAModel),
            Self::ItWasPrinted => Err(NotUndoable::ItWasPrintedOnPaper),
            Self::AnApplication => Err(NotUndoable::AnApplicationCannotBePutBack),
            Self::AnApplicationWasInstalled => Err(NotUndoable::AnApplicationIsYoursToRemove),
            Self::NothingHappened => Err(NotUndoable::NothingHappened),
            Self::NotAnAgents => Err(NotUndoable::NotAnAgents),
        }
    }
}

impl NotUndoable {
    /// Every reason there is, so a surface can show the whole table and a test
    /// can hold each one to having a sentence.
    pub const EVERY: [Self; 14] = [
        Self::ItLeftThisMachine,
        Self::ItWasToldToAModel,
        Self::ItOnlyLooked,
        Self::ItWasPrintedOnPaper,
        Self::AnApplicationCannotBePutBack,
        Self::AnApplicationIsYoursToRemove,
        Self::NotOneOfTheChangesPutBack,
        Self::NothingHappened,
        Self::NotAnAgents,
        Self::NothingKeepsWhatWasThere,
        Self::TheTurnWasNotKept,
        Self::NoLongerKept,
        Self::ChangedSinceItWasDone,
        Self::NothingLeftToPutBack,
    ];

    /// Why, in the words the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self {
            Self::ItLeftThisMachine => words::NOT_UNDONE_IT_LEFT,
            Self::ItWasToldToAModel => words::NOT_UNDONE_A_MODEL_WAS_TOLD,
            Self::ItOnlyLooked => words::NOT_UNDONE_IT_ONLY_LOOKED,
            Self::ItWasPrintedOnPaper => words::NOT_UNDONE_IT_WAS_PRINTED,
            Self::AnApplicationCannotBePutBack => words::NOT_UNDONE_AN_APPLICATION,
            Self::AnApplicationIsYoursToRemove => words::NOT_UNDONE_AN_APPLICATION_INSTALLED,
            Self::NotOneOfTheChangesPutBack => words::NOT_UNDONE_NOT_ONE_OF_THE_CHANGES,
            Self::NothingHappened => words::NOT_UNDONE_NOTHING_HAPPENED,
            Self::NotAnAgents => words::NOT_UNDONE_NOT_AN_AGENTS,
            Self::NothingKeepsWhatWasThere => words::NOT_UNDONE_NOTHING_KEEPS_WHAT_WAS_THERE,
            Self::TheTurnWasNotKept => words::NOT_UNDONE_THE_TURN_WAS_NOT_KEPT,
            Self::NoLongerKept => words::NOT_UNDONE_NO_LONGER_KEPT,
            Self::ChangedSinceItWasDone => words::NOT_UNDONE_CHANGED_SINCE,
            Self::NothingLeftToPutBack => words::NOT_UNDONE_NOTHING_LEFT,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **The three verbs there is a road back for are the three, and each is
    /// read back from its own name.**
    #[test]
    fn the_three_changes_there_is_a_road_back_for_read_back_from_their_verbs() {
        for change in Change::EVERY {
            assert_eq!(Change::of_a_verb(change.verb()), Some(change));
            assert_eq!(
                WhatWasDone::of_a_verb(change.verb(), true).could_be_put_back(),
                Ok(change)
            );
        }
        assert_eq!(Change::EVERY.len(), 3);
    }

    /// **A change verb nobody classified answers *never*, with a reason.** This
    /// is the rule that stops the table lying the day a verb is added: the
    /// default is no road back, and making one is a change that says so.
    #[test]
    fn a_change_verb_nobody_classified_is_not_undoable() {
        for verb in [
            "convert_document",
            "picture_of_the_screen",
            "add_printer",
            "join_network",
            "a_verb_from_next_year",
        ] {
            let what = WhatWasDone::of_a_verb(verb, true);
            assert_eq!(what, WhatWasDone::ChangedSomethingElse, "{verb}");
            assert_eq!(
                what.could_be_put_back(),
                Err(NotUndoable::NotOneOfTheChangesPutBack),
                "{verb}"
            );
        }
    }

    /// **A read verb nobody classified says nothing changed**, rather than
    /// saying there is no road back for something that never moved.
    #[test]
    fn a_read_verb_nobody_classified_says_nothing_changed() {
        for verb in ["read_file", "search_files", "a_question_from_next_year"] {
            let what = WhatWasDone::of_a_verb(verb, false);
            assert_eq!(what, WhatWasDone::OnlyLooked, "{verb}");
            assert_eq!(
                what.could_be_put_back(),
                Err(NotUndoable::ItOnlyLooked),
                "{verb}"
            );
        }
    }

    /// **Printing, applications and installing each have their own refusal**,
    /// because *why not* is the whole of the answer and one sentence for three
    /// different facts would answer none of them.
    #[test]
    fn printing_applications_and_installing_each_say_their_own_why() {
        assert_eq!(
            WhatWasDone::of_a_verb("print_document", true).could_be_put_back(),
            Err(NotUndoable::ItWasPrintedOnPaper)
        );
        for verb in WhatWasDone::ABOUT_AN_APPLICATION {
            assert_eq!(
                WhatWasDone::of_a_verb(verb, true).could_be_put_back(),
                Err(NotUndoable::AnApplicationCannotBePutBack),
                "{verb}"
            );
        }
        assert_eq!(
            WhatWasDone::of_a_verb("install_application", true).could_be_put_back(),
            Err(NotUndoable::AnApplicationIsYoursToRemove)
        );
    }

    /// **What left, what a model was told, what was refused and what no agent
    /// did are each refused in their own words.**
    #[test]
    fn what_left_and_what_nobody_did_are_each_refused_in_their_own_words() {
        for (what, why) in [
            (WhatWasDone::ItLeft, NotUndoable::ItLeftThisMachine),
            (WhatWasDone::AModelWasTold, NotUndoable::ItWasToldToAModel),
            (WhatWasDone::NothingHappened, NotUndoable::NothingHappened),
            (WhatWasDone::NotAnAgents, NotUndoable::NotAnAgents),
        ] {
            assert_eq!(what.could_be_put_back(), Err(why), "{what:?}");
        }
    }

    /// **Every reason has a sentence, and no two reasons read alike** — a
    /// refusal without a reason is a system a person stops trusting, and two
    /// reasons wearing one sentence is the same failure wearing a table.
    #[test]
    fn every_reason_has_its_own_sentence() {
        let strings = in_english();
        let mut said: Vec<String> = Vec::new();
        for why in NotUndoable::EVERY {
            let sentence = why.said(&strings);
            assert!(!sentence.is_a_bug(), "{why:?}: {sentence}");
            said.push(sentence.text().to_owned());
        }
        said.sort_unstable();
        let how_many = said.len();
        said.dedup();
        assert_eq!(said.len(), how_many, "two reasons read the same");
    }
}
