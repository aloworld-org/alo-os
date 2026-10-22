//! `alo-back`: put one folder back as it was, from a place this machine kept.
//!
//! ```text
//! alo-back                      # the places you can return to
//! alo-back <place> <folder>      # put that folder back as it was
//! ```
//!
//! The surface for ★ *and your files are as they were*, in the one place law 2
//! says a person is trusted completely: **alo OS ships a terminal, because a
//! system that does not trust its owner with a shell is a toy.** Nothing here is
//! privileged, nothing here is an agent's, and there is no verb for it —
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s second
//! term forbids one by name: *an agent choosing which of a person's approvals to
//! reverse is the thing the plan's constraint forbids.*
//!
//! # One approval, and it is the sentence
//!
//! The person is shown exactly what will happen — which folder, from when, and
//! where what is in it now will be afterwards — and types the folder's own name
//! to go on. That is the same consent alo OS already takes before it writes a
//! removable drive, and it catches the second mistake as well as the first: a
//! person who meant a different folder types a name that does not match.
//!
//! Approval is never a session. There is no flag that skips it and will not be
//! one: a person who wants no question has a shell and a snapshot directory they
//! own, and a flag here would be a flag an agent could find.
//!
//! # Its exit code is true
//!
//! Unlike `alo-before`, which runs inside a prompt string and must never touch
//! `$?`, this is a command somebody ran on purpose. Zero when the folder went
//! back, non-zero when it did not — so it can be used in a script by the person
//! whose files they are.

//! # And it runs on Linux
//!
//! The filesystem the whole feature rests on is this operating system's, and so
//! is the one call in `alo_as_it_was` that has to be atomic. On any other host
//! `alo_as_it_was` is an empty crate, and this process says so and ends.

