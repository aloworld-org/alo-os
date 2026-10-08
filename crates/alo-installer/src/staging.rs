//! Preparing the computer, once the person has agreed — and putting it back
//! when preparing fails.
//!
//! In order, each step said before it begins:
//!
//! 1. turn Fast Startup off, where the person answered that it should be;
//! 2. shrink the Windows partition — **by the area alone on the road that gives
//!    alo OS a disk of its own, and by the area together with alo OS's space on
//!    the road that keeps Windows**, in one shrink either way;
//! 3. make the area in the space that freed, at the place it freed;
//! 4. format it with the installer's label and give it a drive letter;
//! 5. **on the road that keeps Windows only**, make alo OS's own partition in
//!    the rest of the freed space and label it `ALO-ROOT`, with no drive letter;
//! 6. write the environment's files and the person's choice onto the area, and
//!    read each one back;
//! 7. leave a copy of this program and a Start-menu shortcut, so a person can
//!    get back in from inside Windows (`crate::switching`);
//! 8. write a start-up entry named alo OS pointing at the area's loader into
//!    the firmware — with none of the optional data a copy of Windows' own
//!    entry carries — and list it last so Windows stays the one the computer
//!    starts normally;
//! 9. take the area's letter away again;
//! 10. make the entry the next start, once — the firmware's next-boot choice,
//!     which never changes the default (`docs/booting.md`).
//!
//! **This list went stale once and nothing caught it.** Steps 1, 5 and 7 were
//! missing and step 2 said *by exactly the area*, which the road that keeps
//! Windows made false. It was found by the testing lane reading its own copy of
//! this file against a checklist written from another — so a reader was going
//! to act on one of them. Nothing in the gate compares a module's prose to its
//! own code, so **a step added here is added to this list in the same change**.
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

use alo_installing::Replacing;
use alo_strings::{Filling, Strings, Word};
use serde::Deserialize;

use crate::asking::Answer;
use crate::deciding::{ForAloOs, Offer};
use crate::ended::{Ended, Refusal, Remains};
use crate::environment::{self, TheEnvironment};
use crate::identities::{DiskNumber, Entry, Letter, PartitionNumber};
use crate::machine::TheMachine;
use crate::naming;
use crate::program::Program;
use crate::sizes::{self, THE_AREA};
use crate::words;

