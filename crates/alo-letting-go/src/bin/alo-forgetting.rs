//! `alo-forgetting.service`: the privileged half of the one act a person asks
//! for.
//!
//! Every decision is [`alo_letting_go::forget`]; this is the process that makes
//! them in order on a real machine, and it is deliberately thin because the only
//! thing here a test cannot reach is the machine's own paths and its clock. It
//! is `alo-letting-go`'s sibling and is as nearly the same program as two jobs
//! allow — the expiry obeys a window on a timer, and this obeys a person.
//!
//! It says one kind of thing, to the journal, in English for whoever
//! administers the machine. What a *person* reads is the record, in their own
//! language, from the entry this wrote.
//!
//! **It takes no arguments and reads no environment.** What it acts on is what a
//! person's own session left in `/run/alo/asked-to-forget`, and that carries no
//! person, no folder and no path — so there is nothing on this command line and
//! nothing in that folder a request could steer. That is law 2 at the one place
//! in alo OS where a person removes their own history in one act.

/// Forget everything this machine was keeping for whoever asked.
#[cfg(unix)]
fn main() -> std::process::ExitCode {
    use std::path::Path;
    use std::time::SystemTime;

    use alo_letting_go::{
        Forgotten, OnThisMachine, THE_ASKING, THE_FOLDER, THE_FORGETTING_RECORD, TheMachinesRecord,
        WithTheBase, forget,
    };

    let mut record = match alo_keeping::Writing::opening(Path::new(THE_FORGETTING_RECORD)) {
        Ok(writing) => TheMachinesRecord::of(writing),
        Err(why) => {
            // Nothing is removed without somewhere to say what was removed.
            // ADR 0045's second term is that a person is told which turns lost
            // their undo, and an act that could not tell them would keep the
            // letter of point 5 and drop the half that makes it checkable.
            //
            // **The askings are left where they are.** This is the one failure
            // that happens before anything has been looked at, so nothing has
            // been spent: whoever mends the record starts this again and the
            // person's approval is still worth acting on.
            eprintln!(
                "alo-forgetting: nothing was forgotten, because {THE_FORGETTING_RECORD} could not \
                 be opened: {why:?}"
            );
            return std::process::ExitCode::FAILURE;
        }
    };

    match forget(
        Path::new(THE_FOLDER),
        Path::new(THE_ASKING),
        &OnThisMachine,
        &OnThisMachine,
        &WithTheBase,
        &mut record,
        SystemTime::now(),
    ) {
        Forgotten::NotOnThisMachine => {
            eprintln!(
                "alo-forgetting: this machine keeps nothing an undo could put back, so there was \
                 nothing to forget"
            );
            std::process::ExitCode::SUCCESS
        }
        Forgotten::Done(did) => {
            for said in did.said() {
                eprintln!("alo-forgetting: {said}");
            }
            eprintln!(
                "alo-forgetting: {} kept turns forgotten, for {} people who asked",
                did.turns(),
                did.people()
            );
            // Anything it could not do is already on the journal above. The
            // unit is not a failure for having stepped over a folder it must
            // not touch, or for having been started by a name somebody else
            // left: a unit that reported failure for those is one its
            // administrator stops reading.
            std::process::ExitCode::SUCCESS
        }
    }
}

/// Anywhere else, and it says so rather than pretending.
#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("alo-forgetting is a systemd unit's program and alo OS is Linux (ADR 0011)");
    std::process::ExitCode::FAILURE
}
