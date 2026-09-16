//! The rented evaluator, reached as a program that holds nothing.
//!
//! `automatic.rs` states the rule this file is the whole of: **an automatic
//! configuration never runs in a process that holds a grant.** Here is how that
//! is true rather than intended.
//!
//! | | |
//! |---|---|
//! | It is a **separate process** | started with [`std::process::Command`], so nothing it does happens where a decision is made |
//! | It is started **directly** | no shell, so no argument is read by anything but the program |
//! | Its environment is **cleared** | [`nothing_of_this_machines`] is the whole of what it is given, and there is no line that adds to it |
//! | Its arguments are **checked values** | a configuration address, a road's address and a road's host, each of which was refused if it was not one line |
//! | It is given **no proxy of its own** | the road to the configuration is the network's, not this machine's setting, and a setting that pointed its own evaluator through itself would be a machine that cannot start |
//!
//! It holds no grant because nothing ever granted it one: a grant on this
//! machine is over an agent and an application (ADR 0040), and this is neither.
//! It cannot be handed one either, because everything it is given is on the
//! list above.
//!
//! # Which program is behind the door is the image's
//!
//! [`THE_EVALUATOR`] is the path an alo OS machine has one at, exactly as
//! `alo_software::asked::THE_TOOL` is the path the rented tool is at. **A
//! machine with nothing there refuses every road under an automatic
//! configuration** ([`crate::NotEvaluated::NothingEvaluatesIt`]) rather than
//! going straight out, which is the behaviour the test at the bottom of this
//! file holds and the one that matters on a network where the proxy is a rule.
//!
//! # The C locale, for the same reason the rented tool is given it
//!
//! What the program prints is read to decide something, so it is asked to print
//! in the one language this file can read. A translated answer would be a road
//! nothing recognised, and this crate refuses what it does not recognise.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::automatic::{ConfigurationAddress, MOST_OF_AN_ANSWER, NotEvaluated, TheEvaluator};

/// Where an alo OS machine has something that evaluates an automatic
/// configuration.
pub const THE_EVALUATOR: &str = "/usr/libexec/alo/proxy-configuration";

/// The environment the evaluator is given, and the whole of it.
///
/// Nothing of this machine's: no session, no bus address, no home, no
/// credential, no proxy. `PATH` is here because a program that starts other
/// programs needs one and a program that does not is unharmed by it; the locale
/// is here because the answer is read.
#[must_use]
pub const fn nothing_of_this_machines() -> [(&'static str, &'static str); 3] {
    [("LC_ALL", "C"), ("LANG", "C"), ("PATH", "/usr/bin:/bin")]
}

/// What the evaluator is asked, as an argument list.
///
/// Kept apart from starting the process so the list is testable as a list on
/// any machine — including one where nothing is installed at that path.
/// `alo_software::asked` keeps its lists apart from `alo_software::rented` for
/// the same reason.
#[must_use]
pub fn arguments(at: &ConfigurationAddress, address: &str, host: &str) -> Vec<String> {
    vec![at.as_str().to_owned(), address.to_owned(), host.to_owned()]
}

/// Something that evaluates an automatic configuration, at its place on an
/// alo OS machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheRentedEvaluator {
    /// The program started.
    program: PathBuf,
}

impl Default for TheRentedEvaluator {
    fn default() -> Self {
        Self::at(Path::new(THE_EVALUATOR))
    }
}

impl TheRentedEvaluator {
    /// The evaluator at its place on an alo OS machine.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::default()
    }

    /// The evaluator at some other path — a test's.
    #[must_use]
    pub fn at(program: &Path) -> Self {
        Self {
            program: program.to_path_buf(),
        }
    }

    /// The program, by path.
    #[must_use]
    pub fn program(&self) -> &Path {
        &self.program
    }
}

impl TheEvaluator for TheRentedEvaluator {
    fn asked(
        &self,
        at: &ConfigurationAddress,
        address: &str,
        host: &str,
    ) -> Result<String, NotEvaluated> {
        let mut command = Command::new(&self.program);
        command
            .args(arguments(at, address, host))
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (name, value) in nothing_of_this_machines() {
            command.env(name, value);
        }
        let output = command.output().map_err(|why| {
            // A machine where the program is not there has nothing that
            // evaluates one, which is a different answer from one that ran and
            // failed — and the road is refused either way.
            if why.kind() == std::io::ErrorKind::NotFound {
                NotEvaluated::NothingEvaluatesIt
            } else {
                NotEvaluated::DidNotAnswer {
                    said: format!("{} could not be started: {why}", self.program.display()),
                }
            }
        })?;
        if !output.status.success() {
            return Err(NotEvaluated::DidNotAnswer {
                said: String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(MOST_OF_AN_ANSWER)
                    .collect(),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .chars()
            .take(MOST_OF_AN_ANSWER)
            .collect())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn at() -> ConfigurationAddress {
        ConfigurationAddress::checked("http://wpad.example.com/proxy.config").unwrap()
    }

    /// **Nothing of this machine's reaches the evaluator.** The list is the
    /// whole environment it is given, and nothing on it names a session, a bus,
    /// a home, a credential or the proxy setting itself.
    #[test]
    fn the_evaluator_is_given_nothing_of_this_machines() {
        let given = nothing_of_this_machines();
        assert_eq!(given.len(), 3);
        let names: Vec<&str> = given.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, ["LC_ALL", "LANG", "PATH"]);
        for (name, value) in given {
            for forbidden in [
                "DBUS", "SESSION", "HOME", "XDG", "PROXY", "proxy", "TOKEN", "KEY",
            ] {
                assert!(!name.contains(forbidden), "{name}");
                assert!(!value.contains(forbidden), "{name}={value}");
            }
        }
    }

    /// The evaluator is told where the configuration is and where the road is
    /// going, and nothing else — three arguments, each already checked.
    #[test]
    fn the_evaluator_is_told_where_the_configuration_is_and_where_the_road_goes() {
        assert_eq!(
            arguments(&at(), "https://files.example.com/", "files.example.com"),
            vec![
                "http://wpad.example.com/proxy.config".to_owned(),
                "https://files.example.com/".to_owned(),
                "files.example.com".to_owned(),
            ]
        );
    }

    /// **A machine with nothing at the door refuses**, and the refusal is the
    /// one that says so — never a silent way straight out.
    #[test]
    fn a_machine_with_nothing_that_evaluates_one_refuses_rather_than_going_straight_out() {
        let nowhere = TheRentedEvaluator::at(Path::new(
            "/nonexistent/alo-proxy/nothing-evaluates-a-configuration",
        ));
        assert_eq!(
            nowhere
                .asked(&at(), "https://files.example.com/", "files.example.com")
                .unwrap_err(),
            NotEvaluated::NothingEvaluatesIt
        );
    }

    /// The evaluator on this machine is the one at the pinned path, and a
    /// test's is wherever the test put it.
    #[test]
    fn the_evaluator_on_this_machine_is_the_one_at_the_pinned_path() {
        assert_eq!(
            TheRentedEvaluator::on_this_machine().program(),
            Path::new(THE_EVALUATOR)
        );
        assert_eq!(
            TheRentedEvaluator::at(Path::new("/tmp/pretend")).program(),
            Path::new("/tmp/pretend")
        );
    }
}
