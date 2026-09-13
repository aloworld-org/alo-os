//! What a bound turn **makes** — a file, with an open or without one, a
//! directory, a symbolic link — and a directory it removes, is decided by the
//! kernel, and decided by where the name is.
//!
//! `the_kernel_refuses.rs` covers what a turn opens,
//! `the_kernel_refuses_a_rename.rs` what it moves,
//! `the_kernel_refuses_a_delete_or_a_link.rs` what it removes and links, and
//! `the_kernel_refuses_an_attribute_change.rs` what it changes about a file
//! that is not its contents. What was left was measured rather than assumed,
//! in `what_a_bound_turn_can_still_change.rs`: with a turn bound to one
//! folder and the real programme loaded, a turn could leave an empty file, a
//! file made without an open, a directory and a symbolic link anywhere on the
//! machine, and could remove any empty directory. This file is those
//! reproductions, flipped: every one of them was run against the programme
//! **before** the five hooks below existed and passed in the direction the
//! boundary behaved, and every one is now refused, with the refusal named.
//!
//! # Five hooks, and the question they ask is about the folder
//!
//! `inode_create` is a file made by an open with `O_CREAT`; `inode_mknod` a
//! file made by the call that does not open it; `inode_mkdir` a directory;
//! `inode_symlink` a symbolic link. Each is handed the directory entry for the
//! name being made, and that entry is **negative** — it names a place in a
//! folder rather than a file, and has no inode to be asked about — so each is
//! decided by **the folder the name is made in**, walked upwards exactly as an
//! open would be. That is the answer the rename hook already gives its
//! destination, for the same reason, and it is the folder `alo_files::Reaching`
//! already puts among a turn's places for anything a verb creates.
//! `inode_rmdir` is the one of the five that removes, and the directory it
//! removes exists, so it is decided by its own entry as `inode_unlink` decides
//! a file. `deciding.rs` argues all of it.
//!
//! # Every refusal is beside the thing it must not break
//!
//! Each test makes the same thing **inside** the grant in the same turn and
//! asserts it landed, because a hook that refused every `O_CREAT` would break
//! `alo-files` writing an archive while looking like a boundary. And a process
//! that is not a turn makes every one of these with the programme loaded and
//! is refused none of them — the *nothing outside a turn is affected* half,
//! held here as well as in `the_boundary_decides_and_forgets.rs`.
//!
//! # The link's target is not what is decided
//!
//! A symbolic link inside the grant may point at a file outside it, and is
//! allowed to: a link is a name, and where it leads is decided the moment
//! somebody opens through it, by `file_open` on the file it reaches. The
//! symlink test makes exactly that link inside the grant, then reads through
//! it from inside the turn and is refused — which is the measurement that the
//! allowance buys the turn nothing.
//!
//! # This runs as root, and that is the point rather than a flaw
//!
//! The ordinary permission bits refuse nothing to root, so every *allowed*
//! below is the boundary's own answer and every *refused* is the boundary's
//! own refusal. On a real machine the bits are still there underneath.
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
const THE_CGROUP: &str = "ALO_MAKING_CGROUP";

/// Which thing the child is being asked to make.
const THE_SUBJECT: &str = "ALO_MAKING_SUBJECT";

/// The file nobody granted, which the child is refused an open of first.
const THE_CONTROL: &str = "ALO_MAKING_CONTROL";

/// The name the child tries to make, or remove, outside the grant.
const THE_OUTSIDE: &str = "ALO_MAKING_OUTSIDE";

/// The name the child makes, or removes, inside the grant.
const THE_INSIDE: &str = "ALO_MAKING_INSIDE";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about the control, followed by `allowed` or a number.
const THE_CONTROL_WENT: &str = "alo:control ";

/// What the child says about the making outside the grant.
const THE_OUTSIDE_WENT: &str = "alo:outside ";

