//! Holding the distribution the gates run in open, for exactly as long as a loop
//! is running.
//!
//! # What this is for
//!
//! WSL stops a distribution when nothing is using it, and everything mounted
//! inside goes with it. `/sys/fs/bpf` is a mount a boot makes and a restart
//! forgets, so a loop that runs for hours can pass its readiness check, do a
//! task, and then fail the next one because the distribution went away in
//! between — a failure that says nothing about the change and looks exactly like
//! one that does.
//!
//! One process held open there stops that. It is a `sleep`: it uses nothing,
//! decides nothing, and its only property is existing.
//!
//! # What it deliberately does not do
//!
//! **It mounts nothing.** Keeping a distribution from stopping is not the same
//! as changing what is in it, and the difference matters here: `/sys/fs/bpf` is
//! shared with whoever else is testing on this kernel, and a supervisor that
//! mounted it whenever it wanted to publish would be changing another worker's
//! ground while their tests ran. `crate::gates`' readiness check says so and
//! asks for a coordinated handoff instead.
//!
//! **It restarts nothing and starts no service.** If the distribution is not
//! there, this fails quietly and the readiness checks report it in their own
//! words — which is the honest order, because a supervisor that started WSL
//! would be deciding something about a machine it shares.
//!
//! # Its lifetime is the loop's
//!
//! Started when a run begins and killed when the value is dropped, including on
//! the way out of a failure. A helper that outlived the loop would be a process
//! nobody remembers starting, holding a distribution open for no reason — which
//! is the same untidiness as a lock left behind, and this file is where it is
//! avoided rather than where it is caused.

use std::process::Child;
#[cfg(windows)]
use std::process::{Command, Stdio};

/// How long the helper is asked to sleep for.
///
/// Long enough that no loop outlives it, and finite so that a helper somehow
/// orphaned still goes away by itself. A day is both.
#[cfg(windows)]
const FOR_A_DAY: &str = "86400";

/// A distribution held open, for as long as this value lives.
#[derive(Debug)]
pub struct Awake {
    /// The process doing nothing inside it, when one could be started.
    holding: Option<Child>,
}

impl Awake {
    /// Hold it open, or carry on without.
    ///
    /// **Failure here is not an error.** On a host with no WSL there is nothing
    /// to hold open and nothing to fail, and on one where the distribution is
    /// missing the readiness checks say so in words a person can act on. This
    /// returning empty-handed is not a reason to stop a run.
    #[must_use]
    pub fn started() -> Self {
        Self {
            holding: Self::asleep_inside_it(),
        }
    }

    /// Which process is being held open, when one is.
    ///
    /// Only a test asks: a run does not care which process it is, and the
    /// property worth checking is that it stops existing when this does.
    #[cfg(test)]
    #[must_use]
    pub fn holding(&self) -> Option<u32> {
        self.holding.as_ref().map(Child::id)
    }

    /// One process that does nothing, inside the distribution the gates use.
    #[cfg(windows)]
    fn asleep_inside_it() -> Option<Child> {
        Command::new("wsl")
            .args(["-d", "Ubuntu", "--", "sleep", FOR_A_DAY])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()
    }

    /// On a host that runs the gates directly there is no distribution to keep
    /// awake, and nothing to do.
    #[cfg(not(windows))]
    fn asleep_inside_it() -> Option<Child> {
        None
    }
}

impl Drop for Awake {
    fn drop(&mut self) {
        if let Some(holding) = self.holding.as_mut() {
            drop(holding.kill());
            drop(holding.wait());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Awake;

    /// **A helper this started is gone once the value that owns it is**, which
    /// is what keeps a machine from collecting processes nobody remembers.
    ///
    /// The process it started is asked about by number **after** the drop, so
    /// what is asserted is the operating system's answer rather than this
    /// program's intention.
    ///
    /// On a host with no WSL there is nothing to start and the test says so
    /// rather than asserting a tautology: an earlier version compared a boolean
    /// with itself, which passes everywhere and means nothing.
    #[test]
    fn a_helper_is_gone_once_the_loop_that_owns_it_is() {
        let awake = Awake::started();
        let Some(holding) = awake.holding() else {
            // Nothing to hold open here, and nothing this test can say about a
            // process that was never started.
            return;
        };
        assert!(
            crate::lock::is_alive(holding),
            "the helper was not running even before it was given back"
        );

        drop(awake);

        // Killing is not instant on either host; a bounded wait, and the
        // failure is *still there after this long* rather than *not gone yet*.
        let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::time::Instant::now() < until && crate::lock::is_alive(holding) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(
            !crate::lock::is_alive(holding),
            "process {holding} is still running after the value owning it was dropped"
        );
    }
}
