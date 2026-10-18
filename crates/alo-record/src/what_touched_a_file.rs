//! From *an agent asked what was done to a file* to the answer, out of the
//! record this crate already keeps.
//!
//! `docs/features.md`, starred: *Find a command by what it did. The command
//! that made the PDF… **where nothing connects them it says so** rather than
//! inventing a link.* That last clause is the whole shape of this file.
//!
//! # What this answers, and what it cannot
//!
//! [`Asking::touching`] matches a path against the **recorded arguments** of a
//! call, which means the answer is every verb call that named this file.
//!
//! **A command the person typed is not in the answer, and that is not a
//! defect that can be fixed here.** `Entry::ran_a_command` records that a
//! command ran and the folders the kernel *let it reach* — which is
//! **permitted, not changed**. Answering *what changed this file* out of
//! *where it was allowed to write* would be this repository's oldest fault
//! family in new clothes: a check that stands in for the thing.
//!
//! So an empty answer means **the agent did nothing to this file**, and never
//! that nothing did. A person cannot be left to infer which, so
//! [`WhatTouchedIt::and_only_what_alo_did`] is on every answer and no shape of
//! the type can leave it out — the same rule, and for the same reason, as
//! `alo_terminal`'s *nobody checked this* on a test result.
//!
//! # Why a grant is required over the file
//!
//! The verb is the **agent's** road to the record, not the person's: a person
//! reads the record in the record window, which shows all of it and refuses to
//! narrow by design. An agent that could ask about any path would learn
//! whether a path exists in the record by asking about a guess, which is a
//! real answer about a place nobody granted it. So the grant is over the file
//! the question names, and revoking that folder stops this question the same
//! instant it stops `read_file`.
//!
//! That is a narrowing of what a person can see, not of what they are told:
//! the window is unchanged, and nothing an agent is refused here is hidden
//! from the person whose machine it is.

use alo_strings::Word;

use crate::entry::Entry;
use crate::explain::Asking;
use crate::record::Record;
use crate::words;

/// One thing the record says was done to the file.
///
/// Everything here was worded when it happened, by whoever did it, in the
/// language the person was told it in — so nothing in this type is translated
/// again. [`crate::Line`] is that already-worded sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneThingDone {
    /// When it happened.
    at: std::time::SystemTime,
    /// The sentence the person was shown for it.
    said: crate::Line,
    /// Whether it changed the machine or only answered.
    effect: alo_capability::Effect,
}

impl OneThingDone {
    /// When the record says it happened.
    #[must_use]
    pub const fn at(&self) -> std::time::SystemTime {
        self.at
    }

    /// The sentence the person was shown at the time.
    ///
    /// **Not re-worded here.** A record that said a thing one way on the day
    /// and another way a month later would be two records.
    #[must_use]
    pub const fn said(&self) -> &crate::Line {
        &self.said
    }

    /// Whether it changed the machine or only answered a question.
    ///
    /// The distinction a person actually wants when they ask what happened to
    /// a file: a `read_file` named it and changed nothing.
    #[must_use]
    pub const fn effect(&self) -> alo_capability::Effect {
        self.effect
    }
}

/// What the record says was done to one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatTouchedIt {
    /// The file the question was about, as it was asked.
    file: String,
    /// What was found, oldest first.
    found: Vec<OneThingDone>,
}

impl WhatTouchedIt {
    /// The file the question was about, as the call named it.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// What the record says was done to it, oldest first.
    ///
    /// **Oldest first**, which is the order the record is written in and the
    /// order a person reading a history expects. Empty is an ordinary answer —
    /// see [`Self::and_nothing_names_it`].
    #[must_use]
    pub fn found(&self) -> &[OneThingDone] {
        &self.found
    }

    /// That the record names nothing, where it names nothing.
    ///
    /// [`None`] when something was found. This is the feature's own *where
    /// nothing connects them it says so*: an empty list drawn as an empty list
    /// reads as *nothing happened*, which is a claim this answer cannot make.
    #[must_use]
    pub fn and_nothing_names_it(&self) -> Option<Word> {
        self.found
            .is_empty()
            .then_some(words::NOTHING_IN_THE_RECORD_TOUCHED_IT)
    }

