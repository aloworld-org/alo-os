//! A turn is one thread of this service, and the kernel refuses what it did not
//! grant that thread.
//!
//! `the_kernel_refuses.rs` proved the mechanism with a **child process**, which
//! is the right shape for a test and the wrong shape for a daemon: a service
//! that started a program to do a turn's work would be a service with a program
//! to choose, and law 2 is that there is no such choice anywhere in alo OS.
//!
//! So this file proves the shape `alo-agentd` will really use. Nothing is
//! started. The work is a closure, it runs on the thread that called
//! [`alo_bounding::Turns::doing`], and while it runs the kernel decides every
//! open that thread makes.
//!
//! # Three things are being asserted, and the second is the one that is new
//!
//! - **The kernel refuses this thread**, exactly as it refused a child process.
//! - **It refuses only this thread.** Another thread of the same process, in the
//!   same service, opens the same file at the same moment — because a boundary
//!   that caught the whole process would catch the record, the socket and the
//!   person's own door along with the verb.
//! - **A thread can leave**, and can do it again. Leaving is a write to a
//!   descriptor opened before the thread went in, because *opening* the way out
//!   while inside is an open the boundary correctly refuses.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! The same checks `the_kernel_refuses.rs` names, and it fails loudly on a
//! machine without them for the same reason: a test that quietly skipped itself
//! would report green on every machine where the boundary does nothing at all.
//! Where the boundary comes from is `on_this_kernel/mod.rs`, shared with the
//! other two files here — since ADR 0018 imposing one is a loader's job and
//! opening it is a daemon's, so a test that wants a machine needs both halves.
//!
//! # Nothing is asserted from inside a turn
//!
//! What runs inside the boundary gathers what happened and comes back out; every
//! assertion is made afterwards. A failing assertion inside would panic inside
//! the boundary, and a panic prints a backtrace, and a backtrace opens
//! `/proc/self/maps` — which is an open outside the grant, refused, in the middle
//! of reporting why something else went wrong.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
};

use alo_bounding::{Boundary, Bounds, Cgroup, Turns, place_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// What each test is handed: a folder that is granted, a file in it, a second
/// folder somewhere else, and a private key beside both that is not granted.
struct AMachine {
    /// The granted folder.
    granted: PathBuf,

    /// A file inside it.
    invoice: PathBuf,

    /// A second folder, in another part of the tree.
    ///
    /// What the second half of a `move_file` names. It is granted only by the
    /// test that grants it, so every other test here is still a turn bound to
    /// one place and a file in this folder is outside it.
    elsewhere: PathBuf,

    /// A file inside that one.
    receipt: PathBuf,

    /// A file outside both, of the kind ADR 0013 is about.
    key: PathBuf,
}

/// A directory tree with something worth protecting in it.
///
/// The same shape `the_kernel_refuses.rs` builds and for the same reason: the
/// file has to really exist, or the open fails at lookup with `ENOENT` before
/// `file_open` is reached and the boundary is never consulted.
fn a_machine_with_something_worth_protecting(what: &str) -> AMachine {
    let root = PathBuf::from("/tmp").join(format!("alo-turns-{}-{what}", std::process::id()));
    fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
    fs::create_dir_all(root.join("Archive")).expect("a temporary directory can be made");
    fs::create_dir_all(root.join(".ssh")).expect("a temporary directory can be made");
    fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
    fs::write(root.join("Archive/february.pdf"), b"a receipt").expect("a file can be written");
    fs::write(root.join(".ssh/id_ed25519"), b"not a real key").expect("a file can be written");
    AMachine {
        granted: root.join("Invoices"),
        invoice: root.join("Invoices/march.pdf"),
        elsewhere: root.join("Archive"),
        receipt: root.join("Archive/february.pdf"),
        key: root.join(".ssh/id_ed25519"),
    }
}

/// A service, arranged the way `alo-agentd` will be: its own control group, a
/// threaded subtree inside it, and every turn made in there.
///
/// The subtree is given back and the control group removed before this returns,
/// whatever the test found — a run that left this process inside a cgroup it had
/// removed would break every test after it rather than the one that failed.
fn as_a_service<T>(what: &str, doing: impl FnOnce(&Turns, &mut Boundary, &AMachine) -> T) -> T {
    let machine = a_machine_with_something_worth_protecting(what);
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(what);

    let ours = Cgroup::made(&format!("alo-{what}-{}", std::process::id()))
        .expect("a control group can be made");
    let turns = Turns::under(ours.at()).expect("a service can make a subtree of its own");

    let found = doing(&turns, &mut kernel.boundary, &machine);

    turns
        .given_back()
        .expect("a service can be put back where it was");
    ours.removed()
        .expect("an empty control group can be taken away");
    found
}

/// A bound over one folder, which is what most of these tests grant.
fn only(folder: &Path) -> Bounds {
    Bounds::of_one(place_of(folder).expect("the granted folder is there"))
}

/// What one open came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The file opened.
    Opened,

    /// The open was refused, with the number the machine gave.
    Refused(i32),
}

