//! A turn of this service is refused before its first verb when the machine's
//! boundary has gone — and the refusal is written down.
//!
//! `a_turn_is_bounded_by_the_kernel.rs` is the ordinary path: a real boundary,
//! a real verb, an entry saying it ran. This file is the same service on the
//! same kernel one moment later, after somebody has removed one of the pins
//! that hold the programme on its hooks, which is the one thing on the machine
//! that detaches a hook. `docs/features.md` promises what happens next — *a
//! turn whose boundary cannot be applied does not run: a refusal, not a
//! warning* — and this is that promise held at the layer a person meets it:
//! `alo_turn::Turning`'s own doors, over `alo_agentd::ByTheKernel`, with
//! `alo-keeping` writing the record to a real file.
//!
//! What is asserted:
//!
//! - a read that answered while the boundary was in place is refused once a
//!   hook's pin is gone, with [`alo_turn::NotDone::NotBounded`] — not the
//!   grants refusing, and not the end of the turn;
//! - an approved change is refused the same way, and the file is exactly
//!   where it was: nothing ran;
//! - neither refusal cost the service a thread or closed the turn, so the
//!   service goes on serving and says the same thing to the next request;
//! - the record on disk has the entry that ran and the two refusals as the
//!   machine's own, each carrying the sentence the person read and the
//!   machine's account naming the hook.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! And it loads the programme itself, standing in for `alo-boundaryd`, pinned
//! somewhere of its own and taken away at the end — the sibling file says why.
//! It is one test for the sibling's reason too: `alo_bounding::Turns::under`
//! moves this whole process into a control group, and two of those at once
//! would fight over where the process is.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_agentd::{ByTheKernel, starting};
use alo_bounding::{Imposed, Pinned};
use alo_capability::{Given, Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_keeping::{Reading, Writing};
use alo_record::Happened;
use alo_turn::{Machine, NotDone, Turning};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and this turn last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// Where this process is, as the kernel answers it.
fn where_this_process_is() -> String {
    fs::read_to_string("/proc/self/cgroup")
        .expect("this machine has a unified control group hierarchy")
        .trim()
        .to_owned()
}

/// **A turn is refused before its first verb once the boundary has gone, and
/// the refusal is written down.**
#[test]
fn a_turn_of_this_service_is_refused_when_a_hook_is_no_longer_held() {
    let _kernel = alo_bounding::Waited::on_this_kernel()
        .expect("this kernel can be taken, and nothing is forced if it cannot");
    let root = PathBuf::from("/tmp").join(format!("alo-agentd-unbounded-{}", std::process::id()));
    let invoices = root.join("Invoices");
    let evidence = root.join("evidence");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&invoices).expect("a temporary directory can be made");
    fs::create_dir_all(&evidence).expect("a temporary directory can be made");
    fs::write(invoices.join("march.pdf"), b"March, 4180.00").expect("a file can be written");

    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let record = evidence.join("record.jsonl");
    let mut writing = Writing::opening(&record).expect("a record can be opened");
    let mut indicator = Indicator::default();

    // Standing in for `alo-boundaryd`, somewhere of this test's own.
    let pinned = Pinned::beneath(
        &PathBuf::from("/sys/fs/bpf").join(format!("alo-agentd-gone-{}", std::process::id())),
    );
    pinned.taken_away();
    pinned
        .made()
        .expect("this machine has a BPF filesystem at /sys/fs/bpf");
    let loaded = Imposed::once(&pinned).unwrap_or_else(|why| {
        panic!(
            "no boundary could be imposed on this kernel, so nothing below is being tested: \
             {why}\n\
             This needs root, `CONFIG_BPF_LSM=y`, and `bpf` in the list of security modules the \
             kernel *started* — `cat /sys/kernel/security/lsm`. `docs/hardware.md` has the \
             commands."
        );
    });

    let before = where_this_process_is();
    let mut bounding =
        ByTheKernel::beneath(&pinned).expect("a service can open the map a loader pinned for it");
    let inside = where_this_process_is();

    let (read_while_bounded, read_afterwards, approved_afterwards, lost, closed) = {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut writing,
        )
        .expect("the six declare");
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked("@files", Reach::Folder(invoices.clone()), noon(), hour())
                .expect("a grant over a folder"),
        );
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .expect("a turn can begin");
        let reading = |turning: &mut Turning<'_, '_>, grants: &Grants| {
            turning.reading(
                "read_file",
                &[(
                    "file",
                    Given::text(invoices.join("march.pdf").to_string_lossy().into_owned()),
                )],
                grants,
                noon(),
            )
        };

        let read_while_bounded = reading(&mut turning, &grants);

        // **The one thing on the machine that detaches a hook.** The pin on
        // `file_permission` — the hook that decides every read and write —
        // is removed, as an operator's `rm` would remove it. Eleven hooks are
        // still held; the daemon's descriptor on the map is still open; a
        // listing of the map would show nothing wrong with the map.
        fs::remove_file(pinned.use_hook()).expect("a hook's pin can be removed");

        let read_afterwards = reading(&mut turning, &grants);

        // And a change, all the way round: proposed and approved, which needs
        // no boundary, then carried out, which does.
        let id = turning
            .proposing(
                "rename_file",
                &[
                    (
                        "file",
                        Given::text(invoices.join("march.pdf").to_string_lossy().into_owned()),
                    ),
                    ("name", Given::text("march-final.pdf")),
                ],
                &grants,
                hour(),
                noon(),
            )
            .expect("a change the grants permit is put to somebody");
        let approved_afterwards = turning.approving(id, &grants, noon());

        let lost = turning.a_thread_is_lost();
        let closed = turning.is_closed();
        let _gave_a_grant_back = turning.ending(&mut grants);
        (
            read_while_bounded,
            read_afterwards,
            approved_afterwards,
            lost,
            closed,
        )
    };

    let after_the_turns = where_this_process_is();
    bounding
        .given_back()
        .expect("a service can be put back where it was");
    let after = where_this_process_is();
    drop(writing);
    pinned.taken_away();
    drop(loaded);

    // Everything is asserted out here, because a failing assertion inside a
    // boundary would panic inside one.
    assert_ne!(
        inside, before,
        "the service did not move into a subtree of its own"
    );
    assert_eq!(
        after_the_turns, inside,
        "a refused turn left the service's thread somewhere other than home"
    );
    assert_eq!(after, before, "the service was not put back where it was");

    let read =
        read_while_bounded.expect("a granted read was refused while the boundary was in place");
    assert_eq!(read.read(), Some("March, 4180.00"));

    let refused = read_afterwards.expect_err("a read ran with a hook of the boundary detached");
    assert!(matches!(refused, NotDone::NotBounded(_)), "{refused:?}");
    assert!(
        !refused.was_refused(),
        "the machine's own refusal was reported as the grants refusing"
    );
    assert!(!refused.is_the_end_of_the_turn());
    let said = refused.said(&strings).into_text();
    assert!(said.starts_with("nothing was done"), "{said}");
    assert!(
        !said.contains("file_permission"),
        "the machine's sentence reached the person: {said}"
    );

    let refused = approved_afterwards.expect_err("an approved change ran with the boundary gone");
    assert!(matches!(refused, NotDone::NotBounded(_)), "{refused:?}");
    assert!(
        invoices.join("march.pdf").is_file(),
        "the change ran: the file was renamed on a machine with no boundary"
    );
    assert!(!invoices.join("march-final.pdf").exists());

    assert!(
        !lost,
        "a refusal before the turn began cost the service a thread"
    );
    assert!(
        !closed,
        "a refusal the record kept was reported as the record failing"
    );

    let kept = Reading::at(&record).expect("the record can be read back");
    assert_eq!(
        kept.record().len(),
        3,
        "the read and the two refusals were not all written down"
    );
    let mut entries = kept.record().everything();
    let first = entries
        .next()
        .expect("the read that ran while the boundary was in place is the first entry");
    assert!(first.happened().ran());
    for refusal in entries {
        assert!(refusal.happened().was_stopped());
        assert!(!refusal.happened().ran());
        assert!(
            matches!(
                refusal.happened(),
                Happened::NotBounded { agent, why, machine }
                    if agent.is("@files")
                        && why.as_str().starts_with("nothing was done")
                        && machine.as_str().contains("file_permission")
                        && machine.as_str().contains("docs/quirks.md")
            ),
            "{:?}",
            refusal.happened()
        );
    }

    let _ = fs::remove_dir_all(&root);
}
