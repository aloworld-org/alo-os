//! What alo is told, and what the indicator shows, for every way its work in an
//! application can go — and that **the person's own window is counted at
//! neither setting**.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead as _, BufReader};
use std::os::unix::net::UnixStream;
use std::rc::Rc;

use alo_acting::{ToCount, ToWorkIn, Where};
use alo_applications::Application;
use alo_egress::{HowItIsShown, Indicator};

use super::*;
use crate::counting_during_their_commands::{AskedToCount, CountsWhatIsSent, FindsWhereItRuns};

const FIREFOX: &str = "org.mozilla.firefox";
const TEXT_EDITOR: &str = "org.gnome.TextEditor";

/// The person's own Firefox window runs here.
const THEIRS: u64 = 10;
/// alo's copy of Firefox runs here, once started.
const ALOS: u64 = 20;
/// And what Flatpak moved alo's copy into.
const ALOS_SCOPE: u64 = 21;

/// What the fake kernel holds.
#[derive(Default)]
struct Kernel {
    /// Each counted group's count.
    counted: BTreeMap<u64, u64>,
    /// Every group ever begun, in order.
    begun: Vec<u64>,
    /// Groups the kernel refuses.
    refusing: Vec<u64>,
}

struct Fake(Rc<RefCell<Kernel>>);

impl CountsWhatIsSent for Fake {
    fn begun(&mut self, group: u64) -> bool {
        let mut kernel = self.0.borrow_mut();
        if kernel.refusing.contains(&group) {
            return false;
        }
        kernel.begun.push(group);
        kernel.counted.entry(group).or_insert(0);
        true
    }
    fn so_far(&self, group: u64) -> Option<u64> {
        self.0.borrow().counted.get(&group).copied()
    }
    fn ended(&mut self, group: u64) {
        self.0.borrow_mut().counted.remove(&group);
    }
}

/// Every unit of Firefox, the person's and alo's alike: what ADR 0148's finder
/// answers for a person's command.
struct EveryUnit(Rc<RefCell<Vec<u64>>>);

impl FindsWhereItRuns for EveryUnit {
    fn groups_of(&self, application: &str) -> Vec<u64> {
        if application == FIREFOX {
            self.0.borrow().clone()
        } else {
            Vec::new()
        }
    }
}

/// alo's copies: Firefox has one, the text editor cannot.
#[derive(Default)]
struct Copies {
    /// Whether alo's copy of Firefox is running.
    running: bool,
    /// Whether starting it fails.
    failing: bool,
    /// Whether its groups can be found.
    found: bool,
    /// How many times a start was asked for.
    starts: usize,
}

struct FakeCopies(Rc<RefCell<Copies>>);

impl StartsAlosCopy for FakeCopies {
    fn started(&mut self, application: &str) -> Result<(), NoCopy> {
        let mut copies = self.0.borrow_mut();
        if application != FIREFOX {
            return Err(NoCopy::OfItsOwn);
        }
        if copies.failing {
            return Err(NoCopy::CouldNotStart);
        }
        copies.starts += 1;
        copies.running = true;
        Ok(())
    }
    fn groups_of(&mut self, application: &str) -> Vec<u64> {
        let copies = self.0.borrow();
        if application == FIREFOX && copies.running && copies.found {
            vec![ALOS, ALOS_SCOPE]
        } else {
            Vec::new()
        }
    }
    fn every_copys_groups(&mut self) -> BTreeSet<u64> {
        self.groups_of(FIREFOX).into_iter().collect()
    }
}

struct Machine {
    kernel: Rc<RefCell<Kernel>>,
    copies: Rc<RefCell<Copies>>,
    counts: CountingForCommands,
    working: WorkingInApplications,
    indicator: Indicator,
    start: Instant,
}

impl Machine {
    /// A machine with the person's Firefox window open, and alo's copy's units
    /// visible to ADR 0148's finder too once it runs.
    fn new(can_count: bool) -> Self {
        let kernel = Rc::new(RefCell::new(Kernel::default()));
        let copies = Rc::new(RefCell::new(Copies {
            found: true,
            ..Copies::default()
        }));
        let counter: Option<Box<dyn CountsWhatIsSent>> = if can_count {
            Some(Box::new(Fake(Rc::clone(&kernel))))
        } else {
            None
        };
        let every = Rc::new(RefCell::new(vec![THEIRS, ALOS_SCOPE]));
        Self {
            counts: CountingForCommands::with(counter, Box::new(EveryUnit(every))),
            working: WorkingInApplications::with(Box::new(FakeCopies(Rc::clone(&copies)))),
            kernel,
            copies,
            indicator: Indicator::default(),
            start: Instant::now(),
        }
    }

