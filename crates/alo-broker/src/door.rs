//! Who may ask at all: the agent's service, as the kernel names it.
//!
//! What the kernel records about the process that called `connect` —
//! `SO_PEERCRED` — is the only thing this decides on. It is not a field in a
//! message and nothing a caller can write; `crate::unix` asks for it and this
//! file compares.
//!
//! # A user, and why not a group
//!
//! `alo-sessiond` compares groups, because the greeter is nobody in particular
//! and a group is all it has. `alo-agentd` is somebody: it runs as the person's
//! own login (`image/usr/lib/systemd/system/alo-agentd.service`, ADR 0001 §2),
//! and the logins it must be told apart from are all users of their own — the
//! **agent** (60989), the greeter, the model service, the converter, root. A
//! group would be the wrong question: the person is a member of the agent's
//! group, so that group is a door the agent's own login could knock on.
//!
//! # Root is refused a door, at start-up
//!
//! A broker told the agent's service is root would be a broker any root process
//! could ask — and every other privileged component, which is the whole of what
//! the broker exists to keep apart from the agent. [`Door::for_the_agent_service`]
//! refuses it, for `alo_sessiond::Door::this_process_opens`' reason: it is a
//! fact about how the process was configured, and a test running as root must
//! still be able to make a door with [`Door::handed_to`].

/// The user that is root's.
const ROOT: u32 = 0;

/// Who the door is handed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Door {
    /// The user the kernel must name for a caller to be heard.
    agent_service: u32,
}

/// Why there is no door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotADoor {
    /// The agent's service was said to be root.
    ToldTheAgentServiceIsRoot,
}

impl std::fmt::Display for NotADoor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToldTheAgentServiceIsRoot => write!(
                f,
                "the broker was told alo-agentd runs as root, and it never does: its door would \
                 then hear every privileged process on the machine (ADR 0001 §2)"
            ),
        }
    }
}

impl std::error::Error for NotADoor {}

impl Door {
    /// A door handed to this user, whoever it is.
    #[must_use]
    pub const fn handed_to(agent_service: u32) -> Self {
        Self { agent_service }
    }

    /// The door a running broker opens, for the user `alo-agentd` runs as.
    ///
    /// # Errors
    /// [`NotADoor::ToldTheAgentServiceIsRoot`].
    pub const fn for_the_agent_service(agent_service: u32) -> Result<Self, NotADoor> {
        if agent_service == ROOT {
            return Err(NotADoor::ToldTheAgentServiceIsRoot);
        }
        Ok(Self::handed_to(agent_service))
    }

    /// The user this door is handed to.
    #[must_use]
    pub const fn agent_service(self) -> u32 {
        self.agent_service
    }

    /// Whether the kernel's answer about a caller lets it be heard.
    ///
    /// [`None`] — the kernel would not say — is not heard. Who is at the door is
    /// the first thing this decides on, so not knowing is not a lesser answer.
    #[must_use]
    pub const fn hears(self, caller: Option<u32>) -> bool {
        matches!(caller, Some(user) if user == self.agent_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The person's login, which `alo-agentd` runs as.
    const THE_AGENT_SERVICE: u32 = 1000;

    /// **The agent's service is heard.**
    #[test]
    fn the_agent_service_is_heard() {
        assert!(Door::handed_to(THE_AGENT_SERVICE).hears(Some(THE_AGENT_SERVICE)));
    }

    /// **Nobody else is** — the agent's own login, root, the greeter, the model
    /// service, and a caller the kernel would say nothing about.
    #[test]
    fn nobody_else_is_heard() {
        let door = Door::handed_to(THE_AGENT_SERVICE);
        for caller in [60989, 0, 60990, 60991, 60992, 999, 1001, u32::MAX] {
            assert!(!door.hears(Some(caller)), "{caller} was heard");
        }
        assert!(!door.hears(None));
    }

    /// **Root is no agent's service.**
    #[test]
    fn a_broker_told_the_agent_service_is_root_opens_no_door() {
        assert_eq!(
            Door::for_the_agent_service(0),
            Err(NotADoor::ToldTheAgentServiceIsRoot)
        );
        assert_eq!(
            Door::for_the_agent_service(THE_AGENT_SERVICE).map(Door::agent_service),
            Ok(THE_AGENT_SERVICE)
        );
    }
}
