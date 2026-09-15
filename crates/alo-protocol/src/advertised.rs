//! What a person's shell is told their machine says about itself on the local
//! network.
//!
//! One request on the person's door — `advertised`, carrying nothing — and this
//! is the shape of its answer. *Nothing leaves silently* is law 1's reason, and
//! it is a person's to check: an alo machine tells everything on the link that
//! it exists and, where root installed a workspace server, that it hosts a
//! workspace. Without this answer the only way to read what a machine is
//! telling the office is a packet capture.
//!
//! # What is here is what the advertisement carries, and nothing else
//!
//! `machine` is this machine's identity — the thirty-two characters its presence
//! is advertised under. `port` is the port that presence names, which is where
//! the wire answers proposals, verbs and questions. `workspace` is one of three
//! things: [`HostedWorkspace::Hosts`] with the port the workspace this machine
//! answers for is advertised at, [`HostedWorkspace::HostsNone`] when nothing is
//! advertised because nothing is installed, and
//! [`HostedWorkspace::NotAdvertised`] when a workspace file was there and was
//! refused — so nothing is advertised — with the sentence saying so in the
//! person's language.
//!
//! # And what is not
//!
//! **No address.** The advertisement names no address — a machine answering
//! discovery is heard at whatever address it answered from, and that is a
//! listener's measurement, not something this machine says. **No hostname, no
//! person's name, no model list, no pairing**, because none of those is
//! advertised (ADR 0003: discovery reveals presence and nothing else), and a
//! field here for one would be a field describing an advertisement that does not
//! exist. **No path, owner or mode** in a refusal: those are for whoever
//! installed the server, and they read them in the service log.
//!
//! **No setting goes the other way.** There is no request that changes what is
//! advertised — no *advertise as*, no *discovery off* — and the request this
//! answers carries no field for a port or a path.

use std::num::NonZeroU16;

use alo_strings::Said;
use serde::{Deserialize, Serialize};

use crate::wording::Wording;

/// What this machine advertises on the local network, as a shell draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Advertised {
    /// This machine's identity, as its presence is advertised under it.
    machine: String,
    /// The port its presence names.
    port: u16,
    /// The workspace it answers for, that it answers for none, or why a
    /// workspace installed is not advertised.
    workspace: HostedWorkspace,
}

/// What this machine says about a workspace it hosts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum HostedWorkspace {
    /// A workspace is advertised under this machine's identity, at this port.
    Hosts {
        /// The port it is advertised at.
        port: NonZeroU16,
    },
    /// No workspace is advertised, because none is installed.
    HostsNone {},
    /// No workspace is advertised, because what says one is installed was
    /// refused — and this is what the person is told about it.
    NotAdvertised(Wording),
}

impl HostedWorkspace {
    /// A workspace advertised at `port`.
    #[must_use]
    pub const fn hosts(port: NonZeroU16) -> Self {
        Self::Hosts { port }
    }

    /// No workspace, because none is installed.
    #[must_use]
    pub const fn none() -> Self {
        Self::HostsNone {}
    }

    /// No workspace advertised, because what says one is installed was
    /// refused, in the words of whoever refused it.
    #[must_use]
    pub fn not_advertised(said: &Said) -> Self {
        Self::NotAdvertised(Wording::of(said))
    }

    /// The port a workspace is advertised at, when one is.
    #[must_use]
    pub const fn port(&self) -> Option<NonZeroU16> {
        match self {
            Self::Hosts { port } => Some(*port),
            Self::HostsNone {} | Self::NotAdvertised(_) => None,
        }
    }

    /// Why a workspace installed is not advertised, when that is so.
    #[must_use]
    pub const fn why_not(&self) -> Option<&Wording> {
        match self {
            Self::NotAdvertised(wording) => Some(wording),
            Self::Hosts { .. } | Self::HostsNone {} => None,
        }
    }
}

impl Advertised {
    /// What this machine advertises, as it goes on the wire.
    #[must_use]
    pub fn of(machine: &str, port: u16, workspace: HostedWorkspace) -> Self {
        Self {
            machine: machine.to_owned(),
            port,
            workspace,
        }
    }

    /// This machine's identity.
    #[must_use]
    pub fn machine(&self) -> &str {
        &self.machine
    }

    /// The port its presence names.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// What it says about a workspace.
    #[must_use]
    pub const fn workspace(&self) -> &HostedWorkspace {
        &self.workspace
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use crate::words;
    use alo_strings::Filling;

    /// The studio's identity.
    const THE_STUDIO: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f0";

    /// **The answer's shape is what the advertisement carries and nothing
    /// else**: identity, the presence's port, and one of the three things said
    /// about a workspace — written exactly so, and read back.
    #[test]
    fn the_answer_carries_the_identity_the_port_and_the_workspace_and_nothing_else() {
        let hosting = Advertised::of(
            THE_STUDIO,
            7_610,
            HostedWorkspace::hosts(NonZeroU16::new(8_443).unwrap()),
        );
        let written = serde_json::to_string(&hosting).unwrap();
        assert_eq!(
            written,
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts":{"port":8443}}}"#
        );
        assert_eq!(
            serde_json::from_str::<Advertised>(&written).unwrap(),
            hosting
        );
        assert_eq!(hosting.workspace().port(), NonZeroU16::new(8_443));
        assert_eq!(hosting.workspace().why_not(), None);

        let nothing = Advertised::of(THE_STUDIO, 7_610, HostedWorkspace::none());
        assert_eq!(
            serde_json::to_string(&nothing).unwrap(),
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts-none":{}}}"#
        );

        let said = in_english().say(&words::NOT_READABLE.key(), &Filling::nothing());
        let refused = Advertised::of(THE_STUDIO, 7_610, HostedWorkspace::not_advertised(&said));
        let written = serde_json::to_string(&refused).unwrap();
        assert_eq!(
            serde_json::from_str::<Advertised>(&written).unwrap(),
            refused
        );
        assert_eq!(refused.workspace().port(), None);
        assert_eq!(refused.workspace().why_not().unwrap().text(), said.text());
    }

    /// **A field the advertisement does not carry has nowhere to be**: an
    /// address, a hostname, a name, a model list, a path — beside the answer or
    /// beside the workspace — and a workspace port of zero, are each refused
    /// rather than read around.
    #[test]
    fn an_answer_carrying_anything_the_advertisement_does_not_is_not_read() {
        for written in [
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts-none":{}},"address":"192.168.1.20"}"#,
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts-none":{}},"hostname":"studio"}"#,
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts-none":{}},"models":[]}"#,
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts":{"port":8443,"path":"/etc/alo/workspace.toml"}}}"#,
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts":{"port":0}}}"#,
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610,"workspace":{"hosts-none":{"called":"Axon mail"}}}"#,
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","port":7610}"#,
        ] {
            assert!(
                serde_json::from_str::<Advertised>(written).is_err(),
                "{written}"
            );
        }
    }
}
