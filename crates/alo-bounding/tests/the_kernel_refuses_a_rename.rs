//! The other half of what a turn does to a file, now that the kernel watches
//! it: **moving a name**, refused by the machine rather than by our own code.
//!
//! `the_kernel_refuses.rs` is this file's sibling and covers what a turn
//! *opens*, which until the rename hook was the whole of the boundary. The gap
//! that left was measured rather than argued: with a turn bound to one folder
//! and the real programme loaded, **a plain rename of a file nobody granted
//! succeeded**. ADR 0015 named `inode_rename` beside `file_open` in its own
//! mechanism, and only `file_open` had been built.
//!
//! Two shapes matter and they are opposite ways round:
//!
//! - **an ungranted source**, which is how a file nobody granted gets *into* a
//!   granted folder, where every later read of it is perfectly in bounds;
//! - **an ungranted destination**, which is how something granted gets *out*.
//!
//! Both are refused, and a legitimate move — within one granted folder, or from
//! one granted folder into another — is not. That last one is the test that
//! matters most: a boundary that refused those would be a boundary somebody
//! turns off.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! The same as its sibling, for the same reasons, and it fails loudly on a
//! machine without them rather than skipping itself. Where the boundary comes
//! from is `on_this_kernel/mod.rs`, shared with the other files here.
//!
//! # The turn is a child process, and it has to be
//!
//! A cgroup holds processes, so the work of a turn is a second process and it
//! is this binary re-run. Nothing the child does while bounded opens a file:
//! a rename does not, and the answer goes out on a descriptor opened before it
//! joined anything.

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
const THE_CGROUP: &str = "ALO_RENAME_TEST_CGROUP";

/// Where the child is told the name to move.
const THE_FROM: &str = "ALO_RENAME_TEST_FROM";

/// Where the child is told to move it to.
const THE_TO: &str = "ALO_RENAME_TEST_TO";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says when the rename happened.
const MOVED: &str = "alo:moved";

/// What the child says when it was refused, followed by the number.
const REFUSED: &str = "alo:refused ";

/// Whether the turn is bound when the child moves its file.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Still {
    /// The entry is in the map: the boundary applies.
    Bound,

    /// The cgroup is not a turn at all, which is every other process.
    NotATurn,
}

/// What the child made of its rename.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The name moved.
    Moved,

    /// The machine refused, with the number it gave.
    Refused(i32),
}

