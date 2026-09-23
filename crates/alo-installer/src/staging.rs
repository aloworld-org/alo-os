//! Preparing the computer, once the person has agreed — and putting it back
//! when preparing fails.
//!
//! In order, each step said before it begins:
//!
//! 1. shrink the Windows partition by exactly the area;
//! 2. make the area in the space that freed, at the place it freed;
//! 3. format it with the installer's label and give it a drive letter;
//! 4. write the environment's files and the person's choice onto it, and read
//!    each one back;
//! 5. write a start-up entry named alo OS pointing at the area's loader into
//!    the firmware — with none of the optional data a copy of Windows' own
//!    entry carries — and list it last so Windows stays the one the computer
//!    starts normally;
//! 6. take the area's letter away again;
//! 7. make the entry the next start, once — the firmware's next-boot choice,
//!    which never changes the default (`docs/booting.md`).
//!
//! # Every step before the restart is reversible
//!
//! Each change that succeeds is written into a journal before the next begins,
//! and a step that fails puts back everything in the journal, newest first:
//! the next start is forgotten, the entry removed, the area removed (only if it
//! is still where it was made), and Windows grown back to the size it had.
//!
//! **And a run killed at any step leaves a Windows that starts.** A smaller
//! Windows starts; an extra partition does not stop it; an entry listed last is
//! not started. The one step that changes what starts is the next-start
//! choice, and it is the last thing done before the restart, after the person
//! agreed to exactly that.

use alo_strings::{Filling, Strings, Word};
use serde::Deserialize;

use crate::asking::Answer;
use crate::deciding::{ForAloOs, Offer};
use crate::ended::{Ended, Refusal, Remains};
use crate::environment::{self, TheEnvironment};
use crate::identities::{DiskNumber, Entry, Letter, PartitionNumber};
use crate::machine::TheMachine;
use crate::program::Program;
use crate::sizes::{self, THE_AREA};
use crate::words;

/// One change made, and what putting it back needs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Made {
    /// Windows was shrunk from this size.
    Shrunk {
        /// Its disk.
        disk: DiskNumber,
        /// Its partition.
        partition: PartitionNumber,
        /// The size it had.
        was: u64,
    },
    /// The area was made here.
    TheArea {
        /// Its disk.
        disk: DiskNumber,
        /// Its number.
        partition: PartitionNumber,
        /// Where it begins.
        offset: u64,
    },
    /// Windows said it made a partition, and did not say which.
    AnAreaNotIdentified,
    /// The entry was made.
    TheEntry(Entry),
    /// Windows said it made an entry, and did not say which.
    AnEntryNotIdentified,
    /// The next start was set.
    TheNextStart,
    /// Fast Startup was turned off, from this value.
    FastStartupOff {
        /// What Windows' own value held before it was turned off.
        was: u32,
    },
}

/// What the area's making printed.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MadeTheArea {
    /// Its number.
    partition_number: u32,
    /// Where it begins.
    offset: u64,
}

/// What preparing the area printed.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PreparedTheArea {
    /// The letter it was given.
    drive_letter: String,
}

/// What writing the entry printed.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WroteTheEntry {
    /// The identifier Windows lists it under.
    identifier: String,
}

/// Prepare the computer for the disk the person chose.
///
/// # Errors
/// [`Ended::Refused`] with [`Refusal::PutBack`] when a step failed and every
/// change was put back, and [`Ended::NotPutBack`] naming what remains when
/// putting back failed too.
pub fn stage(
    machine: &mut impl TheMachine,
    strings: &Strings,
    offer: &Offer,
    chosen: &ForAloOs,
    the_environment: &TheEnvironment,
    fast_startup: Answer,
) -> Result<(), Ended> {
    let mut journal = Vec::new();
    match steps(
        machine,
        strings,
        offer,
        chosen,
        the_environment,
        fast_startup,
        &mut journal,
    ) {
        Ok(()) => Ok(()),
        Err(()) => Err(put_back(machine, strings, offer, journal)),
    }
}

