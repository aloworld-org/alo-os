//! A turn whose boundary cannot be applied does not run — asked of the
//! machine before every turn, and measured here in every way a machine loses
//! its boundary under a running service.
//!
//! `docs/features.md` makes the promise and ADR 0015 ends on it: *a refusal,
//! not a warning*. Until 2026-09-12 it was kept once, at start, by opening
//! the map of turns; a machine that lost its boundary afterwards ran every
//! turn unbounded and wrote each one down as bounded. This file reproduced
//! that before the check existed — the whole of task 15's gap, measured — and
//! now holds the check to three states a machine can be in, each beside the
//! state it must not disturb:
//!
//! - **the loader was never run, or its work was taken away** — no map
//!   pinned, and the turn is refused naming the service that pins one;
//! - **the programme is not held on one of its hooks** — a pin removed,
//!   which is the one thing on the machine that detaches a hook, and the turn
//!   is refused naming that hook, for each of the eighteen;
//! - **the loader was run again** — every pin taken away and made afresh, so
//!   the programme on the hooks reads a map this service never opened; the
//!   turn is refused naming both maps as the kernel numbers them.
//!
//! In every one of them the turn's work never runs, no control group is left
//! behind, the kernel holds no entry, and the sentence points at
//! `docs/quirks.md` rather than at a switch — because there is no switch, and
//! the last test here reads this crate's source to say so.
//!
//! # It uses the real door, on this thread
//!
//! Every turn here goes through [`alo_bounding::Turns::doing`], which is what
//! `alo-agentd` runs a verb through, on the thread the assertions are made
//! from — the shape `what_a_turn_inherits.rs` argues for. What is asserted
//! inside a turn is gathered and asserted outside it, because a failing
//! assertion inside a boundary panics inside one.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! And it fails loudly on a machine without them, for the reason every file
//! beside it gives: a test that skipped itself would be green on every
//! machine where nothing is bounded.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    cell::Cell,
    fs::{self, File},
    path::{Path, PathBuf},
};

use alo_bounding::{Boundary, Bounds, Cgroup, Imposed, NotBounded, Pinned, Turns, place_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// `EACCES`, as every Unix numbers it.
const REFUSED: i32 = 13;

/// What one open came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The file opened.
    Opened,

    /// The open was refused, with the number the machine gave.
    Refused(i32),
}

/// Opens a file by name and says what the machine made of it.
fn opening(what: &Path) -> Outcome {
    match File::open(what) {
        Ok(_) => Outcome::Opened,
        Err(why) => Outcome::Refused(why.raw_os_error().unwrap_or(0)),
    }
}

/// A directory tree with something worth protecting in it.
struct AMachine {
    /// Where it all is.
    root: PathBuf,

    /// The folder the turn is granted.
    granted: PathBuf,

    /// A file in it, which a bound turn opens.
    invoice: PathBuf,

    /// A file beside it, which a bound turn is refused.
    key: PathBuf,
}