    fn at(&self, seconds: u64) -> (Instant, SystemTime) {
        (
            self.start + Duration::from_secs(seconds),
            SystemTime::UNIX_EPOCH,
        )
    }

    /// alo asks to work in `application`, `place`; alo's end of the connection.
    fn asked(&mut self, application: &str, place: Where, seconds: u64) -> UnixStream {
        let (alo, door) = UnixStream::pair().unwrap();
        door.set_nonblocking(true).unwrap();
        let asked = ToWorkIn::at(Application::identified(application).unwrap(), place);
        let at = self.at(seconds);
        self.working.taken_in(
            AskedToWork::of(asked, door, Vec::new()),
            Some("Firefox"),
            &mut self.counts,
            &mut self.indicator,
            at,
        );
        alo
    }

    fn looked(&mut self, seconds: u64) {
        let at = self.at(seconds);
        self.working
            .looked_at(&mut self.counts, &mut self.indicator, at);
        self.counts.looked_at(&mut self.indicator, at.0, at.1);
    }

    fn sends(&self, group: u64, bytes: u64) {
        if let Some(count) = self.kernel.borrow_mut().counted.get_mut(&group) {
            *count += bytes;
        }
    }

    fn lines(&self) -> Vec<HowItIsShown> {
        self.indicator
            .showing()
            .iter()
            .filter_map(|shown| shown.showing().alo_at_work())
            .map(alo_egress::AloAtWork::how)
            .collect()
    }

    fn ever_begun(&self) -> Vec<u64> {
        self.kernel.borrow().begun.clone()
    }
}

fn next(alo: &UnixStream) -> String {
    let mut line = String::new();
    BufReader::new(alo).read_line(&mut line).unwrap();
    line.trim_end().to_owned()
}

fn done(alo: &mut UnixStream) {
    alo.write_all(b"done\n").unwrap();
}

/// **The default: a driven copy's departure is shown and counted** — and the
/// person's own window of the same application is not.
#[test]
fn alos_copy_is_counted_and_shown_from_its_first_byte_and_the_persons_window_is_not() {
    let mut machine = Machine::new(true);
    let mut alo = machine.asked(FIREFOX, Where::InAlosCopy, 0);
    assert_eq!(next(&alo), "counting");
    assert_eq!(machine.copies.borrow().starts, 1);
    machine.looked(0);
    assert!(machine.lines().is_empty(), "nothing has been sent yet");

    machine.sends(ALOS_SCOPE, 52_000);
    machine.sends(ALOS, 113);
    machine.sends(THEIRS, 9_999_999);
    machine.looked(1);
    assert_eq!(machine.lines(), [HowItIsShown::Counted]);

    done(&mut alo);
    machine.looked(5);
    assert_eq!(next(&alo), "sent 52113", "the person's bytes are not in it");
    assert!(machine.lines().is_empty());
    assert!(
        !machine.ever_begun().contains(&THEIRS),
        "the person's window was counted: {:?}",
        machine.ever_begun()
    );
}

/// **Turned up: the line is shown, and nothing at all is counted.**
#[test]
fn in_the_persons_window_the_line_is_up_the_whole_time_and_nothing_is_counted() {
    let mut machine = Machine::new(true);
    let mut alo = machine.asked(FIREFOX, Where::InTheirWindow, 0);
    assert_eq!(next(&alo), "showing");
    assert_eq!(machine.lines(), [HowItIsShown::TheirWindow]);
    assert_eq!(machine.copies.borrow().starts, 0, "no copy is started");
    machine.looked(1);
    machine.sends(THEIRS, 500);
    machine.looked(2);
    assert_eq!(machine.lines(), [HowItIsShown::TheirWindow]);
    done(&mut alo);
    machine.looked(3);
    assert_eq!(next(&alo), "uncounted");
    assert!(machine.lines().is_empty());
    assert!(
        machine.ever_begun().is_empty(),
        "the person's window was counted: {:?}",
        machine.ever_begun()
    );
}

/// **Refusal A**: an application that cannot open a copy of its own is
/// refused at the default, and nothing is started, shown or counted.
#[test]
fn an_application_with_no_copy_of_its_own_is_refused_at_the_default() {
    let mut machine = Machine::new(true);
    let alo = machine.asked(TEXT_EDITOR, Where::InAlosCopy, 0);
    assert_eq!(next(&alo), "no-copy-of-its-own");
    assert!(machine.lines().is_empty());
    assert_eq!(machine.working.how_many(), 0);
    assert!(machine.ever_begun().is_empty());
}

#[test]
fn a_copy_that_will_not_start_is_said_so() {
    let mut machine = Machine::new(true);
    machine.copies.borrow_mut().failing = true;
    let alo = machine.asked(FIREFOX, Where::InAlosCopy, 0);
    assert_eq!(next(&alo), "could-not-start");
    assert_eq!(machine.working.how_many(), 0);
}

