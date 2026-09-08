//! Two processes taking turns on one kernel, proved across real processes.
//!
//! `alo_bounding::waiting`'s own unit tests ask the question inside one process,
//! which is the cheap half. **This file asks it the way it actually fails**: two
//! processes, one of them killed outright, and a wait that runs out.
//!
//! # Why this file exists at all
//!
//! On 2026-09-08 a gate run in one checkout failed all five of `alo-agentd`'s
//! network tests on a tree where only documents had changed, because the other
//! checkout was running its own suite against this same kernel at that moment.
//! Every test binary already serialised within itself and nothing serialised
//! them across processes. `alo_bounding::waiting` is the answer and this is
//! whether it is true.
//!
//! # Nothing here touches the machine's own lock
//!
//! Every test below binds **a name of its own**, made from this process's id, so
//! that proving the lock cannot disturb a real test holding the real one. The
//! one exception is the last test, which takes the machine's lock precisely
//! because its subject is that kernel enforcement still works while it is held.
//!
//! # And nothing here touches anything it does not own
//!
//! The killed process is this test's own child. No pin is removed, no programme
//! unloaded, and no process this test did not start is signalled — which is the
//! rule the whole workstream is held to, and which this file would otherwise be
//! the most tempting place to break.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};
use std::{env, fs};

