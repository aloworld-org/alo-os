//! An answer carried to a stand-in daemon on a thread, and what the agent's
//! end of a real socket pair hears afterwards.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Read as _};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex, PoisonError};

use alo_changing::{Answered, Answering, Done, Standing};

use super::TheAnswersOnTheirWay;
use crate::a_change_waiting_reaches_the_person::TheAnswerGiven;
use crate::the_tasks_a_person_opened::WhichTask;

/// A daemon that answers every approve and decline with `answers`, and
/// remembers each number it was given.
#[derive(Debug)]
struct ADaemon {
    answers: Option<Answered>,
    given: Mutex<Vec<(u64, bool)>>,
}

impl ADaemon {
    fn answering(answers: Option<Answered>) -> Arc<Self> {
        Arc::new(Self {
            answers,
            given: Mutex::new(Vec::new()),
        })
    }

    fn given(&self) -> Vec<(u64, bool)> {
        self.given
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Answering for ADaemon {
    fn waiting(&self) -> Option<Vec<Standing>> {
        Some(Vec::new())
    }

    fn approve(&self, number: u64) -> Option<Answered> {
        self.given
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((number, true));
        self.answers.clone()
    }

    fn decline(&self, number: u64) -> Option<Answered> {
        self.given
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((number, false));
        self.answers.clone()
    }
}

/// Carry one answer and wait for it, handing back the agent's end.
fn carried(
    daemon: &Arc<ADaemon>,
    allowed: bool,
    goes_on: bool,
) -> (UnixStream, Option<UnixStream>) {
    let (ours, agents_end) = UnixStream::pair().unwrap();
    let mut answers = TheAnswersOnTheirWay::default();
    answers.carry(
        WhichTask::numbered(0),
        TheAnswerGiven { number: 9, allowed },
        Some(ours),
        goes_on,
        Arc::clone(daemon) as super::TheirDaemon,
    );
    let mut came = answers.every_one_of_them();
    assert_eq!(came.len(), 1, "one answer did not come back once");
    (agents_end, came.pop().and_then(|did| did.goes_on))
}

/// **An agent that goes on is told the daemon's own answer, once, and only
/// after the daemon gave it**; its turn is handed back to be heard on.
#[test]
fn an_agent_that_goes_on_is_told_the_daemons_answer_once() {
    let renamed = Answered::It(Box::new(Done::Renamed { now_at: None }));
    for (allowed, answer) in [(true, renamed), (false, Answered::Declined)] {
        let daemon = ADaemon::answering(Some(answer.clone()));
        let (agents_end, goes_on) = carried(&daemon, allowed, true);

        assert_eq!(daemon.given(), [(9, allowed)], "not one answer, once");
        let mut line = String::new();
        BufReader::new(&agents_end).read_line(&mut line).unwrap();
        assert_eq!(
            line.trim_end(),
            answer.told_to_the_agent().unwrap(),
            "the agent was not told what the daemon said"
        );
        assert!(goes_on.is_some(), "the turn was not handed back");
        // Nothing more was written: the shell says one line per answer.
        drop(goes_on);
        let mut rest = Vec::new();
        (&agents_end).read_to_end(&mut rest).unwrap();
        assert!(rest.is_empty(), "more was said: {rest:?}");
    }
}

/// **An agent from before is let go after the answer, told nothing**, which is
/// how its turn has always ended.
#[test]
fn an_agent_that_does_not_go_on_is_let_go_with_nothing_written() {
    let daemon = ADaemon::answering(Some(Answered::Declined));
    let (agents_end, goes_on) = carried(&daemon, false, false);
    assert!(goes_on.is_none());
    let mut rest = Vec::new();
    (&agents_end).read_to_end(&mut rest).unwrap();
    assert!(rest.is_empty(), "an agent from before was told {rest:?}");
}

/// **A daemon that said nothing is not built on**: the turn ends and the agent
/// is told nothing, because nobody knows what became of the change.
#[test]
fn a_daemon_that_said_nothing_ends_the_turn() {
    let daemon = ADaemon::answering(None);
    let (agents_end, goes_on) = carried(&daemon, true, true);
    assert!(goes_on.is_none());
    let mut rest = Vec::new();
    (&agents_end).read_to_end(&mut rest).unwrap();
    assert!(rest.is_empty(), "the agent was told {rest:?}");
}

/// A daemon whose answer waits until the test lets it.
#[derive(Debug)]
struct ADaemonThatWaits {
    released: Mutex<std::sync::mpsc::Receiver<()>>,
}

impl Answering for ADaemonThatWaits {
    fn waiting(&self) -> Option<Vec<Standing>> {
        Some(Vec::new())
    }

    fn approve(&self, _number: u64) -> Option<Answered> {
        let released = self
            .released
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .recv_timeout(std::time::Duration::from_secs(10));
        released
            .is_ok()
            .then(|| Answered::It(Box::new(Done::Renamed { now_at: None })))
    }

    fn decline(&self, _number: u64) -> Option<Answered> {
        Some(Answered::Declined)
    }
}

/// **Nothing reaches the agent, and nothing comes back to the frame, before the
/// daemon has answered** — and the frame's own look at what came back never
/// waits for it.
#[test]
fn nothing_is_told_before_the_daemon_answers_and_the_frame_never_waits() {
    let (release, released) = std::sync::mpsc::channel();
    let daemon = Arc::new(ADaemonThatWaits {
        released: Mutex::new(released),
    });
    let (ours, agents_end) = UnixStream::pair().unwrap();
    let mut answers = TheAnswersOnTheirWay::default();
    answers.carry(
        WhichTask::numbered(0),
        TheAnswerGiven {
            number: 9,
            allowed: true,
        },
        Some(ours),
        true,
        daemon as super::TheirDaemon,
    );

    let looked = std::time::Instant::now();
    assert!(
        answers.whatever_they_did().is_empty(),
        "an answer came back early"
    );
    assert!(
        looked.elapsed() < std::time::Duration::from_millis(500),
        "the frame waited for the daemon"
    );
    agents_end.set_nonblocking(true).unwrap();
    let early = (&agents_end).read(&mut [0_u8; 1]);
    assert!(
        matches!(&early, Err(why) if why.kind() == std::io::ErrorKind::WouldBlock),
        "the agent heard something, or its turn ended, before the daemon answered: {early:?}"
    );
    agents_end.set_nonblocking(false).unwrap();

    let _ = release.send(());
    let came = answers.every_one_of_them();
    assert_eq!(came.len(), 1);
    let mut line = String::new();
    BufReader::new(&agents_end).read_line(&mut line).unwrap();
    assert!(line.starts_with("alo-answered:"), "{line}");
}

/// **What the frame drains is what the daemon said**: once the answer is back,
/// the frame's own look hands it over, once.
#[test]
fn the_frame_drains_the_answer_once_it_is_back() {
    let daemon = ADaemon::answering(Some(Answered::Declined));
    let mut answers = TheAnswersOnTheirWay::default();
    answers.carry(
        WhichTask::numbered(3),
        TheAnswerGiven {
            number: 4,
            allowed: false,
        },
        None,
        false,
        daemon as super::TheirDaemon,
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut came = Vec::new();
    while came.is_empty() && std::time::Instant::now() < deadline {
        came = answers.whatever_they_did();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(came.len(), 1, "the frame never saw the answer");
    assert_eq!(
        came.first().and_then(|did| did.answered.clone()),
        Some(Answered::Declined)
    );
    assert!(
        answers.whatever_they_did().is_empty(),
        "it was handed over twice"
    );
}
