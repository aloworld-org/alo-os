//! **A reader that falls over late is said to have**, however late and however
//! it falls — and one that keeps running is never said to have fallen.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::*;
use crate::starting_the_screen_reader::TheScreenReader;

/// A manager that answers a scripted state and restart count per look; the
/// last of each is repeated.
struct AWatchedManager {
    /// The states, one per look; [`None`] where the look cannot be asked.
    states: RefCell<VecDeque<Option<&'static str>>>,
    /// The restart counts, one per ask; [`None`] where it cannot be asked.
    restarts: RefCell<VecDeque<Option<u32>>>,
}

impl AWatchedManager {
    /// A manager that answers these states and these restart counts.
    fn answering(states: &[Option<&'static str>], restarts: &[Option<u32>]) -> Self {
        Self {
            states: RefCell::new(states.iter().copied().collect()),
            restarts: RefCell::new(restarts.iter().copied().collect()),
        }
    }
}

/// The next of a script, the last one repeated.
fn next<T: Copy>(script: &RefCell<VecDeque<T>>) -> Option<T> {
    let mut script = script.borrow_mut();
    if script.len() > 1 {
        script.pop_front()
    } else {
        script.front().copied()
    }
}

impl TheManager for AWatchedManager {
    fn start(&self, _unit: &str) -> Result<(), String> {
        unreachable!("a watch never starts the reader")
    }

    fn stop(&self, _unit: &str) -> Result<(), String> {
        unreachable!("a watch never stops the reader")
    }

    fn state(&self, unit: &str) -> Result<String, String> {
        assert_eq!(unit, THE_UNIT);
        next(&self.states)
            .flatten()
            .map(str::to_owned)
            .ok_or_else(|| "the bus is slow".to_owned())
    }

    fn restarts(&self, unit: &str) -> Result<u32, String> {
        assert_eq!(unit, THE_UNIT);
        next(&self.restarts)
            .flatten()
            .ok_or_else(|| "the bus is slow".to_owned())
    }
}

/// The looks a watch answers, one per look, for `looks` looks after it began.
fn watched(manager: &AWatchedManager, looks: usize) -> Vec<Option<TheScreenReaderAnswered>> {
    let mut watch = TheWatch::default();
    watch.began(manager);
    (0..looks).map(|_| watch.looked(manager)).collect()
}

/// **What the walk machine did**: running for a minute, then its unit starting
/// it again — said at the first look that sees it, which is the case the two
/// seconds missed.
#[test]
fn a_reader_that_falls_over_a_minute_later_did_not_start() {
    let mut states = vec![Some("active"); 30];
    states.push(Some("activating"));
    let manager = AWatchedManager::answering(&states, &[Some(0)]);
    let looks = watched(&manager, 31);
    let Some((last, running)) = looks.split_last() else {
        unreachable!("thirty-one looks were asked for");
    };
    assert!(running.iter().all(Option::is_none), "{looks:?}");
    assert_eq!(*last, Some(TheScreenReaderAnswered::DidNotStart));
}

/// **A fall between two looks is caught by the restart count**, though the
/// reader is back and `active` by the time it is looked at.
#[test]
fn a_fall_between_two_looks_is_caught() {
    let manager = AWatchedManager::answering(&[Some("active")], &[Some(2), Some(2), Some(3)]);
    assert_eq!(
        watched(&manager, 2),
        vec![None, Some(TheScreenReaderAnswered::DidNotStart)]
    );
}

/// **Every way a reader stops running while the switch is on is a fall.**
#[test]
fn every_way_of_not_running_is_a_fall() {
    for state in ["failed", "activating", "inactive", "deactivating"] {
        assert!(fell_over(state, Some(0), Some(0)), "{state}");
    }
    assert!(fell_over("active", Some(1), Some(0)));
    assert!(!fell_over("active", Some(0), Some(0)));
    // A count that could not be asked leaves the state to say it.
    assert!(!fell_over("active", None, Some(0)));
    assert!(!fell_over("active", Some(4), None));
    assert!(fell_over("failed", None, None));
}

/// **A reader that keeps running is never said to have fallen**, and a look
/// that cannot be asked is not a fall.
#[test]
fn a_reader_that_keeps_running_is_never_said_to_fall() {
    let running = AWatchedManager::answering(&[Some("active")], &[Some(1)]);
    assert!(watched(&running, 50).iter().all(Option::is_none));
    let slow = AWatchedManager::answering(&[None, Some("active")], &[Some(1)]);
    assert!(watched(&slow, 5).iter().all(Option::is_none));
}

/// A watcher that answers from a script and counts its looks.
struct AScriptedWatcher {
    /// The answers it gives, one per look; [`None`] once they run out.
    answers: VecDeque<Option<TheScreenReaderAnswered>>,
    /// Where each `began` and look is told.
    told: mpsc::Sender<&'static str>,
}

impl Watcher for AScriptedWatcher {
    fn began(&mut self) {
        let _ = self.told.send("began");
    }

