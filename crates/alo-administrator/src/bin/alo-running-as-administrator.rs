//! **Runs one command as administrator, once the person has given their
//! password** (ADR 0150).
//!
//! Started by `alo-running-as-administrator.socket` for one connection, as
//! root, with the connection as its standard input and output. The order it
//! keeps — the agent service asked, the line is one command, the person is an
//! administrator and at a screen, polkit says they gave their password, and
//! only then the run — is `alo_administrator::running`'s, and tested there.

#![cfg(target_os = "linux")]

use std::io::{BufRead, Read, Write};
use std::os::fd::AsFd;
use std::process::ExitCode;

use alo_administrator::answering_as_administrator::{
    Answered, NotAsAdministrator, THE_LONGEST_THE_PERSON_IS_WAITED_FOR,
};
use alo_administrator::on_the_system_bus;
use alo_administrator::running::{TheCaller, TheMachine, answered};
use alo_capability::{ACommand, ItDidNotStart, THE_LONGEST_COMMAND, WhatACommandDid};

/// This machine, asked for real.
struct ThisMachine {
    /// The system bus, when it could be reached.
    bus: Option<zbus::blocking::Connection>,
}

impl TheMachine for ThisMachine {
    fn logins(&self) -> String {
        std::fs::read_to_string(alo_administrator::logins::THE_LOGINS).unwrap_or_default()
    }

    fn groups(&self) -> String {
        std::fs::read_to_string(alo_administrator::logins::THE_GROUPS).unwrap_or_default()
    }

    fn the_session_on_their_screen(&self, person: u32) -> Option<String> {
        on_the_system_bus::the_session_on_their_screen(self.bus.as_ref()?, person)
    }

    fn the_person_said_yes(&self, session: &str, command: &str) -> Result<(), NotAsAdministrator> {
        let bus = self
            .bus
            .as_ref()
            .ok_or(NotAsAdministrator::PolkitDidNotAnswer)?;
        on_the_system_bus::the_person_said_yes(
            bus,
            session,
            command,
            THE_LONGEST_THE_PERSON_IS_WAITED_FOR,
        )
    }

    fn run(&self, command: &ACommand) -> Result<WhatACommandDid, ItDidNotStart> {
        // **As this process, which is root**, bounded and captured exactly as
        // a command run as the person is: `alo_running`'s one road (ADR 0117).
        alo_running::run_as_the_person(command, std::path::Path::new("/"))
    }
}

/// Reads one command, answers one line.
fn main() -> ExitCode {
    let answer = match the_caller() {
        Ok(caller) => {
            let mut line = String::new();
            let read = std::io::stdin()
                .lock()
                .take(u64::try_from(THE_LONGEST_COMMAND + 2).unwrap_or(u64::MAX))
                .read_line(&mut line);
            match read {
                Ok(_) => {
                    let machine = ThisMachine {
                        bus: on_the_system_bus::the_system_bus(THE_LONGEST_THE_PERSON_IS_WAITED_FOR)
                            .map_err(|why| {
                                eprintln!(
                                    "alo-running-as-administrator: the system bus did not answer: \
                                     {why}"
                                );
                            })
                            .ok(),
                    };
                    answered(&machine, &caller, line.trim_end_matches(['\n', '\r']))
                }
                Err(_) => Answered::Refused(NotAsAdministrator::NotACommand),
            }
        }
        Err(why) => Answered::Refused(why),
    };
    // The journal names what happened and never what the command said.
    match &answer {
        Answered::Ran { returned, .. } => {
            eprintln!(
                "alo-running-as-administrator: the person gave their password, and the command \
                 ran as administrator and returned {returned:?}"
            );
        }
        Answered::Refused(why) => {
            eprintln!("alo-running-as-administrator: not run: {why}");
        }
    }
    let mut out = std::io::stdout();
    let _ = out
        .write_all(answer.as_line().as_bytes())
        .and_then(|()| out.flush());
    ExitCode::SUCCESS
}

/// Who is on the other end, as the kernel says: their number, and the control
/// groups it put them in.
fn the_caller() -> Result<TheCaller, NotAsAdministrator> {
    let caller = rustix::net::sockopt::socket_peercred(std::io::stdin().as_fd())
        .map_err(|_| NotAsAdministrator::NotTheAgentService)?;
    let groups = std::fs::read_to_string(format!(
        "/proc/{}/cgroup",
        caller.pid.as_raw_nonzero().get()
    ))
    .map_err(|_| NotAsAdministrator::NotTheAgentService)?;
    Ok(TheCaller {
        uid: caller.uid.as_raw(),
        groups,
    })
}
