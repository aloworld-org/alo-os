//! A workspace on the local network: what its host says about it, and what
//! finding one is — which is a fact written down and nothing more.
//!
//! *A self-hosted workspace on the network is discovered, not configured — no
//! DNS step.* ADR 0003 names it as the second thing discovery is for. The
//! workspace itself — mail, files, chat and documents — is `alo-workplace`'s,
//! and it is outside this repository; what this file owes that repository is
//! the service it advertises under and the closed list it may advertise.
//!
//! # The list, and why it is closed
//!
//! [`WorkspacePresence`] holds an identity and a port, and an advertisement
//! carries those and the version of this protocol the host speaks — **which
//! workspace, where it answers, and the version**, and nothing else. There is
//! no field for the organisation, the people with accounts on it, a URL, a
//! login page, a certificate or what it is called, and reading refuses an
//! advertisement that carries one rather than reading around it
//! (`crate::reading`), for task 1's reason: the failure being guarded against
//! is not only a stranger's packet, it is a workspace two years from now
//! advertising the organisation's name in a key a reader would have skipped.
//!
//! **Which workspace is the identity of the machine that serves it**, spelt as
//! a [`MachineId`] is — thirty-two hexadecimal characters nobody chose. On an
//! alo machine it *is* that machine's identity, so a workspace hosted by a
//! machine this one is paired with is spoken of by the name the person gave
//! that machine; a host that is not an alo machine keeps an identity of its own
//! in the same shape, and is simply not paired with anything. One host serves
//! one workspace, which is what DNS-SD's one instance per name says anyway.
//!
//! # What is found is no more than that
//!
//! [`FoundWorkspace`] is one workspace heard, at the address its answer came
//! from, [`Standing::NotPaired`]. **Finding a workspace confers nothing**: no
//! request, verb or question reaches one because it was found, and nothing in
//! this crate connects to one. It has no public constructor, so the only way to
//! hold one is to have heard an advertisement — an address somebody typed has
//! no road into this type:
//!
//! ```compile_fail
//! use alo_nearby::{FoundWorkspace, MachineId};
//! // There is no way to make a found workspace out of an address.
//! let typed: std::net::SocketAddr = "192.168.1.20:443".parse().unwrap();
//! let _ = FoundWorkspace::heard(MachineId::made().unwrap(), typed.port(), typed.ip());
//! ```

use std::net::{IpAddr, SocketAddr};

use crate::machine::MachineId;
use crate::presence::{Standing, VERSION};

/// The service a workspace answers to on a local network.
///
/// `_alo-workspace._tcp.local` in DNS-SD's spelling: an application protocol
/// named `alo-workspace`, over TCP, on the link-local name space. Its own
/// service rather than a key on [`SERVICE`](crate::SERVICE)'s, so that a machine advertises
/// exactly the same thing whether or not it serves a workspace, and so that a
/// workspace served by something that is not an alo machine is not pretending
/// to be one.
pub const WORKSPACE_SERVICE: &str = "_alo-workspace._tcp.local";

/// What a workspace's host says about it on a local network.
///
/// Made from an identity and a port, and nothing else is reachable from here —
/// see this module's own documentation for why the list is closed in the type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePresence {
    /// Which workspace, by the identity of the machine serving it.
    host: MachineId,
    /// Where it answers.
    port: u16,
}

impl WorkspacePresence {
    /// The workspace served by `host`, at a port.
    #[must_use]
    pub const fn of(host: MachineId, port: u16) -> Self {
        Self { host, port }
    }

    /// Which workspace, by the identity of the machine serving it.
    #[must_use]
    pub const fn host(&self) -> &MachineId {
        &self.host
    }

    /// The port it advertises.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// The name it answers to in the service, which is the identity and the
    /// service's own name — never a name a person or an organisation chose.
    #[must_use]
    pub fn instance(&self) -> String {
        format!("{}.{WORKSPACE_SERVICE}", self.host)
    }

    /// The host name it answers to, which is again the identity.
    #[must_use]
    pub fn target(&self) -> String {
        format!("{}.local", self.host)
    }
}

