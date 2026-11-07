//! **The counter counts what it was asked to, exactly, and nothing else**,
//! measured on a running kernel
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)).
//!
//! The owner allowed one counter: bytes and nothing else, for the control
//! groups a person's command names, for as long as that command runs. Each of
//! those words is a test here:
//!
//! - **bytes, exactly** — a child in a counted control group sends a known
//!   number of datagrams of a known size off the machine, and the count is that
//!   size times that number, headers included, not *roughly* it;
//! - **nothing else** — the same child's datagrams to loopback add nothing,
//!   because nothing left;
//! - **only what was named** — a second child in a control group nobody asked
//!   about sends the same, and gains no entry; the map holds exactly the one
//!   entry the test made;
//! - **only while it runs** — after the count ends, the entry is gone and what
//!   the group sends is not looked at.
//!
//! # It needs root, a BPF filesystem, the BPF LSM, and a route off the machine
//!
//! The first three as `on_this_kernel` says. The fourth because a datagram is
//! sent to `192.0.2.1`, an address reserved for documentation that nothing
//! answers: the packet leaves by the machine's default route, and a machine with
//! none refuses the send, which fails this test rather than skipping it.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env,
    io::{BufRead as _, BufReader, Write as _},
    net::UdpSocket,
    path::Path,
    process::{Command, Stdio},
};

use alo_bounding::{Cgroup, Counter};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Which control group the child puts itself in.
const THE_CGROUP: &str = "ALO_COUNTING_TEST_CGROUP";

/// What the child says once it is inside and waiting.
const READY: &str = "inside";

/// What the child says once it has sent everything.
const SENT: &str = "sent";

/// Somewhere off this machine that nothing answers: TEST-NET-1, RFC 5737.
const OFF_THE_MACHINE: &str = "192.0.2.1:9";

/// Somewhere on it.
const ON_THE_MACHINE: &str = "127.0.0.1:9";

/// How many datagrams each child sends each way.
const DATAGRAMS: u64 = 7;

/// How many bytes each carries.
const PAYLOAD: u64 = 333;

/// What one of them is on the wire as the counter sees it: an IPv4 header of
/// twenty bytes and a UDP header of eight, and the payload.
const ON_THE_WIRE: u64 = 20 + 8 + PAYLOAD;

/// A child of this binary, inside `cgroup`, that has sent its datagrams.
fn a_child_that_sent_from(cgroup: &Cgroup, counter: Option<(&mut Counter, u64)>) {
    let mut child = Command::new(env::current_exe().expect("a test binary knows where it is"))
        .args(["--exact", "--ignored", "--nocapture", "what_a_child_sends"])
        .env(THE_CGROUP, cgroup.at())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");
    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");
    assert_eq!(next_line(&mut saying), READY, "the child never reached its control group");
    if let Some((counter, group)) = counter {
        counter.begun(group).expect("the kernel takes the entry");
    }
    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");
    assert_eq!(next_line(&mut saying), SENT, "the child did not send");
    let finished = child.wait().expect("the child finishes");
    assert!(finished.success(), "the child failed: {finished}");
}

/// The next line the child says that is one of ours, ignoring the test
/// harness's own output.
fn next_line(saying: &mut BufReader<std::process::ChildStdout>) -> String {
    loop {
        let mut line = String::new();
        let read = saying.read_line(&mut line).expect("the child's output is readable");
        assert!(read > 0, "the child stopped talking before it said anything of ours");
        let line = line.trim().to_owned();
        if line == READY || line == SENT {
            return line;
        }
    }
}