    /// That this is what alo did, which no shape of this type omits.
    ///
    /// Takes no argument, so a caller cannot forget it: see this module's
    /// header for what it is protecting a person from concluding.
    #[must_use]
    pub const fn and_only_what_alo_did(&self) -> Word {
        words::ONLY_WHAT_ALO_DID
    }
}

/// Why the question could not be put to the record.
///
/// Neither can be reached from outside: `alo_capability` validated the call
/// against the declaration before it became an [`Authorised`], and
/// [`crate::verbs`]' own tests say the declaration takes a path called `file`.
/// It is a `Result` because a library inside the daemon reporting its own
/// mistake must not take the daemon down with it.
///
/// [`Authorised`]: alo_capability::Authorised
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum NotAsked {
    /// An authority for some other verb.
    #[error("{verb} is not the record's verb")]
    AnotherVerb {
        /// What was asked for instead.
        verb: String,
    },
    /// The declared argument was not a path.
    #[error("{argument} was not given as a path")]
    Malformed {
        /// Which argument.
        argument: &'static str,
    },
}

impl NotAsked {
    /// What to say about it, as a word and the gaps it needs filled.
    ///
    /// Worded here rather than at the caller so that the sentence and the
    /// refusal cannot drift apart.
    #[must_use]
    pub fn said(&self) -> (Word, Vec<(&'static str, String)>) {
        match self {
            Self::AnotherVerb { verb } => (words::NOT_THIS_CRATES, vec![("verb", verb.clone())]),
            Self::Malformed { argument } => (
                words::MISSING,
                vec![
                    ("verb", THE_VERB.to_owned()),
                    ("argument", (*argument).to_owned()),
                ],
            ),
        }
    }
}

/// The name of the one verb this crate offers.
///
/// A constant rather than a literal in three files: a verb routed by one
/// spelling here and another in the executor is routed nowhere, and the
/// compiler cannot see a mistyped string.
pub const THE_VERB: &str = "what_touched_this_file";

/// What the argument is called.
pub const THE_FILE: &str = "file";

/// The file an authorised call of this verb asks about.
///
/// **The only reader of the call**, so that an executor cannot form a second
/// opinion about what arrived.
///
/// # Errors
/// [`NotAsked`], which a validated call of this verb cannot cause.
pub fn asked_about(authorised: &alo_capability::Authorised) -> Result<&std::path::Path, NotAsked> {
    if authorised.verb() != THE_VERB {
        return Err(NotAsked::AnotherVerb {
            verb: authorised.verb().to_owned(),
        });
    }
    let Some(alo_capability::Value::Path(file)) = authorised.call().value(THE_FILE) else {
        return Err(NotAsked::Malformed { argument: THE_FILE });
    };
    Ok(file.as_path())
}

/// What a record says was done to this file.
///
/// The whole of the question, and it is three lines: the path becomes an
/// [`Asking`], the record answers it, and each entry becomes one line of the
/// answer. Nothing here reads a disk — a caller hands over the record it
/// already holds.
#[must_use]
pub fn what_touched(record: &Record, file: &str) -> WhatTouchedIt {
    let asking = Asking::anything().touching(file);
    let found = record
        .answering(&asking)
        .filter_map(one_thing_done)
        .collect();
    WhatTouchedIt {
        file: file.to_owned(),
        found,
    }
}

/// One entry as a line of the answer, where it is one.
///
/// [`None`] for an entry with no [`crate::What`] — a pairing, an update, a
/// command that ran. Those cannot have matched `touching` in the first place,
/// which matches on a call's arguments, so this skips rather than refuses:
/// an entry that cannot describe a call is not evidence of one.
fn one_thing_done(entry: &Entry) -> Option<OneThingDone> {
    let what = entry.what()?;
    Some(OneThingDone {
        at: entry.at(),
        said: what.sentence().clone(),
        effect: what.effect(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err, or an index past a length the \
              test just asserted, is the failure being reported"
)]
#[path = "what_touched_a_file_tests.rs"]
mod tests;
