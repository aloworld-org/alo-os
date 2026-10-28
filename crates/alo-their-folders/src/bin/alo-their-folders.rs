//! Gives the person whose session this is the folders they expect, and stops.
//!
//! ```text
//! alo-their-folders
//! ```
//!
//! Started by the person's **own** systemd manager as
//! `alo-their-folders.service`, enabled `--global` so that every person on the
//! machine has it.
//!
//! # Why the person's manager and not a system unit
//!
//! Because then nothing needs a privilege. The folders have to belong to the
//! person; a program already running as them makes folders that are theirs by
//! the ordinary rules, with no `CAP_CHOWN` and nothing to reason about.
//!
//! **A system unit cannot do this at all.** `alo-compositor` is the obvious
//! caller — it is where an account is made — and it is `User=root` with
//! `CapabilityBoundingSet=` and `AmbientCapabilities=` both empty, which costs
//! it `CAP_DAC_OVERRIDE` as well as `CAP_CHOWN`. A person's home is `0700` and
//! theirs, so that process may not so much as create a directory inside it.
//! [ADR 0119](../../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
//! reached the same wall from the other side and solved it with a capability;
//! this needs none, which is the better answer where it is available.
//!
//! # It takes no arguments and reads one thing
//!
//! `HOME`, which the person's own manager sets and which is the one fact this
//! needs. No account number, because it is already running as the account; no
//! configuration file; nothing from a network.
//!
//! # It runs at every sign-in, not only the first
//!
//! `alo_their_folders::made_for` is unchanged by a second run — a folder that
//! is already there is not a failure and keeps what is in it, which
//! `on_this_disk.rs` tests by writing a file into one and reading it back. So
//! this repairs a machine somebody deleted a folder on, rather than only
//! furnishing a new one, and there is no first-boot condition to get wrong.
//! **That matters here specifically**: `ConditionFirstBoot=` does not fire on
//! this image at all (`docs/quirks.md`), and a unit that depended on it would
//! silently never run.
//!
//! # What it says, and why it says anything
//!
//! One line naming what the person got, because a oneshot that prints nothing
//! is indistinguishable from one that never ran — `alo-giving-a-folder` paid
//! for that lesson and this follows it. Refusals are printed beside it, in the
//! sentence whoever looks after the machine reads, with the operating system's
//! own words after it for a log.
//!
//! # Today these names are English, and that is not this program's doing
//!
//! `alo_their_folders` names each folder through `alo_strings`, so a machine
//! that prefers German makes `Dokumente`. **This program does not prefer one
//! yet.** Since 2026-10-28 the first-start sheet's language is kept and given
//! to the person as their reading language (`alo_shell::the_language_set_up_in`,
//! run by their desktop), but this program loads no translations and asks no
//! settings, so every machine makes the English names until it does — and it
//! runs beside the desktop at sign-in, so on a first sign-in it could not
//! count on the language having been written yet. The one line to change is
//! the `Strings` built below.

use std::path::Path;
use std::process::ExitCode;

use alo_strings::Strings;
use alo_their_folders::{OnThisDisk, made_for, their_folders_words};

/// Makes this person's folders and stops.
fn main() -> ExitCode {
    let Ok(home) = std::env::var("HOME") else {
        eprintln!(
            "alo-their-folders: HOME is not set, so there is nowhere to make \
             anything. Started as alo-their-folders.service in a person's own manager."
        );
        return ExitCode::FAILURE;
    };

    // **This crate's own list and not the machine's whole vocabulary.**
    // `alo-saying` collects every crate's words and depends on this one, so
    // this cannot ask it without the dependency pointing both ways — the same
    // reason `alo-shell` declares its own. Nothing is lost: every sentence
    // this program can say is declared here.
    let strings = match their_folders_words() {
        Ok(vocabulary) => Strings::of(vocabulary),
        Err(why) => {
            eprintln!("alo-their-folders: its own words would not be declared: {why}");
            return ExitCode::FAILURE;
        }
    };

    let made = made_for(Path::new(&home), &strings, &OnThisDisk);

    for (_, at) in made.folders() {
        println!("alo-their-folders: {}", at.display());
    }
    if let Some(record) = made.recorded() {
        println!(
            "alo-their-folders: and {} says which folder is which",
            record.display()
        );
    }
    for refusal in made.refusals() {
        eprintln!(
            "alo-their-folders: {} [{}]",
            refusal.said(&strings).text(),
            refusal.kept_for_a_maintainer()
        );
    }

    // **Nothing made is a failure; some made is not.** A person with two of
    // their three folders is better off than one with none, and a unit that
    // failed every sign-in over a translation nobody has fixed is noise in
    // front of whoever looks after the machine. What went wrong is on the
    // journal either way, which is who the refusals are for.
    if made.folders().is_empty() {
        eprintln!("alo-their-folders: nothing could be made in {home}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
