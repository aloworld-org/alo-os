//! The two tests queue item 26 exists for, run against the kernel this is
//! running on.
//!
//! > a turn granted a folder opens a file inside it and succeeds; the same turn
//! > reaches for `~/.ssh/id_ed25519` and **the kernel returns `EACCES`** — not
//! > our code, the kernel.
//!
//! Everything else in this repository can be tested by calling a function and
//! looking at what comes back. This cannot: what is being asserted is that a
//! program we did not write, running in a mode we cannot enter, refuses a
//! syscall. So the shape is unusual, and each unusual part is here for a
//! reason.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! Loading a BPF LSM program and making a control group are both root's, `bpf`
//! has to be in the list of security modules the kernel actually *started* —
//! which is not the same question as whether it was compiled in, and is the
//! distinction `docs/hardware.md` exists to make — and since ADR 0018 the
//! programme is **pinned**, which needs a `bpf` filesystem mounted at
//! `/sys/fs/bpf`. Where the boundary comes from is `on_this_kernel/mod.rs`,
//! shared with the other two files here.
//!
//! On a machine without those this fails, loudly, naming what is missing. That
//! is deliberate and it is ADR 0015's own rule: *a turn whose boundary cannot
//! be applied does not run.* A test that quietly skipped itself would report
//! green on every machine in the world, including the ones where the boundary
//! does nothing at all, which is the exact failure this file exists to prevent.
//!
//! # The private key is a real file in a temporary directory
//!
//! The item says `~/.ssh/id_ed25519`, and the file has to exist: a path that is
//! not there fails during lookup with `ENOENT`, before `file_open` is reached,
//! so a test against a missing key would pass without the boundary being
//! consulted at all. Writing a fake private key into somebody's real `~/.ssh`
//! is not something a test may do, so the tree is built under a temporary
//! directory with the same shape and the same names. What is being tested is
//! the refusal, and the refusal is about where the file is rather than what it
//! is called.
//!
//! # The turn is a child process, and it has to be
//!
//! A cgroup holds processes, and a boundary applies to everything in one. If
//! the test put *itself* in the turn's cgroup, then the test harness's own
//! files — the ones it opens to report results — would be outside the grant and
//! the kernel would refuse them, which is a test that breaks the thing
//! reporting it. So the work of the turn is a second process, and it is this
//! same binary re-run: [`the_work_a_turn_does`] is an ignored test that the
//! ordinary run never reaches and that the parent invokes by name.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alo_bounding::{Cgroup, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_BOUNDING_TEST_CGROUP";

/// Where the child is told which file to open.
const THE_FILE: &str = "ALO_BOUNDING_TEST_OPEN";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says afterwards, followed by what the machine said.
const OPENED: &str = "alo:opened";

/// What the child says when the open was refused, followed by the number.
const REFUSED: &str = "alo:refused ";

/// Whether the turn is still bound when the child opens its file.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Still {
    /// The entry is in the map: the boundary applies.
    Bound,

    /// The entry has been taken out again, which is what the end of a turn is.
    Over,
}

/// What the child made of its open.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The file opened.
    Opened,

    /// The open was refused, with the number the machine gave.
    Refused(i32),
}

/// Runs one turn: a cgroup, a grant, a child process inside it, and one open.
fn a_turn(named: &str, granted: &[&Path], opening: &Path, still: Still) -> Outcome {
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
        .env(THE_FILE, opening)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");

    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");

    // The child joins the cgroup before anything is bound, because joining
    // means opening `cgroup.procs`, which is not inside anybody's grant.
    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the child never reached its cgroup");

    boundary
        .bound(
            turn,
            places_of(granted).expect("the granted folders are there"),
        )
        .expect("the kernel takes the entry");
    assert!(
        boundary
            .where_bound(turn)
            .expect("the map can be read")
            .is_some(),
        "the kernel should be holding a bound for this turn"
    );
    if still == Still::Over {
        boundary.released(turn).expect("the kernel takes it back");
    }

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let said = next_line_from(&mut saying);
    let outcome = if let Some(number) = said.trim().strip_prefix(REFUSED) {
        Outcome::Refused(number.parse().expect("the child reports a number"))
    } else {
        assert_eq!(said.trim(), OPENED, "the child said something unexpected");
        Outcome::Opened
    };

    child.wait().expect("the child finishes");
    if still == Still::Bound {
        boundary.released(turn).expect("the kernel takes it back");
    }
    assert_eq!(
        boundary.where_bound(turn).expect("the map can be read"),
        None,
        "the turn is over and the kernel should be holding nothing for it"
    );
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    outcome
}

