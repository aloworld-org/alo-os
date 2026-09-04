//! **The LSM decides and forgets**, measured on a real kernel rather than
//! promised in a comment.
//!
//! [ADR 0015](../../../docs/decisions/0015-the-kernel-learns-what-a-turn-is.md)
//! names one dangerous property and this file is the answer to it. A BPF program
//! on the security hooks is called for **every open on the machine**, by every
//! program, for as long as it is attached — so the same mechanism that stops an
//! agent reaching a private key could, with four more lines, be a record of
//! somebody's whole day. Nothing in the kernel would object. The only thing
//! between the two is that the program has nowhere to put what it saw.
//!
//! *Nowhere* is a claim, and a claim in a crate header erodes in a year, one
//! reasonable-sounding feature at a time. So it is counted here instead.
//!
//! # The three places a trace could be left, and all three are checked
//!
//! - **A map.** A ring buffer, a counter, a table of who opened what — any of
//!   them would be a place a program can write. There are two maps and they are
//!   the two the daemon fills; a third is the finding.
//! - **An entry nobody put there.** The map of turns is written by the daemon
//!   and read by the program. An entry appearing while no turn is running is the
//!   program keeping a note.
//! - **A line in the kernel's trace buffer.** `bpf_printk` is the one way a BPF
//!   program says something without a map at all, and what it says goes into the
//!   buffer `/sys/kernel/tracing/trace` reports the size of.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! The same checks the other two files here name, and it fails loudly on a
//! machine without them for the same reason: a test that quietly skipped itself
//! would report green on every machine where the boundary does nothing at all.
//! Where the boundary comes from is `on_this_kernel/mod.rs`, shared with them.
//!
//! It needs one thing they do not — a kernel that is **recording**. A trace
//! buffer switched off would take every line this file exists to catch and drop
//! it silently, so [`what_this_kernel_has_traced`] refuses on a machine where
//! `tracing_on` is not `1` rather than answering zero.
//!
//! # What is run, and why none of it is an agent
//!
//! Queue item 27 says *ordinary programs — not agent turns*, and there are three
//! kinds of those here: this process, other threads of it, and a second process
//! that is this binary run again. None of them is in any turn's control group,
//! which is what every other program on a real machine has in common with them.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use alo_bounding::{Bounds, Cgroup, Turns, place_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// What the kernel reports the size of its trace buffer in.
///
/// Read rather than `trace_pipe`, because the header of this file carries the
/// count of everything ever written — so what is compared is two numbers rather
/// than two copies of a text file that a reader would have to diff.
const THE_TRACE: &str = "/sys/kernel/tracing/trace";

/// Whether that buffer is recording at all.
const IS_TRACING: &str = "/sys/kernel/tracing/tracing_on";

/// The door anybody may write a line into that buffer through.
///
/// Used by one test and one only: the one that proves the count this file reads
/// moves when something is written, so the tests that assert it did **not** move
/// are measuring a live number rather than a stuck one.
const THE_MARKER: &str = "/sys/kernel/tracing/trace_marker";

/// Where the second program is told to spend its day.
const THE_FOLDER: &str = "ALO_FORGETTING_TEST_FOLDER";

/// How many files an ordinary program's day is made of here.
const FILES: usize = 12;

/// How many times over each program opens them.
///
/// Enough that a program keeping a tally would have a visibly different tally,
/// and few enough that the whole file is a second or two.
const ROUNDS: usize = 20;

/// How many other threads of this program go about their own business.
const THREADS: usize = 3;

/// Everything on this machine that could be holding what the program saw.
///
/// Read out of the kernel each time rather than remembered, so what is compared
/// is what the machine has rather than what this file believes it asked for.
#[derive(Debug, PartialEq, Eq)]
struct Held {
    /// Every map the program has, by name, in order.
    maps: Vec<String>,

    /// Every turn the kernel is holding a bound for, in order.
    turns: Vec<u64>,

    /// The offsets the daemon gave it, and the slots it never filled.
    fields: Vec<u32>,

