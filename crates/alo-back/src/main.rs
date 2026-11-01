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
//!
//! # It is written down
//!
//! Since 2026-11-01 the putting back is done by the service that writes this
//! machine's record (`put-back` on the person's door,
//! `docs/contracts/daemon-protocol.md`), which runs as this same person, does
//! exactly what this command used to do itself, and writes a `put-back` entry
//! — the owner's answer that putting back *is written in this list too*, for
//! both of a person's roads. **A service that cannot be reached changes
//! nothing**, and this says so: putting back is always written down. The one
//! difference a script can see: the rare go-back whose folder went back but
//! whose old contents kept their working name now ends in zero, with the
//! sentence saying where they are, because the service answers that it went
//! back.

//! # And it runs on Linux
//!
//! The filesystem the whole feature rests on is this operating system's, and so
//! is the one call in `alo_as_it_was` that has to be atomic. On any other host
//! `alo_as_it_was` is an empty crate, and this process says so and ends.

/// What this command does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod putting_back {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;
    use std::time::{Duration, SystemTime};

    use alo_as_it_was::{Places, what_would_go_back, where_this_person_keeps, words};
    use alo_protocol::{FromAPerson, ToAPerson, where_the_door_is};

    /// How long the service is waited for: a folder is copied before it goes
    /// back, so this is the copy's time and not a network's.
    const PATIENCE: Duration = Duration::from_secs(30 * 60);
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

        // **Through the service that writes the record**, and not here: the
        // owner's answer of 2026-11-01 is that putting back is written in the
        // list of what happened, and that list has one writer. The service
        // runs as this same person and does exactly what this command did —
        // `alo_as_it_was::go_back_together` — and writes it down. A service
        // that cannot be reached changes nothing: putting back is always
        // written down.
        match the_service_puts_back(named, goes.folder(), strings) {
            Ok(ToAPerson::PutBack { said }) => {
                for wording in said {
                    println!("{}", wording.text());
                }
                ExitCode::SUCCESS
            }
            Ok(ToAPerson::Refused(wording)) => {
                eprintln!("{}", wording.text());
                ExitCode::FAILURE
            }
            // An answer that is not about putting back says nothing a person
            // could act on, so the sentence stands without one.
            Ok(_) => {
                say_why_not(strings, "");
                ExitCode::FAILURE
            }
            Err(why) => {
                say_why_not(strings, &why);
                ExitCode::FAILURE
            }
        }
    }

    /// Ask the service to put `folder` back from the place `named`, and its
    /// answer.
    ///
    /// **No moment and no command on the wire**, as the protocol's person
    /// door has it: the place by its name, and the folder.
    fn the_service_puts_back(
        named: &str,
        folder: &Path,
        strings: &Strings,
    ) -> Result<ToAPerson, String> {
        let at = where_the_door_is::the_door_of(rustix::process::getuid().as_raw());
        let stream = UnixStream::connect(&at).map_err(|why| why.to_string())?;
        // **Patient**, because a folder is copied out of the place before it
        // goes back, and a large one takes as long as its copy takes.
        stream
            .set_read_timeout(Some(PATIENCE))
            .map_err(|why| why.to_string())?;
        let line = FromAPerson::PutBack {
            place: named.to_owned(),
            folders: vec![folder.display().to_string()],
        }
        .written()
        .map_err(|why| why.to_string())?;
        (&stream)
            .write_all(line.as_bytes())
            .and_then(|()| (&stream).write_all(b"\n"))
            .map_err(|why| why.to_string())?;
        let mut answer = String::new();
        BufReader::new(&stream)
            .read_line(&mut answer)
            .map_err(|why| why.to_string())?;
        ToAPerson::read(answer.trim_end()).map_err(|why| why.said(strings).into_text())
    }

    /// Say that nothing was put back because the record could not be told.
    fn say_why_not(strings: &Strings, why: &str) {
        eprintln!(
            "{}",
            strings
                .say(
                    &words::NOT_WITHOUT_THE_RECORD.key(),
                    &Filling::of("why", why.to_owned())
                )
                .text()
        );
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
