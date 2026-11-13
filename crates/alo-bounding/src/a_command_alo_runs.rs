//! **A control group for one command alo runs, so the kernel can count what it
//! sends** — and the way this service's thread steps into it for the instant
//! the command begins, and back out
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
//! as amended 2026-11-13).
//!
//! The owner, 2026-11-13: *"Yes, count alo's commands"* — bytes only, only for
//! the seconds alo's command runs, and only alo's command, never the person's.
//! The kernel's counter counts a **control group**, so a command has to have
//! one of its own, and it has to be in it from its first instruction: a command
//! moved in after it started could have sent something first.
//!
//! # Nothing is started here, and nothing is bounded
//!
//! This crate starts no program — `tests/a_turn_is_this_thread.rs` reads its
//! source and fails on the words that would. What is here is the same thing a
//! turn is: **one thread of this service writes one byte into a control group's
//! `cgroup.threads`**. Whatever that thread then starts — and the caller's
//! closure is the only thing it does in there — is born inside the group,
//! because a new process belongs to the group of the thread that made it. Then
//! the thread writes itself back into `home` through the descriptor
//! [`Turns::under`] opened at start, and nothing of this service is left
//! inside.
//!
//! **The group bounds nothing.** It has no entry in `BOUNDS`, so the hooks find
//! nothing for it and let everything through, as for every ordinary program on
//! the machine: a command alo runs at full access is not boxed (ADR 0149). It
//! is made beside the turns, threaded as they are, because this service's
//! subtree is a threaded one and a domain group made in it refuses a process
//! outright (`crate::Turns` has the measurement).
//!
//! # Only a group made here can be stepped into
//!
//! [`Turns::from_inside`] takes an [`ACountedGroup`], which only
//! [`Turns::a_group_for_a_command`] makes, and never a [`Cgroup`]: a door that
//! put this service's thread into any control group would be a door into a
//! turn's, whose boundary would then hold the service.

use std::fs;
use std::io::Write as _;

use crate::{cgroup::Cgroup, failing::NotBounded, turns::Turns};

/// What `cgroup.threads` reads as *the thread writing this*.
const THIS_THREAD: &str = "0";

/// A control group made for one command alo runs, and for nothing else.
#[derive(Debug)]
pub struct ACountedGroup {
    /// The group, beside this service's turns.
    group: Cgroup,
}

impl ACountedGroup {
    /// The number the kernel's counter knows this group by — what
    /// `bpf_skb_cgroup_id` answers for a socket made inside it.
    ///
    /// # Errors
    /// [`NotBounded::NotAPlace`] if the group cannot be looked at.
    pub fn id(&self) -> Result<u64, NotBounded> {
        self.group.id()
    }

    /// Where it is.
    #[must_use]
    pub fn at(&self) -> &std::path::Path {
        self.group.at()
    }

    /// Take the group away, which the kernel allows once nothing is in it.
    ///
    /// # Errors
    /// [`NotBounded::Cgroup`] if something the command started is still running
    /// in it, or the hierarchy would not remove it. The group is left as it
    /// was, uncounted once its count has ended.
    pub fn removed(self) -> Result<(), NotBounded> {
        self.group.removed()
    }
}

impl Turns {
    /// A control group of its own for one command alo runs, made beside this
    /// service's turns and bounding nothing.
    ///
    /// # Errors
    /// [`NotBounded::NotAName`] for a name that is a path, and
    /// [`NotBounded::Cgroup`] for anything the hierarchy would not do.
    pub fn a_group_for_a_command(&self, named: &str) -> Result<ACountedGroup, NotBounded> {
        Ok(ACountedGroup {
            group: self.beginning(named)?,
        })
    }

    /// Call `start` with this thread **inside** `group`, so whatever it starts
    /// is born there, and bring the thread back home straight after.
    ///
    /// Answers `Err` only when the thread could not get in, and then `start`
    /// was never called. Once it was called, its answer comes back whatever
    /// happens next — beside the reason the thread could not come home, if it
    /// could not, because what `start` started is the caller's to wait for
    /// either way.
    ///
    /// # Errors
    /// [`NotBounded::Cgroup`] when the group would not take this thread.
    pub fn from_inside<T>(
        &self,
        group: &ACountedGroup,
        start: impl FnOnce() -> T,
    ) -> Result<(T, Option<NotBounded>), NotBounded> {
        let door = group.group.threads();
        fs::write(&door, THIS_THREAD).map_err(|why| NotBounded::Cgroup {
            what: "cannot put this thread into a command's control group at",
            path: door.display().to_string(),
            why,
        })?;
        let answer = start();
        let mut back = self.the_way_back();
        let not_back = back
            .write_all(THIS_THREAD.as_bytes())
            .err()
            .map(|why| NotBounded::Cgroup {
                what: "cannot bring this thread home from a command's control group to",
                path: self.home().at().display().to_string(),
                why,
            });
        Ok((answer, not_back))
    }
}