    /// How many lines have been written to this kernel's trace buffer, ever.
    traced: u64,
}

impl Held {
    /// What the kernel is holding at this moment.
    ///
    /// Through both halves of ADR 0018, because that is where the three
    /// measurements live: the maps and the offsets are read by the loader, which
    /// is the only thing that can open them, and the turns are read through the
    /// map the daemon writes. What the daemon can reach is deliberately the
    /// smallest of the three.
    fn of(kernel: &AsAMachineHasIt) -> Self {
        let mut maps: Vec<String> = kernel
            .imposed
            .every_map_the_kernel_holds()
            .into_iter()
            .map(str::to_owned)
            .collect();
        maps.sort_unstable();
        let mut turns = kernel
            .boundary
            .every_turn_the_kernel_is_holding()
            .expect("the map of turns can be read");
        turns.sort_unstable();
        Self {
            maps,
            turns,
            fields: kernel
                .imposed
                .every_field_the_kernel_was_given()
                .expect("the map of fields can be read"),
            traced: what_this_kernel_has_traced(),
        }
    }
}

/// How many lines this kernel's trace buffer has been written, ever.
///
/// `bpf_printk` is the one way a BPF program says something without a map, and
/// what it says lands here. The header of `trace` carries the number, so this is
/// a count rather than a comparison of two piles of text.
///
/// **It refuses on a machine that is not recording.** A buffer switched off
/// would take every line this file exists to catch and drop it, and the test
/// would then pass in exactly the same colour as a machine that wrote nothing —
/// which is the failure this whole file is written to prevent somewhere else.
fn what_this_kernel_has_traced() -> u64 {
    let recording = fs::read_to_string(IS_TRACING).unwrap_or_else(|why| {
        panic!(
            "{IS_TRACING} cannot be read, so whether this kernel would record a line a BPF \
             program wrote is unknown, and nothing below would mean anything: {why}"
        )
    });
    assert_eq!(
        recording.trim(),
        "1",
        "this kernel is not recording, so a line written by a BPF program would be dropped and \
         this test would pass without measuring anything. `echo 1 > {IS_TRACING}` is what makes \
         the measurement possible."
    );

    let said = fs::read_to_string(THE_TRACE).unwrap_or_else(|why| {
        panic!(
            "{THE_TRACE} cannot be read, so the one thing a BPF program can write without a map \
             cannot be counted here: {why}"
        )
    });
    let counted = said
        .lines()
        .find_map(|line| line.split_once("entries-written:"))
        .and_then(|(_, after)| after.split_once('/'))
        .and_then(|(_, written)| written.split_whitespace().next())
        .and_then(|written| written.parse().ok());
    let Some(counted) = counted else {
        panic!(
            "{THE_TRACE} does not say how many lines this kernel's trace buffer has been \
             written, so the one thing a BPF program can write without a map cannot be measured. \
             What is read is the `entries-in-buffer/entries-written` header, and what was there \
             instead begins:\n{}",
            said.lines().take(3).collect::<Vec<_>>().join("\n")
        )
    };
    counted
}

/// The whole of *decides and forgets*, asserted against what was there before.
///
/// `after_what` is what the machine was made to do in between, and it goes at
/// the front of every message — because queue item 27 asks for a failure that
/// says what it caught rather than that two values differ.
fn nothing_was_written_down(before: &Held, after: &Held, after_what: &str) {
    assert_eq!(
        after.traced, before.traced,
        "{after_what}, and this kernel's trace buffer went from {} lines to {}: a syscall \
         outside an agent turn left a trace, and nothing outside a turn may. `bpf_printk` is the \
         one way a BPF program writes a line without a map, and \
         `crates/alo-bounding-kernel/src/kernel.rs` is supposed to contain none. If something \
         else on this machine is tracing, `/sys/kernel/tracing/current_tracer` and \
         `/sys/kernel/tracing/set_event` are the first two things to look at.",
        before.traced, after.traced
    );
    assert_eq!(
        after.turns,
        before.turns,
        "{after_what}, and the kernel is now holding {} entries for turns rather than the {} it \
         was left with: a syscall outside an agent turn left a trace, and nothing outside a turn \
         may. The daemon writes that map when a turn begins and empties it when the turn ends, \
         so an entry nobody put there is the program keeping a note of what it saw.",
        after.turns.len(),
        before.turns.len()
    );
    assert_eq!(
        after.fields, before.fields,
        "{after_what}, and the offsets this kernel was given have changed: a syscall outside an \
         agent turn left a trace, and nothing outside a turn may. A spare slot in an array the \
         program can already reach is exactly where a counter would sit. Before: {:?}. After: \
         {:?}.",
        before.fields, after.fields
    );
    assert_eq!(
        after.maps, before.maps,
        "{after_what}, and the program's maps are now {:?} rather than {:?}: a syscall outside an \
         agent turn left a trace, and nothing outside a turn may. A ring buffer, a counter or a \
         table of who opened what is where a record of somebody's day would live, and this \
         program is supposed to have nowhere at all.",
        after.maps, before.maps
    );
}

