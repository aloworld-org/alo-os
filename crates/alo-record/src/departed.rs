//! The only door from an egress into the record.
//!
//! Law 1 has two halves: *every network egress an agent causes is visible at
//! the moment it happens **and afterwards in a record***. `alo-egress` carries
//! the first — nothing is permitted to leave without appearing on the
//! indicator, because [`Indicator::beginning`](alo_egress::Indicator::beginning)
//! is the only maker of a [`Departing`] and it shows the egress before it hands
//! one back. This file carries the second, and it carries it the same way:
//! **the only way to a [`Happened::Left`] entry is a [`Departing`]**, so an
//! egress the indicator never showed is not an entry that can be written.
//!
//! It is a file of its own rather than two more constructors in
//! [`crate::entry`] because it changes for a different reason. `entry.rs`
//! changes when the capability journey does; this changes when what leaves the
//! machine does, and the guarantee above is worth being able to read without
//! reading anything else — law 4.
//!
//! # One departure, one entry
//!
//! An answer from a provider is two facts at once: *where the answer came from*
//! (ADR 0008) and *something left this machine* (law 1). Keeping them as two
//! entries would make law 1's one query count one departure twice, so there is
//! one entry and it is the departure. [`crate::happened`] has the reasoning and
//! what it costs, which is that "where did that answer come from" is read off a
//! destination rather than off an inference source — the same three kinds, the
//! same names, one of them written down.
//!
//! # And what was refused
//!
//! [`Entry::held_back`] is here for the reason every refusal is recorded: a
//! record that kept only what left could not answer what an organisation's
//! egress policy actually stopped, which is the question somebody asks having
//! just set one. It takes a [`NotPermitted`], which only the indicator
//! produces, so a refusal cannot be recorded that the policy did not make —
//! and it is not egress, because nothing left.
//!
//! # And what nobody caused
//!
//! Law 1's sentence is about egress an *agent* causes, and `docs/features.md`
//! makes a second promise beside it — ★ *no telemetry* — about egress with none.
//! [`Entry::left_on_its_own`] is that promise's second half: the machine's own
//! errands are shown while they happen (`alo-egress`' indicator) **and
//! afterwards in a record**, which is the only part of law 1 a person can check
//! at the end of a week rather than in the second it happened.
//!
//! It is the same guarantee by the same means. An [`Underway`] is made only by
//! [`Indicator::beginning_on_its_own`](alo_egress::Indicator::beginning_on_its_own),
//! which shows the errand before it hands one back, so **an errand the
//! indicator never showed is not an entry that can be written** — and there is
//! no way to write one from a bare [`alo_egress::Errand`] and a
//! [`Destination`](alo_egress::Destination), which is what would let a record
//! claim a departure that never appeared to anybody.
//!
//! There is no refusal beside it, and the absence is deliberate: an errand is
//! decided by being on the closed list and by nothing else (`alo-egress`'
//! `errand.rs` says why the organisation's egress policy is not asked), so
//! there is no refusal for a `held_back` twin to record.

use std::time::SystemTime;

use alo_egress::{Departing, NotPermitted, Underway};
use alo_strings::Strings;

use crate::entry::Entry;
use crate::happened::Happened;
use crate::line::Line;

impl Entry {
    /// Something left this machine.
    ///
    /// The moment comes from the departure rather than from the caller, as
    /// [`Entry::ran`] takes its moment from the authorisation: it is the moment
    /// the policy was asked, which is the moment the thing was allowed to
    /// happen. What a person saw on the indicator and what the record says
    /// about it therefore cannot disagree about when.
    ///
    /// The departure is borrowed rather than consumed, because the caller still
    /// has to end it on the indicator when the connection closes.
    ///
    /// ```
    /// use alo_capability::Grantee;
    /// use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
    /// use alo_record::{Asking, Entry, Only, Record};
    /// use std::time::{Duration, SystemTime};
    ///
    /// # fn main() {
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let mut indicator = Indicator::default();
    /// let mut record = Record::default();
    ///
    /// let departing = indicator
    ///     .beginning(
    ///         &EgressPolicy::Anywhere,
    ///         Leaving::because(
    ///             &Grantee::named("@files"),
    ///             Why::Fetching,
    ///             Destination::at("alo.example").expect("a host that can be shown"),
    ///         ),
    ///         now,
    ///     )
    ///     .expect("nothing forbids it");
    /// record.keep(Entry::left(&departing));
    /// indicator.ended(departing);
    ///
    /// // The indicator is quiet again, and what left is still answerable.
    /// assert!(indicator.is_quiet());
    /// let asking = Asking::anything().only(Only::Egress);
    /// assert_eq!(record.answering(&asking).count(), 1);
    ///
    /// // And a departure comes from one place, which is the guarantee:
    /// let _ = Indicator::beginning;
    /// # }
    /// ```
    ///
    /// There is no other way to make one, so there is no way to record an
    /// egress that was never shown:
    ///
    /// ```compile_fail
    /// // `Departing::new` is the indicator's alone.
    /// let _ = alo_egress::departing::Departing::new;
    /// ```
    #[must_use]
    pub fn left(departing: &Departing) -> Self {
        Self::new(
            departing.at(),
            Happened::Left {
                agent: Line::of(departing.agent().as_str()),
                destination: departing.destination().clone(),
                why: departing.why(),
            },
        )
    }

