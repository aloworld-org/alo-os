//! An alo machine that hosts a workspace answers for it, under its own
//! identity, and says nothing more about itself for doing so ([ADR 0003]:
//! discovery reveals presence only).
//!
//! | The acceptance | The test |
//! |---|---|
//! | the question for workspaces is answered with exactly the closed advertisement when a workspace is hosted | [`a_machine_hosting_a_workspace_is_heard_as_one_under_its_own_identity`] |
//! | and stepped over when none is | [`a_machine_hosting_no_workspace_is_heard_as_a_machine_and_nothing_more`] |
//! | the machine's own presence answer is unchanged byte for byte either way | [`what_a_machine_says_about_itself_is_the_same_bytes_whether_or_not_it_hosts`] |
//! | no constructor lets another identity reach the responder | [`no_identity_but_the_machines_own_reaches_a_workspace_answer`], beside the `compile_fail` example in `src/answering.rs` |
//!
//! # What no test here shows
//!
//! Both sides are on this host, over ordinary datagrams. Whether a second
//! physical machine on an office network hears the workspace an alo machine
//! hosts is owed to two machines and `alo-workplace`'s server, and is not
//! claimed here.
//!
//! [ADR 0003]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0003-the-network-is-not-authority.md

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::num::NonZeroU16;
use std::time::Duration;

use alo_nearby::{
    Answering, Around, Looking, MachineId, Presence, Standing, WorkspacePresence, advertising,
};

/// A socket bound to this machine and nowhere else.
fn a_socket() -> UdpSocket {
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    socket
}

/// `answering` on a thread, answering the two questions one look asks; and
/// what a machine looking at it heard.
fn looked_at(answering: Answering, at: SocketAddr) -> Around {
    let answered = std::thread::spawn(move || {
        (0..2)
            .map(|_| answering.answer_one().unwrap())
            .collect::<Vec<_>>()
    });
    let looking = Looking::from(a_socket());
    looking.ask(at).unwrap();
    looking.ask_for_workspaces(at).unwrap();
    let around = looking.around(Duration::from_millis(800)).unwrap();
    answered.join().unwrap();
    around
}

/// **A machine hosting a workspace is heard as one machine and one workspace,
/// both under its own identity, from the one address**, and the workspace is
/// exactly the closed list — which, where, and the version — paired with
/// nothing.
#[test]
fn a_machine_hosting_a_workspace_is_heard_as_one_under_its_own_identity() {
    let machine = MachineId::made().unwrap();
    let socket = a_socket();
    let at = socket.local_addr().unwrap();
    let answering = Answering::on(socket, Presence::of(machine.clone(), 7_610))
        .hosting_a_workspace_at(NonZeroU16::new(8_443).unwrap());

    let around = looked_at(answering, at);

    assert_eq!(around.machines.len(), 1, "{around:?}");
    assert_eq!(around.workspaces.len(), 1, "{around:?}");
    let workspace = around.workspaces.first().unwrap();
    assert_eq!(workspace.host(), &machine);
    assert_eq!(
        workspace.where_it_answers(),
        SocketAddr::new(at.ip(), 8_443)
    );
    assert_eq!(workspace.version(), alo_nearby::VERSION);
    assert_eq!(workspace.standing(), Standing::NotPaired);
    assert_eq!(
        around.machines.first().unwrap().address,
        workspace.address()
    );
}

/// **A machine hosting no workspace is heard as a machine and nothing more**:
/// the question for workspaces is stepped over.
#[test]
fn a_machine_hosting_no_workspace_is_heard_as_a_machine_and_nothing_more() {
    let machine = MachineId::made().unwrap();
    let socket = a_socket();
    let at = socket.local_addr().unwrap();
    let answering = Answering::on(socket, Presence::of(machine.clone(), 7_610));

    let around = looked_at(answering, at);

    assert_eq!(around.machines.len(), 1, "{around:?}");
    assert_eq!(around.machines.first().unwrap().machine, machine);
    assert!(around.workspaces.is_empty(), "{around:?}");
}

/// **What a machine says about itself is the same bytes whether or not it
/// hosts a workspace** — read off the wire, not off the builder.
#[test]
fn what_a_machine_says_about_itself_is_the_same_bytes_whether_or_not_it_hosts() {
    let presence = Presence::of(MachineId::made().unwrap(), 7_610);
    let mut heard = Vec::new();
    for hosting in [false, true] {
        let socket = a_socket();
        let at = socket.local_addr().unwrap();
        let mut answering = Answering::on(socket, presence.clone());
        if hosting {
            answering = answering.hosting_a_workspace_at(NonZeroU16::new(8_443).unwrap());
        }
        let asking = a_socket();
        asking
            .send_to(&advertising::a_question().unwrap(), at)
            .unwrap();
        assert!(answering.answer_one().unwrap().is_some());
        let mut packet = [0_u8; 1_500];
        let (how_many, _) = asking.recv_from(&mut packet).unwrap();
        heard.push(packet.get(..how_many).unwrap().to_vec());
        asking
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        assert!(
            asking.recv_from(&mut packet).is_err(),
            "a question for machines was answered with something more"
        );
    }
    assert_eq!(heard.first(), heard.get(1));
    assert_eq!(
        heard.first().unwrap(),
        &advertising::about(&presence).unwrap()
    );
}

/// **No identity but the machine's own reaches a workspace answer.** Hosting
/// takes a port (the `compile_fail` example in `src/answering.rs` holds that a
/// `WorkspacePresence` cannot be handed in); in the crate's shipped code a
/// `WorkspacePresence` is made in the responder in exactly one place, from its
/// own presence — read off the source, as `FoundWorkspace` is — and what goes
/// out names that identity whatever else exists.
#[test]
fn no_identity_but_the_machines_own_reaches_a_workspace_answer() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/answering.rs"),
    )
    .unwrap();
    let ships = source.split_once("#[cfg(test)]").unwrap().0;
    let making: Vec<&str> = ships
        .lines()
        .filter(|line| {
            line.contains("WorkspacePresence::of(") && !line.trim_start().starts_with("//")
        })
        .collect();
    assert_eq!(making.len(), 1, "{making:?}");
    assert!(
        making
            .iter()
            .all(|line| line.contains("self.presence.machine()")),
        "a workspace is answered for under an identity that is not this machine's: {making:?}"
    );
    for signature in ships
        .lines()
        .filter(|line| line.contains("pub fn") || line.contains("pub const fn"))
    {
        // What it takes, not what it gives back: a responder shows the
        // workspace it answers for, and is handed no identity beside its own
        // presence.
        let takes = signature
            .split_once("->")
            .map_or(signature, |(takes, _)| takes);
        assert!(
            !takes.contains("WorkspacePresence") && !takes.contains("MachineId"),
            "the responder takes an identity: {signature}"
        );
    }

    let machine = MachineId::made().unwrap();
    let somebody_else = MachineId::made().unwrap();
    let socket = a_socket();
    let at = socket.local_addr().unwrap();
    let answering = Answering::on(socket, Presence::of(machine.clone(), 7_610))
        .hosting_a_workspace_at(NonZeroU16::new(8_443).unwrap());
    assert_eq!(
        answering.workspace(),
        Some(&WorkspacePresence::of(machine.clone(), 8_443))
    );
    let around = looked_at(answering, at);
    assert!(
        around
            .workspaces
            .iter()
            .all(|found| *found.host() == machine)
    );
    assert!(
        !around
            .workspaces
            .iter()
            .any(|found| *found.host() == somebody_else)
    );
}