/// What the child says about the same making inside the grant.
const THE_INSIDE_WENT: &str = "alo:inside ";

/// What the child says about using what it made inside the grant.
const THE_AFTER_WENT: &str = "alo:after ";

/// What is in the file nobody granted.
const A_SECRET: &str = "not an invoice";

/// What a turn puts into a file it made inside the grant.
const CONTENTS: &str = "contents from inside a turn";

/// `EACCES`, as every Unix numbers it.
const REFUSED: i32 = 13;

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
    /// The open of the file nobody granted, which must be refused.
    control: Outcome,

    /// The making outside the grant.
    outside: Outcome,

    /// The same making inside the grant.
    inside: Outcome,

    /// The use of what was made inside the grant.
    after: Outcome,
}

/// One thing a turn makes, or removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Making {
    /// A file, by opening it with `O_CREAT` — which is what `alo-files` does
    /// when it writes an archive.
    Create,

    /// A file, by the call that makes one without opening it.
    Mknod,

    /// A directory.
    Mkdir,

    /// A directory removed — an empty one, because one with anything in it
    /// needs its contents unlinked first, and that is already refused.
    Rmdir,

    /// A symbolic link, pointing at the file nobody granted.
    Symlink,
}

impl Making {
    /// Every one, for the process that is not a turn.
    const ALL: [Self; 5] = [
        Self::Create,
        Self::Mknod,
        Self::Mkdir,
        Self::Rmdir,
        Self::Symlink,
    ];

    /// How the parent tells the child which one.
    const fn named(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Mknod => "mknod",
            Self::Mkdir => "mkdir",
            Self::Rmdir => "rmdir",
            Self::Symlink => "symlink",
        }
    }

    /// The child's word, read back.
    fn from(named: &str) -> Self {
        match named {
            "create" => Self::Create,
            "mknod" => Self::Mknod,
            "mkdir" => Self::Mkdir,
            "rmdir" => Self::Rmdir,
            "symlink" => Self::Symlink,
            other => panic!("the parent asked for a making called `{other}`, which is not one"),
        }
    }

    /// The name this makes, or removes, in whichever folder.
    const fn what_it_is_called(self) -> &'static str {
        match self {
            Self::Create => "made.txt",
            Self::Mknod => "made-without-opening.txt",
            Self::Mkdir => "made",
            Self::Rmdir => "empty",
            Self::Symlink => "notes.txt",
        }
    }

    /// Whether this removes rather than makes.
    const fn removes(self) -> bool {
        matches!(self, Self::Rmdir)
    }
}

/// Make, or remove, one thing at this path.
///
/// `target` is what a link points at, and is the file nobody granted for
/// every link here — because the interesting measurement is a link inside
/// the grant leading outside it.
fn making(what: Making, at: &Path, target: &Path) -> std::io::Result<()> {
    match what {
        Making::Create => fs::File::create(at).map(drop),
        Making::Mknod => rustix::fs::mknodat(
            rustix::fs::CWD,
            at,
            rustix::fs::FileType::RegularFile,
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
            0,
        )
        .map_err(|why| std::io::Error::from_raw_os_error(why.raw_os_error())),
        Making::Mkdir => fs::create_dir(at),
        Making::Rmdir => fs::remove_dir(at),
        Making::Symlink => std::os::unix::fs::symlink(target, at),
    }
}

/// Use what was made inside the grant, the way a turn would.
///
/// Bytes into a file, a file into a directory, the directory made again
/// where one was removed — each an allowance the boundary must not break —
/// and a read through the link, which leads outside the grant and must be
/// refused.
fn using(what: Making, made: &Path) -> std::io::Result<()> {
    match what {
        Making::Create | Making::Mknod => fs::write(made, CONTENTS),
        Making::Mkdir => fs::write(made.join("inside.txt"), CONTENTS),
        Making::Rmdir => fs::create_dir(made),
        Making::Symlink => fs::read_to_string(made).map(drop),
    }
}

