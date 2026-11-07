//! What a person's command is told, and what the indicator shows, for every way
//! a count can go: begun, refused, sent, silent, hung up, held too long, and
//! shared with a second command.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime};

use alo_acting::ToCount;
use alo_applications::Application;
use alo_egress::Indicator;

use super::*;

/// What the fake kernel holds, shared with the test so it can send bytes.
#[derive(Default)]
struct Kernel {
    /// Each counted group's count.
    counted: BTreeMap<u64, u64>,
    /// Groups the kernel will refuse to count.
    refusing: Vec<u64>,
    /// Every group ever ended, in order.
    ended: Vec<u64>,
}

/// A counter over the fake kernel.
struct Fake(Rc<RefCell<Kernel>>);

impl CountsWhatIsSent for Fake {
    fn begun(&mut self, group: u64) -> bool {
        let mut kernel = self.0.borrow_mut();
        if kernel.refusing.contains(&group) {
            return false;
        }
        kernel.counted.entry(group).or_insert(0);
        true
    }
    fn so_far(&self, group: u64) -> Option<u64> {
        self.0.borrow().counted.get(&group).copied()
    }
    fn ended(&mut self, group: u64) {
        let mut kernel = self.0.borrow_mut();
        kernel.counted.remove(&group);
        kernel.ended.push(group);
    }
}

/// Where applications run, shared with the test so it can start one.
struct Where(Rc<RefCell<BTreeMap<String, Vec<u64>>>>);

impl FindsWhereItRuns for Where {
    fn groups_of(&self, application: &str) -> Vec<u64> {
        self.0.borrow().get(application).cloned().unwrap_or_default()
    }
}

/// A machine to count on: the kernel, where each application runs, and the
/// counting.
struct Machine {
    kernel: Rc<RefCell<Kernel>>,
    running: Rc<RefCell<BTreeMap<String, Vec<u64>>>>,
    counting: CountingForCommands,
    indicator: Indicator,
    start: Instant,
}

const MAIL: &str = "org.mozilla.Thunderbird";

impl Machine {
    fn with_mail_in(groups: &[u64]) -> Self {
        let kernel = Rc::new(RefCell::new(Kernel::default()));
        let running = Rc::new(RefCell::new(BTreeMap::from([(
            MAIL.to_owned(),
            groups.to_vec(),
        )])));
        let counting = CountingForCommands::with(
            Some(Box::new(Fake(Rc::clone(&kernel)))),
            Box::new(Where(Rc::clone(&running))),
        );
        Self {
            kernel,
            running,
            counting,
            indicator: Indicator::default(),
            start: Instant::now(),
        }
    }

    fn at(&self, seconds: u64) -> Instant {
        self.start + Duration::from_secs(seconds)
    }

    /// A command asks to count mail, at `seconds`; its end of the connection.
    fn asked(&mut self, seconds: u64, called: Option<&str>) -> BufReader<UnixStream> {
        let (command, door) = UnixStream::pair().unwrap();
        door.set_nonblocking(true).unwrap();
        let asked = ToCount::what(Application::identified(MAIL).unwrap());
        self.counting
            .taken_in(AskedToCount::of(asked, door, Vec::new()), called, self.at(seconds));
        BufReader::new(command)
    }

    fn looked(&mut self, seconds: u64) {
        let at = self.at(seconds);
        self.counting
            .looked_at(&mut self.indicator, at, SystemTime::UNIX_EPOCH);
    }

    fn sends(&self, group: u64, bytes: u64) {
        if let Some(count) = self.kernel.borrow_mut().counted.get_mut(&group) {
            *count += bytes;
        }
    }

    fn lines(&self) -> Vec<String> {
        self.indicator
            .showing()
            .iter()
            .filter_map(|shown| shown.showing().during_their_command())
            .map(|during| during.called().to_owned())
            .collect()
    }
}

fn next(reading: &mut BufReader<UnixStream>) -> String {
    let mut line = String::new();
    reading.read_line(&mut line).unwrap();
    line.trim_end().to_owned()
}

fn done(reading: &mut BufReader<UnixStream>) {
    reading.get_mut().write_all(b"done\n").unwrap();
}

/// **The whole of it**: begun, quiet until the first byte, lit from then,
/// answered with exactly what was sent, and every count ended.
#[test]
fn a_command_is_counted_shown_and_told_what_was_sent() {
    let mut machine = Machine::with_mail_in(&[7, 8]);
    let mut command = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "counting");

    machine.looked(0);
    assert!(machine.lines().is_empty(), "lit before anything was sent");

    machine.sends(7, 1000);
    machine.sends(8, 24);
    machine.looked(1);
    assert_eq!(machine.lines(), ["Thunderbird"]);

    done(&mut command);
    machine.looked(5);
    assert_eq!(next(&mut command), "sent 1024");
    assert!(machine.lines().is_empty(), "the line outlived its command");
    assert_eq!(machine.counting.how_many_are_counting(), 0);
    assert!(machine.kernel.borrow().counted.is_empty(), "a count outlived its command");
}

/// **A command that sends nothing never lights the indicator**, and is told
/// so: nothing left, and the light must not say something did.
#[test]
fn a_command_whose_application_sends_nothing_never_lights_the_indicator() {
    let mut machine = Machine::with_mail_in(&[7]);
    let mut command = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "counting");
    machine.looked(1);
    done(&mut command);
    machine.looked(2);
    assert_eq!(next(&mut command), "sent 0");
    assert!(machine.indicator.is_quiet());
}

