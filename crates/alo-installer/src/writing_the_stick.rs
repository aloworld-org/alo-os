//! *Write a stick*, from inside Windows, so a computer with no cable can still
//! be given alo OS.
//!
//! `docs/features.md`, the owner's ruling of 2026-10-20 — *let's build 1 and 2*
//! — and the reason it is not a nicety: **the staged environment is wired-only,
//! and the target audience is developers, most of whom have no cable.** A
//! machine whose only network is Wi-Fi gets as far as the restart and no
//! further, and this is the road that does not need the restart to reach a
//! network at all.
//!
//! # The heaviest warning in that document is about this file
//!
//! > The one destructive thing in this product that reaches a device a person
//! > did not choose for alo OS. Writing a stick **erases it completely**: there
//! > is no road that keeps what is on it, no partial write and no undo, and the
//! > device is not the disk they asked us to install onto. … *A stick that is
//! > offered and refused teaches nothing; a stick that is written by mistake is
//! > somebody's photographs.*
//!
//! So the refusals are the subject of this file and the write is the short part:
//! **nothing removable plugged in**, a name that **does not match exactly**, a
//! person who **types nothing**, and an image that **is not there**. Each one
//! ends the road and changes nothing.
//!
//! # It is `alo-installer`'s and not the environment's
//!
//! The sentences live in `alo-installing` because that is where somebody
//! expected the screens to be drawn, and two of them settle where the **act**
//! belongs: *"Restart this computer and start from the stick you just wrote"* is
//! one computer writing and then booting, which is a person at their existing
//! Windows; and *"You can keep using this computer while it writes"* is not
//! something anybody says of a boot environment, which is a machine doing one
//! thing.
//!
//! **No key is moved.** `alo-installer` already depends on `alo-installing` and
//! `crate::say` takes any `Word`, so this file says `installing.*` sentences
//! from here. ADR 0068 makes a moved key a declaration, and there is nothing to
//! declare: the sentences are right where a translator met them.
//!
//! # The sibling road, and why they read as one thing
//!
//! [`crate::starting_the_stick`] restarts this computer into a stick and
//! **writes nothing** — it assumes one exists. This file makes one. Together a
//! person writes a stick and then starts from it, which is why the last
//! sentences here hand straight over to that road.
//!
//! *Those two titles are one word apart and were read as the same feature on
//! 2026-10-22 — #927 **starts** the stick; nothing wrote one. Said here because
//! the next reader will make the same mistake.*

use alo_installing::words as said;
use alo_strings::{Filling, Strings};

use crate::disks::{Disk, Disks};
use crate::machine::TheMachine;
use crate::program::Program;
use crate::words;

/// How many bytes are written between one telling and the next.
///
/// **Four mebibytes**, which is a compromise between a progress line that moves
/// and a write that spends its time reporting. A stick is tens of thousands of
/// these; the line a person reads is a percentage, so they see it move about
/// once a second on a slow device and more often on a fast one.
const BETWEEN_TELLINGS: u64 = 4 * 1024 * 1024;

/// Which road writing a stick took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Written {
    /// The stick was erased and written, and is ready to start a computer.
    Written {
        /// The device, as a person was shown it.
        device: String,
    },
    /// Nothing removable is plugged in, so there is nothing to offer.
    NothingPluggedIn,
    /// What was typed named none of the devices.
    NotThatName,
    /// The person typed nothing, which is how they decline.
    NotAgreed,
    /// There is no alo OS beside this program to write.
    NoImage,
    /// The disks could not be read.
    NotRead,
    /// The device is smaller than the image, so nothing was erased.
    TooSmall {
        /// The device, as a person was shown it.
        device: String,
        /// How many bytes it holds.
        holds: u64,
        /// How many the image needs.
        needs: u64,
    },
    /// The device refused part-way through, and the stick is now unusable.
    NotWritten {
        /// What the device said.
        why: String,
    },
}