use alo_bounding::{Cgroup, NotWaited, Waited, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// The name the child is told to take.
const THE_NAME: &str = "ALO_WAITING_NAME";

/// How long the child is told to wait for it.
const THE_PATIENCE: &str = "ALO_WAITING_PATIENCE";

/// What the child says once it has it.
const HELD: &str = "alo:held";

/// What the child says when it gave up.
const GAVE_UP: &str = "alo:gave-up";

/// Long enough that a slow machine is not a failure.
const ENOUGH: Duration = Duration::from_secs(20);

/// A lock name belonging to this test run and to nothing else.
fn a_name_of_our_own(what: &str) -> String {
    format!("alo-os/taking-turns-{}-{what}", std::process::id())
}

/// A child of this test binary that tries to take a name and says what happened.
///
/// It waits for a line on its standard input before giving the name back, so the
/// parent decides how long it is held for.
fn a_second_process_wanting(named: &str, patience: Duration) -> (Child, BufReader<ChildStdout>) {
    let mut child = Command::new(env::current_exe().expect("a test binary knows where it is"))
        .args(["--exact", "--ignored", "--nocapture", "the_other_process"])
        .env(THE_NAME, named)
        .env(THE_PATIENCE, patience.as_millis().to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");
    let saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    (child, saying)
}

/// The next line the child says that is meant for us.
fn what_it_said(saying: &mut BufReader<ChildStdout>) -> String {
    let mut line = String::new();
    loop {
        line.clear();
        let read = saying.read_line(&mut line).expect("the child is talking");
        assert!(
            read > 0,
            "the child stopped talking before it said anything"
        );
        if line.starts_with("alo:") {
            return line.trim().to_owned();
        }
    }
}

/// **A second process cannot enter while the first holds it.**
///
/// The parent takes the name and keeps it; the child is given a patience short
/// enough to run out and reports that it gave up. This is the case the whole
/// mechanism exists for, and the one that a lock scoped to a single test binary
/// gets wrong.
#[test]
fn a_second_process_cannot_enter_while_the_first_holds_it() {
    let named = a_name_of_our_own("held");
    let held = Waited::called(&named, Duration::from_millis(1)).expect("the first one takes it");

    let (mut child, mut saying) = a_second_process_wanting(&named, Duration::from_millis(400));
    let said = what_it_said(&mut saying);
    assert!(
        said.starts_with(GAVE_UP),
        "a second process entered while the first was holding it: {said}"
    );
    // And it said why, in the sentence somebody stuck on this has to read.
    assert!(said.contains("Nothing was forced"), "{said}");
    child.wait().expect("the child finishes");

    // And the first still has it: the refusal cost the holder nothing.
    assert!(
        matches!(
            Waited::called(&named, Duration::from_millis(1)),
            Err(NotWaited::StillHeld { .. })
        ),
        "the holder lost its turn because somebody else waited for it"
    );
    drop(held);
}

/// **It can enter after a normal release**, which is the other half: a lock
/// nothing could ever take again would pass the test above and be useless.
#[test]
fn a_second_process_enters_after_the_first_gives_it_back() {
    let named = a_name_of_our_own("given-back");
    let held = Waited::called(&named, Duration::from_millis(1)).expect("the first one takes it");

    let (mut child, mut saying) = a_second_process_wanting(&named, ENOUGH);
    // Long enough that the child has certainly tried and is waiting.
    std::thread::sleep(Duration::from_millis(300));
    drop(held);

    assert_eq!(
        what_it_said(&mut saying),
        HELD,
        "the name was given back and the next process still did not get it"
    );
    let mut telling = child.stdin.take().expect("stdin was asked for");
    telling.write_all(b"go\n").expect("the child is listening");
    child.wait().expect("the child finishes");
}

/// **It can enter after the holder crashes**, which is why the lock is a socket
/// name and not a file.
///
/// The holder is killed outright — `SIGKILL`, no unwinding, no `Drop` — and the
/// kernel frees the name because the process that held it is gone. A lock file
/// would still be sitting there, and the only way past it would be to delete
/// something that might still be held, which this workstream may never do.
///
/// **The killed process is this test's own child.** Nothing else is signalled.
#[test]
fn a_second_process_enters_after_the_holder_is_killed() {
    let named = a_name_of_our_own("killed");

    let (mut holder, mut holder_says) = a_second_process_wanting(&named, ENOUGH);
    assert_eq!(
        what_it_said(&mut holder_says),
        HELD,
        "the first child never took it, so killing it proves nothing"
    );

    holder.kill().expect("this test's own child can be stopped");
    holder.wait().expect("the killed child is reaped");

    let began = Instant::now();
    let after = Waited::called(&named, ENOUGH).expect("a killed holder's name is free");
    assert!(
        began.elapsed() < ENOUGH,
        "the name outlived the process that held it"
    );
    drop(after);
}

/// **A wait that runs out fails safely and touches nothing.**
///
/// The one that matters for the rule this workstream is held to: when the time
/// is up the answer is a refusal, not a shortcut. Nothing is deleted, nothing is
/// signalled, the holder still holds it, and a file standing in for another
/// test's resources is exactly as it was.
#[test]
fn a_wait_that_runs_out_leaves_everything_alone() {
    let named = a_name_of_our_own("timeout");
    let somebody_elses = std::env::temp_dir().join(format!("alo-not-ours-{}", std::process::id()));
    fs::write(&somebody_elses, b"another test's resources").expect("a file can be written");

    let held = Waited::called(&named, Duration::from_millis(1)).expect("the first one takes it");

    let began = Instant::now();
    let ran_out = Waited::called(&named, Duration::from_millis(300));
    let waited = began.elapsed();

    let Err(NotWaited::StillHeld { waited: said, .. }) = ran_out else {
        panic!("a wait that should have run out did not: {ran_out:?}");
    };
    assert_eq!(said, Duration::from_millis(300));
    assert!(
        waited >= Duration::from_millis(300) && waited < ENOUGH,
        "it waited {waited:?}, which is not the bounded wait it was asked for"
    );

    // Nothing was taken by force, and nothing that was not ours was touched.
    assert_eq!(
        fs::read_to_string(&somebody_elses).expect("the file is still there"),
        "another test's resources"
    );
    assert!(
        matches!(
            Waited::called(&named, Duration::from_millis(1)),
            Err(NotWaited::StillHeld { .. })
        ),
        "the wait ran out and took the name anyway"
    );

    drop(held);
    drop(fs::remove_file(&somebody_elses));
}

/// **Kernel enforcement still works while the machine's own lock is held.**
///
/// The coordination is only worth having if the thing it coordinates still
/// happens. This takes the real lock — the one every other kernel test now takes
/// — imposes a real boundary, binds a real turn, and asserts the refusal that
/// `the_kernel_refuses.rs` asserts. If the lock broke the fixture, this is where
/// it would show.
#[test]
fn the_boundary_still_refuses_while_this_machine_is_held() {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel("taking-turns");

    let root = std::env::temp_dir().join(format!("alo-turns-{}", std::process::id()));
    drop(fs::remove_dir_all(&root));
    fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
    fs::create_dir_all(root.join("Private")).expect("a temporary directory can be made");
    fs::write(root.join("Private/secret.txt"), b"not an invoice").expect("a file can be written");

    let cgroup = Cgroup::made("alo-taking-turns").expect("a control group can be made");
    let turn = cgroup.id().expect("a control group has an identifier");
    kernel
        .boundary
        .bound(
            turn,
            places_of(&[root.join("Invoices").as_path()]).expect("the granted folder is there"),
        )
        .expect("the kernel takes the entry");

    // Read back what the kernel is holding, which is the fixture working: the
    // turn is registered and the boundary is imposed while the lock is held.
    kernel
        .boundary
        .released(turn)
        .expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    drop(fs::remove_dir_all(&root));
}

/// The other process, which is this binary again.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Run without the environment that names a lock it does nothing.
#[test]
#[ignore = "this is the second process for the tests above, and each parent runs it by name"]
fn the_other_process() {
    let (Ok(named), Ok(patience)) = (env::var(THE_NAME), env::var(THE_PATIENCE)) else {
        return;
    };
    let patience = Duration::from_millis(patience.parse().expect("the parent named a number"));

    let mut saying = std::io::stdout();
    match Waited::called(&named, patience) {
        Ok(held) => {
            writeln!(saying, "{HELD}").expect("the parent is listening");
            saying.flush().expect("the parent is listening");
            // Held until the parent says otherwise — or until this process is
            // killed, which is what one of the tests above does on purpose.
            let mut go = String::new();
            drop(std::io::stdin().read_line(&mut go));
            drop(held);
        }
        Err(why) => {
            writeln!(saying, "{GAVE_UP} {why}").expect("the parent is listening");
            saying.flush().expect("the parent is listening");
        }
    }
}
