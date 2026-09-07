//! The two mutations left after renames were enforced: **removing a name**, and
//! **giving a file a second one**.
//!
//! `the_kernel_refuses.rs` covers what a turn opens and
//! `the_kernel_refuses_a_rename.rs` what it moves. What was left was measured
//! rather than assumed, the way the rename gap was: with a turn bound to one
//! folder and the real programme loaded, a turn could **delete a file nobody
//! granted** and could **hard-link one into a folder somebody did**. Neither is
//! something any of the six verbs does, which is exactly why the boundary is
//! the right place for it — ADR 0013's floor under a verb with a bug in it,
//! rather than the capability model repeated.
//!
//! # The hard link is the one with history
//!
//! `docs/quirks.md` records that a hard link inside a granted folder is inside
//! every path check, because the granted name genuinely is a name for that
//! file, and `alo-files` answers it by refusing to *read* a file the machine
//! knows by more than one name. This hook answers the other half: a turn cannot
//! **make** the second name in the first place.
//!
//! Neither replaces the other. A link made by somebody else, before the turn
//! began, is still not something the kernel can see the wrongness of — the
//! reading rule is what covers that. This covers the turn making one itself.
//!
//! # What is checked, and what is deliberately not
//!
//! A delete is judged by **the entry being removed**, and a link by **the file
//! being linked and the folder the new name is made in** — the same shape as a
//! rename, and for the same reason: the folder a file sits in is not always a
//! place its call named, so judging by the parent would refuse work the grants
//! allow. `deciding.rs` argues both.
//!
//! **Inside the bound is not the same as authorised**, and this file is not
//! claiming otherwise. `alo-capability` decides; there is no verb that deletes
//! and no verb that links, so a turn doing either is a bug in a verb, and what
//! the kernel adds is that such a bug cannot reach outside the call's own
//! places. One legitimate delete does exist and is tested below: the archive
//! verb removes its own half-written file when a write fails.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! The same as its siblings, for the same reasons, and it fails loudly on a
//! machine without them rather than skipping itself.

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
const THE_CGROUP: &str = "ALO_MUTATION_TEST_CGROUP";

/// Which of the two things the child is to try.
const THE_DOING: &str = "ALO_MUTATION_TEST_DOING";

/// The file the child acts on.
const THE_FROM: &str = "ALO_MUTATION_TEST_FROM";

/// The name a link would be made at, when there is one.
const THE_TO: &str = "ALO_MUTATION_TEST_TO";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says when the machine let it.
const DID_IT: &str = "alo:did-it";

/// What the child says when it was refused, followed by the number.
const REFUSED: &str = "alo:refused ";

/// What the child is asked to do.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Doing {
    /// Remove a name.
    Deleting,

    /// Give a file a second name.
    Linking,
}

impl Doing {
    /// What the parent writes into the environment.
    fn written(self) -> &'static str {
        match self {
            Self::Deleting => "delete",
            Self::Linking => "link",
        }
    }
}

/// Whether the turn is bound when the child acts.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Still {
    /// The entry is in the map: the boundary applies.
    Bound,

    /// The cgroup is not a turn at all, which is every other process.
    NotATurn,
}

/// What the child made of what it tried.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The machine allowed it.
    DidIt,

    /// The machine refused, with the number it gave.
    Refused(i32),
}

