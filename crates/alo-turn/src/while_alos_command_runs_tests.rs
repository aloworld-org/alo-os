//! **What a command alo runs sends is on the indicator while it runs, and its
//! count ends when it ends** (ADR 0148, as amended 2026-11-13).
//!
//! The count is a stand-in here that answers a number per look: the kernel's
//! half is `alo-bounding`'s and `alo-agentd`'s, measured on a running kernel in
//! `crates/alo-bounding/tests/a_command_alo_runs_is_counted_and_nothing_beside_it.rs`.
//! What is under test is the turn's half — when the line goes up, when it comes
//! off, and what the record is handed.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use alo_egress::{Shown, Telling};

use super::*;
use crate::testing::{in_english, noon};

/// What the stand-in count was asked, and what it answers.
#[derive(Debug, Default)]
struct TheCount {
    /// What each look reads, in order; the last is read again once these run
    /// out.
    readings: Vec<Option<u64>>,
    /// How many looks have read it.
    read: usize,
    /// What the final reading answers when the count ends.
    at_the_end: Option<u64>,
    /// How many times the count was ended.
    ended: u32,
    /// How many looks came after it was ended.
    read_after_it_ended: u32,
}

/// A count of one command that answers [`TheCount`]'s numbers.
struct Counted(Rc<RefCell<TheCount>>);

impl ACommandCounted for Counted {
    fn starting(&mut self, start: &mut dyn FnMut() -> io::Result<Child>) -> io::Result<Child> {
        start()
    }

    fn so_far(&self) -> Option<u64> {
        let mut count = self.0.borrow_mut();
        if count.ended > 0 {
            count.read_after_it_ended += 1;
        }
        let at = count.read.min(count.readings.len().saturating_sub(1));
        count.read += 1;
        count.readings.get(at).copied().flatten()
    }

    fn ended(self: Box<Self>) -> Option<u64> {
        let mut count = self.0.borrow_mut();
        count.ended += 1;
        count.at_the_end
    }
}

/// Every list the indicator told, as the sentences on it — what a desktop
/// watching it would have drawn, one list per change.
#[derive(Debug, Clone, Default)]
struct Watched(Arc<Mutex<Vec<Vec<String>>>>);

impl Telling for Watched {
    fn now_showing(&self, showing: &[Shown]) {
        let strings = in_english();
        if let Ok(mut lists) = self.0.lock() {
            lists.push(
                showing
                    .iter()
                    .map(|shown| shown.said(&strings).text().to_owned())
                    .collect(),
            );
        }
    }
}

impl Watched {
    /// Every list told so far.
    fn lists(&self) -> Vec<Vec<String>> {
        self.0.lock().unwrap().clone()
    }
}

/// The line a counted command that sends is shown with.
const SENDING: &str = "curl, a command alo is running, is sending";

/// The line a command is shown with on a machine that cannot count it.
const UNCOUNTED: &str = "alo is running curl, and this machine cannot count what it sends";

/// A count answering `readings` on each look and `at_the_end` when it ends.
fn a_count(readings: &[Option<u64>], at_the_end: Option<u64>) -> Rc<RefCell<TheCount>> {
    Rc::new(RefCell::new(TheCount {
        readings: readings.to_vec(),
        at_the_end,
        ..TheCount::default()
    }))
}

/// **A command that sends is shown while it runs, and counted**: the line goes
/// up at the first look that finds a byte, stays while it runs, comes off when
/// it ends, and what it sent is answered for the record.
#[test]
fn a_command_that_sends_is_shown_while_it_runs_and_counted() {
    let count = a_count(&[Some(0), Some(0), Some(120), Some(300)], Some(4180));
    let watched = Watched::default();
    let mut indicator = Indicator::default().told_to(Box::new(watched.clone()));
    let mut around = WhileItRuns::begun(
        Some(Box::new(Counted(Rc::clone(&count)))),
        &mut indicator,
        "curl",
        noon(),
    );
    around.looking();
    around.looking();
    assert!(
        watched.lists().is_empty(),
        "a line went up before a byte left"
    );
    around.looking();
    assert_eq!(watched.lists(), [vec![SENDING.to_owned()]]);
    around.looking();
    assert_eq!(watched.lists().len(), 1, "one command put up two lines");
    assert_eq!(around.ended(), Some(4180));
    assert_eq!(
        watched.lists(),
        [vec![SENDING.to_owned()], Vec::new()],
        "the line did not come off when the command ended"
    );
    assert!(indicator.is_quiet());
    assert_eq!(count.borrow().ended, 1);
}

/// **A command that sends nothing shows nothing, and the record is handed
/// zero** — the line stays dark, and `0` is a number the kernel measured, not
/// a gap.
#[test]
fn a_command_that_sends_nothing_shows_nothing_and_is_counted_zero() {
    let count = a_count(&[Some(0)], Some(0));
    let watched = Watched::default();
    let mut indicator = Indicator::default().told_to(Box::new(watched.clone()));
    let mut around = WhileItRuns::begun(
        Some(Box::new(Counted(Rc::clone(&count)))),
        &mut indicator,
        "curl",
        noon(),
    );
    for _ in 0..5 {
        around.looking();
    }
    assert_eq!(around.ended(), Some(0));
    assert!(
        watched.lists().is_empty(),
        "the indicator changed for a command that sent nothing: {:?}",
        watched.lists()
    );
    assert_eq!(count.borrow().ended, 1);
}

