//! What a turn inherits from the process it is a thread of, measured rather
//! than assumed.
//!
//! `what_a_bound_turn_can_still_change.rs` is the list of filesystem mutations
//! no hook watches, and `what_a_bound_turn_can_still_reach.rs` is the network's.
//! **This is the third list and it is not made of hooks at all**: a boundary on
//! `file_open` decides at the moment of opening and says nothing afterwards, so
//! every descriptor that existed before a turn began is a descriptor the
//! boundary was never asked about.
//!
//! # Why this file uses the real door and not a child process
//!
//! Its two siblings put a child process in a control group, because what they
//! measure is a hook's answer and a child is the simplest thing to bind. What
//! *this* measures is inheritance, and inheritance is exactly what a child
//! process gets differently: `alo-agentd` does not start anything (law 2, and
//! `crates/alo-bounding/src/turns.rs` has the argument) — a turn is **one thread
//! of the daemon**, sharing the daemon's whole descriptor table for as long as
//! it runs.
//!
//! So every test here goes through [`alo_bounding::Turns::doing`], which is the
//! production door, on the thread the assertions are made from. What a
//! descriptor opened before `doing` was called can do inside it is what a verb
//! with a bug in it could do to the daemon's own record, socket and way out.
//!
//! # The rows this file reproduces, in the words the table uses
//!
//! - `a file open for reading` — every byte of it stays readable inside a turn
//!   that is refused the same file by name, **and what it reads can be written
//!   into the folder the turn was granted.** That is contents leaving a grant,
//!   which is the one thing nothing on the unwatched-mutations list can do.
//! - `a file open for appending` — the shape of the machine's own record, which
//!   `alo_keeping::Writing` holds open for the life of the daemon. A turn can add
//!   a line to it and cannot reopen it.
//! - `a directory descriptor` — `openat` relative to one is still an open, so a
//!   folder handle is not a key to what is in it.
//! - `the way out of a turn` — `Turns::back` is `home/cgroup.threads`, opened
//!   before the first turn ever ran because opening it from inside is an open the
//!   boundary correctly refuses. This is not only a gap: it is the reason closing
//!   the others is a decision rather than a patch, since a boundary that
//!   re-decided about a descriptor at the moment it was *used* would refuse a
//!   turn its own way out.
//!
//! The fifth row of the table, `a socket already connected`, is reproduced in
//! `what_a_bound_turn_can_still_reach.rs` and is not repeated here.
//!
//! One thing measured here is not a row at all, because it is a refusal rather
//! than a gap: **the name the kernel gives a descriptor.** `/proc/self/fd/<n>`
//! is how a descriptor becomes a path again, and an open through one is watched
//! like any other — with the *granted* file reopened the same way as its control,
//! because a boundary that refused everything under `/proc` would look identical
//! and mean nothing.
//!
//! `docs/quirks.md` carries the account with the release that owns closing each
//! row, and `what_a_turn_inherits_is_written_down.rs` holds that table to this
//! file and to the programme.
//!
//! # The control comes first, and the legitimate open beside it
//!
//! A turn that read something proves nothing if the boundary was never applied.
//! So every run below opens the file nobody granted — which must be refused with
//! `EACCES` — and the file somebody did — which must be allowed — and no result
//! is believed unless both happened.
//!
//! # Nothing is asserted from inside a turn
//!
//! `a_turn_is_this_thread.rs`'s rule, for the same reason: a failing assertion
//! inside the boundary panics, a panic prints a backtrace, and a backtrace opens
//! `/proc/self/maps` — an open outside the grant, refused, in the middle of
//! reporting why something else went wrong. Everything here is gathered inside
//! and judged outside.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! The same as its siblings, and it fails loudly on a machine without them
//! rather than skipping itself.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    os::fd::AsRawFd as _,
    path::{Path, PathBuf},
};

