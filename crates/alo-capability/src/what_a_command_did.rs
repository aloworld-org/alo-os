//! **What a command alo ran did**, in the shape the record, the model and the
//! person are told it in.
//!
//! The crate that starts the program (`alo-running`, ADR 0117) makes one of
//! these, `alo-turn` writes it down, and `alo-protocol` carries it to the agent.
//! It lives here because all three already depend on this crate and none of
//! them should depend on each other for it.

/// What a command did once it was running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatACommandDid {
    /// Its exit status, and [`None`] when a signal ended it — a program killed
    /// by a signal has no exit status, and any number put here would be one
    /// nothing said.
    returned: Option<i32>,
    /// What it wrote, standard output and standard error together in the order
    /// they arrived, as text.
    said: String,
    /// Whether it wrote more than was kept.
    cut_short: bool,
    /// Whether it was stopped because it ran longer than a command is let run.
    took_too_long: bool,
}

impl WhatACommandDid {
    /// What a command did, as the crate that ran it saw it.
    #[must_use]
    pub fn of(returned: Option<i32>, said: String, cut_short: bool, took_too_long: bool) -> Self {
        Self {
            returned,
            said,
            cut_short,
            took_too_long,
        }
    }

    /// Its exit status, and [`None`] when a signal ended it.
    #[must_use]
    pub fn returned(&self) -> Option<i32> {
        self.returned
    }

    /// What it wrote.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// Whether it wrote more than was kept.
    #[must_use]
    pub fn cut_short(&self) -> bool {
        self.cut_short
    }

    /// Whether it was stopped for running too long.
    #[must_use]
    pub fn took_too_long(&self) -> bool {
        self.took_too_long
    }
}

/// A command that never started — the shell would not start, or the machine
/// lost track of it before it could say anything.
///
/// The English is for whoever administers the machine; the person is told
/// [`crate::words::NOTHING_RAN_WOULD_NOT_START`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItDidNotStart(String);

impl ItDidNotStart {
    /// Why, in English, for the service log.
    #[must_use]
    pub fn because(why: impl Into<String>) -> Self {
        Self(why.into())
    }

    /// Why, in English.
    #[must_use]
    pub fn why(&self) -> &str {
        &self.0
    }
}