/// The person's answer about Fast Startup, and then steps 1 to 7, each change
/// journalled as it succeeds.
fn steps(
    machine: &mut impl TheMachine,
    strings: &Strings,
    offer: &Offer,
    chosen: &ForAloOs,
    the_environment: &TheEnvironment,
    fast_startup: Answer,
    journal: &mut Vec<Made>,
) -> Result<(), ()> {
    // The only step that changes a setting of Windows rather than a disk, and
    // the first: the person answered this question themselves (ADR 0064 term
    // 9), a failure here is a failure before any disk has changed, and it is
    // journalled so that putting the computer back puts the setting back too.
    match fast_startup {
        Answer::LeaveOn => say(
            machine,
            strings,
            words::FAST_STARTUP_LEFT_ON,
            &Filling::nothing(),
        ),
        Answer::TurnOff => {
            say(
                machine,
                strings,
                words::TURNING_FAST_STARTUP_OFF,
                &Filling::nothing(),
            );
            let was = was_on(machine);
            succeeded(machine, &Program::TurningFastStartupOff)?;
            journal.push(Made::FastStartupOff { was });
        }
    }

    let windows = offer.windows;
    let volume = Filling::of("volume", windows.letter.drive()).and("area", sizes::needed(THE_AREA));

    say(machine, strings, words::SHRINKING_WINDOWS, &volume);
    let shrunk = Made::Shrunk {
        disk: windows.disk,
        partition: windows.partition,
        was: windows.size,
    };
    let shrinking = succeeded(
        machine,
        &Program::Shrinking {
            disk: windows.disk,
            partition: windows.partition,
            to: offer.shrink.to,
        },
    );
    if shrinking.is_err() {
        // A shrink that reported failure is asked about rather than believed:
        // saying *nothing was changed* over a Windows that did get smaller is
        // the one sentence this installer may never say falsely. Where the
        // size cannot be read back either, growing it back is attempted.
        let now = succeeded(machine, &Program::ReadingTheWindowsVolume)
            .ok()
            .and_then(|printed| crate::windows_volume::WindowsVolume::read(Some(&printed)));
        if now.is_none_or(|now| now.size != windows.size) {
            journal.push(shrunk);
        }
        return Err(());
    }
    journal.push(shrunk);

    say(
        machine,
        strings,
        words::MAKING_THE_AREA,
        &Filling::of("disk", offer.windows_disk.as_str()),
    );
    let made = succeeded(
        machine,
        &Program::MakingTheArea {
            disk: windows.disk,
            offset: offer.shrink.area_begins,
            size: THE_AREA,
        },
    )?;
    let Ok(made) = serde_json::from_str::<MadeTheArea>(&made) else {
        journal.push(Made::AnAreaNotIdentified);
        return Err(());
    };
    let partition = PartitionNumber(made.partition_number);
    journal.push(Made::TheArea {
        disk: windows.disk,
        partition,
        offset: made.offset,
    });
    if made.offset != offer.shrink.area_begins {
        return Err(());
    }
    let prepared = succeeded(
        machine,
        &Program::PreparingTheArea {
            disk: windows.disk,
            partition,
            offset: made.offset,
        },
    )?;
    let letter = serde_json::from_str::<PreparedTheArea>(&prepared)
        .ok()
        .and_then(|prepared| Letter::of(&prepared.drive_letter))
        .ok_or(())?;

    say(
        machine,
        strings,
        words::COPYING_THE_INSTALLER,
        &Filling::nothing(),
    );
    let root = std::path::PathBuf::from(letter.root());
    let choice = environment::the_choice(&chosen.after_the_restart);
    for (inside, bytes) in the_environment
        .files()
        .iter()
        .map(|(inside, bytes)| (inside.as_str(), bytes.as_slice()))
        .chain([(environment::THE_CHOICE, choice.as_bytes())])
    {
        let file = environment::beneath(&root, inside);
        machine.write(&file, bytes).map_err(|_| ())?;
        let back = machine.read(&file).map_err(|_| ())?;
        if !environment::is_the_same(bytes, &back) {
            return Err(());
        }
    }

    say(
        machine,
        strings,
        words::ADDING_THE_ENTRY,
        &Filling::nothing(),
    );
    // Written into the firmware itself rather than copied from Windows' boot
    // manager, so it carries none of Windows' optional data (`program.rs`).
    // The program removes what it wrote when it fails after writing, so a
    // failure here leaves no entry to put back.
    let printed = succeeded(
        machine,
        &Program::WritingTheEntry {
            disk: windows.disk,
            partition,
            offset: made.offset,
        },
    )?;
    let Some(entry) = serde_json::from_str::<WroteTheEntry>(&printed)
        .ok()
        .and_then(|wrote| Entry::of(&wrote.identifier))
    else {
        journal.push(Made::AnEntryNotIdentified);
        return Err(());
    };
    journal.push(Made::TheEntry(entry.clone()));
    succeeded(
        machine,
        &Program::ListingTheEntry {
            entry: entry.clone(),
        },
    )?;
    succeeded(
        machine,
        &Program::TakingAwayTheLetter {
            disk: windows.disk,
            partition,
            letter,
        },
    )?;

    // Journalled before it runs: a next start that failed half way is one
    // this installer forgets anyway, and forgetting one that was never set
    // changes nothing.
    journal.push(Made::TheNextStart);
    succeeded(machine, &Program::StartingTheEntryNext { entry })?;
    Ok(())
}

