//! **Where the described thing is, asked of the reader the person chose, and
//! the click made there** — the daemon's part of `accessible.click_at`
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md),
//! amended 2026-11-21: *"Yes, click by description"*).
//!
//! After the OK (or at once at full access) the turn runs every check and
//! pictures the window, and answers with `alo_turn::AThingToFind`. Here, under
//! that **same approval**, the picture goes to the reader with
//! `alo_adapters::where_a_thing_is::the_question`, through the one road a
//! picture leaves by (`crate::reading_a_picture::asking_the_reader`): the
//! indicator shows it as it leaves and the record keeps it (law 1). The
//! reader's one line, or why no reader could be asked, goes back to the turn
//! (`alo_turn::Turning::clicking_where_it_was_found`), which reads it, clicks
//! where exactly one thing was found, and writes the entry.
//!
//! **The model is never told the reader's answer**: what goes back to it is
//! what was clicked and what the window shows after, never where.

use std::time::SystemTime;

use alo_strings::Strings;
use alo_turn::{NotDone, TheScreenRoadAsChosen, Turning, WhatAVerbDid};

use crate::questions::Questions;

/// `did`, with a click still to be found asked of the reader and made — or
/// refused — and every other answer as it was.
///
/// # Errors
/// [`NotDone`], as the turn answers a click not made: written down there.
pub fn found_and_clicked(
    did: WhatAVerbDid,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    road: &dyn TheScreenRoadAsChosen,
    strings: &Strings,
    now: SystemTime,
) -> Result<WhatAVerbDid, NotDone> {
    let Some(thing) = did.thing_to_find() else {
        return Ok(did);
    };
    let read = where_it_is(thing, turning, questions, road, strings, now);
    turning.clicking_where_it_was_found(read)
}

/// The reader's one line about where `thing` is, or why no reader could be
/// asked.
fn where_it_is(
    thing: &alo_turn::AThingToFind,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    road: &dyn TheScreenRoadAsChosen,
    strings: &Strings,
    now: SystemTime,
) -> Result<String, alo_strings::Said> {
    let asked =
        alo_adapters::where_a_thing_is::the_question(thing.named(), thing.description(), strings)
            .into_text();
    crate::reading_a_picture::asking_the_reader(
        thing.before(),
        &asked,
        turning,
        questions,
        road,
        strings,
        now,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "clicking_where_it_is_tests.rs"]
mod tests;
