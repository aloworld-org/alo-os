//! What a bound turn can still **change** on a filesystem, reproduced rather
//! than argued.
//!
//! `what_a_bound_turn_can_still_reach.rs` is the same shape for the network.
//! This is the filesystem's list: the mutations no hook of this boundary
//! watches, each one run against the real loaded programme so that the gap is a
//! failing expectation somebody can watch rather than a paragraph in a report.
//!
//! Each is asserted in the direction it behaves **today**. The day one of them is
//! closed, its assertion fails and says where to come and what to change, which
//! is what the rename and delete gaps did before they were closed.
//!
//! # The list, and the property they all share
//!
//! Seven hooks exist — `file_open`, `file_permission`, `inode_rename`,
//! `inode_unlink`, `inode_link`, `socket_connect` and `socket_sendmsg` — and a
//! filesystem has more verbs than the five of those that are about one.
//! Unwatched:
//! making a **symbolic link**, making a **file** — with an open or without one —
//! making and removing a **directory**, and changing a file's **mode, owner,
//! times or extended attributes**. `docs/quirks.md` carries the list with the
//! release that owns closing each, and
//! `crates/alo-bounding-kernel/src/deciding.rs` carries it beside the code that
//! decides.
//!
//! What they have in common is the property the four filesystem hooks were
//! chosen for: **none of them moves a byte of somebody's file past a grant.**
//! That sentence is what each test here holds up. Every one of them does three
//! things in order — it is refused something to prove the boundary is in force,
//! it performs the unwatched mutation, and then it tries the thing that would
//! turn that mutation into contents leaving, and is refused *that*.
//!
//! # The control comes first, and it is not decoration
//!
//! A turn that changed something proves nothing if the boundary was never
//! applied — an unbounded process changes everything too. So the same child in
//! the same turn is first refused an open of a file nobody granted it, and every
//! result below is thrown away rather than believed unless that happened.
//!
//! # And the legitimate path is beside every refusal
//!
//! Every run also writes a file **inside** the granted folder, which is an
//! `inode_create` and a `file_open` the boundary must allow: `alo-files` makes
//! an archive that way, and a boundary that refused it would be a boundary that
//! broke a verb while looking like it worked.
//!
//! # This runs as root, and that is the point rather than a flaw
//!
//! The ordinary permission bits refuse nothing to root, so every *allowed* below
//! is the boundary's own answer and not a courtesy of the mode. On a real
//! machine `alo-agentd` runs as the person and the ordinary bits are still
//! there: the boundary is a floor **under** them, never a replacement, and
//! `docs/quirks.md` says which of these a person's own permissions would stop
//! anyway.
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
    os::unix::fs::PermissionsExt as _,
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

/// Which unwatched mutation the child is being asked to make.
const THE_SUBJECT: &str = "ALO_CHANGE_SUBJECT";

/// The thing the subject acts on, which is always outside the bound.
const THE_WHERE: &str = "ALO_CHANGE_WHERE";

/// The name the subject makes, when it makes one.
const THE_INTO: &str = "ALO_CHANGE_INTO";

/// A name inside the granted folder, for the legitimate write.
const THE_INSIDE: &str = "ALO_CHANGE_INSIDE";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about the control, followed by `allowed` or a number.
const THE_CONTROL_WENT: &str = "alo:control ";

/// What the child says about the mutation itself.
const THE_SUBJECT_WENT: &str = "alo:subject ";

/// What the child says about the act that would move contents past the grant.
const THE_AFTER_WENT: &str = "alo:after ";

/// What the child says about the legitimate write inside the granted folder.
const THE_INSIDE_WENT: &str = "alo:inside ";

/// What is in the file nobody granted.
const A_SECRET: &str = "not an invoice";

/// What the legitimate write puts inside the granted folder.
const A_LEGITIMATE_WRITE: &str = "an archive being made";

/// The extended attribute a turn sets on a file it cannot read.
const AN_ATTRIBUTE: &str = "user.alo.note";

