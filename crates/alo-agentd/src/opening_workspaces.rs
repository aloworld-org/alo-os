//! The person's door opens a workspace discovery found, at the address it
//! answers from at that moment, and connects to nothing.
//!
//! `crate::listing_workspaces` shows a person which workspaces are on the
//! network, and deliberately nothing more. This is the person's next act —
//! `alo_protocol::FromAPerson::OpenWorkspace`, answered here — and what it
//! carries from the list to the workspace client in their session is **the
//! identity alone**. A shell holding an address from a list that has since aged
//! would be holding a typed address by another route, so the address the
//! answer carries is measured again when the request arrives.
//!
//! # In this order, and every refusal contacts nothing and writes nothing
//!
//! 1. **An identity**, or [`THAT_IS_NOT_A_WORKSPACE`]. A workspace is opened
//!    *by* the identity it was found by: a web address, a hostname or a name
//!    typed where the identity goes is refused before the link is even asked.
//! 2. **The link, asked at the moment** ([`crate::looking::LookingFor::look_around`]),
//!    before any lock is taken, for `crate::looking`'s reason.
//! 3. **Exactly one place answering as that workspace.** None is
//!    [`NO_SUCH_WORKSPACE_ON_THE_NETWORK`]. More than one address is
//!    [`A_WORKSPACE_ANSWERED_FROM_MORE_THAN_ONE_PLACE`]: an advertisement is not
//!    proven, anything on the network can claim any identity, and two claims
//!    nobody can tell apart are not settled by preferring whichever answered
//!    first.
//! 4. **Drawn by the listing's own rule** ([`crate::listing_workspaces::drawn`]),
//!    under the network's lock and then the names': the host's name beside it
//!    only where that machine is paired now and answered from the same address.
//! 5. **Written down, then handed over.** The record gains *a workspace opened
//!    by the person*, naming the identity and the address, **before** the
//!    answer leaves — an address is never handed anywhere the record does not
//!    say, and a record that cannot be written stops the service with nothing
//!    handed.
//!
//! # The daemon dials nothing, and nothing here is a sign-in
//!
//! The answer is the address, for the person's session to hand to the workspace
//! client — `alo-workplace`'s, under the person's own account. Nothing in this
//! file opens a socket to it, proposes a pairing, touches a grant, or lets an
//! agent reach it (ADR 0003); a pairing is not a sign-in and none is consulted
//! for one. Doing something *with* a workspace on another alo machine on this
//! machine's or its agents' behalf is a different act, under a pairing that
//! permits `MayAskIts::Workspace`, and is not this request.
//!
//! An agent sending it is refused by `alo-protocol` before anything reaches this
//! file, in the words an agent approving something gets.

use std::time::SystemTime;

use alo_keeping::NotKept;
use alo_nearby::MachineId;
use alo_protocol::ToAPerson;
use alo_strings::{Filling, Strings};

use crate::holding::Holding;
use crate::listing_workspaces;
use crate::pairing::Nearby;
use crate::words::{
    A_WORKSPACE_ANSWERED_FROM_MORE_THAN_ONE_PLACE, NO_SUCH_WORKSPACE_ON_THE_NETWORK,
    THAT_IS_NOT_A_WORKSPACE, Word,
};