/// **Counting unavailable: the indicator says so plainly, never silence.**
#[test]
fn on_a_machine_with_no_counter_the_line_says_it_cannot_count_for_the_whole_time() {
    let mut machine = Machine::new(false);
    let mut alo = machine.asked(FIREFOX, Where::InAlosCopy, 0);
    assert_eq!(next(&alo), "cannot-count");
    assert_eq!(machine.lines(), [HowItIsShown::Uncounted]);
    machine.looked(1);
    assert_eq!(machine.lines(), [HowItIsShown::Uncounted]);
    done(&mut alo);
    machine.looked(3);
    assert_eq!(next(&alo), "uncounted");
    assert!(machine.lines().is_empty());
}

/// A copy whose units are never found is not a copy that sent nothing: after
/// [`FOUND_WITHIN`] the line says it cannot be counted.
#[test]
fn a_copy_never_found_is_shown_as_uncounted_rather_than_as_silent() {
    let mut machine = Machine::new(true);
    machine.copies.borrow_mut().found = false;
    let mut alo = machine.asked(FIREFOX, Where::InAlosCopy, 0);
    assert_eq!(next(&alo), "counting");
    machine.looked(1);
    assert!(machine.lines().is_empty());
    machine.looked(2);
    assert_eq!(machine.lines(), [HowItIsShown::Uncounted]);
    done(&mut alo);
    machine.looked(4);
    assert_eq!(next(&alo), "uncounted");
}

/// One part of the copy the kernel will not count makes the whole uncounted,
/// said at once.
#[test]
fn a_part_of_the_copy_that_cannot_be_counted_is_said_at_once() {
    let mut machine = Machine::new(true);
    machine.kernel.borrow_mut().refusing.push(ALOS_SCOPE);
    let alo = machine.asked(FIREFOX, Where::InAlosCopy, 0);
    assert_eq!(next(&alo), "cannot-count");
    assert_eq!(machine.lines(), [HowItIsShown::Uncounted]);
}

/// Hung up: the line comes off once it has been seen, and nothing is held.
#[test]
fn a_turn_that_hangs_up_ends_its_line_once_it_has_been_seen() {
    let mut machine = Machine::new(true);
    let alo = machine.asked(FIREFOX, Where::InAlosCopy, 0);
    assert_eq!(next(&alo), "counting");
    machine.sends(ALOS, 10);
    machine.looked(0);
    assert_eq!(machine.lines(), [HowItIsShown::Counted]);
    drop(alo);
    machine.looked(1);
    assert_eq!(machine.lines(), [HowItIsShown::Counted], "seen for two seconds");
    machine.looked(3);
    assert!(machine.lines().is_empty());
    assert!(machine.kernel.borrow().counted.is_empty(), "a count was left running");
}

#[test]
fn work_held_too_long_is_ended_by_the_desktop() {
    let mut machine = Machine::new(true);
    let alo = machine.asked(FIREFOX, Where::InAlosCopy, 0);
    assert_eq!(next(&alo), "counting");
    machine.sends(ALOS, 7);
    machine.looked(THE_LONGEST.as_secs());
    assert_eq!(next(&alo), "stopped 7");
    assert_eq!(machine.working.how_many(), 0);
}

/// **alo's copy is not the person's application**: a person's own command
/// counting Firefox never counts what alo's copy sends.
#[test]
fn a_persons_command_never_counts_alos_copy() {
    let mut machine = Machine::new(true);
    let _alo = machine.asked(FIREFOX, Where::InAlosCopy, 0);
    machine.looked(0);
    let (command, door) = UnixStream::pair().unwrap();
    door.set_nonblocking(true).unwrap();
    let at = machine.at(0);
    machine.counts.taken_in(
        AskedToCount::of(
            ToCount::what(Application::identified(FIREFOX).unwrap()),
            door,
            Vec::new(),
        ),
        Some("Firefox"),
        at.0,
    );
    assert_eq!(next(&command), "counting");
    machine.sends(ALOS_SCOPE, 400);
    machine.sends(THEIRS, 3);
    machine.looked(1);
    let mut command = command;
    command.write_all(b"done\n").unwrap();
    machine.looked(2);
    assert_eq!(next(&command), "sent 3");
}

/// A desktop with nowhere to show a line refuses, and alo may not work.
#[test]
fn a_desktop_with_no_indicator_refuses() {
    let (alo, door) = UnixStream::pair().unwrap();
    let asked = ToWorkIn::at(Application::identified(FIREFOX).unwrap(), Where::InAlosCopy);
    AskedToWork::of(asked, door, Vec::new()).refused();
    assert_eq!(next(&alo), "cannot-show");
}