/// Offer to erase a device a person plugs in and write alo OS onto it.
///
/// Returns which road was taken. **Every road but [`Written::NotWritten`] and
/// [`Written::Written`] changes nothing anywhere**, and those two change only
/// the device the person named.
pub fn write_the_stick(machine: &mut impl TheMachine, strings: &Strings) -> Written {
    say(machine, strings, said::WRITE_A_STICK, &Filling::nothing());

    // **Before anything is offered.** A person who picked a device and typed its
    // name, only to be told there is nothing to write, has been walked through a
    // consent for no reason — and the next time they are asked they will read it
    // less carefully.
    let Some((image, how_big)) = the_image(machine) else {
        say(
            machine,
            strings,
            said::ONLY_A_GENUINE_ALO_OS,
            &Filling::nothing(),
        );
        return Written::NoImage;
    };

    let Some(disks) = the_disks(machine) else {
        say(
            machine,
            strings,
            words::SWITCH_NOT_READ,
            &Filling::nothing(),
        );
        return Written::NotRead;
    };

    // **Removable only, and writable, which is a promise and not a filter.**
    // `features.md`: *the devices offered are removable ones and never the disk
    // this computer runs on*. `Disk::is_removable` is the same reading
    // `crate::starting_the_stick` uses, so the list offered here is the list
    // offered there. A read-only device is left out as well: offering one leads
    // a person through the whole consent to a refusal at the last step.
    let offered: Vec<&Disk> = disks
        .every()
        .filter(|disk| disk.is_removable() && !disk.is_read_only())
        .collect();
    if offered.is_empty() {
        say(
            machine,
            strings,
            words::STICK_NONE_PLUGGED_IN,
            &Filling::nothing(),
        );
        return Written::NothingPluggedIn;
    }

    say(machine, strings, said::WHICH_STICK, &Filling::nothing());
    say(
        machine,
        strings,
        said::ONLY_REMOVABLE_DEVICES,
        &Filling::nothing(),
    );
    say(
        machine,
        strings,
        said::THE_STICK_WILL_BE_ERASED,
        &Filling::nothing(),
    );
    for disk in &offered {
        say(
            machine,
            strings,
            words::STICK_A_DEVICE,
            &Filling::of("device", disks.shown_name(disk)),
        );
    }

    let typed = machine.ask(&strings.say(&said::TYPE_THE_STICKS_NAME.key(), &Filling::nothing()));
    let typed = typed.trim();
    if typed.is_empty() {
        say(
            machine,
            strings,
            words::SWITCH_NOT_AGREED,
            &Filling::nothing(),
        );
        return Written::NotAgreed;
    }

    // **Exactly, and not nearly.** `crate::starting_the_stick`'s reason with
    // more at stake: a near miss there points the next start at another device,
    // and a near miss here **erases** it. No guessing, no closest match, no case
    // folding beyond what the name itself carries.
    let Some(chosen) = offered.iter().find(|disk| disks.shown_name(disk) == typed) else {
        say(
            machine,
            strings,
            words::STICK_NOT_THAT_NAME,
            &Filling::nothing(),
        );
        return Written::NotThatName;
    };
    let named = disks.shown_name(chosen);

    // **A stick the image does not fit on is refused before it is erased.**
    // Writing as far as it goes would leave a person a wiped device that starts
    // nothing, having told them it was ready — and the device they chose is the
    // one thing in this road alo OS destroys. Reachable only now that the size
    // is asked for rather than the file being read.
    if chosen.size() < how_big {
        say(
            machine,
            strings,
            said::CHOOSE_ANOTHER_STICK,
            &Filling::nothing(),
        );
        return Written::TooSmall {
            device: named,
            holds: chosen.size(),
            needs: how_big,
        };
    }

    say(
        machine,
        strings,
        said::THIS_ERASES_THE_STICK,
        &Filling::of("stick", named.clone()),
    );
    say(
        machine,
        strings,
        said::WHAT_GOES_FROM_THE_STICK,
        &Filling::of("holding", chosen.about_how_big()),
    );
    say(
        machine,
        strings,
        said::ONLY_THE_STICK_IS_WRITTEN,
        &Filling::nothing(),
    );
    say(
        machine,
        strings,
        said::LEAVE_THE_STICK_IN,
        &Filling::nothing(),
    );
    say(
        machine,
        strings,
        said::KEEP_USING_THIS_COMPUTER,
        &Filling::nothing(),
    );
    say(machine, strings, said::WRITING, &Filling::nothing());
    say(
        machine,
        strings,
        said::WRITING_TO_THE_STICK,
        &Filling::of("stick", named.clone()),
    );

    let device = chosen.physical_drive();
    let mut how_far: Vec<String> = Vec::new();
    let mut since = 0_u64;
    // **Throttled, because a reading is cheap and a million of them are not.**
    // The device decides how often it calls this; a write of a few gibibytes
    // through a small buffer would otherwise collect hundreds of thousands of
    // strings for a road that says two of them.
    let mut told = |done: u64, all: u64| {
        if all == 0 {
            return;
        }
        let finished = done >= all;
        if !finished && done < since.saturating_add(BETWEEN_TELLINGS) {
            return;
        }
        since = done;
        how_far.push(format!("{}%", done.saturating_mul(100) / all));
    };
    let went = machine.wrote_the_device(&device, &image, &mut told);
    // **Said after the write and not during it.** The progress a person watches
    // is the graphical screen's; this road is a text one, and a line per four
    // mebibytes would bury every sentence around it. What the readings are for
    // is that a caller can show them — `installing.how-far-the-write` is the
    // sentence, and this hands the readings back rather than deciding.
    for reading in reported(&how_far) {
        say(
            machine,
            strings,
            said::HOW_FAR_THE_WRITE_HAS_GOT,
            &Filling::of("how_far", reading),
        );
    }
    if let Err(why) = went {
        // **The stick is unusable now and the person is told so**, which is what
        // `installing.leave-the-stick-in` warned about before this began. There
        // is no partial write worth keeping and no undo: `features.md` says so
        // in the line that adds this road.
        say(machine, strings, words::SWITCH_NOT_SET, &Filling::nothing());
        return Written::NotWritten {
            why: why.to_string(),
        };
    }

    say(
        machine,
        strings,
        said::CHECKED_BEFORE_IT_WAS_WRITTEN,
        &Filling::nothing(),
    );
    say(
        machine,
        strings,
        said::THE_STICK_IS_READY,
        &Filling::nothing(),
    );
    say(
        machine,
        strings,
        said::TAKE_IT_TO_THE_OTHER_COMPUTER,
        &Filling::nothing(),
    );
    say(
        machine,
        strings,
        said::START_THIS_PC_FROM_IT,
        &Filling::nothing(),
    );
    Written::Written { device: named }
}

