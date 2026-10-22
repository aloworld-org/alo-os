//! A place a person can return to, kept before a command they typed.
//!
//! [ADR 0130](../../../docs/decisions/0130-what-takes-the-bracket-before-a-persons-own-command.md),
//! accepted 2026-10-22, option **B**: the person's own shell asks this service,
//! and this service takes the snapshot. Not the OSC 133 mark, for two reasons
//! either of which is enough — `docs/contracts/shell-integration-marks.md`
//! promises a forged mark *cannot make alo do anything*, and any program in the
//! session can print one; and the mark arrives too late, because bash prints it
//! and then execs while alo reads the pseudoterminal on its own schedule.
//!
//! # It is the person's own road, and it is not a turn
//!
//! `crate::the_bracket_this_turn_takes` brackets an **agent's** turn, once, at
//! the moment a change is approved. This brackets a **person's** command,
//! before it runs, because they asked for that. The two share
//! `alo_turn::TheBracket` and nothing else: ADR 0125 named conflating a
//! person's request with an agent's as the part most likely to be got wrong,
//! and a shared road with a flag would be that conflation with a field to sort
//! it out afterwards.
//!
//! # Only the *before* side, and a note beside it
//!
//! A turn takes an *after* side when it ends, because something will compare the
//! two. Nothing compares a person's command. But the **note** — `kept.json` — is
//! not optional the way the second snapshot is: it is the only thing that makes
//! a place sweepable by the privileged remover and findable by the person, since
//! a place's directory name is opaque by contract. So this takes one snapshot
//! and writes one note.
//!
//! # The command runs either way
//!
//! ADR 0045's third term, carried here: *a machine too full to snapshot still
//! runs the turn.* A person whose disk is full still gets to run `ls`. So this
//! answers whether a place was kept and never refuses — and when one was not,
//! it says so, **once, before the command**, because the thing a person needs
//! told is the one that did not happen.

use std::time::SystemTime;

use alo_protocol::{ToAPerson, Wording};
use alo_strings::{Filling, Strings};

use alo_turn::the_bracket_a_turn_takes::{
    ASideOfTheTurn, WhereThisTurnIsKept, where_their_settings_are,
};

use crate::the_bracket_this_turn_takes::WhereBracketsGo;

/// Keep a place to return to, and say whether one was kept.
///
/// `goes` is [`None`] on a session with no home, which is a person's state
/// rather than a fault — and is answered the same way as a snapshot that could
/// not be taken, because from where the person sits those are one thing: no
/// place to return to.
///
/// `named` is what the directory is called. The contract calls bracket names
/// **opaque**, so nothing reads it back and nothing here parses one.
pub fn kept(
    goes: Option<&WhereBracketsGo>,
    named: &str,
    now: SystemTime,
    strings: &Strings,
) -> ToAPerson {
    let why_not = || {
        Some(Wording::of(&strings.say(
            &crate::words::NO_PLACE_TO_RETURN_TO.key(),
            &Filling::nothing(),
        )))
    };
    let Some(goes) = goes else {
        return ToAPerson::APlaceToReturnTo {
            kept: false,
            why_not: why_not(),
        };
    };
    // **The person's own folder is made first, exactly as a turn's bracket
    // does it.** `where_their_settings_are` is what puts the undo directory
    // there; skipping it here would make a place-keeping that works for an
    // agent and fails for a person on a machine nobody had bracketed yet.
    if where_their_settings_are(goes.whose(), goes.home()).is_err() {
        return ToAPerson::APlaceToReturnTo {
            kept: false,
            why_not: why_not(),
        };
    }
    match WhereThisTurnIsKept::made(goes.whose(), named) {
        Ok(kept) => match kept.keeping(alo_kept_undo::BEFORE, goes.home()) {
            // **Only the *before* side.** A turn takes an *after* side when it
            // ends, because somebody will compare them. Nothing compares a
            // person's command — they asked to be able to go back, not to be
            // shown a diff — and an *after* side nothing reads would be a
            // second snapshot of their whole home for every command they run.
            ASideOfTheTurn::Kept(_) => {
                // **And write down that it is there, or it is there for ever.**
                // The privileged remover's own reader —
                // `crates/alo-letting-go/src/found.rs`, named by file because
                // `crates/alo-letting-go/tests/a_turn_cannot_arrive_at_this_road.rs`
                // forbids anything a turn runs inside from spelling that crate's
                // Rust path, and it is right to: forgetting is a road only a
                // person may take (ADR 0045, narrowed 2026-09-22) — reads a place
                // by its `kept.json` and *steps over* a directory whose note will
                // not read, which is
                // how the privileged remover avoids touching what it does not
                // understand. A place with no note is therefore never swept, so
                // a person who turned this on would accumulate one read-only
                // snapshot of their whole home per command they typed, for ever.
                // That defeats ADR 0045's fifth term and its *disk pressure wins
                // over undo*, and it is also what makes a place **findable**: the
                // note carries the moment, and a directory's name is opaque on
                // purpose, so there is nowhere else to read it from.
                //
                // A note that could not be written is reported as *no place
                // kept*, because a snapshot nothing can sweep or find is not a
                // place a person can return to — it is only disk.
                let did = strings.say(
                    &crate::words::BEFORE_A_COMMAND_YOU_TYPED.key(),
                    &Filling::nothing(),
                );
                match kept.what_it_did(now, did.text()) {
                    Ok(()) => ToAPerson::APlaceToReturnTo {
                        kept: true,
                        why_not: None,
                    },
                    Err(_) => ToAPerson::APlaceToReturnTo {
                        kept: false,
                        why_not: why_not(),
                    },
                }
            }
            ASideOfTheTurn::NotKept(_) => ToAPerson::APlaceToReturnTo {
                kept: false,
                why_not: why_not(),
            },
        },
        Err(_) => ToAPerson::APlaceToReturnTo {
            kept: false,
            why_not: why_not(),
        },
    }
}

/// What to call the directory for a place kept at this moment.
///
/// **The same shape the agent's own brackets use**, so a person looking at the
/// undo folder sees one kind of thing rather than two.
#[must_use]
pub fn a_name_for(now: SystemTime, process: u32) -> String {
    let since = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{since}-{process}-their-own")
}

#[cfg(test)]
#[path = "a_place_to_return_to_tests.rs"]
mod tests;
