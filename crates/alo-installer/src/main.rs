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
        Ended, OnThisMachine, PRESS_ENTER_TO_CLOSE, Released, Switched, THE_DEFAULTS_WORD,
        THE_SWITCHS_WORD, TheDefault, TheMachine, install, installer_words, restart_into_alo_os,
        which_system_starts,
    };
    use alo_strings::{Filling, Strings};

    /// Install, and say how it ended in the exit code as well as on the screen.
    ///
    /// **Started with `THE_SWITCHS_WORD` as its only argument it does one other
    /// thing instead**: it offers to restart this computer into alo OS. That is
    /// how the copy the installer leaves behind is started, a start with no
    /// argument is an install as it always was, and any other argument is
    /// refused rather than guessed at.
    pub(crate) fn run() -> ExitCode {
        let vocabulary = match installer_words() {
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
                    "alo-installer: it takes no argument, or {THE_SWITCHS_WORD} or                      {THE_DEFAULTS_WORD} and nothing else."
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
