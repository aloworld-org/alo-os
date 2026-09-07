//! The other half of law 1: **nothing leaves a turn that the person was not
//! shown**, refused by the machine rather than by our own code.
//!
//! `what_a_turn_can_reach_on_the_network.rs` measured the gap — a bound turn
//! refused a file and not refused a socket. This is the hook that closes it,
//! and every case below is one turn on a running kernel with the real programme
//! loaded.
//!
//! # One departure is one destination
//!
//! The thing this file exists to hold down is that a departure the person was
//! shown does **not** become permission to connect. A turn shown one address is
//! refused a second one, on the same port, in the same turn, while the first is
//! still permitted. Anything less and the sentence somebody read would have
//! authorised the internet.
//!
//! # Nothing here reaches a network
//!
//! Every address is a listener **this test owns**, on loopback... except that
//! loopback is exactly what the programme does not check, for the reason
//! `deciding.rs` gives at length: ADR 0007 makes a model on this machine the
//! default and `alo-egress` does not call it a departure at all.
//!
//! So the addresses that have to be *checked* cannot be loopback, and they must
//! also not be reachable — a test that really connected somewhere would be a
//! test that reaches a network. They are taken from `192.0.2.0/24`, which
//! **RFC 5737 reserves for documentation** and which no machine routes. What is
//! asserted is what the *kernel* said about the attempt, which is decided before
//! a packet is sent: `EACCES` when refused, and anything else — a timeout, no
//! route — when permitted. The two are never confused, because a refusal is the
//! only outcome that is `EACCES`.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alo_bounding::{Cgroup, Departure, Departures, Family, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_DEPARTURE_TEST_CGROUP";

/// The addresses the child tries, in order, separated by spaces.
const THE_ADDRESSES: &str = "ALO_DEPARTURE_TEST_ADDRESSES";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about one address, followed by the outcome.
const WENT: &str = "alo:went ";

/// What the child says when it has tried them all.
const DONE: &str = "alo:done";

/// `EACCES`, which is the only thing a refusal by this boundary looks like.
const REFUSED: i32 = 13;

/// A documentation address, which no machine routes (RFC 5737).
const NOWHERE_ROUTED: &str = "192.0.2.10";

/// Another, so that *a second destination* is a different address rather than a
/// different port on the same one.
const SOMEWHERE_ELSE: &str = "192.0.2.20";

/// The port every address in this file uses.
const A_PORT: u16 = 443;

/// What the kernel made of one attempt.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The boundary refused it before anything was sent.
    Refused,

    /// The boundary did not refuse it. What happened next — a timeout, no route
    /// — is the network's business and not this boundary's.
    NotRefused,
}

impl Outcome {
    /// What the child wrote, read back.
    fn from(text: &str) -> Self {
        match text.parse::<i32>() {
            Ok(REFUSED) => Self::Refused,
            _ => Self::NotRefused,
        }
    }
}

