//! Gives one person their place for what an undo keeps, and stops.
//!
//! ```text
//! alo-giving-a-folder <account number> [the folder undo is kept in]
//! ```
//!
//! Started by systemd as `alo-giving-a-folder@<account number>.service`, which
//! is where the one capability lives — see
//! [ADR 0119](../../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md).
//!
//! # Why a template unit rather than a function in the caller
//!
//! **The capability has to be somewhere, and the caller is the wrong place.**
//! The account is written inside `alo-compositor`, which is `User=root` with
//! `CapabilityBoundingSet=` empty — and ADR 0119 measured why it must stay that
//! way: it is the screen a person types a password into and fifty-two times the
//! size of the next candidate.
//!
//! A unit solves it without moving the power: the caller asks systemd to start
//! `alo-giving-a-folder@1000.service`, which it may do **as root with no
//! capabilities at all**, because what authorises starting a unit is who you are
//! and not what you hold. The capability lives in the unit and reaches only this
//! program.
//!
//! # It takes two arguments and reads nothing else
//!
//! No configuration file, no environment, nothing from a network, nothing from
//! a person. The second argument defaults to the one place a machine keeps
//! this, so the unit passes one argument and a test passes two.
//!
//! # It says what happened on the journal and in one line
//!
//! Whoever administers a machine reads this in `journalctl`, so the one line it
//! prints names the account and the folder. A success says so too: a oneshot
//! that prints nothing is indistinguishable from one that never ran, which this
//! repository has already paid for once — a `Finished` with no `Started` reads
//! as silence.

#![cfg(target_os = "linux")]

use std::{path::PathBuf, process::ExitCode};

use alo_giving_a_folder::{NotMade, THE_FOLDER, made_for};

/// Reads two arguments, gives one person their folder, and stops.
fn main() -> ExitCode {
    let mut asked = std::env::args_os().skip(1);

    let Some(person) = asked.next() else {
        eprintln!(
            "alo-giving-a-folder: nothing said which account to give a folder to. \
             Started as alo-giving-a-folder@<account number>.service."
        );
        return ExitCode::FAILURE;
    };
    let said = person.to_string_lossy().into_owned();
    let Ok(person) = said.parse::<u32>() else {
        eprintln!("alo-giving-a-folder: {}", NotMade::NotAnAccountNumber(said));
        return ExitCode::FAILURE;
    };

    let under = asked
        .next()
        .map_or_else(|| PathBuf::from(THE_FOLDER), PathBuf::from);

    match made_for(&under, person) {
        Ok(theirs) => {
            println!(
                "alo-giving-a-folder: account {person} has {} for what an undo would put back",
                theirs.display()
            );
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("alo-giving-a-folder: {why}");
            ExitCode::FAILURE
        }
    }
}
