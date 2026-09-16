//! The rented tool, reached on a machine.
//!
//! [`TheRentedTool`] is [`crate::Tool`] as a process: the program at
//! [`crate::asked::THE_TOOL`], started with the argument list
//! [`crate::asked`] built, its answer read by [`crate::heard`]. That is the
//! whole of it, on purpose — every decision was made before a method here is
//! called, and every answer is read after.
//!
//! # No shell, no environment of the caller's, and the C locale
//!
//! The program is started directly, never through a shell, so no argument is
//! interpreted by anything but the tool. Its environment is cleared and given
//! back only what it needs to find the machine's own installation and to speak
//! in the C locale: what the tool prints is read to decide things
//! ([`crate::heard`]), and a translated message would be a signature failure
//! nothing recognised.
//!
//! # The machine's proxy is on this road, because this road leaves the machine
//!
//! Installing an application, looking for updates to one and fetching one all
//! reach the place applications come from, and on a great many company networks
//! there is no other route out. So the tool is started with the machine's one
//! proxy on it ([`TheRentedTool::taking`]), decided by `alo_proxy::the_way` for
//! the road being taken — and a tool started with nothing decided goes straight
//! out, which is `alo_proxy::Carried::straight` and is what
//! [`TheRentedTool::on_this_machine`] is.
//!
//! The environment is cleared **before** the proxy is put on it, so what the
//! tool honours is this machine's setting and never something a caller's own
//! environment happened to carry.
//!
//! # A tool that is not there did not answer
//!
//! A machine where the program is missing or cannot be started answers every
//! question with [`Failed::DidNotAnswer`], carrying what the machine said, and
//! nothing is guessed about what is installed.

use std::process::{Command, Output, Stdio};

use alo_applications::Application;
use alo_proxy::Carried;

use crate::asked;
use crate::heard;
use crate::source::{Configured, SourceName};
use crate::tool::{Failed, Tool};

/// The rented tool, at its place on an alo OS machine.
///
/// Deliberately not `Clone` and not `PartialEq`: it holds the road out it was
/// given, and that can hold a credential — `alo_proxy::Carried` says why a
/// credential that can be copied or compared is a credential somewhere nobody
/// meant it to be. Its `Debug` is safe because that type's own is.
#[derive(Debug)]
pub struct TheRentedTool {
    /// The program started.
    program: String,
    /// The way out this machine decided for the road the tool is taking.
    taking: Carried,
}

impl Default for TheRentedTool {
    fn default() -> Self {
        Self {
            program: asked::THE_TOOL.to_owned(),
            taking: Carried::straight(),
        }
    }
}

impl TheRentedTool {
    /// The tool at its place on an alo OS machine, going straight out.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::default()
    }

    /// The tool at some other path — a test's.
    ///
    /// `alo_updating::TheBase::at` is the same door for the same reason: what a
    /// program is given can only be shown by starting one, and a machine that
    /// has no rented tool on it is every machine this repository is written on.
    #[must_use]
    pub fn at(program: &str) -> Self {
        Self {
            program: program.to_owned(),
            taking: Carried::straight(),
        }
    }

    /// The program this tool is, by path.
    #[must_use]
    pub fn program(&self) -> &str {
        &self.program
    }

    /// The same tool, taking the way out this machine decided for this road.
    #[must_use]
    pub fn taking(mut self, taking: Carried) -> Self {
        self.taking = taking;
        self
    }

    /// The way out it is taking, for a caller that has to show a person where
    /// this is going — **without the credential**, which is what
    /// `alo_proxy::Carried::shown` answers.
    #[must_use]
    pub fn through(&self) -> &Carried {
        &self.taking
    }

    /// The whole environment the tool is started with, and nothing else is.
    ///
    /// A list rather than a series of calls, for the reason [`crate::asked`]
    /// keeps its argument lists apart from this file: it is then testable as a
    /// list on any machine, including one with no rented tool on it.
    ///
    /// **The proxy is last on purpose.** What a caller's own environment
    /// carried is gone before it is added (`run` in this file clears it), so what
    /// the tool honours is this machine's setting and nothing else.
    #[must_use]
    pub fn environment(&self) -> Vec<(&'static str, String)> {
        let mut given = vec![
            ("LC_ALL", "C".to_owned()),
            ("LANG", "C".to_owned()),
            ("PATH", "/usr/bin:/bin".to_owned()),
        ];
        given.extend(self.taking.variables());
        given
    }

    /// Start the tool with these arguments and wait for it.
    fn run(&self, arguments: &[String]) -> Result<Output, Failed> {
        let mut command = Command::new(&self.program);
        command
            .args(arguments)
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (name, value) in self.environment() {
            command.env(name, value);
        }
        command.output().map_err(|why| Failed::DidNotAnswer {
            said: format!("{} could not be started: {why}", self.program),
        })
    }

    /// A question whose answer is text: what it printed, or why it failed.
    fn answer(&self, arguments: &[String]) -> Result<String, Failed> {
        let output = self.run(arguments)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(heard::failure(&String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// An act: done, or why not.
    fn act(&self, arguments: &[String]) -> Result<(), Failed> {
        self.answer(arguments).map(drop)
    }
}

impl Tool for TheRentedTool {
    fn sources(&self) -> Result<Vec<Configured>, Failed> {
        self.answer(&asked::sources())
            .map(|answered| heard::sources(&answered))
    }

    fn installed(&self) -> Result<Vec<String>, Failed> {
        self.answer(&asked::installed())
            .map(|answered| heard::identifiers(&answered))
    }

    fn open(&self) -> Result<Vec<String>, Failed> {
        self.answer(&asked::open())
            .map(|answered| heard::identifiers(&answered))
    }

    fn install(&self, source: &SourceName, application: &Application) -> Result<(), Failed> {
        self.act(&asked::install(source, application))
    }

    fn updates(&self, source: &SourceName) -> Result<Vec<String>, Failed> {
        self.answer(&asked::updates(source))
            .map(|answered| heard::identifiers(&answered))
    }

    fn update(&self, application: &Application) -> Result<(), Failed> {
        self.act(&asked::update(application))
    }

    fn remove(&self, application: &Application) -> Result<(), Failed> {
        self.act(&asked::remove(application))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A machine without the tool answers nothing, and says so** — every
    /// question fails as *did not answer*, and nothing is guessed.
    #[test]
    fn a_tool_that_is_not_there_did_not_answer() {
        let missing = TheRentedTool::at("/nonexistent/alo-software-test/no-such-tool");
        for answer in [
            missing.sources().map(drop),
            missing.installed().map(drop),
            missing.open().map(drop),
        ] {
            assert!(
                matches!(answer, Err(Failed::DidNotAnswer { ref said }) if said.contains("could not be started")),
                "{answer:?}"
            );
        }
    }

    #[test]
    fn on_a_machine_it_is_the_tool_at_its_place() {
        assert_eq!(TheRentedTool::on_this_machine().program, asked::THE_TOOL);
    }
}
