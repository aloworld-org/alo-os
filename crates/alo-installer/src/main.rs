//! The program a person downloads and runs.
//!
//! It reads nothing but what `alo_installer::install` asks the machine, installs
//! or refuses through it, and ends. Every decision is in the library, where it
//! is tested; this is the order, and what an exit code means.
//!
//! # It speaks English until it is given a language
//!
//! Every sentence is the English beside its key, marked as untranslated by
//! `alo-strings` and never written into this file. Carrying translations inside
//! the download, and choosing among them from Windows' own display language,
//! comes with the Release that ships them (the installer plan's task 5).
//!
//! # And it runs on Windows
//!
//! On any other host there is no disk Windows manages and no start-up list to
//! add to, and this process says so and ends.

use std::process::ExitCode;

/// What this process does on Windows.
#[cfg(windows)]
mod running {
    use std::process::ExitCode;

    use alo_installer::{
        Ended, OnThisMachine, PRESS_ENTER_TO_CLOSE, Released, Removed, Started, Switched,
        THE_DEFAULTS_WORD, THE_REMOVALS_WORD, THE_STICKS_WORD, THE_SWITCHS_WORD, THE_WRITINGS_WORD,
        TheDefault, TheMachine, Written, install, remove_alo_os, restart_into_alo_os,
        start_the_stick, what_this_program_says, which_system_starts, write_the_stick,
    };
    use alo_strings::{Filling, Strings};

