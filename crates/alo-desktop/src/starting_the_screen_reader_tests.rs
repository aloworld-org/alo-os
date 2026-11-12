//! **Every road through starting and stopping the screen reader**, against a
//! manager that answers what each test says — so the refusals are tested as
//! carefully as the start: the reader missing, the manager refusing, a reader
//! that falls over, the switch turned off while it runs, and one that will not
//! stop.

use std::cell::RefCell;
use std::collections::VecDeque;

use super::*;

/// A manager that records what it was asked and answers from a script.
struct AScriptedManager {
    /// What it was asked, in order: `start`, `stop`, `state`.
    asked: RefCell<Vec<&'static str>>,
    /// Whether a start or a stop is refused, and in what words.
    refuses: Option<&'static str>,
    /// The states it answers, one per look; the last is repeated.
    states: RefCell<VecDeque<&'static str>>,
}

impl AScriptedManager {
    /// A manager that answers these states, one per look.
    fn answering(states: &[&'static str]) -> Self {
        Self {
            asked: RefCell::new(Vec::new()),
            refuses: None,
            states: RefCell::new(states.iter().copied().collect()),
        }
    }

    /// A manager that refuses every start and stop in these words.
    fn refusing(why: &'static str) -> Self {
        Self {
            refuses: Some(why),
            ..Self::answering(&["inactive"])
        }
    }

    /// What it was asked, in order.
    fn was_asked(&self) -> Vec<&'static str> {
        self.asked.borrow().clone()
    }
}

impl TheManager for AScriptedManager {
    fn start(&self, unit: &str) -> Result<(), String> {
        assert_eq!(unit, THE_UNIT);
        self.asked.borrow_mut().push("start");
        self.refuses.map_or(Ok(()), |why| Err(why.to_owned()))
    }

    fn stop(&self, unit: &str) -> Result<(), String> {
        assert_eq!(unit, THE_UNIT);
        self.asked.borrow_mut().push("stop");
        self.refuses.map_or(Ok(()), |why| Err(why.to_owned()))
    }

    fn state(&self, unit: &str) -> Result<String, String> {
        assert_eq!(unit, THE_UNIT);
        self.asked.borrow_mut().push("state");
        let mut states = self.states.borrow_mut();
        let state = if states.len() > 1 {
            states.pop_front()
        } else {
            states.front().copied()
        };
        state
            .map(str::to_owned)
            .ok_or_else(|| "no answer".to_owned())
    }
}

/// No waiting in a test: the looks are what is being tested, not the clock.
fn no_wait() {}

/// States as the manager says them.
fn states(said: &[&str]) -> Vec<String> {
    said.iter().map(|state| (*state).to_owned()).collect()
}

/// **A reader that is not on this machine is said to be not here**, and the
/// manager is not asked to start a unit that would only fail.
#[test]
fn a_reader_that_is_not_here_is_not_started() {
    let manager = AScriptedManager::answering(&["active"]);
    assert_eq!(
        started(false, &manager, &no_wait),
        TheScreenReaderAnswered::NotOnThisComputer
    );
    assert!(manager.was_asked().is_empty(), "{:?}", manager.was_asked());
}

/// **A reader that starts and stays running is started**, and it is looked at
/// for the whole two seconds rather than believed at the first answer.
#[test]
fn a_reader_that_stays_running_is_started() {
    let manager = AScriptedManager::answering(&["activating", "active"]);
    assert_eq!(
        started(true, &manager, &no_wait),
        TheScreenReaderAnswered::Started
    );
    let asked = manager.was_asked();
    assert_eq!(asked.first(), Some(&"start"));
    assert_eq!(asked.iter().filter(|it| **it == "state").count(), LOOKS);
}

/// **A manager that refuses the start is a reader that did not start.**
#[test]
fn a_refused_start_did_not_start() {
    let manager = AScriptedManager::refusing("Unit alo-screen-reader.service not found.");
    assert_eq!(
        started(true, &manager, &no_wait),
        TheScreenReaderAnswered::DidNotStart
    );
    assert_eq!(manager.was_asked(), vec!["start"]);
}

/// **A reader that falls over is a reader that did not start**, however it
/// falls: straight to `failed`, or started again by its unit and so back to
/// `activating` after it had been active.
#[test]
fn a_reader_that_falls_over_did_not_start() {
    for script in [
        &["failed"][..],
        &["active", "active", "activating", "active"][..],
        &["active", "deactivating", "failed"][..],
        &["failed", "activating", "active"][..],
        &["activating"][..],
    ] {
        let manager = AScriptedManager::answering(script);
        assert_eq!(
            started(true, &manager, &no_wait),
            TheScreenReaderAnswered::DidNotStart,
            "{script:?}"
        );
    }
}

/// **A start whose state cannot be asked is not said to have worked.**
#[test]
fn a_start_nobody_can_look_at_did_not_start() {
    let manager = AScriptedManager::answering(&[]);
    assert_eq!(
        started(true, &manager, &no_wait),
        TheScreenReaderAnswered::DidNotStart
    );
}

