//! Every string a task window can say, and the English beside each one.
//!
//! The shape is `alo_dock::words`': constants, `alo_strings::Word`, and a test
//! at the bottom putting every key back through `Key::named`.
//!
//! **The person's own words are not here.** A task is named by what somebody
//! asked for, and what they said and what alo replied are theirs and alo's.
//! What is here is the window's own furniture: what kind of thing it is, what
//! is happening in it, the invitation in an empty field, and the sentence that
//! says how far a permission reaches.
//!
//! # Two of these are the ones to get right in translation
//!
//! [`ALO_IS_WORKING`] and [`WAITING_FOR_YOUR_PERMISSION`] are the only words
//! that say whether a person has to do something. They are read across a room,
//! at a glance, by somebody deciding whether to walk over — so a translation
//! that makes them similar to each other costs exactly the distinction they
//! exist to carry. The second is a **request**, not a status: *waiting* is the
//! machine's state and *your permission* is what it is waiting on, and dropping
//! the second half leaves a window that looks merely slow.
//!
//! # Where the processing happens is **not** said here
//!
//! A task window must always show it — law 1, and the design gives it a
//! permanent place beside the model's name. It is not in this file because
//! `alo-models` already says it: `InferenceSource::said` words all four
//! sources, fills in a provider and a region, and is the sentence the rest of
//! this machine uses.
//!
//! **A `tasks.where.this-computer` stood here for part of 2026-10-12 and was
//! removed.** It said *On this computer*, which `models.source.this-machine`
//! already said as *on this machine* — and two sentences about where a person's
//! data goes is the one duplication this product cannot afford. It would also
//! have covered only one of the four sources, so a paired machine or a provider
//! would have been drawn with the local wording or with nothing.
//!
//! # And one that is a promise rather than a label
//!
//! [`WHAT_THIS_TASK_REACHES`] — *permissions and model choices apply to this
//! task* — is the sentence that tells a person a grant here does not follow
//! them into the next task. It is drawn in the quietest ink in the window and
//! it is the most load-bearing sentence in it: somebody who reads it as *this
//! machine* rather than *this task* will allow something they did not mean to.
//! A translation must keep **this task** narrow.

use alo_strings::{Vocabulary, Word};

/// What kind of thing a task window is, under its title.
pub const A_TASK: Word = Word::saying("tasks.window.kind", "alo task").noting(
    "Under the task's title, which is the person's own request. It says what kind of window \
     this is — one where alo is working on something — and is not a status: **what is \
     happening** is a separate line in the same row.",
);

/// Alo is working on it.
pub const ALO_IS_WORKING: Word = Word::saying("tasks.status.working", "alo is working").noting(
    "The status while alo is acting, drawn in the teal ADR 0067 reserves for exactly that. \
     Read at a glance from across a room by somebody deciding whether they are needed, so it \
     must stay clearly different from `tasks.status.waiting` — a translation that makes the \
     two similar costs the distinction they exist for.",
);

/// Alo has stopped and is waiting for the person.
pub const WAITING_FOR_YOUR_PERMISSION: Word =
    Word::saying("tasks.status.waiting", "Waiting for your permission").noting(
        "**A request, not a status.** *Waiting* is what the machine is doing and *your \
         permission* is what it waits on; a translation that keeps only the first half leaves \
         a window that looks merely slow, and somebody will walk away from a task that needed \
         them. Not drawn in teal: teal is alo acting, and this is alo stopped.",
    );

/// The task has stopped.
pub const STOPPED: Word = Word::saying("tasks.status.stopped", "Stopped").noting(
    "After a stop the runtime has confirmed. **Only after** — never when somebody has asked \
     and the work has not ended, which is `tasks.status.stopping`.",
);

/// A stop has been asked for and has not been confirmed.
pub const STOPPING: Word = Word::saying("tasks.status.stopping", "Stopping\u{2026}").noting(
    "Between a person pressing Stop and the runtime confirming. **The ellipsis carries the \
     whole meaning**: this is not *stopped*, and a translation that drops the sense of \
     something still in progress would tell a person work had ended while it was still \
     running. Keep whatever marks continuing in the target language.",
);

/// The task has finished.
pub const COMPLETE: Word = Word::saying("tasks.status.complete", "Complete")
    .noting("The task finished on its own. Not drawn in teal: alo is no longer acting.");

/// The invitation in an empty reply field.
pub const REPLY_TO_ALO: Word = Word::saying(
    "tasks.composer.invitation",
    "Reply to alo or change the direction\u{2026}",
)
.noting(
    "The grey words in the reply field before anybody has typed. **Two things a person \
             can do, and both matter**: answer what alo said, and change what it is doing. A \
             translation shortened to *Reply* loses the second, which is the one somebody \
             reaches for when the work is going the wrong way.",
);

/// How far the choices in this window reach.
pub const WHAT_THIS_TASK_REACHES: Word = Word::saying(
    "tasks.window.scope",
    "Permissions and model choices apply to this task.",
)
.noting(
    "The quietest sentence in the window and the most load-bearing. It tells a person that \
     allowing something here does not follow them into the next task. **Keep *this task* \
     narrow**: somebody who reads it as *this machine* will allow something they did not mean \
     to.",
);

/// Every word this crate can say.
pub const EVERY_WORD: [Word; 8] = [
    A_TASK,
    ALO_IS_WORKING,
    WAITING_FOR_YOUR_PERMISSION,
    STOPPED,
    STOPPING,
    COMPLETE,
    REPLY_TO_ALO,
    WHAT_THIS_TASK_REACHES,
];

/// Why this crate's words did not go into a vocabulary.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// For a caller counting what each crate contributes, which is how
/// `alo-saying` holds its own collection to the sum of its parts.
///
/// # Errors
/// [`WordsError`] if any of these words is not a phrase, which would be a
/// mistake in this file rather than anything a caller did.
pub fn task_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The shell has one vocabulary and every crate adds its own to it, which is
/// what the area at the front of a key is for.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "words_tests.rs"]
mod tests;
