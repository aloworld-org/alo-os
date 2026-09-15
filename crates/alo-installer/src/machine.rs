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
