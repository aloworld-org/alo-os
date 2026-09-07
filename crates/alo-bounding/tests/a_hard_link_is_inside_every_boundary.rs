//! What this boundary makes of a **hard** link, asked of the kernel rather
//! than reasoned about.
//!
//! `alo-files` refuses to read a file the machine knows by more than one name,
//! and the justification for a refusal that will sometimes cost somebody a file
//! they were entitled to is that **nothing else catches it**. The capability
//! model cannot: a hard link inside a granted folder is a second *real* name
//! for one file, so resolving it answers with the granted name and every
//! comparison of paths passes, correctly.
//!
//! The boundary is the other layer, and the claim is that it does not catch it
//! either — because `alo-bounding-kernel` decides an open by walking up from
//! the *file's own* directory entry, and a hard link's entry sits in the
//! granted folder. That is a claim about how the programme works and the whole
//! lesson of the last two of these was that a claim like it gets measured.
//!
//! This is one turn, one granted folder, and one file that lives outside it
//! with a second name inside. **A test that failed here would be good news**:
//! it would mean the kernel refuses this, and `alo-files` could report rather
//! than refuse. It passing is what makes the refusal the only thing standing
//! there.
//!
//! The shape is `the_kernel_refuses.rs`'s and for its reasons: a cgroup holds
//! processes, so the work of a turn is a second process, and it is this binary
//! re-run.

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
const THE_CGROUP: &str = "ALO_HARD_LINK_TEST_CGROUP";

/// Where the child is told which file to open.
const THE_FILE: &str = "ALO_HARD_LINK_TEST_OPEN";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says afterwards, followed by what it read.
const OPENED: &str = "alo:opened ";

/// What the child says when the open was refused, followed by the number.
const REFUSED: &str = "alo:refused ";

/// What the child made of its open.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The file opened, and this is what was in it.
    Opened(String),

    /// The open was refused, with the number the machine gave.
    Refused(i32),
}

/// A granted folder, a private one beside it, and a second name inside the
/// granted folder for the file in the private one.
///
/// Returns the root, the granted folder, and the granted name.
fn a_machine_with_a_second_name_for_something_private(what: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from("/tmp").join(format!("alo-hard-link-{}-{what}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let granted = root.join("Invoices");
    let private = root.join("Private");
    fs::create_dir_all(&granted).expect("a temporary directory can be made");
    fs::create_dir_all(&private).expect("a temporary directory can be made");

    let secret = private.join("secret.txt");
    fs::write(&secret, b"not an invoice").expect("a file can be written");
    let second_name = granted.join("notes.txt");
    fs::hard_link(&secret, &second_name).expect("a hard link can be made");

    (root, granted, second_name)
}

/// Runs one turn: a cgroup, a grant, a child process inside it, and one open.
fn a_turn(named: &str, granted: &[&Path], opening: &Path) -> Outcome {
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

    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the child never reached its cgroup");

    boundary
        .bound(
            turn,
            places_of(granted).expect("the granted folders are there"),
        )
        .expect("the kernel takes the entry");

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let said = next_line_from(&mut saying);
    let said = said.trim();
    let outcome = if let Some(number) = said.strip_prefix(REFUSED) {
        Outcome::Refused(number.parse().expect("the child reports a number"))
    } else if let Some(held) = said.strip_prefix(OPENED) {
        Outcome::Opened(held.to_owned())
    } else {
        panic!("the child said something unexpected: {said}");
    };

    child.wait().expect("the child finishes");
    boundary.released(turn).expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    outcome
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

/// **The control, and it has to come first.** The private file under the name
/// it lives at is outside the grant, and the kernel refuses it — so the
/// boundary really is in force for this turn, and the next test means
/// something.
#[test]
fn the_private_file_under_its_own_name_is_refused_by_the_kernel() {
    let (root, granted, _second_name) =
        a_machine_with_a_second_name_for_something_private("control");
    let outcome = a_turn(
        "alo-hard-link-control",
        &[&granted],
        &root.join("Private/secret.txt"),
    );
    assert_eq!(
        outcome,
        Outcome::Refused(13),
        "the boundary was not in force, so the test below means nothing"
    );
    let _ = fs::remove_dir_all(&root);
}

/// **And the same file, under its name inside the granted folder, opens.**
///
/// One file, two names, one bound turn: refused by one name and read by the
/// other. Nothing is wrong with the boundary — it is doing exactly what ADR
/// 0015 says, which is deciding by where a directory entry sits — and the
/// entry for this name sits in a folder somebody granted.
///
/// What it establishes is that the boundary is not a second answer to this
/// question, so `alo-files`' refusal is the only one there is. If this ever
/// starts failing, that refusal can become a report and `docs/quirks.md`, the
/// contract and `crates/alo-files/src/opening.rs` all change together.
#[test]
fn the_same_file_under_a_granted_name_is_not_refused_by_the_kernel() {
    let (root, granted, second_name) =
        a_machine_with_a_second_name_for_something_private("granted");
    let outcome = a_turn("alo-hard-link-granted", &[&granted], &second_name);
    assert_eq!(
        outcome,
        Outcome::Opened("not an invoice".to_owned()),
        "the kernel refused a hard link inside a granted folder — good news, and \
         alo-files' own refusal can be reconsidered: see docs/quirks.md"
    );
    let _ = fs::remove_dir_all(&root);
}

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Run without the environment that names a cgroup and a file it does
/// nothing at all.
#[test]
#[ignore = "this is the child half of the tests above, and the parent runs it by name"]
fn the_work_a_turn_does() {
    let (Ok(cgroup), Ok(opening)) = (env::var(THE_CGROUP), env::var(THE_FILE)) else {
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

    // From here the boundary is in force. Nothing below opens a file except the
    // one being tested: the answer goes out on a descriptor that was opened
    // before this process joined anything.
    let outcome = match fs::read_to_string(&opening) {
        Ok(held) => format!("{OPENED}{}", held.trim()),
        Err(why) => format!("{REFUSED}{}", why.raw_os_error().unwrap_or(0)),
    };
    writeln!(saying, "{outcome}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}
