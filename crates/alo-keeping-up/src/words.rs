//! Every string this crate can say, and the English beside each one.
//!
//! Where the machine stands, when an update applies, the promise an update
//! keeps, applying one, and going back to the version before. Every sentence is about the person's machine and
//! what they choose, never about what is underneath it.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the base's tooling, not a deployment, not a digest, not an image, not a
//! registry (`docs/features.md`: *the system updates; it does not "pull an
//! image"*). And nothing says *urgent*, *critical* or *required*, because this
//! machine has no update that is any of those. A test at the bottom of this file
//! reads every sentence for all of them.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Where the machine stands — `crate::Standing`.
// ---------------------------------------------------------------------------

/// An update is ready.
pub const READY: Word = Word::saying(
    "keeping-up.ready",
    "An update is ready. It will apply when you choose, and nothing you are doing will be \
     interrupted until then",
)
.noting(
    "Said when this machine has found a newer version of its own system. The update never \
     restarts the machine or closes anything by itself: the person decides when it applies, \
     usually the next time they restart. The sentence must keep both promises — that it waits for \
     the person, and that nothing is interrupted.",
);

/// An update is ready, and nothing has vouched for it.
///
/// The other half of [`READY`], said in its place when nobody has vouched for
/// the build offered (`crate::Vouching`). It is said **before** the person
/// chooses rather than after, because a person told *an update is ready* and
/// then told *it could not be prepared* has been told two true things and
/// learned nothing.
pub const READY_NOT_VOUCHED_FOR: Word = Word::saying(
    "keeping-up.ready-not-vouched-for",
    "A new version of this machine's system is available, but this machine cannot confirm that it \
     came from alo OS. It will not be applied unless it can be confirmed",
)
.noting(
    "Said instead of *an update is ready* when a newer version of alo OS exists but nothing \
     alongside it proves who made it. The machine will not install a version it cannot confirm \
     came from alo OS, and this sentence is shown before the person is offered the choice rather \
     than as a refusal afterwards. It is not an accusation that anything is wrong — most often \
     the proof has simply not been published yet — so it must not read as a warning about an \
     attack.",
);

/// Nothing to update.
pub const UP_TO_DATE: Word = Word::saying("keeping-up.up-to-date", "This machine is up to date")
    .noting("Said when the person checks for an update and there is none to apply.");

/// The answer about updates could not be understood.
pub const ANSWER_NOT_UNDERSTOOD: Word = Word::saying(
    "keeping-up.answer-not-understood",
    "The answer about whether there is an update could not be understood, so nothing on this \
     machine has changed",
)
.noting(
    "Said when this machine asked whether there is an update and the reply did not make sense to \
     it. The important half is the second: the person's machine is exactly as it was.",
);

// ---------------------------------------------------------------------------
// When it applies — `crate::WhenItApplies`.
// ---------------------------------------------------------------------------

/// The choice to apply it at the next restart.
pub const APPLY_AT_THE_NEXT_RESTART: Word = Word::saying(
    "keeping-up.when.at-the-next-restart",
    "Apply it the next time I restart",
)
.noting(
    "One of two choices a person has once an update is ready, written as the person speaking. \
     Chosen when nothing else has been: the update waits, and applies the next time the person \
     restarts this machine themselves.",
);

/// The choice to restart now and apply it.
pub const RESTART_AND_APPLY_NOW: Word = Word::saying(
    "keeping-up.when.now",
    "Restart now and apply it, which closes the applications that are open",
)
.noting(
    "The other of the two choices, written as an instruction the person gives. It says that open \
     applications close, because restarting closes them — the person is choosing that, and must \
     know it before they do.",
);

// ---------------------------------------------------------------------------
// The promise — `crate::THE_RULE`.
// ---------------------------------------------------------------------------

/// An update never restarts the machine.
pub const NEVER_RESTARTS: Word = Word::saying(
    "keeping-up.never.restarts",
    "An update never restarts this machine. It applies when you restart",
)
.noting(
    "The first of three promises shown together where updates are described in Settings. They are \
     promises, not settings: none of them can be turned off, and nothing overrides them.",
);

/// An update never closes an application.
pub const NEVER_CLOSES_AN_APPLICATION: Word = Word::saying(
    "keeping-up.never.closes-an-application",
    "An update never closes an application you have open",
)
.noting("The second of the three promises about updates.");