    /// The egress policy refused to let something leave, so nothing did.
    ///
    /// The refusal comes from the policy and carries what it refused, so the
    /// record can say what the agent tried rather than only that something was
    /// stopped. The moment is passed in, because a refusal is not an authority
    /// and does not carry one.
    ///
    /// **The strings come in, and the rendering is made here** — as
    /// [`Entry::refused`] does since item 9e, and for the same reason. A
    /// refusal reaches this crate as the value the policy made, so what a
    /// person was shown and what the record keeps are one rendering of one
    /// value rather than two accounts of one moment. It goes through [`Line`]
    /// like every other refusal, because it names a destination that came from
    /// a verb's argument.
    ///
    /// A refusal cannot be made up either — [`NotPermitted`] has no public
    /// constructor:
    ///
    /// ```compile_fail
    /// // `NotPermitted::new` is the egress policy's alone.
    /// let _ = alo_egress::refusing::NotPermitted::new;
    /// ```
    #[must_use]
    pub fn held_back(refused: &NotPermitted, strings: &Strings, at: SystemTime) -> Self {
        let leaving = refused.leaving();
        Self::new(
            at,
            Happened::HeldBack {
                agent: Line::of(leaving.agent().as_str()),
                destination: leaving.destination().clone(),
                why: leaving.why(),
                refused: Line::of(refused.said(strings).text()),
            },
        )
    }

