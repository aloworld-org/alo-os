//! Who is at the other end of a connection, as the kernel keeps it.
//!
//! Three numbers arrive with every accepted connection: a process, a user and a
//! group. This file is what they are, and — as much — what they are allowed to
//! decide.
//!
//! # A user is an identity; a process is a detail
//!
//! [`Sides`](crate::Sides) asks a [`Caller`] which user it is and nothing else.
//! The process id is carried, because whoever is looking at a machine that is
//! behaving oddly wants to know which program was talking to it, and it is
//! never part of a decision: process ids are reused, so one that was checked a
//! moment ago describes whatever started since. A door that opened for a pid
//! would be a door that opens for whatever inherits it.
//!
//! The group is carried for the same reason, and is not a decision either — the
//! group is how a caller *reaches* the socket at all (see [`crate::place`]),
//! and being able to knock is not the same as being let in.
//!
//! # `-1` is not a user
//!
//! Every Unix call that answers with a user id answers `-1` when there is
//! none, and the underlying crate's constructors say in their own
//! documentation that they must not be handed it. So it is refused here, at the
//! one door a raw number can come in by, rather than trusted at the several
//! places one is used. A number that reached us from a machine's configuration
//! is exactly as untrusted as one that reached us from a socket.

use crate::refusing::NotAUser;

/// The value every Unix call answers with when there is no user or group.
const NOBODY_AT_ALL: u32 = u32::MAX;

/// A user of this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uid(u32);

impl Uid {
    /// The user this number names.
    ///
    /// # Errors
    ///
    /// [`NotAUser::NoSuchUser`] for `-1`, which is what a Unix call answers
    /// with when there is no user rather than a user anybody can be.
    pub const fn of(raw: u32) -> Result<Self, NotAUser> {
        if raw == NOBODY_AT_ALL {
            return Err(NotAUser::NoSuchUser);
        }
        Ok(Self(raw))
    }

    /// The number, for whoever has to hand it back to the machine.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Whether this is root.
    ///
    /// Asked in one place only — [`Sides::of`](crate::Sides::of), about the
    /// agent — and it is a question about ambient authority (ADR 0001 §2)
    /// rather than about a number being small.
    #[must_use]
    pub const fn is_root(self) -> bool {
        self.0 == 0
    }
}

/// A group on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Gid(u32);

impl Gid {
    /// The group this number names.
    ///
    /// # Errors
    ///
    /// [`NotAUser::NoSuchGroup`] for `-1`, for the reason [`Uid::of`] refuses
    /// it.
    pub const fn of(raw: u32) -> Result<Self, NotAUser> {
        if raw == NOBODY_AT_ALL {
            return Err(NotAUser::NoSuchGroup);
        }
        Ok(Self(raw))
    }

    /// The number, for whoever has to hand it back to the machine.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Who is at the other end of an accepted connection.
///
/// Made in [`crate::unix`] from what the kernel answered and nowhere else:
/// there is no constructor here a caller's own message could reach, which is
/// the whole reason this type is worth anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caller {
    /// The process that connected. Carried, never asked.
    process: i32,
    /// The user it is running as. This is the identity.
    user: Uid,
    /// The group it is running as. Carried, never asked.
    group: Gid,
}

impl Caller {
    /// What the kernel said about this connection.
    pub(crate) const fn known(process: i32, user: Uid, group: Gid) -> Self {
        Self {
            process,
            user,
            group,
        }
    }

    /// The process at the other end.
    ///
    /// For a person reading a log, never for a decision — see this file's own
    /// documentation.
    #[must_use]
    pub const fn process(&self) -> i32 {
        self.process
    }

    /// The user at the other end, which is who this caller is.
    #[must_use]
    pub const fn user(&self) -> Uid {
        self.user
    }

    /// The group at the other end.
    #[must_use]
    pub const fn group(&self) -> Gid {
        self.group
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// An ordinary user id survives being made into one and asked back.
    #[test]
    fn a_user_is_a_number() {
        assert_eq!(Uid::of(1000).unwrap().raw(), 1000);
    }

    /// **`-1` is refused**, at the one door a raw number comes in by.
    #[test]
    fn nobody_at_all_is_not_a_user() {
        assert_eq!(Uid::of(u32::MAX), Err(NotAUser::NoSuchUser));
    }

    /// And the same for a group, which is the number this crate hands to
    /// `chown`.
    #[test]
    fn nobody_at_all_is_not_a_group() {
        assert_eq!(Gid::of(u32::MAX), Err(NotAUser::NoSuchGroup));
    }

    /// The refusal is of one number and not of large ones: `65534` is
    /// `nobody`, and `nobody` signs plenty of daemons in.
    #[test]
    fn one_below_it_is_a_user() {
        assert!(Uid::of(u32::MAX - 1).is_ok());
        assert!(Uid::of(65534).is_ok());
    }

    /// Root is a user like any other here. What it may *be* is
    /// [`crate::Sides`]'s question, not this file's.
    #[test]
    fn root_is_a_user_and_says_so() {
        assert!(Uid::of(0).unwrap().is_root());
        assert!(!Uid::of(1000).unwrap().is_root());
    }

    /// A caller carries all three, and hands back exactly what it was made
    /// with.
    #[test]
    fn a_caller_carries_all_three() {
        let caller = Caller::known(4321, Uid::of(1000).unwrap(), Gid::of(1001).unwrap());
        assert_eq!(caller.process(), 4321);
        assert_eq!(caller.user().raw(), 1000);
        assert_eq!(caller.group().raw(), 1001);
    }
}
