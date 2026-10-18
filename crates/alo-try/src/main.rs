//! The process: resolve the command, work out its places, open the record,
//! open the boundary, make the box, run it.
//!
//! The order is the part only a process can be wrong about, and it is the same
//! order `alo-ask` keeps and for the same reason: **what the person typed is
//! read before the machine is touched.** Somebody who pressed return by
//! mistake is owed *that was not a command*, not *the boundary is not there* —
//! which is true, unasked for, and about something they cannot do anything
//! about.
//!
//! # The command's output is not this process's business
//!
//! Nothing here captures or forwards anything. `alo_running` spawns the box's
//! helper with no stdio configuration, so the command inherits **this**
//! process's streams, and this process inherited the person's terminal from
//! their shell. So a script's progress appears as it happens, and a script that
//! asks a question can be answered.
//!
//! *That is the whole reason the box is made here rather than by the daemon*,
//! and ADR 0125's amendment of 2026-10-18 has the measurement.
//!
//! # Two English sentences survive, and they are the ones `alo-ask` keeps too
//!
//! *This session has no home directory* and *this process has no current
//! directory* stay in English. Neither can happen inside a session, both are
//! read by whoever is fixing one, and `alo-ask` makes the same call for its own
//! *the catalogue this release ships could not be read*. Every sentence a
//! person can actually cause is a word from a vocabulary.
//!
//! # Every road out is written down before this process exits
//!
//! `alo_running::WhereCommandsRun` takes somewhere to write and has no
//! constructor that does not, so a run that could not be recorded did not
//! happen. Nothing here can turn that off, which is ADR 0064's *always on*
//! kept structurally rather than remembered.
//!
//! **What is not written down, said rather than discovered:** a refusal that
//! happens *before* the box is attempted — no command of that name, no
//! boundary pinned, nowhere to make a control group — leaves no entry, because
//! the thing that writes entries is the box runner and it was never built. The
//! person is told; the record is not. That is the honest reading of *every
//! execution and every refusal leaves a record*, which is about what a run did,
//! and nothing ran. **It does mean an empty record file appears on a machine
//! that cannot run anything at all**, holding its head line and no entries,
//! because the record is opened before the boundary is asked for.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::SystemTime;

use alo_bounding::{Boundary, Pinned, THE_ROOT};
use alo_capability::{Grantee, HowCodeRuns};
use alo_running::{AskedToRun, WhereCommandsRun, what_a_try_may_reach};
use alo_strings::{Strings, Vocabulary};
use alo_try::{
    found_on_the_path, the_cgroup_this_process_is_in, the_record_a_person_keeps, where_the_box_goes,
};

/// Who the run is written down as having been asked by.
///
/// **Named for itself rather than for the person**, which is `alo-ask`'s
/// finding about the same type one road along: `alo_capability::Grantee` is an
/// agent or an application, and a command a person typed is neither. Writing a
/// login name into a type whose two kinds are both software would be worse.
const WHO_IS_RUNNING_IT: &str = "alo-try";

