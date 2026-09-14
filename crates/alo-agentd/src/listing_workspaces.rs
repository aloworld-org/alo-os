//! The person's door lists the workspaces on the network, and reaches none of
//! them.
//!
//! *A self-hosted workspace on the network is discovered, not configured — no
//! DNS step.* This is the request a shell asks it with —
//! `alo_protocol::FromAPerson::Workspaces`, answered here — so that a person
//! joining an office is shown the workspace rather than told an address to
//! type.
//!
//! # In this order
//!
//! 1. **The link is asked, at the moment, before any lock is taken**
//!    ([`crate::looking::LookingFor::look_around`]): who is here and which
//!    workspaces, in one window. Nothing is kept between two asks, for
//!    `crate::looking`'s reason — a list kept would age.
//! 2. **Each workspace is written as it was heard**: which one, where discovery
//!    measured it answers, and the version it speaks.
//! 3. **A name beside it only where it is true**, under the network's lock and
//!    then the names' (the order `crate::pairing`'s list takes): the name the
//!    person gave the machine whose identity the workspace was advertised
//!    under — only if this machine is paired with that machine now **and** that
//!    machine answered from the same address in the same window. An
//!    advertisement is not proven, and anything on the network can claim a
//!    named machine's identity; a claim made from somewhere that machine did
//!    not answer is shown by its identity, never under the name.
//!
//! # Finding a workspace confers nothing
//!
//! Nothing here connects to a workspace, dials the address it was heard from,
//! proposes a pairing, writes the record, or touches a grant or a setting —
//! the answer is a list and nothing on the machine moves because of it (ADR
//! 0003). Reaching a workspace found this way is the person's own act, and no
//! request on either door does it. There is no field on the request for an
//! address either, so an address typed into a shell has nowhere here to go.
//!
//! An agent asking is refused by `alo-protocol` before anything reaches this
//! file, in the words an agent approving something gets.

use std::time::SystemTime;

use alo_corridor::Naming as _;
use alo_protocol::{FoundWorkspace, ToAPerson};

use crate::pairing::Nearby;

