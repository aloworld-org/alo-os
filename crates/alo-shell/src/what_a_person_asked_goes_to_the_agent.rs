//! **The join**: what a person typed leaves this process and an answer comes
//! back.
//!
//! Enter in the Dock's composer has recorded a request and opened a task window
//! since `the_composer_a_person_types_in` was written. **Nothing sent it
//! anywhere.** `direct_desktop`'s frame loop drained the request, made a window
//! for it, and that was the end of the road — so a person could ask and watch a
//! window open with nothing behind it.
//!
//! [ADR 0109](../../../docs/decisions/0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md)
//! decided where it goes: a socket the person's group may open, which systemd
//! accepts and hands to one `alo-agent` running as **the agent's own user** —
//! the thing neither this process nor the daemon may do, because the shell runs
//! as the person and `alo-agentd` refuses to run as root.
//!
//! # Nothing here waits, and that is the whole shape of the file
//!
//! A turn is a model thinking and takes seconds. This is read from a compositor's
//! frame loop, so writing and waiting here would stop every window on the
//! machine from drawing until a provider answered.
//!
//! So each request is **handed to a thread of its own**: it connects, writes one
//! line, waits as long as it likes, and posts what came back down a channel.
//! The loop asks `TheAgentsDoor::whatever_came_back` once a frame, which never
//! blocks and usually answers with nothing.
//!
//! One thread per request is right rather than thrifty: ADR 0109 makes one
//! connection one process one turn on the other side too, so a pool here would
//! be a pool of things each waiting on its own turn anyway.
//!
//! # What this does not do
//!
//! **It does not approve anything.** What comes back is words — a sentence
//! waiting, an answer, or a refusal — and a change still waits on the person's
//! own door where every other one does.
//!
//! **It does not decide which model answers.** That is the daemon's, from the
//! person's settings (ADR 0008), and this never learns which one it was.
//!
//! **And it never names the agent's user.** The socket's mode is what decides
//! who may open it, and `SO_PEERCRED` on the other side is what tells the agent
//! whose words these are. Nothing on this wire carries an identity.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::time::Duration;

/// Where a person's words are handed to their agent.
///
/// Part of ADR 0109's published arrangement, and the same path
/// `alo-asking.socket` listens on. Spelt here rather than read from the unit
/// because a compositor must not parse systemd's files to learn a filename —
/// and `tests/the_composer_reaches_the_agent.rs` holds this constant to the
/// unit, so the two cannot drift.
pub const WHERE_WORDS_GO: &str = "/run/alo-asking/socket";

/// How long one request may wait before this gives up on the machine.
///
/// Longer than a model takes and shorter than for ever. `alo-agent`'s own
/// patience at the daemon is five minutes, so this is that and a little: a
/// shorter one here would make this file the thing that decides how long a
/// model may take, which is `alo-models`' to decide.
const PATIENCE: Duration = Duration::from_secs(330);

/// Who asked, so an answer reaches the thing waiting for it.
///
/// **Two, since 2026-10-18.** It was `WhichTask` alone, and the order recorded
/// that widening it *cannot be built alone* because nothing would ever
/// construct a second arm. The owner's decision that an explanation comes
/// through the agent is what constructs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoAsked {
    /// A task window the person opened from the Dock's composer.
    ATaskWindow(crate::the_tasks_a_person_opened::WhichTask),
    /// alo's own terminal, which asked what a command would do before it ran.
    ///
    /// **No identifier, because there is one terminal.** When there are more,
    /// this takes one — and a variant that carried a number nothing could check
    /// would be worse than adding it then.
    TheTerminal,
}

/// What came back for one request.
///
/// **Not `Clone` and not compared whole**, because it can carry the request's
/// socket, which is neither: one request has one line to its agent, and a copy
/// of it would be a second person able to end that agent's turn.
#[derive(Debug)]
pub struct WhatCameBack {
    /// Who asked, so an answer reaches the thing waiting for it.
    pub who: WhoAsked,
    /// What the agent said, in the person's own language already — every
    /// sentence it carries was worded by the crate that owns it.
    pub said: String,
    /// Whether this is the machine failing rather than an answer.
    ///
    /// Told apart because a person reads them differently: an answer belongs in
    /// the conversation, and *this machine could not be asked* belongs beside it
    /// as something wrong with the machine.
    pub the_machines_fault: bool,
    /// The socket this request was asked on, still open, when the agent
    /// answered.
    ///
    /// **It keeps the agent's turn alive.** An agent that proposed a change
    /// holds its turn — and the change in it — until this closes, and then
    /// leaves (`alo_agent::holding_the_turn`). So it is kept while the question
    /// is on the person's screen and dropped once they have answered; dropped
    /// at once, which is what happens to it for any other answer, it lets an
    /// agent with nothing waiting go immediately.
    pub the_turn: Option<UnixStream>,
}

/// The door a person's words go out of.
pub struct TheAgentsDoor {
    /// Where to knock.
    socket: PathBuf,
    /// Threads post here; the frame loop drains.
    posting: Sender<WhatCameBack>,
    /// What has come back and not been read.
    arriving: Receiver<WhatCameBack>,
}

impl Default for TheAgentsDoor {
    fn default() -> Self {
        Self::at(Path::new(WHERE_WORDS_GO))
    }
}

impl TheAgentsDoor {
    /// A door at this socket.
    #[must_use]
    pub fn at(socket: &Path) -> Self {
        let (posting, arriving) = channel();
        Self {
            socket: socket.to_owned(),
            posting,
            arriving,
        }
    }

