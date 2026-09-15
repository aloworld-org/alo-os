//! A process held by its descriptor, so that its number cannot come to mean
//! another process while it is being asked about.
//!
//! A process **number** is a name the kernel gives back when the process is
//! gone, and gives to the next process that needs one. Anything that reads
//! `/proc/<pid>/…` after learning `<pid>` from somebody else is reading about
//! whichever process holds that number *now*. A process **descriptor** (a
//! pidfd) is different: it names one process for as long as it is open, and
//! says, once that process has ended, that it has.
//!
//! [`HeldProcess`] is the backend's hold on the process behind a connection:
//!
//! - [`HeldProcess::given`] takes the descriptor a bus daemon hands over as
//!   `ProcessFD`, which the daemon took from the socket itself, so it is the
//!   connection's process by construction;
//! - [`HeldProcess::opened_for`] opens one for the number a bus daemon that
//!   gives no descriptor handed over (`pidfd_open`), which is the connection's
//!   process only if that number was not reused before it was opened — the
//!   caller checks that with the bus afterwards (`crate::caller`);
//! - [`HeldProcess::is_still_alive`] says, after something was read through
//!   the number, whether the process held is still the one the number names.
//!   While it is, the number cannot have been given to anybody else, so what was
//!   read was read about the process held.
//!
//! The number behind a descriptor is read from the kernel's own description of
//! it, `/proc/self/fdinfo/<fd>`, whose `Pid:` line is the process's number in
//! this process's namespace and `-1` once the process has ended and been reaped.
//! This is always the real `/proc`, whatever directory
//! [`crate::Sandboxes`] reads sandboxes under: the descriptor is this process's.

use std::os::fd::{AsFd, AsRawFd, OwnedFd};

use rustix::process::{Pid, PidfdFlags, pidfd_open};

/// One process, held by a descriptor, with the number it had when it was held.
#[derive(Debug)]
pub struct HeldProcess {
    /// The descriptor, which names this process and no other.
    descriptor: OwnedFd,
    /// Its number, in this process's namespace, when it was held.
    number: u32,
}

impl HeldProcess {
    /// Hold the process `descriptor` names — the `ProcessFD` a bus daemon
    /// handed over — when it names a living process, and `said`, the number
    /// the daemon gave beside it, is that process's.
    ///
    /// [`None`] for a descriptor that is not a process's, a process that has
    /// already ended, one outside this namespace, or a number that disagrees
    /// with the descriptor: a daemon that says two different things is not
    /// believed about either.
    #[must_use]
    pub fn given(descriptor: OwnedFd, said: Option<u32>) -> Option<Self> {
        let number = number_behind(&descriptor)?;
        if said.is_some_and(|said| said != number) {
            return None;
        }
        Some(Self { descriptor, number })
    }

    /// Hold process `number`, when there is one.
    ///
    /// [`None`] when no process has that number, the kernel will not open a
    /// descriptor for it, or it ended while it was being opened.
    #[must_use]
    pub fn opened_for(number: u32) -> Option<Self> {
        let pid = Pid::from_raw(i32::try_from(number).ok()?)?;
        let descriptor = pidfd_open(pid, PidfdFlags::empty()).ok()?;
        (number_behind(&descriptor)? == number).then_some(Self { descriptor, number })
    }

    /// The process's number when it was held — which is still its number,
    /// and nobody else's, for as long as [`Self::is_still_alive`] says so.
    #[must_use]
    pub const fn number(&self) -> u32 {
        self.number
    }

    /// Whether the process held has not ended, so that its number still names
    /// it and has not been given to another process.
    #[must_use]
    pub fn is_still_alive(&self) -> bool {
        number_behind(&self.descriptor) == Some(self.number)
    }
}

/// The number of the living process `descriptor` names, from the kernel's
/// description of the descriptor.
///
/// [`None`] when the descriptor is not a process descriptor (it has no `Pid:`
/// line), the process has ended (`-1`), or it is outside this namespace (`0`).
fn number_behind(descriptor: &impl AsFd) -> Option<u32> {
    let described = std::fs::read_to_string(format!(
        "/proc/self/fdinfo/{}",
        descriptor.as_fd().as_raw_fd()
    ))
    .ok()?;
    let number = described
        .lines()
        .find_map(|line| line.strip_prefix("Pid:"))?
        .trim()
        .parse::<i64>()
        .ok()?;
    u32::try_from(number).ok().filter(|number| *number > 0)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    /// A process of the test's own that waits until it is ended.
    fn a_waiting_process() -> std::process::Child {
        Command::new("sleep")
            .arg("600")
            .stdin(Stdio::null())
            .spawn()
            .unwrap()
    }

    /// **A living process is held by its number, and is alive while it lives.**
    #[test]
    fn a_living_process_is_held_and_alive() {
        let ours = HeldProcess::opened_for(std::process::id()).unwrap();
        assert_eq!(ours.number(), std::process::id());
        assert!(ours.is_still_alive());

        let mut other = a_waiting_process();
        let held = HeldProcess::opened_for(other.id()).unwrap();
        assert_eq!(held.number(), other.id());
        assert!(held.is_still_alive());
        other.kill().unwrap();
        other.wait().unwrap();
    }

    /// **A process that ended after it was held is no longer alive** — the
    /// check that refuses what was read through a number that may now be
    /// another process's.
    #[test]
    fn a_process_that_ended_after_it_was_held_is_not_alive() {
        let mut other = a_waiting_process();
        let held = HeldProcess::opened_for(other.id()).unwrap();
        other.kill().unwrap();
        other.wait().unwrap();
        assert!(!held.is_still_alive());
    }

    /// **A number no process has, and a process already ended, are not held.**
    #[test]
    fn a_number_with_no_process_is_not_held() {
        let mut other = a_waiting_process();
        let number = other.id();
        other.kill().unwrap();
        other.wait().unwrap();
        // The number could in principle have been given to another process
        // already; what matters is that it is never held as the one that ended.
        if let Some(held) = HeldProcess::opened_for(number) {
            assert!(held.is_still_alive());
        }
        for nobody in [0, u32::MAX, u32::try_from(i32::MAX).unwrap()] {
            assert!(HeldProcess::opened_for(nobody).is_none(), "{nobody}");
        }
    }

    /// **A descriptor handed over is held only when it is a living process's
    /// and agrees with the number beside it.**
    #[test]
    fn a_descriptor_handed_over_is_held_only_when_it_agrees() {
        let ours = || {
            pidfd_open(
                Pid::from_raw(i32::try_from(std::process::id()).unwrap()).unwrap(),
                PidfdFlags::empty(),
            )
            .unwrap()
        };
        assert_eq!(
            HeldProcess::given(ours(), Some(std::process::id()))
                .unwrap()
                .number(),
            std::process::id()
        );
        assert!(HeldProcess::given(ours(), None).is_some());
        assert!(HeldProcess::given(ours(), Some(std::process::id() + 1)).is_none());

        let not_a_process = OwnedFd::from(std::fs::File::open("/proc/self/stat").unwrap());
        assert!(HeldProcess::given(not_a_process, None).is_none());

        let mut other = a_waiting_process();
        let theirs = pidfd_open(
            Pid::from_raw(i32::try_from(other.id()).unwrap()).unwrap(),
            PidfdFlags::empty(),
        )
        .unwrap();
        other.kill().unwrap();
        other.wait().unwrap();
        assert!(HeldProcess::given(theirs, None).is_none());
    }
}