/// An update never interrupts the person.
pub const NEVER_INTERRUPTS: Word = Word::saying(
    "keeping-up.never.interrupts",
    "An update never interrupts what you are doing",
)
.noting(
    "The third of the three promises about updates. Interrupting means taking the person away \
     from their work: covering it, taking the keyboard, or asking something they did not start.",
);

// ---------------------------------------------------------------------------
// Applying it — `crate::Staging`, `crate::NotStaged`, `crate::Deployments`.
// ---------------------------------------------------------------------------

/// The update is waiting for the person's next restart.
pub const WAITING_FOR_THE_RESTART: Word = Word::saying(
    "keeping-up.waiting-for-the-restart",
    "The update will apply the next time you restart. Your files and settings stay as they are",
)
.noting(
    "Said once the person has chosen to apply an update at their next restart and the machine has \
     prepared it. Nothing happens until the person restarts. The second sentence is a promise the \
     machine keeps: an update never changes the person's own files, settings or history.",
);

/// The update was not prepared because the machine changed since it was found.
pub const CHANGED_SINCE_IT_WAS_FOUND: Word = Word::saying(
    "keeping-up.changed-since-it-was-found",
    "This machine changed after the update was found, so nothing was changed. Check for the \
     update again",
)
.noting(
    "Said when the person chose to apply an update, but the system running on the machine is no \
     longer the one the update was found for — for example because another update was applied in \
     the meantime. The machine is exactly as it was, and checking again finds the right update.",
);

/// The update the person chose is already waiting.
pub const ALREADY_WAITING: Word = Word::saying(
    "keeping-up.already-waiting",
    "This update is already waiting for your next restart",
)
.noting(
    "Said when the person chooses to apply an update that the machine has already prepared. \
     Nothing is done twice.",
);

/// The two versions named were the same one, so there was no update to stage.
///
/// The one refusal [`crate::Staging::of`] never needed: a [`crate::Ready`] is
/// made only by `Standing::between`, which answers *up to date* when the two
/// builds match, so a `Ready` holding one build twice cannot exist. An
/// approval that arrived from somewhere else can hold exactly that, which is
/// why it is refused rather than staged — telling the base to change a machine
/// to what it is already meant to be running is not an update, and a road that
/// quietly did nothing would leave a person waiting for a restart that changes
/// nothing.
pub const NOT_AN_UPDATE: Word = Word::saying(
    "keeping-up.not-an-update",
    "This machine was asked to change its system from one version to that same version, so there \
     was nothing to do and nothing was changed",
)
.noting(
    "Said when something asked this machine to update from a version of alo OS to the very same \
     version, which is not an update at all — most often because whatever asked was working from \
     information that had gone out of date. It is not a mistake of the person's, and the \
     important half, as in every other refusal here, is that nothing was changed.",
);

/// Which version of its system the machine runs could not be read.
pub const RUNNING_NOT_KNOWN: Word = Word::saying(
    "keeping-up.running-not-known",
    "Which version of its system this machine is running could not be read, so nothing was \
     changed",
)
.noting(
    "Said when the machine could not find out which version of alo OS it is running, so it could \
     not safely prepare or record an update. The machine is exactly as it was.",
);

/// The update could not be prepared.
pub const NOT_PREPARED: Word = Word::saying(
    "keeping-up.not-prepared",
    "The update could not be prepared, so nothing was changed. The next restart starts this \
     machine as it is now",
)
.noting(
    "Said when the machine tried to prepare an update the person chose and could not — for \
     example because the download did not finish. The important half is that the machine is \
     unchanged and will start normally. There is a separate sentence for the other reason an \
     update is not prepared, that the machine could not confirm the version came from alo OS, \
     because a person who is told only that something failed learns nothing they can act on.",
);

/// The update was refused because the machine could not confirm it is alo OS.
///
/// Its own sentence rather than [`NOT_PREPARED`]. Until this existed, a
/// download that did not finish and a version that could not be shown to be
/// alo OS read as one line, which told a person neither — and of the two, the
/// second is the one they can do something about and the one a machine sold on
/// sovereignty cannot afford to be vague about.
pub const NOT_GENUINE: Word = Word::saying(
    "keeping-up.not-genuine",
    "This machine could not confirm that the new version came from alo OS, so it was not \
     installed and nothing was changed. The next restart starts this machine as it is now",
)
.noting(
    "Said when the person chose to apply an update and the machine refused it because it could \
     not confirm who made that version. It is the strongest promise this machine keeps about its \
     own system: it would rather stay as it is than install something it cannot confirm. The \
     important half, as in every other refusal here, is that nothing was changed.",
);