/// A folder with ordinary things in it: no grant, no turn, and nothing an agent
/// has ever been near.
fn an_ordinary_folder(what: &str) -> PathBuf {
    let at = PathBuf::from("/tmp").join(format!("alo-forgetting-{}-{what}", std::process::id()));
    fs::create_dir_all(&at).expect("a temporary directory can be made");
    for which in 0..FILES {
        fs::write(at.join(format!("ordinary-{which}")), b"an ordinary file")
            .expect("a file can be written");
    }
    at
}

/// A day's work for one program: every file in the folder, [`ROUNDS`] times
/// over, and the count of what it opened.
fn an_ordinary_days_opens(folder: &Path) -> usize {
    (0..ROUNDS).map(|_| opening_everything_in(folder)).sum()
}

/// One pass over the folder, opening each thing in it.
///
/// Reading the directory is an open of its own and is deliberately not counted:
/// what the count is for is the failure message, and a number that is honestly
/// low is better there than one that is arguably high.
fn opening_everything_in(folder: &Path) -> usize {
    let mut opened = 0;
    for entry in fs::read_dir(folder).expect("the ordinary folder is there") {
        let entry = entry.expect("a directory entry can be read").path();
        drop(fs::File::open(&entry).expect("an ordinary program can open its own files"));
        opened += 1;
    }
    opened
}

/// [`THREADS`] more threads of this same program, because nothing on a machine
/// is one thread and a boundary that noticed threads differently would be worth
/// knowing about.
fn and_on_other_threads(folder: &Path) -> usize {
    std::thread::scope(|threads| {
        let running: Vec<_> = (0..THREADS)
            .map(|_| threads.spawn(|| opening_everything_in(folder)))
            .collect();
        running
            .into_iter()
            .map(|thread| thread.join().expect("the thread finishes"))
            .sum()
    })
}

