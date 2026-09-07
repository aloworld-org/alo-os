//! What an `O_PATH` handle is to this boundary, asked of the kernel rather
//! than of a manual page.
//!
//! Queue item 6c needs handles on two folders in order to move a file without
//! resolving either name twice, and `crates/alo-files` cannot take them the
//! ordinary way: an open is an open, and a turn's boundary permits opening only
//! what its call named. The folder a `move_file` takes a file **out of** is not
//! that.
//!
//! `O_PATH` is the candidate. It opens a *reference to a place* rather than a
//! file — it can be `openat`'d from and `renameat`'d with, and it cannot be
//! read, written, or listed. The claim worth testing is that Linux does not run
//! `security_file_open` for one, which would mean handles this boundary never
//! sees.
//!
//! **That claim being true is not the same as it being safe**, and this file
//! asks both questions. A handle the boundary cannot see is only acceptable if
//! nothing can be done *through* it that the boundary would have refused —
//! so every probe below that opens something ungranted is followed by a probe
//! that tries to use it.
//!
//! # It is one child process, and it reports every probe
//!
//! The shape is `the_kernel_refuses.rs`'s and for its reasons: a cgroup holds
//! processes, so the work of a turn is a second process, and it is this binary
//! re-run. What differs is that this child performs several probes rather than
//! one open, because the answer here is a *set* of facts that only mean
//! something together — an `O_PATH` handle that opened and then let a file be
//! read is a hole, and either half alone would read as good news.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    collections::BTreeMap,
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alo_bounding::{Cgroup, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_O_PATH_TEST_CGROUP";

/// Where the child is told the tree it is probing is.
const THE_ROOT: &str = "ALO_O_PATH_TEST_ROOT";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What every probe's answer begins with.
const PROBE: &str = "alo:probe ";

/// What the child says when it has no more to say.
const DONE: &str = "alo:done";

/// What one probe made of what it tried.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Said {
    /// It worked.
    Fine,

    /// The machine refused, with the number it gave.
    No(i32),

    /// It was not attempted, because what it needed did not happen.
    NotReached,
}

impl Said {
    /// What the child wrote, read back.
    fn from(text: &str) -> Self {
        match text {
            "fine" => Self::Fine,
            "not-reached" => Self::NotReached,
            other => Self::No(other.parse().unwrap_or(0)),
        }
    }

    /// What to write.
    fn written(&self) -> String {
        match self {
            Self::Fine => "fine".to_owned(),
            Self::NotReached => "not-reached".to_owned(),
            Self::No(number) => number.to_string(),
        }
    }
}

/// A granted folder with a file in it, and an ungranted one next to it.
fn a_machine_with_something_worth_protecting() -> PathBuf {
    let root = PathBuf::from("/tmp").join(format!("alo-o-path-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
    fs::create_dir_all(root.join("Elsewhere")).expect("a temporary directory can be made");
    fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
    fs::write(root.join("Elsewhere/secret.txt"), b"not an invoice").expect("a file can be written");
    root
}

/// Runs one turn and collects everything the child reported.
fn what_the_kernel_says(named: &str, root: &Path, granted: &[&Path]) -> BTreeMap<String, Said> {
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
            "the_probes_a_turn_does",
        ])
        .env(THE_CGROUP, cgroup.at())
        .env(THE_ROOT, root)
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

    let mut answers = BTreeMap::new();
    loop {
        let said = next_line_from(&mut saying);
        let said = said.trim();
        if said == DONE {
            break;
        }
        let probe = said
            .strip_prefix(PROBE)
            .expect("every line from the child is a probe or the end");
        let (name, outcome) = probe
            .split_once(' ')
            .expect("a probe is a name and what happened");
        answers.insert(name.to_owned(), Said::from(outcome));
    }

    child.wait().expect("the child finishes");
    boundary.released(turn).expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    answers
}

/// The next line the child says that is meant for us.
fn next_line_from(saying: &mut BufReader<std::process::ChildStdout>) -> String {
    let mut line = String::new();
    loop {
        line.clear();
        let read = saying.read_line(&mut line).expect("the child is talking");
        assert!(read > 0, "the child stopped talking before it was done");
        if line.starts_with("alo:") {
            return line;
        }
    }
}

/// What one probe answered, named so a failure says which.
fn answer<'a>(answers: &'a BTreeMap<String, Said>, probe: &str) -> &'a Said {
    answers
        .get(probe)
        .unwrap_or_else(|| panic!("the child never reported {probe}: {answers:?}"))
}

