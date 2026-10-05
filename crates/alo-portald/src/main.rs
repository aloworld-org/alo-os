//! The portal backend, as a machine really runs it: one person's session bus,
//! one name on it, and nothing held between requests.
//!
//! `alo_portald` is the machine this assembles; `alo_portals` is every decision
//! it answers with. This file is deliberately thin, because what a process is —
//! the order, and what an exit code means — is the only part a test cannot
//! reach: it asks the kernel which uid it is, it finds the person's own bus
//! under `/run/user/<uid>`, and it takes
//! `org.freedesktop.portal.Desktop` on it.
//!
//! # It stays, and the name stays with it
//!
//! `Backend::serve_on` answers until the `Served` it returns is dropped, so this
//! is an ordinary long-running service and stopping it gives the name up. That is
//! the honest shape: there is nowhere else for a bus name to be held.
//!
//! # Whose bus, and why no environment variable
//!
//! `alo_secrets::TheBus::of_this_process` takes a uid from the kernel **and
//! nothing else**, which is that crate's way of keeping
//! `DBUS_SESSION_BUS_ADDRESS` out of a decision made on the person's behalf —
//! a property of the signature rather than a promise in a comment. This follows
//! it: the bus is the one belonging to the uid this process runs as, so a unit
//! that started it as the wrong person fails to find a bus rather than answering
//! on somebody else's.
//!
//! # It says one kind of thing, and it says it to a service log
//!
//! Nothing here is read by the person using the machine. A bus that is not
//! there, a name another backend already holds, a grants file in the wrong
//! place: those are read by whoever is standing the machine up, in English.
//! What the person reads is one of `alo_portals`' own words, in their language,
//! rendered by whatever drew the dialog — and this is what the log says
//! underneath it. `alo-sessiond` is the same arrangement for the same reason.
//!
//! # And it runs on Linux
//!
//! On any other host there is no `/run/user/<uid>/bus` and no session for a
//! portal to serve. It ends in failure rather than in success, because a process
//! that exited cleanly would be telling a supervisor that a machine's
//! applications can reach their portals.

