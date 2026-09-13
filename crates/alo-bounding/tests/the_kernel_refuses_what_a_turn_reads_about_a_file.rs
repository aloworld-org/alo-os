//! What a bound turn **learns about** a file it may not open — its size,
//! owner, mode and times, the value of an extended attribute, the names of
//! its attributes, and where a symbolic link points — is decided by the
//! kernel, and decided by the file being asked about.
//!
//! Every hook before these decides what a turn does *to* a file:
//! `the_kernel_refuses.rs` what it opens, `the_kernel_refuses_a_rename.rs`
//! what it moves, `the_kernel_refuses_a_delete_or_a_link.rs` what it removes
//! and links, `the_kernel_refuses_an_attribute_change.rs` what it changes
//! about a file that is not its contents, and
//! `the_kernel_refuses_what_a_turn_makes.rs` what it makes. None decided what
//! a turn finds out *about* a file. Inside a bound turn, `stat(2)` on a path
//! outside the grant answered with its size, owner, mode and times;
//! `getxattr(2)` returned the value of a `user.*` attribute, which is
//! somewhere a person's application keeps bytes that are not the file's
//! contents — a comment, an origin, a checksum; `listxattr(2)` returned their
//! names; and `readlink(2)` returned where a link points. A turn refused a
//! folder's listing by `file_permission` could still ask each name in it
//! whether it exists and how big it is, which is *context is offered, never
//! watched* failing by another road. This file is those reproductions,
//! flipped: every one of them was run against the programme **before** the
//! four hooks below existed and passed in the direction the boundary behaved,
//! and every one is now refused, with the refusal named.
//!
//! # Four hooks, and the question they ask is about the file
//!
//! `inode_getattr` is handed a `struct path` — the one an open reaches an
//! entry through as `file_open` does via `f_path` — and is asked by every
//! `stat`, `lstat`, `fstat` and `statx` on the machine; `inode_getxattr`,
//! `inode_listxattr` and `inode_readlink` are handed the directory entry of
//! the file being asked about. The file exists, so this is the question
//! `decide_attribute` already asks and the same walk answers it: the entry,
//! upwards, until a granted place is met or the top of the filesystem is.
//! `deciding.rs` argues all of it, and says why a socket and a pipe are
//! stepped aside from by `inode_getattr` for the reason `file_permission`
//! steps aside from them.
//!
//! # Every refusal is beside the thing it must not break
//!
//! `alo-files` asks `symlink_metadata` of every path it was given before it
//! opens one, and reads a file's size and link count through the descriptor
//! it opened; every one of those paths is among a turn's places, and a hook
//! that refused them would refuse every verb while looking like a boundary.
//! So each test asks the same question **inside** the grant in the same turn
//! and asserts it was answered — with the right answer, not merely without an
//! error. And a process that is not a turn asks every one of these with the
//! programme loaded and is refused none of them.
//!
//! # A descriptor opened before the turn began is asked about too
//!
//! `fstat` reaches `inode_getattr` through the descriptor's own path, so a
//! handle to a file outside the grant that was open before the turn began is
//! refused its size inside the turn, as `file_permission` refuses it a byte.
//! `what_a_turn_inherits.rs` used `fstat` as its proof that an inherited
//! handle was still a handle; it uses `fcntl` now, which asks no hook this
//! boundary sits on, and the `fstat` is measured here instead.
//!
//! # One question is still answered, and it is left standing
//!
//! A file's POSIX access list is read with `getxattr` under the name
//! `system.posix_acl_access`, and since Linux 6.2 that read never reaches
//! `inode_getxattr`: the kernel routes it to `inode_get_acl`, a hook of its
//! own, exactly as it routes the write to `inode_set_acl` past
//! `inode_setxattr`. This boundary does not sit on `inode_get_acl`, so a
//! bound turn refused the names of a file's attributes can still read its
//! access list. That is reproduced below in the direction it behaves, the
//! way task 14 left a file's flags and task 18 was handed them, and
//! `docs/quirks.md` names it with the task that closes it.
//!
//! # This runs as root, and that is the point rather than a flaw
//!
//! The ordinary permission bits refuse nothing to root, so every *answered*
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
    env,
    fs::{self, File},
    io::{BufRead as _, BufReader, Write as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alo_bounding::{Cgroup, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_ASKING_CGROUP";

/// Which question the child is being asked to put.
const THE_SUBJECT: &str = "ALO_ASKING_SUBJECT";

/// The file nobody granted, which the child is refused an open of first, and
/// which every link outside the grant points at.
const THE_CONTROL: &str = "ALO_ASKING_CONTROL";

/// The file inside the grant, which every link inside the grant points at.
const THE_INVOICE: &str = "ALO_ASKING_INVOICE";

/// The thing the child asks about outside the grant.
const THE_OUTSIDE: &str = "ALO_ASKING_OUTSIDE";

/// The thing the child asks about inside the grant.
const THE_INSIDE: &str = "ALO_ASKING_INSIDE";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about the control, followed by `allowed` or a number.
const THE_CONTROL_WENT: &str = "alo:control ";

/// What the child says about the question outside the grant.
const THE_OUTSIDE_WENT: &str = "alo:outside ";

/// What the child says about the same question inside the grant.
const THE_INSIDE_WENT: &str = "alo:inside ";

/// What is in the file nobody granted.
const A_SECRET: &str = "not an invoice";

/// What is in the file inside the grant.
const AN_INVOICE: &str = "an invoice";

/// The attribute a person's application put on both files.
const ATTRIBUTE: &str = "user.alo.origin";

/// What it says.
const A_VALUE: &[u8] = b"https://example.invalid/where-it-came-from";

/// The name a file's POSIX access list is read under.
const AN_ACCESS_LIST: &str = "system.posix_acl_access";

/// `EACCES`, as every Unix numbers it.
const REFUSED: i32 = 13;

/// What a question that was answered wrongly is reported as: not a refusal
/// the machine made, and not an answer either.
const A_WRONG_ANSWER: i32 = 0;

/// What one attempt came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The machine answered, and the answer was the right one.
    Answered,

    /// The machine refused, with the number it gave — or answered with the
    /// wrong answer, which is [`A_WRONG_ANSWER`].
    Refused(i32),
}

impl Outcome {
    /// What the child wrote, read back.
    fn from(text: &str) -> Self {
        text.parse().map_or(Self::Answered, Self::Refused)
    }
}

/// What a whole run came to: the three things the child reports, in order.
#[derive(Debug)]
struct Went {
    /// The open of the file nobody granted, which must be refused.
    control: Outcome,

    /// The question outside the grant.
    outside: Outcome,

    /// The same question inside the grant.
    inside: Outcome,
}

/// One thing a turn asks about a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Asking {
    /// Its size, mode, owner and times, by name: `lstat(2)`.
    Stat,

    /// Its size through a descriptor that was open before the turn began:
    /// `fstat(2)`.
    Held,

    /// The value of an extended attribute: `getxattr(2)`.
    Attribute,

    /// The names of its extended attributes: `listxattr(2)`.
    Names,

    /// Where a symbolic link points: `readlink(2)`.
    Link,

    /// Its POSIX access list, which is a `getxattr(2)` the kernel routes to
    /// a hook this boundary does not sit on.
    AccessList,
}

