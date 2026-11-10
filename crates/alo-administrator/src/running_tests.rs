//! The runner's order, with the machine played by the test: every refusal runs
//! nothing, and one yes runs the command exactly once.

use std::cell::{Cell, RefCell};

use super::*;

/// A machine whose person is `alo`, an administrator, signed in at session 2.
struct AMachine {
    /// The groups list, which a test changes to take the person out of wheel.
    groups: String,
    /// The session on their screen.
    session: Option<String>,
    /// What polkit says.
    says: Result<(), NotAsAdministrator>,
    /// Whether the shell starts.
    starts: bool,
    /// What polkit was asked about.
    asked: RefCell<Vec<(u32, String, String)>>,
    /// How many times a command ran.
    ran: Cell<usize>,
}

impl AMachine {
    /// The ordinary machine, on which a yes runs the command.
    fn ordinary() -> Self {
        Self {
            groups: "wheel:x:10:alo\n".to_owned(),
            session: Some("2".to_owned()),
            says: Ok(()),
            starts: true,
            asked: RefCell::new(Vec::new()),
            ran: Cell::new(0),
        }
    }
}

impl TheMachine for AMachine {
    fn logins(&self) -> String {
        "root:x:0:0::/root:/bin/bash\nalo:x:1000:1000::/home/alo:/bin/bash\n\
         alo-agent:x:60989:60989::/:/sbin/nologin\n"
            .to_owned()
    }
    fn groups(&self) -> String {
        self.groups.clone()
    }
    fn the_session_on_their_screen(&self, person: u32) -> Option<String> {
        (person == 1000).then(|| self.session.clone()).flatten()
    }
    fn the_person_said_yes(
        &self,
        person: u32,
        session: &str,
        command: &str,
    ) -> Result<(), NotAsAdministrator> {
        self.asked
            .borrow_mut()
            .push((person, session.to_owned(), command.to_owned()));
        self.says
    }
    fn run(&self, command: &ACommand) -> Result<WhatACommandDid, ItDidNotStart> {
        self.ran.set(self.ran.get() + 1);
        if self.starts {
            Ok(WhatACommandDid::of(
                Some(0),
                format!("ran {}", command.text()),
                false,
                false,
            ))
        } else {
            Err(ItDidNotStart::because("no shell"))
        }
    }
}

/// The agent service, as the kernel describes it.
fn the_agent_service() -> TheCaller {
    TheCaller {
        uid: 1000,
        groups: "0::/system.slice/alo-agentd.service\n".to_owned(),
    }
}

/// **One yes runs the command exactly once, as asked**, and polkit was asked
/// about the person, their own session and the command itself.
#[test]
fn one_yes_runs_it_once() {
    let machine = AMachine::ordinary();
    let came = answered(&machine, &the_agent_service(), "dnf install -y htop");
    assert_eq!(
        came,
        Answered::Ran {
            returned: Some(0),
            said: "ran dnf install -y htop".to_owned(),
            cut_short: false,
            took_too_long: false
        }
    );
    assert_eq!(machine.ran.get(), 1);
    assert_eq!(
        *machine.asked.borrow(),
        [(1000, "2".to_owned(), "dnf install -y htop".to_owned())]
    );
}

/// **A cancelled sheet, a wrong password and a sheet nobody could show each
/// run nothing**, and say which it was.
#[test]
fn every_no_from_polkit_runs_nothing_and_says_which() {
    for said in [
        NotAsAdministrator::TheyCancelled,
        NotAsAdministrator::NotAccepted,
        NotAsAdministrator::NothingCanAsk,
        NotAsAdministrator::NoAnswerInTime,
        NotAsAdministrator::PolkitDidNotAnswer,
    ] {
        let machine = AMachine {
            says: Err(said),
            ..AMachine::ordinary()
        };
        assert_eq!(
            answered(&machine, &the_agent_service(), "true"),
            Answered::Refused(said)
        );
        assert_eq!(machine.ran.get(), 0, "{said:?} ran something");
    }
}

/// **Nobody signed in at a screen is nothing that can ask**, and polkit is
/// not even asked.
#[test]
fn nobody_at_a_screen_is_nothing_that_can_ask() {
    let machine = AMachine {
        session: None,
        ..AMachine::ordinary()
    };
    assert_eq!(
        answered(&machine, &the_agent_service(), "true"),
        Answered::Refused(NotAsAdministrator::NothingCanAsk)
    );
    assert!(machine.asked.borrow().is_empty());
    assert_eq!(machine.ran.get(), 0);
}

/// **An account that is not an administrator is told so**, before polkit
/// could offer somebody else's password to be typed.
#[test]
fn an_account_that_is_not_an_administrator_is_told_so() {
    let machine = AMachine {
        groups: "wheel:x:10:\n".to_owned(),
        ..AMachine::ordinary()
    };
    assert_eq!(
        answered(&machine, &the_agent_service(), "true"),
        Answered::Refused(NotAsAdministrator::NotAnAdministrator)
    );
    assert!(machine.asked.borrow().is_empty());
    assert_eq!(machine.ran.get(), 0);
}

/// **Only the agent service, running as a person, may ask** — not the
/// person's own terminal, not the agent's login, not root.
#[test]
fn only_the_agent_service_may_ask() {
    let machine = AMachine::ordinary();
    for caller in [
        TheCaller {
            uid: 1000,
            groups: "0::/user.slice/user-1000.slice/session-2.scope\n".to_owned(),
        },
        TheCaller {
            uid: 60989,
            groups: "0::/system.slice/alo-agentd.service\n".to_owned(),
        },
        TheCaller {
            uid: 0,
            groups: "0::/system.slice/alo-agentd.service\n".to_owned(),
        },
        TheCaller {
            uid: 1000,
            groups: "0::/system.slice/alo-agentd.services\n".to_owned(),
        },
    ] {
        assert_eq!(
            answered(&machine, &caller, "true"),
            Answered::Refused(NotAsAdministrator::NotTheAgentService),
            "{caller:?}"
        );
    }
    assert!(machine.asked.borrow().is_empty());
    assert_eq!(machine.ran.get(), 0);
    let inside = TheCaller {
        uid: 1000,
        groups: "0::/system.slice/alo-agentd.service/turn-3\n".to_owned(),
    };
    assert!(
        inside.is_the_agent_service(),
        "a group the service made is still its own"
    );
}

/// **A line that is not one command is refused before anything is asked.**
#[test]
fn a_line_that_is_not_a_command_is_refused() {
    let machine = AMachine::ordinary();
    for line in ["", "   ", "true\nrm -rf /"] {
        assert_eq!(
            answered(&machine, &the_agent_service(), line),
            Answered::Refused(NotAsAdministrator::NotACommand),
            "{line:?}"
        );
    }
    assert!(machine.asked.borrow().is_empty());
}

/// **A yes and a shell that will not start is said**, not taken for a run.
#[test]
fn a_shell_that_will_not_start_is_said() {
    let machine = AMachine {
        starts: false,
        ..AMachine::ordinary()
    };
    assert_eq!(
        answered(&machine, &the_agent_service(), "true"),
        Answered::Refused(NotAsAdministrator::DidNotStart)
    );
    assert_eq!(machine.ran.get(), 1);
}