/// A granted folder with a file in it, and a private one nobody granted holding
/// something worth protecting.
struct AMachine {
    /// Everything this test made, for taking away afterwards.
    root: PathBuf,

    /// The one folder the turn is bound to.
    granted: PathBuf,

    /// A folder nobody granted.
    private: PathBuf,

    /// A file in it, with contents the tests assert are undisturbed.
    secret: PathBuf,
}

impl AMachine {
    /// One, named after the test that made it, ready for this making: what
    /// the making removes is put there first, in both folders, from outside
    /// any turn.
    fn ready_for(what: &str, making: Making) -> Self {
        let root = PathBuf::from("/tmp").join(format!("alo-making-{}-{what}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for folder in ["Invoices", "Private"] {
            fs::create_dir_all(root.join(folder)).expect("a temporary directory can be made");
        }
        fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
        let secret = root.join("Private/secret.txt");
        fs::write(&secret, A_SECRET).expect("a file can be written");
        let machine = Self {
            granted: root.join("Invoices"),
            private: root.join("Private"),
            secret,
            root,
        };
        if making.removes() {
            for folder in [&machine.granted, &machine.private] {
                fs::create_dir(folder.join(making.what_it_is_called()))
                    .expect("what the turn will try to remove can be made first");
            }
        }
        machine
    }

    /// Where this making goes outside the grant.
    fn outside(&self, making: Making) -> PathBuf {
        self.private.join(making.what_it_is_called())
    }

    /// Where the same making goes inside the grant.
    fn inside(&self, making: Making) -> PathBuf {
        self.granted.join(making.what_it_is_called())
    }

    /// Take it away, whatever the test found.
    fn taken_away(&self) {
        let _ = fs::remove_dir_all(&self.root);
    }

    /// **Nothing was made or removed outside the grant**, and the secret is
    /// as it was.
    fn nothing_happened_outside(&self, making: Making) {
        let outside = self.outside(making);
        if making.removes() {
            assert!(
                outside.is_dir(),
                "the directory a bound turn was refused the removal of is gone anyway"
            );
        } else {
            assert!(
                outside.symlink_metadata().is_err(),
                "a bound turn was refused a {making:?} outside its grant and {} is there anyway, \
                 which is the empty name the reproduction used to find: the refusal came from \
                 `file_open` after the create, not from the hook on the create",
                outside.display()
            );
        }
        assert!(self.private.is_dir(), "the private folder is gone");
        assert_eq!(
            fs::read_to_string(&self.secret).expect("the secret is readable"),
            A_SECRET,
            "the secret was disturbed"
        );
    }

    /// **The making inside the grant landed**, read from outside any turn.
    fn it_happened_inside(&self, making: Making) {
        let inside = self.inside(making);
        let found = inside.symlink_metadata();
        match making {
            Making::Create | Making::Mknod => {
                let found = found.expect("the file a bound turn made inside its grant is there");
                assert!(found.file_type().is_file(), "what was made is not a file");
                assert_eq!(
                    fs::read_to_string(&inside).expect("the file is readable"),
                    CONTENTS,
                    "the file a bound turn made inside its grant holds nothing, so the allowance \
                     was of a name and not of a file"
                );
            }
            Making::Mkdir => {
                assert!(
                    found.is_ok_and(|found| found.is_dir()),
                    "the directory a bound turn made inside its grant is not there"
                );
                assert_eq!(
                    fs::read_to_string(inside.join("inside.txt")).expect("the file is readable"),
                    CONTENTS,
                    "the file a bound turn made in the directory it made holds nothing"
                );
            }
            Making::Rmdir => assert!(
                found.is_ok_and(|found| found.is_dir()),
                "the directory a bound turn removed inside its grant was not made again, so \
                 either the removal or the making after it was refused"
            ),
            Making::Symlink => {
                let found = found.expect("the link a bound turn made inside its grant is there");
                assert!(
                    found.file_type().is_symlink(),
                    "what was made is not a link"
                );
                assert_eq!(
                    fs::read_to_string(&inside).expect("the link can be followed outside a turn"),
                    A_SECRET,
                    "outside the turn the link leads to the secret, and if it did not the refusal \
                     of the read through it would be about a broken link rather than the boundary"
                );
            }
        }
    }
}

/// One turn: a cgroup, a bound over one folder, and a child inside it making
/// one thing outside the bound and the same thing inside.
fn a_bound_turn(named: &str, machine: &AMachine, making: Making) -> Went {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(named);
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
        .env(THE_SUBJECT, making.named())
        .env(THE_CONTROL, &machine.secret)
        .env(THE_OUTSIDE, machine.outside(making))
        .env(THE_INSIDE, machine.inside(making))
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
        outside: said_about(&mut saying, THE_OUTSIDE_WENT),
        inside: said_about(&mut saying, THE_INSIDE_WENT),
        after: said_about(&mut saying, THE_AFTER_WENT),
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

/// The whole of one measurement: the boundary was in force, the making
/// outside the grant was refused at the syscall with nothing made or removed,
/// the same making inside it landed and could be used, and the file nobody
/// granted is as it was.
fn refused_outside_and_allowed_inside(what: &str, making: Making) -> Went {
    let machine = AMachine::ready_for(what, making);
    let went = a_bound_turn(&format!("alo-making-{what}"), &machine, making);

    assert_eq!(
        went.control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so nothing this turn did means anything"
    );
    assert_eq!(
        went.outside,
        Outcome::Refused(REFUSED),
        "a bound turn made a {making:?} outside its grant. The five hooks `inode_create`, \
         `inode_mknod`, `inode_mkdir`, `inode_rmdir` and `inode_symlink` in \
         crates/alo-bounding-kernel/src/kernel.rs are what refuse this, and `decide_making` \
         in deciding.rs — or `decide_delete`, for a directory removed — is what they ask"
    );
    assert_eq!(
        went.inside,
        Outcome::Allowed,
        "a bound turn was refused a {making:?} inside the folder it was granted — this is a \
         regression rather than a gap being closed, and it would break a verb that writes an \
         archive into its grant"
    );
    machine.nothing_happened_outside(making);
    machine.it_happened_inside(making);
    machine.taken_away();
    went
}

/// **A bound turn cannot make a file outside its grant by opening one**, and
/// nothing is left behind.
///
/// Until 2026-09-13 this was the sharpest of the reproductions: the open was
/// refused, as it always had been, and the empty file was there anyway,
/// because `inode_create` ran before `file_open` and nothing decided at it.
/// The refusal number is the same before and after; what changed is that the
/// name is not there, and that is what the parent looks for. Inside the
/// grant the same `O_CREAT` open is what `alo-files` makes when it writes an
/// archive, and it lands and takes bytes.
#[test]
fn a_file_made_by_opening_is_inside_the_grant() {
    let went = refused_outside_and_allowed_inside("create", Making::Create);
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a bound turn was refused a write into the file it had just made inside its grant"
    );
}

/// **A bound turn cannot make a file outside its grant without opening it
/// either.** `mknod(2)` opens nothing, so until 2026-09-13 it met no hook at
/// all and the call succeeded; it is refused now, and the file inside the
/// grant is made and takes bytes.
#[test]
fn a_file_made_without_opening_is_inside_the_grant() {
    let went = refused_outside_and_allowed_inside("mknod", Making::Mknod);
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a bound turn was refused a write into the file it had just made inside its grant"
    );
}

