//! The two sides of the socket, and the one question that decides which is
//! which.
//!
//! `crates/alo-protocol` has two public request types and two public answer
//! types, and nothing in it can turn one into the other. What it cannot do is
//! say which of them a given connection gets, because a message carries nothing
//! about who sent it — deliberately, and
//! `docs/contracts/daemon-protocol.md` says so: there is no `agent` field, no
//! `as`, and no token that could be copied. This file is the answer, and the
//! answer is the user the kernel says is at the other end.
//!
//! # Two users, or no socket at all
//!
//! [`Sides::of`] refuses a machine where the person and the agent are the same
//! login. It is the load-bearing refusal here: on such a machine every
//! connection would satisfy both tests, whichever one was asked first would
//! win, and *the side that proposes a change cannot approve it* would be a
//! sentence in a contract with nothing underneath it. Refusing to open the
//! socket at all is the only honest answer — a daemon that started anyway,
//! with both doors quietly become one, is exactly the failure ADR 0001 §5
//! exists to prevent, running.
//!
//! # And the agent is not root
//!
//! ADR 0001 §2: no ambient authority. An agent running as root would hold
//! authority the person themselves does not have, and every grant in
//! `alo-capability` would be a rule it is being asked to follow rather than one
//! it is under. That is refused here, where the machine is described, rather
//! than left to whoever writes the service file.
//!
//! **The person may be root, and that is not this file's to refuse.** ADR 0001
//! §2 says `alo-agentd` runs as the signed-in person and never as root, which
//! is a fact about the *process* and belongs to the process — queue item 21d,
//! which is what has a `main` to refuse it in. A number in a configuration
//! file, on its own, is not yet a daemon running as anybody.
//!
//! # What is not asked
//!
//! Not the group: it is how a caller reaches the socket (see [`crate::place`]),
//! and reaching a door is not being let through it. Not the process: it is
//! reused, and a decision made about one is a decision about whatever starts
//! next. See [`crate::caller`].

use crate::caller::{Caller, Gid, Uid};
use crate::refusing::{NotACaller, NotTwoSides};

/// Which of `alo-protocol`'s two doors a connection is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// An agent, during a turn: reads, proposals and questions.
    Agent,
    /// The person's own shell: approvals, declines and what is waiting.
    Person,
}

/// The two users this machine has, and the group they meet in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sides {
    /// The signed-in person, who `alo-agentd` runs as.
    person: Uid,
    /// The agent, which is a login of its own.
    agent: Uid,
    /// The group that may reach the socket at all.
    shared: Gid,
}

impl Sides {
    /// The two sides of this machine's socket.
    ///
    /// `shared` is the group the socket and its directory are handed to, so
    /// that the agent — which is a different user — can reach the path at all.
    /// It grants nothing on its own: being in it means being able to knock.
    ///
    /// # Errors
    ///
    /// [`NotTwoSides::OneUser`] if the person and the agent are one login, and
    /// [`NotTwoSides::AgentAsRoot`] if the agent is root. Both are described in
    /// this file's own documentation, and in both of them there is no `Sides`
    /// to open a socket with.
    pub const fn of(person: Uid, agent: Uid, shared: Gid) -> Result<Self, NotTwoSides> {
        if person.raw() == agent.raw() {
            return Err(NotTwoSides::OneUser { uid: person.raw() });
        }
        if agent.is_root() {
            return Err(NotTwoSides::AgentAsRoot);
        }
        Ok(Self {
            person,
            agent,
            shared,
        })
    }

    /// The signed-in person.
    #[must_use]
    pub const fn person(&self) -> Uid {
        self.person
    }

    /// The agent's own login.
    #[must_use]
    pub const fn agent(&self) -> Uid {
        self.agent
    }

    /// The group that may reach the socket.
    #[must_use]
    pub const fn shared(&self) -> Gid {
        self.shared
    }

