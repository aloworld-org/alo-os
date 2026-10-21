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
        Ended, Environment, OnThisMachine, Told, WHERE_IT_IS, install, installing_words,
    };
    use alo_strings::Strings;

    /// The eight layouts the environment offers, and what each calls itself.
    ///
    /// The names are endonyms and are never translated — a person choosing a
    /// keyboard may not yet be able to read the interface language, and in any
    /// case a layout's own name is the same in every one. They sit here rather
    /// than in `words.rs` for exactly that reason, which is how this crate
    /// already treats a disk's own name.
    ///
    /// The order is the one `alo_keyboards::EVERY_LAYOUT` offers them in, and
    /// the **first is the one the keyboard screen shows as its guess**. Until
    /// the Windows road stages a better guess, that is Belgian — which is a
    /// guess the screen does not ask a person to trust: it asks them to press
    /// keys and see whether what appears matches what is printed on them.
    const THE_LAYOUTS_OFFERED: &[(&str, &str)] = &[
        ("be", "Belgisch"),
        ("fr", "Français"),
        ("de", "Deutsch"),
        ("es", "Español"),
        ("it", "Italiano"),
        ("nl", "Nederlands"),
        ("gb", "English (UK)"),
        ("us", "English (US)"),
    ];

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

        // **Ask, where nobody has been asked.** The Windows road stages the
        // answers on the kernel command line before the restart, and a screen
        // asking them again would be asking a person to repeat themselves. The
        // stick road stages nothing, and until now the environment refused it
        // with *no disk chosen* — a true sentence about a question it had never
        // put to anybody.
        //
        // **What this does not yet do**, said here rather than left to be
        // discovered: the walk collects a keyboard, a network, its password
        // and a disk, and then the install below still reads the disk off the
        // command line. **Joining the network and installing onto the disk a
        // person chose on screen is the next change**, and until it lands a
        // person on the stick road meets every screen, answers every question,
        // and then meets the same refusal as before.
        let line = std::fs::read_to_string("/proc/cmdline").unwrap_or_default();
        if Told::from_the_command_line(&line).is_err() {
            // Both lists are handed in rather than reached for, because the
            // asking holds no machine of its own — which is what lets the
            // whole walk be tested without one.
            //
            // **Neither is read here yet, and that is the honest state.**
            // Listing the disks is `Program::ListingTheDisks` and belongs in
            // the same change that installs onto the disk a person chose;
            // scanning for networks is **not a `Program` at all** yet, so
            // there is nothing to call. With no networks the walk passes over
            // the network question rather than showing an empty list, which is
            // what a computer on a cable should see in any case.
            let networks: Vec<alo_installing::ANetwork> = Vec::new();
            let disks: Vec<alo_installing::Offered> = Vec::new();

            match alo_installing::ask(&strings, THE_LAYOUTS_OFFERED, &networks, &disks) {
                Ok(Some(answers)) => println!(
                    "alo-installing: the keyboard is {}, the network is {}, the disk is {}",
                    answers.layout,
                    answers.network.as_deref().unwrap_or("a cable"),
                    answers.disk
                ),
                // No display, no keyboard, or nobody finished — all of which
                // the console below already covers, which is what a person
                // read before this existed.
                Ok(None) => {}
                Err(why) => eprintln!("alo-installing: the screen could not be drawn: {why}"),
            }
        }

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
