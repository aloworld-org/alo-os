//! **A command alo runs is counted from its first instruction, exactly, and
//! nothing beside it is**, measured on a running kernel
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
//! as amended 2026-11-13).
//!
//! The owner, 2026-11-13: *"Yes, count alo's commands"* — bytes only, only for
//! the seconds alo's command runs, and only alo's command, never the person's.
//! What `alo-agentd` does for one command is this file's order, on this
//! kernel:
//!
//! - **a group of its own, beside the service's turns**
//!   (`Turns::a_group_for_a_command`), and a count begun on it;
//! - **born inside it**: this thread steps in, starts a child, and steps back
//!   out (`Turns::from_inside`) — and is home again afterwards, so nothing of
//!   the service is left in the command's group;
//! - **exactly what it sent off the machine**, headers included, and nothing it
//!   sent to itself;
//! - **nothing beside it**: a second child started from the service's own
//!   group sends the same, at the same time, and gains no entry — the map holds
//!   exactly the command's;
//! - **only while it runs**: once the count ends, the entry is gone, and the
//!   group is taken away.
//!
//! # It needs root, a BPF filesystem, the BPF LSM, and a route off the machine
//!
//! As `the_counter_counts_only_what_it_was_asked_to.rs` does, for the same
//! reasons: a datagram is sent to `192.0.2.1`, an address reserved for
//! documentation that nothing answers, and a machine with no default route
//! refuses the send, which fails this test rather than skipping it.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    io::{BufRead as _, BufReader, Write as _},
    net::UdpSocket,
    path::Path,
    process::{Child, Command, Stdio},
};

use alo_bounding::{Cgroup, Counter, Turns};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Set for a child of this binary that is to send and stop.
const SEND: &str = "ALO_COMMAND_COUNTING_CHILD";

/// What the child says once it has sent everything.
const SENT: &str = "sent";

/// Somewhere off this machine that nothing answers: TEST-NET-1, RFC 5737.
const OFF_THE_MACHINE: &str = "192.0.2.1:9";

/// Somewhere on it.
const ON_THE_MACHINE: &str = "127.0.0.1:9";

/// How many datagrams each child sends each way.
const DATAGRAMS: u64 = 5;

/// How many bytes each carries.
const PAYLOAD: u64 = 271;

/// What one of them is on the wire as the counter sees it: an IPv4 header of
/// twenty bytes and a UDP header of eight, and the payload.
const ON_THE_WIRE: u64 = 20 + 8 + PAYLOAD;

/// A child of this binary that sends its datagrams and stops, started from
/// wherever this thread is.
fn a_child_that_sends() -> std::io::Result<Child> {
    Command::new(env::current_exe()?)
        .args(["--exact", "--ignored", "--nocapture", "what_a_child_sends"])
        .env(SEND, "1")
        .stdout(Stdio::piped())
        .spawn()
}

/// Wait for a child to say it sent everything, and to finish.
fn sent_and_finished(mut child: Child) {
    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    loop {
        let mut line = String::new();
        let read = saying
            .read_line(&mut line)
            .expect("the child's output is readable");
        assert!(read > 0, "the child stopped before it said it had sent");
        if line.trim() == SENT {
            break;
        }
    }
    let finished = child.wait().expect("the child finishes");
    assert!(finished.success(), "the child failed: {finished}");
}

/// The control group this thread is in, as the kernel says it.
fn where_this_thread_is() -> String {
    let said = fs::read_to_string("/proc/thread-self/cgroup").expect("the kernel says where");
    said.lines()
        .find_map(|line| line.strip_prefix("0::"))
        .expect("there is a unified hierarchy")
        .trim()
        .to_owned()
}

/// `at`, as `/proc` names a control group: from the top of the hierarchy.
fn as_the_kernel_names_it(at: &Path) -> String {
    let under = at
        .strip_prefix("/sys/fs/cgroup")
        .expect("a control group is under the hierarchy");
    format!("/{}", under.display())
}

/// **Exactly what the command sent off the machine; nothing from the program
/// beside it; this thread home again; and nothing after the count ends.**
#[test]
fn a_command_alo_runs_is_counted_from_its_first_instruction_and_nothing_beside_it_is() {
    let _order = on_this_kernel::one_at_a_time();
    let kernel = AsAMachineHasIt::on_this_kernel("alos-command");
    let mut counter = Counter::opened(kernel.pinned().counted())
        .expect("the counter was pinned and can be opened by path, which is what the daemon does");

    let ours = Cgroup::made(&format!("alo-alos-command-{}", std::process::id()))
        .expect("a control group can be made");
    let turns = Turns::under(ours.at()).expect("a service can make a subtree of its own");

    let group = turns
        .a_group_for_a_command("command-1")
        .expect("a command's group can be made beside the turns");
    let id = group.id().expect("a control group has an identifier");
    counter.begun(id).expect("the kernel takes the entry");

    let (started, not_back) = turns
        .from_inside(&group, a_child_that_sends)
        .expect("this thread can step into a command's group");
    assert!(
        not_back.is_none(),
        "this thread could not come home: {not_back:?}"
    );
    assert_eq!(
        where_this_thread_is(),
        as_the_kernel_names_it(turns.home().at()),
        "the service's own thread was left in the command's group"
    );
    let command = started.expect("the child starts");
    // **The program beside it**, started from the service's own group at the
    // same time, sending the same.
    let beside = a_child_that_sends().expect("the child starts");
    sent_and_finished(command);
    sent_and_finished(beside);

    assert_eq!(
        counter.so_far(id).expect("the map can be read"),
        Some(DATAGRAMS * ON_THE_WIRE),
        "the command's count is not what it sent off the machine — {DATAGRAMS} datagrams of \
         {ON_THE_WIRE} bytes — so it was not inside from the start, counted loopback, or counted \
         the program beside it"
    );
    assert_eq!(
        counter.every_group_counted().expect("the map can be read"),
        vec![id],
        "something other than the command's own group is being counted"
    );

    counter.ended(id).expect("the kernel takes the entry back");
    assert!(
        counter
            .every_group_counted()
            .expect("the map can be read")
            .is_empty(),
        "the count outlived the command"
    );
    group
        .removed()
        .expect("a command's group that has emptied can be taken away");
    turns
        .given_back()
        .expect("a service can be put back where it was");
    ours.removed()
        .expect("an empty control group can be taken away");
}

/// The child: one socket made where it was born, then datagrams off the
/// machine and to itself.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Without the environment that tells it to, it does nothing.
#[test]
#[ignore = "this is the child half of the test above, and the parent runs it by name"]
fn what_a_child_sends() {
    if env::var_os(SEND).is_none() {
        return;
    }
    let socket = UdpSocket::bind("0.0.0.0:0").expect("a datagram socket can be made");
    let payload = vec![b'x'; usize::try_from(PAYLOAD).expect("a small number")];
    for _ in 0..DATAGRAMS {
        socket
            .send_to(&payload, OFF_THE_MACHINE)
            .expect("a datagram can leave by the default route");
        socket
            .send_to(&payload, ON_THE_MACHINE)
            .expect("a datagram can be sent to this machine");
    }
    let mut saying = std::io::stdout();
    writeln!(saying, "{SENT}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");
}
