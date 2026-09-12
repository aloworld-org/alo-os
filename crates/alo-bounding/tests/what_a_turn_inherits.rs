//! What a turn inherits from the process it is a thread of, and what the
//! boundary says about each of those things now — measured rather than
//! assumed, and measured in the other direction until 2026-09-12.
//!
//! `what_a_bound_turn_can_still_change.rs` is the list of filesystem mutations
//! no hook watches, and `what_a_bound_turn_can_still_reach.rs` is the network's.
//! **This is the third list and it was not made of hooks at all**: a boundary
//! on `file_open` decides at the moment of opening and says nothing afterwards,
//! so every descriptor that existed before a turn began was a descriptor the
//! boundary was never asked about. This file reproduced that — the same thread
//! refused `open` on a private key and reading every byte of it through a
//! descriptor that already existed, then writing what it read into the folder
//! it *was* granted — and it was the one gap in this crate that moved contents
//! past a grant.
//!
//! **Since 2026-09-12 the boundary is asked on every use.** `file_permission`
//! runs on every read and write on the machine, asks whose thread is reading,
//! and walks up from the file's own directory entry exactly as an open would;
//! `crates/alo-bounding-kernel/src/deciding.rs` has the whole of it. So every
//! assertion below that used to say *reaches* now says *refused*, with the
//! number the kernel gave, and beside each of them is the use the same hook
//! must not break: a descriptor to a file inside the grant, a listing of a
//! folder inside it, and a socket, which is left to the hook that can read
//! where its bytes are going.
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
//! - `a file open for reading` — refused the first byte, and nothing of it
//!   reaches the folder the turn was granted. This is the row that moved
//!   contents past a grant, and the one the hook exists for.
//! - `a file open for writing` — refused, and the file is as it was.
//! - `a file open for appending` — the shape of the machine's own record,
//!   which `alo_keeping::Writing` holds open for the life of the daemon. A turn
//!   used to be able to add a line to it; now it cannot, and it still cannot
//!   reopen it.
//! - `a directory descriptor` — `openat` relative to one is still an open and
//!   still refused, and now listing it is refused too, while a descriptor to
//!   a folder *inside* the grant lists as it always did.
//! - `the way out of a turn` — `home/cgroup.threads`, which used to be written
//!   by the turn's own thread through a descriptor opened before the first
//!   turn ever ran. That write is refused now, and the turn ends anyway,
//!   because a thread of the service that is not in a turn brings it home;
//!   `crates/alo-bounding/src/inside.rs` has the arrangement. A verb with a
//!   bug in it can no longer end its own boundary early.
//! - `a socket already connected` — reproduced in
//!   `what_a_bound_turn_can_still_reach.rs`, where `socket_sendmsg` closed it
//!   first. What this file adds is the carve-out on the file hook: a write on
//!   a Unix socket the daemon inherited is allowed *here*, so that the hook
//!   that reads where a message is going is the one that decides about it.
//! - `a pipe` — left alone, for the reason a Unix socket is: it holds no
//!   contents of its own. Every sibling of this file that binds a child
//!   process talks to it over one from inside the turn, which is how the
//!   refusal of a pipe was found on the day the hook landed.
//!
//! One thing measured here is not a row at all, because it was always a
//! refusal: **the name the kernel gives a descriptor.** `/proc/self/fd/<n>` is
//! how a descriptor becomes a path again, and an open through one is watched
//! like any other — with the *granted* file reopened the same way as its
//! control, because a boundary that refused everything under `/proc` would look
//! identical and mean nothing.
//!
//! And one thing is deliberately **not** measured here, because it cannot be
//! honestly: a mapping. `mmap` of a file is `mmap_file`, not a read, and it is
//! not hooked — but there is no safe spelling of `mmap` in Rust and `unsafe` is
//! forbidden outside `alo-bounding-kernel`'s one file, so the committed suite
//! cannot reproduce it. `docs/quirks.md` says so beside the table, and
//! `what_a_turn_inherits_is_written_down.rs` holds that table to this file and
//! to the programme.
//!
//! # The control comes first, and the legitimate open beside it
//!
//! A turn that was refused something proves nothing if the boundary was
//! refusing everything, and a turn that was allowed something proves nothing if
//! the boundary was never applied. So every run below opens the file nobody
//! granted — which must be refused with `EACCES` — and the file somebody did —
//! which must be allowed — and no result is believed unless both happened.
//!
//! # Nothing is asserted from inside a turn
//!
//! `a_turn_is_this_thread.rs`'s rule, for the same reason and one more: a
//! failing assertion inside the boundary panics, a panic prints a backtrace, and
//! a backtrace opens `/proc/self/maps` — an open outside the grant, refused, in
//! the middle of reporting why something else went wrong. And since the kernel
//! decides about descriptors too, the panic's own message to the terminal this
//! test inherited is refused as well. Everything here is gathered inside and
//! judged outside.
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
    os::{fd::AsRawFd as _, unix::net::UnixStream},
    path::{Path, PathBuf},
};