/// What it is set to.
const AN_ATTRIBUTE_VALUE: &[u8] = b"changed by a turn";

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

    /// The unwatched mutation.
    subject: Outcome,

    /// The act that would turn that mutation into contents leaving.
    after: Outcome,

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

    /// A folder nobody granted.
    private: PathBuf,

    /// A file in it, with contents the tests assert are undisturbed.
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
            private: root.join("Private"),
            secret,
            root,
        }
    }

    /// Take it away, whatever the test found.
    fn taken_away(&self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// One turn: a cgroup, a bound over one folder, and a child inside it making
/// one unwatched change.
///
/// The bound is the granted folder and nothing else, so everything the subject
/// touches is outside it and every refusal below is the boundary's.
fn a_bound_turn(named: &str, machine: &AMachine, subject: &str, into: &Path) -> Went {
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
        .env(THE_SUBJECT, subject)
        .env(THE_WHERE, &machine.secret)
        .env(THE_INTO, into)
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
        subject: said_about(&mut saying, THE_SUBJECT_WENT),
        after: said_about(&mut saying, THE_AFTER_WENT),
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

/// The four things every one of these tests asserts about a run, before it
/// asserts anything about its own subject.
///
/// The control is what says the boundary was in force at all, and the
/// legitimate write is what says it was not simply refusing everything.
fn the_boundary_was_in_force(went: &Went) {
    assert_eq!(
        went.control,
        Outcome::Refused(13),
        "the boundary was not in force, so nothing this turn did means anything"
    );
    assert_eq!(
        went.inside,
        Outcome::Allowed,
        "a turn was refused a write inside the folder it was granted, which is what \
         `archive_folder` does — this is a regression rather than a gap being closed"
    );
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

/// What is in a file, for the assertions that nothing was disturbed.
fn holds(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{} is not readable: {why}", path.display()))
}

/// **A turn makes a symbolic link, and cannot read through it.**
///
/// `inode_symlink` is unwatched, so a turn puts a name inside the folder it was
/// granted that leads to a file nobody granted it. Every path check in
/// `alo-files` refuses a link on the way in anyway, and this is the floor
/// underneath: the boundary walks up from the file the open actually reached, so
/// the *link's* name being inside a grant buys nothing at all.
///
/// A name is not contents. What this leaves behind is a pointer, in a folder the
/// person granted, to a file the turn was never able to read — and the test
/// reads through it from outside the turn to prove the pointer is real, so the
/// refusal is the boundary rather than a link that led nowhere.
#[test]
fn a_symbolic_link_is_made_and_the_turn_cannot_read_through_it() {
    let machine = AMachine::with_something_worth_protecting("symlink");
    let link = machine.granted.join("notes.txt");
    let went = a_bound_turn("alo-change-symlink", &machine, "symlink", &link);
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "a symbolic link was refused to a bound turn, which means `inode_symlink` is watched \
         now — say so in crates/alo-bounding-kernel/src/deciding.rs, in \
         crates/alo-bounding/src/lib.rs and in the table in docs/quirks.md, and turn this into \
         the refusal it should be"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "a bound turn read a file nobody granted it by putting a link to it inside the folder \
         somebody did, which is contents leaving a grant"
    );

    assert!(
        link.symlink_metadata()
            .expect("the link is there")
            .file_type()
            .is_symlink(),
        "the link was not made, so this test proves nothing about the hook it is named after"
    );
    assert_eq!(
        holds(&link),
        A_SECRET,
        "outside the turn the link leads to the secret, and if it did not the refusal above \
         would be about a broken link rather than about the boundary"
    );
    assert_eq!(holds(&machine.secret), A_SECRET, "the secret was disturbed");

    machine.taken_away();
}

/// **A turn makes a file outside its bound, and it stays empty.**
///
/// `inode_create` is unwatched and `file_open` is not, and an open with
/// `O_CREAT` meets them in that order — so the inode is made and the open that
/// would have written to it is refused. What a bound turn can leave in a folder
/// nobody granted is an empty file with a name of its choosing.
///
/// That is litter rather than a way out: no byte of anybody's document is in it,
/// and the turn cannot put one there, because putting one there is an open.
#[test]
fn a_file_is_made_outside_the_bound_and_stays_empty() {
    let machine = AMachine::with_something_worth_protecting("create");
    let made = machine.private.join("made.txt");
    let went = a_bound_turn("alo-change-create", &machine, "create", &made);
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(13),
        "the open that follows the create was allowed, so a bound turn can write a file into a \
         folder nobody granted — this is contents leaving a grant"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "a second attempt to write the file it had just made was allowed"
    );

    let left = made
        .metadata()
        .expect("the create hook is unwatched, so the file is there even though the open failed");
    assert_eq!(
        left.len(),
        0,
        "the file a refused open left behind has bytes in it"
    );

    machine.taken_away();
}

/// **A turn makes a file outside its bound without opening it at all**, and
/// still cannot put anything in it.
///
/// `inode_create` is the hook an open with `O_CREAT` meets; `inode_mknod` is the
/// hook the call that makes a file *without* opening it meets, and it is
/// unwatched too — so unlike the test above, this one is not refused anything.
/// A bound turn makes the inode and the call succeeds.
///
/// It is the same empty file at the end of it. Writing to what it made is an
/// open, and an open is watched, so the two hooks differ in what the turn is
/// told and not in what it can leave behind.
#[test]
fn a_file_is_made_outside_the_bound_without_being_opened_and_stays_empty() {
    let machine = AMachine::with_something_worth_protecting("mknod");
    let made = machine.private.join("made-without-opening.txt");
    let went = a_bound_turn("alo-change-mknod", &machine, "mknod", &made);
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "making a file without opening it was refused, which means `inode_mknod` is watched now \
         — say so in crates/alo-bounding-kernel/src/deciding.rs, in \
         crates/alo-bounding/src/lib.rs and in the table in docs/quirks.md"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "a bound turn wrote to the file it had made in a folder nobody granted, which is \
         contents leaving a grant"
    );

    assert_eq!(
        made.metadata()
            .expect("the file the turn made is there")
            .len(),
        0,
        "the file a bound turn made outside its bound has bytes in it"
    );

    machine.taken_away();
}