/// A granted folder with a file in it, a second granted folder, and a private
/// one that nobody granted holding two files.
///
/// Returns the root. The shape is fixed so every test below reads the same.
fn a_machine_with_something_worth_protecting(what: &str) -> PathBuf {
    let root = PathBuf::from("/tmp").join(format!("alo-rename-{}-{what}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for folder in ["Invoices", "Archive", "Private"] {
        fs::create_dir_all(root.join(folder)).expect("a temporary directory can be made");
    }
    fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
    fs::write(root.join("Private/secret.txt"), b"not an invoice").expect("a file can be written");
    fs::write(root.join("Private/keep.txt"), b"a file that must survive")
        .expect("a file can be written");
    root
}

/// Runs one turn: a cgroup, a grant, a child process inside it, and one rename.
fn a_turn(named: &str, granted: &[&Path], from: &Path, to: &Path, still: Still) -> Outcome {
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
        .env(THE_FROM, from)
        .env(THE_TO, to)
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

    if still == Still::Bound {
        boundary
            .bound(
                turn,
                places_of(granted).expect("the granted folders are there"),
            )
            .expect("the kernel takes the entry");
    }

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let said = next_line_from(&mut saying);
    let said = said.trim();
    let outcome = if let Some(number) = said.strip_prefix(REFUSED) {
        Outcome::Refused(number.parse().expect("the child reports a number"))
    } else {
        assert_eq!(said, MOVED, "the child said something unexpected");
        Outcome::Moved
    };

    child.wait().expect("the child finishes");
    if still == Still::Bound {
        boundary.released(turn).expect("the kernel takes it back");
    }
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

/// What is in a file, for the assertions about nothing having been disturbed.
fn holds(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{} is not readable: {why}", path.display()))
}

/// **An ungranted source is refused**, which is the way *in*.
///
/// A file nobody granted, renamed into a folder somebody did. Every read of it
/// afterwards would be a read of a granted path, and the record of the turn
/// would name only the folder the person granted — which is the *record becomes
/// an observation rather than a claim* line of ADR 0013 failing quietly. It
/// used to succeed.
#[test]
fn a_rename_out_of_a_folder_nobody_granted_is_refused_by_the_kernel() {
    let machine = a_machine_with_something_worth_protecting("ungranted-source");
    let from = machine.join("Private/secret.txt");
    let to = machine.join("Invoices/secret.txt");

    let outcome = a_turn(
        "alo-rename-in",
        &[&machine.join("Invoices")],
        &from,
        &to,
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::Refused(13),
        "the kernel should have refused this rename with EACCES"
    );
    // Nothing moved, and nothing was lost at either end.
    assert_eq!(holds(&from), "not an invoice", "the source was disturbed");
    assert!(
        !to.exists(),
        "the file arrived in the granted folder anyway"
    );
    assert_eq!(
        holds(&machine.join("Private/keep.txt")),
        "a file that must survive"
    );

    let _ = fs::remove_dir_all(&machine);
}

/// **An ungranted destination is refused**, which is the way *out*.
///
/// The file is granted and the folder it is going to is not, so this is a
/// granted document being put somewhere the person never approved. Refused for
/// the same reason and by the same walk, on the other entry.
#[test]
fn a_rename_into_a_folder_nobody_granted_is_refused_by_the_kernel() {
    let machine = a_machine_with_something_worth_protecting("ungranted-destination");
    let from = machine.join("Invoices/march.pdf");
    let to = machine.join("Private/march.pdf");

    let outcome = a_turn(
        "alo-rename-out",
        &[&machine.join("Invoices")],
        &from,
        &to,
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::Refused(13),
        "the kernel should have refused this rename with EACCES"
    );
    assert_eq!(holds(&from), "an invoice", "the source was disturbed");
    assert!(!to.exists(), "the file left the granted folder anyway");

    let _ = fs::remove_dir_all(&machine);
}

/// **A failed rename replaces nothing**, which is the half a refusal could
/// still get wrong.
///
/// The destination is a real file somebody else's, and an ordinary `rename`
/// would replace it. The refusal happens before anything is touched, so it is
/// still there with its own contents afterwards.
#[test]
fn a_refused_rename_leaves_what_was_already_at_the_destination() {
    let machine = a_machine_with_something_worth_protecting("preserved");
    let from = machine.join("Invoices/march.pdf");
    let to = machine.join("Private/keep.txt");

    let outcome = a_turn(
        "alo-rename-preserve",
        &[&machine.join("Invoices")],
        &from,
        &to,
        Still::Bound,
    );

    assert_eq!(outcome, Outcome::Refused(13));
    assert_eq!(holds(&from), "an invoice", "the source was disturbed");
    assert_eq!(
        holds(&to),
        "a file that must survive",
        "a refused rename replaced what was already there"
    );

    let _ = fs::remove_dir_all(&machine);
}

/// **A move inside one granted folder is not refused.**
///
/// This is `rename_file`, and the test that stops the hook above from being a
/// boundary that refuses everything and looks like it works.
#[test]
fn a_rename_within_one_granted_folder_is_allowed() {
    let machine = a_machine_with_something_worth_protecting("within");
    let from = machine.join("Invoices/march.pdf");
    let to = machine.join("Invoices/march-2026.pdf");

    let outcome = a_turn(
        "alo-rename-within",
        &[&machine.join("Invoices")],
        &from,
        &to,
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::Moved,
        "a granted rename was refused inside its own boundary"
    );
    assert!(!from.exists());
    assert_eq!(holds(&to), "an invoice");

    let _ = fs::remove_dir_all(&machine);
}

/// **A move from one granted folder into another is not refused.**
///
/// This is `move_file`, and it is the case that decides whether the hook can be
/// built at all without widening anything: what a turn is bound to for a move
/// is the file and the folder it is going into, and neither the folder it comes
/// *out of* nor any grant had to grow for this to be allowed.
#[test]
fn a_move_between_two_granted_folders_is_allowed() {
    let machine = a_machine_with_something_worth_protecting("between");
    let from = machine.join("Invoices/march.pdf");
    let to = machine.join("Archive/march.pdf");

    let outcome = a_turn(
        "alo-rename-between",
        &[&machine.join("Invoices"), &machine.join("Archive")],
        &from,
        &to,
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::Moved,
        "a granted move was refused inside its own boundary"
    );
    assert!(!from.exists());
    assert_eq!(holds(&to), "an invoice");

    let _ = fs::remove_dir_all(&machine);
}

/// And the case that is almost every rename on the machine: a process that is
/// not a turn is not bounded by anybody's grant.
///
/// The same rename the first test had refused, in a cgroup with no entry in the
/// map — a person's own file manager moving their own file. It is the allow
/// half of *the LSM decides and forgets*.
#[test]
fn a_process_that_is_not_a_turn_moves_what_it_always_could() {
    let machine = a_machine_with_something_worth_protecting("not-a-turn");
    let from = machine.join("Private/secret.txt");
    let to = machine.join("Invoices/secret.txt");

    let outcome = a_turn(
        "alo-rename-elsewhere",
        &[&machine.join("Invoices")],
        &from,
        &to,
        Still::NotATurn,
    );

    assert_eq!(
        outcome,
        Outcome::Moved,
        "the boundary refused a rename by something that is not an agent turn"
    );
    assert_eq!(holds(&to), "not an invoice");

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
    let (Ok(cgroup), Ok(from), Ok(to)) =
        (env::var(THE_CGROUP), env::var(THE_FROM), env::var(THE_TO))
    else {
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

    // From here the boundary may be in force. A rename opens nothing, so this
    // is the only thing the kernel is asked about, and the answer goes out on a
    // descriptor that was opened before this process joined anything.
    let outcome = match fs::rename(&from, &to) {
        Ok(()) => MOVED.to_owned(),
        Err(why) => format!("{REFUSED}{}", why.raw_os_error().unwrap_or(0)),
    };
    writeln!(saying, "{outcome}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}
