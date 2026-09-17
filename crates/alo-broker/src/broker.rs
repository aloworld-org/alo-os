//! The door's one decision, in the order it is made.
//!
//! The broker does not decide *whether* — the turn did, and a person approved
//! it. It decides only **that this is one of its verbs, exactly**, asked by the
//! agent's service, under an approval issued for exactly that and not yet
//! spent. In order, each step refusing before the next is reached:
//!
//! | | refused as |
//! |---|---|
//! | 1. the kernel names the caller as the agent's service | `not-the-agent-service` |
//! | 2. the line is a request | `not-a-request` |
//! | 3. its verb and argument are exactly one of the list's | `not-one-of-its-verbs` |
//! | 4. its token was issued for exactly that, under this broker's key | `not-approved` |
//! | 5. the token is fresh | `approval-lapsed` |
//! | 6. the token has not been spent — and it is spent here | `approval-spent` |
//! | 7. the verb is carried out, once | `not-carried` |
//!
//! # Written down before it is answered
//!
//! Every one of those answers is an entry in the record **before** it is
//! handed back, and a request that passed 1 to 6 is written down before it is
//! carried out — so nothing is ever carried out that the record does not
//! already say. A caller that was not the agent's service is written down
//! without its line having been looked at.
//!
//! A record that cannot be written stops the broker: that request is answered
//! `not-kept` and not carried out, and so is every request after it. That is
//! `alo-turn`'s rule — *a turn that could not write something down does nothing
//! else* — held by the component with more authority than a turn.

use std::time::SystemTime;

use alo_record::{AtTheBroker, Entry};

use crate::answer::Answer;
use crate::approving::ApprovingKey;
use crate::door::Door;
use crate::keeping::{Carrying, Recording};
use crate::request::{NotRead, Request};
use crate::spent::{NotSpendable, Spent};

/// The broker: a door, a key, what it has spent, and the two seams.
pub struct Broker<R, C> {
    /// Who may ask.
    door: Door,
    /// What a turn's approvals are proven with.
    key: ApprovingKey,
    /// The genuine approvals already used.
    spent: Spent,
    /// Where every answer is written down.
    recording: R,
    /// What carries a verb out.
    carrying: C,
    /// Whether a record could not be written, after which nothing is carried.
    stopped: bool,
}

impl<R: Recording, C: Carrying> Broker<R, C> {
    /// A broker with this door and key, writing to `recording` and handing
    /// verbs to `carrying`.
    #[must_use]
    pub fn new(door: Door, key: ApprovingKey, recording: R, carrying: C) -> Self {
        Self {
            door,
            key,
            spent: Spent::default(),
            recording,
            carrying,
            stopped: false,
        }
    }

    /// Whether a caller the kernel named is somebody whose line is read at all.
    #[must_use]
    pub const fn hears(&self, caller: Option<u32>) -> bool {
        self.door.hears(caller)
    }

    /// Answer one request from a caller the kernel named, heard at `now`.
    ///
    /// `line` is what arrived, without its newline, and is not looked at unless
    /// [`Broker::hears`] the caller.
    pub fn heard(&mut self, caller: Option<u32>, line: &[u8], now: SystemTime) -> Answer {
        if self.stopped {
            return Answer::NotKept;
        }
        if !self.door.hears(caller) {
            return self.refused(None, None, AtTheBroker::NotTheAgentService, now);
        }
        let request = match Request::read(line) {
            Ok(request) => request,
            Err(NotRead::NotARequest) => {
                return self.refused(None, None, AtTheBroker::NotARequest, now);
            }
            Err(NotRead::NotOneOfItsVerbs { asked_for }) => {
                return self.refused(Some(&asked_for), None, AtTheBroker::NotOneOfItsVerbs, now);
            }
        };
        let verb = *request.verb();
        let token = *request.token();
        let named = Some(verb.name());
        if !self.key.issued(&verb, &token) {
            return self.refused(named, None, AtTheBroker::NotApproved, now);
        }
        let approval = token.approval();
        match self.spent.spend(&token, now) {
            Ok(()) => {}
            Err(NotSpendable::Lapsed) => {
                return self.refused(named, Some(approval), AtTheBroker::ApprovalLapsed, now);
            }
            Err(NotSpendable::AlreadySpent) => {
                return self.refused(named, Some(approval), AtTheBroker::ApprovalSpent, now);
            }
        }
        if !self.kept(Entry::handed_on_by_the_broker(verb.name(), approval, now)) {
            return Answer::NotKept;
        }
        match self.carrying.carry(verb, approval) {
            Ok(()) => Answer::Carried,
            Err(_) => self.refused(named, Some(approval), AtTheBroker::NotCarried, now),
        }
    }

    /// Where this broker writes down what it answered.
    #[must_use]
    pub const fn recording(&self) -> &R {
        &self.recording
    }

    /// What this broker hands verbs to.
    #[must_use]
    pub const fn carrying(&self) -> &C {
        &self.carrying
    }

    /// Write a refusal down, and answer with it — or with `not-kept`.
    fn refused(
        &mut self,
        verb: Option<&str>,
        approval: Option<u64>,
        why: AtTheBroker,
        now: SystemTime,
    ) -> Answer {
        if self.kept(Entry::refused_by_the_broker(verb, approval, why, now)) {
            Answer::Refused(why)
        } else {
            Answer::NotKept
        }
    }

    /// Write an entry down; a failure stops this broker for good.
    fn kept(&mut self, entry: Entry) -> bool {
        if self.recording.keep(entry).is_err() {
            self.stopped = true;
        }
        !self.stopped
    }
}
