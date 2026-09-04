//! The agent service, as a machine really runs it.
//!
//! `alo_agentd` is every decision this service is made of; this is the process
//! that makes them in order. It is deliberately thin, because what a process is
//! — the order, and what an exit code means — is the only thing here that a test
//! cannot reach: it reads `/etc/alo/agentd.toml`, opens the person's door in
//! `/run/alo`, and opens a record on a real disk.
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
        ByTheKernel, Described, Listening, NotStarted, Place, Served, THE_DESCRIPTION, Waking,
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
    /// The three orderings worth reading twice are all in that argument. The
    /// vocabulary is loaded **before** the record, so a record that will not
    /// open is refused in the words `alo-keeping` already wrote rather than in
    /// a second sentence about a disk. The boundary is found **before the
    /// socket and after the record**, because a service that cannot bound a
    /// turn has nothing to offer anybody (ADR 0015) and a machine with no
    /// boundary is one that will not serve rather than one that cannot say why.
    /// The socket is bound **last**, so nothing on this machine can knock on a
    /// service that is still deciding whether it can run.
    ///
    /// # The subtree is given back here, and this is the only place it can be
    ///
    /// Finding the boundary moves this process into a control group of its own,
    /// and `alo_bounding::Turns::given_back` is deliberately not a `Drop`:
    /// moving a process between control groups can fail, and a machine filling
    /// with the remains of daemons with nothing saying so is what a swallowed
    /// failure looks like a month later. So it is given back on both roads out —
    /// the service that stopped and the service that failed — and what it said
    /// is a line in the log rather than a different exit code, because a service
    /// that ran and then could not tidy up did run.
    ///
    /// **What is not given back is the boundary**, since ADR 0018. This process
    /// did not load it — `alo-boundaryd` did, at boot — so a service stopping
    /// leaves the machine enforcing, which is the right way round for a service
    /// that runs as the person whose agent is being bounded.
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

        let mut bounding = ByTheKernel::found().map_err(|why| NotStarted::NoBoundary {
            why: why.to_string(),
        })?;

        // Not `?`, because from here there is a subtree on the machine and a
        // programme in the kernel that belong to this process: every road out
        // goes through the giving back below, including the one where the
        // socket could not be bound.
        let served = match Listening::at(
            Place::for_person(described.sides().person()),
            described.sides(),
        ) {
            Ok(listening) => starting::until_stopped(
                &described,
                &listening,
                &waking,
                &strings,
                &mut bounding,
                &mut writing,
            ),
            Err(why) => Err(NotStarted::from(why)),
        };
        if let Err(why) = bounding.given_back() {
            eprintln!("alo-agentd: the boundary could not be given back: {why}");
        }
        served
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
