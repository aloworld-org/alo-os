//! *Machines find each other with zero configuration — no addresses typed, no
//! accounts*, and [ADR 0003]'s limit on what that may cost: **discovery
//! reveals presence and nothing else.**
//!
//! The crate's own tests take the pieces apart. This one walks the road from
//! outside and holds the four sentences a person is owed:
//!
//! | The acceptance | The test |
//! |---|---|
//! | a machine advertises, and a second one reads it back as one machine, not paired | [`a_machine_on_the_network_is_found_by_another_and_nothing_is_open_to_it`] |
//! | what it advertises carries nothing a person chose and nothing about what it is doing | [`what_a_machine_advertises_is_its_identity_the_service_and_a_port`] |
//! | the identity is stable across a restart, and is not a serial | [`the_identity_survives_a_restart_and_outlives_nothing_else`] |
//! | nothing here connects to anything it finds, and nothing here is configurable | [`discovery_dials_nothing_the_wire_dials_what_it_measured_and_nothing_has_a_setting`] |
//!
//! # What no test here shows
//!
//! Both sides are on this host, over ordinary datagrams. That shows the packets
//! are right and the road is walked. **Whether a second physical machine on an
//! office network hears this one is owed to two machines**, and is not claimed
//! anywhere in this file.
//!
//! [ADR 0003]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0003-the-network-is-not-authority.md

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::net::UdpSocket;
use std::time::Duration;

use alo_nearby::{Answering, Looking, MachineId, Presence, SERVICE, Standing, advertising};

/// A socket bound to this machine and nowhere else.
fn a_socket() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    socket
}

/// **One machine advertises; another finds it; and finding it opened nothing.**
///
/// The last clause is the one ADR 0003 turns on. A machine that has been found
/// is [`Standing::NotPaired`], and there is no other arm to be: being on the
/// same network is not authority, and this is where a type would have said
/// otherwise if anybody had let it.
#[test]
fn a_machine_on_the_network_is_found_by_another_and_nothing_is_open_to_it() {
    let machine = MachineId::made().unwrap();
    let there = a_socket();
    let at = there.local_addr().unwrap();
    let answering = Answering::on(there, Presence::of(machine.clone(), 7_610));
    let answered = std::thread::spawn(move || answering.answer_one());

    let looking = Looking::from(a_socket());
    looking.ask(at).unwrap();
    let found = looking.found(Duration::from_secs(5)).unwrap();

    assert!(answered.join().unwrap().unwrap().is_some());
    assert_eq!(found.len(), 1, "{found:?}");
    let one = found.first().unwrap();
    assert_eq!(one.machine, machine);
    assert_eq!(one.port, 7_610);
    // Where it answers is where it was heard from, measured off the answer
    // rather than advertised, with the port it advertised.
    assert_eq!(
        one.where_it_answers(),
        std::net::SocketAddr::new(at.ip(), 7_610)
    );
    assert_eq!(one.standing, Standing::NotPaired);
}

/// **What is in the packet is the identity, the service and the port** — and
/// this test is written as a list of what must *not* be in it, because that is
/// the failure worth catching.
///
/// Everything on a network can read an advertisement, including a machine
/// nobody has paired with and nobody owns. So the packet is searched for this
/// machine's own name, the name of the person logged into it, and the words a
/// later version would most plausibly start saying.
#[test]
fn what_a_machine_advertises_is_its_identity_the_service_and_a_port() {
    let machine = MachineId::made().unwrap();
    let packet = advertising::about(&Presence::of(machine.clone(), 7_610)).unwrap();
    let written = String::from_utf8_lossy(&packet).to_lowercase();

    // What it does say, and all of what it says.
    assert!(written.contains(machine.as_str()));
    assert!(written.contains("_alo-os"));
    assert!(written.contains("v=1"));

    // What this machine knows about itself and does not tell anybody.
    for known in [
        std::env::var("COMPUTERNAME").unwrap_or_default(),
        std::env::var("HOSTNAME").unwrap_or_default(),
        std::env::var("USERNAME").unwrap_or_default(),
        std::env::var("USER").unwrap_or_default(),
        std::env::var("USERDOMAIN").unwrap_or_default(),
    ] {
        let known = known.to_lowercase();
        if known.len() < 3 {
            continue;
        }
        assert!(
            !written.contains(&known),
            "the advertisement carries `{known}`, which is more than presence"
        );
    }

    // And the words a later version would most plausibly start saying. None of
    // them can be in a packet this crate builds, because `Presence` has no
    // field for any of them — which is what makes this a test of the type
    // rather than of the writer.
    for more_than_presence in [
        "model", "gguf", "grant", "paired", "account", "owner", "user", "org", "busy", "windows",
        "linux",
    ] {
        assert!(
            !written.contains(more_than_presence),
            "the advertisement carries `{more_than_presence}`"
        );
    }
}

