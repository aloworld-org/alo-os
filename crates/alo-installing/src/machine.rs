//! Everything the sequence needs from the machine it runs on, as one seam.
//!
//! The sequence in `crate::sequence` is where every decision is, and it is
//! tested against a machine that is a list of answers. What a real machine adds
//! — reading `/proc/cmdline`, waiting for udev, starting a program, writing to a
//! console — is here as five methods, and `crate::running` is those five on
//! Linux and nothing else.

use std::time::Duration;

use alo_strings::Said;

use crate::disk::DiskName;
use crate::program::{Program, Ran};

/// How long the sequence waits for the chosen disk to appear.
///
/// A disk that is there appears within a second or two of the environment
/// starting, as the kernel and udev find it; half a minute is for a slow
/// controller, and not for a disk that is not connected, which never appears.
pub const THE_DISK_APPEARS_WITHIN: Duration = Duration::from_secs(30);

/// How often the person is told the write is still going.
pub const STILL_EVERY: Duration = Duration::from_secs(60);

/// How long the person has to read that alo OS is installed, before the
/// restart.
pub const BEFORE_RESTARTING: Duration = Duration::from_secs(10);

/// The machine, as the sequence asks it things.
pub trait TheMachine {
    /// Put one sentence in front of the person, and in the machine's log.
    fn say(&mut self, said: &Said);

    /// The kernel command line this environment was started with.
    ///
    /// # Errors
    /// Whatever reading it said.
    fn command_line(&mut self) -> std::io::Result<String>;

    /// Wait up to `at_most` for the named disk to appear, and say where the
    /// kernel put it.
    ///
    /// [`None`] when it did not appear.
    fn wait_for(&mut self, disk: &DiskName, at_most: Duration) -> Option<std::path::PathBuf>;

    /// Run one program to its end.
    ///
    /// While it runs, `still` is said every `every`: a program that runs for a
    /// quarter of an hour on a screen that does not change is a machine a
    /// person turns off.
    ///
    /// # Errors
    /// When it could not be started at all.
    fn run(&mut self, program: &Program, still: &Said, every: Duration) -> std::io::Result<Ran>;

    /// Do nothing for a while.
    fn pause(&mut self, for_as_long_as: Duration);
}
