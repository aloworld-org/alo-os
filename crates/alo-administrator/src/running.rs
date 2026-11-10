//! **The runner's order**: who asked, what, whether the person said yes, and
//! only then the run — once.
//!
//! `alo-running-as-administrator` is started as root for one connection. What
//! it does is decided here, with the machine handed in, so every refusal is a
//! test rather than a hope:
//!
//! 1. **The caller is the agent service**, running as a person — the kernel's
//!    answer about the socket, and the control group the kernel put it in.
//! 2. **The line is one command**, by `alo_capability::ACommand`'s own rule.
//! 3. **The person is an administrator** (`wheel`), or polkit would ask
//!    somebody else's password, which nobody here wants asked.
//! 4. **The person has a desktop signed in**, which is where the sheet is.
//! 5. **polkit says the person gave their password** — asked about the
//!    process that leads *their session*, so the sheet appears on the screen
//!    in front of them and not for the service, which is in no session at
//!    all. Never about the session itself, which polkit 126 aborts on
//!    (`crate::the_process_polkit_is_asked_about`).
//! 6. Only then the command runs, as root, bounded as every command alo runs
//!    is bounded.
//!
//! A refusal at any step runs nothing, and none of them falls back to running
//! the command as the person.

use alo_capability::{ACommand, ItDidNotStart, WhatACommandDid};

use crate::answering_as_administrator::{Answered, NotAsAdministrator};
use crate::logins;

/// The control group the agent service runs in
/// (`image/usr/lib/systemd/system/alo-agentd.service`).
pub const THE_AGENT_SERVICES_GROUP: &str = "/system.slice/alo-agentd.service";

/// Who is on the other end of the socket, as the kernel says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheCaller {
    /// Their login's number.
    pub uid: u32,
    /// The control groups `/proc/<pid>/cgroup` lists for them.
    pub groups: String,
}

impl TheCaller {
    /// Whether the kernel put this caller inside the agent service.
    #[must_use]
    pub fn is_the_agent_service(&self) -> bool {
        self.groups.lines().any(|line| {
            line.rsplit_once(':').is_some_and(|(_, path)| {
                path == THE_AGENT_SERVICES_GROUP
                    || path.starts_with(&format!("{THE_AGENT_SERVICES_GROUP}/"))
            })
        })
    }
}

/// What the runner asks of the machine, handed in.
pub trait TheMachine {
    /// The machine's list of logins.
    fn logins(&self) -> String;
    /// The machine's list of groups.
    fn groups(&self) -> String;
    /// The session `person` has signed in at on a screen, if they have one.
    fn the_session_on_their_screen(&self, person: u32) -> Option<String>;
    /// Whether polkit says `person`, in `session`, gave their password for
    /// `command`.
    ///
    /// # Errors
    /// What polkit's answer means ([`crate::what_polkit_said`]), or
    /// [`NotAsAdministrator::PolkitDidNotAnswer`] /
    /// [`NotAsAdministrator::NoAnswerInTime`].
    fn the_person_said_yes(
        &self,
        person: u32,
        session: &str,
        command: &str,
    ) -> Result<(), NotAsAdministrator>;
    /// Run `command` as this process, bounded.
    ///
    /// # Errors
    /// [`ItDidNotStart`] when the shell would not start.
    fn run(&self, command: &ACommand) -> Result<WhatACommandDid, ItDidNotStart>;
}

/// **One request, answered**: decided in the order this file's header gives,
/// and run at most once.
#[must_use]
pub fn answered(machine: &dyn TheMachine, caller: &TheCaller, line: &str) -> Answered {
    match decided(machine, caller, line) {
        Ok(did) => Answered::of(&did),
        Err(why) => Answered::Refused(why),
    }
}

/// [`answered`], as a result.
fn decided(
    machine: &dyn TheMachine,
    caller: &TheCaller,
    line: &str,
) -> Result<WhatACommandDid, NotAsAdministrator> {
    let logins = machine.logins();
    let person = logins::a_person(&logins, caller.uid)
        .filter(|_| caller.is_the_agent_service())
        .ok_or(NotAsAdministrator::NotTheAgentService)?;
    let command = ACommand::of(line).map_err(|_| NotAsAdministrator::NotACommand)?;
    if !logins::is_an_administrator(&machine.groups(), person) {
        return Err(NotAsAdministrator::NotAnAdministrator);
    }
    let session = machine
        .the_session_on_their_screen(caller.uid)
        .ok_or(NotAsAdministrator::NothingCanAsk)?;
    machine.the_person_said_yes(caller.uid, &session, command.text())?;
    machine
        .run(&command)
        .map_err(|_| NotAsAdministrator::DidNotStart)
}

#[cfg(test)]
#[path = "running_tests.rs"]
mod tests;