    /// Where this knocks, for a test to say so.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// Hand this request over, and return at once.
    ///
    /// **Never blocks.** The connecting, the writing and the waiting all happen
    /// on a thread of the request's own; what comes back arrives at
    /// [`Self::whatever_came_back`] some frames later.
    ///
    /// A machine with no socket there is not a failure worth stopping for — it
    /// is a machine where nothing is listening for a person's words, which is
    /// what a desktop with no daemon running looks like. It comes back as an
    /// answer saying so.
    /// `may_reach` is the folders this person has granted. They travel with
    /// the request so the model is not left guessing a path —
    /// `crate::what_the_agent_may_reach` holds why this side is the one that
    /// says so, and why naming a wrong one costs a round and nothing else.
    pub fn ask(&self, who: WhoAsked, request: &str, may_reach: &[std::path::PathBuf]) {
        let socket = self.socket.clone();
        let request = request.to_owned();
        let may_reach: Vec<std::path::PathBuf> = may_reach.to_vec();
        let posting = self.posting.clone();
        // A thread that cannot be started is reported the same way a socket
        // that cannot be reached is: as something wrong with the machine,
        // beside the request rather than instead of it.
        let started = std::thread::Builder::new()
            .name("alo-asking".to_owned())
            .spawn(move || {
                let came_back = ask_and_wait(&socket, &request, &may_reach);
                drop(posting.send(came_back.into_what_came_back(who)));
            });
        if let Err(why) = started {
            drop(self.posting.send(WhatCameBack {
                who,
                said: format!("this machine could not ask: {why}"),
                the_machines_fault: true,
                the_turn: None,
            }));
        }
    }

    /// Everything that has come back since this was last asked.
    ///
    /// **Drained rather than read**, so one answer reaches one window once —
    /// the same rule `Server::take_what_was_asked_of_alo` keeps at the other
    /// end of this road, and for the same reason.
    pub fn whatever_came_back(&self) -> Vec<WhatCameBack> {
        let mut came = Vec::new();
        loop {
            match self.arriving.try_recv() {
                Ok(what) => came.push(what),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return came,
            }
        }
    }
}

/// What one request's thread found.
enum Said {
    /// The agent answered, in words, on this socket.
    TheAgent(String, UnixStream),
    /// The machine could not be asked.
    TheMachine(String),
}

impl Said {
    /// The same thing, addressed to whoever asked.
    fn into_what_came_back(self, who: WhoAsked) -> WhatCameBack {
        match self {
            Self::TheAgent(said, socket) => WhatCameBack {
                who,
                said,
                the_machines_fault: false,
                the_turn: Some(socket),
            },
            Self::TheMachine(said) => WhatCameBack {
                who,
                said,
                the_machines_fault: true,
                the_turn: None,
            },
        }
    }
}

/// Connect, write one line, and read what comes back until it closes.
///
/// **Everything the agent wrote, not the first line.** `alo-agent` prints a
/// waiting change as two lines — the sentence is on the second — so reading one
/// would hand a person *a change is waiting for you to answer, as number 7* and
/// drop the sentence they are being asked to approve. Which is the one thing on
/// this wire that must never be lost.
fn ask_and_wait(socket: &Path, request: &str, may_reach: &[std::path::PathBuf]) -> Said {
    let machine = |why: &dyn std::fmt::Display| Said::TheMachine(format!("{why}"));

    let Ok(mut talking) = UnixStream::connect(socket) else {
        return Said::TheMachine(format!(
            "nothing is listening at {} for what you asked",
            socket.display()
        ));
    };
    if let Err(why) = talking.set_read_timeout(Some(PATIENCE)) {
        return machine(&why);
    }
    if let Err(why) = talking.set_write_timeout(Some(PATIENCE)) {
        return machine(&why);
    }
    // **The folders this person granted, first, and only when there are any.**
    // A line that does not begin with the marker is the request itself, which
    // is what keeps `alo-ask` and every other caller written before this
    // working unchanged. Tab-separated because a folder may hold a space and
    // may not hold a tab on any filesystem alo OS stands up on.
    let folders = may_reach
        .iter()
        .map(|folder| folder.display().to_string())
        .collect::<Vec<String>>()
        .join("\t");
    let said = if folders.is_empty() {
        format!("{request}\n")
    } else {
        format!("alo-may-reach:{folders}\n{request}\n")
    };
    if let Err(why) = talking
        .write_all(said.as_bytes())
        .and_then(|()| talking.flush())
    {
        return machine(&why);
    }
    // **This end is not shut for writing**, and it was until 2026-10-26. A
    // shut writing side is a close the agent reads at once, and the agent now
    // waits on exactly that to learn the person has answered
    // (`alo_agent::holding_the_turn`) — so shutting it here ended every
    // agent's turn before the card it was holding open could be drawn.
    // `alo-agent` reads one line, or two with the folders, and stops at each
    // newline, so it never needed the shut to know the request had ended.

    let mut heard = String::new();
    let mut reading = BufReader::new(&talking);
    loop {
        let mut line = String::new();
        match reading.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => heard.push_str(&line),
            Err(why) => return machine(&why),
        }
    }
    let heard = heard.trim();
    if heard.is_empty() {
        // Closed with nothing written, which is what the agent's own door does
        // to a caller it will not serve. From here it is the machine.
        return Said::TheMachine("your agent closed without answering".to_owned());
    }
    // **Handed back still open**, for `WhatCameBack::the_turn`'s reason.
    Said::TheAgent(heard.to_owned(), talking)
}

#[cfg(test)]
#[path = "what_a_person_asked_goes_to_the_agent_tests.rs"]
mod tests;
