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

    /// How many bytes a file holds, or [`None`] when it is not there.
    ///
    /// **Added 2026-10-22, and the reason is a fault this replaced.** The road
    /// that writes a stick asked *is the image there* by calling
    /// [`Self::read`] — *a whole file* — and throwing every byte away. The image
    /// is **gigabytes**: `docs/booting.md` measured a written disk at 9.6 GB.
    /// On the kind of laptop a person writes a stick from that is an
    /// out-of-memory failure **before the first warning is drawn**, and on a
    /// large one a multi-second stall for a question `metadata` answers in
    /// microseconds.
    ///
    /// **No test could have found it.** A fake's `read` returns a small `Vec`,
    /// which is correct and fast and says nothing about the real thing. Found by
    /// the desktop lane reading the code, which is the only mechanism that
    /// finds this family.
    ///
    /// It answers the size rather than a yes, because the write road needs the
    /// size anyway: a device smaller than the image is a refusal somebody will
    /// want, and asking twice would be two answers about one file.
    ///
    /// # Errors
    /// Nothing. A file that is not there, or cannot be asked about, is [`None`]
    /// — from here those are one thing, and the sentence a person reads is the
    /// same either way.
    fn how_big_is(&mut self, file: &Path) -> Option<u64>;

    /// Write a file onto a whole device, erasing what was on it.
    ///
    /// **The only call in this program that destroys something a person did not
    /// ask alo OS to install onto.** `docs/features.md` carries the longest
    /// warning in the document about it: *a stick that is written by mistake is
    /// somebody's photographs.* So it is a method of its own rather than
    /// [`Self::write`] with a device path — a seam a reader can find, and one no
    /// other road in this crate can reach by accident.
    ///
    /// `device` is the device as this machine names it, which on Windows means a
    /// physical-drive path and not a drive letter: a letter names a volume, and
    /// writing a whole-disk image through one would write inside the partition
    /// it already has rather than replacing the partition table.
    ///
    /// `told` is called as it goes, with how many bytes have been written and
    /// how many there are in all, so a person watching sees
    /// `installing.how-far-the-write-has-got` move. It is called from this
    /// thread; nothing here is asynchronous.
    ///
    /// # Errors
    /// Whatever the device said. A refusal here has written **some** of the
    /// image in the general case, which is why
    /// `installing.leave-the-stick-in` warns before this starts rather than
    /// after it fails.
    fn wrote_the_device(
        &mut self,
        device: &str,
        from: &Path,
        told: &mut dyn FnMut(u64, u64),
    ) -> std::io::Result<()>;
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

        fn how_big_is(&mut self, _file: &std::path::Path) -> Option<u64> {
            panic!("a question asks about no file")
        }

        fn wrote_the_device(
            &mut self,
            _device: &str,
            _from: &std::path::Path,
            _told: &mut dyn FnMut(u64, u64),
        ) -> std::io::Result<()> {
            panic!("a question writes no device")
        }

        fn pause(&mut self, _for_as_long_as: std::time::Duration) {
            panic!("nothing here waits on a timer")
        }
    }
}