/// Every workspace discovery finds on the network at the moment, each with the
/// name of the paired machine hosting it where the person gave one.
#[must_use]
pub fn listed(nearby: &Nearby<'_>, now: SystemTime) -> ToAPerson {
    // 1. The link, asked before the lock: it waits on the network, and nothing
    //    on this machine changes while it does.
    let around = nearby.looking.look_around();

    // 2 and 3. As heard, with a name only where it is true.
    let shared = nearby.network.locked();
    let found = around
        .workspaces
        .iter()
        .map(|workspace| {
            let host = workspace.host();
            let answered_there = around
                .machines
                .iter()
                .any(|machine| machine.machine == *host && machine.address == workspace.address());
            let called = (answered_there && shared.pairings().paired_with(host, now))
                .then(|| nearby.network.names().called(host))
                .flatten();
            FoundWorkspace::of(
                host.as_str(),
                workspace.where_it_answers(),
                workspace.version(),
            )
            .hosted_by(called.as_deref())
        })
        .collect();
    ToAPerson::workspaces(found)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::Cell;
    use std::net::{Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
    use std::time::Duration;

    use alo_capability::Grants;
    use alo_nearby::{MachineId, MayAskIts, Presence, WorkspacePresence, advertising};
    use alo_protocol::FromAPerson;
    use alo_record::Record;
    use alo_remembering::MachineName;

    use super::*;
    use crate::answering::what_a_person_said;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::holding::Holding;
    use crate::looking::LookingFor;
    use crate::network::TheNetwork;
    use crate::rereading::WhatIsGranted;
    use crate::testing::{
        NothingIsRemembered, TheStudioIsAt, a_message, hour, noon, nothing_has_been_chosen,
        on_a_machine_that_answers, on_a_machine_with_no_turn, paired_between, reception,
        the_studio,
    };

    /// What the person's shell sends to ask.
    const ASKING: &str = r#"{"workspaces":{}}"#;

    /// The link as the daemon really asks it — [`crate::looking::around_at`],
    /// the function the shipped `Wire` calls — pointed at a socket a test holds
    /// rather than at the multicast group.
    #[derive(Debug)]
    struct TheLinkAt(SocketAddr);

    impl LookingFor for TheLinkAt {
        fn look_for(&self, _machine: &MachineId) -> Option<alo_nearby::Found> {
            None
        }

        fn look_around(&self) -> alo_nearby::Around {
            crate::looking::around_at(self.0)
        }
    }

    /// Something on the link that answers the first question it hears with
    /// every one of `packets`, and then hears the second question and says
    /// nothing more — which is what one host answering for a machine and a
    /// workspace looks like from here.
    fn the_link_answering_with(packets: Vec<Vec<u8>>) -> (SocketAddr, std::thread::JoinHandle<()>) {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let at = socket.local_addr().unwrap();
        let answering = std::thread::spawn(move || {
            let mut heard = [0_u8; 1_500];
            let (_, who) = socket.recv_from(&mut heard).unwrap();
            for packet in &packets {
                socket.send_to(packet, who).unwrap();
            }
            drop(socket.recv_from(&mut heard));
        });
        (at, answering)
    }

    /// The studio's workspace, advertised at `port`.
    fn the_studios_workspace_at(port: u16) -> Vec<u8> {
        advertising::about_a_workspace(&WorkspacePresence::of(the_studio(), port)).unwrap()
    }

    /// The studio saying it exists.
    fn the_studio_saying_it_exists() -> Vec<u8> {
        advertising::about(&Presence::of(the_studio(), 7_610)).unwrap()
    }

    /// Reception, paired with the studio for its models, the studio named.
    fn reception_paired_with_the_named_studio() -> TheNetwork {
        let network = TheNetwork::on(reception());
        let (on_reception, _) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        network.locked().pairings_mut().keep(on_reception);
        network
            .names()
            .named(
                &the_studio(),
                Some(MachineName::checked("the studio machine").unwrap()),
                network.locked().pairings(),
                noon(),
            )
            .unwrap();
        network
    }

    /// The person asks, on a machine with no turn, and what was written down
    /// while they did comes back beside the answer.
    fn the_person_asks(network: &TheNetwork, looking: &dyn LookingFor) -> (ToAPerson, Record) {
        let mut record = Record::default();
        let said = on_a_machine_with_no_turn(
            "listing-workspaces",
            &mut record,
            |machine, _, strings, _, _| {
                let mut grants = Grants::default();
                what_a_person_said(
                    &a_message(ASKING),
                    &mut Holding::Nobody(machine),
                    &mut WhatIsGranted::of(&mut grants, &NothingIsRemembered),
                    &Nearby { network, looking },
                    strings,
                    noon(),
                )
                .unwrap()
            },
        );
        (said, record)
    }

    /// **A workspace advertised on a network where nothing is paired is listed
    /// with where discovery measured it answers — and nothing is contacted.**
    /// Where it answers is a listener this test holds, and nothing connected
    /// to it; nothing was paired or proposed, and nothing was written down.
    #[test]
    fn a_workspace_on_a_network_where_nothing_is_paired_is_listed_and_nothing_is_contacted() {
        let workspace = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        workspace.set_nonblocking(true).unwrap();
        let port = workspace.local_addr().unwrap().port();
        let (at, answering) = the_link_answering_with(vec![the_studios_workspace_at(port)]);
        let network = TheNetwork::on(reception());

        let (said, record) = the_person_asks(&network, &TheLinkAt(at));
        answering.join().unwrap();

        let found = said.workspaces_found().unwrap();
        assert_eq!(found.len(), 1, "{said:?}");
        let one = found.first().unwrap();
        assert_eq!(one.machine(), the_studio().as_str());
        assert_eq!(one.answers_at(), format!("127.0.0.1:{port}"));
        assert_eq!(one.speaks(), "1");
        assert_eq!(one.called(), None);

        assert_eq!(
            workspace.accept().map(|_| ()).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "something connected to a workspace because it was found"
        );
        let shared = network.locked();
        assert!(shared.pairings().every().is_empty(), "finding it paired");
        assert!(shared.proposals().every().is_empty(), "finding it proposed");
        drop(shared);
        assert!(record.is_empty(), "finding a workspace wrote the record");
    }

    /// **A workspace hosted by a paired machine the person named is listed by
    /// that name**, beside its identity — where that machine answered from the
    /// same address in the same look.
    #[test]
    fn a_workspace_hosted_by_a_paired_machine_is_listed_by_the_name_its_person_gave_it() {
        let (at, answering) = the_link_answering_with(vec![
            the_studio_saying_it_exists(),
            the_studios_workspace_at(8_443),
        ]);
        let network = reception_paired_with_the_named_studio();

        let (said, _) = the_person_asks(&network, &TheLinkAt(at));
        answering.join().unwrap();

        let found = said.workspaces_found().unwrap();
        assert_eq!(found.len(), 1, "{said:?}");
        let one = found.first().unwrap();
        assert_eq!(one.machine(), the_studio().as_str());
        assert_eq!(one.answers_at(), "127.0.0.1:8443");
        assert_eq!(one.called(), Some("the studio machine"));
    }

    /// **A workspace claiming a named machine's identity is not shown under
    /// that name when that machine did not answer there** — an advertisement is
    /// not proven, and a name beside a stranger's workspace would be the name
    /// lending it the trust the person has in that machine.
    #[test]
    fn a_workspace_claiming_a_named_machine_is_not_named_when_that_machine_did_not_answer() {
        let (at, answering) = the_link_answering_with(vec![the_studios_workspace_at(8_443)]);
        let network = reception_paired_with_the_named_studio();

        let (said, _) = the_person_asks(&network, &TheLinkAt(at));
        answering.join().unwrap();

        let one = said.workspaces_found().unwrap().first().unwrap().clone();
        assert_eq!(one.machine(), the_studio().as_str());
        assert_eq!(one.called(), None, "a claimed identity borrowed a name");
    }

    /// **A workspace hosted by a machine that answered but is not paired has no
    /// name**, even though a name was once given to it: a name is a paired
    /// machine's, and a revoked pairing's name went with it.
    #[test]
    fn a_workspace_hosted_by_a_machine_not_paired_now_has_no_name() {
        let (at, answering) = the_link_answering_with(vec![
            the_studio_saying_it_exists(),
            the_studios_workspace_at(8_443),
        ]);
        let network = reception_paired_with_the_named_studio();
        assert!(network.locked().pairings_mut().revoke(&the_studio()));

        let (said, _) = the_person_asks(&network, &TheLinkAt(at));
        answering.join().unwrap();

        assert_eq!(
            said.workspaces_found().unwrap().first().unwrap().called(),
            None
        );
    }

    /// **Nothing on the network is a list with nothing on it**, not a refusal:
    /// an office with no workspace is not a machine in trouble.
    #[test]
    fn a_network_with_no_workspace_on_it_answers_with_nothing_found() {
        let quiet = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let network = TheNetwork::on(reception());
        let (said, record) = the_person_asks(&network, &TheLinkAt(quiet.local_addr().unwrap()));
        assert_eq!(said.workspaces_found(), Some([].as_slice()), "{said:?}");
        assert!(record.is_empty());
    }

    /// **An agent asking which workspaces are nearby is refused in the words an
    /// agent approving something gets**, and the network is not even asked.
    #[test]
    fn an_agent_asking_which_workspaces_are_nearby_is_refused_as_an_approval_would_be() {
        let network = reception_paired_with_the_named_studio();
        let looking = TheStudioIsAt {
            at: "127.0.0.1:9".parse().unwrap(),
            looked: Cell::new(0),
        };
        let corridor = Corridor {
            network: &network,
            looking: &looking,
            naming: network.names(),
        };
        let mut questions = nothing_has_been_chosen();
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, grants, strings| {
            let said = what_an_agent_said(
                &a_message(ASKING),
                turning,
                &mut questions,
                Some(&corridor),
                grants,
                strings,
                hour(),
                noon(),
            );
            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert!(
                refusal
                    .text()
                    .contains("an agent cannot answer a question that was put to a person"),
                "{refusal:?}"
            );
        });
        assert_eq!(looking.looked.get(), 0);
    }

    /// The request reads as the person's and as none of the other kinds the
    /// dispatch tells apart.
    #[test]
    fn the_request_is_a_persons_and_about_nothing_else() {
        let asked = FromAPerson::read(&a_message(ASKING)).unwrap();
        assert_eq!(asked, FromAPerson::Workspaces);
        assert!(!asked.is_about_a_pairing());
        assert!(!asked.is_about_a_name());
        assert!(crate::pairing::AboutAPairing::of(&asked).is_none());
    }
}
