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
//! # A tool that is not there did not answer
//!
//! A machine where the program is missing or cannot be started answers every
//! question with [`Failed::DidNotAnswer`], carrying what the machine said, and
//! nothing is guessed about what is installed.

use std::process::{Command, Output, Stdio};

use alo_applications::Application;

use crate::asked;
use crate::heard;
use crate::source::{Configured, SourceName};
use crate::tool::{Failed, Tool};

/// The rented tool, at its place on an alo OS machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheRentedTool {
    /// The program started.
    program: String,
}

impl Default for TheRentedTool {
    fn default() -> Self {
        Self {
            program: asked::THE_TOOL.to_owned(),
        }
    }
}

impl TheRentedTool {
    /// The tool at its place on an alo OS machine.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::default()
    }

    /// Start the tool with these arguments and wait for it.
    fn run(&self, arguments: &[String]) -> Result<Output, Failed> {
        Command::new(&self.program)
            .args(arguments)
            .env_clear()
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|why| Failed::DidNotAnswer {
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
        let missing = TheRentedTool {
            program: "/nonexistent/alo-software-test/no-such-tool".to_owned(),
        };
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