use alo_bounding::{Bounds, Cgroup, Turns, place_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// What is in the file nobody granted.
const A_KEY: &str = "not a real key";

/// What the daemon had already written into the record.
const A_LINE_THE_DAEMON_WROTE: &str = "{\"kept\":\"an execution nobody disputes\"}\n";

/// What a turn adds to it through a descriptor nobody asked it about.
const A_LINE_NO_VERB_WROTE: &str = "{\"kept\":\"an execution that never happened\"}\n";

/// The file a thread joins or leaves a control group by writing into.
const THE_THREADS: &str = "cgroup.threads";

/// What one attempt came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The machine allowed it.
    Allowed,

    /// The machine refused, with the number it gave.
    Refused(i32),
}

/// What the machine made of one attempt.
fn went(done: std::io::Result<()>) -> Outcome {
    match done {
        Ok(()) => Outcome::Allowed,
        Err(why) => Outcome::Refused(why.raw_os_error().unwrap_or(0)),
    }
}

/// Opens a file by name and says what the machine made of it.
fn opening(what: &Path) -> Outcome {
    went(File::open(what).map(drop))
}

/// What every run reports: the two that say the boundary was in force, and the
/// two the test is actually about.
#[derive(Debug)]
struct Went {
    /// An open, by name, of the file nobody granted. Must be refused.
    control: Outcome,

    /// An open, by name, of the file somebody granted. Must be allowed.
    granted: Outcome,

    /// What the inherited descriptor itself did.
    subject: Outcome,

    /// What that descriptor could not be turned into.
    after: Outcome,
}

/// A folder that is granted, a file in it, a private folder nobody granted with
/// something worth protecting in it, and a record beside them both.
struct AMachine {
    /// Everything this test made, for taking away afterwards.
    root: PathBuf,

    /// The one folder a turn is bound to.
    granted: PathBuf,

    /// A file inside it, which every run must be allowed to open.
    invoice: PathBuf,

    /// A folder nobody granted.
    private: PathBuf,

    /// A file in it, which every run must be refused by name.
    key: PathBuf,

    /// The machine's record, outside the grant, of the shape
    /// `alo_keeping::Writing` holds open for the life of the daemon.
    record: PathBuf,
}