/// What every refusal says to a person, in their own language.
fn what_it_can_say() -> Result<Vocabulary, alo_capability::WordsError> {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// What a person reads for a fault in the machine, and what an administrator
/// reads after it.
///
/// **Two lines, and the second is never worked into the first.** The person's
/// sentence is `capability.running.no-box` — *nothing ran, because this machine
/// could not build the box it runs commands in* — which is true of every one of
/// these and is all they can act on. The English behind it names the file or the
/// kernel, and its reader is whoever administers the machine.
///
/// `alo_turn::words::COULD_NOT_LOOK_AT_THE_CATALOGUE` keeps the same two apart
/// for the same reason, and says so in as many words.
fn the_machine_could_not(strings: &Strings, detail: &str) {
    eprintln!(
        "{}",
        strings.say(
            &alo_capability::words::NOTHING_RAN_NO_BOX.key(),
            &alo_strings::Filling::nothing(),
        )
    );
    eprintln!("alo-try: {detail}");
}

/// The exit code a command's own status becomes.
///
/// A program that ran and returned 1 **ran**, and this process says so with its
/// own code rather than treating it as a failure of the box — the distinction
/// `alo_running::WhatHappened::the_command_was_happy` draws.
fn as_this_process_ends(ended: std::process::ExitStatus) -> ExitCode {
    match ended.code() {
        Some(code) => u8::try_from(code).map_or(ExitCode::FAILURE, ExitCode::from),
        // Killed by a signal: no code of its own, and not a success.
        None => ExitCode::FAILURE,
    }
}

fn main() -> ExitCode {
    let vocabulary = match what_it_can_say() {
        Ok(vocabulary) => vocabulary,
        // English, and the only English a person could see here: the sentence
        // that would have said this in their language is the one that could not
        // be built. Whoever reads it is fixing a word list.
        Err(why) => {
            eprintln!("alo-try: {why}");
            return ExitCode::FAILURE;
        }
    };
    let strings = Strings::of(vocabulary);

    // **What the person typed, before the machine is touched.**
    let mut asked = std::env::args_os().skip(1);
    let Some(named) = asked.next() else {
        eprintln!(
            "{}",
            strings.say(
                &alo_capability::words::NOTHING_RAN_NOT_A_COMMAND.key(),
                &alo_strings::Filling::nothing(),
            )
        );
        return ExitCode::FAILURE;
    };
    let arguments: Vec<std::ffi::OsString> = asked.collect();

    let Ok(standing_in) = std::env::current_dir() else {
        eprintln!(
            "alo-try: this process has no current directory, so a relative command cannot be resolved"
        );
        return ExitCode::FAILURE;
    };
    let Some(program) = found_on_the_path(
        &named.to_string_lossy(),
        &standing_in,
        std::env::var("PATH").ok().as_deref(),
    ) else {
        eprintln!(
            "{}",
            strings.say(
                &alo_capability::words::NOTHING_RAN_NOT_A_COMMAND.key(),
                &alo_strings::Filling::nothing(),
            )
        );
        return ExitCode::FAILURE;
    };

    let places = match what_a_try_may_reach(&program) {
        Ok(places) => places,
        Err(why) => {
            the_machine_could_not(&strings, &why.to_string());
            return ExitCode::FAILURE;
        }
    };

    // The record, before the box: a run that cannot be written down does not
    // happen, so there is nothing to undo if this is where it stops.
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        eprintln!(
            "alo-try: this session has no home directory, so there is nowhere to write down what ran"
        );
        return ExitCode::FAILURE;
    };
    let keeping = the_record_a_person_keeps(&home);
    if let Some(folder) = keeping.parent()
        && let Err(why) = std::fs::create_dir_all(folder)
    {
        the_machine_could_not(
            &strings,
            &format!("{} could not be made: {why}", folder.display()),
        );
        return ExitCode::FAILURE;
    }
    let mut record = match alo_keeping::Writing::opening(&keeping) {
        Ok(record) => record,
        Err(why) => {
            eprintln!("{}", why.said(&strings));
            return ExitCode::FAILURE;
        }
    };

    // The boundary `alo-boundaryd` left pinned, opened by path. Nothing here
    // imposes one: that needs root and is the boundary daemon's whole job.
    let pinned = Pinned::beneath(std::path::Path::new(THE_ROOT));
    let mut boundary = match Boundary::opened(&pinned) {
        Ok(boundary) => boundary,
        Err(why) => {
            the_machine_could_not(
                &strings,
                &format!("the kernel boundary is not there: {why}"),
            );
            return ExitCode::FAILURE;
        }
    };

    // Somewhere to make a domain control group, which is the subtree systemd
    // delegates to this person.
    // **A child of this process's own cgroup**, because cgroup v2 asks whether
    // the *common ancestor* may be written and not whether the destination
    // may. `where_the_box_goes`' documentation has the measurement that found
    // this, and the first version of it was wrong.
    let said = match std::fs::read_to_string("/proc/self/cgroup") {
        Ok(said) => said,
        Err(why) => {
            the_machine_could_not(
                &strings,
                &format!("/proc/self/cgroup could not be read: {why}"),
            );
            return ExitCode::FAILURE;
        }
    };
    let Some(mine) = the_cgroup_this_process_is_in(&said) else {
        the_machine_could_not(
            &strings,
            "this process is in no control group of its own, so there is nowhere a box may be \
             made: the unit this shell descends from needs `Delegate=`",
        );
        return ExitCode::FAILURE;
    };
    let boxes = where_the_box_goes(&mine);
    if let Err(why) = std::fs::create_dir_all(&boxes) {
        the_machine_could_not(
            &strings,
            &format!(
                "{} could not be made, so there is nowhere to put the box: {why}",
                boxes.display()
            ),
        );
        return ExitCode::FAILURE;
    }

    let wants = AskedToRun {
        // **The word is the choice.** A person who typed `try` asked for a
        // sealed box for this one command, whatever their setting says about
        // how code runs by default — which is what the word is for.
        chose: HowCodeRuns::InASealedBox,
        program,
        arguments,
        may_reach: places,
    };

    let mut where_they_run = WhereCommandsRun::new(&mut boundary, &boxes, &mut record);
    let who_asked = Grantee::named(WHO_IS_RUNNING_IT);
    match where_they_run.ran(
        &who_asked,
        WHO_IS_RUNNING_IT,
        &wants,
        &strings,
        SystemTime::now(),
    ) {
        Ok(what_happened) => as_this_process_ends(what_happened.ended),
        Err(not_run) => {
            // What a person reads, and then what an administrator reads. The
            // second is `NotRun`'s own English and is deliberately not worked
            // into the first — `alo-turn`'s `COULD_NOT_LOOK_AT_THE_CATALOGUE`
            // keeps the same two apart for the same reason.
            eprintln!(
                "{}",
                strings.say(&not_run.the_word().key(), &alo_strings::Filling::nothing())
            );
            eprintln!("alo-try: {not_run}");
            ExitCode::FAILURE
        }
    }
}