/// **Exactly what a counted group sent off the machine, and nothing it sent to
/// itself; nothing for a group nobody named; and nothing after the count ends.**
#[test]
fn the_counter_counts_a_named_group_exactly_and_nothing_else() {
    let _order = on_this_kernel::one_at_a_time();
    let kernel = AsAMachineHasIt::on_this_kernel("counting");
    let mut counter = Counter::opened(kernel.pinned()).unwrap_or_else(|why| {
        panic!("the counter was pinned and could not be opened by path, which is what the desktop does: {why}")
    });
    assert_eq!(
        counter.every_group_counted().expect("the map can be read"),
        Vec::<u64>::new(),
        "the counter holds an entry before anybody asked for one"
    );

    let named = Cgroup::made("alo-counting-named").expect("a control group can be made");
    let nobodys = Cgroup::made("alo-counting-nobodys").expect("a control group can be made");
    let group = named.id().expect("a control group has an identifier");
    let other = nobodys.id().expect("a control group has an identifier");

    a_child_that_sent_from(&named, Some((&mut counter, group)));
    a_child_that_sent_from(&nobodys, None);

    assert_eq!(
        counter.so_far(group).expect("the map can be read"),
        Some(DATAGRAMS * ON_THE_WIRE),
        "the counted group's count is not what it sent off the machine — {DATAGRAMS} datagrams \
         of {ON_THE_WIRE} bytes — so it counted loopback, missed something, or counted another \
         group's packets"
    );
    assert_eq!(
        counter.so_far(other).expect("the map can be read"),
        None,
        "a group nobody asked about has an entry: the program made one"
    );
    assert_eq!(
        counter.every_group_counted().expect("the map can be read"),
        vec![group],
        "the map holds something other than the one entry that was made"
    );

    counter.ended(group).expect("the kernel takes the entry back");
    counter.ended(group).expect("ending a count twice is ending it once");
    a_child_that_sent_from(&named, None);
    assert_eq!(
        counter.so_far(group).expect("the map can be read"),
        None,
        "the group was still counted after its count ended"
    );
    assert!(
        counter.every_group_counted().expect("the map can be read").is_empty(),
        "an entry outlived the count it was made for"
    );

    named.removed().expect("an empty control group can be taken away");
    nobodys.removed().expect("an empty control group can be taken away");
}

/// **A count begun twice keeps what it had**: two commands counting one
/// application must not have the second wipe the first's bytes.
#[test]
fn a_count_begun_again_keeps_what_it_had() {
    let _order = on_this_kernel::one_at_a_time();
    let kernel = AsAMachineHasIt::on_this_kernel("counting-twice");
    let mut counter = Counter::opened(kernel.pinned()).expect("the counter can be opened");
    let named = Cgroup::made("alo-counting-twice").expect("a control group can be made");
    let group = named.id().expect("a control group has an identifier");

    a_child_that_sent_from(&named, Some((&mut counter, group)));
    counter.begun(group).expect("a second beginning is not refused");
    assert_eq!(
        counter.so_far(group).expect("the map can be read"),
        Some(DATAGRAMS * ON_THE_WIRE),
        "beginning a count that was already running set it back"
    );

    counter.ended(group).expect("the kernel takes the entry back");
    named.removed().expect("an empty control group can be taken away");
}

/// **A machine whose counter was never pinned is told so**, rather than
/// answering *nothing was sent*.
#[test]
fn a_machine_with_no_counter_is_refused_rather_than_answered_zero() {
    let nowhere = alo_bounding::Pinned::beneath(Path::new("/sys/fs/bpf/alo-no-counter-here"));
    let refused = Counter::opened(&nowhere).expect_err("a counter nobody pinned was opened");
    assert!(
        matches!(refused, alo_bounding::NotBounded::NoBoundaryHere { ref path } if path.ends_with("counted")),
        "{refused}"
    );
}

/// The child: into the control group, then seven datagrams off the machine and
/// seven to itself, from one socket made inside.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Without the environment that names a control group it does nothing.
#[test]
#[ignore = "this is the child half of the tests above, and the parent runs it by name"]
fn what_a_child_sends() {
    let Ok(cgroup) = env::var(THE_CGROUP) else {
        return;
    };
    std::fs::write(
        Path::new(&cgroup).join("cgroup.procs"),
        std::process::id().to_string(),
    )
    .expect("a process can put itself in a control group");

    // Made inside, so the socket belongs to this control group.
    let socket = UdpSocket::bind("0.0.0.0:0").expect("a datagram socket can be made");
    let mut saying = std::io::stdout();
    writeln!(saying, "{READY}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");
    let mut go = String::new();
    std::io::stdin().read_line(&mut go).expect("the parent says when");

    let payload = [b'a'; PAYLOAD as usize];
    for _ in 0..DATAGRAMS {
        socket
            .send_to(&payload, OFF_THE_MACHINE)
            .expect("a datagram can leave by the default route");
        socket
            .send_to(&payload, ON_THE_MACHINE)
            .expect("a datagram can be sent to this machine");
    }
    writeln!(saying, "{SENT}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");
}
