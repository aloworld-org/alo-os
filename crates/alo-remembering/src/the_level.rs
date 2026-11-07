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
//!
//! # A second answer in the same file: alo in the terminal
//!
//! *Added 2026-11-06, additively.* `docs/features.md` lets a person choose that
//! what they ask alo for in the terminal runs without the OK — ADR 0064's full
//! trust, applied to the terminal — and that answer is kept here, beside the
//! level, under the same rules about who may have written it:
//!
//! ```toml
//! format = 1
//! chose = "asking-each-time"
//! in-the-terminal = "at-full-trust"
//! ```
//!
//! **Its words are two of the level's**: `asking-each-time` is the switch off
//! and `at-full-trust` is it on. `in-a-sealed-box` means nothing for a request
//! typed at a prompt and is refused, as any other word is.
//!
//! **Each key is its own answer, and either may be absent.** A file holding
//! only `in-the-terminal` is a person who switched the terminal on and has not
//! chosen a level for running code, and is read as exactly that.
//! [`NotTheLevel::SaysNothing`] is now a file saying **neither** — still a file
//! somebody edited into saying nothing, which is the case it was for.
//!
//! # A key nobody declared is refused
//!
//! `docs/contracts/how-code-runs-file.md` has said so since it was written, and
//! until 2026-11-06 nothing did it: `serde` reads around a key it was not told
//! about. With a second key that matters, because a person who wrote
//! `in-the-termnal` by hand would have been told nothing and left asking. So
//! the file is read strictly — after the format is checked on its own, so that
//! a file from a later alo OS is still refused as *not for me* rather than as a
//! complaint about a key that alo OS defines.

use alo_capability::{AloInTheTerminal, HowCodeRuns};
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
    /// A file that is there and says no level, and nothing about the
    /// terminal either.
    ///
    /// **Not the same as no file.** No file is a person who has not chosen; a
    /// file with neither `chose` nor `in-the-terminal` in it is one somebody
    /// edited into saying nothing, and they are told rather than quietly
    /// returned to the default.
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
    /// `in-the-terminal` says a word that is neither of its two.
    ///
    /// Refused whole, level and all, for [`Self::NoSuchLevel`]'s reason:
    /// reading the half that parsed would be guessing what the rest meant.
    #[error(
        "`{said}` is not an answer for the terminal: the two are `{ASKING_EACH_TIME}` and `{AT_FULL_TRUST}`"
    )]
    NotATerminalAnswer {
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
/// Both answers are an [`Option`] so that a file omitting one parses and is
/// read as that answer not given, and a file omitting both is refused by
/// [`read`] with a sentence of its own rather than failing as a shape error
/// that says nothing about what to do about it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// Which shape the file is in.
    format: u32,
    /// The level, in the person's own words.
    #[serde(skip_serializing_if = "Option::is_none")]
    chose: Option<String>,
    /// Whether alo in the terminal acts without the OK, in two of the level's
    /// words.
    #[serde(rename = "in-the-terminal", skip_serializing_if = "Option::is_none")]
    in_the_terminal: Option<String>,
}

/// Only the format, read first and leniently, so that a file from a later alo
/// OS is refused for being one before its keys are judged.
#[derive(Debug, Deserialize)]
struct OnlyTheFormat {
    /// Which shape the file is in.
    format: u32,
}

/// Everything the file says: the level a person chose for running code, and
/// whether alo in the terminal acts without the OK.
///
/// **Either may be absent, and absence is never filled in here.** No `chose`
/// is a person who has not chosen a level; no `in-the-terminal` is the switch
/// as a machine arrives, which is off. Which default an absence stands for is
/// the reader's to say, at the one place each is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WhatTheFileSays {
    /// The level, when one was chosen.
    pub chose: Option<HowCodeRuns>,
    /// The terminal's answer, when one was given.
    pub in_the_terminal: Option<AloInTheTerminal>,
}