/// **A turn makes and removes empty directories anywhere, and empties none of
/// them.**
///
/// `inode_mkdir` and `inode_rmdir` are both unwatched, so a bound turn can make
/// a folder in a place nobody granted and take it away again. A directory holds
/// no bytes of anybody's file, and the one thing that would make this matter —
/// removing a folder that has something *in* it — needs the contents unlinked
/// first, which is `inode_unlink` and is watched.
///
/// So the `after` here is a delete this boundary already refuses, standing where
/// it stands to show why the two unwatched hooks beside it are not a way to
/// destroy somebody's folder.
#[test]
fn directories_are_made_and_removed_but_a_folder_with_anything_in_it_is_not() {
    let machine = AMachine::with_something_worth_protecting("directory");
    let made = machine.private.join("made");
    let went = a_bound_turn("alo-change-directory", &machine, "directory", &made);
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "making and removing an empty directory outside the bound was refused, which means \
         `inode_mkdir` and `inode_rmdir` are watched now — say so in \
         crates/alo-bounding-kernel/src/deciding.rs, in crates/alo-bounding/src/lib.rs and in \
         the table in docs/quirks.md"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "a bound turn removed a name outside its bound, so the folder it cannot remove today \
         could be emptied and then removed"
    );

    assert!(
        !made.exists(),
        "the directory the turn made was not removed again, so one half of this ran"
    );
    assert!(machine.private.exists(), "the private folder is gone");
    assert_eq!(holds(&machine.secret), A_SECRET, "the secret was disturbed");

    machine.taken_away();
}