    fn looked(&mut self) -> Option<TheScreenReaderAnswered> {
        let _ = self.told.send("looked");
        self.answers.pop_front().flatten()
    }
}

/// The answers the worker gives, waited for until `count` have come.
fn answers(reader: &mut TheScreenReader, count: usize) -> Vec<TheScreenReaderAnswered> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut answers = Vec::new();
    while answers.len() < count && Instant::now() < deadline {
        if let Some(answer) = reader.answered() {
            answers.push(answer);
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    answers
}

/// **The worker says a late fall, once, after it said *started*** — and stops
/// looking until the switch is turned over, so the person hears it once.
#[test]
fn the_worker_says_a_late_fall_once() {
    let (told, what_was_told) = mpsc::channel();
    let watcher = AScriptedWatcher {
        answers: [None, None, Some(TheScreenReaderAnswered::DidNotStart)]
            .into_iter()
            .collect(),
        told,
    };
    let mut reader = TheScreenReader::acted_on_by(
        |_| TheScreenReaderAnswered::Started,
        watcher,
        Duration::from_millis(1),
    );
    reader.is(true);
    assert_eq!(
        answers(&mut reader, 2),
        vec![
            TheScreenReaderAnswered::Started,
            TheScreenReaderAnswered::DidNotStart
        ]
    );
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(reader.answered(), None, "said once");
    assert_eq!(
        what_was_told.try_iter().collect::<Vec<_>>(),
        vec!["began", "looked", "looked", "looked"],
        "no look after the fall was said"
    );
}

/// **Only a reader that started is watched**: one that did not start, or was
/// stopped, is not looked at.
#[test]
fn only_a_started_reader_is_watched() {
    let (told, what_was_told) = mpsc::channel();
    let watcher = AScriptedWatcher {
        answers: VecDeque::new(),
        told,
    };
    let mut reader = TheScreenReader::acted_on_by(
        |on| {
            if on {
                TheScreenReaderAnswered::DidNotStart
            } else {
                TheScreenReaderAnswered::Stopped
            }
        },
        watcher,
        Duration::from_millis(1),
    );
    reader.is(true);
    reader.is(false);
    assert_eq!(
        answers(&mut reader, 2),
        vec![
            TheScreenReaderAnswered::DidNotStart,
            TheScreenReaderAnswered::Stopped
        ]
    );
    std::thread::sleep(Duration::from_millis(50));
    assert!(what_was_told.try_iter().next().is_none());
}

/// **Turning the switch off ends the watch**, and on again begins a new one.
#[test]
fn the_switch_turned_over_ends_the_watch_and_begins_another() {
    let (told, what_was_told) = mpsc::channel();
    let watcher = AScriptedWatcher {
        answers: VecDeque::new(),
        told,
    };
    let mut reader = TheScreenReader::acted_on_by(
        |on| {
            if on {
                TheScreenReaderAnswered::Started
            } else {
                TheScreenReaderAnswered::Stopped
            }
        },
        watcher,
        Duration::from_millis(1),
    );
    reader.is(true);
    assert_eq!(answers(&mut reader, 1).len(), 1);
    reader.is(false);
    assert_eq!(
        answers(&mut reader, 1),
        vec![TheScreenReaderAnswered::Stopped]
    );
    std::thread::sleep(Duration::from_millis(30));
    let told_before = what_was_told.try_iter().collect::<Vec<_>>();
    std::thread::sleep(Duration::from_millis(30));
    assert!(
        what_was_told.try_iter().next().is_none(),
        "nothing is looked at after the stop: {told_before:?}"
    );
    reader.is(true);
    assert_eq!(
        answers(&mut reader, 1),
        vec![TheScreenReaderAnswered::Started]
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut again = Vec::new();
    while !again.contains(&"looked") && Instant::now() < deadline {
        again.extend(what_was_told.try_iter());
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(again.first(), Some(&"began"), "{again:?}");
}