/// What this command does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod putting_back {
    use std::io::{BufRead, Write};
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;
    use std::time::SystemTime;

    use alo_as_it_was::{Places, go_back, what_would_go_back, where_this_person_keeps, words};
    use alo_saying::Loaded;
    use alo_strings::{Filling, Strings};

    /// Every step, in order.
    pub fn main() -> ExitCode {
        let strings = everything_it_can_say();
        let whose = where_this_person_keeps(rustix::process::getuid().as_raw());
        let places = Places::under(&whose);

        // **What would not read is said first, and never instead.** A place that
        // cannot be read is a place this person cannot return to, and it would
        // otherwise simply be missing from the list below with nothing to show that
        // it had been.
        for why in places.stepped_over() {
            say(
                &strings,
                words::SOMETHING_WAS_NOT_READ,
                &Filling::of("why", why.clone()),
            );
        }

        let asked: Vec<String> = std::env::args().skip(1).collect();
        match asked.as_slice() {
            [] => {
                listed(&places, &strings);
                ExitCode::SUCCESS
            }
            [named, folder] => put_back(&places, named, Path::new(folder), &strings),
            _ => {
                listed(&places, &strings);
                ExitCode::FAILURE
            }
        }
    }

    /// Show the places, newest first.
    fn listed(places: &Places, strings: &Strings) {
        if places.kept().is_empty() {
            say(strings, words::NOTHING_IS_KEPT, &Filling::nothing());
            return;
        }
        say(strings, words::WHAT_YOU_CAN_RETURN_TO, &Filling::nothing());
        for place in places.kept() {
            // Not a translated string: a name, a moment and the sentence written
            // down at the time, which is already in the language it was written in.
            println!(
                "  {}  {}  {}",
                place.at().file_name().unwrap_or_default().to_string_lossy(),
                alo_as_it_was::when::as_text(place.taken()),
                place.did()
            );
        }
        say(strings, words::HOW_TO_ASK, &Filling::nothing());
    }

    /// Put one folder back, having asked.
    fn put_back(places: &Places, named: &str, folder: &Path, strings: &Strings) -> ExitCode {
        let Some(place) = places.named(named) else {
            say(
                strings,
                words::NO_SUCH_PLACE,
                &Filling::of("named", named.to_owned()),
            );
            return ExitCode::FAILURE;
        };
        let Some(home) = this_persons_home() else {
            // The same sentence as a folder outside the home, because from where the
            // person sits it is the same thing: this machine cannot say which files
            // are theirs, so it will not put any of them back.
            say(
                strings,
                words::NOT_IN_YOUR_HOME,
                &Filling::of("folder", folder.display().to_string()),
            );
            return ExitCode::FAILURE;
        };

        let goes = match what_would_go_back(
            &place.before(),
            folder,
            &home,
            place.taken(),
            SystemTime::now(),
        ) {
            Ok(goes) => goes,
            Err(why) => {
                eprintln!("{}", why.said(strings).text());
                return ExitCode::FAILURE;
            }
        };

        say(
            strings,
            words::THIS_IS_WHAT_WOULD_HAPPEN,
            &Filling::of("folder", goes.folder().display().to_string())
                .and("taken", goes.taken_as_text())
                .and("aside", goes.what_was_here().display().to_string()),
        );
        if !they_typed_its_name(goes.folder(), strings) {
            say(strings, words::THAT_IS_NOT_ITS_NAME, &Filling::nothing());
            return ExitCode::FAILURE;
        }

        match go_back(&goes) {
            Ok(()) => {
                say(
                    strings,
                    words::IT_IS_AS_IT_WAS,
                    &Filling::of("folder", goes.folder().display().to_string())
                        .and("aside", goes.what_was_here().display().to_string()),
                );
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("{}", why.said(strings).text());
                // **Half-done is not success.** The folder did go back, so the exit
                // code says something happened — but the person was told the final
                // name is wrong, and a script that treated this as a clean go-back
                // would be wrong about where the old contents are.
                if why.the_folder_went_back() {
                    ExitCode::from(2)
                } else {
                    ExitCode::FAILURE
                }
            }
        }
    }

    /// Ask, and answer whether they typed the folder's own name.
    ///
    /// A line that cannot be read is a no. This is asked on the terminal the person
    /// is standing at; a `stdin` that is a closed pipe is not somebody saying yes.
    fn they_typed_its_name(folder: &Path, strings: &Strings) -> bool {
        say(strings, words::TYPE_ITS_NAME, &Filling::nothing());
        let _ = std::io::stdout().flush();
        let mut typed = String::new();
        if std::io::stdin().lock().read_line(&mut typed).is_err() {
            return false;
        }
        let wanted = folder.file_name().unwrap_or_default();
        !typed.trim().is_empty() && typed.trim() == wanted.to_string_lossy()
    }

    /// This person's home, as the session says it is.
    ///
    /// `HOME` and nothing else: it is what the session set, and it is what
    /// `alo-agentd` snapshotted — `WhereBracketsGo` reads the same variable. A
    /// second answer about where somebody's home is would be the one that disagrees
    /// with the copy on the disk.
    fn this_persons_home() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        let home = PathBuf::from(home);
        home.is_absolute().then_some(home)
    }

    /// Say one of this crate's sentences on the output stream.
    fn say(strings: &Strings, word: words::Word, filling: &Filling) {
        println!("{}", strings.say(&word.key(), filling).text());
    }

    /// Everything this command can say, in whatever languages this machine has.
    ///
    /// This crate's own words and nothing else. A command that assembled the whole
    /// machine's vocabulary would load every other crate's strings to print four
    /// sentences, and `alo_saying::Loaded` leaving a line out is survivable — so the
    /// narrow list is the right one here and the broad one belongs to the daemon.
    fn everything_it_can_say() -> Strings {
        let mut vocabulary = alo_strings::Vocabulary::default();
        match words::declare_into(&mut vocabulary) {
            Ok(()) => Loaded::at(vocabulary, alo_saying::the_translations()).into_strings(),
            // **English rather than silence.** A vocabulary this crate's own words
            // will not go into is this repository disagreeing with itself, and the
            // person in front of it still asked to see their places.
            Err(_) => Strings::of(alo_strings::Vocabulary::default()),
        }
    }
}

/// Put a folder back, on the machine this is for.
#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    putting_back::main()
}

/// Say that this is not the machine, and end.
///
/// A place to return to is a read-only snapshot on the filesystem alo OS picks
/// at install, and swapping a folder for one is a single kernel call this
/// operating system has. There is nothing here to put back anywhere else. It
/// ends in failure, because a person who typed this asked for something and did
/// not get it.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "alo-back puts a folder back from a place alo OS kept, which needs the filesystem alo OS \
         installs onto; there is nothing for it to read here"
    );
    std::process::ExitCode::FAILURE
}