impl AMachine {
    /// One, named after the test that made it.
    fn with_something_worth_protecting(what: &str) -> Self {
        let root = PathBuf::from("/tmp").join(format!("alo-inherit-{}-{what}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for folder in ["Invoices", "Private"] {
            fs::create_dir_all(root.join(folder)).expect("a temporary directory can be made");
        }
        let invoice = root.join("Invoices/march.pdf");
        let key = root.join("Private/id_ed25519");
        let record = root.join("record.jsonl");
        fs::write(&invoice, b"an invoice").expect("a file can be written");
        fs::write(&key, A_KEY).expect("a file can be written");
        fs::write(&record, A_LINE_THE_DAEMON_WROTE).expect("a file can be written");
        Self {
            granted: root.join("Invoices"),
            invoice,
            private: root.join("Private"),
            key,
            record,
            root,
        }
    }

    /// Take it away, whatever the test found.
    fn taken_away(&self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A bound over the one granted folder, which is what every test here grants.
fn only(folder: &Path) -> Bounds {
    Bounds::of_one(place_of(folder).expect("the granted folder is there"))
}

/// One turn, in the shape `alo-agentd` really runs one, with something opened
/// before it began.
///
/// `beforehand` runs **outside** any boundary and produces whatever the turn
/// will inherit; `inside` runs on this thread with the kernel deciding every
/// open it makes, and answers with the two outcomes this test is about. The two
/// that say the boundary was in force are taken here so that no test can forget
/// them.
///
/// The service's subtree is given back and its control group removed before this
/// returns, whatever the test found: a run that left this process inside a
/// cgroup it had removed would break every test after it rather than the one
/// that failed.
fn a_turn_that_inherited<T>(
    what: &str,
    machine: &AMachine,
    beforehand: impl FnOnce(&Turns, &AMachine) -> T,
    inside: impl FnOnce(&mut T, &AMachine) -> (Outcome, Outcome),
) -> Went {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(what);

    let ours = Cgroup::made(&format!("alo-inherit-{what}-{}", std::process::id()))
        .expect("a control group can be made");
    let turns = Turns::under(ours.at()).expect("a service can make a subtree of its own");

    // Opened before the thread is in any boundary, which is the whole subject.
    let mut held = beforehand(&turns, machine);

    let went = turns
        .doing(
            &mut kernel.boundary,
            &format!("turn-{what}"),
            only(&machine.granted),
            || {
                let control = opening(&machine.key);
                let granted = opening(&machine.invoice);
                let (subject, after) = inside(&mut held, machine);
                Went {
                    control,
                    granted,
                    subject,
                    after,
                }
            },
        )
        .expect(
            "the turn can be bounded, and left again through the descriptor opened before it \
             began — see the way-out test in this file",
        );

    drop(held);
    turns
        .given_back()
        .expect("a service can be put back where it was");
    ours.removed()
        .expect("an empty control group can be taken away");
    went
}

/// The two things every test here asserts before it asserts anything of its own.
///
/// The control is what says the boundary was in force at all, and the granted
/// open is what says it was not simply refusing everything.
fn the_boundary_was_in_force(went: &Went) {
    assert_eq!(
        went.control,
        Outcome::Refused(13),
        "the boundary was not in force, so nothing this turn did means anything"
    );
    assert_eq!(
        went.granted,
        Outcome::Allowed,
        "a turn was refused the file it was granted, which is a regression rather than a gap \
         being reproduced"
    );
}

/// What is in a file, read from outside the turn.
fn holds(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{} is not readable: {why}", path.display()))
}

/// **A file opened before the turn began is readable inside it, whole — and
/// what it reads can be written where the grant allows.**
///
/// The gap at its simplest, and then the thing that makes it matter. The same
/// thread, in the same instant, is refused `open` on the private key with
/// `EACCES` and reads every byte of it through a descriptor that already
/// existed. `file_open` decided once, before there was a turn to decide about,
/// and there is no second hook that asks again.
///
/// **So an inherited descriptor is a way for contents to leave a grant**, which
/// is the one property every unwatched filesystem mutation was measured against
/// and found not to have. The turn writes the key into the folder it *was*
/// granted, where a `move_file` or an `archive_folder` could carry it onwards
/// and where the record would name only a granted path. Nothing in the list in
/// `what_a_bound_turn_can_still_change.rs` does that; this does, and it is the
/// reason this is its own piece of work rather than a row in that table.
///
/// Nothing alo OS ships hands a verb a descriptor — the six verbs take paths,
/// and `alo-files` opens what it opens from inside the boundary. This is the
/// floor under a verb with a bug in it, which is what ADR 0013 says the boundary
/// is for, and the floor has a hole in it of exactly this shape.
#[test]
fn a_file_opened_before_the_turn_began_is_still_readable_inside_it() {
    let machine = AMachine::with_something_worth_protecting("read");
    let copied = machine.granted.join("copied.txt");
    let mut said = String::new();
    let went = a_turn_that_inherited(
        "read",
        &machine,
        |_, machine| File::open(&machine.key).expect("the key opens outside any turn"),
        |held, _| {
            let read = went(held.read_to_string(&mut said).map(drop));
            // And out again, into the folder this turn really was granted.
            let out = went(fs::write(&copied, said.as_bytes()));
            (read, out)
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "a read through an inherited descriptor was refused, which means something now decides \
         about a descriptor after it is opened — say so in crates/alo-bounding/src/lib.rs and in \
         the table in docs/quirks.md, and turn this into the refusal it should be"
    );
    assert_eq!(
        said, A_KEY,
        "the read was allowed and produced something other than the file's contents, so this \
         test is measuring something it is not named after"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a turn was refused a write inside the folder it was granted, which is what \
         `archive_folder` does — a regression rather than a gap being reproduced"
    );
    assert_eq!(
        holds(&copied),
        A_KEY,
        "the contents of a file nobody granted did not reach the folder somebody did, so this \
         test proves nothing about contents leaving a grant"
    );

    machine.taken_away();
}

/// **A file opened for appending before the turn began can be appended to
/// inside it — and this is the record's own shape.**
///
/// `alo_keeping::Writing` opens the machine's record with `append(true)` when
/// the daemon starts and holds it for the life of the process. A turn is a
/// thread of that process, so that descriptor is in the turn's table for as long
/// as the turn runs, and no hook of this boundary is consulted about a write.
///
/// What that permits is a line in the record that no execution caused. What it
/// does **not** permit is anything else about the record: `O_APPEND` puts every
/// write at the end, so nothing already written can be altered, and opening the
/// record by name — to read it, to truncate it, to open it a second time without
/// `O_APPEND` — is an open outside the bound and is refused. Both halves are
/// measured here, and the second is what keeps this a gap in *addition* rather
/// than a way to rewrite history.
#[test]
fn a_record_opened_before_the_turn_began_is_still_appendable_inside_it() {
    let machine = AMachine::with_something_worth_protecting("append");
    let went = a_turn_that_inherited(
        "append",
        &machine,
        |_, machine| {
            OpenOptions::new()
                .append(true)
                .open(&machine.record)
                .expect("the record opens outside any turn")
        },
        |held, machine| {
            let appended = went(
                held.write_all(A_LINE_NO_VERB_WROTE.as_bytes())
                    .and_then(|()| held.flush()),
            );
            (appended, opening(&machine.record))
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "an append through the descriptor the daemon holds the record open with was refused, \
         which means something now decides about a write — say so in \
         crates/alo-bounding/src/lib.rs and in the table in docs/quirks.md"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "a bound turn opened the machine's record by name, so it could read or replace the \
         record rather than only add to it — that is a wider hole than the one this test is \
         named after"
    );
    assert_eq!(
        holds(&machine.record),
        format!("{A_LINE_THE_DAEMON_WROTE}{A_LINE_NO_VERB_WROTE}"),
        "the line a turn wrote is not at the end of the record with the daemon's line still \
         whole in front of it, so this measured something other than an append"
    );

    machine.taken_away();
}

/// **A descriptor cannot be turned back into an open through the name the
/// kernel gives it.**
///
/// `/proc/self/fd/<n>` is how a descriptor becomes a path again, and reopening
/// through one is a `file_open` like any other: the boundary walks up from the
/// file the open really reached, so the private key is refused there exactly as
/// it is by its own name.
///
/// **The control for that is the granted file reopened the same way**, and it is
/// not decoration. Everything under `/proc` is outside the bound, so a boundary
/// that simply refused `/proc/self/fd/<n>` on sight would produce this same
/// refusal while proving nothing about where the walk starts. The invoice's own
/// descriptor reopens, so the walk really does follow the link to the file it
/// names — which is what makes the refusal above a statement about the key.
#[test]
fn an_inherited_descriptor_cannot_be_reopened_through_the_name_the_kernel_gives_it() {
    let machine = AMachine::with_something_worth_protecting("reopen");
    let went = a_turn_that_inherited(
        "reopen",
        &machine,
        |_, machine| {
            (
                File::open(&machine.key).expect("the key opens outside any turn"),
                File::open(&machine.invoice).expect("the invoice opens outside any turn"),
            )
        },
        |held, _| {
            let (key, invoice) = held;
            (
                opening(Path::new(&format!("/proc/self/fd/{}", key.as_raw_fd()))),
                opening(Path::new(&format!("/proc/self/fd/{}", invoice.as_raw_fd()))),
            )
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(13),
        "a bound turn reopened a file nobody granted it through the name the kernel gives its \
         descriptor, which would make every inherited descriptor a way to open its file afresh \
         — with truncation, and outside the record of what the turn was allowed to reach"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "the *granted* file was refused through the same kind of name, so the refusal above is \
         about /proc rather than about where the file is — which would mean this test proves \
         nothing about the boundary's walk"
    );
    assert_eq!(holds(&machine.key), A_KEY, "the key was disturbed");

    machine.taken_away();
}

/// **A directory opened before the turn began is not a key to what is in it.**
///
/// A folder handle is the one inherited descriptor that looks like it should
/// widen a grant: `openat` takes a directory and a name, and the name never
/// touches the boundary's map. It does not widen anything, because `openat` is
/// still an open — the hook is handed the file that was opened and walks up from
/// there, and the base descriptor is not part of the question.
///
/// So what a turn inherits from a folder handle is the folder, not its contents:
/// the handle stays valid, and every file it can name is refused.
#[test]
fn a_directory_opened_before_the_turn_began_is_not_a_key_to_what_is_in_it() {
    let machine = AMachine::with_something_worth_protecting("directory");
    let went = a_turn_that_inherited(
        "directory",
        &machine,
        |_, machine| File::open(&machine.private).expect("the folder opens outside any turn"),
        |held, _| {
            let through = rustix::fs::openat(
                &*held,
                "id_ed25519",
                rustix::fs::OFlags::RDONLY,
                rustix::fs::Mode::empty(),
            );
            let opened = went(
                through
                    .map(drop)
                    .map_err(|why| std::io::Error::from_raw_os_error(why.raw_os_error())),
            );
            // And the handle itself is still a handle, so the refusal above is
            // the boundary rather than a descriptor that had gone stale.
            let still_there = went(held.metadata().map(drop));
            (opened, still_there)
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(13),
        "a bound turn opened a file nobody granted it by naming it relative to a folder handle \
         it inherited, which would make one directory descriptor a grant over everything \
         beneath it"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "the inherited folder handle was not usable at all, so the refusal above may be about a \
         broken descriptor rather than about the boundary"
    );

    machine.taken_away();
}

/// **The way out of a turn is itself a descriptor opened before the turn
/// began**, which is why closing this gap is a decision rather than a patch.
///
/// `Turns::back` is `home/cgroup.threads`, opened when the service started and
/// held for the life of the daemon, and leaving a boundary is a write to it.
/// That arrangement exists because the alternative does not work: opening that
/// file from inside is an open outside the grant, and this test measures the
/// kernel refusing exactly that, with `EACCES`, while the turn is running.
///
/// **The turn nevertheless left**, which is what reaching these assertions
/// proves — `a_turn_that_inherited` expects `doing` to return, and `doing`
/// returns only after `Inside::leaving` has written a byte through that
/// descriptor and the service has been put back where it was.
///
/// So a boundary that re-decided about a descriptor at the moment it was used
/// would refuse a turn its own way out, and the fix is not "check on every
/// write". `docs/quirks.md` and this workstream's report say what the real
/// answers are and which of them needs an ADR.
#[test]
fn the_way_out_of_a_turn_is_a_descriptor_the_boundary_would_refuse_to_open() {
    let machine = AMachine::with_something_worth_protecting("way-out");
    let went = a_turn_that_inherited(
        "way-out",
        &machine,
        |turns, _| {
            let home = turns.home().at().to_path_buf();
            let root = home
                .parent()
                .expect("home is a folder inside the service's own control group")
                .to_path_buf();
            (
                home.join(THE_THREADS),
                root.join("turn-way-out").join(THE_THREADS),
            )
        },
        |back, _| {
            let (home, own) = back;
            (
                went(OpenOptions::new().write(true).open(&*home).map(drop)),
                // And the turn's own, for the same reason: a turn that could
                // open one of these could put itself anywhere in the hierarchy.
                went(OpenOptions::new().write(true).open(&*own).map(drop)),
            )
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(13),
        "a bound turn opened the file threads leave a boundary through, which would let a verb \
         with a bug in it put itself back among the daemon's own threads and finish its work \
         outside every grant"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(13),
        "a bound turn opened its own control group's `cgroup.threads`, so it could move itself \
         out of the boundary it is in without any descriptor being inherited at all"
    );

    machine.taken_away();
}