/// Whether the machine updated could not be written down.
pub const NOT_WRITTEN_DOWN: Word = Word::saying(
    "keeping-up.not-written-down",
    "This machine could not write down whether it started on an updated version of its system. \
     Nothing else was changed",
)
.noting(
    "Said when the machine starts and cannot keep, in its own history, the fact that it has just \
     been updated — for example because the place that fact is kept could not be read or written. \
     The machine itself is working; what is missing is the line in its history.",
);

// ---------------------------------------------------------------------------
// Going back — `crate::GoingBack`, `crate::CannotGoBack`, `crate::Returning`.
// ---------------------------------------------------------------------------

/// Going back is offered.
pub const GOING_BACK_OFFERED: Word = Word::saying(
    "keeping-up.going-back.offered",
    "This machine can go back to the version of its system it ran before its last update. Your \
     files and your own settings stay as they are. Accounts, passwords and settings for the whole \
     machine that changed since that update go back to how they were",
)
.noting(
    "The sentence a person approves before their machine returns to the earlier version of alo OS. \
     Every part of it is a promise the machine keeps and must survive translation: the person's own \
     files and personal settings are not touched, but changes made since the update to user \
     accounts, passwords and settings that apply to everyone on the machine do not come back with \
     it.",
);

/// Going back is offered, and sets aside an update that is waiting.
pub const GOING_BACK_SETS_ASIDE_AN_UPDATE: Word = Word::saying(
    "keeping-up.going-back.sets-aside-an-update",
    "This machine can go back to the version of its system it ran before its last update. Your \
     files and your own settings stay as they are. Accounts, passwords and settings for the whole \
     machine that changed since that update go back to how they were. The update waiting for \
     your next restart will not apply",
)
.noting(
    "The same sentence as the offer to go back, said when the person had also chosen a newer update \
     that is waiting for their next restart. Going back cancels that waiting update, and the last \
     sentence tells them so before they approve.",
);

/// The choice to go back at the next restart.
pub const GO_BACK_AT_THE_NEXT_RESTART: Word = Word::saying(
    "keeping-up.going-back.at-the-next-restart",
    "Go back the next time I restart",
)
.noting(
    "One of two choices once going back to the earlier version is offered, written as the person \
     speaking. Nothing happens until the person restarts the machine themselves.",
);

/// The choice to restart now and go back.
pub const RESTART_AND_GO_BACK_NOW: Word = Word::saying(
    "keeping-up.going-back.now",
    "Restart now and go back, which closes the applications that are open",
)
.noting(
    "The other of the two choices, written as an instruction the person gives. It says that open \
     applications close, because restarting closes them.",
);

/// Going back is waiting for the person's next restart.
pub const GOING_BACK_AT_THE_RESTART: Word = Word::saying(
    "keeping-up.going-back.waiting-for-the-restart",
    "This machine will go back to the version it ran before the next time you restart. Your files \
     stay as they are",
)
.noting(
    "Said once the person has chosen to go back and the machine has prepared it. Nothing happens \
     until the person restarts.",
);

/// There is nothing to go back to.
pub const NOTHING_TO_GO_BACK_TO: Word = Word::saying(
    "keeping-up.going-back.nothing-before",
    "This machine has no earlier version of its system to go back to",
)
.noting(
    "Said instead of offering to go back, when this machine has only ever run the version it runs \
     now — for example before its first update.",
);

/// The version before is no longer kept.
pub const NO_LONGER_KEPT: Word = Word::saying(
    "keeping-up.going-back.no-longer-kept",
    "The version of its system this machine ran before is no longer kept on it, so it cannot go \
     back to it",
)
.noting(
    "Said instead of offering to go back, when the machine knows which version it ran before but no \
     longer has a copy of it. It is said before anything is offered, so nothing is started that \
     could fail halfway.",
);

/// Going back is already waiting.
pub const ALREADY_GOING_BACK: Word = Word::saying(
    "keeping-up.going-back.already-waiting",
    "This machine will already go back to the version it ran before the next time you restart",
)
.noting(
    "Said when the person chooses to go back and has already chosen it. Nothing is done twice.",
);

/// The machine changed after going back was offered.
pub const CHANGED_SINCE_GOING_BACK_WAS_OFFERED: Word = Word::saying(
    "keeping-up.going-back.changed-since-it-was-offered",
    "This machine changed after going back was offered, so nothing was changed. Look again at \
     what it can go back to",
)
.noting(
    "Said when the person approved going back, but the machine is no longer as it was when that was \
     offered — for example an update was prepared in the meantime. What they approved would not be \
     what happens, so nothing is done.",
);

