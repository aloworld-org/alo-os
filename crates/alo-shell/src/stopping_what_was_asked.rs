//! **Stop, on the road a request took** — the shell's half of
//! [ADR 0139](../../../docs/decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md).
//!
//! A request goes out on a connection of its own, on a thread of its own
//! (`crate::what_a_person_asked_goes_to_the_agent`), and the agent reads the
//! other end. Stopping it is **shutting this side's sending half**: a new
//! meaning on the asking socket (`docs/contracts/asking-socket.md`), which the
//! agent hears between rounds and while a question is out, and on which it asks
//! for nothing more and has the daemon cancel what it was sending.
//!
//! **This says nothing about whether it worked.** The request's own thread goes
//! on reading until the agent's side closes, and that close is the one moment
//! the window may say *stopped* — never the press.
//!
//! # A turn held for a person's answer is stopped the same way
//!
//! After a proposal, the request's connection is not with its thread any more
//! but with the card that keeps the agent's turn open
//! (`crate::a_change_waiting_reaches_the_person`). [`TheRequestsRunning`] only
//! knows connections still being read, so a held one is stopped by
//! [`stop_a_held_turn`], which shuts it the same way and reads it to its end on
//! a thread of its own.

use std::io::Read as _;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, PoisonError};

use crate::what_a_person_asked_goes_to_the_agent::{WhatCameBack, WhoAsked};

/// One request still being read, and whether a Stop was sent on it.
/// What became of a Stop this side tried to hand to the agent.
///
/// **Three answers, not a `bool`.** Until 2026-11-10 this answered `true` or
/// `false`, and `false` meant both *nothing was running* and *the Stop could
/// not be handed over* — so a Stop that never reached the agent was reported
/// to the person as *stopped*, the one thing a Stop must never claim while it
/// might be false.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheStop {
    /// Nothing was running, so the work had already ended.
    NothingWasRunning,
    /// Handed over: *stopped* follows once the agent has gone.
    Sent,
    /// Something was running and the Stop could not be handed over, so
    /// nothing can confirm the work ended.
    CouldNotBeSent,
}

#[derive(Debug)]
struct Running {
    /// Who asked.
    who: WhoAsked,
    /// A second handle on its connection.
    connection: UnixStream,
    /// Whether the person stopped it.
    stopped: bool,
}

/// **The requests whose answers are still being read**, by who asked.
#[derive(Debug, Clone, Default)]
pub(crate) struct TheRequestsRunning(Arc<Mutex<Vec<Running>>>);

impl TheRequestsRunning {
    /// A request from `who` is out on `connection`.
    pub(crate) fn began(&self, who: WhoAsked, connection: &UnixStream) {
        if let Ok(connection) = connection.try_clone() {
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(Running {
                    who,
                    connection,
                    stopped: false,
                });
        }
    }

    /// The request from `who` has been read to its end: forget it, and say
    /// whether it was stopped.
    pub(crate) fn ended(&self, who: WhoAsked) -> bool {
        let mut running = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(at) = running.iter().position(|one| one.who == who) else {
            return false;
        };
        running.remove(at).stopped
    }

    /// **Stop the request from `who`**: shut this side's sending half. Answers
    /// whether one was running, and whether the Stop could be handed over.
    pub(crate) fn stop(&self, who: WhoAsked) -> TheStop {
        let mut running = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(one) = running.iter_mut().find(|one| one.who == who) else {
            return TheStop::NothingWasRunning;
        };
        one.stopped = true;
        if one.connection.shutdown(std::net::Shutdown::Write).is_ok() {
            TheStop::Sent
        } else {
            TheStop::CouldNotBeSent
        }
    }
}

/// **Stop a turn the card is holding open**: shut its sending half, and once
/// the agent's side has closed, say so to whoever drains `posting`.
///
/// The change on the card is neither approved nor declined by this: the turn
/// it lives in ends, so it lapses with it.
pub(crate) fn stop_a_held_turn(who: WhoAsked, turn: UnixStream, posting: Sender<WhatCameBack>) {
    drop(turn.shutdown(std::net::Shutdown::Write));
    let read_to_the_end = move || {
        let mut ignored = [0_u8; 256];
        let mut reading = &turn;
        while matches!(reading.read(&mut ignored), Ok(read) if read > 0) {}
        drop(posting.send(WhatCameBack::stopped(who)));
    };
    // **No thread, no word**: the connection is shut and dropped here, and
    // nothing is posted — so the window stays *stopping* rather than claiming
    // *stopped* without having seen the agent go.
    drop(
        std::thread::Builder::new()
            .name("alo-asking-stop".to_owned())
            .spawn(read_to_the_end),
    );
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::the_tasks_a_person_opened::WhichTask;

    /// **A Stop shuts the sending side the agent reads**, and the request is
    /// remembered as stopped when it ends.
    #[test]
    fn a_stop_shuts_the_side_the_agent_reads_and_is_remembered() {
        let running = TheRequestsRunning::default();
        let who = WhoAsked::ATaskWindow(WhichTask::numbered(0));
        let (shell, mut agent) = UnixStream::pair().unwrap();
        running.began(who, &shell);
        assert_eq!(running.stop(who), TheStop::Sent);
        assert_eq!(
            agent.read(&mut [0_u8; 1]).unwrap(),
            0,
            "the agent heard nothing"
        );
        assert!(running.ended(who));
        assert!(!running.ended(who), "a request ended twice");
    }

    /// **A Stop for something not running stops nothing**, and a request
    /// nobody stopped ends as not stopped.
    #[test]
    fn a_stop_for_nothing_running_stops_nothing() {
        let running = TheRequestsRunning::default();
        let who = WhoAsked::ATaskWindow(WhichTask::numbered(1));
        assert_eq!(running.stop(who), TheStop::NothingWasRunning);
        let (shell, _agent) = UnixStream::pair().unwrap();
        running.began(who, &shell);
        assert!(!running.ended(who));
    }

    /// **A held turn is stopped, and *stopped* is said only once the agent's
    /// side has closed.**
    #[test]
    fn a_held_turn_is_said_to_be_stopped_only_when_the_agent_goes() {
        let who = WhoAsked::ATaskWindow(WhichTask::numbered(2));
        let (shell, agent) = UnixStream::pair().unwrap();
        let (posting, arriving) = std::sync::mpsc::channel();
        stop_a_held_turn(who, shell, posting);
        assert!(
            arriving
                .recv_timeout(std::time::Duration::from_millis(200))
                .is_err(),
            "said stopped while the agent was still there"
        );
        drop(agent);
        let came = arriving
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        assert!(came.stopped);
        assert_eq!(came.who, who);
    }
}
