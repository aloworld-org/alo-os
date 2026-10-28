//! One entry of the account as the words drawn for it, in the order a person
//! reads them.
//!
//! Nothing here is worded. Each entry is:
//!
//! - **the day it happened**, above the first entry of each day in view —
//!   `alo_formats::TheDay`: *Today*, *Yesterday*, or the date as the person's
//!   region writes one (frame `82:887`, and the owner's decision of 2026-10-28
//!   that every change of day gets one);
//! - **the time of day** at its head, as the person's region writes it, before
//!   the clause — *17:38  alo shortened the introduction*;
//! - **the clause** — `alo_recounting::Outcome::said`, the vocabulary's words
//!   for what became of it;
//! - **whose authority** it was under, where the record names an agent — and
//!   nothing at all where it names none, which is alo OS's own errand: a name
//!   drawn there would be an authority the screen invented;
//! - **the machine's own sentence** — what the call was, as it was worded
//!   when it happened;
//! - **that the person said yes**, where the record holds the approval a change
//!   ran from — `alo_recounting::Approved::said`, so the words are the
//!   vocabulary's and the fact is the record's;
//! - **why it was refused**, in the words the person was shown then;
//! - **where something went**, as `alo_egress::Destination::shown` words it,
//!   which is the one rendering the indicator used at the time.
//!
//! What a model typed and the machine never turned into a call is not drawn on
//! a line of its own: the machine's refusal already quotes it, inside a sentence
//! the machine wrote, so it cannot be mistaken for a description of what
//! happened.
//!
//! Newest first: the account holds entries in the order they happened, and a
//! person asking what the machine did reads the most recent first. Nothing is
//! left out and nothing is reordered beyond that one reversal.
//!
//! **The first entry in view always carries its day**, even when the entry
//! above it — out of view — was the same day: a person who has moved down the
//! record is otherwise reading times with no day to them.

use alo_formats::TheDay;
use alo_recounting::{Account, Outcome, Told};
use alo_strings::Strings;

use crate::record_clock::RecordClock;

/// The words drawn for one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntryWords {
    /// The day above it, when it is the first of its day in view.
    pub(crate) day: Option<String>,
    /// The time of day it happened, as the person writes one — [`None`] only
    /// for a moment no calendar holds.
    pub(crate) time: Option<String>,
    /// What became of it.
    pub(crate) outcome: Outcome,
    /// The clause at the head of the entry, as the vocabulary answered it.
    pub(crate) clause: String,
    /// Whose authority it was under — [`None`] for the machine's own errand.
    pub(crate) agent: Option<String>,
    /// Everything after the clause and the agent, each drawn whole: the
    /// machine's sentence, that the person said yes to it, why it was refused,
    /// where something went.
    pub(crate) after: Vec<String>,
}

/// The account's entries, most recent first, from `moved_past` entries back.
pub(crate) fn newest_first<'a>(
    account: &'a Account,
    moved_past: usize,
    strings: &'a Strings,
    clock: RecordClock<'a>,
) -> impl Iterator<Item = EntryWords> + 'a {
    let mut the_day_above: Option<TheDay> = None;
    account
        .told()
        .iter()
        .rev()
        .skip(moved_past)
        .map(move |told| {
            let day = clock.formats.day_of(told.at(), clock.now);
            let heading = if day.is_some() && day != the_day_above {
                day.as_ref().map(|day| day.said(strings))
            } else {
                None
            };
            the_day_above = day;
            words_of(told, strings, heading, clock)
        })
}

/// The words for one entry.
fn words_of(
    told: &Told,
    strings: &Strings,
    day: Option<String>,
    clock: RecordClock<'_>,
) -> EntryWords {
    let mut after = Vec::new();
    if let Some(sentence) = told.sentence() {
        after.push(sentence.as_str().to_owned());
    }
    if let Some(approved) = told.approved() {
        after.push(approved.said(strings).into_text());
    }
    if let Some(because) = told.because() {
        after.push(because.as_str().to_owned());
    }
    if let Some(went_to) = told.went_to() {
        after.push(went_to.shown(strings));
    }
    EntryWords {
        day,
        time: clock.formats.time_of(told.at()),
        outcome: told.outcome(),
        clause: told.outcome().said(strings).into_text(),
        agent: told.agent().map(|agent| agent.as_str().to_owned()),
        after,
    }
}
