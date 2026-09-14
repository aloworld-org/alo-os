//! What a person's shell is told about the workspaces on the local network.
//!
//! One request on the person's door — `workspaces`, carrying nothing — and
//! this is the shape of each workspace its answer lists. *A self-hosted
//! workspace on the network is discovered, not configured*: a shell draws these
//! so that a person joining an office is shown the workspace rather than told
//! an address to type.
//!
//! # What is here is the closed list, the address measured, and a name
//!
//! `machine` is which workspace — the identity of the machine that said it
//! serves it, thirty-two characters nobody chose. `answers_at` is where it
//! answers: **the address discovery measured** off the answer and the port it
//! advertised, never an address anybody typed. `speaks` is the version of the
//! protocol it advertised. `called` is the name the person here gave the
//! paired machine hosting it (`name-machine`) — only where that machine is
//! paired with this one now and answered from that same address in the same
//! look, so an advertisement claiming a named machine's identity from
//! somewhere else is never shown under that name; absent otherwise.
//!
//! # And what is not
//!
//! **No standing.** Finding a workspace confers nothing (ADR 0003): there is no
//! field saying it is trusted, reachable, signed in to or paired, because none
//! of those is true of something found. **No address goes the other way**:
//! the person's door has no request that names an address to be dialled as a
//! workspace, so an address typed into a shell has nowhere on this wire to go.
//!
//! # Opening one is by identity, answered in this same shape
//!
//! `open-workspace` names a workspace by `machine` alone, and its answer,
//! `workspace-opened`, is one of these: the address in it is the one that
//! workspace answered from **when the daemon looked on receiving the request**,
//! never one kept from an earlier list. The daemon hands it to the person's
//! session and dials nothing.

use serde::{Deserialize, Serialize};

/// One workspace found on the local network, as a shell draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoundWorkspace {
    /// Which workspace, by the identity of the machine serving it.
    machine: String,
    /// Where it answers: the address discovery measured and the port it
    /// advertised, as `address:port`.
    answers_at: String,
    /// The version of the protocol it advertised.
    speaks: String,
    /// What the person here called the paired machine hosting it, where they
    /// gave it a name and it answered from the same address. Absent — never
    /// empty — otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    called: Option<String>,
}

impl FoundWorkspace {
    /// A workspace found, as it goes on the wire.
    #[must_use]
    pub fn of(machine: &str, answers_at: std::net::SocketAddr, speaks: &str) -> Self {
        Self {
            machine: machine.to_owned(),
            answers_at: answers_at.to_string(),
            speaks: speaks.to_owned(),
            called: None,
        }
    }

    /// The same workspace, carrying the name of the paired machine hosting it.
    #[must_use]
    pub fn hosted_by(mut self, called: Option<&str>) -> Self {
        self.called = called.map(ToOwned::to_owned);
        self
    }

    /// Which workspace, by the identity of the machine serving it.
    #[must_use]
    pub fn machine(&self) -> &str {
        &self.machine
    }

    /// Where it answers, as `address:port`.
    #[must_use]
    pub fn answers_at(&self) -> &str {
        &self.answers_at
    }

    /// The version of the protocol it advertised.
    #[must_use]
    pub fn speaks(&self) -> &str {
        &self.speaks
    }

    /// What the person here called the paired machine hosting it, if anything.
    #[must_use]
    pub fn called(&self) -> Option<&str> {
        self.called.as_deref()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A workspace is drawn with which one, where it answers and the version**,
    /// and the name of its host beside them only when there is one.
    #[test]
    fn a_workspace_found_carries_the_closed_list_and_reads_back() {
        let at: std::net::SocketAddr = "192.168.1.20:8443".parse().unwrap();
        let found = FoundWorkspace::of("0f1e2d3c4b5a69788796a5b4c3d2e1f0", at, "1");
        let written = serde_json::to_string(&found).unwrap();
        assert_eq!(
            written,
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","answers_at":"192.168.1.20:8443","speaks":"1"}"#
        );
        assert_eq!(
            serde_json::from_str::<FoundWorkspace>(&written).unwrap(),
            found
        );

        let named = found.hosted_by(Some("the studio machine"));
        assert_eq!(named.called(), Some("the studio machine"));
        let written = serde_json::to_string(&named).unwrap();
        assert!(
            written.contains(r#""called":"the studio machine""#),
            "{written}"
        );
        assert_eq!(
            serde_json::from_str::<FoundWorkspace>(&written).unwrap(),
            named
        );
    }

    /// **Nothing a found workspace is drawn with says it is trusted or signed
    /// in to**, and a field nobody declared is refused rather than read around.
    #[test]
    fn a_workspace_found_carries_no_standing_and_refuses_a_field_nobody_declared() {
        for message in [
            r#"{"machine":"m","answers_at":"10.0.0.1:443","speaks":"1","trusted":true}"#,
            r#"{"machine":"m","answers_at":"10.0.0.1:443","speaks":"1","signed_in":true}"#,
            r#"{"machine":"m","answers_at":"10.0.0.1:443","speaks":"1","organisation":"axon"}"#,
        ] {
            assert!(
                serde_json::from_str::<FoundWorkspace>(message).is_err(),
                "{message}"
            );
        }
    }
}