/// A granted folder with a file in it, a second granted folder, and a private
/// one nobody granted holding a file worth protecting.
fn a_machine_with_something_worth_protecting(what: &str) -> PathBuf {
    let root = PathBuf::from("/tmp").join(format!("alo-mutation-{}-{what}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for folder in ["Invoices", "Archive", "Private"] {
        fs::create_dir_all(root.join(folder)).expect("a temporary directory can be made");
    }
    fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
    fs::write(root.join("Private/secret.txt"), b"not an invoice").expect("a file can be written");
    root
}

/// Runs one turn: a cgroup, a grant, a child process inside it, and one act.
fn a_turn(
    named: &str,
    granted: &[&Path],
    doing: Doing,
    from: &Path,
    to: &Path,
    still: Still,
) -> Outcome {
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
        .env(THE_DOING, doing.written())
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
        assert_eq!(said, DID_IT, "the child said something unexpected");
        Outcome::DidIt
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

/// **A turn cannot delete a file nobody granted it.**
///
/// Nothing in the six verbs deletes anything, so a turn doing this is a verb
/// with a bug in it — and ADR 0013's whole argument is that such a bug should
/// meet the machine rather than only our own code. It used to succeed.
#[test]
fn deleting_a_file_nobody_granted_is_refused_by_the_kernel() {
    let machine = a_machine_with_something_worth_protecting("delete-ungranted");
    let secret = machine.join("Private/secret.txt");

    let outcome = a_turn(
        "alo-delete-ungranted",
        &[&machine.join("Invoices")],
        Doing::Deleting,
        &secret,
        Path::new(""),
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::Refused(13),
        "the kernel should have refused this delete with EACCES"
    );
    assert_eq!(
        holds(&secret),
        "not an invoice",
        "the file was deleted, or was disturbed"
    );

    let _ = fs::remove_dir_all(&machine);
}

/// **And it can delete one inside its own reach**, which is not a courtesy —
/// the archive verb does exactly this.
///
/// When writing an archive fails part of the way through, `alo-files` removes
/// the half-written file it made, in the folder the archive was going into,
/// which is a place the call named. A boundary that refused this would break a
/// verb that works today, which is why this test is here rather than a note
/// saying it was thought about.
#[test]
fn deleting_a_file_inside_the_granted_folder_is_allowed() {
    let machine = a_machine_with_something_worth_protecting("delete-granted");
    let half_written = machine.join("Archive/invoices.zip");
    fs::write(&half_written, b"half an archive").expect("a file can be written");

    let outcome = a_turn(
        "alo-delete-granted",
        &[&machine.join("Invoices"), &machine.join("Archive")],
        Doing::Deleting,
        &half_written,
        Path::new(""),
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::DidIt,
        "the boundary refused a verb its own cleanup"
    );
    assert!(!half_written.exists());

    let _ = fs::remove_dir_all(&machine);
}

/// **A turn cannot give a file nobody granted a second name inside a folder
/// somebody did** — the way *in*, and the one with history.
///
/// This is the hard link `docs/quirks.md` describes, made by the turn itself.
/// Afterwards every read of it would be a read of a granted path, which is why
/// `alo-files` refuses to read a file with more than one name. This is the
/// other end of the same problem: the link is not made at all.
#[test]
fn linking_a_file_nobody_granted_into_a_granted_folder_is_refused_by_the_kernel() {
    let machine = a_machine_with_something_worth_protecting("link-in");
    let secret = machine.join("Private/secret.txt");
    let second_name = machine.join("Invoices/notes.txt");

    let outcome = a_turn(
        "alo-link-in",
        &[&machine.join("Invoices")],
        Doing::Linking,
        &secret,
        &second_name,
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::Refused(13),
        "the kernel should have refused this link with EACCES"
    );
    assert!(
        !second_name.exists(),
        "the second name was made in the granted folder anyway"
    );
    assert_eq!(holds(&secret), "not an invoice", "the source was disturbed");

    let _ = fs::remove_dir_all(&machine);
}

/// **And it cannot give a granted file a name in a folder nobody granted** —
/// the way *out*.
///
/// A second name outside the grant is a copy of somebody's document that
/// survives everything the turn does afterwards, and it costs no bytes to make.
#[test]
fn linking_a_granted_file_into_a_folder_nobody_granted_is_refused_by_the_kernel() {
    let machine = a_machine_with_something_worth_protecting("link-out");
    let invoice = machine.join("Invoices/march.pdf");
    let second_name = machine.join("Private/march.pdf");

    let outcome = a_turn(
        "alo-link-out",
        &[&machine.join("Invoices")],
        Doing::Linking,
        &invoice,
        &second_name,
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::Refused(13),
        "the kernel should have refused this link with EACCES"
    );
    assert!(
        !second_name.exists(),
        "the second name was made outside the grant anyway"
    );
    assert_eq!(holds(&invoice), "an invoice", "the source was disturbed");

    let _ = fs::remove_dir_all(&machine);
}

/// **A link with both ends inside the bound is not refused.**
///
/// The test that stops the two above from being a boundary that refuses
/// everything and looks like it works. No verb makes one, so nothing depends on
/// this — what it establishes is that the hook decides by *where*, which is the
/// property the refusals are worth anything because of.
#[test]
fn linking_within_one_granted_folder_is_allowed() {
    let machine = a_machine_with_something_worth_protecting("link-within");
    let invoice = machine.join("Invoices/march.pdf");
    let second_name = machine.join("Invoices/march-copy.pdf");

    let outcome = a_turn(
        "alo-link-within",
        &[&machine.join("Invoices")],
        Doing::Linking,
        &invoice,
        &second_name,
        Still::Bound,
    );

    assert_eq!(
        outcome,
        Outcome::DidIt,
        "the boundary refused a link with both ends inside it"
    );
    assert_eq!(holds(&second_name), "an invoice");

    let _ = fs::remove_dir_all(&machine);
}

/// And the case that is almost every delete on the machine: a process that is
/// not a turn is not bounded by anybody's grant.
#[test]
fn a_process_that_is_not_a_turn_deletes_what_it_always_could() {
    let machine = a_machine_with_something_worth_protecting("not-a-turn-delete");
    let secret = machine.join("Private/secret.txt");

    let outcome = a_turn(
        "alo-delete-elsewhere",
        &[&machine.join("Invoices")],
        Doing::Deleting,
        &secret,
        Path::new(""),
        Still::NotATurn,
    );

    assert_eq!(
        outcome,
        Outcome::DidIt,
        "the boundary refused a delete by something that is not an agent turn"
    );
    assert!(!secret.exists());

    let _ = fs::remove_dir_all(&machine);
}

/// The same for links, because a person's own backup tool makes them all day.
#[test]
fn a_process_that_is_not_a_turn_links_what_it_always_could() {
    let machine = a_machine_with_something_worth_protecting("not-a-turn-link");
    let secret = machine.join("Private/secret.txt");
    let second_name = machine.join("Invoices/notes.txt");

    let outcome = a_turn(
        "alo-link-elsewhere",
        &[&machine.join("Invoices")],
        Doing::Linking,
        &secret,
        &second_name,
        Still::NotATurn,
    );

    assert_eq!(
        outcome,
        Outcome::DidIt,
        "the boundary refused a link by something that is not an agent turn"
    );
    assert_eq!(holds(&second_name), "not an invoice");

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
    let (Ok(cgroup), Ok(doing), Ok(from), Ok(to)) = (
        env::var(THE_CGROUP),
        env::var(THE_DOING),
        env::var(THE_FROM),
        env::var(THE_TO),
    ) else {
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

    // From here the boundary may be in force. Neither of these opens a file, so
    // each is the only thing the kernel is asked about, and the answer goes out
    // on a descriptor opened before this process joined anything.
    let done = if doing == "delete" {
        fs::remove_file(&from)
    } else {
        fs::hard_link(&from, &to)
    };
    let outcome = match done {
        Ok(()) => DID_IT.to_owned(),
        Err(why) => format!("{REFUSED}{}", why.raw_os_error().unwrap_or(0)),
    };
    writeln!(saying, "{outcome}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}