/// **No egress of a command alo runs escapes the indicator** — the guarantee
/// `CLAUDE.md` makes in public, for commands: every way a command can end
/// having sent something leaves the line told to whoever watches before the
/// indicator goes quiet. That includes one that sent and ended between two
/// looks, and one stopped for running too long, whose last look found bytes.
#[test]
fn no_egress_of_a_command_alo_runs_escapes_the_indicator() {
    for (readings, at_the_end) in [
        // Seen sending on a look, and ended by itself.
        (vec![Some(10)], Some(10)),
        // Sent and ended between two looks: never seen on a look at all.
        (vec![Some(0)], Some(64)),
        // No look at all — it ended before the first.
        (vec![], Some(1)),
        // Stopped after two minutes: looked at until then, and the final
        // reading could not be taken, so the last look's stands.
        (vec![Some(0), Some(900)], None),
    ] {
        let count = a_count(&readings, at_the_end);
        let watched = Watched::default();
        let mut indicator = Indicator::default().told_to(Box::new(watched.clone()));
        let mut around = WhileItRuns::begun(
            Some(Box::new(Counted(Rc::clone(&count)))),
            &mut indicator,
            "curl",
            noon(),
        );
        for _ in 0..readings.len() {
            around.looking();
        }
        let sent = around.ended();
        assert!(
            sent.is_some_and(|bytes| bytes > 0),
            "{readings:?}: {sent:?}"
        );
        let lists = watched.lists();
        assert!(
            lists
                .iter()
                .any(|list| list.iter().any(|line| line == SENDING)),
            "{readings:?}: a command that sent {sent:?} bytes was never on the indicator: \
             {lists:?}"
        );
        assert_eq!(
            lists.last(),
            Some(&Vec::new()),
            "{readings:?}: the indicator was not quiet once it ended"
        );
        assert!(indicator.is_quiet());
    }
}

/// **The count ends when the command ends, once, and nothing reads it after**
/// — so nothing alo's command did is counted outside the seconds it ran.
#[test]
fn the_count_ends_when_the_command_ends_and_is_never_read_again() {
    let count = a_count(&[Some(5)], Some(5));
    let mut indicator = Indicator::default();
    let mut around = WhileItRuns::begun(
        Some(Box::new(Counted(Rc::clone(&count)))),
        &mut indicator,
        "curl",
        noon(),
    );
    around.looking();
    let read_while_it_ran = count.borrow().read;
    assert_eq!(around.ended(), Some(5));
    // `ended` took the value, so there is nothing left to look through; what
    // is checked is that ending it read nothing more.
    let ended = count.borrow();
    assert_eq!(ended.ended, 1, "the count was not ended exactly once");
    assert_eq!(
        ended.read, read_while_it_ran,
        "the count was read as it ended"
    );
    assert_eq!(ended.read_after_it_ended, 0);
}

/// **A machine that cannot count shows the command for its whole run, before
/// it starts, and says so** — and the record is handed no number, because
/// none was counted. The refusal path rides along: this crate starts nothing,
/// so the start here fails, and a start that fails still takes the line off.
#[test]
fn a_machine_that_cannot_count_shows_the_command_for_its_whole_run() {
    let watched = Watched::default();
    let mut indicator = Indicator::default().told_to(Box::new(watched.clone()));
    let mut around = WhileItRuns::begun(None, &mut indicator, "curl", noon());
    let seen_at_the_start = watched.clone();
    let mut start = || {
        assert_eq!(
            seen_at_the_start.lists(),
            [vec![UNCOUNTED.to_owned()]],
            "the line was not up before the command started"
        );
        Err(io::Error::other("it would not start"))
    };
    assert!(around.starting(&mut start).is_err());
    around.looking();
    assert_eq!(
        around.ended(),
        None,
        "a number was made up for a count nobody took"
    );
    assert_eq!(
        watched.lists(),
        [vec![UNCOUNTED.to_owned()], Vec::new()],
        "the line did not stay up for the run, or did not come off"
    );
    assert!(indicator.is_quiet());
}

/// **A count that could not be read at all is answered as none, never as
/// zero**: *it sent nothing* and *nobody could tell* are two different things
/// to be told.
#[test]
fn a_count_that_could_not_be_read_is_none_and_never_zero() {
    let count = a_count(&[None], None);
    let mut indicator = Indicator::default();
    let mut around = WhileItRuns::begun(
        Some(Box::new(Counted(Rc::clone(&count)))),
        &mut indicator,
        "curl",
        noon(),
    );
    around.looking();
    assert_eq!(around.ended(), None);
    assert!(indicator.is_quiet());
}