    /// Which door this caller is on.
    ///
    /// # Errors
    ///
    /// [`NotACaller::Stranger`] for anybody else. There is no third door and no
    /// lesser one: a connection that is neither the person nor the agent is
    /// closed, because every request in `alo-protocol` is one of theirs.
    pub const fn which(&self, caller: &Caller) -> Result<Side, NotACaller> {
        let uid = caller.user().raw();
        if uid == self.agent.raw() {
            return Ok(Side::Agent);
        }
        if uid == self.person.raw() {
            return Ok(Side::Person);
        }
        Err(NotACaller::Stranger { uid })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The person, the agent and a group, as an ordinary machine would have
    /// them.
    fn a_machine() -> Sides {
        Sides::of(
            Uid::of(1000).unwrap(),
            Uid::of(989).unwrap(),
            Gid::of(989).unwrap(),
        )
        .unwrap()
    }

    /// A caller running as this user, with a process id and group that are
    /// nothing to do with the decision.
    fn calling_as(uid: u32) -> Caller {
        Caller::known(4321, Uid::of(uid).unwrap(), Gid::of(989).unwrap())
    }

    /// The agent's user gets the agent's door and the person's gets the
    /// person's.
    #[test]
    fn each_user_gets_their_own_door() {
        let sides = a_machine();
        assert_eq!(sides.which(&calling_as(989)).unwrap(), Side::Agent);
        assert_eq!(sides.which(&calling_as(1000)).unwrap(), Side::Person);
    }

    /// **Everybody else is a stranger**, and the refusal names them so a log
    /// says who came knocking.
    #[test]
    fn everybody_else_is_a_stranger() {
        let sides = a_machine();
        let refused = sides.which(&calling_as(1001)).unwrap_err();
        assert!(matches!(refused, NotACaller::Stranger { uid: 1001 }));
    }

    /// Root is a stranger like any other. It can do as it likes to the machine
    /// by other means, and what it may not do is arrive here holding the
    /// person's approvals — ADR 0004: no administrator acts as a person.
    #[test]
    fn root_is_a_stranger_too() {
        let sides = a_machine();
        assert!(matches!(
            sides.which(&calling_as(0)).unwrap_err(),
            NotACaller::Stranger { uid: 0 }
        ));
    }

    /// **One login cannot be both sides.** This is the refusal the division
    /// stands on: on such a machine the door that proposes a change would also
    /// be the door that approves it.
    #[test]
    fn one_login_cannot_be_both_sides() {
        let both = Sides::of(
            Uid::of(1000).unwrap(),
            Uid::of(1000).unwrap(),
            Gid::of(1000).unwrap(),
        );
        assert_eq!(both, Err(NotTwoSides::OneUser { uid: 1000 }));
    }

    /// An agent running as root is refused where the machine is described,
    /// rather than where somebody remembers to check (ADR 0001 §2).
    #[test]
    fn the_agent_may_not_be_root() {
        let rooted = Sides::of(
            Uid::of(1000).unwrap(),
            Uid::of(0).unwrap(),
            Gid::of(1000).unwrap(),
        );
        assert_eq!(rooted, Err(NotTwoSides::AgentAsRoot));
    }

    /// The person may be root here, because *the daemon does not run as root*
    /// is a fact about a running process and is refused by the process. A
    /// machine described this way still has two doors.
    #[test]
    fn the_person_being_root_is_the_processs_refusal_and_not_this_one() {
        let sides = Sides::of(
            Uid::of(0).unwrap(),
            Uid::of(989).unwrap(),
            Gid::of(989).unwrap(),
        )
        .unwrap();
        assert_eq!(sides.which(&calling_as(0)).unwrap(), Side::Person);
    }

    /// **The group decides nothing.** A caller in the shared group whose user
    /// is neither is a stranger — being able to reach the socket is not being
    /// allowed through it.
    #[test]
    fn the_group_opens_no_door() {
        let sides = a_machine();
        let in_the_group = Caller::known(77, Uid::of(4242).unwrap(), Gid::of(989).unwrap());
        assert!(matches!(
            sides.which(&in_the_group).unwrap_err(),
            NotACaller::Stranger { uid: 4242 }
        ));
    }

    /// **The process decides nothing either.** Two connections from one user
    /// land on one door whatever process made them, because a process id is
    /// reused and a door that turned on one would turn on whatever inherits it.
    #[test]
    fn the_process_decides_nothing() {
        let sides = a_machine();
        let first = Caller::known(11, Uid::of(989).unwrap(), Gid::of(989).unwrap());
        let second = Caller::known(999_999, Uid::of(989).unwrap(), Gid::of(0).unwrap());
        assert_eq!(sides.which(&first).unwrap(), sides.which(&second).unwrap());
        assert_eq!(sides.which(&first).unwrap(), Side::Agent);
    }

    /// The three the machine was described with come back as they went in.
    #[test]
    fn a_machine_says_who_it_has() {
        let sides = a_machine();
        assert_eq!(sides.person().raw(), 1000);
        assert_eq!(sides.agent().raw(), 989);
        assert_eq!(sides.shared().raw(), 989);
    }
}
