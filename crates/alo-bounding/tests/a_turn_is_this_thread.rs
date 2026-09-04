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
//! # It needs root, and a kernel that started the BPF security module
//!
//! The same three checks `the_kernel_refuses.rs` names, and it fails loudly on a
//! machine without them for the same reason: a test that quietly skipped itself
//! would report green on every machine where the boundary does nothing at all.
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
    sync::{Mutex, MutexGuard, OnceLock, mpsc},
};

use alo_bounding::{Boundary, Cgroup, Turns, place_of};

/// The boundary, loaded once for the whole run.
///
/// One program on `file_open` rather than one per test, and the lock is what
/// makes these tests one at a time — which they have to be, because
/// [`Turns::under`] moves this whole process into a control group of its own.
fn the_boundary() -> &'static Mutex<Boundary> {
    static LOADED: OnceLock<Mutex<Boundary>> = OnceLock::new();
    LOADED.get_or_init(|| {
        Mutex::new(Boundary::imposed().unwrap_or_else(|why| {
            panic!(
                "the boundary could not be imposed on this kernel, so nothing below is being \
                 tested: {why}\n\
                 This needs root, `CONFIG_BPF_LSM=y`, and `bpf` in the list of security modules \
                 the kernel *started* — `cat /sys/kernel/security/lsm`, which is not the same \
                 question as how the kernel was built. `docs/hardware.md` has the three commands."
            )
        }))
    })
}

/// What each test is handed: a folder that is granted, a file in it, and a
/// private key beside it that is not.
struct AMachine {
    /// The granted folder.
    granted: PathBuf,

    /// A file inside it.
    invoice: PathBuf,

    /// A file outside it, of the kind ADR 0013 is about.
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
    fs::create_dir_all(root.join(".ssh")).expect("a temporary directory can be made");
    fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
    fs::write(root.join(".ssh/id_ed25519"), b"not a real key").expect("a file can be written");
    AMachine {
        granted: root.join("Invoices"),
        invoice: root.join("Invoices/march.pdf"),
        key: root.join(".ssh/id_ed25519"),
    }
}

/// A service, arranged the way `alo-agentd` will be: its own control group, a
/// threaded subtree inside it, and every turn made in there.
///
/// The subtree is given back and the control group removed before this returns,
/// whatever the test found — a run that left this process inside a cgroup it had
/// removed would break every test after it rather than the one that failed.
fn as_a_service<T>(
    what: &str,
    doing: impl FnOnce(&Turns, &mut MutexGuard<'_, Boundary>, &AMachine) -> T,
) -> T {
    let machine = a_machine_with_something_worth_protecting(what);
    let mut boundary = the_boundary().lock().expect("nothing panicked holding it");

    let ours = Cgroup::made(&format!("alo-{what}-{}", std::process::id()))
        .expect("a control group can be made");
    let turns = Turns::under(ours.at()).expect("a service can make a subtree of its own");

    let found = doing(&turns, &mut boundary, &machine);

    turns
        .given_back()
        .expect("a service can be put back where it was");
    ours.removed()
        .expect("an empty control group can be taken away");
    found
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
        let granted = place_of(&machine.granted).expect("the granted folder is there");
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

        let granted = place_of(&machine.granted).expect("the granted folder is there");
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
        let granted = place_of(&machine.granted).expect("the granted folder is there");
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
            for starting in ["Command", "fork(", "exec", "posix_spawn"] {
                assert!(
                    !line.contains(starting),
                    "{} names {starting}, and law 2 is that nothing here starts a program:\n{line}",
                    file.display()
                );
            }
        }
    }
    assert!(
        read >= 8,
        "only {read} source files were read, so this test is not looking at the crate"
    );
}

/// Whether a line of Rust says something to a reader rather than to the machine.
///
/// Crude on purpose: it catches the whole-line comments this crate is written
/// in, and a `Command` hidden after code on a line that starts with `//` is not
/// a thing that happens by accident.
fn is_a_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}