impl Asking {
    /// Every one, for the process that is not a turn.
    const ALL: [Self; 6] = [
        Self::Stat,
        Self::Held,
        Self::Attribute,
        Self::Names,
        Self::Link,
        Self::AccessList,
    ];

    /// How the parent tells the child which one.
    const fn named(self) -> &'static str {
        match self {
            Self::Stat => "stat",
            Self::Held => "held",
            Self::Attribute => "attribute",
            Self::Names => "names",
            Self::Link => "link",
            Self::AccessList => "access-list",
        }
    }

    /// The child's word, read back.
    fn from(named: &str) -> Self {
        match named {
            "stat" => Self::Stat,
            "held" => Self::Held,
            "attribute" => Self::Attribute,
            "names" => Self::Names,
            "link" => Self::Link,
            "access-list" => Self::AccessList,
            other => panic!("the parent asked for a question called `{other}`, which is not one"),
        }
    }

    /// Whether this is asked of a link rather than of the file itself.
    const fn is_about_a_link(self) -> bool {
        matches!(self, Self::Link)
    }
}

/// A file, and what a right answer about it is: what it holds and, when it
/// is a link, where it leads.
struct About<'a> {
    /// The file, or the link, being asked about.
    at: &'a Path,

    /// What the file holds, so a size can be checked.
    holds: &'a str,

    /// Where the link leads, when it is one.
    leads_to: &'a Path,
}

