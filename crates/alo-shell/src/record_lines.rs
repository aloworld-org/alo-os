//! One entry of the account as the words drawn for it, in the order a person
//! reads them.
//!
//! Nothing here is worded. Each entry is:
//!
//! - **the clause** at its head — `alo_recounting::Outcome::said`, the
//!   vocabulary's words for what became of it;
//! - **whose authority** it was under, where the record names an agent — and
//!   nothing at all where it names none, which is alo OS's own errand: a name
//!   drawn there would be an authority the screen invented;
//! - **the machine's own sentence** — what the call was, as it was worded
//!   when it happened;
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

use alo_recounting::{Account, Outcome, Told};
use alo_strings::Strings;

/// The words drawn for one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntryWords {
    /// What became of it.
    pub(crate) outcome: Outcome,
    /// The clause at the head of the entry, as the vocabulary answered it.
    pub(crate) clause: String,
    /// Whose authority it was under — [`None`] for the machine's own errand.
    pub(crate) agent: Option<String>,
    /// Everything after the clause and the agent, each drawn whole: the
    /// machine's sentence, why it was refused, where something went.
    pub(crate) after: Vec<String>,
}

/// The account's entries, most recent first, from `moved_past` entries back.
pub(crate) fn newest_first<'a>(
    account: &'a Account,
    moved_past: usize,
    strings: &'a Strings,
) -> impl Iterator<Item = EntryWords> + 'a {
    account
        .told()
        .iter()
        .rev()
        .skip(moved_past)
        .map(move |told| words_of(told, strings))
}

/// The words for one entry.
fn words_of(told: &Told, strings: &Strings) -> EntryWords {
    let mut after = Vec::new();
    if let Some(sentence) = told.sentence() {
        after.push(sentence.as_str().to_owned());
    }
    if let Some(because) = told.because() {
        after.push(because.as_str().to_owned());
    }
    if let Some(went_to) = told.went_to() {
        after.push(went_to.shown(strings));
    }
    EntryWords {
        outcome: told.outcome(),
        clause: told.outcome().said(strings).into_text(),
        agent: told.agent().map(|agent| agent.as_str().to_owned()),
        after,
    }
}
