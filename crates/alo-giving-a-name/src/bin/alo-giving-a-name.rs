//! Gives one person their own name, in a file only they can read, and stops.
//!
//! ```text
//! alo-giving-a-name <account number> [the folder names are given in] [the account store]
//! ```
//!
//! Started by systemd as `alo-giving-a-name@<account number>.service`, which
//! `alo-sessiond` starts at every sign-in and which is where the one capability
//! lives — see
//! [ADR 0136](../../../../docs/decisions/0136-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md).
//! The second and third arguments default to the one place a machine keeps
//! each, so the unit passes one argument and a test passes three.
//!
//! # It says what happened on the journal, and never the name
//!
//! Whoever administers a machine reads this in `journalctl`, so the one line
//! it prints names the account number and the file. **It does not print the
//! name**: the journal is readable by more people than the file is, and a line
//! that said it would undo the mode the file was given.

#![cfg(target_os = "linux")]

use std::{path::PathBuf, process::ExitCode};

use alo_giving_a_name::{NotGiven, given_to};

/// Reads up to three arguments, gives one person their name, and stops.
fn main() -> ExitCode {
    let mut asked = std::env::args_os().skip(1);

    let Some(person) = asked.next() else {
        eprintln!(
            "alo-giving-a-name: nothing said which account to give a name to. \
             Started as alo-giving-a-name@<account number>.service."
        );
        return ExitCode::FAILURE;
    };
    let said = person.to_string_lossy().into_owned();
    let Ok(person) = said.parse::<u32>() else {
        eprintln!("alo-giving-a-name: {}", NotGiven::NotAnAccountNumber(said));
        return ExitCode::FAILURE;
    };

    let under = asked
        .next()
        .map_or_else(|| PathBuf::from(alo_accounts::THE_NAMES), PathBuf::from);
    let accounts = asked
        .next()
        .map_or_else(|| PathBuf::from(alo_accounts::THE_ACCOUNTS), PathBuf::from);

    match given_to(&under, &accounts, person) {
        Ok(theirs) => {
            println!(
                "alo-giving-a-name: account {person} has its name in {}, readable by it alone",
                theirs.display()
            );
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("alo-giving-a-name: {why}");
            ExitCode::FAILURE
        }
    }
}
