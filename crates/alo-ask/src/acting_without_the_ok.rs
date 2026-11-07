//! **What alo in the terminal does with a change, when the person has let it
//! act without the OK.**
//!
//! `docs/features.md`, the owner's decision of 2026-11-06: *in Settings →
//! Terminal, the person can let what they ask alo for in the terminal run
//! without the OK — ADR 0064's third level, full trust, applied to the
//! terminal.* The switch is `alo_capability::AloInTheTerminal`, kept in
//! `/var/lib/alo/how-code-runs.toml` beside the level for running code.
//!
//! # Who says yes, and it is still the person's door
//!
//! **Nothing here carries anything out.** The agent proposes, as it always
//! does; the daemon holds the change, as it always does; and what this sends is
//! `Approve` on **the person's door** — the same line, on the same socket, that
//! `alo-shell` sends when somebody presses *Allow once*. This process runs as
//! the person, because the person typed it, so the daemon's `SO_PEERCRED` puts
//! it on that door and nothing else could. Which means the three things ADR
//! 0064 keeps on at every level are kept by the code that keeps them for every
//! approval: the daemon records the change and what it did, keeps a copy from
//! before it for Put back, and lights the indicator for whatever leaves. **One
//! approval is still one execution** — `alo_capability::Approved::redeem`'s,
//! not this file's.
//!
//! # Only this request's change, and only while the switch is on
//!
//! - **The change must be the one this request left waiting**: the number and
//!   the sentence the agent wrote back on this connection
//!   (`alo_protocol::what_the_agent_says_back`), and the same number and
//!   sentence among what the daemon says is waiting. A change the alo Bar, the
//!   launcher or a task window is holding is none of those, and is left for
//!   the person to answer where they asked.
//! - **The switch is read at the moment of answering**, after the daemon has
//!   said what is waiting and before anything is sent — not when the request
//!   was typed. Switching it off while alo is working takes effect for the
//!   change already waiting, which is then left unanswered and lapses with the
//!   turn.
//! - **A reply that is not a change approves nothing**, whatever its words: a
//!   model's answer is kept off the line that says a change waits.

use alo_capability::AloInTheTerminal;
use alo_changing::{Answered, Answering};
use alo_protocol::what_the_agent_says_back::the_change_said_to_be_waiting;

/// What the terminal did about the reply to one request.
#[derive(Debug, PartialEq, Eq)]
pub enum WhatTheTerminalDid {
    /// The reply left no change waiting: an answer, a refusal, or words.
    NothingWaited,
    /// The reply said a change was waiting, and the daemon was not holding
    /// that change — or did not say what it holds. Nothing was sent.
    NotTheChangeWaiting,
    /// The switch was off by the time the change could be answered, so it was
    /// left alone.
    SwitchedOffFirst,
    /// The change was carried out without the OK.
    WentAhead {
        /// What the person would have been asked, which is what was done.
        sentence: String,
        /// What the daemon said about it, and [`None`] when it said nothing.
        answered: Option<Answered>,
    },
}

/// Answer the change this reply left waiting, if the switch is on when it
/// comes to it.
///
/// `the_switch_now` is read once, at the last moment before anything is sent;
/// on a real machine it reads the person's file again.
pub fn what_the_terminal_does(
    reply: &str,
    daemon: &dyn Answering,
    the_switch_now: &dyn Fn() -> AloInTheTerminal,
) -> WhatTheTerminalDid {
    let Some((number, sentence)) = the_change_said_to_be_waiting(reply) else {
        return WhatTheTerminalDid::NothingWaited;
    };
    let ours = daemon.waiting().is_some_and(|waiting| {
        waiting
            .iter()
            .any(|it| it.number() == number && it.sentence().text() == sentence)
    });
    if !ours {
        return WhatTheTerminalDid::NotTheChangeWaiting;
    }
    if !the_switch_now().acts_without_asking() {
        return WhatTheTerminalDid::SwitchedOffFirst;
    }
    WhatTheTerminalDid::WentAhead {
        sentence: sentence.to_owned(),
        answered: daemon.approve(number),
    }
}

#[cfg(test)]
#[path = "acting_without_the_ok_tests.rs"]
mod tests;