/// A second **program**: this binary again, at the ignored test below.
///
/// A thread of this process is still this process, and item 27 says programs.
/// This one is in no turn's control group, has never been near a grant, and is
/// what almost everything on a real machine is.
fn and_an_ordinary_program(folder: &Path) -> usize {
    let ran = Command::new(env::current_exe().expect("a test binary knows where it is"))
        .args([
            "--exact",
            "--ignored",
            "--nocapture",
            "the_day_an_ordinary_program_has",
        ])
        .env(THE_FOLDER, folder)
        .stdout(Stdio::null())
        .status()
        .expect("this binary can be run again");
    assert!(
        ran.success(),
        "the second program did not have the day it was asked for, so what it opened is unknown"
    );
    FILES * ROUNDS
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
/// Nothing else: this is what runs inside a boundary, so it opens exactly one
/// thing and holds no opinion about it. `a_turn_is_this_thread.rs` says at
/// length why nothing is asserted in there.
fn opening(what: &Path) -> Outcome {
    match fs::File::open(what) {
        Ok(_) => Outcome::Opened,
        Err(why) => Outcome::Refused(why.raw_os_error().unwrap_or(0)),
    }
}

/// **The program has nowhere to put what it sees.**
///
/// Two maps, and they are the two the daemon fills before it attaches: the turns
/// it is told about, and where this kernel keeps its fields. Both are written by
/// userspace and only read by the program.
///
/// A third name appearing here is the finding, whatever it is called and
/// whatever kind it is — a program that had somewhere to write would have to
/// have somewhere, and this is the list of everywhere it has.
#[test]
fn the_program_has_nowhere_to_write_what_it_sees() {
    let _order = on_this_kernel::one_at_a_time();
    let kernel = AsAMachineHasIt::on_this_kernel("nowhere-to-write");
    let held = Held::of(&kernel);
    assert_eq!(
        held.maps,
        ["BOUNDS", "FIELDS"],
        "the boundary has somewhere to write that it did not have before: {:?}. A BPF LSM is \
         called for every open on this machine, and the only thing between that and a record of \
         somebody's day is that the program has nowhere to put what it saw.",
        held.maps
    );
}

/// **The item, in one test.** Ordinary programs spend a day opening files under
/// the loaded LSM, and afterwards nothing on the machine is different.
///
/// Three kinds of program, because the interesting failure is a boundary that
/// treats one of them specially: this process, [`THREADS`] more threads of it,
/// and a second process that is this binary run again. None of them is in a
/// turn's control group, so for every one of their opens the program does one
/// hash lookup, misses, and returns — which is the path almost every open on a
/// real machine takes.
#[test]
fn ordinary_programs_run_under_the_boundary_and_nothing_is_written_down() {
    let _order = on_this_kernel::one_at_a_time();
    let kernel = AsAMachineHasIt::on_this_kernel("an-ordinary-day");
    let folder = an_ordinary_folder("day");

    let before = Held::of(&kernel);
    assert!(
        before.turns.is_empty(),
        "the kernel is holding {} turns, so what follows is not happening outside one and this \
         test is measuring something else",
        before.turns.len()
    );

    let opened = an_ordinary_days_opens(&folder)
        + and_on_other_threads(&folder)
        + and_an_ordinary_program(&folder);
    assert!(
        opened >= FILES * ROUNDS * 2,
        "only {opened} files were opened, so the boundary was barely asked anything and a \
         program that wrote one line in a hundred would not have been caught"
    );

    let after = Held::of(&kernel);
    nothing_was_written_down(
        &before,
        &after,
        &format!("{opened} files were opened by programs that are not agent turns"),
    );
}

/// **The dangerous moment is not the ordinary one.**
///
/// For a program that is not a turn the LSM does one lookup and returns, so
/// *nothing was written down* is nearly free. The moment worth measuring is the
/// one where the program does its whole job: a turn is bound, it walks a
/// directory entry up to the top of a filesystem, it compares what it found
/// against a grant, and it **refuses** an open.
///
/// That is the exact instant a program would have something worth recording, and
/// the assertion is that it recorded nothing — no entry, no counter, no line.
/// The refusal is asserted first, because a kernel that allowed the open would
/// make everything after it a measurement of nothing.
///
/// What is written down about that refusal is alo OS's own record, made in
/// userspace by `alo-turn` out of the value the capability model produced. The
/// kernel's half decides and forgets; ours decides and remembers, and they are
/// different halves on purpose.
#[test]
fn a_turn_the_kernel_refused_is_not_written_down_either() {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel("a-refused-turn");
    let folder = an_ordinary_folder("turn");
    let granted = folder.join("granted");
    fs::create_dir_all(&granted).expect("a temporary directory can be made");
    let invoice = granted.join("march.pdf");
    fs::write(&invoice, b"an invoice").expect("a file can be written");
    let outside = folder.join("ordinary-0");

    let before = Held::of(&kernel);

    let ours = Cgroup::made(&format!("alo-forgetting-{}", std::process::id()))
        .expect("a control group can be made");
    let turns = Turns::under(ours.at()).expect("a service can make a subtree of its own");
    let found = turns
        .doing(
            &mut kernel.boundary,
            "turn-forgetting",
            Bounds::of_one(place_of(&granted).expect("the granted folder is there")),
            || (opening(&invoice), opening(&outside)),
        )
        .expect("the turn can be bounded");
    turns
        .given_back()
        .expect("a service can be put back where it was");
    ours.removed()
        .expect("an empty control group can be taken away");

    assert_eq!(
        found.0,
        Outcome::Opened,
        "the granted file was refused to the turn that was granted it"
    );
    assert_eq!(
        found.1,
        Outcome::Refused(13),
        "the kernel allowed an open outside the bound, so it never did the work this test is \
         about and what follows would be measuring nothing"
    );

    let after = Held::of(&kernel);
    nothing_was_written_down(
        &before,
        &after,
        "the kernel walked a filesystem and refused an open inside an agent turn",
    );
}

/// **The three measurements above would notice something**, which is the half a
/// green run cannot tell you.
///
/// A test that reads a counter passes on the day the counter stops moving, and
/// it passes in exactly the same colour as one that read a counter and found it
/// still. So each of the three is made to move here, deliberately and by this
/// test rather than by the program: a line goes into the kernel's trace buffer,
/// an entry goes into the map of turns and comes out again, and the offsets are
/// read back and found to be a real kernel's rather than a map of zeroes.
///
/// The fourth — the list of maps — has no twin here and cannot have one: giving
/// the program a third map is a change to `crates/alo-bounding-kernel`, which is
/// compiled into this binary, so the only way to fake it would be to ship the
/// thing being looked for.
#[test]
fn the_checks_would_notice_something_being_written() {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel("would-notice");
    let boundary = &mut kernel.boundary;

    let traced = what_this_kernel_has_traced();
    fs::write(THE_MARKER, b"alo: proving this kernel counts a line\n")
        .expect("a line can be written into this kernel's trace buffer");
    assert!(
        what_this_kernel_has_traced() > traced,
        "a line was written into this kernel's trace buffer and the count did not move, so the \
         tests above would not notice a `bpf_printk` either and are asserting nothing"
    );

    // A cgroup identifier no cgroup has, so no open on this machine can present
    // it and nothing is bounded by putting it there. It is taken out again below.
    let nobody = u64::MAX;
    boundary
        .bound(
            nobody,
            Bounds::of_one(place_of(Path::new("/tmp")).expect("/tmp is there")),
        )
        .expect("the kernel takes the entry");
    assert!(
        boundary
            .every_turn_the_kernel_is_holding()
            .expect("the map of turns can be read")
            .contains(&nobody),
        "an entry was put into the map of turns and reading the map back did not find it, so an \
         entry the program wrote would not be found either"
    );
    boundary.released(nobody).expect("the kernel takes it back");
    assert!(
        !boundary
            .every_turn_the_kernel_is_holding()
            .expect("the map of turns can be read")
            .contains(&nobody),
        "the entry was taken out of the map of turns and reading the map back still finds it"
    );

    let fields = kernel
        .imposed
        .every_field_the_kernel_was_given()
        .expect("the map of fields can be read");
    assert!(
        fields.iter().filter(|offset| **offset != 0).count() >= 6,
        "the offsets read back out of this kernel are {fields:?}, which is not what a kernel's \
         structures look like — so a counter appearing in a spare slot would be compared against \
         a reading that was never right in the first place"
    );
}

/// The day an ordinary program has, which is a second process because a thread
/// of this one is still this one.
///
/// Never run by an ordinary pass — it is ignored, and the test above asks for it
/// by name. Run without the environment naming a folder it does nothing at all,
/// so `cargo test -- --include-ignored` is harmless.
#[test]
#[ignore = "this is the second program the test above runs, and it runs it by name"]
fn the_day_an_ordinary_program_has() {
    let Ok(folder) = env::var(THE_FOLDER) else {
        return;
    };
    let opened = an_ordinary_days_opens(Path::new(&folder));
    assert_eq!(
        opened,
        FILES * ROUNDS,
        "the second program opened {opened} files rather than the {} it was asked for",
        FILES * ROUNDS
    );
}