    /// Install, and say how it ended in the exit code as well as on the screen.
    ///
    /// **Started with one of this crate's own words as its only argument it does
    /// one other thing instead**: `THE_SWITCHS_WORD` offers to restart this
    /// computer into alo OS, `THE_STICKS_WORD` offers to restart it into a
    /// device a person plugs in, `THE_DEFAULTS_WORD` shows and changes which
    /// system it starts when nobody chooses, and `THE_REMOVALS_WORD` removes
    /// alo OS again. That is
    /// how the copy the installer leaves behind is started, a start with no
    /// argument is an install as it always was, and any other argument is
    /// refused rather than guessed at.
    pub(crate) fn run() -> ExitCode {
        let vocabulary = match what_this_program_says() {
            Ok(vocabulary) => vocabulary,
            Err(why) => {
                eprintln!("alo-installer: its own words would not declare: {why}");
                return ExitCode::FAILURE;
            }
        };
        let strings = Strings::of(vocabulary);
        let mut machine = match OnThisMachine::found() {
            Ok(machine) => machine,
            Err(why) => {
                eprintln!("alo-installer: Windows' system directory was not found: {why}");
                return ExitCode::FAILURE;
            }
        };
        let arguments: Vec<String> = std::env::args().skip(1).collect();
        match arguments.as_slice() {
            [] => {}
            [one] if one == THE_SWITCHS_WORD => {
                let switched = restart_into_alo_os(&mut machine, &strings);
                if switched == (Switched::Restarting { restarted: true }) {
                    return ExitCode::SUCCESS;
                }
                let _read =
                    machine.ask(&strings.say(&PRESS_ENTER_TO_CLOSE.key(), &Filling::nothing()));
                return match switched {
                    Switched::Restarting { .. } | Switched::NotAgreed => ExitCode::SUCCESS,
                    Switched::NotThere | Switched::NotRead | Switched::NotSet => ExitCode::FAILURE,
                };
            }
            [one] if one == THE_REMOVALS_WORD => {
                let removed = remove_alo_os(&mut machine, &strings);
                let _read =
                    machine.ask(&strings.say(&PRESS_ENTER_TO_CLOSE.key(), &Filling::nothing()));
                return match removed {
                    Removed::Gone { .. } | Removed::NotAgreed => ExitCode::SUCCESS,
                    Removed::NotThere
                    | Removed::NotFound
                    | Removed::NotRead
                    | Removed::EntryNotRemoved
                    | Removed::DiskNotCleared => ExitCode::FAILURE,
                };
            }
            // **The road that starts a device a person plugs in.** The same
            // shape as the switch above it: one word, one act, nothing on any
            // disk changed. `docs/features.md`: *and Windows starts the stick,
            // so nobody hunts for a boot key.*
            [one] if one == THE_STICKS_WORD => {
                let started = start_the_stick(&mut machine, &strings);
                if started == (Started::Restarting { restarted: true }) {
                    return ExitCode::SUCCESS;
                }
                let _read =
                    machine.ask(&strings.say(&PRESS_ENTER_TO_CLOSE.key(), &Filling::nothing()));
                return match started {
                    // A person who declined, or a device the firmware does not
                    // list, are both ordinary ends rather than failures: the
                    // machine is as it was and they were told why.
                    Started::Restarting { .. }
                    | Started::NotAgreed
                    | Started::NotThatName
                    | Started::NotListed
                    | Started::NothingPluggedIn => ExitCode::SUCCESS,
                    Started::NotRead | Started::NotSet => ExitCode::FAILURE,
                };
            }
            // **The road that writes a device a person plugs in**, and the
            // sibling of the one above it: that one starts a stick that already
            // exists, this one makes one. `docs/features.md`, the owner's
            // ruling of 2026-10-20: *let's build 1 and 2.*
            //
            // **The only road in this program that destroys something nobody
            // asked alo OS to install onto**, which is why every refusal is
            // reported and only two outcomes touched the device at all.
            [one] if one == THE_WRITINGS_WORD => {
                let written = write_the_stick(&mut machine, &strings);
                let _read =
                    machine.ask(&strings.say(&PRESS_ENTER_TO_CLOSE.key(), &Filling::nothing()));
                return match written {
                    // A person who declined, a name that matched nothing, a
                    // device too small and a machine with nothing plugged in are
                    // all ordinary ends rather than failures: the device is as
                    // it was and they were told why.
                    Written::Written { .. }
                    | Written::NotAgreed
                    | Written::NotThatName
                    | Written::TooSmall { .. }
                    | Written::NothingPluggedIn => ExitCode::SUCCESS,
                    // These two are a machine that could not do what was asked:
                    // nothing to write, or a device that stopped answering with
                    // the stick now unusable.
                    Written::NoImage | Written::NotRead | Written::NotWritten { .. } => {
                        ExitCode::FAILURE
                    }
                };
            }
            [one] if one == THE_DEFAULTS_WORD => {
                let ended = which_system_starts(&mut machine, &strings);
                let _read =
                    machine.ask(&strings.say(&PRESS_ENTER_TO_CLOSE.key(), &Filling::nothing()));
                return match ended {
                    TheDefault::Changed(_) | TheDefault::Kept(_) => ExitCode::SUCCESS,
                    TheDefault::NotThere | TheDefault::NotRead | TheDefault::NotReached => {
                        ExitCode::FAILURE
                    }
                };
            }
            _ => {
                eprintln!(
                    "alo-installer: it takes no argument, or {THE_SWITCHS_WORD}, \
                     {THE_STICKS_WORD}, {THE_WRITINGS_WORD}, {THE_DEFAULTS_WORD} or \
                     {THE_REMOVALS_WORD} \
                     and nothing else."
                );
                return ExitCode::FAILURE;
            }
        }
        let ended = install(&mut machine, &strings, Released::this_build());
        if ended == (Ended::Staged { restarted: true }) {
            return ExitCode::SUCCESS;
        }
        // The window a double-click opened closes when this process ends, and a
        // person has to be able to read why it stopped.
        let _read = machine.ask(&strings.say(&PRESS_ENTER_TO_CLOSE.key(), &Filling::nothing()));
        match ended {
            Ended::Staged { .. } => ExitCode::SUCCESS,
            Ended::Refused(_) | Ended::NotPutBack(_) => ExitCode::FAILURE,
        }
    }
}

fn main() -> ExitCode {
    #[cfg(windows)]
    {
        running::run()
    }
    #[cfg(not(windows))]
    {
        eprintln!("alo-installer runs on Windows, on the computer alo OS is installed onto.");
        ExitCode::FAILURE
    }
}