/// Going back could not be prepared.
pub const GOING_BACK_NOT_PREPARED: Word = Word::saying(
    "keeping-up.going-back.not-prepared",
    "Going back could not be prepared, so nothing was changed. The next restart starts this \
     machine as it is now",
)
.noting(
    "Said when the person chose to go back and the machine could not prepare it. The important half \
     is that the machine is unchanged and will start normally.",
);

// ---------------------------------------------------------------------------
// Putting back what an agent did — `crate::AnUndo`, `crate::NotUndoable`,
// `crate::WhatWasKept`, `crate::HowFarBack`.
// ---------------------------------------------------------------------------

/// Putting back what an agent changed, offered.
pub const UNDO_OFFERED: Word = Word::saying(
    "keeping-up.undo.offered",
    "Put back {what}, as it was before the agent changed it on {when}",
)
.noting(
    "The sentence a person approves before their machine puts their own files back as they were \
     before the agent changed them — the agent being the assistant built into alo OS and not a \
     person. {what} is the names of the files and folders that change back, one after another \
     with a comma between them; {when} is the day and time of the change, already written in the \
     person's own language. Approving this sentence puts those back and nothing else.",
);

/// It left this machine.
pub const NOT_UNDONE_IT_LEFT: Word = Word::saying(
    "keeping-up.undo.it-left",
    "This left your machine, and nothing here can call it back",
)
.noting(
    "Said instead of offering to put something back, when what the person is looking at is \
     something that went somewhere else — a question put to a model elsewhere, a file handed to \
     another machine, anything sent. It is gone beyond this machine's reach, and saying anything \
     softer would be a promise the machine cannot keep.",
);

/// A model was told it.
pub const NOT_UNDONE_A_MODEL_WAS_TOLD: Word = Word::saying(
    "keeping-up.undo.a-model-was-told",
    "A model was told this, and nothing can untell it",
)
.noting(
    "Said instead of offering to put something back, when what the person is looking at is a \
     question that was answered. A model here means the language model that answers on this \
     machine.",
);

/// It only looked.
pub const NOT_UNDONE_IT_ONLY_LOOKED: Word = Word::saying(
    "keeping-up.undo.it-only-looked",
    "This only looked at something, so nothing changed that could be put back",
)
.noting(
    "Said instead of offering to put something back, when what the person is looking at is the \
     agent reading or searching rather than changing anything — the agent being the assistant \
     built into alo OS and not a person. Nothing is wrong: there is simply nothing to put back.",
);

/// It was printed on paper.
pub const NOT_UNDONE_IT_WAS_PRINTED: Word = Word::saying(
    "keeping-up.undo.it-was-printed",
    "This was printed on paper, which your machine cannot take back",
)
.noting(
    "Said instead of offering to put something back, when what the person is looking at is a \
     document that was printed.",
);

/// An application is not something the machine puts back.
pub const NOT_UNDONE_AN_APPLICATION: Word = Word::saying(
    "keeping-up.undo.an-application",
    "Opening, arranging or closing an application is not something your machine puts back, and an \
     application that closed may have let go of what was open in it",
)
.noting(
    "Said instead of offering to put something back, when what the person is looking at is the \
     agent having opened, brought forward, arranged or closed an application — the agent being \
     the assistant built into alo OS and not a person. The second half is the important one: \
     reopening an application would not bring back what was unsaved in it, so the machine does \
     not pretend that reopening it is putting anything back.",
);

/// Removing an application is the person's own act.
pub const NOT_UNDONE_AN_APPLICATION_INSTALLED: Word = Word::saying(
    "keeping-up.undo.an-application-installed",
    "Removing an application is yours to do, where you install and remove them, rather than \
     something put back here",
)
.noting(
    "Said instead of offering to put something back, when what the person is looking at is an \
     application the agent installed. Removing it is a deliberate act the person makes in the \
     place applications are managed, and the machine will not do it as a side effect of undoing \
     something.",
);

/// Not one of the changes the machine puts back.
pub const NOT_UNDONE_NOT_ONE_OF_THE_CHANGES: Word = Word::saying(
    "keeping-up.undo.not-one-of-the-changes",
    "This is not one of the changes your machine can put back",
)
.noting(
    "Said instead of offering to put something back, for a change the machine has no way of \
     returning — setting up a printer, joining a network, and anything else that is not a file of \
     the person's being renamed, moved or archived. It is the honest answer for anything the \
     machine has not been taught to return, including something added after this was written.",
);

