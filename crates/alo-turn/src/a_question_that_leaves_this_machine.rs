//! Law 1's sequence for a question that leaves this machine, on the machine
//! rather than on a turn.
//!
//! *Nothing leaves silently. Every network egress an agent causes is visible at
//! the moment it happens and afterwards in a record.*
//!
//! This was `crate::asking`'s, spread across six methods of [`Turning`], and it
//! moved here on 2026-10-19 for one reason: **a person's question needs the
//! same sequence and is not a turn.**
//! [ADR 0125](../../../docs/decisions/0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)
//! decided that a short-lived command asks the daemon to cause its egress, and
//! the daemon is not a turn either — it holds a [`Machine`].
//!
//! # One copy, two callers, and the reason it is not two copies
//!
//! `put_off_this_machine`'s own documentation named the failure mode before
//! this file existed: **a call that compiles, with the right types, while
//! nobody sees anything.** Two copies of this sequence is how one of them
//! quietly stops showing — and the one that stops is the one nobody is
//! watching, which on this road would be the person's.
//!
//! So [`Turning`] delegates here and adds only what is a turn's: closing itself
//! when the record breaks, and remembering a thread left inside a boundary that
//! is over.
//!
//! # The order is the law, and it is the only thing here
//!
//! | step | why it is where it is |
//! |---|---|
//! | the boundary and the indicator, taken together | `Machine` holds both, and a caller that could take one without the other could bound a departure nobody was shown |
//! | the boundary imposed, permitting the one resolved address | ADR 0020. The address is resolved **by the caller, before this** — a request to an address nobody registered reaches somewhere nobody was shown |
//! | the question put **inside** it, through a closure run once | one permission is one attempt, and `alo_egress::Indicator::beginning` hands back the `Departing` that is the permission |
//! | `Entry::left` written **before** the line comes off | law 1's two halves are one moment. A record written afterwards is a record that can be missed by exactly the crash that matters |
//! | the departure ended | the line comes off the indicator, last |
//!
//! # Every road out is written down, including the five that are not an answer
//!
//! No boundary; a boundary imposed that ran nothing; held back by the rule; it
//! left and nothing came back; and the record itself breaking. Each is an entry
//! or a refusal with its own sentence, and `crate::asking`'s header has the
//! table of which.

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::SystemTime;

use alo_asking::{Answer, Asked, DidNotAnswer, NotAsked};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_keeping::NotKept;
use alo_record::Entry;

use crate::machine::Machine;
use crate::unanswered::NoAnswer;
use crate::unbounded::NoBoundary;

/// What became of a question that was meant to leave this machine.
///
/// The outcome, and the two things a **turn** has to do about it that a daemon
/// does not. They are reported rather than done here because this file holds a
/// machine and neither is a machine's business: a turn closes, and a daemon
/// serving a person's question has no turn to close.
#[derive(Debug)]
pub struct WhatBecameOfIt {
    /// The answer, or why there was none.
    pub outcome: Result<Answer, NoAnswer>,
    /// The record would not take an entry, so a caller that keeps evidence
    /// must stop.
    ///
    /// **Separate from the outcome**, because the outcome already says what a
    /// person is told and this says what the caller must do next — and a turn
    /// that read one for the other would go on serving with nothing being
    /// written down.
    pub the_record_broke: bool,
    /// A thread of the caller is inside a boundary that is over.
    pub a_thread_was_lost: bool,
}

impl WhatBecameOfIt {
    /// An outcome with neither of the two having happened.
    fn just(outcome: Result<Answer, NoAnswer>) -> Self {
        Self {
            outcome,
            the_record_broke: false,
            a_thread_was_lost: false,
        }
    }
}

/// Why nothing left this machine, when there was no boundary to put a question
/// inside.
///
/// **A type of its own rather than a [`WhatBecameOfIt`]**, because this road
/// has no answer on it — ever. Handing back a `Result` here would make every
/// caller write an `Ok` arm that cannot be taken, and the honest thing to put
/// in one is nothing.
#[derive(Debug)]
pub(crate) struct NothingLeft {
    /// What the caller is told.
    pub why: NoAnswer,
    /// The record would not take the refusal either.
    pub the_record_broke: bool,
    /// A thread of the caller is inside a boundary that is over.
    pub a_thread_was_lost: bool,
}

impl From<NothingLeft> for WhatBecameOfIt {
    fn from(nothing: NothingLeft) -> Self {
        Self {
            outcome: Err(nothing.why),
            the_record_broke: nothing.the_record_broke,
            a_thread_was_lost: nothing.a_thread_was_lost,
        }
    }
}