/// **What a start came to**, read from the looks alone.
#[test]
fn what_a_start_came_to_is_read_from_every_look() {
    assert_eq!(
        what_the_start_came_to(&states(&["activating", "active", "active"])),
        TheScreenReaderAnswered::Started
    );
    assert_eq!(
        what_the_start_came_to(&states(&["active", "activating", "active"])),
        TheScreenReaderAnswered::DidNotStart
    );
    assert_eq!(
        what_the_start_came_to(&states(&[])),
        TheScreenReaderAnswered::DidNotStart
    );
}

/// **The switch turned off while the reader runs stops it**, and it is watched
/// until it is gone.
#[test]
fn the_switch_turned_off_while_it_runs_stops_it() {
    let manager = AScriptedManager::answering(&["active", "deactivating", "inactive"]);
    assert_eq!(
        stopped(&manager, &no_wait),
        TheScreenReaderAnswered::Stopped
    );
    assert_eq!(manager.was_asked(), vec!["state", "stop", "state", "state"]);
}

/// **A reader that is not running is not asked to stop** — every sign-in with
/// the switch off, which must say nothing and ask the manager nothing more.
#[test]
fn a_reader_that_is_not_running_is_left_alone() {
    for gone in ["inactive", "failed"] {
        let manager = AScriptedManager::answering(&[gone]);
        assert_eq!(
            stopped(&manager, &no_wait),
            TheScreenReaderAnswered::Stopped
        );
        assert_eq!(manager.was_asked(), vec!["state"]);
    }
}

/// **A reader that will not stop is said not to have**, so the person is not
/// told the screen has gone quiet while it is still being read aloud.
#[test]
fn a_reader_that_will_not_stop_did_not_stop() {
    let manager = AScriptedManager::answering(&["active"]);
    assert_eq!(
        stopped(&manager, &no_wait),
        TheScreenReaderAnswered::DidNotStop
    );
    let refused = AScriptedManager {
        states: RefCell::new(["active"].into_iter().collect()),
        ..AScriptedManager::refusing("Access denied")
    };
    assert_eq!(
        stopped(&refused, &no_wait),
        TheScreenReaderAnswered::DidNotStop
    );
}

/// **Only a language is ever written into the reader's environment.**
#[test]
fn only_a_language_reaches_the_readers_environment() {
    assert_eq!(the_language_line("de").as_deref(), Some("LC_ALL=de\n"));
    assert_eq!(
        the_language_line("pt-PT").as_deref(),
        Some("LC_ALL=pt_PT\n")
    );
    assert_eq!(the_language_line("ga").as_deref(), Some("LC_ALL=ga\n"));
    for not_a_language in [
        "",
        "d",
        "deutsch",
        "de\nLD_PRELOAD=/tmp/x",
        "de_DE_extra",
        "de DE",
        "1a",
        "de_",
    ] {
        assert_eq!(
            the_language_line(not_a_language),
            None,
            "{not_a_language:?}"
        );
    }
}

/// **The language is written where the unit reads it**, and a session with no
/// runtime folder writes nothing rather than somewhere else.
#[test]
fn the_language_is_written_where_the_unit_reads_it() {
    let Ok(folder) = tempfile::tempdir() else {
        unreachable!("no temporary folder");
    };
    the_language_is(Some(folder.path()), Some("el"));
    assert_eq!(
        std::fs::read_to_string(folder.path().join(THE_LANGUAGE_FILE))
            .ok()
            .as_deref(),
        Some("LC_ALL=el\n")
    );
    the_language_is(None, Some("el"));
    let Ok(other) = tempfile::tempdir() else {
        unreachable!("no temporary folder");
    };
    the_language_is(Some(other.path()), Some("not a language"));
    assert!(!other.path().join(THE_LANGUAGE_FILE).exists());
}

/// **One ask per change, carried out in order, and each answered once.**
#[test]
fn each_change_is_asked_once_and_answered_in_order() {
    let (asked, what_was_asked) = mpsc::channel();
    let mut reader = TheScreenReader::acted_on_by(move |on| {
        let _ = asked.send(on);
        if on {
            TheScreenReaderAnswered::Started
        } else {
            TheScreenReaderAnswered::Stopped
        }
    });
    reader.is(true);
    reader.is(true);
    reader.is(false);
    let mut answers = Vec::new();
    while answers.len() < 2 {
        if let Some(answer) = reader.answered() {
            answers.push(answer);
        } else {
            std::thread::yield_now();
        }
    }
    assert_eq!(
        answers,
        vec![
            TheScreenReaderAnswered::Started,
            TheScreenReaderAnswered::Stopped
        ]
    );
    assert_eq!(
        what_was_asked.try_iter().collect::<Vec<_>>(),
        vec![true, false],
        "the same switch twice is asked once"
    );
    assert_eq!(reader.answered(), None, "each answered once");
}