/// **A short burst stays on the indicator long enough to be seen.**
#[test]
fn a_line_stays_until_it_has_been_seen() {
    let mut machine = Machine::with_mail_in(&[7]);
    let mut command = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "counting");
    machine.sends(7, 10);
    machine.looked(10);
    done(&mut command);
    machine.looked(11);
    assert_eq!(next(&mut command), "sent 10");
    assert_eq!(machine.lines(), ["Thunderbird"], "it came off before it was seen");
    machine.looked(12);
    assert!(machine.lines().is_empty(), "it stayed past the time it was owed");
}

/// **A command that hangs up ends its count**, and nothing stays counting
/// because a process crashed.
#[test]
fn a_command_that_hangs_up_ends_its_count() {
    let mut machine = Machine::with_mail_in(&[7]);
    let mut command = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "counting");
    drop(command);
    machine.looked(1);
    assert_eq!(machine.counting.how_many_are_counting(), 0);
    assert_eq!(machine.kernel.borrow().ended, [7]);
}

/// **A count held too long is ended by the desktop**, and the command is told
/// it was stopped rather than finished.
#[test]
fn a_count_held_too_long_is_stopped() {
    let mut machine = Machine::with_mail_in(&[7]);
    let mut command = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "counting");
    machine.sends(7, 5);
    machine.looked(THE_LONGEST.as_secs() - 1);
    assert_eq!(machine.counting.how_many_are_counting(), 1);
    machine.looked(THE_LONGEST.as_secs());
    assert_eq!(next(&mut command), "stopped 5");
    assert_eq!(machine.counting.how_many_are_counting(), 0);
    assert!(machine.kernel.borrow().counted.is_empty());
}

/// **An application that starts during the command is counted from when its
/// unit appears.**
#[test]
fn an_application_started_during_the_command_is_counted_once_it_appears() {
    let mut machine = Machine::with_mail_in(&[]);
    let mut command = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "counting");
    machine.looked(1);
    machine.running.borrow_mut().insert(MAIL.to_owned(), vec![9]);
    machine.looked(2);
    machine.sends(9, 300);
    done(&mut command);
    machine.looked(3);
    assert_eq!(next(&mut command), "sent 300");
}

/// **Two commands counting one application each get their own share**, and
/// the count ends only with the second.
#[test]
fn two_commands_on_one_application_each_get_their_own_share() {
    let mut machine = Machine::with_mail_in(&[7]);
    let mut first = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut first), "counting");
    machine.sends(7, 100);
    let mut second = machine.asked(1, Some("Thunderbird"));
    assert_eq!(next(&mut second), "counting");
    machine.sends(7, 20);

    done(&mut first);
    machine.looked(2);
    assert_eq!(next(&mut first), "sent 120");
    assert!(
        machine.kernel.borrow().counted.contains_key(&7),
        "the first command's end took the second's count away"
    );
    done(&mut second);
    machine.looked(3);
    assert_eq!(next(&mut second), "sent 20");
    assert!(machine.kernel.borrow().counted.is_empty());
}

/// **What cannot be counted is said, never answered as a count**: an
/// application not installed, a machine with no counter, too many commands at
/// once, and an application none of whose parts the kernel will count.
#[test]
fn what_cannot_be_counted_is_refused_by_name() {
    let mut machine = Machine::with_mail_in(&[7]);
    let mut not_installed = machine.asked(0, None);
    assert_eq!(next(&mut not_installed), "not-installed");

    let mut nowhere = CountingForCommands::with(
        None,
        Box::new(Where(Rc::new(RefCell::new(BTreeMap::new())))),
    );
    let (command, door) = UnixStream::pair().unwrap();
    nowhere.taken_in(
        AskedToCount::of(ToCount::what(Application::identified(MAIL).unwrap()), door, Vec::new()),
        Some("Thunderbird"),
        Instant::now(),
    );
    assert_eq!(next(&mut BufReader::new(command)), "cannot-count");
    assert_eq!(nowhere.how_many_are_counting(), 0);

    let mut held = Vec::new();
    for _ in 0..THE_MOST_AT_ONCE {
        let mut command = machine.asked(0, Some("Thunderbird"));
        assert_eq!(next(&mut command), "counting");
        held.push(command);
    }
    let mut one_too_many = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut one_too_many), "cannot-count");

    let mut refused = Machine::with_mail_in(&[7]);
    refused.kernel.borrow_mut().refusing.push(7);
    let mut command = refused.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "cannot-count");
}

/// **A count that missed part of the application says so**: `stopped`, not
/// `sent`, because the number is less than everything it sent.
#[test]
fn a_count_that_missed_part_of_the_application_is_not_called_complete() {
    let mut machine = Machine::with_mail_in(&[7, 8]);
    machine.kernel.borrow_mut().refusing.push(8);
    let mut command = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "counting");
    machine.sends(7, 40);
    done(&mut command);
    machine.looked(1);
    assert_eq!(next(&mut command), "stopped 40");
}

/// **A command that says something other than `done` is finished with**,
/// with no answer: a caller with a bug is not told a number.
#[test]
fn a_command_that_says_anything_but_done_is_finished_with() {
    let mut machine = Machine::with_mail_in(&[7]);
    let mut command = machine.asked(0, Some("Thunderbird"));
    assert_eq!(next(&mut command), "counting");
    command.get_mut().write_all(b"finished\n").unwrap();
    machine.looked(1);
    assert_eq!(machine.counting.how_many_are_counting(), 0);
    assert_eq!(next(&mut command), "", "a caller with a bug was told something");
}
