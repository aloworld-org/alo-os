//! **The person's answer to a change is carried to the daemon off the frame
//! loop**, and what it did comes back a frame or more later.
//!
//! Until 2026-11-15 Allow once sent `approve` from inside the key press and
//! waited up to five seconds for the daemon's answer, on the loop that draws
//! every window and serves this shell's application door. Some changes are
//! carried out by the daemon **asking that door** — opening a file in an
//! application is one (`docs/contracts/application-door.md`). The daemon then
//! waited on a door nobody could serve until the shell stopped waiting on the
//! daemon, so such a change could never finish, and the person was told the
//! machine did not say what happened. Read off the code
//! (`alo_changing::door`'s five-second patience, this loop's own wait) after a
//! walk at the default level stopped there; not watched on a machine.
//!
//! So an answer is handed to a thread of its own, as a request to the agent
//! already is (`crate::what_a_person_asked_goes_to_the_agent`). The thread
//! sends the one `approve` or `decline`, waits as long as the daemon takes,
//! **lets the agent's turn go only after the daemon answered** — the change
//! runs inside that turn — and posts what came back. The loop drains it once a
//! frame and tells the person.
//!
//! # What it does not do
//!
//! **It does not keep *one approval is one execution*.** One press makes one
//! [`crate::a_change_waiting_reaches_the_person::TheAnswerGiven`], which this
//! sends once; the daemon is what refuses a second answer to a number.

use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use alo_changing::{Answered, Answering};

use crate::a_change_waiting_reaches_the_person::TheAnswerGiven;
use crate::the_tasks_a_person_opened::WhichTask;

/// The person's daemon, as a thread may hold it.
pub(crate) type TheirDaemon = Arc<dyn Answering + Send + Sync>;

/// What one answer did, for the task it was given in.
#[derive(Debug)]
pub(crate) struct WhatAnAnswerDid {
    /// The task whose card was answered.
    pub(crate) which: WhichTask,
    /// What the daemon said, and [`None`] when it said nothing.
    pub(crate) answered: Option<Answered>,
    /// The agent's turn, when it goes on after this answer and was told what
    /// the answer did — for the next thing it says to be heard on it.
    pub(crate) goes_on: Option<UnixStream>,
}

/// Answers on their way to the daemon.
#[derive(Debug)]
pub(crate) struct TheAnswersOnTheirWay {
    /// Threads post here.
    posting: Sender<WhatAnAnswerDid>,
    /// The frame loop drains here.
    arriving: Receiver<WhatAnAnswerDid>,
    /// How many have been handed over and not yet drained.
    under_way: usize,
}

impl Default for TheAnswersOnTheirWay {
    fn default() -> Self {
        let (posting, arriving) = channel();
        Self {
            posting,
            arriving,
            under_way: 0,
        }
    }
}

impl TheAnswersOnTheirWay {
    /// Carry this answer to the daemon, holding the agent's turn until it has
    /// answered, and return at once. **Never blocks.**
    pub(crate) fn carry(
        &mut self,
        which: WhichTask,
        given: TheAnswerGiven,
        the_turn: Option<UnixStream>,
        goes_on: bool,
        daemon: TheirDaemon,
    ) {
        let posting = self.posting.clone();
        self.under_way += 1;
        let started = std::thread::Builder::new()
            .name("alo-answering".to_owned())
            .spawn(move || {
                let answered = if given.allowed {
                    daemon.approve(given.number)
                } else {
                    daemon.decline(given.number)
                };
                let goes_on = told_the_agent(the_turn, goes_on, answered.as_ref());
                drop(posting.send(WhatAnAnswerDid {
                    which,
                    answered,
                    goes_on,
                }));
            });
        // A thread that could not start sent nothing: the person is told the
        // machine did not say, which is true, and the change lapses.
        if started.is_err() {
            drop(self.posting.send(WhatAnAnswerDid {
                which,
                answered: None,
                goes_on: None,
            }));
        }
    }

    /// Every answer that has come back since this was last asked. **Never
    /// blocks.**
    pub(crate) fn whatever_they_did(&mut self) -> Vec<WhatAnAnswerDid> {
        let mut came = Vec::new();
        while let Ok(did) = self.arriving.try_recv() {
            came.push(did);
        }
        self.under_way = self.under_way.saturating_sub(came.len());
        came
    }

    /// Wait for every answer handed over, for a test that stands in for the
    /// frames in between. Gives up after a bound, so a test never hangs.
    #[cfg(test)]
    pub(crate) fn every_one_of_them(&mut self) -> Vec<WhatAnAnswerDid> {
        let mut came = Vec::new();
        while self.under_way > came.len()
            && let Ok(did) = self
                .arriving
                .recv_timeout(std::time::Duration::from_secs(30))
        {
            came.push(did);
        }
        self.under_way = self.under_way.saturating_sub(came.len());
        came
    }
}

/// **Tell an agent that goes on what the answer did**, and hand its socket
/// back to be heard on — or let it go.
///
/// The line is written only **after** the daemon answered, so the change has
/// already run or not; it is the daemon's answer as the daemon wrote it, and
/// it approves nothing. An agent that does not go on, or a daemon that said
/// nothing, ends the turn by the socket closing here: **only now does the agent
/// go**, after the answer and never under it.
fn told_the_agent(
    the_turn: Option<UnixStream>,
    goes_on: bool,
    answered: Option<&Answered>,
) -> Option<UnixStream> {
    use std::io::Write as _;
    let the_turn = the_turn?;
    if !goes_on {
        return None;
    }
    let line = answered?.told_to_the_agent()?;
    (&the_turn)
        .write_all(format!("{line}\n").as_bytes())
        .ok()
        .map(|()| the_turn)
}

impl crate::Server {
    /// An answer the person gave has come back from the daemon: tell them what
    /// it did, and send a reply they held for it (ADR 0138).
    ///
    /// **The held reply goes only now**, after the daemon answered and the
    /// agent's turn was let go: the daemon serves one turn at a time, so a reply
    /// sent while the answer was on its way would be turned away.
    ///
    /// **Unless the agent goes on** (`goes_on`): then the reply is held until
    /// what it says next arrives, because its turn is still under way.
    pub(crate) fn an_answer_came_back_from_the_daemon(
        &mut self,
        did: &WhatAnAnswerDid,
        goes_on: bool,
        strings: &alo_strings::Strings,
    ) {
        if let Some(task) = self.the_tasks.the_mut(did.which) {
            // **The person is told what their answer did**, which nothing did
            // until 2026-10-26 — `crate::what_an_answer_did` says why. It is
            // also what moves the task off *waiting for your permission*.
            task.alo_says(alo_tasks::Turn::from_alo(
                crate::what_an_answer_did::what_an_answer_did(did.answered.as_ref(), strings),
            ));
            // **And alo is still working** while its turn goes on: the next
            // change or its last words are on their way.
            if goes_on {
                task.and_alo_goes_on();
            }
        }
        if !goes_on {
            self.send_what_was_held(did.which, strings);
        }
    }

    /// Wait for every answer on its way and tell the person, as the frames in
    /// between would.
    #[cfg(test)]
    pub(crate) fn every_answer_came_back(&mut self, strings: &alo_strings::Strings) {
        for did in self.the_answers.every_one_of_them() {
            self.an_answer_came_back_from_the_daemon(&did, did.goes_on.is_some(), strings);
        }
    }
}

#[cfg(test)]
#[path = "an_answer_goes_off_the_frame_tests.rs"]
mod tests;
