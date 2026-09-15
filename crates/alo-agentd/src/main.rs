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
    use std::time::SystemTime;

    use alo_agentd::{
        ByTheKernel, Described, Listening, NotStarted, Place, Served, THE_DESCRIPTION,
        THE_HOSTED_WORKSPACE, THE_IDENTITY, TheNames, TheNamesFile, ThePairingsFile,
        ThePersonsFile, Waking, WhatIsGranted, Wire, session, signalling, starting, unix,
    };
    use alo_capability::Grants;
    use alo_keeping::Writing;
    use alo_nearby::{MachineId, Pairings};
    use alo_remembering::{
        MachineNames, NotRemembered, THE_GRANTS, THE_MACHINE_NAMES, THE_PAIRINGS,
    };

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
    /// # And the session is checked as soon as there is a person to check it
    /// against
    ///
    /// `alo_agentd::session` refuses an environment naming somebody else's
    /// session, and it runs directly after the description because the
    /// description is what says who the person is. Before the vocabulary,
    /// before the record and before the socket: a machine wired into the wrong
    /// session is one to stop on, and stopping is cheapest before anything has
    /// been opened.
    ///
    /// **What is not given back is the boundary**, since ADR 0018. This process
    /// did not load it — `alo-boundaryd` did, at boot — so a service stopping
    /// leaves the machine enforcing, which is the right way round for a service
    /// that runs as the person whose agent is being bounded.
    fn served() -> Result<Served, NotStarted> {
        let us = unix::us()?;
        starting::not_as_root(us)?;

        let described = Described::at(Path::new(THE_DESCRIPTION), us)?;
        session::in_the_persons_session(described.sides().person())?;

        let saying = starting::what_this_machine_says()?;
        for line in saying.damage().lines() {
            eprintln!("alo-agentd: {line}");
        }
        let strings = saying.into_strings();

        let mut writing =
            Writing::opening(described.record()).map_err(|why| NotStarted::NoRecord {
                said: why.said(&strings).text().to_owned(),
            })?;

        let mut grants = whatever_was_granted()?;
        // The same file, and the only way back to it once the service is
        // running: a way to **read** it, handed in beside the list it was read
        // into. `alo_agentd::rereading` has the argument, and the short of it is
        // that nothing below this line can write a byte of it.
        let remembering = ThePersonsFile::at(Path::new(THE_GRANTS));
        // And the other file in that folder, the other way round: read once
        // here, and **written** by the service at the moment a pairing is
        // kept or revoked, through a value that holds this path and can read
        // nothing (`alo_agentd::keeping_pairings`).
        let pairings = whatever_was_paired()?;
        let keeping_pairings = ThePairingsFile::at(Path::new(THE_PAIRINGS));
        // And the third, beside them: what the person here called the machines
        // they paired with, read against the pairings just read so a name whose
        // pairing is gone does not come back, and written by the service when
        // the person names one, clears a name or revokes a pairing.
        let names = TheNames::remembering(
            whatever_was_named(&pairings)?,
            Box::new(TheNamesFile::at(Path::new(THE_MACHINE_NAMES))),
        );

        let (waking, stop) = Waking::made().map_err(|why| NotStarted::NoStop { why })?;
        signalling::on_sigterm(stop)?;

        let mut bounding = ByTheKernel::found().map_err(|why| NotStarted::NoBoundary {
            why: why.to_string(),
        })?;

        // Not `?`, because from here there is a subtree on the machine and a
        // programme in the kernel that belong to this process: every road out
        // goes through the giving back below, including the ones where the
        // port or the socket could not be bound.
        // And the workspace this machine hosts, if root installed one: read
        // once, here, and a file that cannot be believed advertises nothing
        // rather than stopping a machine the person can still be served at —
        // kept on the wire as what the person can act on, so their door can
        // tell them why without the file being read again.
        let hosted = alo_agentd::advertised(Path::new(THE_HOSTED_WORKSPACE), |why| {
            eprintln!("alo-agentd: {why}");
        });

        let served = match who_this_machine_is()
            .and_then(Wire::bound)
            .map(|wire| wire.hosting(hosted))
            .map_err(NotStarted::from)
            .and_then(|wire| {
                Listening::at(
                    Place::for_person(described.sides().person()),
                    described.sides(),
                )
                .map(|listening| (wire, listening))
                .map_err(NotStarted::from)
            }) {
            Ok((wire, listening)) => starting::until_stopped(
                &described,
                &listening,
                &waking,
                &wire,
                &strings,
                &mut WhatIsGranted::of(&mut grants, &remembering),
                pairings,
                Box::new(keeping_pairings),
                names,
                &mut bounding,
                &mut writing,
            ),
            Err(why) => Err(why),
        };
        if let Err(why) = bounding.given_back() {
            eprintln!("alo-agentd: the boundary could not be given back: {why}");
        }
        served
    }

    /// Who this machine is on the network, kept beside the record.
    ///
    /// The one place in this service that names the file the identity is
    /// kept in, as `whatever_was_granted` is for the grants: made the first
    /// time, read afterwards, and refused rather than replaced when it
    /// cannot be read — a new identity would be a new machine to everything
    /// this one had paired with.
    fn who_this_machine_is() -> Result<MachineId, alo_agentd::refusing::NotBound> {
        MachineId::remembered_at(Path::new(THE_IDENTITY)).map_err(|why| {
            alo_agentd::refusing::NotBound::NoWire {
                what: "the machine's identity",
                why: std::io::Error::other(why.to_string()),
            }
        })
    }

    /// What this person had granted before this process existed.
    ///
    /// The one place in this service that names the file a machine keeps its
    /// grants in. What comes back is a value; everything downstream of here has
    /// only that, which is what makes the file unreachable from the socket.
    ///
    /// # A machine with nothing granted, and a machine whose grants will not
    /// read, are two different machines
    ///
    /// No file at all is the ordinary first morning: nobody has picked a folder
    /// yet, the list is empty, and every verb is refused in the grants' own
    /// words. That is not an error and is not reported as one.
    ///
    /// A file that is **there** and is not believable stops the process, and
    /// the reason is the one `alo_agentd::trusting` gives about the machine
    /// description: whoever can write this file says what this machine's agent
    /// may reach. A daemon that shrugged and served under an empty list would
    /// be a machine where deleting somebody's grants and corrupting them look
    /// the same from outside — and the person would find out by discovering
    /// their agent can no longer read their invoices, which is exactly the
    /// silence law 1 is about.
    ///
    /// The clock is read here, once, because expiry is measured from now: what
    /// has already run out is dropped as the list is read rather than carried
    /// into the service and filtered later.
    fn whatever_was_granted() -> Result<Grants, NotStarted> {
        match alo_remembering::remembered(Path::new(THE_GRANTS), SystemTime::now()) {
            Ok(grants) => Ok(grants),
            Err(NotRemembered::NotThere { .. }) => Ok(Grants::default()),
            Err(why) => Err(NotStarted::NoGrants {
                why: why.to_string(),
            }),
        }
    }

    /// What this machine was paired with before this process existed.
    ///
    /// `whatever_was_granted`'s twin for the other file in that folder, with
    /// the same two machines told apart: no file is a machine that has never
    /// paired, which starts; a file that is there and cannot be believed stops
    /// the process, because a daemon that served paired with nothing would
    /// make *somebody tampered with your pairings* look like *you have not
    /// paired yet*. A pairing that ended while the machine was off is dropped
    /// as the list is read.
    fn whatever_was_paired() -> Result<Pairings, NotStarted> {
        match alo_remembering::pairings_remembered(Path::new(THE_PAIRINGS), SystemTime::now()) {
            Ok(pairings) => Ok(pairings),
            Err(NotRemembered::NotThere { .. }) => Ok(Pairings::none()),
            Err(why) => Err(NotStarted::NoPairings {
                why: why.to_string(),
            }),
        }
    }

    /// What the person here called the machines this one is paired with, before
    /// this process existed.
    ///
    /// `whatever_was_paired`'s twin, with the same two machines told apart: no
    /// file is a person who has named nothing, which starts; a file that is
    /// there and cannot be believed stops the process, because whoever could
    /// write it could put one machine's name on another machine's evidence. A
    /// name whose pairing is not in `pairings` is dropped as the list is read.
    fn whatever_was_named(pairings: &Pairings) -> Result<MachineNames, NotStarted> {
        match alo_remembering::machine_names_remembered(
            Path::new(THE_MACHINE_NAMES),
            pairings,
            SystemTime::now(),
        ) {
            Ok(names) => Ok(names),
            Err(NotRemembered::NotThere { .. }) => Ok(MachineNames::none()),
            Err(why) => Err(NotStarted::NoMachineNames {
                why: why.to_string(),
            }),
        }
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
