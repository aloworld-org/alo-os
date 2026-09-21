//! `alo-letting-go.service`: the privileged half of letting go of an expired
//! undo.
//!
//! Every decision is [`alo_letting_go::sweep`]; this is the process that makes
//! them in order on a real machine, and it is deliberately thin because the
//! only thing here a test cannot reach is the machine's own paths and its
//! clock.
//!
//! It says one kind of thing, to the journal, in English for whoever
//! administers the machine. What a *person* reads is the record, in their own
//! language, from the entry this wrote.
//!
//! **It takes no arguments and reads no environment.** There is nothing on its
//! command line, so there is nothing there a request could reach — which is law
//! 2 at the one place in alo OS that removes a person's own history.

/// Let go of what the window no longer reaches, on the machine alo OS is for.
#[cfg(unix)]
fn main() -> std::process::ExitCode {
    use std::path::Path;
    use std::time::SystemTime;

    use alo_letting_go::{
        OnThisMachine, Swept, THE_FOLDER, THE_RECORD, TheMachinesRecord, WithTheBase, sweep,
    };

    let mut record = match alo_keeping::Writing::opening(Path::new(THE_RECORD)) {
        Ok(writing) => TheMachinesRecord::of(writing),
        Err(why) => {
            // Nothing is removed without somewhere to say what was removed.
            // ADR 0045's second term is that a person is told which turns lost
            // their undo, and a run that could not tell them would be the
            // machine keeping the letter of the first term and dropping the
            // second.
            eprintln!(
                "alo-letting-go: nothing was let go, because {THE_RECORD} could not be opened: \
                 {why:?}"
            );
            return std::process::ExitCode::FAILURE;
        }
    };

    match sweep(
        Path::new(THE_FOLDER),
        &OnThisMachine,
        &WithTheBase,
        &mut record,
        SystemTime::now(),
    ) {
        Swept::NotOnThisMachine => {
            eprintln!(
                "alo-letting-go: this machine keeps nothing an undo could put back, so there was \
                 nothing to let go of"
            );
            std::process::ExitCode::SUCCESS
        }
        Swept::Done(did) => {
            for said in did.said() {
                eprintln!("alo-letting-go: {said}");
            }
            eprintln!(
                "alo-letting-go: {} outside the window, {} for room",
                did.outside_the_window(),
                did.for_room()
            );
            // Anything it could not do is already on the journal above. The
            // unit is not a failure for having stepped over a folder it must
            // not touch: a timer that reported failure every hour for a file
            // somebody typed wrong is one its administrator stops reading.
            std::process::ExitCode::SUCCESS
        }
    }
}

/// Anywhere else, and it says so rather than pretending.
#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("alo-letting-go is a systemd unit's program and alo OS is Linux (ADR 0011)");
    std::process::ExitCode::FAILURE
}
