//! A command runs inside its box, and the kernel refuses what the box was not
//! given.
//!
//! This is the test ADR 0064's sealed box exists to pass: *inside a box that
//! sees the project and nothing else.* Everything else in this crate is
//! arithmetic over paths; this is the only file that asks a kernel.
//!
//! # Both halves, because one of them proves nothing alone
//!
//! - **A command reaches what it was given.** Without this, a box that refused
//!   everything would pass the test below and be useless.
//! - **A command is refused what it was not given**, by the kernel, with
//!   `EACCES`.
//!
//! The two runs differ **only** in which folder was named in the bound. Same
//! command, same argument, same box, same machine, one line apart — so what is
//! being measured is the bound and not the weather.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! It fails loudly on a machine without them rather than skipping, which is
//! ADR 0015's rule and `alo-bounding`'s: a test that quietly skipped would
//! report green on every machine where the boundary does nothing at all.
//!
//! # The fixture is `alo-bounding`'s own, included rather than copied
//!
//! `on_this_kernel/mod.rs` imposes a boundary the way a booted machine has one
//! — the loader's half and the daemon's half over one programme — and its own
//! documentation says why it is shared: *a fixture that drifted between three
//! files would be three different machines being tested.* That argument does
//! not stop at a crate boundary, so this includes that file rather than keeping
//! a fourth copy of it.
//!
//! **What it costs is a coupling**, named here so the next reader is not
//! surprised: this test reads a path into another crate's tests, and a lane
//! that moves that file breaks this. The alternative was a copy that would
//! drift silently, and a copy that drifts is the worse of the two because
//! nothing announces it.
//!
//! # Why the command is `/bin/cat` and the bound has three places in it
//!
//! `exec` happens after the admission, so the dynamic loader and every library
//! the command needs are opened **inside** the boundary. A box granted only the
//! folder being read cannot run a dynamically linked command at all: it dies in
//! the loader, before its first instruction, and the test would then show a
//! refusal that had nothing to do with what it was reading.
//!
//! So the bound names the command's folder and the library folder as well as
//! the folder under test, which is what a real sealed box has to do too and is
//! why `docs/quirks.md` carries it.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use alo_capability::HowCodeRuns;
use alo_record::Record;
use alo_running::{AskedToRun, NotRun, WhereCommandsRun};
use alo_strings::Strings;

#[path = "../../alo-bounding/tests/on_this_kernel/mod.rs"]
mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// The program this crate builds, as cargo tells a test where it put it.
///
/// Not [`alo_running::THE_HELPER`], which is where a machine running alo OS has
/// it installed — a path that does not exist on a machine running these tests.
const HELPER: &str = env!("CARGO_BIN_EXE_alo-enter-the-box");

/// Where the libraries a dynamically linked command needs live on this machine.
///
/// Asked of the machine rather than assumed, because `/lib` is a link to
/// `/usr/lib` on some machines and a folder on others, and a bound made of the
/// link would be a bound around the link.
fn where_the_libraries_are() -> PathBuf {
    for maybe in ["/usr/lib", "/lib"] {
        let path = Path::new(maybe);
        if path.is_dir() {
            return path
                .canonicalize()
                .unwrap_or_else(|why| panic!("{maybe} is a folder and would not resolve: {why}"));
        }
    }
    panic!("this machine has no library folder, so no dynamically linked command can run");
}

/// A folder of this test's own with one file in it, and the file's path.
fn a_folder_with_a_file_in_it(what: &str) -> (PathBuf, PathBuf) {
    let folder = std::env::temp_dir().join(format!("alo-running-{}-{what}", std::process::id()));
    drop(fs::remove_dir_all(&folder));
    fs::create_dir_all(&folder).expect("a temporary folder can be made");
    let file = folder.join("what-it-says");
    fs::write(&file, b"this is what the file says\n").expect("a file can be written");
    (folder, file)
}

/// Where boxes are made for this test: an ordinary control group at the root of
/// the hierarchy, which is what `alo-bounding`'s own kernel tests use.
///
/// **Not a folder under a turn root**, which cannot work: `Turns` makes its
/// `home` threaded, that makes the root above it `domain threaded`, and a
/// domain group beside it comes out `domain invalid` and refuses a process.
/// Measured on 2026-10-16 and recorded in `docs/quirks.md`.
fn where_boxes_go() -> PathBuf {
    let at = PathBuf::from("/sys/fs/cgroup").join(format!("alo-running-{}", std::process::id()));
    fs::create_dir_all(&at).expect(
        "a control group can be made, which needs root and cgroup v2 — without it nothing \
         below is being tested",
    );
    at
}

