//! The base's own program, run with arguments this repository decided.
//!
//! ADR 0011: the base is rented and unmodified, and it is spoken to through its
//! own command, `bootc`. **Law 2 is kept by what reaches this file**: the only
//! arguments ever handed to it are `status --format json` and the list
//! [`alo_keeping_up::Staging::arguments`] wrote, and no shell is between them
//! and the program — each argument is one argument, however it is spelt.
//!
//! **Nothing is read from the environment and nothing is inherited on stdin.**
//! The program is named by path; what it answers on stdout is the answer, and
//! what it says on stderr is kept for whoever administers the machine.
//!
//! # The machine's proxy is on this road, because fetching a build leaves it
//!
//! Staging an update pulls a system onto the disk, and on a great many company
//! networks there is no other route out. So the program is started with the
//! machine's one proxy on it ([`TheBase::taking`]), decided by
//! `alo_proxy::the_way` for the road being taken — set explicitly, so what the
//! program honours is this machine's setting rather than whatever the service
//! that started this process happened to be carrying. A base given nothing goes
//! straight out, which is `alo_proxy::Carried::straight` and is what
//! [`TheBase::on_this_machine`] is.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use alo_proxy::Carried;

use crate::refusing::NotAnswered;

/// Where the base's program is on an alo OS machine.
pub const THE_PROGRAM: &str = "/usr/bin/bootc";

/// How much of what the program said on stderr is kept, in bytes: enough to
/// name the reason, and not a whole log.
const WHAT_IS_KEPT_OF_WHAT_IT_SAID: usize = 2048;

/// Something the base can be asked through: the program itself, or a test's
/// stand-in that writes down what it was asked.
///
/// One method, taking arguments this repository already decided. A stand-in
/// changes who answers, never what is asked: every caller in this crate builds
/// its arguments from `alo_keeping_up` before it reaches this.
pub trait Base {
    /// Ask with these arguments and hand back what was printed.
    ///
    /// # Errors
    /// [`NotAnswered::NotStarted`] when nothing could be asked at all, and
    /// [`NotAnswered::SaidNo`] when it answered unsuccessfully, carrying the end
    /// of what it said.
    fn asked(&self, arguments: &[String]) -> Result<Vec<u8>, NotAnswered>;
}

/// The base's program.
///
/// Deliberately not `Clone` and not `PartialEq`: it holds the road out it was
/// given, and that can hold a credential — `alo_proxy::Carried` says why a
/// credential that can be copied or compared is a credential somewhere nobody
/// meant it to be. Its `Debug` is safe because that type's own is.
#[derive(Debug)]
pub struct TheBase {
    /// The program, by path.
    program: PathBuf,
    /// The way out this machine decided for the road the base is taking.
    taking: Carried,
}

impl TheBase {
    /// The base on this machine, at [`THE_PROGRAM`], going straight out.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::at(Path::new(THE_PROGRAM))
    }

    /// The base's program at some other path — a test's.
    #[must_use]
    pub fn at(program: &Path) -> Self {
        Self {
            program: program.to_path_buf(),
            taking: Carried::straight(),
        }
    }

    /// The same base, taking the way out this machine decided for this road.
    #[must_use]
    pub fn taking(mut self, taking: Carried) -> Self {
        self.taking = taking;
        self
    }

    /// The program's path.
    #[must_use]
    pub fn program(&self) -> &Path {
        &self.program
    }

    /// The way out it is taking.
    #[must_use]
    pub fn through(&self) -> &Carried {
        &self.taking
    }

    /// What the program is told about the machine's proxy, and the whole of it.
    ///
    /// A list rather than a series of calls, so it is testable as a list on a
    /// machine with no base on it — which is every machine this repository is
    /// written on.
    #[must_use]
    pub fn environment(&self) -> Vec<(&'static str, String)> {
        self.taking.variables()
    }
}

impl Base for TheBase {
    fn asked(&self, arguments: &[String]) -> Result<Vec<u8>, NotAnswered> {
        let mut command = Command::new(&self.program);
        command.args(arguments).stdin(Stdio::null());
        for (name, value) in self.environment() {
            command.env(name, value);
        }
        let output = command.output().map_err(|why| NotAnswered::NotStarted {
            program: self.program.display().to_string(),
            why: why.to_string(),
        })?;
        if output.status.success() {
            return Ok(output.stdout);
        }
        let said = String::from_utf8_lossy(&output.stderr);
        let from = said
            .char_indices()
            .map(|(at, _)| at)
            .find(|at| said.len() - at <= WHAT_IS_KEPT_OF_WHAT_IT_SAID)
            .unwrap_or(said.len());
        Err(NotAnswered::SaidNo {
            program: self.program.display().to_string(),
            code: output.status.code(),
            said: said.get(from..).unwrap_or_default().trim().to_owned(),
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The base on an alo OS machine is the program the image ships.
    #[test]
    fn the_base_on_this_machine_is_the_program_the_image_ships() {
        assert_eq!(
            TheBase::on_this_machine().program(),
            Path::new("/usr/bin/bootc")
        );
    }

    /// **A program that is not there is refused, naming it.**
    #[test]
    fn a_program_that_is_not_there_is_not_started() {
        let refused = TheBase::at(Path::new("/nowhere/bootc"))
            .asked(&["status".to_owned()])
            .unwrap_err();
        assert!(
            matches!(&refused, NotAnswered::NotStarted { program, .. } if program == "/nowhere/bootc"),
            "{refused:?}"
        );
    }

    /// **A program that exits unsuccessfully is a refusal**, never an empty
    /// answer.
    #[cfg(unix)]
    #[test]
    fn a_program_that_fails_is_a_refusal_and_not_an_empty_answer() {
        let refused = TheBase::at(Path::new("/bin/false"))
            .asked(&["switch".to_owned()])
            .unwrap_err();
        assert!(
            matches!(refused, NotAnswered::SaidNo { code: Some(1), .. }),
            "{refused:?}"
        );
    }

    /// **Each argument is one argument**: nothing is read by a shell, so text
    /// that a shell would split or run arrives exactly as it was written.
    #[cfg(unix)]
    #[test]
    fn each_argument_arrives_as_one_argument_and_nothing_is_run_by_a_shell() {
        let answered = TheBase::at(Path::new("/bin/echo"))
            .asked(&[
                "a; touch /tmp/alo-updating-was-run".to_owned(),
                "$(b)".to_owned(),
            ])
            .unwrap();
        assert_eq!(answered, b"a; touch /tmp/alo-updating-was-run $(b)\n");
        assert!(!Path::new("/tmp/alo-updating-was-run").exists());
    }
}