/// **A turn changes the permissions of a file it cannot read, and still cannot
/// read it.**
///
/// `inode_setattr` is unwatched, so mode, owner and times are a bound turn's to
/// change on any file the ordinary permissions let it. The boundary is not
/// fooled by the result, because it decides by **where a file is** and not by
/// what its mode says: the same open is refused after `0o777` as before it.
///
/// What it costs is real and is not contents: a mode left wide open outlives the
/// turn, and what the person's *other* software may then read is not this
/// boundary's question. `docs/quirks.md` says so rather than leaving it implied.
#[test]
fn a_files_permissions_are_changed_and_it_is_still_not_readable() {
    let machine = AMachine::with_something_worth_protecting("mode");
    let went = a_bound_turn("alo-change-mode", &machine, "mode", Path::new(""));
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "changing the mode of a file nobody granted was refused, which means `inode_setattr` is \
         watched now — say so in crates/alo-bounding-kernel/src/deciding.rs, in \
         crates/alo-bounding/src/lib.rs and in the table in docs/quirks.md"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "a bound turn read a file nobody granted it by changing its mode first, which would \
         mean the boundary decides by permission rather than by place"
    );

    let mode = machine
        .secret
        .metadata()
        .expect("the secret is there")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o777,
        "the mode was not changed, so this test proves nothing about the hook it is named after"
    );
    assert_eq!(holds(&machine.secret), A_SECRET, "the secret was disturbed");

    machine.taken_away();
}

/// **A turn sets an extended attribute on a file it cannot read, and still
/// cannot read it.**
///
/// `inode_setxattr` is the other half of the attribute gap and is unwatched too.
/// An attribute is a place to put bytes that is not the file's contents, so this
/// is the one of the list that could look like a way to *store* something —
/// and it is not a way to move anybody's document, because filling it needs
/// bytes the turn cannot read in the first place.
#[test]
fn an_extended_attribute_is_set_and_the_file_is_still_not_readable() {
    let machine = AMachine::with_something_worth_protecting("xattr");
    let went = a_bound_turn("alo-change-xattr", &machine, "xattr", Path::new(""));
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "setting an extended attribute on a file nobody granted was refused, which means \
         `inode_setxattr` is watched now — say so in \
         crates/alo-bounding-kernel/src/deciding.rs, in crates/alo-bounding/src/lib.rs and in \
         the table in docs/quirks.md"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "a bound turn read a file nobody granted it after writing an attribute on it"
    );

    let mut read = [0u8; 64];
    let length = rustix::fs::getxattr(&machine.secret, AN_ATTRIBUTE, &mut read[..])
        .expect("the attribute the turn set is there");
    assert_eq!(
        read.get(..length),
        Some(AN_ATTRIBUTE_VALUE),
        "the attribute was not set, so this test proves nothing about the hook it is named after"
    );
    assert_eq!(holds(&machine.secret), A_SECRET, "the secret was disturbed");

    machine.taken_away();
}

/// **A bound turn cannot start a program to do any of this for it**, because
/// starting one is an open of the program's own file.
///
/// The unwatched hooks above are what a turn can do with the calls it makes
/// itself. The obvious next question is whether it can run something else that
/// makes the calls for it — `truncate`, `chmod`, `rm` — and the answer is no,
/// for a reason that is not a sixth hook: `execve` opens the file it is going to
/// run, `file_open` is watched, and a program outside the bound is a file
/// outside the bound.
///
/// It is a floor under law 2 rather than the law itself. `alo-capability` is
/// what makes there be no verb that runs a command; this is what a bug in one
/// would meet. A dynamically linked program inside a granted folder would still
/// be refused its loader and its libraries, which are outside it — so this is
/// not a way to make one run either.
///
/// The measurement was an accident of trying to reproduce a `truncate(2)`
/// through the coreutil, which was refused here rather than at the truncate.
/// `docs/quirks.md` has what that left unreproduced.
#[test]
fn a_turn_cannot_start_a_program_that_is_outside_its_bound() {
    let machine = AMachine::with_something_worth_protecting("program");
    let went = a_bound_turn("alo-change-program", &machine, "program", Path::new(""));
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(13),
        "a bound turn started a program whose file is outside its bound, so every mutation the \
         hooks above do not watch could be made by something the turn ran instead of by the turn"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "the program's own file opened, which would mean the refusal above was about something \
         other than the boundary"
    );

    machine.taken_away();
}