/// What came of one run: whether the command was happy, and what the record
/// says about it.
struct WhatCameOfIt {
    /// Whether the command itself reported success.
    happy: bool,
    /// Everything the run wrote down.
    kept: Record,
}

/// Reads `file` in a box bound to `may_reach`, and says what came of it.
///
/// **It keeps the record the run wrote**, which is the half of *every execution
/// and every refusal leaves a record* that needs a real boundary — the refusal
/// half runs on any machine and lives in
/// `every_run_and_every_refusal_is_written_down.rs`.
fn reading_in_a_box(
    named: &str,
    file: &Path,
    may_reach: Vec<PathBuf>,
) -> Result<WhatCameOfIt, NotRun> {
    let _order = on_this_kernel::one_at_a_time();
    let mut machine = AsAMachineHasIt::on_this_kernel(named);
    let boxes = where_boxes_go();

    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).expect("the capability words declare");
    let strings = Strings::of(vocabulary);

    let asked = AskedToRun {
        chose: HowCodeRuns::InASealedBox,
        program: PathBuf::from("/bin/cat"),
        arguments: vec![file.as_os_str().to_owned()],
        may_reach,
    };

    let mut kept = Record::default();
    let what_happened = {
        let mut where_they_run = WhereCommandsRun::with_the_helper_at(
            &mut machine.boundary,
            &boxes,
            &mut kept,
            Path::new(HELPER),
        );
        where_they_run.ran(
            &alo_capability::Grantee::named("@terminal"),
            named,
            &asked,
            &strings,
            SystemTime::now(),
        )
    };

    drop(fs::remove_dir_all(&boxes));
    what_happened.map(|happened| WhatCameOfIt {
        happy: happened.the_command_was_happy(),
        kept,
    })
}

/// **A command reaches what its box was given.**
///
/// The control. Without it the refusal below would be satisfied by a box that
/// refused everything, which is every broken box there is.
#[test]
fn a_command_reaches_what_its_box_was_given() {
    let (folder, file) = a_folder_with_a_file_in_it("granted");

    let came_of_it = reading_in_a_box(
        "granted",
        &file,
        vec![
            folder.clone(),
            PathBuf::from("/usr/bin"),
            where_the_libraries_are(),
        ],
    )
    .expect("a run bound to the folder it reads should have happened");

    assert!(
        came_of_it.happy,
        "a command bound to the folder it reads could not read it, so the bound is wrong \
         rather than the refusal below being right"
    );

    // **And the run was written down**, which is the other half of the
    // guarantee and the half that needs a kernel to reach.
    let kept: Vec<_> = came_of_it.kept.everything().collect();
    assert_eq!(kept.len(), 1, "a run left {} entries", kept.len());
    let entry = kept.first().expect("a run left one entry");
    assert!(matches!(
        entry.happened(),
        alo_record::Happened::RanACommand {
            the_kernel_was_holding_it: true,
            ..
        }
    ));
    assert!(
        !entry.happened().was_stopped(),
        "a command that ran was written down as a refusal"
    );

    drop(fs::remove_dir_all(&folder));
}

/// **And the kernel refuses what its box was not given.**
///
/// Everything is the same as above except the folder being read is not in the
/// bound. `cat` then cannot open it and says so, which is the kernel's `EACCES`
/// arriving as a failed command.
#[test]
fn the_kernel_refuses_what_its_box_was_not_given() {
    let (folder, file) = a_folder_with_a_file_in_it("refused");
    let (elsewhere, _) = a_folder_with_a_file_in_it("elsewhere");

    let came_of_it = reading_in_a_box(
        "refused",
        &file,
        vec![
            // Everything the command needs to *start*, and not the folder it
            // was asked to read — so a failure here is about the bound and not
            // about a loader that could not open a library.
            elsewhere.clone(),
            PathBuf::from("/usr/bin"),
            where_the_libraries_are(),
        ],
    )
    .expect("a run bound to somewhere else should still have happened");

    assert!(
        !came_of_it.happy,
        "a command read a file its box was never given, so the boundary is not holding"
    );

    // It ran and was refused by the kernel, so it is a **run** in the record
    // with a failing status — not a refusal, which is what a request that never
    // started would be.
    let kept: Vec<_> = came_of_it.kept.everything().collect();
    assert_eq!(kept.len(), 1);
    assert!(matches!(
        kept.first().expect("a run left one entry").happened(),
        alo_record::Happened::RanACommand { .. }
    ));

    drop(fs::remove_dir_all(&folder));
    drop(fs::remove_dir_all(&elsewhere));
}