/// Nothing happened.
pub const NOT_UNDONE_NOTHING_HAPPENED: Word = Word::saying(
    "keeping-up.undo.nothing-happened",
    "Nothing happened here, so there is nothing to put back",
)
.noting(
    "Said instead of offering to put something back, when what the person is looking at is \
     something that was refused, or never happened at all. The line is in their machine's history \
     because a refusal is worth keeping, not because anything changed.",
);

/// No agent did it.
pub const NOT_UNDONE_NOT_AN_AGENTS: Word = Word::saying(
    "keeping-up.undo.not-an-agents",
    "No agent did this, so there is nothing of an agent's to put back",
)
.noting(
    "Said instead of offering to put something back, when what the person is looking at is \
     something the machine itself did — the agent being the assistant built into alo OS and not a \
     person. Returning to an earlier version of the system, for one, is its own thing a person \
     chooses elsewhere.",
);

/// This machine keeps nothing of what a person's files were.
pub const NOT_UNDONE_NOTHING_KEEPS_WHAT_WAS_THERE: Word = Word::saying(
    "keeping-up.undo.nothing-keeps-what-was-there",
    "This machine does not keep what your files were before the agent changed them, so it cannot \
     put them back",
)
.noting(
    "Said instead of offering to put something back, on a machine whose disk was set up without \
     the means to hold an earlier state of a folder — the agent being the assistant built into \
     alo OS and not a person. It is the plain truth rather than a fault: the machine says what it \
     cannot do instead of offering something that would fail.",
);

/// There was no room to keep it for this one change.
pub const NOT_UNDONE_THE_TURN_WAS_NOT_KEPT: Word = Word::saying(
    "keeping-up.undo.the-turn-was-not-kept",
    "There was not enough room to keep what your files were before this change, so it cannot be \
     put back",
)
.noting(
    "Said instead of offering to put something back, when the machine was too short of space to \
     hold what the files were before this one change. The work the person asked for still \
     happened — the machine does not stop working to protect the ability to put things back — and \
     this line says what that cost.",
);

/// What was kept has been let go.
pub const NOT_UNDONE_NO_LONGER_KEPT: Word = Word::saying(
    "keeping-up.undo.no-longer-kept",
    "What your files were before this change is no longer kept, so it cannot be put back",
)
.noting(
    "Said instead of offering to put something back, when the machine did keep what the files \
     were and has since let it go — because the change is older than the machine keeps, because \
     the disk needed the room, or because the person chose to forget it.",
);

/// Something has changed since.
pub const NOT_UNDONE_CHANGED_SINCE: Word = Word::saying(
    "keeping-up.undo.changed-since",
    "Something this would put back has changed since, so putting it back would take away that \
     newer change too",
)
.noting(
    "Said instead of offering to put something back, when what would be returned is no longer as \
     the agent left it — the person edited it, or something later moved it. The machine refuses \
     the whole thing rather than returning part of it: a person's own newer work is never taken \
     away to undo an older change.",
);

/// Nothing is left to put back.
pub const NOT_UNDONE_NOTHING_LEFT: Word = Word::saying(
    "keeping-up.undo.nothing-left",
    "Nothing of this change is left to put back",
)
.noting(
    "Said instead of offering to put something back, when the machine compared how the person's \
     files were before and after the change and found them the same. Nothing was lost; there is \
     simply nothing to return.",
);

/// Forgetting everything that could be put back.
pub const FORGETTING_WHAT_CAN_BE_PUT_BACK: Word = Word::saying(
    "keeping-up.undo.forgetting",
    "Forget what your files were before the changes the agent made, so that none of them can be \
     put back afterwards. This frees the space it is holding",
)
.noting(
    "The sentence a person approves to let go of everything their machine is holding so that \
     changes can be put back — the agent being the assistant built into alo OS and not a person. \
     It is one act covering all of it, and both halves have to survive translation: nothing can \
     be put back afterwards, and the space comes back.",
);