/// **Stable across a restart, and not a hardware serial.**
///
/// Both halves in one test because they are one sentence: the identity lives in
/// a file, which is what makes it survive a restart and what makes it *not*
/// survive a reinstall. An identifier that outlived a reinstall would be a
/// tracker, and the second half of this test is what a well-meaning change
/// towards a MAC address or a TPM key would fail.
#[test]
fn the_identity_survives_a_restart_and_outlives_nothing_else() {
    let one = somewhere_of_our_own("first").join("machine");
    let other = somewhere_of_our_own("second").join("machine");

    let before = MachineId::remembered_at(&one).unwrap();
    let after_a_restart = MachineId::remembered_at(&one).unwrap();
    let after_a_reinstall = MachineId::remembered_at(&other).unwrap();

    assert_eq!(before, after_a_restart, "the machine forgot who it was");
    assert_ne!(
        before, after_a_reinstall,
        "the identity outlived the installation, which makes it a tracker"
    );

    drop(std::fs::remove_dir_all(one.parent().unwrap()));
    drop(std::fs::remove_dir_all(other.parent().unwrap()));
}

/// **Discovery connects to nothing it finds, the one wire this crate has
/// dials only what discovery measured, and nothing here is configurable.**
///
/// Read off the crate's own source, which is the only way to hold *there is no
/// such thing here*. Two failures are being guarded against, and neither is a
/// stranger's:
///
/// A **connection from discovery** would be the half of ADR 0003 task 1 does
/// not build. Discovery ends at a fact written down; turning that fact into a
/// machine this one will talk to is mutual pairing, and a `TcpStream::connect`
/// appearing in the files that advertise, look and read would mean that had
/// been skipped. Task 7 built the pairing wire, in exactly two files —
/// `dialling.rs`, which opens a connection, and `receiving.rs`, which accepts
/// one — and those two are the only files permitted to; every other file is
/// held to the original promise. And `dialling.rs` takes its address from a
/// `Found` or from a `Waiting` made from one, never from a string: no address
/// is spelt in it.
///
/// A **setting** would be the trusted-network setting arriving by the back
/// door. ADR 0003 says there is no such setting, and *discovery off for this
/// network* or *advertise as* would each be one under another name.
#[test]
fn discovery_dials_nothing_the_wire_dials_what_it_measured_and_nothing_has_a_setting() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = 0_usize;
    let mut dials = Vec::new();
    for file in std::fs::read_dir(&source).unwrap() {
        let file = file.unwrap().path();
        if file.extension().is_none_or(|of| of != "rs") {
            continue;
        }
        let written = std::fs::read_to_string(&file).unwrap();
        read += 1;
        let name = file
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name == "dialling.rs" || name == "receiving.rs" {
            dials.push(name.clone());
            let ships = written
                .split_once("#[cfg(test)]")
                .map_or(written.as_str(), |(before, _)| before);
            assert!(
                !ships.contains("parse(") && !ships.contains("SocketAddr::new"),
                "{name} spells an address of its own rather than taking what discovery measured"
            );
            continue;
        }
        // Comments say what this crate does not do, so only the code is read —
        // and only the code that ships. A test may look up this machine's own
        // name in the environment in order to prove the packet does not carry
        // it, which is the opposite of the thing being looked for.
        let ships = written
            .split_once("#[cfg(test)]")
            .map_or(written.as_str(), |(before, _)| before);
        let code: String = ships
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for dialling in ["TcpStream", "TcpListener", "connect("] {
            assert!(
                !code.contains(dialling),
                "{} opens a connection, which only the pairing wire's two files may",
                file.display()
            );
        }
        for a_setting in ["Settings", "config", "Config", "env::var"] {
            assert!(
                !code.contains(a_setting),
                "{} reads a setting, and ADR 0003 says there is no trusted-network setting",
                file.display()
            );
        }
    }
    assert!(read > 4, "only {read} files were read");
    dials.sort_unstable();
    assert_eq!(
        dials,
        ["dialling.rs", "receiving.rs"],
        "the two files of the pairing wire are not both there to be held"
    );
}

/// The service is spelled once, in the crate, so a change to it is a change
/// every machine sees rather than a string edited in one of two places.
#[test]
fn the_service_is_named_in_one_place() {
    assert_eq!(SERVICE, "_alo-os._tcp.local");
    let packet = advertising::a_question().unwrap();
    assert!(
        String::from_utf8_lossy(&packet).contains("_alo-os"),
        "the question asks for some other service than the one the crate names"
    );
}

/// A directory of this test's own, so two tests never share a file.
fn somewhere_of_our_own(called: &str) -> std::path::PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-nearby-road-{called}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    drop(std::fs::remove_dir_all(&at));
    at
}
