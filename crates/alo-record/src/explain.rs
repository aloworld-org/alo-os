//! What can be asked of the record.
//!
//! ADR 0001 §7: *"explain what it did" is a query, not a log to grep.* This
//! file is the difference between those two things. The questions a person and
//! a security review actually ask —
//!
//! - what did `@files` do this afternoon?
//! - what was it stopped from doing?
//! - what ran under the grant I have just revoked?
//! - what did that one approval cause?
//! - what left this machine today?
//! - what did the machine do with nobody having asked it to?
//!
//! — are asked here in the record's own terms, and answered from the fields the
//! entries carry. None of them is a search for text. A record answered by
//! matching strings would answer differently the day somebody rewords a
//! refusal, and would answer *whatever an argument was named after* rather than
//! what actually happened.
//!
//! An [`Asking`] is criteria and nothing else: it holds no reference to a
//! record, so the same question can be put to the record in memory, to one read
//! back off a disk, and to one somebody has been handed to look at.
//!
//! **There is no explaining in words here.** The explanation a person reads is
//! already in the entry — it is the sentence they approved, generated from the
//! validated arguments before anything ran. Composing a second description
//! afterwards would be writing prose about evidence, in one language, and the
//! prose would be what people quoted.

use std::time::SystemTime;

use crate::entry::Entry;

/// Which kinds of entry a question is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Only {
    /// What ran.
    Executions,
    /// What was stopped — the well-formed calls that were refused, the attempts
    /// that never became calls at all, and the egress the policy held back.
    Refusals,
    /// What left this machine (law 1).
    ///
    /// One entry per departure, because a departure is one entry: a question
    /// answered somewhere else *is* the egress it caused rather than a second
    /// thing beside it, and an egress the policy refused never left at all.
    ///
    /// **Everything that left, not only what an agent caused**, which is item
    /// 16's *one indicator, not two* asked afterwards rather than watched at
    /// the time: a person who wants to know what left their machine is not
    /// asking a question about authorship. [`Only::OnItsOwn`] is the half of
    /// this with nobody behind it.
    Egress,
    /// What alo OS did with nobody having asked it to (★ *no telemetry*).
    ///
    /// The three reasons on [`alo_egress::Errand`] and nothing else, because
    /// there is nothing else — this is the question somebody puts to the record
    /// having just read that promise, and a promise nobody can check afterwards
    /// is a sentence.
    ///
    /// It narrows [`Only::Egress`] rather than sitting beside it: every errand
    /// left the machine. What an agent caused is the rest, and is asked for by
    /// naming the agent.
    OnItsOwn,
}

/// A question put to the record.
///
/// Every part is optional and the parts narrow together: an [`Asking`] with
/// nothing set matches everything, which is the honest default for a question
/// nobody has narrowed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Asking {
    /// Whose entries, matched exactly.
    agent: Option<String>,
    /// From this moment, included.
    from: Option<SystemTime>,
    /// Until this moment, not included — the same side of the boundary as
    /// [`alo_capability::Grant::is_active_at`], so two spans that meet do not
    /// both claim the moment between them.
    until: Option<SystemTime>,
    /// Which kinds of entry.
    only: Option<Only>,
    /// Under which grant.
    under_grant: Option<u64>,
    /// From which approval.
    from_approval: Option<u64>,
}

impl Asking {
    /// Everything the record holds.
    #[must_use]
    pub fn anything() -> Self {
        Self::default()
    }

    /// Only this agent's entries, matched exactly.
    ///
    /// Exactly, like every identity in the capability model: a question about
    /// `@files` that also answered for `@Files` would be answering about an
    /// agent nobody asked about.
    ///
    /// **An errand answers no name at all.** What alo OS did on its own is
    /// under nobody's authority, so no spelling of any agent finds it — which
    /// is what stops a question about one agent's day from quietly including
    /// the machine's, and is why the record has no identity for the system to
    /// be asked about by.
    #[must_use]
    pub fn by(mut self, agent: &str) -> Self {
        self.agent = Some(agent.trim().to_owned());
        self
    }

    /// Only what happened in this span — `from` included, `until` not.
    #[must_use]
    pub fn between(mut self, from: SystemTime, until: SystemTime) -> Self {
        self.from = Some(from);
        self.until = Some(until);
        self
    }

    /// Only entries of this kind.
    #[must_use]
    pub fn only(mut self, only: Only) -> Self {
        self.only = Some(only);
        self
    }