/// Opens a file and says what the machine made of it.
///
/// Nothing else: this is what runs inside the boundary, so it opens exactly one
/// thing and holds no opinion about it.
fn opening(what: &Path) -> Outcome {
    match fs::File::open(what) {
        Ok(_) => Outcome::Opened,
        Err(why) => Outcome::Refused(why.raw_os_error().unwrap_or(0)),
    }
}

/// **The whole item, in one test.** A turn is this thread; the file inside the
/// grant opens; the private key beside it is refused by the kernel; and once the
/// turn is over the same key opens again.
///
/// Nothing in this repository made the middle decision. `alo-capability` was not
/// asked, no verb was validated, no policy was consulted, and no process was
/// started: a thread opened a file and the machine said no.
#[test]
fn a_turn_is_this_thread_and_the_kernel_refuses_what_it_did_not_grant() {
    let (inside, after) = as_a_service("thread", |turns, boundary, machine| {
        let granted = only(&machine.granted);
        let inside = turns
            .doing(boundary, "turn-1", granted, || {
                (opening(&machine.invoice), opening(&machine.key))
            })
            .expect("the turn can be bounded");
        (inside, opening(&machine.key))
    });

    assert_eq!(
        inside.0,
        Outcome::Opened,
        "the granted file was refused to the turn that was granted it"
    );
    assert_eq!(
        inside.1,
        Outcome::Refused(13),
        "the kernel should have refused this open with EACCES"
    );
    assert_eq!(
        after,
        Outcome::Opened,
        "the turn is over and the authority should be gone with it"
    );
}

/// **A turn is bound to the places its execution named, and to all of them.**
/// Item 26b's question, answered against a real kernel: a `move_file` names the
/// file and the folder it is going into, they are in two different parts of the
/// tree, and both open — while the private key beside them is still refused.
///
/// The refusal is what makes the test worth anything. A bound that had quietly
/// become *everything under the root* would open all three, and the first two
/// assertions alone could not tell that apart from working.
///
/// It is also the one thing on this machine that says the two halves agree about
/// the *width* of an entry. `alo-bounding-map` decides how many places one entry
/// holds and where the count sits, and a daemon writing nine words that the
/// kernel read as two would refuse everything the second place covers without
/// anything anywhere saying so.
#[test]
fn a_turn_is_bounded_to_every_place_its_execution_named() {
    let found = as_a_service("two-places", |turns, boundary, machine| {
        let granted = alo_bounding::places_of(&[&machine.granted, &machine.elsewhere])
            .expect("both folders are there and two is not too many");
        turns
            .doing(boundary, "turn-two-places", granted, || {
                (
                    opening(&machine.invoice),
                    opening(&machine.receipt),
                    opening(&machine.key),
                )
            })
            .expect("the turn can be bounded")
    });

    assert_eq!(
        found.0,
        Outcome::Opened,
        "the first place the execution named was refused to it"
    );
    assert_eq!(
        found.1,
        Outcome::Opened,
        "the second place the execution named was refused to it, so a move could never run"
    );
    assert_eq!(
        found.2,
        Outcome::Refused(13),
        "a turn bounded to two folders reached a third thing, so the bound is not a bound"
    );
}