/// A granted folder, so that a turn has a bound at all.
///
/// Named for the test as well as the process: these run beside each other, and
/// one root shared between them is each tidying up while the others are still
/// using it.
fn a_folder_to_grant(what: &str) -> PathBuf {
    let root = PathBuf::from("/tmp").join(format!("alo-departure-{}-{what}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
    root
}

/// A destination as the map holds one.
fn shown(address: &str, port: u16) -> Departure {
    let four: std::net::Ipv4Addr = address.parse().expect("a written address");
    Departure::of(Family::Four, u128::from(four.to_bits()), port)
}

/// One turn: a cgroup, a bound with these destinations in it, and a child that
/// tries each address in turn.
fn a_turn(named: &str, granted: &Path, shown_them: Departures, trying: &[String]) -> Vec<Outcome> {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(named);
    let boundary = &mut kernel.boundary;
    let cgroup = Cgroup::made(named).expect("a control group can be made");
    let turn = cgroup.id().expect("a control group has an identifier");

    let mut child = Command::new(env::current_exe().expect("a test binary knows where it is"))
        .args([
            "--exact",
            "--ignored",
            "--nocapture",
            "the_work_a_turn_does",
        ])
        .env(THE_CGROUP, cgroup.at())
        .env(THE_ADDRESSES, trying.join(" "))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");

    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");

    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the child never reached its cgroup");

    let places = places_of(&[granted]).expect("the granted folder is there");
    boundary
        .bound(turn, places.and_shown(shown_them))
        .expect("the kernel takes the entry");

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let mut went = Vec::new();
    loop {
        let said = next_line_from(&mut saying);
        let said = said.trim();
        if said == DONE {
            break;
        }
        let what = said
            .strip_prefix(WENT)
            .expect("every line is an outcome or the end");
        went.push(Outcome::from(what));
    }

    child.wait().expect("the child finishes");
    boundary.released(turn).expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    went
}

/// The next line the child says that is meant for us.
fn next_line_from(saying: &mut BufReader<std::process::ChildStdout>) -> String {
    let mut line = String::new();
    loop {
        line.clear();
        let read = saying.read_line(&mut line).expect("the child is talking");
        assert!(
            read > 0,
            "the child stopped talking before it said anything"
        );
        if line.starts_with("alo:") {
            return line;
        }
    }
}

/// Every address, as the child is given them.
fn addresses(each: &[&str]) -> Vec<String> {
    each.iter()
        .map(|address| format!("{address}:{A_PORT}"))
        .collect()
}

/// **Default deny.** A turn shown nothing connects to nothing.
///
/// This is the state every turn begins in and nothing has to be written for it:
/// a bound with no destinations in it refuses every connection the turn makes.
#[test]
fn a_turn_shown_nothing_is_refused_every_destination() {
    let machine = a_folder_to_grant("none");
    let went = a_turn(
        "alo-departure-none",
        &machine.join("Invoices"),
        Departures::none(),
        &addresses(&[NOWHERE_ROUTED, SOMEWHERE_ELSE]),
    );
    assert_eq!(went, [Outcome::Refused, Outcome::Refused]);
    let _ = fs::remove_dir_all(&machine);
}

/// **And the one it was shown is permitted.**
///
/// The test that stops the one above from being a boundary that refuses
/// everything and looks like it works.
#[test]
fn a_turn_shown_a_destination_may_reach_that_one() {
    let machine = a_folder_to_grant("shown");
    let went = a_turn(
        "alo-departure-shown",
        &machine.join("Invoices"),
        Departures::of(&[shown(NOWHERE_ROUTED, A_PORT)]).expect("one is not too many"),
        &addresses(&[NOWHERE_ROUTED]),
    );
    assert_eq!(
        went,
        [Outcome::NotRefused],
        "the boundary refused a destination the person was shown"
    );
    let _ = fs::remove_dir_all(&machine);
}

/// **One departure is one destination, and this is the whole point.**
///
/// The same turn, with one address shown, reaching a *different* one. If a
/// displayed departure became permission to connect, this would pass — and the
/// sentence somebody read would have authorised every address there is.
#[test]
fn a_departure_is_not_permission_to_reach_anywhere_else() {
    let machine = a_folder_to_grant("another");
    let went = a_turn(
        "alo-departure-another",
        &machine.join("Invoices"),
        Departures::of(&[shown(NOWHERE_ROUTED, A_PORT)]).expect("one is not too many"),
        &addresses(&[NOWHERE_ROUTED, SOMEWHERE_ELSE]),
    );
    assert_eq!(
        went,
        [Outcome::NotRefused, Outcome::Refused],
        "one displayed departure became permission for a destination nobody showed"
    );
    let _ = fs::remove_dir_all(&machine);
}

/// **And not permission for another port on the same address**, which is a
/// different service and a different thing to have been shown.
#[test]
fn a_departure_is_not_permission_for_another_port() {
    let machine = a_folder_to_grant("port");
    let went = a_turn(
        "alo-departure-port",
        &machine.join("Invoices"),
        Departures::of(&[shown(NOWHERE_ROUTED, A_PORT)]).expect("one is not too many"),
        &[format!("{NOWHERE_ROUTED}:8080")],
    );
    assert_eq!(went, [Outcome::Refused]);
    let _ = fs::remove_dir_all(&machine);
}

/// **Withdrawal.** The daemon rewrites the entry without a destination, and the
/// next connection to it is refused.
///
/// Two turns rather than one, because what is being tested is that the map is
/// what decides and the programme holds nothing between calls: the same address
/// is permitted while it is written and refused once it is not.
#[test]
fn a_destination_withdrawn_is_refused_next_time() {
    let machine = a_folder_to_grant("withdrawn");
    let granted = machine.join("Invoices");
    let trying = addresses(&[NOWHERE_ROUTED]);

    let while_shown = a_turn(
        "alo-departure-while",
        &granted,
        Departures::of(&[shown(NOWHERE_ROUTED, A_PORT)]).expect("one is not too many"),
        &trying,
    );
    assert_eq!(while_shown, [Outcome::NotRefused]);

    let once_withdrawn = a_turn(
        "alo-departure-withdrawn",
        &granted,
        Departures::none(),
        &trying,
    );
    assert_eq!(
        once_withdrawn,
        [Outcome::Refused],
        "a withdrawn destination was still reachable"
    );

    let _ = fs::remove_dir_all(&machine);
}

/// **Loopback is not checked**, and that is a decision rather than an oversight.
///
/// ADR 0007 makes a model on this machine the default, and `alo-egress` does not
/// call a question answered here a departure at all — so nothing is ever shown
/// for it and nothing would ever be written. Refusing it would break the
/// ordinary case the product is built around. `deciding.rs` says so, and
/// `docs/quirks.md` records what it costs.
#[test]
fn a_turn_reaches_this_machine_without_having_been_shown_it() {
    let machine = a_folder_to_grant("listener");
    let ours = TcpListener::bind("127.0.0.1:0").expect("a loopback listener can be made");
    let address = ours
        .local_addr()
        .expect("a listener knows its own address")
        .to_string();

    let went = a_turn(
        "alo-departure-loopback",
        &machine.join("Invoices"),
        Departures::none(),
        &[address],
    );
    assert_eq!(
        went,
        [Outcome::NotRefused],
        "loopback was refused, which breaks a model running on this machine"
    );
    let _ = fs::remove_dir_all(&machine);
}

/// And the case that is almost every connection on the machine: a process that
/// is not a turn is not bounded by anybody's departure.
#[test]
fn a_process_that_is_not_a_turn_connects_as_it_always_could() {
    let machine = a_folder_to_grant("not-a-turn");
    let ours = TcpListener::bind("127.0.0.1:0").expect("a loopback listener can be made");
    let address = ours
        .local_addr()
        .expect("a listener knows its own address")
        .to_string();

    // This process is not in the turn's cgroup, so while a turn is bound and
    // refusing everything, the same connection is made here.
    let _bound = a_turn(
        "alo-departure-elsewhere",
        &machine.join("Invoices"),
        Departures::none(),
        &addresses(&[NOWHERE_ROUTED]),
    );
    std::net::TcpStream::connect(&address).expect("this process is not a turn");

    let _ = fs::remove_dir_all(&machine);
}

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Run without the environment that names a cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the tests above, and the parent runs it by name"]
fn the_work_a_turn_does() {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    let (Ok(cgroup), Ok(addresses)) = (env::var(THE_CGROUP), env::var(THE_ADDRESSES)) else {
        return;
    };

    fs::write(
        Path::new(&cgroup).join("cgroup.procs"),
        std::process::id().to_string(),
    )
    .expect("a process can put itself in a control group");

    let mut saying = std::io::stdout();
    writeln!(saying, "{READY}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    let mut go = String::new();
    std::io::stdin()
        .read_line(&mut go)
        .expect("the parent says when");

    // From here the boundary is in force. A connect opens no file, so each of
    // these is the only thing the kernel is asked about, and the answers go out
    // on a descriptor opened before this process joined anything.
    //
    // The wait is short because a refusal is immediate and everything else is
    // a documentation address nothing routes: what is being read is *which*
    // failure, and `EACCES` is the only one this boundary produces.
    for address in addresses.split_whitespace() {
        let went = match address.parse::<SocketAddr>() {
            Err(_) => "unparsed".to_owned(),
            Ok(where_to) => match TcpStream::connect_timeout(&where_to, Duration::from_secs(2)) {
                Ok(_) => "connected".to_owned(),
                Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
            },
        };
        writeln!(saying, "{WENT}{went}").expect("the parent is listening");
    }
    writeln!(saying, "{DONE}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}