/// One change made, and what putting it back needs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Made {
    /// Windows was shrunk from this size, by this much.
    Shrunk {
        /// Its disk.
        disk: DiskNumber,
        /// Its partition.
        partition: PartitionNumber,
        /// The size it had.
        was: u64,
        /// How much smaller it was made.
        ///
        /// Carried rather than worked out again when putting back, because
        /// what a person is told their Windows lost has to be what it lost:
        /// the amount depends on the road, and the road is not something
        /// `put_back` can see.
        by: u64,
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
    /// alo OS's own space was made here, on the road that keeps Windows.
    ///
    /// Journalled apart from the area because it is put back apart from it:
    /// the area is this installer's and goes when the installer has finished
    /// either way, and this is the person's new partition, which goes only if
    /// something after it fails.
    TheSpace {
        /// Its disk.
        disk: DiskNumber,
        /// Its number.
        partition: PartitionNumber,
        /// Where it begins.
        offset: u64,
    },
    /// Windows said it made alo OS's space, and did not say which partition.
    ///
    /// Nothing can be put back by number here, so putting back says what it
    /// could not do rather than guessing at a partition on a disk somebody's
    /// Windows is on.
    ASpaceNotIdentified,
    /// The entry was made.
    TheEntry(Entry),
    /// Windows said it made an entry, and did not say which.
    AnEntryNotIdentified,
    /// The next start was set.
    TheNextStart,
    /// The copy of this program and its shortcut were left in place.
    TheWayBack,
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
    replacing: Replacing,
) -> Result<(), Ended> {
    let mut journal = Vec::new();
    match steps(
        machine,
        strings,
        offer,
        chosen,
        replacing,
        the_environment,
        fast_startup,
        &mut journal,
    ) {
        Ok(()) => Ok(()),
        Err(()) => Err(put_back(machine, strings, offer, journal)),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the road is the eighth, and it is carried rather than decided: \
             every one of these is something a person chose or a check found, and \
             grouping them would hide which of them the caller may not invent"
)]
/// The person's answer about Fast Startup, and then steps 1 to 7, each change
/// journalled as it succeeds.
fn steps(
    machine: &mut impl TheMachine,
    strings: &Strings,
    offer: &Offer,
    chosen: &ForAloOs,
    replacing: Replacing,
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

    // **How much Windows gives up is the road the person chose**, and the road
    // is carried on the disk they named rather than asked about separately
    // (`deciding::ForAloOs::beside`). Keeping Windows frees the installer's
    // area *and* alo OS's own space, in one shrink: two shrinks would leave a
    // machine half-moved if the second failed, and `BesideWindows` already
    // asked the volume for all of it together or there is no road at all.
    //
    // `None` is the older road and still the common one - an empty disk alo
    // OS takes whole, where Windows gives up nothing but the area.
    let beside = chosen.beside;
    let shrink_to = beside.map_or(offer.shrink.to, |beside| beside.to);
    let area_begins = beside.map_or(offer.shrink.area_begins, |beside| beside.area_begins);

    // **What this says is the number that is about to be taken.** It was
    // the area, a constant gibibyte, because that was the only amount any
    // road had ever taken - and on the road that keeps Windows it would
    // have asked a person to watch a gigabyte go and taken sixty.
    let volume = Filling::of("volume", windows.letter.drive())
        .and("area", sizes::taken(windows.size.saturating_sub(shrink_to)));
    say(machine, strings, words::SHRINKING_WINDOWS, &volume);
    let shrunk = Made::Shrunk {
        disk: windows.disk,
        partition: windows.partition,
        was: windows.size,
        by: windows.size.saturating_sub(shrink_to),
    };
    let shrinking = succeeded(
        machine,
        &Program::Shrinking {
            disk: windows.disk,
            partition: windows.partition,
            to: shrink_to,
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
            offset: area_begins,
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
    if made.offset != area_begins {
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

    // **alo OS's own space, on the road that keeps Windows - made by
    // Windows rather than after the restart.** A Linux that repartitioned
    // a disk Windows is on, while Windows' own file system was last
    // written by a kernel that has not finished with it, is how somebody
    // loses a Windows. Windows lays out its own disk; alo OS only fills
    // what it was given.
    //
    // The label is the whole of the boot environment's permission to write
    // that partition (`alo_installing::may_keep_what_is_there`), so a
    // space made and not labelled is a space the environment refuses -
    // which is the safe way round for a failure between the two steps.
    let alo_oss_space = match beside {
        None => None,
        Some(beside) => {
            say(
                machine,
                strings,
                words::MAKING_THE_SPACE,
                &Filling::of("space", sizes::taken(beside.alo_os))
                    .and("disk", offer.windows_disk.as_str()),
            );
            let printed = succeeded(
                machine,
                &Program::MakingTheSpace {
                    disk: windows.disk,
                    offset: beside.alo_os_begins,
                    size: beside.alo_os,
                },
            )?;
            let Ok(made) = serde_json::from_str::<MadeTheArea>(&printed) else {
                journal.push(Made::ASpaceNotIdentified);
                return Err(());
            };
            let space = PartitionNumber(made.partition_number);
            journal.push(Made::TheSpace {
                disk: windows.disk,
                partition: space,
                offset: made.offset,
            });
            // Journalled before it is checked, for the area's reason: a
            // partition that exists somewhere other than where it was asked
            // for is still a partition that has to be taken away.
            if made.offset != beside.alo_os_begins {
                return Err(());
            }
            succeeded(
                machine,
                &Program::LabellingTheSpace {
                    disk: windows.disk,
                    partition: space,
                    offset: made.offset,
                },
            )?;
            Some(space)
        }
    };

    say(
        machine,
        strings,
        words::COPYING_THE_INSTALLER,
        &Filling::nothing(),
    );
    let root = std::path::PathBuf::from(letter.root());
    // The road is the caller's, never this file's: it comes from the consent
    // that named it, and `staging.rs` has no way to decide to erase a disk.
    //
    // **Three names on the road that keeps Windows**, where the one name is
    // not enough: the disk alone tells the environment nothing about which
    // partition on it is alo OS's and which one holds the loader Windows
    // already starts from. Both are derived here, from Windows' own numbers,
    // and a number that cannot be turned into a name is a refusal rather
    // than a guess.
    //
    // It cannot say *replace this disk* as well: `the_choice_beside_what_is
    // _there` takes no `Replacing` and so has nothing to say it with. That is
    // the shape rather than a check, which is why there is no test that the
    // check was armed.
    let choice = match (alo_oss_space, offer.the_start_up_area) {
        (Some(space), Some(start_up_area)) => {
            let space = naming::a_partition_after_the_restart(&chosen.after_the_restart, space.0)
                .ok_or(())?;
            let start_up_area =
                naming::a_partition_after_the_restart(&chosen.after_the_restart, start_up_area.0)
                    .ok_or(())?;
            environment::the_choice_beside_what_is_there(
                &chosen.after_the_restart,
                &space,
                &start_up_area,
            )
        }
        // A space was made and the disk could not say where the firmware
        // starts from, or the other way about. Neither is a command line
        // this installer may write: one road's half and the other's half is
        // not a road.
        (Some(_), None) | (None, Some(_)) if beside.is_some() => return Err(()),
        _ => environment::the_choice(&chosen.after_the_restart, replacing),
    };
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

    // The way back in, left where a person finds it: a copy of this program,
    // which does one other thing when it is started with the switch's own word
    // (`crate::switching`), and a shortcut in the Start menu. A copy rather
    // than a second program, so there is nothing else to build, to sign or to
    // keep in step with this one.
    say(
        machine,
        strings,
        words::LEAVING_THE_WAY_BACK,
        &Filling::nothing(),
    );
    let mine = machine.this_program().map_err(|_| ())?;
    let bytes = machine.read(&mine).map_err(|_| ())?;
    let left = std::path::Path::new(crate::program::THE_PROGRAMS_HOME)
        .join(crate::program::THE_PROGRAMS_NAME);
    machine.write(&left, &bytes).map_err(|_| ())?;
    if machine.read(&left).map_err(|_| ())? != bytes {
        return Err(());
    }
    succeeded(machine, &Program::MakingTheShortcut)?;
    journal.push(Made::TheWayBack);

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
    // **Anything left in the freed region stops Windows growing back**, and
    // on the road that keeps Windows there are two things that can be left
    // there. Named for what it means rather than for the area, because a
    // flag called `area_remains` that is also set by a space is the kind of
    // name the next reader believes.
    let mut the_freed_space_is_taken = false;
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
            Made::TheWayBack => {
                if succeeded(machine, &Program::RemovingWhatWasLeft).is_err() {
                    remains.push(Remains::TheWayBack);
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
                    the_freed_space_is_taken = true;
                    remains.push(Remains::TheArea(offer.windows_disk.clone()));
                }
            }
            Made::AnAreaNotIdentified => {
                the_freed_space_is_taken = true;
                remains.push(Remains::TheArea(offer.windows_disk.clone()));
            }
            Made::TheSpace {
                disk,
                partition,
                offset,
            } => {
                let removed = succeeded(
                    machine,
                    &Program::RemovingTheSpace {
                        disk,
                        partition,
                        offset,
                    },
                );
                if removed.is_err() {
                    the_freed_space_is_taken = true;
                    remains.push(Remains::TheSpace(offer.windows_disk.clone()));
                }
            }
            Made::ASpaceNotIdentified => {
                the_freed_space_is_taken = true;
                remains.push(Remains::TheSpace(offer.windows_disk.clone()));
            }
            Made::Shrunk {
                disk,
                partition,
                was,
                by,
            } => {
                // Windows grows into the region the shrink freed, so with
                // anything still in it - the area, or alo OS's own space - it
                // cannot, and is not asked to.
                let grown = !the_freed_space_is_taken
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
                    remains.push(Remains::Smaller {
                        volume: offer.windows.letter,
                        by,
                    });
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