/// **The whole question, and every part of it, in one turn.**
///
/// One turn, granted `Invoices` and nothing else, and a folder next to it that
/// nobody granted. What the assertions below are worth depends on them being
/// the same turn under the same boundary, which is why this is one test rather
/// than seven.
///
/// # What this kernel said, and what `alo-files` was built on
///
/// | | |
/// |---|---|
/// | `O_PATH` on an ungranted **folder** | opens |
/// | `O_PATH` on an ungranted **file** | opens |
/// | `openat` a file **through** that handle | `EACCES` |
/// | reopening it through `/proc/self/fd` | `EACCES` |
/// | `renameat2` between two `O_PATH` handles | works |
///
/// The first two are the claim item 6c wanted checked, and they are true: an
/// `O_PATH` open does not reach `security_file_open`. **The next two are why
/// that is allowed to matter.** A handle the boundary never saw would be a hole
/// if anything could be read through it, and nothing can: the walk in
/// `alo-bounding-kernel` goes up from the *file's own* directory entry, which
/// is where the file is rather than which handle it was reached from. So an
/// `O_PATH` handle is a reference to a place and confers no reading — which is
/// exactly the authority `alo-files` needs to move a name and no more.
#[test]
fn an_o_path_handle_opens_where_a_read_would_not_and_still_reads_nothing() {
    let machine = a_machine_with_something_worth_protecting();
    let answers = what_the_kernel_says("alo-o-path", &machine, &[&machine.join("Invoices")]);

    // The controls. Without these the rest could all be a boundary that was
    // never in force, which is the failure mode a test like this has.
    assert_eq!(
        answer(&answers, "read-ungranted-file"),
        &Said::No(13),
        "the boundary was not in force, so nothing else here means anything: {answers:?}"
    );
    assert_eq!(
        answer(&answers, "read-granted-file"),
        &Said::Fine,
        "the boundary refused a file it granted: {answers:?}"
    );

    // The claim `alo-files` is built on. If a kernel ever starts checking these,
    // `opening.rs` has to go back to resolving a rename's folders by name, and
    // this is what says so.
    assert_eq!(
        answer(&answers, "path-open-ungranted-folder"),
        &Said::Fine,
        "this kernel checks O_PATH opens, so alo-files cannot take a handle on the folder a \
         move takes a file out of — see crates/alo-files/src/opening.rs: {answers:?}"
    );
    assert_eq!(
        answer(&answers, "path-open-ungranted-file"),
        &Said::Fine,
        "this kernel checks O_PATH opens: {answers:?}"
    );

    // **And the two that decide whether the claim above is allowed to be used.**
    // A handle the boundary cannot see is only acceptable while nothing can be
    // done through it that the boundary would have refused.
    assert_eq!(
        answer(&answers, "read-through-path-handle"),
        &Said::No(13),
        "a file nobody granted was read through an O_PATH handle — the boundary has a hole \
         in it and alo-files must stop taking one: {answers:?}"
    );
    assert_eq!(
        answer(&answers, "reopen-through-proc-self-fd"),
        &Said::No(13),
        "an O_PATH handle was turned back into a readable file through /proc/self/fd, which \
         is the classic way round: {answers:?}"
    );

    // The move this is all for can be made from handles at all.
    assert_eq!(
        answer(&answers, "rename-with-path-handles"),
        &Said::Fine,
        "renameat2 will not take O_PATH handles on this kernel: {answers:?}"
    );

    // **And the fact that keeps this honest.** A rename is not on this
    // boundary's hook at all — ADR 0015 names `inode_rename` beside `file_open`
    // and only `file_open` is built — so moving a file nobody granted is not
    // something the kernel stops today. Taking O_PATH handles in `alo-files`
    // therefore widens nothing: it closes a race in our own code, above a
    // boundary that was never watching this syscall. That is worth a test
    // rather than a sentence, because the day the hook arrives this changes and
    // somebody should be told.
    assert_eq!(
        answer(&answers, "rename-ungranted-file"),
        &Said::Fine,
        "a rename outside the grant was refused, so this boundary now watches renames and \
         docs/quirks.md, ADR 0015's note and alo-files' reasoning are all out of date: \
         {answers:?}"
    );

    let _ = fs::remove_dir_all(&machine);
}

