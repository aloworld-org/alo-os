//! The level a person chose for running code, as it is written down.
//!
//! ```toml
//! format = 1
//! chose = "asking-each-time"
//! ```
//!
//! # Not choosing is a state, and it is not the sealed box
//!
//! [`alo_capability::HowCodeRuns`] has a [`Default`] and it is
//! [`InASealedBox`](alo_capability::HowCodeRuns::InASealedBox), which is the
//! right answer to *what does a machine do* and the wrong answer to *what did
//! this person decide*. `docs/features.md` says it in one line — **the default
//! is that they have not chosen** — and a surface that ticked the sealed box
//! for somebody who has never seen the screen would be showing them a default
//! dressed as their own decision.
//!
//! So **absence is the only way to say *not chosen***: no file at all, which is
//! [`crate::the_level_remembered`]'s [`NotRemembered::NotThere`]. A file that
//! exists and omits `chose` is refused ([`NotTheLevel::SaysNothing`]) rather
//! than read as nobody having chosen, because the two are different claims and
//! somebody who deleted the line by hand is owed the difference. It is also why
//! `#[serde(default)]` is not on the key: a default can be failed to override,
//! and an absence cannot be overridden wrongly.
//!
//! [`NotRemembered::NotThere`]: crate::NotRemembered::NotThere
//!
//! # It is written in the person's own words
//!
//! `in-a-sealed-box`, `asking-each-time`, `at-full-trust`.
//! `docs/contracts/how-code-runs-file.md` is a file a person may edit, and a
//! level kept as `0`, `1` or `2` is a level nobody can read in their own
//! settings — over the one question of how much of their machine an agent may
//! use.
//!
//! **One list of the three spellings, in [`the_word_for`], held to
//! `HowCodeRuns`'s own `serde` names by a test over
//! [`HowCodeRuns::ALL`](alo_capability::HowCodeRuns::ALL).** The alternative —
//! reaching the derived name through a `toml` round trip — was written first
//! and thrown away: it read the word back out by splitting on quote marks,
//! which is a parser pretending to be a lookup. A match plus a test that walks
//! every level is one list a reader can see and one check that fails when it
//! drifts.
//!
//! # A spelling this does not know is refused rather than guessed
//!
//! There is no nearest match. A file saying `full-trust` where `at-full-trust`
//! was meant is refused whole and named, because the two plausible repairs —
//! guessing the nearest level, or falling back to the default — are *choosing
//! for the person*, and law 5 gives that to nobody.

use alo_capability::HowCodeRuns;
use serde::{Deserialize, Serialize};

/// Which shape of this file this alo OS writes and reads.
pub const THE_LEVEL_FORMAT: u32 = 1;

/// The word the file uses for a sealed box.
pub const IN_A_SEALED_BOX: &str = "in-a-sealed-box";

/// The word the file uses for asking each time.
pub const ASKING_EACH_TIME: &str = "asking-each-time";

/// The word the file uses for full trust.
pub const AT_FULL_TRUST: &str = "at-full-trust";

/// Why the level a person chose was not read, or could not be written.
///
/// English, with a `Display`, for this crate's reason; and **every arm means no
/// level was read** — never a level guessed and never one defaulted.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotTheLevel {
    /// The text is not the shape this file takes.
    #[error("the level this machine keeps is not the shape it takes: {why}")]
    NotTheShape {
        /// What would not parse, in the parser's words.
        why: String,
    },
    /// Written for an alo OS this is not.
    #[error(
        "this level says format {format}, and this alo OS reads only format {}",
        THE_LEVEL_FORMAT
    )]
    AnotherFormat {
        /// The number the file says.
        format: u32,
    },
    /// A file that is there and says no level.
    ///
    /// **Not the same as no file.** No file is a person who has not chosen; a
    /// file with no `chose` in it is one somebody edited into saying nothing,
    /// and they are told rather than quietly returned to the default.
    #[error("this file says no level, and a file that is here is expected to say one")]
    SaysNothing,
    /// A level spelt in a way this does not know.
    #[error(
        "`{said}` is not a level: the three are `{IN_A_SEALED_BOX}`, `{ASKING_EACH_TIME}` and `{AT_FULL_TRUST}`"
    )]
    NoSuchLevel {
        /// What it said.
        said: String,
    },
    /// The level could not be written down.
    #[error("the level a person chose could not be written down: {why}")]
    NotWrittenDown {
        /// What the writer said.
        why: String,
    },
}

/// The file, as `serde` sees it.
///
/// `chose` is an [`Option`] so that a file omitting it parses and is then
/// refused by [`read`] with a sentence of its own, rather than failing as a
/// shape error that says nothing about what to do about it.
#[derive(Debug, Serialize, Deserialize)]
struct Written {
    /// Which shape the file is in.
    format: u32,
    /// The level, in the person's own words.
    #[serde(skip_serializing_if = "Option::is_none")]
    chose: Option<String>,
}

/// The level this text says a person chose.
///
/// # Errors
///
/// [`NotTheLevel::NotTheShape`] for text that is not this file;
/// [`NotTheLevel::AnotherFormat`] for a file from an alo OS this is not,
/// refused before the level is looked at; [`NotTheLevel::SaysNothing`] for a
/// file that declares no level; and [`NotTheLevel::NoSuchLevel`] for a spelling
/// this does not know, which is never guessed at.
pub fn read(text: &str) -> Result<HowCodeRuns, NotTheLevel> {
    let written: Written = toml::from_str(text).map_err(|why| NotTheLevel::NotTheShape {
        why: why.to_string(),
    })?;
    if written.format != THE_LEVEL_FORMAT {
        return Err(NotTheLevel::AnotherFormat {
            format: written.format,
        });
    }
    let said = written.chose.ok_or(NotTheLevel::SaysNothing)?;
    the_level_called(&said)
}

/// The level this word names.
///
/// Its own function because the person's door takes a level as a word too, and
/// one answer to *which level is this word* is better than two that could
/// disagree.
///
/// # Errors
///
/// [`NotTheLevel::NoSuchLevel`] for anything that is not one of the three.
pub fn the_level_called(said: &str) -> Result<HowCodeRuns, NotTheLevel> {
    match said {
        IN_A_SEALED_BOX => Ok(HowCodeRuns::InASealedBox),
        ASKING_EACH_TIME => Ok(HowCodeRuns::AskingEachTime),
        AT_FULL_TRUST => Ok(HowCodeRuns::AtFullTrust),
        _ => Err(NotTheLevel::NoSuchLevel {
            said: said.to_owned(),
        }),
    }
}

/// The word this file writes for a level.
#[must_use]
pub const fn the_word_for(chose: HowCodeRuns) -> &'static str {
    match chose {
        HowCodeRuns::InASealedBox => IN_A_SEALED_BOX,
        HowCodeRuns::AskingEachTime => ASKING_EACH_TIME,
        HowCodeRuns::AtFullTrust => AT_FULL_TRUST,
    }
}

/// This level, written as the file holds it.
///
/// # Errors
///
/// [`NotTheLevel::NotWrittenDown`] if the file will not serialise, which is a
/// bug in this crate rather than anything a person did.
pub fn written(chose: HowCodeRuns) -> Result<String, NotTheLevel> {
    let it = Written {
        format: THE_LEVEL_FORMAT,
        chose: Some(the_word_for(chose).to_owned()),
    };
    toml::to_string(&it).map_err(|why| NotTheLevel::NotWrittenDown {
        why: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_level_tests.rs"]
mod tests;
