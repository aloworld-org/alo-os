//! *Restart into the stick*, from inside Windows, so nobody hunts for a boot
//! key.
//!
//! `docs/features.md`: **And Windows starts the stick, so nobody hunts for a
//! boot key.** *The program running on Windows offers to restart straight into
//! it — no two-second window, no F2, F10, F12 or Esc that differs on every
//! machine, and no Fast Startup to turn off first.* The owner's note on that
//! line is the argument for the road existing at all:
//!
//! > This is where people are lost before they begin, and it is **not a
//! > technical obstacle — it is a trivia question about their own laptop.**
//!
//! # The same shape as *Restart into alo OS*, deliberately
//!
//! [`crate::switching`] is the same act pointed at a different entry, and this
//! file is its sibling rather than its cousin:
//!
//! - it changes **the next start only** (`BootNext`), never the order and
//!   never the default, so the choice is for one restart;
//! - it takes the person's word for it first, **typed**, as everything in this
//!   program that changes the computer does;
//! - it changes nothing on any disk at all — no partition, no file, nothing
//!   written to the device it starts.
//!
//! The one real difference is that *which entry* is not a constant here. alo
//! OS's entry is found by its name; a removable device's is found by matching
//! the device against what the firmware calls it, which can honestly fail —
//! see [`crate::entries::the_one_that_starts`].
//!
//! # Why the device is named and not numbered
//!
//! A person types the device's own name back, which is the consent the disk
//! already takes rather than a second ceremony invented here. `features.md`
//! asks for it in as many words beside the stick-writing road — *each is named
//! the way a person recognises it* and *nothing is written until they type
//! that device's own name* — and the reason is sharper on this road than it
//! looks: a number mistyped by one points the computer at a different device,
//! and a person who has two sticks in has no way to know which they got.
//!
//! # What this road does not do
//!
//! **It does not write anything to the stick.** Writing one is its own road
//! with its own consent, and it carries the heaviest warning in
//! `features.md` because *a stick that is written by mistake is somebody's
//! photographs*. This road only chooses what the computer starts from next.
//!
//! **And it does not turn Fast Startup off.** The promise says *no Fast
//! Startup to turn off first*, and that is true for a different reason than it
//! sounds: `BootNext` is read by the firmware before Windows resumes, so a
//! fast-started Windows does not bypass it. `crate::fast_startup` is asked
//! about on the install road, where it does matter.

use alo_strings::{Filling, Strings, Word};

use crate::disks::{Disk, Disks};
use crate::entries;
use crate::machine::TheMachine;
use crate::program::Program;
use crate::words;

/// The word that asks this program to start a device a person plugs in.
///
/// **A word and not a flag**, as the other three roads are and in the same
/// shape: a person reads it in a sentence the program printed and types it
/// back. `THE_SWITCHS_WORD` is its sibling and names the same act pointed at a
/// different thing.
pub const THE_STICKS_WORD: &str = "start-the-stick";

/// What came of asking to start a removable device.
///
/// **Every refusal is its own name**, rather than one `Refused` carrying a
/// sentence. A caller deciding an exit code, and a test asserting which road
/// was taken, both need to tell *the firmware would not change* from *the
/// person said no* — and a type that could not would make them read the words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Started {
    /// The next start was set and the restart was asked for.
    Restarting {
        /// Whether the restart itself was accepted.
        restarted: bool,
    },
    /// Nothing removable is plugged in.
    NothingPluggedIn,
    /// What was typed named none of the devices.
    NotThatName,
    /// The person typed nothing, which is how they decline.
    NotAgreed,
    /// The firmware keeps no entry for that device.
    NotListed,
    /// The disks could not be read.
    NotRead,
    /// The firmware refused the change.
    NotSet,
}