/// One workspace, heard on the local network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundWorkspace {
    /// Which workspace, by the identity of the machine that said it serves it.
    ///
    /// Said, not proven: anything on a network can advertise under any
    /// identity. What this is good for is saying which one; what a person or a
    /// pairing makes of it is decided elsewhere, and a name beside it is only
    /// ever given where the machine with that identity answered from the same
    /// address at the same moment (`alo-agentd`'s listing).
    host: MachineId,
    /// The port it advertised.
    port: u16,
    /// The address its answer came from — measured, never advertised, for
    /// [`crate::Found::address`]'s reason.
    address: IpAddr,
    /// Where else the same workspace answered from in the same look, one
    /// address for each further network it was heard on alone
    /// ([`crate::Around::heard_on_each`]).
    also_at: Vec<IpAddr>,
    /// The version it speaks, which is the one version this crate reads.
    version: &'static str,
    /// What finding it means for this machine, which is nothing.
    standing: Standing,
}

impl FoundWorkspace {
    /// A workspace heard from `address`, which is not paired with anything,
    /// because hearing it pairs nothing.
    ///
    /// Crate-private: see this module's documentation. Only reading an
    /// advertisement makes one.
    pub(crate) const fn heard(host: MachineId, port: u16, address: IpAddr) -> Self {
        Self {
            host,
            port,
            address,
            also_at: Vec::new(),
            version: VERSION,
            standing: Standing::NotPaired,
        }
    }

    /// Which workspace, by the identity of the machine that said it serves it.
    #[must_use]
    pub const fn host(&self) -> &MachineId {
        &self.host
    }

    /// The port it advertised.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// The address its answer came from.
    #[must_use]
    pub const fn address(&self) -> IpAddr {
        self.address
    }

    /// Every address it answered from in the look it was found in: the one
    /// heard first, then one for each further network it was heard on alone.
    pub fn addresses(&self) -> impl Iterator<Item = IpAddr> + '_ {
        std::iter::once(self.address).chain(self.also_at.iter().copied())
    }

    /// The same workspace, also heard from `address` on another network.
    pub(crate) fn also_heard_at(&mut self, address: IpAddr) {
        if self.addresses().all(|already| already != address) {
            self.also_at.push(address);
        }
    }

    /// The version of this protocol it speaks.
    #[must_use]
    pub const fn version(&self) -> &'static str {
        self.version
    }

    /// What finding it means for this machine, which is nothing.
    #[must_use]
    pub const fn standing(&self) -> Standing {
        self.standing
    }

    /// Where it answers: the address it was heard from and the port it
    /// advertised.
    ///
    /// Written down for the person to act on, and dialled by nothing in this
    /// crate or because it was found.
    #[must_use]
    pub const fn where_it_answers(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{FoundWorkspace, WORKSPACE_SERVICE, WorkspacePresence};
    use crate::machine::MachineId;
    use crate::presence::{SERVICE, Standing, VERSION};

    /// The two services are two, and neither is a suffix of the other, so an
    /// answer about one is never read as the other.
    #[test]
    fn a_workspace_has_a_service_of_its_own() {
        assert_eq!(WORKSPACE_SERVICE, "_alo-workspace._tcp.local");
        assert!(!WORKSPACE_SERVICE.ends_with(SERVICE));
        assert!(!SERVICE.ends_with(WORKSPACE_SERVICE));
    }

    /// The identity a test workspace is served under.
    fn a_host() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// The name a workspace answers to is the identity and the service — not
    /// the organisation's name, which is what DNS-SD would ordinarily put there.
    #[test]
    fn a_workspace_is_named_on_the_network_by_its_identity_and_the_service() {
        let presence = WorkspacePresence::of(a_host(), 8_443);
        assert_eq!(
            presence.instance(),
            format!("0f1e2d3c4b5a69788796a5b4c3d2e1f0.{WORKSPACE_SERVICE}")
        );
        assert_eq!(presence.target(), "0f1e2d3c4b5a69788796a5b4c3d2e1f0.local");
    }

    /// **A workspace found is not paired with**, speaks the one version, and
    /// answers where it was heard from.
    #[test]
    fn a_workspace_that_has_been_found_is_not_paired_with() {
        let found =
            FoundWorkspace::heard(a_host(), 8_443, std::net::Ipv4Addr::new(10, 0, 0, 7).into());
        assert_eq!(found.standing(), Standing::NotPaired);
        assert_eq!(found.version(), VERSION);
        assert_eq!(found.where_it_answers().to_string(), "10.0.0.7:8443");
    }
}