    /// Only what ran against this grant.
    ///
    /// The question somebody asks having just revoked one: *what had it already
    /// been used for?*
    #[must_use]
    pub fn under_grant(mut self, grant: u64) -> Self {
        self.under_grant = Some(grant);
        self
    }

    /// Only what ran from this approval.
    ///
    /// One approval causes exactly one execution, so this answers with one
    /// entry or with none — and *none* is itself worth being able to ask about.
    #[must_use]
    pub fn from_approval(mut self, approval: u64) -> Self {
        self.from_approval = Some(approval);
        self
    }

    /// Whether one entry is part of the answer.
    ///
    /// Every part that was set has to match. Adding one narrows the answer and
    /// never widens it, which is what lets a question be built up without
    /// having to be read backwards.
    #[must_use]
    pub fn matches(&self, entry: &Entry) -> bool {
        self.agent
            .as_ref()
            .is_none_or(|agent| entry.agent().is_some_and(|whose| whose.is(agent)))
            && self.from.is_none_or(|from| entry.at() >= from)
            && self.until.is_none_or(|until| entry.at() < until)
            && self.only.is_none_or(|only| is_only(entry, only))
            && self
                .under_grant
                .is_none_or(|grant| entry.happened().against().contains(&grant))
            && self
                .from_approval
                .is_none_or(|approval| entry.happened().from_approval() == Some(approval))
    }
}