/// Open the workspace the person named by its identity: the address it answers
/// from now, written down and handed back — or one sentence saying why nothing
/// was opened.
///
/// # Errors
///
/// [`NotKept`] when the opening could not be written down. The service stops
/// on it, and the address is not handed to the person's session: a workspace
/// opened with no record of it is exactly what *nothing leaves silently* is
/// there to prevent.
pub fn opened(
    machine: &str,
    nearby: &Nearby<'_>,
    holding: &mut Holding<'_, '_, '_>,
    strings: &Strings,
    now: SystemTime,
) -> Result<ToAPerson, NotKept> {
    let say = |word: Word| ToAPerson::refused(&strings.say(&word.key(), &Filling::nothing()));

    // 1. An identity, before the link is asked.
    let Ok(identity) = MachineId::read(machine.trim()) else {
        return Ok(say(THAT_IS_NOT_A_WORKSPACE));
    };

    // 2. The link, at the moment.
    let around = nearby.looking.look_around();

    // 3. Exactly one place answering as that workspace.
    let mut answering = around
        .workspaces
        .iter()
        .filter(|workspace| *workspace.host() == identity);
    let Some(one) = answering.next() else {
        return Ok(say(NO_SUCH_WORKSPACE_ON_THE_NETWORK));
    };
    if answering.any(|another| another.where_it_answers() != one.where_it_answers()) {
        return Ok(say(A_WORKSPACE_ANSWERED_FROM_MORE_THAN_ONE_PLACE));
    }

    // 4. Drawn by the listing's rule, and the lock let go before the record.
    let drawn = {
        let shared = nearby.network.locked();
        listing_workspaces::drawn(one, &around, &shared, nearby.network, now)
    };

    // 5. Written down, then handed over.
    holding.a_workspace_was_opened(identity.as_str(), drawn.answers_at(), now)?;
    Ok(ToAPerson::workspace_opened(drawn))
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
    use alo_nearby::{MayAskIts, Presence, WorkspacePresence, advertising};
    use alo_protocol::FromAPerson;
    use alo_record::{Happened, Record};
    use alo_remembering::MachineName;

    use super::*;
    use crate::answering::what_a_person_said;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::looking::LookingFor;
    use crate::network::TheNetwork;
    use crate::rereading::WhatIsGranted;
    use crate::testing::{
        NothingIsRemembered, a_message, hour, in_english, noon, nothing_has_been_chosen,
        on_a_machine_that_answers, on_a_machine_with_no_turn, paired_between, reception,
        the_studio,
    };

    /// A workspace nobody here has heard of before.
    fn the_warehouse() -> MachineId {
        MachineId::read("11112222333344445555666677778888").unwrap()
    }

    /// What the person's shell sends to open `machine`.
    fn opening(machine: &str) -> String {
        format!(
            r#"{{"open-workspace":{{"machine":{}}}}}"#,
            serde_json::to_string(machine).unwrap()
        )
    }

    /// The link as the daemon really asks it — [`crate::looking::around_at`] —
    /// pointed at a socket a test holds, counting how often it was asked.
    #[derive(Debug)]
    struct TheLinkAt {
        at: SocketAddr,
        looked: Cell<u32>,
    }

    impl TheLinkAt {
        fn of(at: SocketAddr) -> Self {
            Self {
                at,
                looked: Cell::new(0),
            }
        }
    }

    impl LookingFor for TheLinkAt {
        fn look_for(&self, _machine: &MachineId) -> Option<alo_nearby::Found> {
            None
        }

        fn look_around(&self) -> alo_nearby::Around {
            self.looked.set(self.looked.get().saturating_add(1));
            crate::looking::around_at(self.at)
        }
    }

    /// Something on the link that answers the first question it hears with
    /// every one of `packets`, then hears the second question and says nothing
    /// more — one host answering both the machine and the workspace question.
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

    /// A link nothing on answers.
    fn a_quiet_link() -> UdpSocket {
        UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap()
    }

    /// `host`'s workspace, advertised at `port`.
    fn a_workspace_of(host: MachineId, port: u16) -> Vec<u8> {
        advertising::about_a_workspace(&WorkspacePresence::of(host, port)).unwrap()
    }

    /// Somewhere a workspace client would connect, which this test holds and
    /// can ask afterwards whether anything did.
    fn a_workspace_listening() -> TcpListener {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        listener
    }

    /// Nothing connected to `listener`.
    fn nothing_connected_to(listener: &TcpListener) {
        assert_eq!(
            listener.accept().map(|_| ()).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "something connected to a workspace the daemon only had to name"
        );
    }

    /// The person sends `line`, on a machine with no turn; what was written
    /// down while they did comes back beside the answer.
    fn the_person_says(
        line: &str,
        network: &TheNetwork,
        looking: &dyn LookingFor,
    ) -> (ToAPerson, Record) {
        let mut record = Record::default();
        let said = on_a_machine_with_no_turn(
            "opening-workspaces",
            &mut record,
            |machine, _, strings, _, _| {
                let mut grants = Grants::default();
                what_a_person_said(
                    &a_message(line),
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

    /// The sentence this crate says for `word`.
    fn saying(word: Word) -> String {
        in_english()
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    }

    /// **The person opens a found workspace by its identity, and is answered
    /// with the one address it answered from at that moment** — written down as
    /// *a workspace opened by the person*, naming the identity and the address,
    /// under nobody's authority. The daemon itself connects to nothing: the
    /// listener at that address is never connected to, and nothing is paired.
    #[test]
    fn opening_a_found_workspace_answers_with_the_address_measured_records_it_and_connects_to_nothing()
     {
        let workspace = a_workspace_listening();
        let port = workspace.local_addr().unwrap().port();
        let (at, answering) = the_link_answering_with(vec![a_workspace_of(the_studio(), port)]);
        let network = TheNetwork::on(reception());
        let link = TheLinkAt::of(at);

        let (said, record) = the_person_says(&opening(the_studio().as_str()), &network, &link);
        answering.join().unwrap();

        let opened = said.opened_workspace().unwrap();
        assert_eq!(opened.machine(), the_studio().as_str());
        assert_eq!(opened.answers_at(), format!("127.0.0.1:{port}"));
        assert_eq!(opened.speaks(), "1");
        assert_eq!(opened.called(), None);
        assert_eq!(link.looked.get(), 1, "the link was not asked at the moment");

        assert_eq!(record.len(), 1, "{record:?}");
        let entry = record.everything().next().unwrap();
        assert_eq!(entry.agent(), None);
        assert!(!entry.happened().caused_egress());
        assert!(matches!(
            entry.happened(),
            Happened::WorkspaceOpened { workspace, answers_at }
                if workspace.is(the_studio().as_str())
                    && answers_at.as_str() == format!("127.0.0.1:{port}")
        ));

        nothing_connected_to(&workspace);
        let shared = network.locked();
        assert!(shared.pairings().every().is_empty(), "opening it paired");
        assert!(shared.proposals().every().is_empty(), "opening it proposed");
    }

    /// **The address is the one measured when the request arrives, never one
    /// from an earlier list.** The workspace was listed at one port, then
    /// answered from another; opening it answers with the second.
    #[test]
    fn a_workspace_that_moved_since_it_was_listed_is_opened_where_it_answers_now() {
        let network = TheNetwork::on(reception());

        let (before, answering) =
            the_link_answering_with(vec![a_workspace_of(the_studio(), 8_443)]);
        let (listed, _) = the_person_says(r#"{"workspaces":{}}"#, &network, &TheLinkAt::of(before));
        answering.join().unwrap();
        assert_eq!(
            listed
                .workspaces_found()
                .unwrap()
                .first()
                .unwrap()
                .answers_at(),
            "127.0.0.1:8443"
        );

        let (now, answering) = the_link_answering_with(vec![a_workspace_of(the_studio(), 9_443)]);
        let (said, record) = the_person_says(
            &opening(the_studio().as_str()),
            &network,
            &TheLinkAt::of(now),
        );
        answering.join().unwrap();

        assert_eq!(
            said.opened_workspace().unwrap().answers_at(),
            "127.0.0.1:9443"
        );
        assert!(matches!(
            record.everything().next().unwrap().happened(),
            Happened::WorkspaceOpened { answers_at, .. } if answers_at.is("127.0.0.1:9443")
        ));
    }

    /// **What is not an identity is refused in words, and contacts nothing**:
    /// the link is not even asked, nothing is written down, and a web address,
    /// an address and a port, a hostname or a name are all the same refusal.
    #[test]
    fn opening_something_that_is_not_an_identity_is_refused_and_contacts_nothing() {
        let quiet = a_quiet_link();
        let link = TheLinkAt::of(quiet.local_addr().unwrap());
        let network = TheNetwork::on(reception());

        for named in [
            "192.168.1.20:8443",
            "https://mail.axon.example",
            "mail.axon.local",
            "the studio machine",
            "",
            "0f1e2d3c4b5a69788796a5b4c3d2e1f",
        ] {
            let (said, record) = the_person_says(&opening(named), &network, &link);
            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert_eq!(refusal.text(), saying(THAT_IS_NOT_A_WORKSPACE), "{named}");
            assert!(record.is_empty(), "a refusal was written down for {named}");
        }
        assert_eq!(
            link.looked.get(),
            0,
            "the link was asked about a non-identity"
        );
    }

    /// **When no workspace of that identity answered, nothing is opened and
    /// nothing is contacted** — not even the workspace that did answer, under
    /// another identity.
    #[test]
    fn opening_a_workspace_that_did_not_answer_is_refused_and_contacts_nothing() {
        let another = a_workspace_listening();
        let port = another.local_addr().unwrap().port();
        let (at, answering) = the_link_answering_with(vec![a_workspace_of(the_warehouse(), port)]);
        let network = TheNetwork::on(reception());

        let (said, record) = the_person_says(
            &opening(the_studio().as_str()),
            &network,
            &TheLinkAt::of(at),
        );
        answering.join().unwrap();

        let refusal = said.refusal().unwrap();
        assert!(!refusal.is_a_bug(), "{refusal:?}");
        assert_eq!(refusal.text(), saying(NO_SUCH_WORKSPACE_ON_THE_NETWORK));
        assert!(record.is_empty(), "a refusal was written down");
        nothing_connected_to(&another);
    }

    /// **When more than one address answered for the same identity, none of
    /// them is opened** — a claim nobody can tell apart is not settled by
    /// taking the first — and neither place is contacted.
    #[test]
    fn a_workspace_answering_from_more_than_one_address_is_not_opened_and_neither_is_contacted() {
        let first = a_workspace_listening();
        let second = a_workspace_listening();
        let (at, answering) = the_link_answering_with(vec![
            a_workspace_of(the_studio(), first.local_addr().unwrap().port()),
            a_workspace_of(the_studio(), second.local_addr().unwrap().port()),
        ]);
        let network = TheNetwork::on(reception());

        let (said, record) = the_person_says(
            &opening(the_studio().as_str()),
            &network,
            &TheLinkAt::of(at),
        );
        answering.join().unwrap();

        let refusal = said.refusal().unwrap();
        assert!(!refusal.is_a_bug(), "{refusal:?}");
        assert_eq!(
            refusal.text(),
            saying(A_WORKSPACE_ANSWERED_FROM_MORE_THAN_ONE_PLACE)
        );
        assert!(record.is_empty(), "a refusal was written down");
        nothing_connected_to(&first);
        nothing_connected_to(&second);
    }

    /// **The same answer heard twice from one address is one place**, and is
    /// opened: only two different addresses are a claim nobody can settle.
    #[test]
    fn a_workspace_heard_twice_from_one_address_is_opened() {
        let (at, answering) = the_link_answering_with(vec![
            a_workspace_of(the_studio(), 8_443),
            a_workspace_of(the_studio(), 8_443),
        ]);
        let network = TheNetwork::on(reception());

        let (said, record) = the_person_says(
            &opening(the_studio().as_str()),
            &network,
            &TheLinkAt::of(at),
        );
        answering.join().unwrap();

        assert_eq!(
            said.opened_workspace().unwrap().answers_at(),
            "127.0.0.1:8443"
        );
        assert_eq!(record.len(), 1);
    }

    /// **A request carrying an address is not a request**, on the person's own
    /// door: refused as unreadable, the link not asked, nothing written.
    #[test]
    fn a_request_to_open_a_workspace_carrying_an_address_is_not_a_request() {
        let quiet = a_quiet_link();
        let link = TheLinkAt::of(quiet.local_addr().unwrap());
        let network = TheNetwork::on(reception());

        for line in [
            format!(
                r#"{{"open-workspace":{{"machine":"{}","answers_at":"127.0.0.1:8443"}}}}"#,
                the_studio().as_str()
            ),
            r#"{"open-workspace":{"answers_at":"127.0.0.1:8443"}}"#.to_owned(),
            r#"{"open-workspace":{"url":"https://mail.axon.example"}}"#.to_owned(),
        ] {
            assert!(FromAPerson::read(&a_message(&line)).is_err(), "{line}");
            let (said, record) = the_person_says(&line, &network, &link);
            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert!(record.is_empty(), "{line}");
        }
        assert_eq!(link.looked.get(), 0);
    }

    /// **A workspace hosted by a paired machine the person named is opened
    /// under that name**, by the listing's own rule: the machine answered from
    /// the same address in the same look.
    #[test]
    fn a_workspace_hosted_by_a_named_paired_machine_is_opened_under_its_name() {
        let (at, answering) = the_link_answering_with(vec![
            advertising::about(&Presence::of(the_studio(), 7_610)).unwrap(),
            a_workspace_of(the_studio(), 8_443),
        ]);
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

        let (said, record) = the_person_says(
            &opening(the_studio().as_str()),
            &network,
            &TheLinkAt::of(at),
        );
        answering.join().unwrap();

        let opened = said.opened_workspace().unwrap();
        assert_eq!(opened.called(), Some("the studio machine"));
        assert!(matches!(
            record.everything().next().unwrap().happened(),
            Happened::WorkspaceOpened { workspace, .. } if workspace.is(the_studio().as_str())
        ));
    }

    /// **An agent sending it is refused in the words an agent approving
    /// something gets**, the link is not asked, and nothing is opened.
    #[test]
    fn an_agent_opening_a_workspace_is_refused_as_an_approval_would_be() {
        let network = TheNetwork::on(reception());
        let quiet = a_quiet_link();
        let link = TheLinkAt::of(quiet.local_addr().unwrap());
        let corridor = Corridor {
            network: &network,
            looking: &link,
            naming: network.names(),
        };
        let mut questions = nothing_has_been_chosen();
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, grants, strings| {
            let said = what_an_agent_said(
                &a_message(&opening(the_studio().as_str())),
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
        assert_eq!(link.looked.get(), 0);
        assert!(
            !record
                .everything()
                .any(|entry| matches!(entry.happened(), Happened::WorkspaceOpened { .. })),
            "{record:?}"
        );
    }
}