/// Which of the readings are worth saying on a text road.
///
/// The first and the last, and nothing between: a person reading lines wants to
/// know it started and that it finished, and a line per four mebibytes would
/// bury the sentences around them. The graphical screen shows all of them.
fn reported(how_far: &[String]) -> Vec<String> {
    match how_far {
        [] => Vec::new(),
        [only] => vec![only.clone()],
        [first, .., last] => vec![first.clone(), last.clone()],
    }
}

/// The alo OS beside this program and how big it is, or nothing.
///
/// Beside the installer for `crate::environment`'s reason: what a person
/// downloaded from the release page arrived together, so the image is where this
/// program is rather than somewhere a person has to find.
///
/// **It asks the size and does not read the file.** This called
/// `TheMachine::read` — *a whole file* — and discarded every byte, to answer
/// *is it there*. The image is gigabytes, so on the kind of laptop a person
/// writes a stick from that was an out-of-memory failure **before the first
/// warning was drawn**. Found by the desktop lane reading the code on
/// 2026-10-22; no test here could have seen it, because a fake's `read` returns
/// a small `Vec` and is correct and fast about nothing.
fn the_image(machine: &mut impl TheMachine) -> Option<(std::path::PathBuf, u64)> {
    let beside = machine.downloaded_into().ok()?.join(THE_IMAGE);
    let how_big = machine.how_big_is(&beside)?;
    Some((beside, how_big))
}

/// What the written image is called beside the installer.
pub const THE_IMAGE: &str = "alo-os.raw";

/// The word that asks this program to write a device a person plugs in.
///
/// **A word and not a flag**, as the other four roads are and in the same
/// shape: a person reads it in a sentence the program printed and types it
/// back. `crate::THE_STICKS_WORD` is its sibling — that one *starts* a stick
/// that already exists and writes nothing, this one makes one.
///
/// *Added in the same change as the road, because a road nothing reaches is the
/// fault this repository has found seven times. It was missing for the length of
/// one review: `write_the_stick` was declared, exported and tested, and no word
/// a person could type arrived at it — found by the desktop lane, not by me.*
pub const THE_WRITINGS_WORD: &str = "write-the-stick";

/// This computer's disks, or nothing when they could not be read.
fn the_disks(machine: &mut impl TheMachine) -> Option<Disks> {
    let printed = machine.run(&Program::ListingTheDisks).ok()?;
    printed
        .succeeded
        .then(|| Disks::read(Some(&printed.printed)))
        .flatten()
}

/// Say one sentence.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: words::Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}

#[cfg(test)]
#[path = "writing_the_stick_tests.rs"]
mod tests;
