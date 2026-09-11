//! The opener, as a machine really runs it: root, in the greeter's group,
//! holding no capability, with one door.
//!
//! `alo_sessiond` is every decision this component is made of; this is the
//! process that makes them in order. It is deliberately thin, because what a
//! process is — the order, and what an exit code means — is the only thing here
//! a test cannot reach: it asks the kernel which group it is in, and it binds
//! `/run/alo-sessiond/sign-in.sock`, which is the real machine's door rather
//! than a path a test may write to.
//!
//! # It stays, and the session stays with it
//!
//! Unlike `alo-boundaryd`, which loads a boundary the kernel then holds, this
//! process **is** what holds the session: `logind` hands back a descriptor and
//! the session lasts while somebody keeps it. So the unit is an ordinary
//! long-running service, and stopping it signs the machine out. That is the
//! honest shape rather than a limitation — there is nowhere else for a session
//! to be held, and a `oneshot` that exited would take the person's session with
//! it a fraction of a second after opening it.
//!
//! # It says one kind of thing, and it says it to a service log
//!
//! Nothing this file writes is read by the person using the machine. A `Group=`
//! line naming root, a bus that is not there, a `logind` that said no: those
//! are read by whoever is standing the machine up, in English. What the person
//! reads is one of `alo_sessiond::EVERY_WORD`, in their own language, rendered
//! by the surface that drew the screen — and this is what the log says
//! underneath it.
//!
//! # And it runs on Linux
//!
//! On any other host there is no `logind` and no session to open. It ends in
//! failure rather than in success, because a process that exited cleanly would
//! be telling a supervisor that a machine can be signed in to.

/// What this process does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod running {
    use std::path::Path;
    use std::process::ExitCode;

    use alo_sessiond::listening::Listening;
    use alo_sessiond::{
        Answered, Door, Opening, THE_DOOR, TheMachinesAccounts, TheMachinesLogind, our_group,
    };

    /// Open the door and answer whoever knocks, until this service is stopped.
    ///
    /// `FAILURE` is a machine that could not be signed in to: a `Group=` line
    /// that leaves this process in root's group, or a door that would not bind.
    /// There is no third code. A knock that is refused is **not** a failure of
    /// this process — it is an answer, and it goes back down the connection it
    /// came in on.
    pub fn main() -> ExitCode {
        let group = our_group();
        let door = match Door::this_process_opens(group) {
            Ok(door) => door,
            Err(why) => {
                eprintln!("alo-sessiond: this machine cannot be signed in to: {why}");
                return ExitCode::FAILURE;
            }
        };

        let listening = match Listening::at(Path::new(THE_DOOR), door.group()) {
            Ok(listening) => listening,
            Err(why) => {
                eprintln!(
                    "alo-sessiond: this machine cannot be signed in to: the door at {THE_DOOR} \
                     would not open: {why}"
                );
                return ExitCode::FAILURE;
            }
        };

        eprintln!(
            "alo-sessiond: the door at {THE_DOOR} is open to group {group} and to nobody else; \
             this process holds no capability, and what it can do is open one session"
        );

        let mut opening = Opening::at(door, TheMachinesAccounts, TheMachinesLogind::default());
        loop {
            match listening.answer_one(&mut opening) {
                Ok(answered) => said(&answered),
                // A caller that went away mid-sentence, or a connection the
                // machine would not accept. Neither is this service failing:
                // the next knock is the one that matters, and a door that
                // stopped listening because somebody hung up would be a machine
                // one dropped connection away from being unusable.
                Err(why) => eprintln!("alo-sessiond: a caller was not answered: {why}"),
            }
        }
    }

    /// What happened, for the service log.
    ///
    /// The key rather than the sentence: the sentence belongs to whoever has a
    /// person and a language in front of them, and a log that printed English
    /// here would be a second rendering of somebody else's string.
    fn said(answered: &Answered) {
        match answered {
            Answered::Opened => {
                eprintln!("alo-sessiond: a session is open, and it lasts as long as this process");
            }
            Answered::Refused(key) => {
                eprintln!("alo-sessiond: a knock was refused, and the surface was told {key}");
            }
        }
    }
}

/// The opener, on the machine it is for.
#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    running::main()
}

/// Anywhere else, and it says so rather than pretending.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "alo-sessiond opens a systemd-logind session and there is none on this host: alo OS is \
         Linux (ADR 0011), and a process that exited cleanly here would be telling a supervisor \
         that this machine can be signed in to"
    );
    std::process::ExitCode::FAILURE
}