/// Whether an entry is of the kind being asked about.
fn is_only(entry: &Entry, only: Only) -> bool {
    match only {
        Only::Executions => entry.happened().ran(),
        Only::Refusals => entry.happened().was_stopped(),
        Only::Egress => entry.happened().caused_egress(),
        Only::OnItsOwn => entry.happened().on_its_own(),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::record::Record;
    use crate::test_calls::{
        archiving_march, asking_alo, departing, fetching_a_model, files, granting, granting_both,
        hour, listing_invoices, noon, not_permitted, proposing,
    };
    use crate::testing::in_english;
    use alo_capability::{Approvals, Authorised, Grants};
    use alo_egress::EgressPolicy;

    /// An afternoon with something of every kind in it, and the numbers the
    /// tests below ask about.
    struct Afternoon {
        /// Everything that happened.
        record: Record,
        /// The grant the change ran against.
        grant: u64,
        /// The approval the change ran from.
        approval: u64,
    }

    fn afternoon() -> Afternoon {
        let mut record = Record::default();
        let grants = granting_both();
        let grant = grants.active_at(noon()).next().unwrap().id.as_u64();

        // A read, at noon.
        let read = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();
        record.keep(Entry::ran(&read, &in_english()));

        // A change, approved and run, an hour later.
        let mut approvals = Approvals::default();
        let id = approvals.propose(proposing(&archiving_march(), &grants));
        let approved = approvals.approve(id, noon() + hour()).unwrap();
        let running = approved.redeem(&grants, noon() + hour()).unwrap();
        record.keep(Entry::ran(&running, &in_english()));

        // A refusal, and an attempt that never became a call, two hours later.
        let refused = Authorised::read(
            &listing_invoices(),
            &files(),
            &granting(&["/home/anna/Taxes"]),
            noon() + hour() * 2,
        )
        .unwrap_err();
        record.keep(Entry::refused(
            &refused,
            &files(),
            &in_english(),
            noon() + hour() * 2,
        ));
        record.keep(Entry::turned_away(
            "delete_everything",
            "there is no verb called delete_everything",
            &files(),
            noon() + hour() * 2,
        ));

        // And another agent's question, answered somewhere else entirely —
        // which is to say, a departure.
        record.keep(Entry::left(&departing(asking_alo(), noon() + hour() * 2)));

        // And the machine itself, fetching a model with nobody having asked
        // it to. An afternoon with no errand in it would be an afternoon these
        // questions were never put an errand about.
        record.keep(Entry::left_on_its_own(&fetching_a_model(
            noon() + hour() * 3,
        )));
        Afternoon {
            record,
            grant,
            approval: id.as_u64(),
        }
    }

    fn how_many(record: &Record, asking: &Asking) -> usize {
        record.answering(asking).count()
    }

    /// A question nobody has narrowed is answered with everything, rather than
    /// with nothing.
    #[test]
    fn a_question_with_nothing_set_asks_about_everything() {
        let afternoon = afternoon();
        assert_eq!(
            how_many(&afternoon.record, &Asking::anything()),
            afternoon.record.len()
        );
    }

    /// *What did `@files` do this afternoon?* — one agent, one span, and
    /// nothing about another agent leaking into the answer.
    #[test]
    fn what_one_agent_did_in_one_span_is_one_question() {
        let afternoon = afternoon();
        let asking = Asking::anything()
            .by("@files")
            .between(noon(), noon() + hour() * 2);
        assert_eq!(how_many(&afternoon.record, &asking), 2);

        // The whole afternoon, and now the other agent's question as well.
        assert_eq!(
            how_many(&afternoon.record, &Asking::anything().by("@files")),
            4
        );
        assert_eq!(
            how_many(&afternoon.record, &Asking::anything().by("@mail")),
            1
        );
    }

    /// Identities are matched exactly here as everywhere else: a question about
    /// one agent is never answered about another that is spelled nearly the
    /// same.
    #[test]
    fn an_agent_is_matched_exactly() {
        let afternoon = afternoon();
        assert_eq!(
            how_many(&afternoon.record, &Asking::anything().by("@Files")),
            0
        );
        assert_eq!(
            how_many(&afternoon.record, &Asking::anything().by("@file")),
            0
        );
    }

    /// **What was it stopped from doing?** — the question a security review
    /// asks, answered as a query rather than by reading every entry.
    #[test]
    fn what_was_refused_is_a_question_the_record_answers() {
        let afternoon = afternoon();
        let refusals = Asking::anything().only(Only::Refusals);
        assert_eq!(how_many(&afternoon.record, &refusals), 2);

        let executions = Asking::anything().only(Only::Executions);
        assert_eq!(how_many(&afternoon.record, &executions), 2);

        // And the two do not overlap: nothing both ran and was stopped.
        assert!(
            afternoon
                .record
                .answering(&refusals)
                .all(|entry| !entry.happened().ran())
        );
    }

    /// **What ran under the grant I have just revoked?** — the question that
    /// makes revoking a grant an act somebody can reason about afterwards.
    #[test]
    fn what_ran_under_one_grant_is_a_question_the_record_answers() {
        let afternoon = afternoon();
        let asking = Asking::anything().under_grant(afternoon.grant);
        assert_eq!(how_many(&afternoon.record, &asking), 2);

        // A grant nothing ran under answers with nothing, rather than with
        // everything.
        assert_eq!(
            how_many(&afternoon.record, &Asking::anything().under_grant(99)),
            0
        );
    }

    /// One approval causes exactly one execution, and the record can be asked
    /// which one it was — or that it caused none.
    #[test]
    fn what_one_approval_caused_is_one_entry_or_none() {
        let afternoon = afternoon();
        let asking = Asking::anything().from_approval(afternoon.approval);
        assert_eq!(how_many(&afternoon.record, &asking), 1);
        assert!(
            afternoon
                .record
                .answering(&asking)
                .all(|entry| entry.happened().ran())
        );
        assert_eq!(
            how_many(&afternoon.record, &Asking::anything().from_approval(99)),
            0
        );
    }

    /// **Law 1, as a query.** What left this machine is answerable afterwards
    /// and not only at the moment the indicator fired — and it counts what alo
    /// OS did itself, because that left too.
    #[test]
    fn what_left_the_machine_is_a_question_the_record_answers() {
        let afternoon = afternoon();
        let asking = Asking::anything().only(Only::Egress);
        assert_eq!(how_many(&afternoon.record, &asking), 2);
        assert_eq!(
            how_many(&afternoon.record, &asking.clone().by("@mail")),
            1,
            "one of the two was an agent's"
        );

        // A working day answered on this machine leaves nothing to find, which
        // is the measurement law 1 promises rather than the promise itself.
        let mut local = Record::default();
        for hours in 0..8 {
            local.keep(Entry::answered_here(&files(), noon() + hour() * hours));
        }
        assert_eq!(local.len(), 8);
        assert_eq!(how_many(&local, &asking), 0);
    }

    /// **★ No telemetry, as a question rather than as a promise.** Somebody who
    /// has just read *alo OS reaches the network for these reasons and no
    /// others* can put that to the record and be answered with what it actually
    /// did, which is the only half of a promise anybody can check.
    #[test]
    fn what_the_machine_did_on_its_own_is_a_question_the_record_answers() {
        let afternoon = afternoon();
        let asking = Asking::anything().only(Only::OnItsOwn);
        assert_eq!(how_many(&afternoon.record, &asking), 1);
        assert!(
            afternoon
                .record
                .answering(&asking)
                .all(|entry| entry.happened().errand().is_some() && entry.agent().is_none())
        );

        // A machine that has done nothing on its own answers with nothing,
        // rather than with everything that left.
        let mut agents_only = Record::default();
        agents_only.keep(Entry::left(&departing(asking_alo(), noon())));
        assert_eq!(how_many(&agents_only, &asking), 0);
        assert_eq!(
            how_many(&agents_only, &Asking::anything().only(Only::Egress)),
            1
        );
    }

    /// **An errand belongs to no agent's day.** This is the decision item 16a
    /// made, asked as the query that would have exposed the other answer: an
    /// identity for the system — however carefully it was chosen not to be a
    /// `Grantee` — would be a name somebody's question could match.
    #[test]
    fn no_agent_however_it_is_spelled_answers_for_what_the_machine_did_itself() {
        let afternoon = afternoon();
        for name in ["alo OS", "alo", "@alo", "alo-os", "system", "@files"] {
            let asking = Asking::anything().by(name).only(Only::OnItsOwn);
            assert_eq!(how_many(&afternoon.record, &asking), 0, "{name}");
        }
    }

    /// **An egress the policy refused is a refusal, not a departure.** A record
    /// that counted what was stopped as something that left would answer law
    /// 1's question with a number larger than the truth, which is the way of
    /// being wrong that teaches people to stop reading it.
    #[test]
    fn what_was_held_back_answers_the_refusals_and_never_the_egress() {
        let mut record = Record::default();
        record.keep(Entry::held_back(
            &not_permitted(&EgressPolicy::NothingLeaves, asking_alo()),
            &in_english(),
            noon(),
        ));
        assert_eq!(how_many(&record, &Asking::anything().only(Only::Egress)), 0);
        assert_eq!(
            how_many(&record, &Asking::anything().only(Only::Refusals)),
            1
        );
        assert_eq!(how_many(&record, &Asking::anything().by("@mail")), 1);
    }

    /// A span includes where it starts and stops before where it ends, so two
    /// spans that meet do not both claim the moment between them.
    #[test]
    fn a_span_includes_its_start_and_stops_before_its_end() {
        let afternoon = afternoon();
        let morning = Asking::anything().between(noon(), noon() + hour());
        let later = Asking::anything().between(noon() + hour(), noon() + hour() * 2);
        assert_eq!(how_many(&afternoon.record, &morning), 1);
        assert_eq!(how_many(&afternoon.record, &later), 1);
        assert_eq!(
            how_many(&afternoon.record, &morning) + how_many(&afternoon.record, &later),
            2
        );
    }

    /// Narrowing a question never widens the answer. This is what lets one be
    /// built up a part at a time without having to be read backwards.
    #[test]
    fn every_part_of_a_question_narrows_it() {
        let afternoon = afternoon();
        let mut asking = Asking::anything();
        let mut answers = how_many(&afternoon.record, &asking);
        for narrowed in [
            Asking::anything().by("@files"),
            Asking::anything().by("@files").only(Only::Refusals),
            Asking::anything()
                .by("@files")
                .only(Only::Refusals)
                .between(noon(), noon() + hour()),
        ] {
            let now = how_many(&afternoon.record, &narrowed);
            assert!(now <= answers, "{asking:?} then {narrowed:?}");
            asking = narrowed;
            answers = now;
        }
        assert_eq!(answers, 0);
    }

    /// A question is asked of the record in its own terms, so it can be put to
    /// one read back off a disk exactly as it was put to the one in memory.
    #[test]
    fn a_question_can_be_asked_of_a_record_read_back_off_a_disk() {
        let afternoon = afternoon();
        let written = serde_json::to_string(&afternoon.record).unwrap();
        let read = serde_json::from_str::<Record>(&written).unwrap();
        let asking = Asking::anything().by("@files").only(Only::Refusals);
        assert_eq!(
            how_many(&read, &asking),
            how_many(&afternoon.record, &asking)
        );

        // And the record it was read into is not an authority: nothing in it
        // permits anything.
        assert!(!archiving_march().permitted_by(&Grants::default(), &files(), noon()));
    }
}