/// What this process does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod running {
    use std::process::ExitCode;
    use std::sync::Arc;

    use alo_portald::ThisMachine;
    use alo_portals::{AnswersFile, Backend, Sandboxes, ThePlace};
    use alo_secrets::{TheBus, TheKeyring};

    /// Where a caller's sandbox is read from.
    const THE_PROCESSES: &str = "/proc";

    /// Assemble the backend and answer until this process is stopped.
    pub fn main() -> ExitCode {
        // The person's own bus, from the uid the kernel says this is. Nothing
        // here reads an environment variable to find it.
        let bus = match TheBus::of_this_process() {
            Ok(bus) => bus,
            Err(why) => {
                eprintln!(
                    "alo-portald: there is no session bus this process may use: {why:?}. \
                     It must run as the person whose session it serves, after they have \
                     signed in."
                );
                return ExitCode::FAILURE;
            }
        };

        let keyring = match TheKeyring::opened(&bus) {
            Ok(keyring) => keyring,
            Err(why) => {
                eprintln!("alo-portald: the keyring on that bus could not be opened: {why:?}");
                return ExitCode::FAILURE;
            }
        };

        // **Where the answers go is `alo-portals`' decision, not this crate's.**
        // `ThePlace` reads it off this login's own variables — the state home, or
        // the home directory with the default under it — and `made` creates the
        // folder `0700` and hands back the file. A path spelled here would have
        // been this crate carrying policy, which is the one thing task 1 says it
        // must not do; the first draft of this file did exactly that and named a
        // machine-wide path for a per-login record.
        let place = match ThePlace::for_this_login(
            std::env::var_os(alo_portals::STATE_HOME).as_deref(),
            std::env::var_os(alo_portals::HOME).as_deref(),
        ) {
            Some(place) => place,
            None => {
                eprintln!(
                    "alo-portald: this login has nowhere to keep its answers — neither \
                     ${} nor ${} names an absolute directory. Every answer is written \
                     down before it is given, so there is nothing to run here.",
                    alo_portals::STATE_HOME,
                    alo_portals::HOME
                );
                return ExitCode::FAILURE;
            }
        };
        let answers = match place.made() {
            Ok(file) => file.to_path_buf(),
            Err(why) => {
                eprintln!("alo-portald: this login's answers file could not be made: {why}");
                return ExitCode::FAILURE;
            }
        };
        let record = match AnswersFile::opened(&answers) {
            Ok(record) => record,
            Err(why) => {
                eprintln!(
                    "alo-portald: {} could not be opened: {why}. Every answer is written \
                     down before it is given, so a backend that cannot write does not answer.",
                    answers.display()
                );
                return ExitCode::FAILURE;
            }
        };

        // The person's folder, from what the session says — the same reading
        // `alo-choosing` does at sign-in, and the same refusal when a login has
        // no home directory.
        let theirs = match alo_choosing::the_persons_folder(
            std::env::var_os("XDG_CONFIG_HOME").as_deref(),
            std::env::var_os("HOME").as_deref(),
        ) {
            Ok(folder) => folder.folder().to_path_buf(),
            Err(why) => {
                eprintln!(
                    "alo-portald: this login has no folder to read settings from: {why:?}. \
                     A portal backend answers about one person, and there is no person here."
                );
                return ExitCode::FAILURE;
            }
        };

        // This person's grants, and not the machine's.
        //
        // `docs/decisions/0088-a-machines-grants-belong-to-a-person.md` names
        // this reader zero times and so does `alo_remembering::whose`, which is
        // the thing that made the migration worth measuring rather than
        // assuming: wired into the agent alone, this process would go on reading
        // a path that no longer holds anybody's grants, and **every test in
        // `alo-remembering` would still pass**, because that crate knows neither
        // daemon.
        //
        // The uid is `bus.whose()` — the number the kernel gave this process,
        // already read above and not asked a second way. This process runs as
        // the person whose session it serves, which is the one fact the grants
        // file used to lack, and taking the uid from the bus keeps one source
        // for it rather than two that could disagree.
        //
        // The agent does the move; this only reads. So a portal that starts
        // before any agent has sees no file and answers that nothing is
        // granted, which is true rather than a stub.
        let grants = alo_remembering::the_persons_grants(
            std::path::Path::new(alo_remembering::THE_FOLDER),
            bus.whose(),
        );
        let machine = ThisMachine::reading(grants, &theirs);
        eprintln!(
            "alo-portald: grants from {}, settings from {}, answers in {}, sandboxes from {THE_PROCESSES}",
            machine.grants_file().display(),
            machine.their_folder().display(),
            answers.display()
        );
        eprintln!(
            "alo-portald: OpenURI will refuse every request. Nothing in this repository \
             can say what a person has installed, so what-opens-what is unread rather \
             than empty; alo_portald::this_machine says why."
        );

        let backend = Backend::answering_from(
            Arc::new(machine),
            Arc::new(keyring),
            Sandboxes::under(THE_PROCESSES),
            Arc::new(record),
        );

        // `serve_on` wants an address rather than a path, and the one this bus is
        // at is the only one it may take.
        let address = format!("unix:path={}", bus.at_path().display());
        let served = match backend.serve_on(&address) {
            Ok(served) => served,
            Err(why) => {
                eprintln!("alo-portald: the portals were not served on {address}: {why:?}");
                return ExitCode::FAILURE;
            }
        };
        eprintln!("alo-portald: answering the decided portals on {address}");

        // The name is held while `served` is. There is nothing to poll and
        // nothing to do between requests: the machine is read at each one.
        std::thread::park();
        drop(served);
        ExitCode::SUCCESS
    }
}

/// On a host with no session bus, this is not a thing that can run.
#[cfg(not(target_os = "linux"))]
mod running {
    use std::process::ExitCode;

    pub fn main() -> ExitCode {
        eprintln!(
            "alo-portald: this host has no /run/user/<uid>/bus and no session for a \
             portal backend to serve. It is refused rather than run, because exiting \
             cleanly would say an application's portals are answered."
        );
        ExitCode::FAILURE
    }
}

fn main() -> std::process::ExitCode {
    running::main()
}
