//! Which system this computer starts when nobody chooses, changed from
//! Windows.
//!
//! [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
//! term 1: the answer is kept **once**, in the loader's own environment block
//! on the EFI system partition, *the one filesystem both systems can read and
//! write*. Term 3: Windows changes it through *the small program the installer
//! leaves behind, which already runs elevated, and writes the same file on the
//! same partition. It is the Windows side of one setting, not a second
//! setting.* This file is that side.
//!
//! # It is the same code, not the same shape
//!
//! Nothing here parses or writes an environment block. `alo-starting` owns
//! that format — [`EnvironmentBlock`], and [`TheStartingChoice`] over it — and
//! this crate calls it. A second implementation of a file two systems share is
//! how the two answers ADR 0066 forbids would come back, even if both were
//! written carefully.
//!
//! # What it touches
//!
//! The EFI system partition, given a drive letter for as long as the change
//! takes and taken away again; one file on it, written at exactly the length
//! it was read at; and nothing else. No disk is repartitioned, no start-up
//! entry is added or removed, and the firmware's own next start — which is
//! `crate::switching`'s one act — is not touched.

use alo_starting::{EnvironmentBlock, System, THE_BLOCK_ON_THE_ESP, TheStartingChoice};
use alo_strings::{Filling, Strings, Word};

use crate::identities::Letter;
use crate::machine::TheMachine;
use crate::program::Program;
use crate::words;

/// The argument that makes this program the one that changes the default.
pub const THE_DEFAULTS_WORD: &str = "which-system-starts";

/// The letter the EFI system partition is given while it is being read or
/// written.
///
/// `S:` is what Windows' own documentation uses in every example of mounting
/// it, and the walk's guest uses the same, so a person who looks while this
/// runs sees the letter they have read about.
pub const THE_START_PARTITIONS_LETTER: &str = "S";

/// How the default was left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheDefault {
    /// It is this system now, and it was written.
    Changed(System),
    /// It is this system, and the person left it alone.
    Kept(System),
    /// The loader's file is not where both systems keep it.
    NotThere,
    /// The file is there and is not an environment block, or could not be
    /// written back at the length it was read at.
    NotRead,
    /// The EFI system partition could not be reached from Windows.
    NotReached,
}

/// Show which system starts when nobody chooses, and change it if the person
/// says so.
pub fn which_system_starts(machine: &mut impl TheMachine, strings: &Strings) -> TheDefault {
    say(
        machine,
        strings,
        words::DEFAULT_STARTING,
        &Filling::nothing(),
    );
    let Some(letter) = Letter::of(THE_START_PARTITIONS_LETTER) else {
        return TheDefault::NotReached;
    };
    if ran(machine, &Program::GivingTheStartPartitionALetter { letter }).is_none() {
        say(
            machine,
            strings,
            words::DEFAULT_NOT_REACHED,
            &Filling::nothing(),
        );
        return TheDefault::NotReached;
    }
    let ended = with_the_partition(machine, strings, letter);
    // Taken away again whatever happened, because a start partition left with
    // a letter is a partition every program on the computer can write into.
    let _ = ran(
        machine,
        &Program::TakingTheStartPartitionsLetterAway { letter },
    );
    let (word, filling) = ended.said_as();
    say(machine, strings, word, &filling);
    ended
}

/// The reading, the question and the writing, with the partition reachable.
fn with_the_partition(
    machine: &mut impl TheMachine,
    strings: &Strings,
    letter: Letter,
) -> TheDefault {
    let file = the_block(letter);
    let Ok(bytes) = machine.read(&file) else {
        return TheDefault::NotThere;
    };
    let Ok(mut block) = EnvironmentBlock::read(&bytes) else {
        return TheDefault::NotRead;
    };
    let now = TheStartingChoice::read(&block);
    say(
        machine,
        strings,
        words::DEFAULT_IS,
        &Filling::of("system", system_said(now, strings)),
    );

    let other = match now {
        System::AloOs => System::Windows,
        System::Windows => System::AloOs,
    };
    let typed = machine.ask(&strings.say(
        &words::DEFAULT_TYPE_TO_CHANGE.key(),
        &Filling::of("system", system_said(other, strings)),
    ));
    if !crate::asking::is_the_word(&typed, words::DEFAULT_CHANGE_IT, strings) {
        return TheDefault::Kept(now);
    }

    if TheStartingChoice::write(&mut block, other).is_err() {
        return TheDefault::NotRead;
    }
    let Ok(written) = block.written() else {
        return TheDefault::NotRead;
    };
    if machine.write(&file, &written).is_err() {
        return TheDefault::NotRead;
    }
    // Read back through the same reader the loader's side uses: a write that
    // did not take is not a change, and nothing here says one happened.
    let Ok(back) = machine.read(&file) else {
        return TheDefault::NotRead;
    };
    match EnvironmentBlock::read(&back) {
        Ok(back) if TheStartingChoice::read(&back) == other => TheDefault::Changed(other),
        Ok(_) | Err(_) => TheDefault::NotRead,
    }
}

/// The loader's own file, named from Windows: the partition's letter and the
/// one path `alo-starting` says it is at.
#[must_use]
pub fn the_block(letter: Letter) -> std::path::PathBuf {
    let inside = THE_BLOCK_ON_THE_ESP.replace('/', "\\");
    std::path::PathBuf::from(format!("{}{inside}", letter.drive()))
}

/// What a system is called where a person reads it.
fn system_said(system: System, strings: &Strings) -> String {
    let word = match system {
        System::AloOs => words::THE_SYSTEM_ALO_OS,
        System::Windows => words::THE_SYSTEM_WINDOWS,
    };
    strings.say(&word.key(), &Filling::nothing()).into_text()
}

impl TheDefault {
    /// The sentence this ending is said in, and what fills it.
    #[must_use]
    fn said_as(&self) -> (Word, Filling) {
        match self {
            Self::Changed(_) => (words::DEFAULT_CHANGED, Filling::nothing()),
            Self::Kept(_) => (words::DEFAULT_KEPT, Filling::nothing()),
            Self::NotThere => (words::DEFAULT_NOT_THERE, Filling::nothing()),
            Self::NotRead => (words::DEFAULT_NOT_READ, Filling::nothing()),
            Self::NotReached => (words::DEFAULT_NOT_REACHED, Filling::nothing()),
        }
    }
}

/// What a program printed, when it ran and succeeded.
fn ran(machine: &mut impl TheMachine, program: &Program) -> Option<String> {
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