/// Offer to restart this computer into a device a person plugs in.
///
/// Returns which road was taken. Nothing on any disk is changed on any of
/// them.
pub fn start_the_stick(machine: &mut impl TheMachine, strings: &Strings) -> Started {
    say(machine, strings, words::STICK_STARTING, &Filling::nothing());

    let Some(disks) = the_disks(machine) else {
        say(
            machine,
            strings,
            words::SWITCH_NOT_READ,
            &Filling::nothing(),
        );
        return Started::NotRead;
    };

    // **Removable only, which is a promise and not a filter.** `features.md`:
    // *the devices offered are removable ones and never the disk this computer
    // runs on*.
    let removable: Vec<&Disk> = disks.every().filter(|d| d.is_removable()).collect();
    if removable.is_empty() {
        say(
            machine,
            strings,
            words::STICK_NONE_PLUGGED_IN,
            &Filling::nothing(),
        );
        return Started::NothingPluggedIn;
    }

    say(
        machine,
        strings,
        words::STICK_WHICH_ONE,
        &Filling::nothing(),
    );
    // Each on its own line, by the name `disks` already gives it — which
    // numbers two devices of one name rather than showing a person two
    // identical rows.
    for disk in &removable {
        say(
            machine,
            strings,
            words::STICK_A_DEVICE,
            &Filling::of("device", disks.shown_name(disk)),
        );
    }

    let typed = machine.ask(&strings.say(&words::STICK_TYPE_THE_NAME.key(), &Filling::nothing()));
    let typed = typed.trim();
    if typed.is_empty() {
        say(
            machine,
            strings,
            words::SWITCH_NOT_AGREED,
            &Filling::nothing(),
        );
        return Started::NotAgreed;
    }

    // **Exactly, and not nearly.** A near miss on this road points the computer
    // at a different device, and the person who mistyped is the one least able
    // to tell which they got — so no guessing, no closest match, no case
    // folding beyond what the name itself carries.
    let Some(chosen) = removable
        .iter()
        .find(|disk| disks.shown_name(disk) == typed)
    else {
        say(
            machine,
            strings,
            words::STICK_NOT_THAT_NAME,
            &Filling::nothing(),
        );
        return Started::NotThatName;
    };
    let named = disks.shown_name(chosen);

    let Some(printed) = read(machine, &Program::ListingTheStartEntries) else {
        say(
            machine,
            strings,
            words::SWITCH_NOT_READ,
            &Filling::nothing(),
        );
        return Started::NotRead;
    };

    // **A firmware that does not list it is an ordinary answer, not a fault.**
    // A computer that has never started from this device often keeps no entry
    // for it, and the honest thing is to say so and leave the person their own
    // start-up menu — rather than point the next start at whichever removable
    // entry happened to come first.
    let Some(entry) = entries::the_one_that_starts(&printed, &named) else {
        say(
            machine,
            strings,
            words::STICK_FIRMWARE_DOES_NOT_LIST_IT,
            &Filling::of("device", named.clone()),
        );
        return Started::NotListed;
    };

    say(
        machine,
        strings,
        words::STICK_WILL_RESTART,
        &Filling::of("device", named.clone()),
    );
    let word = strings
        .say(&words::SWITCH_AGREED.key(), &Filling::nothing())
        .into_text();
    let agreed = machine.ask(&strings.say(
        &words::SWITCH_TYPE_TO_AGREE.key(),
        &Filling::of("word", word),
    ));
    if !crate::switching::agreed(&agreed, strings) {
        say(
            machine,
            strings,
            words::SWITCH_NOT_AGREED,
            &Filling::nothing(),
        );
        return Started::NotAgreed;
    }

    // The next start, and nothing else. A failure here leaves the firmware as
    // it was: the next start is forgotten rather than left half set, so the
    // computer starts as it does today.
    if read(machine, &Program::StartingTheEntryNext { entry }).is_none() {
        let _ = read(machine, &Program::ForgettingTheNextStart);
        say(machine, strings, words::STICK_NOT_SET, &Filling::nothing());
        return Started::NotSet;
    }

    Started::Restarting {
        restarted: read(machine, &Program::Restarting).is_some(),
    }
}

/// The disks this computer has, or [`None`] where they would not read.
fn the_disks(machine: &mut impl TheMachine) -> Option<Disks> {
    Disks::read(read(machine, &Program::ListingTheDisks).as_deref())
}

/// What a program printed, where it ran and succeeded.
///
/// This file's own, as [`crate::switching`]'s is its own: two roads that
/// read programs are not a shared abstraction, and a helper lifted into one
/// would be a third place to look for four lines.
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