/// What this text says.
///
/// # Errors
///
/// [`NotTheLevel::NotTheShape`] for text that is not this file, including a
/// key this file does not declare; [`NotTheLevel::AnotherFormat`] for a file
/// from an alo OS this is not, refused before anything else is looked at;
/// [`NotTheLevel::SaysNothing`] for a file that says neither answer;
/// [`NotTheLevel::NoSuchLevel`] and [`NotTheLevel::NotATerminalAnswer`] for a
/// spelling this does not know, which is never guessed at.
pub fn read(text: &str) -> Result<WhatTheFileSays, NotTheLevel> {
    let shape = |why: toml::de::Error| NotTheLevel::NotTheShape {
        why: why.to_string(),
    };
    let only: OnlyTheFormat = toml::from_str(text).map_err(shape)?;
    if only.format != THE_LEVEL_FORMAT {
        return Err(NotTheLevel::AnotherFormat {
            format: only.format,
        });
    }
    let written: Written = toml::from_str(text).map_err(shape)?;
    if written.chose.is_none() && written.in_the_terminal.is_none() {
        return Err(NotTheLevel::SaysNothing);
    }
    Ok(WhatTheFileSays {
        chose: written
            .chose
            .as_deref()
            .map(the_level_called)
            .transpose()?,
        in_the_terminal: written
            .in_the_terminal
            .as_deref()
            .map(the_terminal_answer_called)
            .transpose()?,
    })
}

/// The terminal's answer this word names.
///
/// # Errors
///
/// [`NotTheLevel::NotATerminalAnswer`] for anything but the two.
fn the_terminal_answer_called(said: &str) -> Result<AloInTheTerminal, NotTheLevel> {
    match said {
        ASKING_EACH_TIME => Ok(AloInTheTerminal::AsksFirst),
        AT_FULL_TRUST => Ok(AloInTheTerminal::ActsWithoutAsking),
        _ => Err(NotTheLevel::NotATerminalAnswer {
            said: said.to_owned(),
        }),
    }
}

/// The word this file writes for the terminal's answer.
const fn the_terminal_word_for(answer: AloInTheTerminal) -> &'static str {
    match answer {
        AloInTheTerminal::AsksFirst => ASKING_EACH_TIME,
        AloInTheTerminal::ActsWithoutAsking => AT_FULL_TRUST,
    }
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

/// What the file is to say, written as the file holds it.
///
/// # Errors
///
/// [`NotTheLevel::NotWrittenDown`] if the file will not serialise, which is a
/// bug in this crate rather than anything a person did.
pub fn written(says: WhatTheFileSays) -> Result<String, NotTheLevel> {
    let it = Written {
        format: THE_LEVEL_FORMAT,
        chose: says.chose.map(|chose| the_word_for(chose).to_owned()),
        in_the_terminal: says
            .in_the_terminal
            .map(|answer| the_terminal_word_for(answer).to_owned()),
    };
    toml::to_string(&it).map_err(|why| NotTheLevel::NotWrittenDown {
        why: why.to_string(),
    })
}

impl NotTheLevel {
    /// The sentence a person reads for this refusal.
    ///
    /// Every arm has one, because every arm of this enum is reachable from the
    /// Settings surface — unlike `crate::NotRemembered`, whose grants arms are
    /// read by a daemon into a log. `crate::words` has the argument.
    #[must_use]
    pub const fn word(&self) -> alo_strings::Word {
        match self {
            Self::NotTheShape { .. } => crate::words::LEVEL_NOT_THE_SHAPE,
            Self::AnotherFormat { .. } => crate::words::LEVEL_FROM_A_NEWER_ALO_OS,
            Self::SaysNothing => crate::words::LEVEL_SAYS_NOTHING,
            Self::NoSuchLevel { .. } => crate::words::LEVEL_NO_SUCH_LEVEL,
            Self::NotATerminalAnswer { .. } => crate::words::LEVEL_NOT_A_TERMINAL_ANSWER,
            Self::NotWrittenDown { .. } => crate::words::LEVEL_NOT_WRITTEN_DOWN,
        }
    }

    /// What this refusal puts into its sentence besides the path.
    ///
    /// Only [`NoSuchLevel`](Self::NoSuchLevel) and
    /// [`NotATerminalAnswer`](Self::NotATerminalAnswer) have anything: the word the file
    /// actually said, quoted back exactly as written, because it is data and is
    /// never translated. The parser's own words and the format numbers are
    /// deliberately **not** carried — they name rows in a file most people did
    /// not open, and a person acts on *this file is not for me* rather than on
    /// a column number.
    #[must_use]
    pub fn filling_beyond_the_path(&self) -> Option<(&'static str, String)> {
        match self {
            Self::NoSuchLevel { said } | Self::NotATerminalAnswer { said } => {
                Some(("said", said.clone()))
            }
            Self::NotTheShape { .. }
            | Self::AnotherFormat { .. }
            | Self::SaysNothing
            | Self::NotWrittenDown { .. } => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_level_tests.rs"]
mod tests;
