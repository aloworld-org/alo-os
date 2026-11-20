//! **What each request has reached**, per level: the ratchet (the owner,
//! 2026-11-20). *A request that has reached 10/10 at a level may not fall
//! below 10/10 there.*
//!
//! The record is one committed file, `what-each-request-has-reached.txt`:
//! a line `reached <level> <request>` for each request that took the road on
//! every one of at least the bar's tries (`crate::the_bar`) of a measured
//! run at that level. **It is raised only from a measured run**: the every-try
//! check writes what it measured (`ALO_EVERY_TRY_MEASURED`, kept as the
//! run's `every-try-measured` artifact), and
//! `cargo run -p alo-every-try -- reached <measured file>...` adds what took
//! every try. **It is never lowered**: the check refuses a change whose record
//! lacks an entry its base had, for a request still in the set. And an own
//! request with an entry at a level is held to every try there
//! (`crate::counted::counted_with`).

use std::collections::BTreeSet;

use crate::a_try::TheLevel;
use crate::tallied::Tallied;
use crate::the_bar::THE_BAR;

/// The head of the file.
const THE_HEAD: &str = "\
# What each of alo's practice requests has reached, per level: each request
# that took the road on every try of a measured run (crates/alo-every-try,
# `what_each_has_reached`). A request here is held to every try at that level
# when a change owns it, and no change may take its line away.
#
# Raised only from a measured run: download a run's every-try-measured
# artifact and run `cargo run -p alo-every-try -- reached <file>...`.
";

/// The record's file name, in this crate's directory.
pub const THE_FILE: &str = "what-each-request-has-reached.txt";

/// The record as committed.
pub const AS_COMMITTED: &str = include_str!("../what-each-request-has-reached.txt");

/// A record or a measurement that could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{what} does not read, at line {line}: {why}")]
pub struct NotRead {
    /// What was being read.
    what: &'static str,
    /// The line, counted from one.
    line: usize,
    /// What is wrong with it.
    why: String,
}

/// The name a level is written by.
#[must_use]
pub const fn the_name_of(level: TheLevel) -> &'static str {
    match level {
        TheLevel::FullAccess => "full-access",
        TheLevel::AskEachTime => "ask-each-time",
    }
}

/// The level written `named`.
#[must_use]
pub fn the_level_named(named: &str) -> Option<TheLevel> {
    [TheLevel::FullAccess, TheLevel::AskEachTime]
        .into_iter()
        .find(|level| the_name_of(*level) == named)
}

/// The lines of `text` that are not comments or blank, each with its number.
fn the_lines(text: &str) -> impl Iterator<Item = (usize, Vec<&str>)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|(at, line)| (at + 1, line.split('\t').collect()))
}

/// A level's name, as read: refused when it names no level.
fn a_level(named: &str, what: &'static str, line: usize) -> Result<String, NotRead> {
    the_level_named(named)
        .map(|level| the_name_of(level).to_owned())
        .ok_or_else(|| NotRead {
            what,
            line,
            why: format!("`{named}` is not a level"),
        })
}

/// What one run measured at one level: each request, how many tries took the
/// road, and of how many.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Measured {
    /// Each request: its level's name, its name, on the road, tried.
    each: Vec<(String, String, usize, usize)>,
}

impl Measured {
    /// What `tallied` measured at `level`.
    #[must_use]
    pub fn of(level: TheLevel, tallied: &[Tallied]) -> Self {
        Self {
            each: tallied
                .iter()
                .map(|one| {
                    (
                        the_name_of(level).to_owned(),
                        one.request().named().to_owned(),
                        one.on_the_road(),
                        one.tries().len(),
                    )
                })
                .collect(),
        }
    }

    /// As its file holds it.
    #[must_use]
    pub fn written(&self) -> String {
        self.each
            .iter()
            .map(|(level, named, on, of)| format!("measured\t{level}\t{named}\t{on}\t{of}\n"))
            .collect()
    }

    /// What `text` holds.
    ///
    /// # Errors
    /// [`NotRead`], for a line that is not a measurement.
    pub fn read(text: &str) -> Result<Self, NotRead> {
        const WHAT: &str = "what was measured";
        let mut each = Vec::new();
        for (line, fields) in the_lines(text) {
            let refused = |why: &str| NotRead {
                what: WHAT,
                line,
                why: why.to_owned(),
            };
            let ["measured", level, named, on, of] = fields.as_slice() else {
                return Err(refused(
                    "a line is `measured`, a level, a request and two counts",
                ));
            };
            let count = |said: &str| {
                said.parse::<usize>()
                    .map_err(|_| refused("a count is a whole number"))
            };
            each.push((
                a_level(level, WHAT, line)?,
                (*named).to_owned(),
                count(on)?,
                count(of)?,
            ));
        }
        Ok(Self { each })
    }
}

/// The record: each level and request that took the road on every try of a
/// measured run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Reached {
    /// Each level's name and request.
    at: BTreeSet<(String, String)>,
}

impl Reached {
    /// What `text` holds.
    ///
    /// # Errors
    /// [`NotRead`], for a line that is not an entry, a level that is not one,
    /// or an entry written twice.
    pub fn read(text: &str) -> Result<Self, NotRead> {
        const WHAT: &str = "the record of what each request has reached";
        let mut at = BTreeSet::new();
        for (line, fields) in the_lines(text) {
            let refused = |why: &str| NotRead {
                what: WHAT,
                line,
                why: why.to_owned(),
            };
            let ["reached", level, named] = fields.as_slice() else {
                return Err(refused("a line is `reached`, a level and a request"));
            };
            if !at.insert((a_level(level, WHAT, line)?, (*named).to_owned())) {
                return Err(refused("an entry is written twice"));
            }
        }
        Ok(Self { at })
    }

    /// As its file holds it, headed by what it is.
    #[must_use]
    pub fn written(&self) -> String {
        let mut text = String::from(THE_HEAD);
        for (level, named) in &self.at {
            text.push_str(&format!("reached\t{level}\t{named}\n"));
        }
        text
    }

    /// Whether `request` has reached every try at `level`.
    #[must_use]
    pub fn has(&self, level: TheLevel, request: &str) -> bool {
        self.at
            .contains(&(the_name_of(level).to_owned(), request.to_owned()))
    }

    /// **The record raised by `measured`**: each request that took the road on
    /// every one of at least the bar's tries is added at its level. Nothing is
    /// ever taken away.
    #[must_use]
    pub fn raised_by(mut self, measured: &Measured) -> Self {
        for (level, named, on, of) in &measured.each {
            if *of >= THE_BAR.tries() && on == of {
                self.at.insert((level.clone(), named.clone()));
            }
        }
        self
    }

    /// Every entry of `before` this record does not hold, for a request still
    /// in the set (`in_the_set`): each a lowering.
    #[must_use]
    pub fn lowered_from(
        &self,
        before: &Self,
        in_the_set: &dyn Fn(&str) -> bool,
    ) -> Vec<(String, String)> {
        before
            .at
            .iter()
            .filter(|entry| !self.at.contains(*entry) && in_the_set(&entry.1))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
#[path = "what_each_has_reached_tests.rs"]
mod tests;