/// A program every Linux has, used as a file rather than for what it does.
const A_PROGRAM: &str = "/bin/true";

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and each parent asks for it by
/// name. Run without the environment that names a cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the tests above, and each parent runs it by name"]
fn the_work_a_turn_does() {
    let (Ok(cgroup), Ok(control), Ok(subject), Ok(acting_on), Ok(into), Ok(inside)) = (
        env::var(THE_CGROUP),
        env::var(THE_CONTROL),
        env::var(THE_SUBJECT),
        env::var(THE_WHERE),
        env::var(THE_INTO),
        env::var(THE_INSIDE),
    ) else {
        return;
    };
    let (acting_on, into, inside) = (
        PathBuf::from(acting_on),
        PathBuf::from(into),
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
    let control = went(fs::File::open(&control).map(drop));
    writeln!(saying, "{THE_CONTROL_WENT}{control}").expect("the parent is listening");

    let (subject, after) = match subject.as_str() {
        // A name inside the granted folder that leads out of it.
        "symlink" => (
            went(std::os::unix::fs::symlink(&acting_on, &into)),
            went(fs::read_to_string(&into).map(drop)),
        ),
        // A file in a folder nobody granted. The create is unwatched and the
        // open that follows it is not, so this is one call that meets both.
        "create" => (
            went(fs::File::create(&into).map(drop)),
            went(fs::write(&into, b"contents from inside a turn")),
        ),
        // The same file, made by the call that does not open it, so nothing
        // watched is met at all.
        "mknod" => (
            went(
                rustix::fs::mknodat(
                    rustix::fs::CWD,
                    &into,
                    rustix::fs::FileType::RegularFile,
                    rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
                    0,
                )
                .map_err(|why| std::io::Error::from_raw_os_error(why.raw_os_error())),
            ),
            went(fs::write(&into, b"contents from inside a turn")),
        ),
        // An empty folder made and taken away, and then the delete that would be
        // needed to empty one that is not.
        "directory" => (
            went(fs::create_dir(&into).and_then(|()| fs::remove_dir(&into))),
            went(fs::remove_file(&acting_on)),
        ),
        // The mode of a file this turn cannot open, and then opening it.
        "mode" => (
            went(fs::set_permissions(
                &acting_on,
                fs::Permissions::from_mode(0o777),
            )),
            went(fs::File::open(&acting_on).map(drop)),
        ),
        // Something else to make the calls this turn cannot, and then the open
        // that starting it really is.
        "program" => (
            went(Command::new(A_PROGRAM).status().map(drop)),
            went(fs::File::open(A_PROGRAM).map(drop)),
        ),
        // An extended attribute on a file this turn cannot open, and then
        // opening it.
        _ => (
            went(
                rustix::fs::setxattr(
                    &acting_on,
                    AN_ATTRIBUTE,
                    AN_ATTRIBUTE_VALUE,
                    rustix::fs::XattrFlags::empty(),
                )
                .map_err(|why| std::io::Error::from_raw_os_error(why.raw_os_error())),
            ),
            went(fs::File::open(&acting_on).map(drop)),
        ),
    };
    writeln!(saying, "{THE_SUBJECT_WENT}{subject}").expect("the parent is listening");
    writeln!(saying, "{THE_AFTER_WENT}{after}").expect("the parent is listening");

    // And the legitimate one: a file written inside the folder this turn was
    // granted, which is what `archive_folder` does and what the boundary must
    // not refuse.
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