/// Put a question that leaves this machine from inside a boundary permitting
/// `to` and nothing else — on the interface `held_to` names, where it names
/// one — show it while it is in flight, and write down what became of it.
///
/// One road for a provider, a paired machine and a person's own question,
/// because what law 1 and ADR 0020 ask of all three is the same.
///
/// `asked_by` is written into the refusals that name whoever asked. For an
/// agent it is the agent; for a person's question it is the command that put
/// it, which is `alo-ask`'s own recorded finding about the shape of
/// [`Grantee`]: it has two kinds and both are software, so a person is named by
/// the thing they typed rather than by a login written into a type for
/// programs.
pub fn a_question_that_leaves_this_machine(
    machine: &mut Machine<'_>,
    to: &[SocketAddr],
    held_to: Option<NonZeroU32>,
    asked_by: &Grantee,
    now: SystemTime,
    put: impl FnOnce(&mut Indicator) -> Result<Asked, NotAsked>,
) -> WhatBecameOfIt {
    let (bounding, indicator) = machine.bounding_and_indicator();
    // Moved into the closure once. The door takes `self` because one permission
    // is one attempt, and this is that one attempt.
    let mut once = Some(put);
    let mut outcome = None;
    let mut putting = || {
        if let Some(put) = once.take() {
            outcome = Some(put(indicator));
        }
    };
    let bounded = match held_to {
        Some(interface) => bounding.carrying_out_a_departure_on(to, interface, &mut putting),
        None => bounding.carrying_out_a_departure(to, &mut putting),
    };
    match (bounded, outcome) {
        (Ok(()), Some(outcome)) => what_it_did(machine, outcome, now),
        // A boundary that could not be imposed is ADR 0015's rule rather than a
        // smaller question: nothing was asked and nothing left. A boundary that
        // was imposed and ran nothing is the same fact about this machine, said
        // the same way — and both are written down as the machine's own
        // refusal.
        (Err(why), _) => nothing_was_bounded(machine, why, asked_by, now).into(),
        (Ok(()), None) => nothing_was_bounded(
            machine,
            NoBoundary::because(
                "the boundary was imposed and the question was not put inside it".to_owned(),
            ),
            asked_by,
            now,
        )
        .into(),
    }
}

/// What came back from somewhere off this machine, written down and then handed
/// over.
///
/// Every road where something left ends with the departure written **before**
/// the line comes off the indicator and before the caller hears anything:
/// `alo_record::Entry::left` is made from the token the door hands back and
/// from nothing else, which is what makes an egress the indicator showed an
/// entry that can be written.
fn what_it_did(
    machine: &mut Machine<'_>,
    outcome: Result<Asked, NotAsked>,
    now: SystemTime,
) -> WhatBecameOfIt {
    match outcome {
        Ok(asked) => {
            let kept = keep(machine, Entry::left(asked.departing()));
            let answer = asked.ended(machine.indicator());
            match kept {
                Ok(()) => WhatBecameOfIt::just(Ok(answer)),
                Err(why) => broke(after_it_left(why)),
            }
        }
        Err(NotAsked::DidNotAnswer(unanswered)) => it_did_not_answer(machine, *unanswered),
        Err(NotAsked::HeldBack(refused)) => {
            let entry = Entry::held_back(&refused, machine.strings(), now);
            match keep(machine, entry) {
                Ok(()) => WhatBecameOfIt::just(Err(NoAnswer::HeldBack(refused))),
                Err(why) => broke(nothing_left(why)),
            }
        }
        // Neither of these addressed anything or sent anything, and
        // `crate::unanswered` has the argument for why neither is an entry.
        Err(NotAsked::CannotBeShown(why)) => {
            WhatBecameOfIt::just(Err(NoAnswer::CannotBeShown(why)))
        }
        Err(NotAsked::Miswired(why)) => WhatBecameOfIt::just(Err(why.into())),
    }
}

/// It went, and nothing came back.
///
/// A road of its own because it is the one a record of successes would lose:
/// the question left, so law 1's *what left this machine today* has to find it,
/// and the failure the person is shown is handed on afterwards without this
/// crate reading a word of it.
fn it_did_not_answer(machine: &mut Machine<'_>, unanswered: DidNotAnswer) -> WhatBecameOfIt {
    let kept = keep(machine, Entry::left(unanswered.departing()));
    let failed = unanswered.ended(machine.indicator());
    match kept {
        Ok(()) => WhatBecameOfIt::just(Err(NoAnswer::DidNotAnswer(Box::new(failed)))),
        Err(why) => broke(after_it_left(why)),
    }
}

/// There was no boundary to put the question inside, so it was not put —
/// written down, and then said.
///
/// The same entry a file verb leaves when the machine would not bound it,
/// `alo_record::Entry::not_bounded`, with the sentence the person was shown and
/// the machine's own account of its boundary; `carrying.rs` says why the
/// machine's refusal is written down at all. Nothing left and nothing was
/// shown.
pub(crate) fn nothing_was_bounded(
    machine: &mut Machine<'_>,
    why: NoBoundary,
    asked_by: &Grantee,
    now: SystemTime,
) -> NothingLeft {
    let a_thread_was_lost = why.a_thread_is_still_inside();
    let said = why.said(machine.strings());
    let entry = Entry::not_bounded(asked_by, said.text(), why.why(), now);
    let (why, the_record_broke) = match keep(machine, entry) {
        Ok(()) => (NoAnswer::NotBounded(why), false),
        Err(kept) => (nothing_left(kept), true),
    };
    NothingLeft {
        why,
        the_record_broke,
        a_thread_was_lost,
    }
}

/// Write one entry down.
fn keep(machine: &mut Machine<'_>, entry: Entry) -> Result<(), NotKept> {
    machine.kept().keep(entry)
}

/// An outcome where the record broke.
fn broke(why: NoAnswer) -> WhatBecameOfIt {
    WhatBecameOfIt {
        outcome: Err(why),
        the_record_broke: true,
        a_thread_was_lost: false,
    }
}

/// The record broke and nothing had left the machine.
fn nothing_left(why: NotKept) -> NoAnswer {
    NoAnswer::NotRecorded {
        why,
        after_it_left: false,
    }
}

/// The record broke after the question had already left.
fn after_it_left(why: NotKept) -> NoAnswer {
    NoAnswer::NotRecorded {
        why,
        after_it_left: true,
    }
}