impl AMachine {
    /// The same shape `the_kernel_refuses.rs` builds and for the same reason:
    /// the key has to exist, or the open fails at lookup before `file_open` is
    /// reached and the boundary is never consulted.
    fn with_something_worth_protecting(what: &str) -> Self {
        let root =
            PathBuf::from("/tmp").join(format!("alo-in-place-{}-{what}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
        fs::create_dir_all(root.join(".ssh")).expect("a temporary directory can be made");
        fs::write(root.join("Invoices/march.pdf"), b"an invoice").expect("a file can be written");
        fs::write(root.join(".ssh/id_ed25519"), b"not a real key").expect("a file can be written");
        Self {
            granted: root.join("Invoices"),
            invoice: root.join("Invoices/march.pdf"),
            key: root.join(".ssh/id_ed25519"),
            root,
        }
    }

    /// Take it away, whatever the test found.
    fn taken_away(&self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A bound over the one granted folder.
fn only(folder: &Path) -> Bounds {
    Bounds::of_one(place_of(folder).expect("the granted folder is there"))
}

/// What one turn's work found, gathered inside and asserted outside.
#[derive(Debug)]
struct Went {
    /// The key, which says the boundary was in force.
    control: Outcome,

    /// The invoice, which says it was not simply refusing everything.
    granted: Outcome,
}

/// A service, arranged the way `alo-agentd` is: its own control group, a
/// threaded subtree inside it, and every turn made in there — over a boundary
/// the test may take apart.
///
/// The subtree is given back and the control group removed before this
/// returns, whatever the test found, so a failure here breaks one test rather
/// than every test after it.
fn as_a_service<T>(
    what: &str,
    doing: impl FnOnce(&Turns, &mut AsAMachineHasIt, &Cgroup, &AMachine) -> T,
) -> T {
    let machine = AMachine::with_something_worth_protecting(what);
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(what);

    let ours = Cgroup::made(&format!("alo-in-place-{what}-{}", std::process::id()))
        .expect("a control group can be made");
    let turns = Turns::under(ours.at()).expect("a service can make a subtree of its own");

    let found = doing(&turns, &mut kernel, &ours, &machine);

    turns
        .given_back()
        .expect("a service can be put back where it was");
    ours.removed()
        .expect("an empty control group can be taken away");
    machine.taken_away();
    found
}

/// One turn through the production door, whose work says whether it ran.
///
/// The work opens the key and the invoice and nothing else, and what it found
/// comes back out to be asserted; `ran` is set the moment the work starts, so
/// a refusal can be held to *nothing ran* rather than to *nothing came back*.
fn a_turn(
    turns: &Turns,
    boundary: &mut Boundary,
    named: &str,
    machine: &AMachine,
    ran: &Cell<bool>,
) -> Result<Went, NotBounded> {
    turns.doing(boundary, named, only(&machine.granted), || {
        ran.set(true);
        Went {
            control: opening(&machine.key),
            granted: opening(&machine.invoice),
        }
    })
}

/// The three things every refusal here has to leave behind: nothing.
///
/// The work never started, the turn's control group is not on the machine,
/// and the kernel holds no entry for anybody — so a machine that refused a
/// turn is exactly the machine it was before the turn was asked for.
fn nothing_was_left(ran: &Cell<bool>, ours: &Cgroup, named: &str, boundary: &Boundary) {
    assert!(
        !ran.get(),
        "the turn's work ran on a machine with no boundary"
    );
    assert!(
        !ours.at().join(named).exists(),
        "a refused turn left its control group behind"
    );
    assert_eq!(
        boundary
            .every_turn_the_kernel_is_holding()
            .expect("the map can be read"),
        Vec::<u64>::new(),
        "a refused turn left an entry in the kernel"
    );
}

/// The sentence says the boundary was not there, and where to look.
///
/// Every refusal here is read by whoever administers the machine, and it has
/// to send them somewhere: `docs/quirks.md` has the states this file measures
/// and what to check for each. It does not say the person did anything.
fn points_at_the_quirks(refused: &NotBounded) {
    let said = refused.to_string();
    assert!(said.contains("docs/quirks.md"), "{said}");
    assert!(said.contains("boundary"), "{said}");
}

/// **A machine whose boundary is in place is unaffected.** The control this
/// whole file rests on: the same door, the same fixture, nothing removed — the
/// turn runs, the key is refused inside it and the invoice opens, and when it
/// is over the kernel holds nothing.
#[test]
fn a_turn_runs_where_the_boundary_is_in_place() {
    let (went, ran) = as_a_service("in-place", |turns, kernel, _, machine| {
        let ran = Cell::new(false);
        let went = a_turn(turns, &mut kernel.boundary, "turn-in-place", machine, &ran)
            .expect("a turn on a machine with its boundary in place is bounded and runs");
        assert_eq!(
            kernel
                .boundary
                .every_turn_the_kernel_is_holding()
                .expect("the map can be read"),
            Vec::<u64>::new()
        );
        (went, ran.get())
    });

    assert!(ran, "the work did not run");
    assert_eq!(
        went.control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so this control proves nothing"
    );
    assert_eq!(
        went.granted,
        Outcome::Opened,
        "a bound turn was refused its own grant"
    );
}

/// **A hook whose pin is gone is a turn refused, by name, for every one of the
/// eighteen.** Removing a pin is the one thing on the machine that detaches a
/// hook, and a boundary with seventeen hooks is a boundary with a gap the
/// width of the eighteenth. The pins are taken in reverse order so that each
/// turn is refused over the pin just removed rather than over the first one
/// missing — which since 2026-09-13 means `inode_symlink`'s is the first
/// taken, and the list is read from `Pinned::every_hook_named` so that no line
/// here changed when it arrived, nor when the five before it did.
///
/// Until 2026-09-12 every one of these turns ran, and the key was refused only
/// while `file_open`'s own pin happened to be among those still there.
#[test]
fn a_turn_is_refused_when_the_programme_is_not_held_on_a_hook() {
    as_a_service("hook-not-held", |turns, kernel, ours, machine| {
        let hooks: Vec<(&'static str, PathBuf)> = kernel
            .pinned()
            .every_hook_named()
            .into_iter()
            .map(|(hook, at)| (hook, at.to_path_buf()))
            .collect();
        for (hook, at) in hooks.into_iter().rev() {
            fs::remove_file(&at)
                .unwrap_or_else(|why| panic!("{} could not be removed: {why}", at.display()));
            let named = format!("turn-without-{hook}");
            let ran = Cell::new(false);

            let refused = a_turn(turns, &mut kernel.boundary, &named, machine, &ran)
                .expect_err("a turn ran with a hook of its boundary detached");

            assert!(
                matches!(&refused, NotBounded::HookIsNotHeld { hook: named_hook, path, .. }
                    if *named_hook == hook && *path == at.display().to_string()),
                "{refused}"
            );
            assert!(!refused.a_thread_is_still_inside());
            assert!(refused.to_string().contains(hook), "{refused}");
            points_at_the_quirks(&refused);
            nothing_was_left(&ran, ours, &named, &kernel.boundary);
        }
    });
}

/// **The map of turns no longer pinned is a turn refused naming the loader.**
/// The pin the daemon itself opened at start is gone — a boundary taken away
/// under a running service — and the sentence is the one a machine reads when
/// `alo-boundaryd` never ran, because from the daemon's side those are the
/// same machine.
#[test]
fn a_turn_is_refused_when_the_map_is_not_pinned() {
    as_a_service("map-not-pinned", |turns, kernel, ours, machine| {
        let bounds = kernel.pinned().bounds().to_path_buf();
        fs::remove_file(&bounds).expect("the map's pin can be removed");
        let ran = Cell::new(false);

        let refused = a_turn(
            turns,
            &mut kernel.boundary,
            "turn-without-a-map",
            machine,
            &ran,
        )
        .expect_err("a turn ran with the map of turns no longer pinned");

        assert!(
            matches!(&refused, NotBounded::NoBoundaryHere { path } if *path == bounds.display().to_string()),
            "{refused}"
        );
        points_at_the_quirks(&refused);
        nothing_was_left(&ran, ours, "turn-without-a-map", &kernel.boundary);
    });
}

/// **A loader run again is a turn refused naming both maps.** Every pin is
/// taken away and made afresh, exactly as an operator restarting
/// `alo-boundaryd` by hand does it, so the programme now on every hook reads
/// a map this service never opened. The turn is refused with the two numbers
/// the kernel gives the maps, and the sentence says to restart the service.
///
/// **This is the reproduction that made the file.** Run against the crate
/// before `in_place` existed, this test failed with the turn *running*, the
/// key **opened** and the invoice opened: a bound turn with the run of the
/// machine, on a machine where every pin was in place and the daemon had
/// written its entry. A pin listing would have shown nothing wrong.
#[test]
fn a_turn_is_refused_when_the_loader_has_been_run_again() {
    as_a_service("loader-run-again", |turns, kernel, ours, machine| {
        let pinned: Pinned = kernel.pinned().clone();
        pinned.taken_away();
        pinned.made().expect("the pin directory can be made again");
        let _again = Imposed::once(&pinned)
            .unwrap_or_else(|why| panic!("the boundary could not be imposed a second time: {why}"));
        let ran = Cell::new(false);

        let refused = a_turn(
            turns,
            &mut kernel.boundary,
            "turn-under-a-stale-map",
            machine,
            &ran,
        )
        .expect_err("a turn ran under a map no programme reads");

        assert!(
            matches!(&refused, NotBounded::NotTheSameBoundary { path, held, pinned: at_the_pin }
                if *path == pinned.bounds().display().to_string() && held != at_the_pin),
            "{refused}"
        );
        assert!(
            refused.to_string().contains("restart alo-agentd"),
            "{refused}"
        );
        points_at_the_quirks(&refused);
        nothing_was_left(&ran, ours, "turn-under-a-stale-map", &kernel.boundary);

        // And the machine's own boundary — the second loader's — is untouched
        // by any of it: a fresh open of the pin is a boundary in place, so a
        // restarted service would run its turns bounded.
        let fresh = Boundary::opened(&pinned).expect("the boundary the second loader pinned opens");
        assert!(fresh.in_place().is_ok());
    });
}

/// **The whole boundary taken away is a service that runs no turn**, which is
/// the machine a loader never ran on, met after start rather than before it.
#[test]
fn a_turn_is_refused_when_the_boundary_has_been_taken_away() {
    as_a_service("taken-away", |turns, kernel, ours, machine| {
        kernel.pinned().taken_away();
        let ran = Cell::new(false);

        let refused = a_turn(
            turns,
            &mut kernel.boundary,
            "turn-with-nothing",
            machine,
            &ran,
        )
        .expect_err("a turn ran on a machine whose boundary had been taken away");

        assert!(
            matches!(refused, NotBounded::NoBoundaryHere { .. }),
            "{refused}"
        );
        points_at_the_quirks(&refused);
        nothing_was_left(&ran, ours, "turn-with-nothing", &kernel.boundary);
    });
}

/// **A service does not start where the map is pinned and a hook is not.**
/// The same question, asked at the door: a daemon that opened the map and
/// served would be one whose every turn was refused at the first verb, and the
/// answer belongs at start where it names the boot rather than a turn.
#[test]
fn a_service_does_not_start_where_a_hook_is_not_held() {
    let _order = on_this_kernel::one_at_a_time();
    let kernel = AsAMachineHasIt::on_this_kernel("no-start");
    let (hook, at) = kernel.pinned().every_hook_named()[6];
    let at = at.to_path_buf();
    fs::remove_file(&at).expect("a hook's pin can be removed");

    let refused = Boundary::opened(kernel.pinned())
        .map(drop)
        .expect_err("a service opened a boundary with a hook detached");

    assert!(
        matches!(&refused, NotBounded::HookIsNotHeld { hook: named, .. } if *named == hook),
        "{refused}"
    );
    assert_eq!(hook, "file_permission");
    points_at_the_quirks(&refused);
}

/// **There is no environment variable, and this test is what keeps it that
/// way.** The promise refuses a warning, and a variable that let a turn run
/// unbounded on a development machine would be the warning with a name. So
/// nothing in this crate reads the environment at run time at all — the one
/// `env!` there is, in `imposing.rs`, is the compiler's, resolved before the
/// binary exists, and is the spelling this test lets through.
#[test]
fn nothing_in_this_crate_reads_the_environment() {
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
                reads_the_environment(line).is_none(),
                "{} reads the environment with {}, and a boundary has no switch:\n{line}",
                file.display(),
                reads_the_environment(line).unwrap_or_default()
            );
        }
    }
    assert!(
        read >= 10,
        "only {read} source files were read, so this test is not looking at the crate"
    );
}

/// **The check above would catch something.** Every way the environment is
/// read in Rust is put in front of it, and so is the compile-time spelling it
/// has to let through.
#[test]
fn the_check_catches_every_way_the_environment_is_read() {
    for shape in [
        "if std::env::var(\"ALO_UNBOUNDED\").is_ok() {",
        "let permitted = env::var_os(\"ALO_NO_BOUNDARY\");",
        "for (name, value) in env::vars() {",
        "unsafe { libc::getenv(name) }",
        "std::env::vars_os().any(|(name, _)| name == \"ALO_DEGRADED\")",
    ] {
        assert!(
            reads_the_environment(shape).is_some(),
            "a line that reads the environment is not caught: {shape}"
        );
    }
    for ordinary in [
        "aya::include_bytes_aligned!(concat!(env!(\"OUT_DIR\"), \"/alo-bounding-kernel\"))",
        "/// A variable that let a turn run unbounded would be the warning with a name.",
        "let environment = Environment::of_this_turn();",
    ] {
        assert!(
            reads_the_environment(ordinary).is_none(),
            "an ordinary line was called a read of the environment: {ordinary}"
        );
    }
}

/// Which way of reading the environment a line of Rust names, if it names one.
///
/// `env::var` covers `var`, `var_os`, `vars` and `vars_os`; `getenv` is the C
/// library's. `env!` is not on the list because it is not a read: the
/// compiler resolves it, and a binary cannot be told anything through it.
fn reads_the_environment(line: &str) -> Option<&'static str> {
    ["env::var", "getenv"]
        .into_iter()
        .find(|reading| line.contains(reading))
}

/// Whether a line of Rust says something to a reader rather than to the
/// machine.
fn is_a_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}