/// **A bound turn cannot make a directory outside its grant.** Inside it, the
/// directory is made and a file is made in it, because a directory a turn may
/// make is one it may fill.
#[test]
fn a_directory_made_is_inside_the_grant() {
    let went = refused_outside_and_allowed_inside("mkdir", Making::Mkdir);
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a bound turn was refused a file inside the directory it had just made inside its grant"
    );
}

/// **A bound turn cannot remove an empty directory outside its grant.** The
/// one of the five that removes, decided by the directory's own entry as a
/// file unlinked is. Inside the grant the directory goes, and is made again
/// afterwards to show the two hooks agree about the same folder.
#[test]
fn a_directory_removed_is_inside_the_grant() {
    let went = refused_outside_and_allowed_inside("rmdir", Making::Rmdir);
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a bound turn was refused making again the directory it had just removed inside its \
         grant"
    );
}

/// **A bound turn cannot make a symbolic link outside its grant, and a link
/// it makes inside its grant buys it nothing.** The link inside points at the
/// file nobody granted; the turn is allowed to make it, because a link is a
/// name and the hook decides where the name goes, and is refused reading
/// through it, because that is `file_open` on the file it leads to. Outside
/// the turn the link leads to the secret, which the parent checks so the
/// refusal is the boundary's and not a broken link's.
#[test]
fn a_symbolic_link_made_is_inside_the_grant_and_leads_nowhere_the_turn_can_go() {
    let went = refused_outside_and_allowed_inside("symlink", Making::Symlink);
    assert_eq!(
        went.after,
        Outcome::Refused(REFUSED),
        "a bound turn read a file nobody granted it through a link it made inside its grant, \
         which is contents leaving a grant — `file_open` walks from the file an open reaches, \
         never from the link"
    );
}