/// Put back everything in the journal, newest first, and say how that ended.
fn put_back(
    machine: &mut impl TheMachine,
    strings: &Strings,
    offer: &Offer,
    journal: Vec<Made>,
) -> Ended {
    say(
        machine,
        strings,
        words::PUTTING_IT_BACK,
        &Filling::nothing(),
    );
    let mut remains = Vec::new();
    let mut area_remains = false;
    for made in journal.into_iter().rev() {
        match made {
            // Put back to the value Windows held, not to a value this
            // installer chose: a computer whose value was something else is
            // left as it was found.
            Made::FastStartupOff { was } => {
                if succeeded(machine, &Program::TurningFastStartupBackOn { was }).is_err() {
                    remains.push(Remains::FastStartupOff);
                }
            }
            Made::TheNextStart => {
                if succeeded(machine, &Program::ForgettingTheNextStart).is_err() {
                    remains.push(Remains::TheNextStart);
                }
            }
            Made::TheEntry(entry) => {
                if succeeded(machine, &Program::RemovingTheEntry { entry }).is_err() {
                    remains.push(Remains::TheEntry);
                }
            }
            Made::AnEntryNotIdentified => remains.push(Remains::TheEntry),
            Made::TheArea {
                disk,
                partition,
                offset,
            } => {
                let removed = succeeded(
                    machine,
                    &Program::RemovingTheArea {
                        disk,
                        partition,
                        offset,
                    },
                );
                if removed.is_err() {
                    area_remains = true;
                    remains.push(Remains::TheArea(offer.windows_disk.clone()));
                }
            }
            Made::AnAreaNotIdentified => {
                area_remains = true;
                remains.push(Remains::TheArea(offer.windows_disk.clone()));
            }
            Made::Shrunk {
                disk,
                partition,
                was,
            } => {
                // Windows grows into the space the area was in, so with the
                // area still there it cannot, and is not asked to.
                let grown = !area_remains
                    && succeeded(
                        machine,
                        &Program::GrowingWindowsBack {
                            disk,
                            partition,
                            to: was,
                        },
                    )
                    .is_ok();
                if !grown {
                    remains.push(Remains::Smaller(offer.windows.letter));
                }
            }
        }
    }
    if remains.is_empty() {
        Ended::Refused(Refusal::PutBack)
    } else {
        Ended::NotPutBack(remains)
    }
}

/// What Windows' Fast Startup value held before it was turned off.
///
/// The value is read again here rather than carried from the checks: the
/// question was asked about a value read minutes ago, and putting a setting
/// back means putting back what was actually there. A value that cannot be
/// read now is put back as Windows' own default, `1`, which is what *on*
/// means — the answer was given about a computer whose Fast Startup was on.
fn was_on(machine: &mut impl TheMachine) -> u32 {
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Read {
        /// The value.
        hiberboot_enabled: Option<u32>,
    }

    succeeded(machine, &Program::ReadingFastStartup)
        .ok()
        .and_then(|printed| serde_json::from_str::<Read>(&printed).ok())
        .and_then(|read| read.hiberboot_enabled)
        .filter(|value| *value != 0)
        .unwrap_or(1)
}

/// What a change printed, when it ran and succeeded.
fn succeeded(machine: &mut impl TheMachine, program: &Program) -> Result<String, ()> {
    match machine.run(program) {
        Ok(ran) if ran.succeeded => Ok(ran.printed),
        Ok(_) | Err(_) => Err(()),
    }
}

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}