/// **A folder that was granted to the turn and not named by the execution is
/// outside the bound.** The narrower answer, measured rather than argued: the
/// same two folders exist and the same turn runs, and a bound made of one of
/// them refuses the other.
///
/// This is the difference between item 26b's two candidate answers. If the bound
/// were the turn's whole grant, this file would open.
#[test]
fn a_place_the_execution_did_not_name_is_outside_the_bound() {
    let found = as_a_service("narrower", |turns, boundary, machine| {
        turns
            .doing(boundary, "turn-narrower", only(&machine.granted), || {
                (opening(&machine.invoice), opening(&machine.receipt))
            })
            .expect("the turn can be bounded")
    });

    assert_eq!(found.0, Outcome::Opened);
    assert_eq!(
        found.1,
        Outcome::Refused(13),
        "a folder this execution never named was inside its boundary"
    );
}

/// **The boundary is around the thread and not around the service.** Another
/// thread of the same process opens the private key at the moment the turn's
/// thread is being refused it.
///
/// This is what a whole process in the cgroup would have cost: `alo-agentd`
/// serves the person's door, writes the record and holds a socket on threads
/// that are not doing the agent's work, and every one of those would have been
/// inside the agent's grant.
///
/// The other thread is started **before** the turn begins, because a thread
/// inherits the control group of whichever thread created it — one started
/// inside the turn would be inside the turn.
#[test]
fn the_other_threads_of_this_service_are_not_in_the_turn() {
    let (in_the_turn, elsewhere) = as_a_service("elsewhere", |turns, boundary, machine| {
        let (ask, asked) = mpsc::channel::<PathBuf>();
        let (say, said) = mpsc::channel::<Outcome>();
        let elsewhere = std::thread::spawn(move || {
            while let Ok(what) = asked.recv() {
                if say.send(opening(&what)).is_err() {
                    return;
                }
            }
        });

        let granted = only(&machine.granted);
        let found = turns
            .doing(boundary, "turn-elsewhere", granted, || {
                let mine = opening(&machine.key);
                ask.send(machine.key.clone()).expect("the thread is there");
                let theirs = said.recv().expect("the thread answers");
                (mine, theirs)
            })
            .expect("the turn can be bounded");

        drop(ask);
        elsewhere.join().expect("the thread finishes");
        found
    });

    assert_eq!(
        in_the_turn,
        Outcome::Refused(13),
        "the thread doing the turn's work was not bounded"
    );
    assert_eq!(
        elsewhere,
        Outcome::Opened,
        "a thread of this service that is not doing an agent's work was caught by the agent's \
         boundary"
    );
}

/// **A thread leaves through a door it opened before it went in**, and can do it
/// again.
///
/// Two turns in a row through one service. The second is the assertion: if
/// leaving had opened anything, the first turn would have been refused it and
/// there would be no second.
///
/// It also asserts that a turn takes its control group away with it, which is
/// the visible half of ADR 0015's *the entry is removed, and authority is gone*:
/// a turn that is over is not a place on this machine any more.
#[test]
fn a_thread_leaves_through_a_door_it_opened_before_it_went_in() {
    let found = as_a_service("again", |turns, boundary, machine| {
        let granted = only(&machine.granted);
        let mut found = Vec::new();
        for named in ["turn-first", "turn-second"] {
            let outcome = turns
                .doing(boundary, named, granted, || opening(&machine.key))
                .expect("the turn can be bounded");
            found.push((outcome, turns.home().at().with_file_name(named).exists()));
        }
        found
    });

    for (outcome, still_there) in found {
        assert_eq!(
            outcome,
            Outcome::Refused(13),
            "a turn was not bounded, so leaving the one before it opened something"
        );
        assert!(
            !still_there,
            "the turn is over and its control group is still on the machine"
        );
    }
}