/// **A process that is not a turn makes what it always could.** Every one
/// of the makings above, by a process in no turn's control group with the
/// programme loaded, and every one of them lands — the boundary looks up a
/// control group, misses, and steps aside.
#[test]
fn a_process_that_is_not_a_turn_makes_what_it_always_could() {
    let _order = on_this_kernel::one_at_a_time();
    let _kernel = AsAMachineHasIt::on_this_kernel("alo-making-not-a-turn");
    for what in Making::ALL {
        let machine = AMachine::ready_for(&format!("not-a-turn-{}", what.named()), what);
        let inside = machine.inside(what);
        let made = making(what, &inside, &machine.secret);
        assert!(
            made.is_ok(),
            "a process that is not a turn was refused a {what:?}: {made:?}. The boundary is \
             deciding about something that is not in any turn's control group"
        );
        let used = using(what, &inside);
        assert!(
            used.is_ok(),
            "a process that is not a turn was refused the use of what it made: {used:?}"
        );
        machine.it_happened_inside(what);
        machine.taken_away();
    }
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

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and each parent asks for it by
/// name. Run without the environment that names a cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the tests above, and each parent runs it by name"]
fn the_work_a_turn_does() {
    let (Ok(cgroup), Ok(subject), Ok(control), Ok(outside), Ok(inside)) = (
        env::var(THE_CGROUP),
        env::var(THE_SUBJECT),
        env::var(THE_CONTROL),
        env::var(THE_OUTSIDE),
        env::var(THE_INSIDE),
    ) else {
        return;
    };
    let what = Making::from(&subject);
    let (control, outside, inside) = (
        PathBuf::from(control),
        PathBuf::from(outside),
        PathBuf::from(inside),
    );

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
    let refused = went(fs::File::open(&control).map(drop));
    writeln!(saying, "{THE_CONTROL_WENT}{refused}").expect("the parent is listening");

    let made_outside = went(making(what, &outside, &control));
    writeln!(saying, "{THE_OUTSIDE_WENT}{made_outside}").expect("the parent is listening");

    let made_inside = went(making(what, &inside, &control));
    writeln!(saying, "{THE_INSIDE_WENT}{made_inside}").expect("the parent is listening");

    let used = went(using(what, &inside));
    writeln!(saying, "{THE_AFTER_WENT}{used}").expect("the parent is listening");
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