/// The next line the child says that is meant for us.
///
/// The child is a test binary, so it prints a line about running one test
/// before it prints anything of its own. Everything that is not ours is stepped
/// over rather than parsed.
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

/// A folder that is granted, a file inside it, and a private key that is not.
fn a_machine_with_something_worth_protecting() -> PathBuf {
    let root = PathBuf::from("/tmp").join(format!("alo-bounding-{}", std::process::id()));
    fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
    fs::create_dir_all(root.join(".ssh")).expect("a temporary directory can be made");
    fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
    fs::write(root.join(".ssh/id_ed25519"), b"not a real key").expect("a file can be written");
    root
}

/// The first of the two: a turn granted a folder opens a file inside it, and
/// the kernel lets it.
///
/// This test is also the only thing checking that the two halves agree about
/// what a place *is* — the device number `stat` reports is packed differently
/// from the one the kernel keeps, and a boundary that skipped the conversion
/// would refuse this file while looking perfectly healthy.
#[test]
fn a_turn_granted_a_folder_opens_a_file_inside_it() {
    let machine = a_machine_with_something_worth_protecting();
    let outcome = a_turn(
        "alo-bounding-inside",
        &[&machine.join("Invoices")],
        &machine.join("Invoices/march.pdf"),
        Still::Bound,
    );
    assert_eq!(outcome, Outcome::Opened);
}

/// The second, and the one the whole item is for: the same turn reaches for a
/// private key, and **the kernel** refuses it.
///
/// Nothing in this repository made that decision. `alo-capability` was not
/// asked, no verb was validated, no policy was consulted: a process opened a
/// file and the machine said no.
#[test]
fn the_same_turn_reaching_for_a_private_key_is_refused_by_the_kernel() {
    let machine = a_machine_with_something_worth_protecting();
    let outcome = a_turn(
        "alo-bounding-outside",
        &[&machine.join("Invoices")],
        &machine.join(".ssh/id_ed25519"),
        Still::Bound,
    );
    assert_eq!(
        outcome,
        Outcome::Refused(13),
        "the kernel should have refused this open with EACCES"
    );
}

/// ADR 0015's third line: *the entry is removed, and authority is gone — not
/// revoked later, gone.*
///
/// The same cgroup, the same process, the same file that was refused above —
/// and it opens, because the turn is over.
#[test]
fn when_the_turn_is_over_the_authority_is_gone() {
    let machine = a_machine_with_something_worth_protecting();
    let outcome = a_turn(
        "alo-bounding-over",
        &[&machine.join("Invoices")],
        &machine.join(".ssh/id_ed25519"),
        Still::Over,
    );
    assert_eq!(outcome, Outcome::Opened);
}

/// And the case that is almost every open on the machine: a process that is not
/// a turn is not bounded by anybody's grant.
///
/// This test process is not in the turn's cgroup, so while a turn is bound and
/// refusing the private key, the same file opens here. It is the allow half of
/// *the LSM decides and forgets*; the forgetting half is queue item 27.
#[test]
fn a_process_that_is_not_a_turn_reaches_what_it_always_could() {
    let machine = a_machine_with_something_worth_protecting();
    let key = machine.join(".ssh/id_ed25519");
    let refused = a_turn(
        "alo-bounding-elsewhere",
        &[&machine.join("Invoices")],
        &key,
        Still::Bound,
    );
    assert_eq!(refused, Outcome::Refused(13));
    fs::read(&key).expect("this process is not a turn and may read its own files");
}

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Run without the environment that names a cgroup and a file it does
/// nothing at all, so `cargo test -- --include-ignored` is harmless.
#[test]
#[ignore = "this is the child half of the tests above, and the parent runs it by name"]
fn the_work_a_turn_does() {
    let (Ok(cgroup), Ok(opening)) = (env::var(THE_CGROUP), env::var(THE_FILE)) else {
        return;
    };

    // Joining the cgroup opens a file that is inside nobody's grant, which is
    // why the parent waits until this has happened before it binds anything.
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

    // From here the boundary may be in force. Nothing below opens a file except
    // the one being tested: the answer goes out on a descriptor that was opened
    // before this process joined anything.
    let outcome = match fs::File::open(&opening) {
        Ok(_) => OPENED.to_owned(),
        Err(why) => format!("{REFUSED}{}", why.raw_os_error().unwrap_or(0)),
    };
    writeln!(saying, "{outcome}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}
