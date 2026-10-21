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
        DiskName, Disks, Ended, Environment, OnThisMachine, Program, STILL_EVERY, TheMachine,
        TheMedium, Told, WHERE_IT_IS, WhyThereIsNoNetwork, every_network, install,
        installing_words, what_may_be_offered, words,
    };
    use alo_strings::{Filling, Strings};

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

    /// Ask a person everything, and give the install the disk they chose.
    ///
    /// [`None`] where there was no display, no keyboard, or they never
    /// finished — and then the install reads the command line as it always
    /// has, finds nothing, and refuses in words a person can act on. **A
    /// refusal somebody can read beats a program that stops.**
    ///
    /// # What is still owed here, and it is one thing
    ///
    /// The network a person chose, and its password, are collected and **not
    /// yet joined**. `joining_a_network` has the two commands and the
    /// `--temporary` flag that keeps the password out of any keyfile; what it
    /// has never had is a caller. That is the next change, and it is the last
    /// one between this and an install a person can finish over Wi-Fi.
    fn the_screens_answer(strings: &Strings) -> Option<Told> {
        // Both lists are read before anything is drawn, and handed in rather
        // than reached for: the asking holds no machine of its own, which is
        // what lets the whole walk be tested without one.
        //
        // **A failure is a short list rather than a stop.** A machine with no
        // wireless card cannot scan, and the walk then passes over the network
        // question — which is also what a computer on a cable should see.
        let scanned = OnThisMachine
            .run(
                &Program::ScanningForNetworks,
                &strings.say(&words::CONNECTING.key(), &Filling::nothing()),
                STILL_EVERY,
            )
            .ok()
            .filter(|ran| ran.succeeded)
            .map(|ran| every_network(&ran.printed))
            .unwrap_or_default();

        let listed = OnThisMachine
            .run(
                &Program::ListingTheDisks,
                &strings.say(&words::LOOKING_FOR_THE_DISK.key(), &Filling::nothing()),
                STILL_EVERY,
            )
            .ok()
            .filter(|ran| ran.succeeded)
            .and_then(|ran| Disks::read(&ran.printed).ok());
        let offered = listed
            .as_ref()
            .map(|disks| what_may_be_offered(disks, &TheMedium::of(disks), strings).offered)
            .unwrap_or_default();

        let answers = alo_installing::ask(strings, THE_LAYOUTS_OFFERED, &scanned, &offered)
            .inspect_err(|why| eprintln!("alo-installing: the screen could not be drawn: {why}"))
            .ok()
            .flatten()?;

        // Noted and not said: the vocabulary never names a device, and
        // whoever helps somebody afterwards needs to know which one it was.
        // **The password is not among these**, and never is: it reaches the
        // network manager's arguments and nothing that is written down.
        OnThisMachine.note(&format!(
            "the screens: keyboard {}, network {}, disk {}",
            answers.layout,
            answers.network.as_deref().unwrap_or("none, a cable"),
            answers.disk
        ));

        // **Join it, where one was chosen.** A cable needs nothing done, and
        // the install waits for a connection by itself either way.
        //
        // A join that fails is **not** a refusal to install: the install waits
        // for the network next and says which of five things went wrong in
        // words a person can act on, through `why_there_is_no_network`. Two
        // refusals for one fault would make a person read the first and act on
        // it while the second was the true one.
        if let Some(network) = answers.network.as_deref() {
            let still = strings.say(&words::CONNECTING.key(), &Filling::nothing());
            // **Asked of the kernel, not assumed.** This was `true` for one
            // commit and that was the same fault as a bool standing in for an
            // exit code: with it hardcoded, `NotJoined::NoCard` could never
            // be the answer, so a person whose wireless card has no firmware
            // would have been told their password was refused.
            //
            // The kernel's own words rather than the network manager's,
            // because `nmcli` prints a device's state as a TRANSLATED string
            // and this would match on an English machine and silently match
            // nothing on a German one -- which is why
            // `why_there_is_no_network` reads `ip -json link` instead.
            let there_is_a_card = !matches!(
                WhyThereIsNoNetwork::read(
                    OnThisMachine
                        .run(
                            &Program::AskingAboutTheNetworkCards,
                            &strings.say(&words::CONNECTING.key(), &Filling::nothing()),
                            STILL_EVERY,
                        )
                        .ok()
                        .filter(|ran| ran.succeeded)
                        .map(|ran| ran.printed)
                        .as_deref(),
                ),
                WhyThereIsNoNetwork::NoCard
            );
            match alo_installing::join(
                &mut OnThisMachine,
                &still,
                network,
                &answers.password,
                there_is_a_card,
            ) {
                Ok(()) => OnThisMachine.note(&format!("joined {network}")),
                Err(why) => OnThisMachine.note(&format!("{network} was not joined: {why:?}")),
            }
        }

        // **A disk whose name the install will not accept is not installed
        // onto.** The screens offer only what the machine listed, so this
        // cannot normally fail — and if it ever did, refusing here is the
        // difference between a refusal a person reads and a write addressed to
        // a name nothing checked.
        DiskName::named(&answers.disk)
            .inspect_err(|why| {
                OnThisMachine.note(&format!("the disk chosen on screen: {why}"));
            })
            .ok()
            .map(Told::taking_this_disk_whole)
    }

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
        // stick road stages nothing, and until this existed the environment
        // refused it with *no disk chosen* — a true sentence about a question
        // it had never put to anybody.
        let line = std::fs::read_to_string("/proc/cmdline").unwrap_or_default();
        let chosen = if Told::from_the_command_line(&line).is_ok() {
            // The Windows road. Nothing is asked and nothing is passed: the
            // install reads the line itself, exactly as it always has.
            None
        } else {
            the_screens_answer(&strings)
        };

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
            chosen,
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