/// A handle opened before the turn began, for the one question that is
/// asked through a descriptor rather than by name.
struct Handles {
    /// To the file outside the grant.
    outside: File,

    /// To the file inside the grant.
    inside: File,
}

/// Put one question, and say whether the answer was the right one.
///
/// A wrong answer is reported as [`A_WRONG_ANSWER`] rather than as an
/// answer, because a hook that let a `stat` through and had it answer with
/// the wrong size would be worse than one that refused it.
fn asking(what: Asking, about: &About<'_>, held: Option<&File>) -> std::io::Result<()> {
    let wrong = |why: String| std::io::Error::other(why);
    match what {
        Asking::Stat => {
            let found = fs::symlink_metadata(about.at)?;
            (found.len() == about.holds.len() as u64)
                .then_some(())
                .ok_or_else(|| wrong(format!("stat said {} bytes", found.len())))
        }
        Asking::Held => {
            let found = held
                .expect("a handle was opened before the turn")
                .metadata()?;
            (found.len() == about.holds.len() as u64)
                .then_some(())
                .ok_or_else(|| wrong(format!("fstat said {} bytes", found.len())))
        }
        Asking::Attribute => {
            let mut read = [0u8; 128];
            let length = rustix::fs::getxattr(about.at, ATTRIBUTE, &mut read[..])
                .map_err(|why| std::io::Error::from_raw_os_error(why.raw_os_error()))?;
            (read.get(..length) == Some(A_VALUE))
                .then_some(())
                .ok_or_else(|| wrong("getxattr answered with a different value".to_owned()))
        }
        Asking::Names => {
            let mut read = [0u8; 512];
            let length = rustix::fs::listxattr(about.at, &mut read[..])
                .map_err(|why| std::io::Error::from_raw_os_error(why.raw_os_error()))?;
            let names = read.get(..length).unwrap_or_default();
            names
                .split(|byte| *byte == 0)
                .any(|name| name == ATTRIBUTE.as_bytes())
                .then_some(())
                .ok_or_else(|| wrong("listxattr answered without the attribute".to_owned()))
        }
        Asking::Link => {
            let leads_to = fs::read_link(about.at)?;
            (leads_to == about.leads_to)
                .then_some(())
                .ok_or_else(|| wrong(format!("readlink said {}", leads_to.display())))
        }
        Asking::AccessList => {
            let mut read = [0u8; 128];
            let length = rustix::fs::getxattr(about.at, AN_ACCESS_LIST, &mut read[..])
                .map_err(|why| std::io::Error::from_raw_os_error(why.raw_os_error()))?;
            (read.get(..length) == Some(an_access_list().as_slice()))
                .then_some(())
                .ok_or_else(|| {
                    wrong("the access list read back is not the one put there".to_owned())
                })
        }
    }
}

/// A valid POSIX access list the kernel has to **keep**: version two, then
/// the owner, one named user, the group, a mask and everybody else.
///
/// The named user and the mask are what make it more than the mode bits.
/// An access list that says no more than the mode is not stored at all —
/// the kernel folds it into the mode and a read of it answers `ENODATA` —
/// so the least list `the_boundary_decides_and_forgets.rs` puts on its
/// files would leave nothing here for a turn to be answered.
fn an_access_list() -> Vec<u8> {
    let mut list = Vec::with_capacity(44);
    list.extend_from_slice(&2u32.to_le_bytes());
    for (tag, permissions, id) in [
        (0x01u16, 6u16, u32::MAX),
        (0x02, 4, 1),
        (0x04, 4, u32::MAX),
        (0x10, 4, u32::MAX),
        (0x20, 0, u32::MAX),
    ] {
        list.extend_from_slice(&tag.to_le_bytes());
        list.extend_from_slice(&permissions.to_le_bytes());
        list.extend_from_slice(&id.to_le_bytes());
    }
    list
}

