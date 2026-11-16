//! An answer carried to a stand-in daemon on a thread, and what the frame
//! drains afterwards.

use std::sync::{Arc, Mutex, PoisonError};

use alo_changing::{Answered, Answering, Standing};

use super::TheAnswersOnTheirWay;
use crate::a_change_waiting_reaches_the_person::TheAnswerGiven;
use crate::the_tasks_a_person_opened::WhichTask;

/// A daemon that answers every decline with `Declined`, and counts them.
#[derive(Debug, Default)]
struct ADaemon {
    declined: Mutex<Vec<u64>>,
}

impl Answering for ADaemon {
    fn waiting(&self) -> Option<Vec<Standing>> {
        Some(Vec::new())
    }

    fn approve(&self, _number: u64) -> Option<Answered> {
        None
    }

    fn decline(&self, number: u64) -> Option<Answered> {
        self.declined
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(number);
        Some(Answered::Declined)
    }
}

/// **What the frame drains is what the daemon said**: once the answer is back,
/// the frame's own look hands it over, once, and the daemon was asked once.
#[test]
fn the_frame_drains_the_answer_once_it_is_back() {
    let daemon = Arc::new(ADaemon::default());
    let mut answers = TheAnswersOnTheirWay::default();
    answers.carry(
        WhichTask::numbered(3),
        TheAnswerGiven {
            number: 4,
            allowed: false,
        },
        None,
        Arc::clone(&daemon) as super::TheirDaemon,
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
    assert_eq!(
        *daemon
            .declined
            .lock()
            .unwrap_or_else(PoisonError::into_inner),
        [4],
        "not one answer, once"
    );
}
