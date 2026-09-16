//! One argument an adapter's verb declares, in the contract's own kinds.
//!
//! The contract's manifest names an argument's type in a word, and an adapter
//! author may write any of them — including the three that are never loaded.
//! They are kinds here rather than absent on purpose: **an adapter whose verb
//! takes a script, a command or free text is refused in words naming the verb
//! and the argument** ([`crate::NotLoaded::TakesCode`]), which is what an
//! author reading the refusal needs, and what a type that could not express the
//! mistake would never tell them.

use alo_strings::Word;

/// What one argument takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A full path on this machine. A grant must cover it.
    Path,
    /// One name, of at most this many characters.
    Name {
        /// The most characters it may be.
        longest: usize,
    },
    /// A whole number, both ends included.
    Count {
        /// The smallest.
        least: i64,
        /// The largest.
        most: i64,
    },
    /// One of the options the adapter wrote down.
    Choice(&'static [Offer]),
    /// Free text. Never loaded: text a model writes is not an argument.
    Text,
    /// A script, in the language named. Never loaded (ADR 0001 §1).
    Script {
        /// The language the application would run it in.
        language: &'static str,
    },
    /// A command line. Never loaded (ADR 0001 §1).
    Command,
}

impl Kind {
    /// What the contract calls this kind.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Name { .. } => "name",
            Self::Count { .. } => "count",
            Self::Choice(_) => "choice",
            Self::Text => "text",
            Self::Script { .. } => "script",
            Self::Command => "command",
        }
    }

    /// Whether what arrives in it is something that runs, or text a model
    /// wrote that could become something that runs.
    #[must_use]
    pub const fn is_code(self) -> bool {
        matches!(self, Self::Text | Self::Script { .. } | Self::Command)
    }
}

/// One option of a [`Kind::Choice`]: the name a model sends, and the word a
/// person reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offer {
    /// What a model sends and the record keeps.
    pub name: &'static str,
    /// What a person reads.
    pub words: Word,
}

/// One argument of an adapter's verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterArg {
    /// The name the argument arrives under.
    pub name: &'static str,
    /// What it is for, in the words a person would use.
    pub purpose: Word,
    /// What it takes.
    pub kind: Kind,
}