/// How far back an undo reaches has to reach something.
pub const NOT_A_WINDOW: Word = Word::saying(
    "keeping-up.undo.not-a-window",
    "How far back changes can be put back has to be at least one day and at least one change, so \
     this was not kept",
)
.noting(
    "Said when a person, or the organisation that manages their machine, set how far back changes \
     can be put back to nothing at all. The setting is unchanged. To hold nothing, the person \
     forgets what is held, which is a separate act they are shown the cost of.",
);

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 44] = [
    READY,
    READY_NOT_VOUCHED_FOR,
    UP_TO_DATE,
    ANSWER_NOT_UNDERSTOOD,
    APPLY_AT_THE_NEXT_RESTART,
    RESTART_AND_APPLY_NOW,
    NEVER_RESTARTS,
    NEVER_CLOSES_AN_APPLICATION,
    NEVER_INTERRUPTS,
    WAITING_FOR_THE_RESTART,
    CHANGED_SINCE_IT_WAS_FOUND,
    ALREADY_WAITING,
    NOT_AN_UPDATE,
    RUNNING_NOT_KNOWN,
    NOT_PREPARED,
    NOT_GENUINE,
    NOT_WRITTEN_DOWN,
    GOING_BACK_OFFERED,
    GOING_BACK_SETS_ASIDE_AN_UPDATE,
    GO_BACK_AT_THE_NEXT_RESTART,
    RESTART_AND_GO_BACK_NOW,
    GOING_BACK_AT_THE_RESTART,
    NOTHING_TO_GO_BACK_TO,
    NO_LONGER_KEPT,
    ALREADY_GOING_BACK,
    CHANGED_SINCE_GOING_BACK_WAS_OFFERED,
    GOING_BACK_NOT_PREPARED,
    UNDO_OFFERED,
    NOT_UNDONE_IT_LEFT,
    NOT_UNDONE_A_MODEL_WAS_TOLD,
    NOT_UNDONE_IT_ONLY_LOOKED,
    NOT_UNDONE_IT_WAS_PRINTED,
    NOT_UNDONE_AN_APPLICATION,
    NOT_UNDONE_AN_APPLICATION_INSTALLED,
    NOT_UNDONE_NOT_ONE_OF_THE_CHANGES,
    NOT_UNDONE_NOTHING_HAPPENED,
    NOT_UNDONE_NOT_AN_AGENTS,
    NOT_UNDONE_NOTHING_KEEPS_WHAT_WAS_THERE,
    NOT_UNDONE_THE_TURN_WAS_NOT_KEPT,
    NOT_UNDONE_NO_LONGER_KEPT,
    NOT_UNDONE_CHANGED_SINCE,
    NOT_UNDONE_NOTHING_LEFT,
    FORGETTING_WHAT_CAN_BE_PUT_BACK,
    NOT_A_WINDOW,
];

/// The one sentence here with anything to fill in.
///
/// Every other word this crate says is whole: an update, going back and a
/// refusal each say the same thing whatever machine reads them. The offer to
/// put something back is the exception, and has to be — a person approving it
/// is approving **these** files and **that** moment, and a sentence that named
/// neither would be a person agreeing to something they were not told.
pub const THE_ONE_WITH_GAPS: Word = UNDO_OFFERED;

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// say so — and [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn keeping_up_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every key is a key, in this crate's area, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "keeping-up", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = keeping_up_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note, and one of them has gaps.**
    ///
    /// Nothing this crate says names a build, a place or a time, so there is
    /// nothing to fill — except the offer to put files back, which a person
    /// cannot check without being told which files and when. Those two gaps are
    /// named here, so a third arriving in any sentence fails this rather than
    /// quietly putting `{}` in front of somebody.
    #[test]
    fn every_word_carries_a_note_and_only_the_offer_to_put_back_has_gaps() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            let phrase = word.phrase().unwrap();
            let mut gaps: Vec<&str> = phrase.source().gaps().iter().map(String::as_str).collect();
            gaps.sort_unstable();
            if word.named() == THE_ONE_WITH_GAPS.named() {
                assert_eq!(gaps, ["what", "when"], "{}", word.named());
            } else {
                assert!(gaps.is_empty(), "{} has a gap", word.named());
            }
        }
    }

    /// **Nothing here names the machinery, hedges, or calls an update urgent.**
    #[test]
    fn nothing_here_names_the_machinery_hedges_or_calls_an_update_urgent() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "bootc",
                "ostree",
                "deployment",
                "digest",
                "image",
                "registry",
                "container",
                "pull",
                "sha256",
                "reboot",
                "rollback",
                "roll back",
                "rolled back",
                "snapshot",
                "urgent",
                "critical",
                "required",
                "must ",
                "forced",
                "probably",
                "might",
                "perhaps",
                "possibly",
            ] {
                assert!(
                    !said.contains(forbidden),
                    "{} says \"{forbidden}\"",
                    word.named()
                );
            }
        }
    }
}
