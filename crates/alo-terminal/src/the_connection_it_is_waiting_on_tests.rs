//! That the hex decodes the way the kernel means it, and that a real socket is
//! found.
//!
//! # The decoding is tested on the string the kernel really wrote
//!
//! `0100007F:AEAB`, taken from `/proc/net/tcp` on 2026-10-17 for a socket bound
//! to loopback. **Both halves are wrong in a way that looks right**: read the
//! address forwards and it is `1.0.0.127`, a real address somewhere else; read
//! the port backwards and it is 43950, a real port that is not this one. So the
//! test names the answer rather than the method.

use std::net::{IpAddr, Ipv4Addr};

use super::{TheConnection, an_address, an_end, the_connection_being_read_by};

/// **Loopback, as the kernel writes it.**
#[test]
fn the_address_is_little_endian_and_the_port_is_not() {
    let end = an_end("0100007F:AEAB").expect("the kernel's own string reads");
    assert_eq!(
        end.address,
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        "the address was read the wrong way round: 0100007F is 127.0.0.1"
    );
    assert_eq!(
        end.port, 44_715,
        "the port was read the wrong way round: AEAB is 44715"
    );

    // **The control for each half.** Both of these are what the other order
    // gives, and neither must be the answer above.
    assert_ne!(end.address, IpAddr::V4(Ipv4Addr::new(1, 0, 0, 127)));
    assert_ne!(end.port, 43_950);

    // And it reads as a person would expect to see it.
    assert_eq!(format!("{end}"), "127.0.0.1:44715");
}

/// **A socket with no far end is listening**, which is a state rather than a
/// failure to read one.
#[test]
fn a_socket_with_no_far_end_is_listening() {
    let nowhere = an_end("00000000:0000").expect("the kernel's own string reads");
    assert_eq!(nowhere.address, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    assert_eq!(nowhere.port, 0);
}

/// **An address of a length this does not know is refused**, rather than being
/// read as whatever `from_str_radix` makes of it.
#[test]
fn an_address_of_the_wrong_length_is_refused() {
    assert!(an_address("").is_none());
    assert!(
        an_address("0100").is_none(),
        "four characters is not an address"
    );
    assert!(an_address("0100007").is_none(), "seven is not eight");
    assert!(an_address("not hex!").is_none());
}

/// **IPv6 is four little-endian words**, and loopback is the one worth naming.
///
/// `::1` is written as three zero words and then `01000000` — the one in the
/// *last* word, little-endian within it.
#[test]
fn ipv6_is_four_little_endian_words() {
    let loopback = an_address("00000000000000000000000001000000")
        .expect("thirty-two characters is an address");
    assert_eq!(
        loopback,
        IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
        "the four words were not read in the kernel's order"
    );
}

/// **A real socket on this machine is found and named.**
///
/// A listener is opened, and the connection is looked up by the inode its own
/// descriptor reports — so what is measured is the whole road from a descriptor
/// to an address, not the decoding alone.
#[test]
fn a_real_socket_is_found_from_its_own_descriptor() {
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let port = listener
        .local_addr()
        .expect("a bound listener has an address")
        .port();

    // Which inode this process's listener is, read the way the module reads it.
    let mut found = None;
    for entry in std::fs::read_dir("/proc/self/fd").expect("this process has descriptors") {
        let at = entry.expect("a directory entry").path();
        let Ok(points) = std::fs::read_link(&at) else {
            continue;
        };
        let Some(inode) = points
            .to_str()
            .and_then(|it| it.strip_prefix("socket:["))
            .and_then(|it| it.strip_suffix(']'))
        else {
            continue;
        };
        if let Some(TheConnection::OverTheNetwork { kind, here, away }) =
            super::the_connection_numbered(inode)
            && here.port == port
        {
            found = Some((kind, here, away));
            break;
        }
    }

    let (kind, here, away) = found.expect("the listener this test opened is in the kernel's table");
    assert_eq!(kind, "tcp");
    assert_eq!(here.address, IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(here.port, port);
    assert_eq!(away, None, "a listener has no far end yet");
}

/// **A command whose input is not a socket is not reported as having one.**
///
/// The control: a reader that answered about any descriptor would name
/// something here, and there is nothing to name.
#[test]
fn a_command_whose_input_is_not_a_socket_has_no_connection() {
    // This test's own standard input is a pipe or a file, never a socket.
    assert_eq!(
        the_connection_being_read_by(std::process::id()),
        None,
        "this process's input is not a socket and something was named anyway"
    );
}

/// **Each shape says its own thing**, so a caller showing one is not showing
/// the same sentence for a listener, a conversation and a socket that never
/// leaves the machine.
#[test]
fn each_kind_of_connection_says_its_own_thing() {
    let here = an_end("0100007F:AEAB").expect("reads");
    let away = an_end("0100007F:0050").expect("reads");

    let talking = TheConnection::OverTheNetwork {
        kind: "tcp",
        here,
        away: Some(away),
    };
    let listening = TheConnection::OverTheNetwork {
        kind: "tcp",
        here,
        away: None,
    };
    let named = TheConnection::OnThisMachine {
        path: Some("/run/alo/door".to_owned()),
    };
    let unnamed = TheConnection::OnThisMachine { path: None };

    let said: Vec<String> = [&talking, &listening, &named, &unnamed]
        .iter()
        .map(|it| format!("{it}"))
        .collect();
    // Paired with what each must say, so a reader sees which sentence is which
    // rather than four index numbers.
    for (said, must_say) in
        said.iter()
            .zip(["to 127.0.0.1:80", "listening", "/run/alo/door", "no name"])
    {
        assert!(
            said.contains(must_say),
            "a connection does not say {must_say:?}: {said}"
        );
    }

    for (one, two) in said.iter().zip(said.iter().skip(1)) {
        assert_ne!(one, two, "two kinds of connection read the same");
    }
}
