//! The agent service, as a machine really runs it.
//!
//! `alo_agentd` is every decision this service is made of; this is the process
//! that makes them in order. It is deliberately thin, because what a process is
//! — the order, and what an exit code means — is the only thing here that a test
//! cannot reach: it reads `/etc/alo/agentd.toml`, takes its directory from the
//! session it was started inside, and opens a record on a real disk.
//! `alo_agentd::starting` is the same sequence with each step a value, and is
//! where the arguments and the tests live.
//!
//! # It says one kind of thing, and it says it to a service log
//!
//! Nothing this file writes is read by the person using the machine. A
//! description that will not parse, a socket another daemon holds, a disk with
//! no room on it: those are read by whoever is standing the machine up, in
//! English, which is `alo_agentd::refusing`'s argument. What the person reads
//! travels back over the socket in their own language and is made by the crates
//! that refuse.
//!
//! # And it runs on Linux
//!
//! The service is a Unix socket and the credentials a kernel keeps for one. On
//! any other host `alo_agentd` is an empty crate, and this process says so and
//! ends rather than starting something that could not serve anybody.

/// What this process does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod running {
    use std::path::Path;
    use std::process::ExitCode;

    use alo_agentd::{
        Described, Listening, NotStarted, Place, Served, THE_DESCRIPTION, Waking, session,
        signalling, starting, unix,
    };
    use alo_keeping::Writing;

    /// Serve until somebody asks the service to stop, and say what it did.
    ///
    /// `SUCCESS` when a service ran and was stopped, `FAILURE` when one did not
    /// start or stopped for a reason that is the machine's. There is no third
    /// code and there will not be one: a supervisor restarts a service or it
    /// does not, and saying *how* the machine was wrong in a number would be
    /// saying it in the one place nobody can put a sentence.
    pub fn main() -> ExitCode {
        match served() {
            Ok(served) => {
                eprintln!(
                    "alo-agentd stopped: {} turns, {} messages, {} strangers turned away, {} entries removed, {} shortenings refused",
                    served.turns(),
                    served.messages(),
                    served.strangers_turned_away(),
                    served.entries_removed(),
                    served.shortenings_refused(),
                );
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("alo-agentd did not run: {why}");
                ExitCode::FAILURE
            }
        }
    }

    /// Everything, in the order `alo_agentd::starting` argues for.
    ///
    /// The two orderings worth reading twice are both in that argument. The
    /// vocabulary is loaded **before** the record, so a record that will not
    /// open is refused in the words `alo-keeping` already wrote rather than in
    /// a second sentence about a disk. The socket is bound **last**, so nothing
    /// on this machine can knock on a service that is still deciding whether it
    /// can run.
    fn served() -> Result<Served, NotStarted> {
        let us = unix::us()?;
        starting::not_as_root(us)?;

        let described = Described::at(Path::new(THE_DESCRIPTION), us)?;

        let saying = starting::what_this_machine_says()?;
        for line in saying.damage().lines() {
            eprintln!("alo-agentd: {line}");
        }
        let strings = saying.into_strings();

        let mut writing =
            Writing::opening(described.record()).map_err(|why| NotStarted::NoRecord {
                said: why.said(&strings).text().to_owned(),
            })?;

        let (waking, stop) = Waking::made().map_err(|why| NotStarted::NoStop { why })?;
        signalling::on_sigterm(stop)?;

        let listening = Listening::at(
            Place::under(&session::from_the_environment()?),
            described.sides(),
        )?;

        starting::until_stopped(&described, &listening, &waking, &strings, &mut writing)
    }
}

/// The service, on the machine it is for.
#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    running::main()
}

/// Say that this is not the machine, and end.
///
/// alo OS boots Linux and this service is a Unix socket's peer credentials, so
/// on any other host there is nothing here to start. It ends in failure rather
/// than in success, because a process that exited cleanly would be telling a
/// supervisor that a service had run.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "alo-agentd is the agent service of alo OS, which boots Linux; there is nothing for it to \
         listen on here"
    );
    std::process::ExitCode::FAILURE
}
