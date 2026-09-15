//! The one process the boot environment runs.
//!
//! It reads what the recipe built into it, installs or refuses through
//! `alo_installing::install`, and ends. Every decision is in the library, where
//! it is tested; this is the order, and what an exit code means.
//!
//! # It speaks English until it is told a language
//!
//! The environment is staged by the program a person ran in their own language,
//! and that program (the installer plan's task 3) is where the language and the
//! translations for it come from. Until it stages them, every sentence here is
//! the English beside its key — marked as untranslated by `alo-strings`, never
//! written into this file.
//!
//! # And it runs on Linux
//!
//! On any other host there is no kernel command line, no udev and no disk to
//! write, and this process says so and ends.

use std::process::ExitCode;

/// What this process does inside the boot environment.
#[cfg(target_os = "linux")]
mod running {
    use std::path::Path;
    use std::process::ExitCode;

    use alo_installing::{
        Ended, Environment, OnThisMachine, WHERE_IT_IS, install, installing_words,
    };
    use alo_strings::Strings;

    /// Install, and say how it ended in the exit code as well as on the screen.
    pub(crate) fn run() -> ExitCode {
        let vocabulary = match installing_words() {
            Ok(vocabulary) => vocabulary,
            Err(why) => {
                eprintln!("alo-installing: its own words would not declare: {why}");
                return ExitCode::FAILURE;
            }
        };
        let strings = Strings::of(vocabulary);
        let inside = Path::new(WHERE_IT_IS);
        let environment = Environment::read(
            inside,
            std::fs::read_to_string(inside.join(alo_image::THE_PIN)),
            |key| std::fs::File::open(key).is_ok(),
        );
        match install(
            &mut OnThisMachine,
            &strings,
            environment.as_ref().map_err(Clone::clone),
        ) {
            Ended::Installed(_) => ExitCode::SUCCESS,
            Ended::Refused(_) | Ended::NotInstalled(_) => ExitCode::FAILURE,
        }
    }
}

fn main() -> ExitCode {
    #[cfg(target_os = "linux")]
    {
        running::run()
    }
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("alo-installing runs inside the alo OS boot environment, which is Linux.");
        ExitCode::FAILURE
    }
}