/// A granted folder with a file in it, and a private one nobody granted holding
/// something worth protecting — both files with an attribute and an access
/// list on them and a link beside them, put there from outside any turn.
struct AMachine {
    /// Everything this test made, for taking away afterwards.
    root: PathBuf,

    /// The one folder the turn is bound to.
    granted: PathBuf,

    /// The file in it.
    invoice: PathBuf,

    /// A file in the folder nobody granted, with contents the tests assert
    /// are undisturbed.
    secret: PathBuf,
}

impl AMachine {
    /// One, named after the test that made it.
    fn ready_for(what: &str) -> Self {
        let root = PathBuf::from("/tmp").join(format!("alo-asking-{}-{what}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for folder in ["Invoices", "Private"] {
            fs::create_dir_all(root.join(folder)).expect("a temporary directory can be made");
        }
        let invoice = root.join("Invoices/march.pdf");
        let secret = root.join("Private/secret.txt");
        for (file, holds) in [(&invoice, AN_INVOICE), (&secret, A_SECRET)] {
            fs::write(file, holds).expect("a file can be written");
            rustix::fs::setxattr(file, ATTRIBUTE, A_VALUE, rustix::fs::XattrFlags::empty())
                .expect("an attribute can be put on a file outside any turn");
            rustix::fs::setxattr(
                file,
                AN_ACCESS_LIST,
                &an_access_list(),
                rustix::fs::XattrFlags::empty(),
            )
            .expect("an access list can be put on a file outside any turn");
            std::os::unix::fs::symlink(file, file.with_file_name("notes.txt"))
                .expect("a link can be made beside a file outside any turn");
        }
        Self {
            granted: root.join("Invoices"),
            invoice,
            secret,
            root,
        }
    }

    /// What this question is asked about outside the grant.
    fn outside(&self, asking: Asking) -> PathBuf {
        if asking.is_about_a_link() {
            self.secret.with_file_name("notes.txt")
        } else {
            self.secret.clone()
        }
    }

    /// What the same question is asked about inside the grant.
    fn inside(&self, asking: Asking) -> PathBuf {
        if asking.is_about_a_link() {
            self.invoice.with_file_name("notes.txt")
        } else {
            self.invoice.clone()
        }
    }

    /// Take it away, whatever the test found.
    fn taken_away(&self) {
        let _ = fs::remove_dir_all(&self.root);
    }

    /// **The file nobody granted is exactly as it was**, read from outside
    /// any turn — a question changes nothing, and that is asserted rather
    /// than assumed.
    fn the_secret_is_undisturbed(&self) {
        assert_eq!(
            fs::read_to_string(&self.secret).expect("the secret is readable"),
            A_SECRET,
            "the secret was disturbed by a question about it"
        );
    }
}

/// One turn: a cgroup, a bound over one folder, and a child inside it putting
/// one question outside the bound and the same question inside.
fn a_bound_turn(named: &str, machine: &AMachine, asking: Asking) -> Went {
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
        .env(THE_SUBJECT, asking.named())
        .env(THE_CONTROL, &machine.secret)
        .env(THE_INVOICE, &machine.invoice)
        .env(THE_OUTSIDE, machine.outside(asking))
        .env(THE_INSIDE, machine.inside(asking))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");

    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");

    // The child joins the cgroup before anything is bound, because joining means
    // opening `cgroup.procs`, which is not inside anybody's grant — and it
    // opens the handles it holds through the turn before that, so they are
    // descriptors that were open before the turn began.
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

/// The whole of one measurement: the boundary was in force, the question
/// outside the grant was refused at the syscall, the same question inside it
/// was answered with the right answer, and the file nobody granted is as it
/// was.
fn refused_outside_and_answered_inside(what: &str, asking: Asking) {
    let machine = AMachine::ready_for(what);
    let went = a_bound_turn(&format!("alo-asking-{what}"), &machine, asking);

    assert_eq!(
        went.control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so nothing this turn did means anything"
    );
    assert_eq!(
        went.outside,
        Outcome::Refused(REFUSED),
        "a bound turn was answered a {asking:?} about a file outside its grant. The four hooks \
         `inode_getattr`, `inode_getxattr`, `inode_listxattr` and `inode_readlink` in \
         crates/alo-bounding-kernel/src/kernel.rs are what refuse this, and `decide_asking` \
         in deciding.rs — or `decide_question`, for the three handed an entry — is what they \
         ask. Before 2026-09-13 this assertion was the other way round and passed"
    );
    assert_eq!(
        went.inside,
        Outcome::Answered,
        "a bound turn was refused a {asking:?} about a file inside the folder it was granted, \
         or answered wrongly — this is a regression rather than a gap being closed, and it \
         would break every verb, because `alo-files` asks this of every path it is given"
    );
    machine.the_secret_is_undisturbed();
    machine.taken_away();
}

/// **A bound turn cannot ask the size, mode, owner or times of a file
/// outside its grant**, and is answered about one inside it. Until
/// 2026-09-13 `stat(2)` met no hook this boundary sat on, so a turn refused a
/// folder's listing could still ask each name in it whether it was there and
/// how big it was.
#[test]
fn a_files_size_mode_owner_and_times_are_inside_the_grant() {
    refused_outside_and_answered_inside("stat", Asking::Stat);
}

/// **A descriptor opened before the turn began is refused its size inside
/// it.** `fstat` reaches the same hook through the descriptor's own path, so
/// the inherited handle `file_permission` refuses a byte is refused its size
/// as well — and a handle to a file inside the grant answers.
#[test]
fn a_descriptor_opened_before_the_turn_is_refused_its_size_inside_it() {
    refused_outside_and_answered_inside("held", Asking::Held);
}

/// **A bound turn cannot read an extended attribute of a file outside its
/// grant.** A `user.*` attribute is a byte somebody put there, and it is
/// answered inside the grant with the value that was put.
#[test]
fn an_extended_attribute_is_inside_the_grant() {
    refused_outside_and_answered_inside("attribute", Asking::Attribute);
}

/// **A bound turn cannot list the attributes of a file outside its grant**,
/// and inside it the list names the attribute that is there.
#[test]
fn the_names_of_a_files_attributes_are_inside_the_grant() {
    refused_outside_and_answered_inside("names", Asking::Names);
}

/// **A bound turn cannot read where a symbolic link outside its grant
/// points**, and inside it the link's target is answered. Where a link leads
/// is somebody's filesystem laid out in words, and a turn that could read
/// every link on the machine could map it without opening a file.
#[test]
fn where_a_symbolic_link_points_is_inside_the_grant() {
    refused_outside_and_answered_inside("link", Asking::Link);
}

/// **A file's access list is not yet inside the grant**, and this test
/// stands until it is.
///
/// Reading `system.posix_acl_access` is a `getxattr(2)` the kernel routes to
/// `inode_get_acl` since Linux 6.2, past `inode_getxattr` — the same routing
/// that made `inode_set_acl` a hook of its own beside `inode_setxattr`. This
/// boundary does not sit on `inode_get_acl`, so a bound turn refused the
/// names of a file's attributes is still answered its access list. Asserted
/// in the direction it behaves, so the day the hook lands this fails and
/// says where to come; `docs/quirks.md` names it under *What a turn reads
/// about a file is inside the grant*, and the kernel-enforcement plan's task
/// 20 is what closes it.
#[test]
fn a_files_access_list_is_not_yet_inside_the_grant() {
    let machine = AMachine::ready_for("access-list");
    let went = a_bound_turn("alo-asking-access-list", &machine, Asking::AccessList);

    assert_eq!(
        went.control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so nothing this turn did means anything"
    );
    assert_eq!(
        went.outside,
        Outcome::Answered,
        "a bound turn was refused the access list of a file outside its grant. That is the gap \
         closing: `inode_get_acl` is on the programme, or the kernel now routes the read \
         through `inode_getxattr`. Flip this assertion into the refusal, move the row in \
         docs/quirks.md, and mark the plan's task"
    );
    assert_eq!(
        went.inside,
        Outcome::Answered,
        "a bound turn was refused the access list of a file inside its grant"
    );
    machine.the_secret_is_undisturbed();
    machine.taken_away();
}

/// **A process that is not a turn is answered what it always was.** Every
/// question above, by a process in no turn's control group with the
/// programme loaded, and every one of them answered — the boundary looks up
/// a control group, misses, and steps aside.
#[test]
fn a_process_that_is_not_a_turn_is_answered_what_it_always_was() {
    let _order = on_this_kernel::one_at_a_time();
    let _kernel = AsAMachineHasIt::on_this_kernel("alo-asking-not-a-turn");
    for what in Asking::ALL {
        let machine = AMachine::ready_for(&format!("not-a-turn-{}", what.named()));
        let inside = machine.inside(what);
        let held = File::open(&machine.invoice).expect("the invoice opens outside any turn");
        let answered = asking(
            what,
            &About {
                at: &inside,
                holds: AN_INVOICE,
                leads_to: &machine.invoice,
            },
            Some(&held),
        );
        assert!(
            answered.is_ok(),
            "a process that is not a turn was refused a {what:?}, or answered wrongly: \
             {answered:?}. The boundary is deciding about something that is not in any turn's \
             control group"
        );
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
    let (Ok(cgroup), Ok(subject), Ok(control), Ok(invoice), Ok(outside), Ok(inside)) = (
        env::var(THE_CGROUP),
        env::var(THE_SUBJECT),
        env::var(THE_CONTROL),
        env::var(THE_INVOICE),
        env::var(THE_OUTSIDE),
        env::var(THE_INSIDE),
    ) else {
        return;
    };
    let what = Asking::from(&subject);
    let (control, invoice, outside, inside) = (
        PathBuf::from(control),
        PathBuf::from(invoice),
        PathBuf::from(outside),
        PathBuf::from(inside),
    );

    // Opened before this process is in any turn, so that by the time the
    // parent binds the turn these are descriptors that were open before it
    // began — the shape `what_a_turn_inherits.rs` measures.
    let handles = Handles {
        outside: File::open(&control).expect("the secret opens before the turn"),
        inside: File::open(&invoice).expect("the invoice opens before the turn"),
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

    // From here the boundary is in force. The control first, because it is what
    // says the boundary is in force at all.
    let refused = went(File::open(&control).map(drop));
    writeln!(saying, "{THE_CONTROL_WENT}{refused}").expect("the parent is listening");

    let asked_outside = went(asking(
        what,
        &About {
            at: &outside,
            holds: A_SECRET,
            leads_to: &control,
        },
        Some(&handles.outside),
    ));
    writeln!(saying, "{THE_OUTSIDE_WENT}{asked_outside}").expect("the parent is listening");

    let asked_inside = went(asking(
        what,
        &About {
            at: &inside,
            holds: AN_INVOICE,
            leads_to: &invoice,
        },
        Some(&handles.inside),
    ));
    writeln!(saying, "{THE_INSIDE_WENT}{asked_inside}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}

/// What the machine made of one attempt, as the parent reads it.
///
/// A refusal is its number; an answer that was wrong is [`A_WRONG_ANSWER`],
/// which is what an error the machine did not make has for a number.
fn went(done: std::io::Result<()>) -> String {
    match done {
        Ok(()) => "answered".to_owned(),
        Err(why) => why.raw_os_error().unwrap_or(A_WRONG_ANSWER).to_string(),
    }
}
