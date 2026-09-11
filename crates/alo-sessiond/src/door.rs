//! Who may knock at all, which is a group and not a name.
//!
//! The socket is handed to whatever group this process is in, exactly as
//! `alo-agentd` hands a person's socket to the agent's group — and this file is
//! the second half of that, which the filesystem cannot do: the kernel says who
//! connected, and a caller whose group is not the door's is turned away before
//! its line is read.
//!
//! # Why the check is here as well as on the socket
//!
//! Because the mode on a path is a fact about a path, and paths are edited.
//! `/run` is a tmpfs the image makes, the socket is created by this process at
//! start-up, and between those two facts there are several ways for a door to
//! end up reachable by somebody it was not meant for — a wrong
//! `RuntimeDirectoryMode=`, an administrator who changed something, a bug here.
//! `SO_PEERCRED` is not a mode: it is what the kernel recorded about the
//! process that called `connect`, and it cannot be edited by anybody.
//!
//! # It is the group, and deliberately not the user
//!
//! The sign-in surface is a login of its own and its number is the image's, not
//! this crate's. What this process can know about that login without being
//! configured is that it is in **this process's own group**, because that is the
//! group its own unit file put it in and the group the door was handed to. A
//! door that compared user numbers would need to be told one, which is an
//! argument that selects what a privileged component does — the thing ADR 0018
//! spends its length refusing.
//!
//! # Root's group is refused, and it is refused at start-up rather than here
//!
//! A `Group=` line left off a unit leaves a root process in root's group, and a
//! door handed to root's group is a door the sign-in surface can never reach —
//! a machine nobody can sign in to, which comes up as a blank screen rather
//! than as an error. [`Door::this_process_opens`] is where that is refused,
//! because it is a fact about how this process was *started* rather than about
//! what a door is: it is the same division `alo-boundaryd` makes, where
//! `NotLoaded::TheRootsGroup` is raised by the loader reading its own group and
//! not by the value it goes on to build.

use crate::refusing::NotADoor;

/// The group that is root's.
const THE_ROOTS_GROUP: u32 = 0;

/// The door this process opens, and the group it is handed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Door {
    /// The group a caller has to be in to be heard.
    group: u32,
}

impl Door {
    /// A door handed to this group.
    #[must_use]
    pub const fn handed_to(group: u32) -> Self {
        Self { group }
    }

    /// The door **this process** opens, given the group its unit put it in.
    ///
    /// # Errors
    ///
    /// [`NotADoor::TheRootsGroup`], which is a unit file with no `Group=` line
    /// on it rather than anything this process could do differently. It is
    /// refused here and not in [`Door::handed_to`] because what is wrong is how
    /// the process was started: a test binding a socket on a machine where it
    /// happens to be root is not a machine that cannot be signed in to, and a
    /// value that refused to exist would make that refusal untestable on the
    /// only kind of machine that can bind sockets as root.
    pub const fn this_process_opens(group: u32) -> Result<Self, NotADoor> {
        if group == THE_ROOTS_GROUP {
            return Err(NotADoor::TheRootsGroup);
        }
        Ok(Self::handed_to(group))
    }

    /// The group this door is handed to.
    #[must_use]
    pub const fn group(self) -> u32 {
        self.group
    }

    /// Whether a caller in this group may knock.
    #[must_use]
    pub const fn may_knock(self, caller: u32) -> bool {
        caller == self.group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ordinary machine: the door is the greeter's group, and the greeter
    /// is heard.
    #[test]
    fn a_caller_in_the_doors_group_may_knock() {
        let door = Door::handed_to(60990);
        assert_eq!(door.group(), 60990);
        assert!(door.may_knock(60990));
    }

    /// **Everybody else is turned away**, including the person this machine is
    /// for and including root — being able to reach a socket is not the same as
    /// being let in, and on this door nothing but the surface is.
    #[test]
    fn nobody_else_may_knock() {
        let door = Door::handed_to(60990);
        for caller in [0, 1000, 60989, 60991, u32::MAX] {
            assert!(!door.may_knock(caller), "{caller} was let in");
        }
    }

    /// **A process in root's group opens no door.** It is a unit file with no
    /// `Group=` line, and the machine it makes is one nobody can sign in to —
    /// which shows up as a screen that does nothing rather than as an error, so
    /// the process says it at start-up instead.
    #[test]
    fn a_process_in_roots_group_opens_no_door() {
        assert_eq!(Door::this_process_opens(0), Err(NotADoor::TheRootsGroup));
    }

    /// And every other group is one this process may open a door to, which is
    /// the half that would otherwise be a refusal nobody had seen succeed.
    #[test]
    fn any_other_group_is_a_door_this_process_may_open() {
        for group in [1_u32, 1000, 60989, 60990, u32::MAX] {
            assert_eq!(
                Door::this_process_opens(group).map(Door::group),
                Ok(group),
                "{group}"
            );
        }
    }
}