/// The probes, which are a second process because a cgroup holds processes.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Run without the environment that names a cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the test above, and the parent runs it by name"]
fn the_probes_a_turn_does() {
    use std::os::fd::AsRawFd;

    use rustix::fs::{CWD, Mode, OFlags, RenameFlags, openat, renameat_with};

    let (Ok(cgroup), Ok(root)) = (env::var(THE_CGROUP), env::var(THE_ROOT)) else {
        return;
    };
    let root = PathBuf::from(root);

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

    // From here the boundary is in force. Everything below reports rather than
    // panicking: a panic would print a backtrace, and reading `/proc/self/maps`
    // is an open like any other.
    let mut said: Vec<(&str, Said)> = Vec::new();
    let mut report = |name: &'static str, outcome: Said| said.push((name, outcome));

    let of = |result: Result<(), rustix::io::Errno>| match result {
        Ok(()) => Said::Fine,
        Err(why) => Said::No(why.raw_os_error()),
    };

    // The two controls: the boundary is in force, and it grants what it granted.
    report(
        "read-granted-file",
        of(fs::read(root.join("Invoices/march.pdf"))
            .map(|_| ())
            .map_err(|why| rustix::io::Errno::from_raw_os_error(why.raw_os_error().unwrap_or(0)))),
    );
    report(
        "read-ungranted-file",
        of(fs::read(root.join("Elsewhere/secret.txt"))
            .map(|_| ())
            .map_err(|why| rustix::io::Errno::from_raw_os_error(why.raw_os_error().unwrap_or(0)))),
    );

    // The claim: an `O_PATH` open of a folder nobody granted.
    let path_flags = OFlags::PATH.union(OFlags::CLOEXEC).union(OFlags::DIRECTORY);
    let ungranted_folder = openat(CWD, root.join("Elsewhere"), path_flags, Mode::empty());
    report(
        "path-open-ungranted-folder",
        match &ungranted_folder {
            Ok(_) => Said::Fine,
            Err(why) => Said::No(why.raw_os_error()),
        },
    );

    // And what can be done with it, which is the question that decides.
    report(
        "read-through-path-handle",
        match &ungranted_folder {
            Err(_) => Said::NotReached,
            Ok(folder) => match openat(
                folder,
                "secret.txt",
                OFlags::RDONLY.union(OFlags::CLOEXEC),
                Mode::empty(),
            ) {
                Ok(_) => Said::Fine,
                Err(why) => Said::No(why.raw_os_error()),
            },
        },
    );

    // The same question for a file, and the classic way of turning an `O_PATH`
    // handle back into a readable one.
    let ungranted_file = openat(
        CWD,
        root.join("Elsewhere/secret.txt"),
        OFlags::PATH.union(OFlags::CLOEXEC),
        Mode::empty(),
    );
    report(
        "path-open-ungranted-file",
        match &ungranted_file {
            Ok(_) => Said::Fine,
            Err(why) => Said::No(why.raw_os_error()),
        },
    );
    report(
        "reopen-through-proc-self-fd",
        match &ungranted_file {
            Err(_) => Said::NotReached,
            Ok(held) => {
                let again = format!("/proc/self/fd/{}", held.as_raw_fd());
                match fs::read(&again) {
                    Ok(_) => Said::Fine,
                    Err(why) => Said::No(why.raw_os_error().unwrap_or(0)),
                }
            }
        },
    );

    // Whether the move this is all for can be made from handles at all.
    let granted_folder = openat(CWD, root.join("Invoices"), path_flags, Mode::empty());
    report(
        "rename-with-path-handles",
        match &granted_folder {
            Err(why) => Said::No(why.raw_os_error()),
            Ok(folder) => match renameat_with(
                folder,
                "march.pdf",
                folder,
                "march-2026.pdf",
                RenameFlags::NOREPLACE,
            ) {
                Ok(()) => Said::Fine,
                Err(why) => Said::No(why.raw_os_error()),
            },
        },
    );

    // And what a rename of something nobody granted does today, which is the
    // fact item 6c is about.
    report(
        "rename-ungranted-file",
        match renameat_with(
            CWD,
            root.join("Elsewhere/secret.txt"),
            CWD,
            root.join("Elsewhere/moved.txt"),
            RenameFlags::NOREPLACE,
        ) {
            Ok(()) => Said::Fine,
            Err(why) => Said::No(why.raw_os_error()),
        },
    );

    for (name, outcome) in said {
        writeln!(saying, "{PROBE}{name} {}", outcome.written()).expect("the parent is listening");
    }
    writeln!(saying, "{DONE}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}