use alo_bounding::{Bounds, Cgroup, Turns, place_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// What is in the file nobody granted.
const A_KEY: &str = "not a real key";

/// What is in the file somebody did.
const AN_INVOICE: &str = "an invoice";

/// What the daemon had already written into the record.
const A_LINE_THE_DAEMON_WROTE: &str = "{\"kept\":\"an execution nobody disputes\"}\n";

/// What a turn tries to add to it through a descriptor nobody asked it about.
const A_LINE_NO_VERB_WROTE: &str = "{\"kept\":\"an execution that never happened\"}\n";

/// What a turn tries to put in a file it was never granted.
const A_REPLACEMENT: &str = "a key somebody else chose";

/// What a turn adds to the file it was granted, through a descriptor it
/// inherited.
const A_NOTE: &str = " — paid";

/// What a turn says to the person, on the socket the daemon inherited.
const AN_ANSWER: &[u8] = b"{\"answered\":\"from inside the turn\"}";

/// What a bound child says to the test that is waiting for it.
const A_WORD: &[u8] = b"alo: refused 13\n";

/// The file a thread joins or leaves a control group by writing into.
const THE_THREADS: &str = "cgroup.threads";

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

/// Reads the first entry of a folder through a descriptor that already
/// existed, and says what the machine made of it.
///
/// `getdents` and nothing else: [`rustix::fs::Dir::new`] takes the descriptor
/// itself rather than reopening the folder through it, so the only thing the
/// kernel is asked about is the read. A reopen would be an `open`, which is
/// the sibling test's subject and not this one's.
fn listing(folder: File) -> Outcome {
    match rustix::fs::Dir::new(folder) {
        Err(why) => Outcome::Refused(why.raw_os_error()),
        Ok(mut entries) => match entries.next() {
            Some(Err(why)) => Outcome::Refused(why.raw_os_error()),
            Some(Ok(_)) | None => Outcome::Allowed,
        },
    }
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

    /// What the same run measured beside it.
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
        fs::write(&invoice, AN_INVOICE).expect("a file can be written");
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
/// open, read and write it makes, and answers with the two outcomes this test
/// is about. The two that say the boundary was in force are taken here so that
/// no test can forget them.
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
            "the turn can be bounded, and brought home again by a thread of the service that \
             was never in it — see the way-out test in this file",
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
        Outcome::Refused(REFUSED),
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

/// **A file opened before the turn began is refused inside it, at the first
/// byte — and nothing of it reaches the folder the turn was granted.**
///
/// The gap at its simplest, closed. Until 2026-09-12 this test asserted the
/// opposite: the same thread, in the same instant, refused `open` on the
/// private key with `EACCES` and reading every byte of it through a descriptor
/// that already existed, then writing the key into the folder it *was* granted
/// — where a `move_file` or an `archive_folder` could carry it onwards and
/// where the record would name only a granted path. That was contents leaving
/// a grant, the one thing nothing on the unwatched-mutations list can do, and
/// the reason this was its own piece of work.
///
/// `file_permission` decides now, on the read, asked of the thread reading:
/// the key's descriptor is walked up from the key's own directory entry, meets
/// no granted place, and the read fails with `EACCES` before a byte has
/// moved. The write into the granted folder is still allowed — it is the
/// legitimate half, and what `archive_folder` does — and what it writes is
/// nothing, because nothing was read.
#[test]
fn a_file_opened_before_the_turn_began_is_refused_inside_it() {
    let machine = AMachine::with_something_worth_protecting("read");
    let copied = machine.granted.join("copied.txt");
    let mut said = String::new();
    let went = a_turn_that_inherited(
        "read",
        &machine,
        |_, machine| File::open(&machine.key).expect("the key opens outside any turn"),
        |held, _| {
            let read = went(held.read_to_string(&mut said).map(drop));
            // And whatever was read goes into the folder this turn really was
            // granted, which is the write a verb legitimately makes.
            let out = went(fs::write(&copied, said.as_bytes()));
            (read, out)
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(REFUSED),
        "a turn read a file nobody granted through a descriptor that existed before the turn \
         began, so the boundary decides only about opens again and contents can leave a grant \
         — `file_permission` in crates/alo-bounding-kernel/src/kernel.rs is what stops it"
    );
    assert_eq!(
        said, "",
        "the read was refused and still produced bytes, so a byte moved before the refusal"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a turn was refused a write inside the folder it was granted, which is what \
         `archive_folder` does — the hook is refusing the legitimate half as well"
    );
    assert_eq!(
        holds(&copied),
        "",
        "the contents of a file nobody granted reached the folder somebody did"
    );
    assert_eq!(holds(&machine.key), A_KEY, "the key was disturbed");

    machine.taken_away();
}

/// **A file opened for writing before the turn began is refused a write inside
/// it, and is as it was afterwards.**
///
/// The other direction of the same descriptor: a verb with a bug in it holding
/// a writable handle to a file nobody granted could replace what is in it, and
/// the record would say nothing. The write is refused with `EACCES`, the
/// handle itself is still a handle — `fstat` asks no hook, and it is measured
/// so that the refusal cannot be a descriptor that had gone stale — and the
/// file still says what it said.
#[test]
fn a_file_opened_for_writing_before_the_turn_began_is_refused_inside_it() {
    let machine = AMachine::with_something_worth_protecting("write");
    let went = a_turn_that_inherited(
        "write",
        &machine,
        |_, machine| {
            OpenOptions::new()
                .write(true)
                .open(&machine.key)
                .expect("the key opens for writing outside any turn")
        },
        |held, _| {
            let wrote = went(
                held.write_all(A_REPLACEMENT.as_bytes())
                    .and_then(|()| held.flush()),
            );
            let still_there = went(held.metadata().map(drop));
            (wrote, still_there)
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(REFUSED),
        "a turn wrote into a file nobody granted through a descriptor that existed before the \
         turn began, so a verb with a bug in it can replace what is in a file outside its grant"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "the inherited descriptor was not usable at all, so the refusal above may be about a \
         broken descriptor rather than about the boundary"
    );
    assert_eq!(
        holds(&machine.key),
        A_KEY,
        "the write was refused and the file changed anyway"
    );

    machine.taken_away();
}

/// **A descriptor to a file inside the grant is untouched**: read through and
/// written through inside the turn exactly as it would be outside one.
///
/// The test that keeps the one above from being a boundary that refuses every
/// descriptor and looks like it works. The invoice is inside the granted
/// folder, so the walk from its directory entry meets the grant on the first
/// step up, and the hook's answer is the same as `file_open`'s would be for
/// the same file by name.
#[test]
fn a_descriptor_to_a_file_inside_the_grant_is_untouched() {
    let machine = AMachine::with_something_worth_protecting("granted");
    let mut said = String::new();
    let went = a_turn_that_inherited(
        "granted",
        &machine,
        |_, machine| {
            OpenOptions::new()
                .read(true)
                .append(true)
                .open(&machine.invoice)
                .expect("the invoice opens outside any turn")
        },
        |held, _| {
            let read = went(held.read_to_string(&mut said).map(drop));
            let wrote = went(
                held.write_all(A_NOTE.as_bytes())
                    .and_then(|()| held.flush()),
            );
            (read, wrote)
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "a turn was refused a read through a descriptor to a file inside its own grant, so the \
         hook on reads and writes is not walking to the grant the way the hook on opens does"
    );
    assert_eq!(
        said, AN_INVOICE,
        "the read was allowed and produced something else"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a turn was refused a write through a descriptor to a file inside its own grant"
    );
    assert_eq!(
        holds(&machine.invoice),
        format!("{AN_INVOICE}{A_NOTE}"),
        "the write inside the grant was allowed and did not land"
    );

    machine.taken_away();
}

/// **A file opened for appending before the turn began is refused a line
/// inside it — and this is the record's own shape.**
///
/// `alo_keeping::Writing` opens the machine's record with `append(true)` when
/// the daemon starts and holds it for the life of the process. A turn is a
/// thread of that process, so that descriptor is in the turn's table for as
/// long as the turn runs, and until 2026-09-12 a turn could add a line to the
/// record that no execution caused. It cannot now: the append is refused at
/// the write, the record is as the daemon left it, and opening the record by
/// name is refused as it always was. The record is written by the service,
/// outside the turn, which is where `alo-turn`'s `carrying.rs` always wrote it.
#[test]
fn a_record_opened_before_the_turn_began_is_refused_a_line_inside_it() {
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
        Outcome::Refused(REFUSED),
        "a turn added a line to the machine's record through the descriptor the daemon holds \
         it open with, so an execution that never happened can be written into the record from \
         inside a boundary"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(REFUSED),
        "a bound turn opened the machine's record by name, so it could read or replace the \
         record rather than only add to it"
    );
    assert_eq!(
        holds(&machine.record),
        A_LINE_THE_DAEMON_WROTE,
        "the append was refused and the record changed anyway"
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
        Outcome::Refused(REFUSED),
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
        Outcome::Refused(REFUSED),
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

/// **A directory opened before the turn began cannot be listed inside it
/// either — and one inside the grant still can.**
///
/// Reading a folder's entries is `getdents`, which is a read and passes
/// `file_permission` like one. The names of what is in a private folder are
/// not its contents, but they are something a person did not grant, and the
/// hook does not distinguish: the walk from the folder's own entry meets no
/// granted place and the listing is refused. The granted folder's descriptor,
/// walked the same way, meets the grant at once and lists — which is what
/// `list_folder` does on a folder handle, and is the half that must not break.
#[test]
fn a_directory_opened_before_the_turn_began_cannot_be_listed_inside_it() {
    let machine = AMachine::with_something_worth_protecting("listing");
    let went = a_turn_that_inherited(
        "listing",
        &machine,
        |_, machine| {
            (
                Some(File::open(&machine.private).expect("the folder opens outside any turn")),
                Some(File::open(&machine.granted).expect("the folder opens outside any turn")),
            )
        },
        |held, _| {
            let (private, granted) = held;
            (
                listing(
                    private
                        .take()
                        .expect("the private folder was opened beforehand"),
                ),
                listing(
                    granted
                        .take()
                        .expect("the granted folder was opened beforehand"),
                ),
            )
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(REFUSED),
        "a bound turn listed a folder nobody granted through a descriptor it inherited, so the \
         hook on reads is not asked about a directory's entries"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a bound turn was refused the listing of a folder inside its own grant, through a \
         descriptor to that folder — the hook is refusing the legitimate half as well"
    );

    machine.taken_away();
}

/// **A socket the daemon inherited is left to the hook that can decide about
/// it**: a write on a Unix socket inside a turn is allowed by the hook on
/// reads and writes, and so is the read on the other end.
///
/// The carve-out, measured. A socket is a file too, and a walk from a socket's
/// directory entry meets no granted place — so a hook that treated it like any
/// other file would refuse the daemon its answer to the person and a question
/// its provider. `file_permission` reads the kind from the inode's mode and
/// steps aside for a socket; `socket_sendmsg` then decides about the message by
/// where its bytes are going, and a Unix socket is not egress.
/// `what_a_bound_turn_can_still_reach.rs` holds that hook's half of the
/// arrangement; this holds the file hook's.
#[test]
fn a_socket_opened_before_the_turn_began_is_left_to_the_hook_that_decides_messages() {
    let machine = AMachine::with_something_worth_protecting("socket");
    let mut heard = Vec::new();
    let went = a_turn_that_inherited(
        "socket",
        &machine,
        |_, _| UnixStream::pair().expect("a pair of Unix sockets can be made outside any turn"),
        |held, _| {
            let (daemon, person) = held;
            let answered = went(daemon.write_all(AN_ANSWER).and_then(|()| daemon.flush()));
            let mut listened = vec![0; AN_ANSWER.len()];
            let heard_back = went(person.read_exact(&mut listened));
            heard = listened;
            (answered, heard_back)
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "a bound turn was refused a write on a Unix socket the daemon inherited, so the daemon \
         cannot answer the person from inside a turn — the hook on reads and writes is deciding \
         about a socket by its place in the filesystem rather than leaving it to socket_sendmsg"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a bound turn was refused a read on a Unix socket it inherited"
    );
    assert_eq!(
        heard, AN_ANSWER,
        "what was written on the socket is not what arrived"
    );

    machine.taken_away();
}

/// **A pipe opened before the turn began carries on inside it**, in both
/// directions.
///
/// The other half of the carve-out, and it was found rather than designed:
/// the day `file_permission` landed refusing everything but a socket, every
/// sibling of this file that binds a child process stopped hearing from it,
/// because a child bound into a turn says what it found over the pipe its
/// parent gave it. A pipe holds no contents of its own — what comes through
/// it a process outside the boundary put there, and what goes into it reaches
/// a process this service already talks to — so it is not a file at rest and
/// not a place a grant is over, which is the same reasoning as a Unix socket.
#[test]
fn a_pipe_opened_before_the_turn_began_carries_on_inside_it() {
    let machine = AMachine::with_something_worth_protecting("pipe");
    let mut heard = Vec::new();
    let went = a_turn_that_inherited(
        "pipe",
        &machine,
        |_, _| std::io::pipe().expect("a pipe can be made outside any turn"),
        |held, _| {
            let (reading, writing) = held;
            let said = went(writing.write_all(A_WORD).and_then(|()| writing.flush()));
            let mut listened = vec![0; A_WORD.len()];
            let heard_back = went(reading.read_exact(&mut listened));
            heard = listened;
            (said, heard_back)
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Allowed,
        "a bound turn was refused a write on a pipe it inherited, so a child bound into a turn \
         cannot tell its parent what it found — the hook on reads and writes is deciding about \
         a pipe by its place in the filesystem"
    );
    assert_eq!(
        went.after,
        Outcome::Allowed,
        "a bound turn was refused a read on a pipe it inherited"
    );
    assert_eq!(
        heard, A_WORD,
        "what was written into the pipe is not what came out"
    );

    machine.taken_away();
}

/// **The way out of a turn is refused to the turn itself — and the turn ends
/// anyway.**
///
/// `home/cgroup.threads` is what a thread writes into to leave a boundary, and
/// until 2026-09-12 the turn's own thread wrote it, through a descriptor
/// `Turns::under` had opened before the first turn ever ran, because opening
/// it from inside is refused. That arrangement was the fourth row of the
/// table this file reproduces: a verb with a bug in it could have ended its
/// own boundary early through the same descriptor.
///
/// This test opens that file before the turn, exactly as the service does, and
/// writes the same byte through it from inside. **It is refused with
/// `EACCES`**: the cgroup filesystem is not a place any grant is over, so the
/// walk meets nothing, and a turn cannot write itself out. The turn's own
/// `cgroup.threads` by name is refused as it always was. **And the turn
/// ended**, which is what reaching these assertions proves —
/// `a_turn_that_inherited` expects `doing` to return, and since the same day
/// `doing` returns only after a thread of the service that was never in the
/// turn has written this thread's number into `home/cgroup.threads` on its
/// behalf. `crates/alo-bounding/src/inside.rs` has the arrangement.
#[test]
fn the_way_out_of_a_turn_is_refused_to_the_turn_and_the_turn_ends_anyway() {
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
                OpenOptions::new()
                    .write(true)
                    .open(home.join(THE_THREADS))
                    .expect("the way out opens outside any turn, as the service opens it"),
                root.join("turn-way-out").join(THE_THREADS),
            )
        },
        |held, _| {
            let (back, own) = held;
            (
                // The byte the way out used to be: this thread, into `home`.
                went(back.write_all(b"0").and_then(|()| back.flush())),
                // And the turn's own, by name, for the same reason: a turn that
                // could open one of these could put itself anywhere in the
                // hierarchy.
                went(OpenOptions::new().write(true).open(&*own).map(drop)),
            )
        },
    );
    the_boundary_was_in_force(&went);

    assert_eq!(
        went.subject,
        Outcome::Refused(REFUSED),
        "a bound turn wrote itself out of its boundary through the descriptor the service holds \
         `home/cgroup.threads` open with, which would let a verb with a bug in it put itself \
         back among the daemon's own threads and finish its work outside every grant"
    );
    assert_eq!(
        went.after,
        Outcome::Refused(REFUSED),
        "a bound turn opened its own control group's `cgroup.threads`, so it could move itself \
         out of the boundary it is in without any descriptor being inherited at all"
    );

    machine.taken_away();
}
