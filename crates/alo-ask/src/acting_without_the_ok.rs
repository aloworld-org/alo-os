//! **What alo in the terminal does with a change: asks for the OK, or — when the
//! person has let it — acts without it.**
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
//!   change already waiting, which is then asked about instead.
//! - **A reply that is not a change approves nothing**, whatever its words: a
//!   model's answer is kept off the line that says a change waits, so a model
//!   that writes *Run this command? [y/N]* itself is printed as words and never
//!   asked about.
//!
//! # With the switch off, the person is asked
//!
//! *Added 2026-11-07.* ADR 0124's *Run this command? [y/N]*, in the terminal
//! the request was typed in (`crate::asking_in_the_terminal`). **Yes** sends one
//! approval of this change's number — the line *Allow once* sends. **No** sends
//! one decline, which the daemon records. **No answer** — no key while the
//! question stands, or a terminal that went away — sends nothing, and the
//! change lapses with the turn.

use std::time::Duration;

use alo_capability::AloInTheTerminal;
use alo_changing::{Answered, Answering, Standing};
use alo_protocol::what_the_agent_says_back::the_change_said_to_be_waiting;

use crate::asking_in_the_terminal::TheirAnswer;

/// How long the person is asked for, when the service did not say how long its
/// question stands.
///
/// The service's own figure is used whenever it gives one; this is only for a
/// service that does not, and it is the agent's own bound on a turn.
pub const WHEN_THE_SERVICE_DOES_NOT_SAY: Duration = Duration::from_secs(300);

/// What the terminal did about the reply to one request.
#[derive(Debug, PartialEq, Eq)]
pub enum WhatTheTerminalDid {
    /// The reply left no change waiting: an answer, a refusal, or words.
    NothingWaited,
    /// The reply said a change was waiting, and the daemon was not holding
    /// that change — or did not say what it holds. Nothing was sent.
    NotTheChangeWaiting,
    /// The change was carried out without the OK, because the person chose
    /// that in Settings → Terminal.
    WentAhead {
        /// What the person would have been asked, which is what was done.
        sentence: String,
        /// What the daemon said about it, and [`None`] when it said nothing.
        answered: Option<Answered>,
    },
    /// The person was asked and said yes.
    TheySaidYes {
        /// What the daemon said about it, and [`None`] when it said nothing.
        answered: Option<Answered>,
    },
    /// The person was asked and said no, and the no was sent.
    TheySaidNo {
        /// What the daemon said about it, and [`None`] when it said nothing.
        answered: Option<Answered>,
    },
    /// The person was asked and did not answer — no key in time, or the
    /// terminal went away. Nothing was sent.
    NobodyAnswered,
}

/// Answer the change this reply left waiting: without the OK if the switch is
/// on when it comes to it, and otherwise by asking the person.
///
/// `the_switch_now` is read once, at the last moment before anything is sent;
/// on a real machine it reads the person's file again. `ask` puts *Run this
/// command? [y/N]* to the person, given the change and how long it stands.
pub fn what_the_terminal_does(
    reply: &str,
    daemon: &dyn Answering,
    the_switch_now: &dyn Fn() -> AloInTheTerminal,
    ask: &mut dyn FnMut(Duration) -> TheirAnswer,
) -> WhatTheTerminalDid {
    let Some((number, sentence)) = the_change_said_to_be_waiting(reply) else {
        return WhatTheTerminalDid::NothingWaited;
    };
    let Some(ours) = daemon.waiting().and_then(|waiting| {
        waiting
            .into_iter()
            .find(|it| it.number() == number && it.sentence().text() == sentence)
    }) else {
        return WhatTheTerminalDid::NotTheChangeWaiting;
    };
    if the_switch_now().acts_without_asking() {
        return WhatTheTerminalDid::WentAhead {
            sentence: sentence.to_owned(),
            answered: daemon.approve(number),
        };
    }
    // **Asked, and the answer is the person's alone.** One yes is one approval
    // of this number; one no is one decline of it, which the daemon records.
    match ask(how_long_it_stands(&ours)) {
        TheirAnswer::Yes => WhatTheTerminalDid::TheySaidYes {
            answered: daemon.approve(number),
        },
        TheirAnswer::No => WhatTheTerminalDid::TheySaidNo {
            answered: daemon.decline(number),
        },
        TheirAnswer::NoAnswerInTime | TheirAnswer::TheTerminalWentAway => {
            WhatTheTerminalDid::NobodyAnswered
        }
    }
}

/// How long the question about this change stands, as the service says.
fn how_long_it_stands(change: &Standing) -> Duration {
    change
        .lapses_in()
        .map_or(WHEN_THE_SERVICE_DOES_NOT_SAY, Duration::from_secs)
}

#[cfg(test)]
#[path = "acting_without_the_ok_tests.rs"]
mod tests;
