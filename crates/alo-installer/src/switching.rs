//! *Restart into alo OS*, from inside Windows.
//!
//! The installer plan's task 4 and
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md):
//! switching between the two systems is something each of them offers from
//! inside itself, and a person who has to find a firmware key has not been
//! given that. alo OS's side is `crates/alo-starting` — *Restart into
//! Windows*, which sets the firmware's next start and leaves the default
//! alone. **This is the same act from the other side**, and it is deliberately
//! the same shape:
//!
//! - it changes **the next start only** (`BootNext`), never the order and
//!   never the default, so the choice is for one restart;
//! - it takes the person's word for it first, typed, as everything this
//!   program does that changes the computer is;
//! - it changes nothing else at all — no disk, no partition, no file.
//!
//! # Why it is this program
//!
//! The installer is what the person already has and already ran. Rather than
//! ship a second program, `crate::staging` leaves this one where the person
//! can find it, and it does this when it is started with
//! [`THE_SWITCHS_WORD`] as its only argument (`main.rs`). A start with no
//! argument is an install, as it always was.
//!
//! # What it does not do
//!
//! It never makes alo OS the system the computer starts by default: that is
//! the person's own setting, changed from either side, and it is task 17 of
//! the plan on the alo OS side. It never removes anything. And it does not
//! know whether alo OS is installed — it knows only what the firmware lists,
//! which is what a restart can actually reach.

use alo_strings::{Filling, Strings, Word};

use crate::entries;
use crate::machine::{BEFORE_RESTARTING, TheMachine};
use crate::program::Program;
use crate::words;

/// The argument that makes this program the switch rather than the installer.
///
/// Not a translated word: it is written into the shortcut the installer makes,
/// read by this program alone, and never typed by a person.
pub const THE_SWITCHS_WORD: &str = "restart-into-alo-os";

/// How the switch ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Switched {
    /// The next start is alo OS, and the computer was asked to restart.
    Restarting {
        /// Whether the restart was asked for and accepted.
        restarted: bool,
    },
    /// The person did not agree, and nothing was changed.
    NotAgreed,
    /// The firmware lists no entry named alo OS, so there is nothing to start.
    NotThere,
    /// The firmware's list could not be read.
    NotRead,
    /// The next start could not be set, and nothing was changed.
    NotSet,
}

/// Offer to restart into alo OS, and do it if the person says so.
pub fn restart_into_alo_os(machine: &mut impl TheMachine, strings: &Strings) -> Switched {
    say(
        machine,
        strings,
        words::SWITCH_STARTING,
        &Filling::nothing(),
    );
    let Some(printed) = read(machine, &Program::ListingTheStartEntries) else {
        say(
            machine,
            strings,
            words::SWITCH_NOT_READ,
            &Filling::nothing(),
        );
        return Switched::NotRead;
    };
    let Some(entry) = entries::the_one_named_alo_os(&printed) else {
        say(
            machine,
            strings,
            words::SWITCH_NOT_THERE,
            &Filling::nothing(),
        );
        return Switched::NotThere;
    };

    say(
        machine,
        strings,
        words::SWITCH_WILL_RESTART,
        &Filling::nothing(),
    );
    // The question names the word to type, in the language it is read in.
    let word = strings
        .say(&words::SWITCH_AGREED.key(), &Filling::nothing())
        .into_text();
    let typed = machine.ask(&strings.say(
        &words::SWITCH_TYPE_TO_AGREE.key(),
        &Filling::of("word", word),
    ));
    if !agreed(&typed, strings) {
        say(
            machine,
            strings,
            words::SWITCH_NOT_AGREED,
            &Filling::nothing(),
        );
        return Switched::NotAgreed;
    }

    // The next start, and nothing else. A failure here leaves the firmware as
    // it was, and the next start is forgotten rather than left half set.
    if read(machine, &Program::StartingTheEntryNext { entry }).is_none() {
        let _ = read(machine, &Program::ForgettingTheNextStart);
        say(machine, strings, words::SWITCH_NOT_SET, &Filling::nothing());
        return Switched::NotSet;
    }
    say(
        machine,
        strings,
        words::SWITCH_RESTARTING,
        &Filling::nothing(),
    );
    machine.pause(BEFORE_RESTARTING);
    let restarted = read(machine, &Program::Restarting).is_some();
    if !restarted {
        say(
            machine,
            strings,
            words::SWITCH_RESTART_IT_YOURSELF,
            &Filling::nothing(),
        );
    }
    Switched::Restarting { restarted }
}

/// Whether what the person typed is the word that agrees, in their language or
/// in the source — the same rule as the answers in `crate::asking`.
fn agreed(typed: &str, strings: &Strings) -> bool {
    let typed: String = typed
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if typed.is_empty() {
        return false;
    }
    let word = words::SWITCH_AGREED;
    let theirs = strings
        .say(&word.key(), &Filling::nothing())
        .into_text()
        .to_lowercase();
    typed == theirs.trim() || typed == word.says().to_lowercase()
}

/// What a program printed, when it ran and succeeded.
fn read(machine: &mut impl TheMachine, program: &Program) -> Option<String> {
    machine
        .run(program)
        .ok()
        .filter(|ran| ran.succeeded)
        .map(|ran| ran.printed)
}

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}