    /// alo OS reached the network on its own (★ *no telemetry*).
    ///
    /// The moment comes from the errand rather than from the caller, as
    /// [`Entry::left`] takes its moment from the departure: it is the moment
    /// the errand went on the indicator, so what a person saw and what the
    /// record says about it cannot disagree about when.
    ///
    /// The errand is borrowed rather than consumed, because the caller still
    /// has to end it on the indicator when the connection closes.
    ///
    /// **Nothing is passed in about whose it was, because there is nothing to
    /// pass.** [`crate::happened`] has the whole of why an invented identity for
    /// the system would be a lie in the one field a security review reads
    /// first; the short version is that nobody granted alo OS anything.
    ///
    /// ```
    /// use alo_egress::{Destination, Errand, Indicator, OnItsOwn};
    /// use alo_record::{Asking, Entry, Only, Record};
    /// use std::time::{Duration, SystemTime};
    ///
    /// # fn main() {
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let mut indicator = Indicator::default();
    /// let mut record = Record::default();
    ///
    /// let underway = indicator.beginning_on_its_own(
    ///     OnItsOwn::for_(
    ///         Errand::FetchingAModel,
    ///         Destination::at("models.alo.example").expect("a host that can be shown"),
    ///     ),
    ///     now,
    /// );
    /// record.keep(Entry::left_on_its_own(&underway));
    /// indicator.ended_on_its_own(underway);
    ///
    /// // It left, so law 1's question finds it — and it was nobody's doing,
    /// // so the question about that finds it too.
    /// assert!(indicator.is_quiet());
    /// assert_eq!(record.answering(&Asking::anything().only(Only::Egress)).count(), 1);
    /// assert_eq!(record.answering(&Asking::anything().only(Only::OnItsOwn)).count(), 1);
    ///
    /// // And no agent's day includes it, because there is no agent on it.
    /// assert_eq!(record.answering(&Asking::anything().by("alo OS")).count(), 0);
    /// # }
    /// ```
    ///
    /// There is no other way to make one, so there is no way to record an
    /// errand that was never shown:
    ///
    /// ```compile_fail
    /// // `Underway::new` is the indicator's alone.
    /// let _ = alo_egress::underway::Underway::new;
    /// ```
    #[must_use]
    pub fn left_on_its_own(underway: &Underway) -> Self {
        Self::new(
            underway.at(),
            Happened::LeftOnItsOwn {
                errand: underway.errand(),
                destination: underway.destination().clone(),
            },
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::explain::{Asking, Only};
    use crate::record::Record;
    use crate::test_calls::{
        asking_alo, departing, fetching_a_model, files, hour, mail, noon, not_permitted,
        the_catalogue, to_alo, to_the_studio, underway,
    };
    use crate::testing::in_english;
    use alo_egress::{Destination, EgressPolicy, Errand, Leaving, Why};

    /// **Law 1's second half.** What left is kept with everything a person or a
    /// security review needs afterwards — who, where to, why, and when — worked
    /// out once at the moment the policy permitted it.
    #[test]
    fn what_left_is_recorded_with_who_where_why_and_when() {
        let entry = Entry::left(&departing(asking_alo(), noon()));
        assert_eq!(entry.at(), noon());
        assert!(entry.agent().is_some_and(|agent| agent.is("@mail")));
        assert_eq!(entry.happened().destination(), Some(&to_alo()));
        assert_eq!(entry.happened().why_it_was_leaving(), Some(Why::Asking));
        assert!(entry.happened().caused_egress());
        assert!(!entry.happened().was_stopped());
    }

    /// **The moment is the departure's**, not the recorder's, so the indicator
    /// and the record cannot disagree about when something left.
    #[test]
    fn the_moment_comes_from_the_departure_rather_than_from_whoever_wrote_it_down() {
        let departing = departing(asking_alo(), noon() + hour());
        assert_eq!(Entry::left(&departing).at(), departing.at());
        assert_eq!(Entry::left(&departing).at(), noon() + hour());
    }

    /// **One departure is one entry**, so law 1's question answers with the
    /// number of things that actually left. A question answered somewhere else
    /// is that departure and is not also an answer entry beside it.
    #[test]
    fn a_question_answered_somewhere_else_is_one_entry_and_not_two() {
        let mut record = Record::default();
        record.keep(Entry::left(&departing(asking_alo(), noon())));
        record.keep(Entry::answered_here(&mail(), noon() + hour()));

        let egress = Asking::anything().only(Only::Egress);
        assert_eq!(record.answering(&egress).count(), 1);
        assert_eq!(
            record.len(),
            2,
            "the local answer is kept, and is not egress"
        );
        assert!(
            record
                .answering(&egress)
                .all(|entry| entry.happened().why_it_was_leaving() == Some(Why::Asking))
        );
    }

    /// **Law 1: the corridor is egress too.** A paired machine in the next room
    /// is a departure with an entry of its own, and the record says where
    /// rather than staying silent because it was nearby.
    #[test]
    fn an_answer_from_the_next_room_is_recorded_as_a_departure() {
        let leaving = Leaving::because(&mail(), Why::Asking, to_the_studio());
        let entry = Entry::left(&departing(leaving, noon()));
        assert!(entry.happened().caused_egress());
        assert_eq!(entry.happened().destination(), Some(&to_the_studio()));
    }

    /// **The refusal path.** An egress the policy refused is kept as a refusal,
    /// with what was refused and why — and it is not counted as something that
    /// left, because nothing did.
    #[test]
    fn an_egress_the_policy_refused_is_recorded_and_is_not_counted_as_egress() {
        let refused = not_permitted(&EgressPolicy::NothingLeaves, asking_alo());
        let entry = Entry::held_back(&refused, &in_english(), noon());

        assert!(entry.happened().was_stopped());
        assert!(!entry.happened().caused_egress());
        assert!(entry.agent().is_some_and(|agent| agent.is("@mail")));
        assert_eq!(entry.happened().destination(), Some(&to_alo()));
        assert!(
            entry
                .happened()
                .why_stopped()
                .is_some_and(|why| why.as_str().contains("nothing leave")),
            "{entry:?}"
        );

        let mut record = Record::default();
        record.keep(entry);
        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Egress))
                .count(),
            0
        );
        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Refusals))
                .count(),
            1
        );
    }

    /// The policy that keeps everything in the building refuses a provider and
    /// permits the machine in the next room, and the record keeps both — which
    /// is what makes "what did our policy actually stop" a question worth
    /// asking.
    #[test]
    fn what_a_policy_stopped_and_what_it_let_through_are_both_kept() {
        let mut record = Record::default();
        record.keep(Entry::held_back(
            &not_permitted(&EgressPolicy::InTheBuilding, asking_alo()),
            &in_english(),
            noon(),
        ));
        record.keep(Entry::left(&departing(
            Leaving::because(&mail(), Why::Asking, to_the_studio()),
            noon() + hour(),
        )));

        assert_eq!(record.len(), 2);
        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Egress))
                .count(),
            1
        );
        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Refusals))
                .count(),
            1
        );
    }

    /// **A destination cannot rewrite the record it appears in.** The host came
    /// from a verb's argument, so it goes through the same refusals everywhere
    /// — `alo-egress` will not make a destination of it at all, which is why
    /// there is nothing for this file to strip.
    #[test]
    fn an_address_that_could_rewrite_a_line_never_reaches_the_record() {
        assert!(Destination::at("alo.example\u{1b}[2K and nothing left").is_err());

        // What does reach the record is the policy's own words, and they go
        // through `Line` like every other refusal.
        let refused = not_permitted(
            &EgressPolicy::NothingLeaves,
            Leaving::because(
                &files(),
                Why::Sending,
                Destination::at("alo.example").unwrap(),
            ),
        );
        let written =
            serde_json::to_string(&Entry::held_back(&refused, &in_english(), noon())).unwrap();
        assert!(!written.contains('\u{1b}'), "{written}");
    }

    /// **Law 1's second half, for the egress its first sentence does not
    /// cover.** What alo OS did with nobody having asked is kept with what it
    /// was, where it reached and when — and with nobody, because there was
    /// nobody.
    #[test]
    fn what_the_machine_did_on_its_own_is_recorded_with_what_where_and_when() {
        let entry = Entry::left_on_its_own(&fetching_a_model(noon()));
        assert_eq!(entry.at(), noon());
        assert_eq!(entry.agent(), None);
        assert_eq!(entry.happened().errand(), Some(Errand::FetchingAModel));
        assert_eq!(entry.happened().destination(), Some(&the_catalogue()));
        assert!(entry.happened().caused_egress());
        assert!(entry.happened().on_its_own());
        assert!(!entry.happened().was_stopped());
    }

    /// **The moment is the errand's**, as it is the departure's, so what a
    /// person saw on the indicator and what the record says cannot disagree
    /// about when the machine reached the network.
    #[test]
    fn the_moment_comes_from_the_errand_rather_than_from_whoever_wrote_it_down() {
        let underway = fetching_a_model(noon() + hour());
        assert_eq!(Entry::left_on_its_own(&underway).at(), underway.at());
        assert_eq!(Entry::left_on_its_own(&underway).at(), noon() + hour());
    }

    /// **Every reason there is can be recorded, and there are only three.** A
    /// reason that could happen and could not be written down would be the hole
    /// in the *no telemetry* promise that the closed list exists to prevent.
    #[test]
    fn every_reason_alo_os_reaches_the_network_can_be_written_down() {
        let mut record = Record::default();
        for (index, errand) in Errand::EVERY.into_iter().enumerate() {
            let at = noon() + hour() * u32::try_from(index).unwrap();
            record.keep(Entry::left_on_its_own(&underway(
                errand,
                the_catalogue(),
                at,
            )));
        }
        assert_eq!(record.len(), Errand::EVERY.len());
        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::OnItsOwn))
                .filter_map(|entry| entry.happened().errand())
                .collect::<Vec<_>>(),
            Errand::EVERY.to_vec()
        );
    }

    /// **What left and who caused it are two questions.** An agent's departure
    /// and the machine's errand are both things that left, so law 1's question
    /// answers with both — and *which of them nobody asked for* is the second
    /// question rather than a different list.
    #[test]
    fn what_left_this_machine_counts_the_errands_too() {
        let mut record = Record::default();
        record.keep(Entry::left(&departing(asking_alo(), noon())));
        record.keep(Entry::left_on_its_own(&fetching_a_model(noon() + hour())));

        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Egress))
                .count(),
            2
        );
        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::OnItsOwn))
                .count(),
            1
        );
        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Refusals))
                .count(),
            0,
            "nothing was refused, and an errand is never refused"
        );

        // **No agent's day includes an errand**, whatever it is called. This is
        // the decision as a query: an identity for the system would have made
        // one of these answer 1.
        for name in ["alo OS", "alo", "@alo", "system", "@mail"] {
            let asking = Asking::anything().by(name);
            let mine = record.answering(&asking);
            assert_eq!(
                mine.filter(|entry| entry.happened().on_its_own()).count(),
                0,
                "{name}"
            );
        }
        assert_eq!(record.answering(&Asking::anything().by("@mail")).count(), 1);
    }

    /// An egress entry outlives the session that wrote it, so it has to survive
    /// being written down and read back — still saying that something left, and
    /// still saying where.
    #[test]
    fn an_egress_entry_survives_being_written_down_and_read_back() {
        for entry in [
            Entry::left(&departing(asking_alo(), noon())),
            Entry::held_back(
                &not_permitted(&EgressPolicy::NothingLeaves, asking_alo()),
                &in_english(),
                noon(),
            ),
            Entry::answered_here(&mail(), noon()),
            Entry::left_on_its_own(&fetching_a_model(noon())),
        ] {
            let written = serde_json::to_string(&entry).unwrap();
            let read = serde_json::from_str::<Entry>(&written).ok();
            assert_eq!(read.as_ref(), Some(&entry), "{written}");
            assert_eq!(
                read.map(|read| read.happened().caused_egress()),
                Some(entry.happened().caused_egress()),
                "{written}"
            );
        }
    }
}
