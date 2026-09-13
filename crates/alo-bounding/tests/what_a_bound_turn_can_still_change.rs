//! What a bound turn can still **change** on a filesystem, reproduced rather
//! than argued.
//!
//! `what_a_bound_turn_can_still_reach.rs` is the same shape for the network.
//! This was the filesystem's list: the mutations no hook of this boundary
//! watched, each one run against the real loaded programme so that the gap was
//! a failing expectation somebody could watch rather than a paragraph in a
//! report — asserted in the direction it behaved, so that the day one was
//! closed its assertion failed and said where to come.
//!
//! # The list, and where it went
//!
//! Twenty-two hooks exist — `file_open`, `file_permission`, `inode_rename`,
//! `inode_unlink`, `inode_link`, `inode_setattr`, `inode_setxattr`,
//! `inode_removexattr`, `inode_set_acl`, `inode_remove_acl`, `file_ioctl`,
//! `inode_create`, `inode_mknod`, `inode_mkdir`, `inode_rmdir`,
//! `inode_symlink`, `inode_getattr`, `inode_getxattr`, `inode_listxattr`,
//! `inode_readlink`, `socket_connect` and `socket_sendmsg` — and every row
//! this file once held is a refusal now. The last four are not mutations at
//! all and were never rows here: what a turn learns *about* a file it may
//! not open — its size, an attribute, the attribute names, where a link
//! points — is measured in `the_kernel_refuses_what_a_turn_reads_about_a_file.rs`,
//! refused outside the grant beside the same question answered inside it. Changing a file's **mode, owner, times,
//! size or attributes** was here until 2026-09-12 and its **inode flags**
//! until 2026-09-13; `the_kernel_refuses_an_attribute_change.rs` is where
//! those reproductions went when they flipped. Making a **symbolic link**,
//! making a **file** with an open or without one, and making and removing a
//! **directory** were here until 2026-09-13, each measured landing outside
//! the grant with a refused open beside it; `the_kernel_refuses_what_a_turn_makes.rs`
//! is where they went, each refused outside the grant beside the same thing
//! made inside it. `docs/quirks.md` carries the list with the date each row
//! closed, and `crates/alo-bounding-kernel/src/deciding.rs` carries it beside
//! the code that decides.
//!
//! # What is left, and why it stays
//!
//! Two things, and neither is a gap. **The legitimate write**: every run
//! writes a file **inside** the granted folder, which is an `inode_create`
//! and a `file_open` the boundary must allow — `alo-files` makes an archive
//! that way, and a boundary that refused it would be a boundary that broke a
//! verb while looking like it worked. It is asserted on every run here as it
//! is in every sibling, and it is what says the boundary is not simply
//! refusing everything. And **starting a program**, which is not a mutation
//! at all but is the question every list of unwatched mutations raises: can a
//! turn run something else to make the calls for it. It cannot, because
//! `execve` opens the file it runs, and that is measured below.
//!
//! # The control comes first, and it is not decoration
//!
//! A turn that changed something proves nothing if the boundary was never
//! applied — an unbounded process changes everything too. So the same child in
//! the same turn is first refused an open of a file nobody granted it, and every
//! result below is thrown away rather than believed unless that happened.
//!
//! # This runs as root, and that is the point rather than a flaw
//!
//! The ordinary permission bits refuse nothing to root, so every *allowed* below
//! is the boundary's own answer and not a courtesy of the mode. On a real
//! machine `alo-agentd` runs as the person and the ordinary bits are still
//! there: the boundary is a floor **under** them, never a replacement.
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
    io::{BufRead as _, BufReader, Write as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alo_bounding::{Cgroup, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_CHANGE_CGROUP";

/// The file the child must be refused, whatever else it manages.
const THE_CONTROL: &str = "ALO_CHANGE_CONTROL";

/// A name inside the granted folder, for the legitimate write.
const THE_INSIDE: &str = "ALO_CHANGE_INSIDE";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about the control, followed by `allowed` or a number.
const THE_CONTROL_WENT: &str = "alo:control ";

/// What the child says about starting a program.
const THE_PROGRAM_WENT: &str = "alo:program ";

/// What the child says about opening the program's own file.
const THE_OPEN_WENT: &str = "alo:open ";

/// What the child says about the legitimate write inside the granted folder.
const THE_INSIDE_WENT: &str = "alo:inside ";

/// What is in the file nobody granted.
const A_SECRET: &str = "not an invoice";

/// What the legitimate write puts inside the granted folder.
const A_LEGITIMATE_WRITE: &str = "an archive being made";

/// A program every Linux has, used as a file rather than for what it does.
const A_PROGRAM: &str = "/bin/true";

/// What one attempt came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The machine allowed it.
    Allowed,

    /// The machine refused, with the number it gave.
    Refused(i32),
}

impl Outcome {
    /// What the child wrote, read back.
    fn from(text: &str) -> Self {
        text.parse().map_or(Self::Allowed, Self::Refused)
    }
}

/// What a whole run came to: the four things the child reports, in order.
#[derive(Debug)]
struct Went {
    /// The open of a file nobody granted, which must be refused.
    control: Outcome,

    /// Starting a program whose file is outside the bound.
    program: Outcome,

    /// Opening that program's file, which is what starting it really is.
    open: Outcome,

    /// The legitimate write inside the granted folder.
    inside: Outcome,
}

/// A granted folder with a file in it, and a private one nobody granted holding
/// something worth protecting.
struct AMachine {
    /// Everything this test made, for taking away afterwards.
    root: PathBuf,

    /// The one folder the turn is bound to.
    granted: PathBuf,

    /// A file in the folder nobody granted, with contents the test asserts
    /// are undisturbed.
    secret: PathBuf,
}

impl AMachine {
    /// One, named after the test that made it.
    fn with_something_worth_protecting(what: &str) -> Self {
        let root = PathBuf::from("/tmp").join(format!("alo-change-{}-{what}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for folder in ["Invoices", "Private"] {
            fs::create_dir_all(root.join(folder)).expect("a temporary directory can be made");
        }
        fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
        let secret = root.join("Private/secret.txt");
        fs::write(&secret, A_SECRET).expect("a file can be written");
        Self {
            granted: root.join("Invoices"),
            secret,
            root,
        }
    }

    /// Take it away, whatever the test found.
    fn taken_away(&self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// One turn: a cgroup, a bound over one folder, and a child inside it.
///
/// The bound is the granted folder and nothing else, so everything but the
/// legitimate write is outside it and every refusal below is the boundary's.
fn a_bound_turn(named: &str, machine: &AMachine) -> Went {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(named);
    let cgroup = Cgroup::made(named).expect("a control group can be made");
    let turn = cgroup.id().expect("a control group has an identifier");
    let inside = machine.granted.join("archive.zip");

    let mut child = Command::new(env::current_exe().expect("a test binary knows where it is"))
        .args([
            "--exact",
            "--ignored",
            "--nocapture",
            "the_work_a_turn_does",
        ])
        .env(THE_CGROUP, cgroup.at())
        .env(THE_CONTROL, &machine.secret)
        .env(THE_INSIDE, &inside)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");

    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");

    // The child joins the cgroup before anything is bound, because joining means
    // opening `cgroup.procs`, which is not inside anybody's grant.
    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the child never reached its cgroup");

    kernel
        .boundary
        .bound(
            turn,
            places_of(&[machine.granted.as_path()]).expect("the granted folder is there"),
        )
        .expect("the kernel takes the entry");

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let went = Went {
        control: said_about(&mut saying, THE_CONTROL_WENT),
        program: said_about(&mut saying, THE_PROGRAM_WENT),
        open: said_about(&mut saying, THE_OPEN_WENT),
        inside: said_about(&mut saying, THE_INSIDE_WENT),
    };

    child.wait().expect("the child finishes");
    kernel
        .boundary
        .released(turn)
        .expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    went
}

/// The next line the child says that begins with this, read as an outcome.
fn said_about(saying: &mut BufReader<std::process::ChildStdout>, about: &str) -> Outcome {
    let said = next_line_from(saying);
    let said = said.trim();
    match said.strip_prefix(about) {
        Some(what) => Outcome::from(what),
        None => panic!("the child said `{said}` where `{about}...` was expected"),
    }
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

/// **A bound turn cannot start a program to make calls for it**, because
/// starting one is an open of the program's own file — and the write inside
/// its grant, which is what an archive is, still lands.
///
/// The hooks decide what a turn can do with the calls it makes itself. The
/// obvious next question is whether it can run something else that makes the
/// calls instead — `truncate`, `chmod`, `rm`, `mkdir` — and the answer is
/// no, for a reason that is not another hook: `execve` opens the file it is
/// going to run, `file_open` is watched, and a program outside the bound is a
/// file outside the bound.
///
/// It is a floor under law 2 rather than the law itself. `alo-capability` is
/// what makes there be no verb that runs a command; this is what a bug in one
/// would meet. A dynamically linked program inside a granted folder would still
/// be refused its loader and its libraries, which are outside it — so this is
/// not a way to make one run either.
///
/// The measurement was an accident of trying to reproduce a `truncate(2)`
/// through the coreutil, which was refused here rather than at the truncate.
/// The truncation itself is reproduced since 2026-09-12 in
/// `the_kernel_refuses_an_attribute_change.rs`, through a descriptor opened
/// before the turn began, and refused there.
#[test]
fn a_turn_cannot_start_a_program_that_is_outside_its_bound() {
    let machine = AMachine::with_something_worth_protecting("program");
    let went = a_bound_turn("alo-change-program", &machine);

    assert_eq!(
        went.control,
        Outcome::Refused(13),
        "the boundary was not in force, so nothing this turn did means anything"
    );
    assert_eq!(
        went.program,
        Outcome::Refused(13),
        "a bound turn started a program whose file is outside its bound, so every mutation a \
         hook does not watch could be made by something the turn ran instead of by the turn"
    );
    assert_eq!(
        went.open,
        Outcome::Refused(13),
        "the program's own file opened, which would mean the refusal above was about something \
         other than the boundary"
    );
    assert_eq!(
        went.inside,
        Outcome::Allowed,
        "a turn was refused a write inside the folder it was granted, which is what \
         `archive_folder` does — this is a regression rather than a gap being closed"
    );
    assert_eq!(
        fs::read_to_string(machine.granted.join("archive.zip")).expect("the archive is readable"),
        A_LEGITIMATE_WRITE,
        "the write inside the grant was allowed and left nothing, so the allowance was of a \
         name and not of a file"
    );
    assert_eq!(
        fs::read_to_string(&machine.secret).expect("the secret is readable"),
        A_SECRET,
        "the secret was disturbed"
    );

    machine.taken_away();
}

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Run without the environment that names a cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the test above, and the parent runs it by name"]
fn the_work_a_turn_does() {
    let (Ok(cgroup), Ok(control), Ok(inside)) = (
        env::var(THE_CGROUP),
        env::var(THE_CONTROL),
        env::var(THE_INSIDE),
    ) else {
        return;
    };
    let inside = PathBuf::from(inside);

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

    // From here the boundary is in force. The control first, because it is what
    // says the boundary is in force at all.
    let control = went(fs::File::open(&control).map(drop));
    writeln!(saying, "{THE_CONTROL_WENT}{control}").expect("the parent is listening");

    // Something else to make the calls this turn cannot, and then the open
    // that starting it really is.
    let program = went(Command::new(A_PROGRAM).status().map(drop));
    writeln!(saying, "{THE_PROGRAM_WENT}{program}").expect("the parent is listening");
    let open = went(fs::File::open(A_PROGRAM).map(drop));
    writeln!(saying, "{THE_OPEN_WENT}{open}").expect("the parent is listening");

    // And the legitimate one: a file written inside the folder this turn was
    // granted, which is what `archive_folder` does and what the boundary must
    // not refuse — an `inode_create` and a `file_open`, both allowed.
    let legitimate = went(fs::write(&inside, A_LEGITIMATE_WRITE));
    writeln!(saying, "{THE_INSIDE_WENT}{legitimate}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}

/// What the machine made of one attempt, as the parent reads it.
fn went(done: std::io::Result<()>) -> String {
    match done {
        Ok(()) => "allowed".to_owned(),
        Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
    }
}
