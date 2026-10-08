//! Everything the installer needs from the computer it runs on, as one seam.
//!
//! Every decision is in `crate::sequence` and the files it calls, and they are
//! tested against a machine that is a list of answers. What a real Windows adds
//! — writing to a console, reading a typed line, starting one of Windows' own
//! programs, reading and writing a file — is here as a handful of methods, and
//! `crate::on_windows` is those methods on Windows and nothing else.

use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_strings::Said;

use crate::program::Program;

/// How long the person has to read that everything is ready, before the restart.
pub const BEFORE_RESTARTING: Duration = Duration::from_secs(10);

/// What a program did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ran {
    /// Whether it said it succeeded.
    pub succeeded: bool,
    /// What it printed to its standard output.
    pub printed: String,
}

/// The computer, as the installer asks it things.
pub trait TheMachine {
    /// Put one sentence in front of the person.
    fn say(&mut self, said: &Said);

    /// Put a question in front of the person and read the one line they type.
    ///
    /// An empty line when nothing could be read: a console that is gone is a
    /// person who did not agree.
    fn ask(&mut self, said: &Said) -> String;

    /// Run one of Windows' own programs to its end.
    ///
    /// # Errors
    /// When it could not be started at all.
    fn run(&mut self, program: &Program) -> std::io::Result<Ran>;

    /// The directory the installer was downloaded into — the one its own
    /// executable is in.
    ///
    /// # Errors
    /// When Windows will not say where this program is.
    fn downloaded_into(&mut self) -> std::io::Result<PathBuf>;

    /// This program's own file, so a copy of it can be left where a person
    /// finds it (`crate::switching`).
    ///
    /// # Errors
    /// When Windows will not say where this program is.
    fn this_program(&mut self) -> std::io::Result<PathBuf>;

    /// A whole file.
    ///
    /// # Errors
    /// Whatever reading it said.
    fn read(&mut self, file: &Path) -> std::io::Result<Vec<u8>>;

    /// Write a whole file, making the directories above it.
    ///
    /// # Errors
    /// Whatever writing it said.
    fn write(&mut self, file: &Path, bytes: &[u8]) -> std::io::Result<()>;

    /// Do nothing for a while.
    fn pause(&mut self, for_as_long_as: Duration);
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected call is the failure being reported"
)]
pub(crate) mod tests {
    use super::{Ran, TheMachine};
    use crate::program::Program;

    /// A machine that answers each question with the next of these, and does
    /// nothing else.
    ///
    /// Shared rather than copied into each module that asks a question: two
    /// test machines are two ideas of what a machine does, and they drift. The
    /// same reason `crate::disks::tests::PRINTED` is reused.
    ///
    /// **Every method but `say` and `ask` panics.** A question runs no program
    /// and reads no file, so a test that reached one of them is a test whose
    /// subject did something it was not asked to - which is a failure worth
    /// hearing about rather than a default worth returning.
    pub(crate) struct Answering {
        /// What it will say, in order.
        answers: Vec<String>,
        /// How many questions it was asked.
        pub(crate) asked: usize,
    }

    impl Answering {
        /// A machine with these answers ready, in this order.
        pub(crate) fn of(answers: &[&str]) -> Self {
            Self {
                answers: answers.iter().map(|a| (*a).to_owned()).collect(),
                asked: 0,
            }
        }
    }

    impl TheMachine for Answering {
        fn say(&mut self, _said: &alo_strings::Said) {}

        /// The next answer, or an empty line once they run out.
        ///
        /// An empty line is what a console that has gone away gives, so running
        /// out is the same thing as a person who is no longer there - and every
        /// question in this installer has to be safe in that case.
        fn ask(&mut self, _said: &alo_strings::Said) -> String {
            let answer = self.answers.get(self.asked).cloned().unwrap_or_default();
            self.asked += 1;
            answer
        }

        fn run(&mut self, _program: &Program) -> std::io::Result<Ran> {
            panic!("a question runs no program")
        }

        fn downloaded_into(&mut self) -> std::io::Result<std::path::PathBuf> {
            panic!("a question downloads nothing")
        }

        fn this_program(&mut self) -> std::io::Result<std::path::PathBuf> {
            panic!("a question does not look for this program")
        }

        fn read(&mut self, _file: &std::path::Path) -> std::io::Result<Vec<u8>> {
            panic!("a question reads no file")
        }

        fn write(&mut self, _file: &std::path::Path, _bytes: &[u8]) -> std::io::Result<()> {
            panic!("a question writes no file")
        }

        fn pause(&mut self, _for_as_long_as: std::time::Duration) {
            panic!("nothing here waits on a timer")
        }
    }
}
