//! One turn of this service, carried out inside a boundary the kernel really
//! imposes.
//!
//! `alo-bounding`'s own tests prove the kernel refuses: a thread inside a turn's
//! control group opens a file it was granted and is refused a private key beside
//! it, and nothing in this repository made that decision. What they cannot prove
//! is the **wiring** — that the places `alo-agentd` works out for a real call are
//! the places that real call has to open — and that is what this file is for.
//!
//! It is the difference between a mechanism and a machine. A boundary computed
//! one place too narrow does not look wrong: it looks like a verb that failed,
//! on a machine where every verb fails, with the kernel quietly refusing an open
//! nobody wrote down. So the assertion here is the ordinary path — a read
//! answers, an archive is written — under a boundary that is really in force,
//! and law 3's *done means the machine still works* is exactly that sentence.
//!
//! **What this file deliberately does not assert is the refusal**, and the
//! division is on purpose rather than for want of trying. Proving the kernel
//! says no needs something inside the boundary that opens what it should not,
//! and the only thing that runs in there is the verb — so a test of it would
//! mean putting a probe inside a turn, which is a thing this service must not be
//! able to do. `alo-bounding`'s `a_turn_is_this_thread.rs` proves the refusal
//! against the same kernel in the same suite, holding the same [`ByTheKernel`]
//! pieces one layer down. The two halves are the claim.
//!
//! # It needs root, and a kernel that started the BPF security module
//!
//! The same three checks `alo-bounding`'s tests name, and it fails loudly on a
//! machine without them for the same reason: a test that quietly skipped itself
//! would report green on every machine where nothing is bounded at all.
//!
//! # It is one test, and that is not laziness
//!
//! `alo_bounding::Turns::under` moves **this whole process** into a control
//! group of its own and gives it back afterwards, so two of these running at
//! once would fight over where this process is. A test binary runs its tests in
//! parallel threads, so the way to have one at a time is to have one — and a
//! file with a single test in it is the arrangement that cannot be broken by
//! somebody adding a second one carelessly, because they would have to read this
//! paragraph to do it.
//!
//! What the rest of this crate's tests are run with is `crate::testing`'s
//! `NothingIsBounded`, which is why they can be parallel and why this exists.

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
use alo_capability::{Given, Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_keeping::{Reading, Writing};
use alo_turn::{Machine, Turning};

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

/// **A turn of this service runs inside a boundary the kernel imposes, and does
/// what it was asked.**
///
/// Four things in one test, because they are one sequence on one machine:
///
/// 1. **The service makes its subtree**, which is what `ByTheKernel::imposed`
///    does to the process it is called in — the kernel's own answer about where
///    this process is changes, and that is asserted rather than assumed.
/// 2. **A read answers under the boundary.** The one place that call names is
///    the one place it opens, so a bound made of the wrong paths would come back
///    as *the machine could not*.
/// 3. **An archive answers under the boundary**, which is the widest call the
///    six have: it reads a folder, creates a file in a second folder, and the
///    second is a place nothing but `alo_files::Reaching`'s *folder above
///    anything it would create* rule puts inside the bound. This is the test
///    item 26c's arithmetic was owed against a real kernel.
/// 4. **Both are on the disk afterwards**, which is the boundary being around
///    the verb and not around the entry: the record is a file nobody granted,
///    and a thread bounded across the writing of it would be refused its own
///    evidence.
///
/// Then the subtree is given back and this process is where it started.
#[test]
fn a_turn_of_this_service_runs_inside_a_boundary_and_does_what_it_was_asked() {
    let root = PathBuf::from("/tmp").join(format!("alo-agentd-bounded-{}", std::process::id()));
    let invoices = root.join("Invoices");
    let archive = root.join("Archive");
    let evidence = root.join("evidence");
    fs::create_dir_all(&invoices).expect("a temporary directory can be made");
    fs::create_dir_all(&archive).expect("a temporary directory can be made");
    fs::create_dir_all(&evidence).expect("a temporary directory can be made");
    fs::write(invoices.join("march.pdf"), b"March, 4180.00").expect("a file can be written");

    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let record = evidence.join("record.jsonl");
    let mut writing = Writing::opening(&record).expect("a record can be opened");
    let mut indicator = Indicator::default();

    let before = where_this_process_is();
    let mut bounding = ByTheKernel::imposed().unwrap_or_else(|why| {
        panic!(
            "no boundary could be imposed on this kernel, so nothing below is being tested: \
             {why}\n\
             This needs root, `CONFIG_BPF_LSM=y`, and `bpf` in the list of security modules the \
             kernel *started* — `cat /sys/kernel/security/lsm`, which is not the same question \
             as how the kernel was built. `docs/hardware.md` has the three commands."
        );
    });
    let inside = where_this_process_is();

    let (read, archived) = {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut writing,
        )
        .expect("the six declare");
        let mut grants = Grants::default();
        for folder in [&invoices, &archive] {
            grants.grant(
                Grant::checked("@files", Reach::Folder(folder.clone()), noon(), hour())
                    .expect("a grant over a folder"),
            );
        }
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .expect("a turn can begin");

        let read = turning.reading(
            "read_file",
            &[(
                "file",
                Given::text(invoices.join("march.pdf").to_string_lossy().into_owned()),
            )],
            &grants,
            noon(),
        );

        // The widest call there is: two places, and the second is inside the
        // bound only because something has to be able to create a name in it.
        // It is a change, so it goes the whole way round — proposed, approved,
        // and only then carried out inside the boundary.
        let id = turning
            .proposing(
                "archive_folder",
                &[
                    (
                        "folder",
                        Given::text(invoices.to_string_lossy().into_owned()),
                    ),
                    ("into", Given::text(archive.to_string_lossy().into_owned())),
                    ("name", Given::text("invoices.zip")),
                ],
                &grants,
                hour(),
                noon(),
            )
            .expect("a change the grants permit is put to somebody");
        let archived = turning.approving(id, &grants, noon());
        let _gave_a_grant_back = turning.ending(&mut grants);
        (read, archived)
    };

    let after_the_turn = where_this_process_is();
    bounding
        .given_back()
        .expect("a service can be put back where it was");
    let after = where_this_process_is();
    drop(writing);

    // Everything is asserted out here, because a failing assertion inside a
    // boundary would panic inside one, and a panic prints a backtrace, and
    // reading `/proc/self/maps` is an open like any other.
    assert_ne!(
        inside, before,
        "the service did not move into a control group subtree of its own"
    );
    assert!(
        inside.ends_with("/home"),
        "the service's threads are not where `alo_bounding::Turns` puts them: {inside}"
    );
    assert_eq!(
        after_the_turn, inside,
        "the thread that did the turn's work did not come back out of it"
    );
    assert_eq!(after, before, "the service was not put back where it was");

    let read = read.expect("a granted file was refused to the turn that named it");
    assert_eq!(
        read.read(),
        Some("March, 4180.00"),
        "the boundary let the open through and the answer is not what is in the file"
    );

    let archived = archived.expect("the widest call the six have was refused inside its boundary");
    let made = archived.archived().expect("an archive answers with itself");
    assert_eq!(made.things(), 1);
    assert!(
        archive.join("invoices.zip").is_file(),
        "the archive was answered for and is not on the disk"
    );

    let kept = Reading::at(&record).expect("the record can be read back");
    assert_eq!(
        kept.record().len(),
        2,
        "the entries were not written down, which is the boundary being around the record"
    );
    assert!(
        kept.record()
            .everything()
            .all(|entry| entry.happened().ran()),
        "a turn inside a real boundary reported something other than having run"
    );
}