/// **Nothing in this crate starts a program**, and law 2 is why that is a test
/// rather than a sentence.
///
/// A boundary is a thing you put work *inside*, and the obvious way to put work
/// inside something is to start it there. Every one of those shapes needs a
/// program to name, and a program alo OS starts on an agent's behalf is one
/// review away from a program an agent named. The answer this crate gives is
/// that the work is already running — it is a thread of the service — so this
/// reads the crate's own source and holds it to that.
///
/// The tests are deliberately not read: `the_kernel_refuses.rs` starts a child
/// on purpose, because proving the kernel refuses a *process* is what item 26
/// was for. Documentation is not read either — these files explain at length
/// that they start nothing, and a test that could not tell a promise from a call
/// would fail on the sentence making the promise.
///
/// # `exec` is looked for as a call and not as four letters
///
/// It was four letters until item 26b, and that item is what found the flaw:
/// *execution* is this repository's own word for what a turn does — ADR 0001 §5
/// says *one approval, one execution* — so a function named after one failed a
/// test about `execve`. The word is not going away, and skipping the file it
/// appeared in would have been the gate weakened to pass it. So what is looked
/// for is the shapes the syscall family really takes in Rust, and
/// [`starts_a_program`] is the list with its own test beside it.
#[test]
fn nothing_in_this_crate_starts_a_program() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = 0_usize;
    for file in fs::read_dir(&source).expect("this crate has a src directory") {
        let file = file.expect("a directory entry can be read").path();
        if file.extension().is_none_or(|is| is != "rs") {
            continue;
        }
        let written = fs::read_to_string(&file).expect("a source file can be read");
        read += 1;
        for line in written.lines().filter(|line| !is_a_comment(line)) {
            assert!(
                starts_a_program(line).is_none(),
                "{} names {}, and law 2 is that nothing here starts a program:\n{line}",
                file.display(),
                starts_a_program(line).unwrap_or_default()
            );
        }
    }
    assert!(
        read >= 9,
        "only {read} source files were read, so this test is not looking at the crate"
    );
}

/// **The check above would catch something**, which is the half of it that a
/// green run cannot tell you.
///
/// A test that reads source for a forbidden spelling passes on the day the
/// spelling stops being looked for, and it passes in exactly the same colour. So
/// every way a program is started in Rust is put in front of it here, and so is
/// the word that made the old list wrong.
#[test]
fn the_check_catches_every_way_a_program_is_started() {
    for shape in [
        "let child = Command::new(\"sh\").spawn()?;",
        "std::process::Command::new(what).status()",
        "use std::os::unix::process::CommandExt;",
        "let never = doing.exec();",
        "unsafe { libc::fork() }",
        "libc::execve(path, args, environment)",
        "libc::execvp(name, args)",
        "libc::execlp(name, arg)",
        "libc::fexecve(fd, args, environment)",
        "posix_spawn(&mut pid, path, ...)",
    ] {
        assert!(
            starts_a_program(shape).is_some(),
            "a line that starts a program is not caught: {shape}"
        );
    }

    for ordinary in [
        "fn the_paths_an_execution_named_are_the_places_it_is_bounded_to() {",
        "let executed = what_this_execution_named(call);",
        "/// A turn's execution is one thread of the service.",
    ] {
        assert!(
            starts_a_program(ordinary).is_none(),
            "an ordinary line was called a program being started: {ordinary}"
        );
    }
}

/// Which way of starting a program a line of Rust names, if it names one.
///
/// The four letters of `exec` on their own are in *execution*, which is what a
/// turn does; these are the spellings that are a syscall. `.exec()` covers the
/// standard library's, `execv` covers `execve`, `execvp` and `fexecve`, and
/// `execl` covers `execlp` — so every member of the family is here under one of
/// two prefixes.
fn starts_a_program(line: &str) -> Option<&'static str> {
    ["Command", "fork(", "exec(", "execv", "execl", "posix_spawn"]
        .into_iter()
        .find(|starting| line.contains(starting))
}

/// Whether a line of Rust says something to a reader rather than to the machine.
///
/// Crude on purpose: it catches the whole-line comments this crate is written
/// in, and a `Command` hidden after code on a line that starts with `//` is not
/// a thing that happens by accident.
fn is_a_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}
