//! What is leaving this machine right now.
//!
//! Law 1: *every network egress an agent causes is visible at the moment it
//! happens.* On a machine sold on sovereignty the indicator is a feature and not
//! a diagnostic, so this is a list a person can read — who, where to and why —
//! rather than a light that is on or off.
//!
//! **Nothing gets permission without appearing here.** [`Indicator::beginning`]
//! is the only way to obtain a [`Departing`], and it puts the egress on the list
//! before it hands one back. A caller cannot be permitted and unshown, because
//! there is no order of operations in which that state exists.
//!
//! **A quiet indicator is the measurement.** `CLAUDE.md` promises that a working
//! day with a local model produces zero inference egress, and
//! [`Indicator::is_quiet`] is what that looks like from inside: a question
//! answered on this machine never becomes a [`crate::Leaving`] at all, so it has
//! nothing to put here. The honest measurement is still taken at the network
//! boundary — this says what the machine *believes*, and the two agreeing is the
//! claim.
//!
//! **Nothing here reads the clock**, as everywhere else in this repository: the
//! moment is passed in, so what the indicator shows and what a record says
//! cannot disagree about when something happened.

use std::time::SystemTime;

use alo_strings::{Said, Strings};
use serde::Serialize;

use crate::departing::Departing;
use crate::leaving::Leaving;
use crate::policy::EgressPolicy;
use crate::refusing::NotPermitted;

/// Which line on the indicator an egress is.
///
/// Unique for the life of the indicator. Numbers are never reused, so a line
/// ended late — a connection that finished while the shell was drawing the list
/// from a moment ago — cannot take away one that began since.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ShownId(u64);

impl ShownId {
    /// The number behind the handle, for showing and storing.
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// One line the indicator is showing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Shown {
    /// Which line this is.
    id: ShownId,
    /// When it began.
    at: SystemTime,
    /// What is leaving.
    leaving: Leaving,
}

impl Shown {
    /// Which line this is.
    #[must_use]
    pub fn id(&self) -> ShownId {
        self.id
    }

    /// When it began.
    #[must_use]
    pub fn at(&self) -> SystemTime {
        self.at
    }

    /// What is leaving.
    #[must_use]
    pub fn leaving(&self) -> &Leaving {
        &self.leaving
    }

    /// The line a person reads, in the language they read it in.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        self.leaving.said(strings)
    }
}

/// Everything leaving this machine at this moment.
///
/// Held by whatever draws the shell. It is a plain list rather than a stream of
/// events on purpose: a person glancing at their machine wants to know what is
/// happening *now*, and a list answers that after a missed notification where a
/// stream does not. What happened *earlier* is a different question, and it is
/// the record's.
#[derive(Debug, Default)]
pub struct Indicator {
    /// What is showing, oldest first.
    showing: Vec<Shown>,
    /// The next line number. Never reused.
    next: u64,
}

impl Indicator {
    /// Ask the policy, and — if it permits — begin showing this egress.
    ///
    /// The two are one call because they must not be two: an egress that was
    /// permitted and not shown is exactly the failure law 1 exists to prevent,
    /// and separating the decision from the display is how that failure gets
    /// written by accident.
    ///
    /// The [`Departing`] that comes back is what a caller must hold to open the
    /// connection, and it is the only thing that means so.
    ///
    /// ```
    /// use alo_capability::Grantee;
    /// use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why, egress_words};
    /// use alo_strings::Strings;
    /// use std::time::{Duration, SystemTime};
    ///
    /// # fn main() {
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let strings = Strings::of(egress_words().expect("this crate's own words"));
    /// let mut indicator = Indicator::default();
    /// assert!(indicator.is_quiet());
    ///
    /// let leaving = Leaving::because(
    ///     &Grantee::named("@files"),
    ///     Why::Fetching,
    ///     Destination::at("alo.example").expect("a host that can be shown"),
    /// );
    /// let departing = indicator
    ///     .beginning(&EgressPolicy::Anywhere, leaving, now)
    ///     .expect("nothing forbids this");
    /// assert_eq!(
    ///     indicator.showing()[0].said(&strings).text(),
    ///     "@files is fetching something from alo.example",
    /// );
    ///
    /// indicator.ended(departing);
    /// assert!(indicator.is_quiet());
    /// # }
    /// ```
    ///
    /// The same departure ended twice is not a program, so one connection can
    /// never take two lines off the indicator:
    ///
    /// ```compile_fail
    /// use alo_capability::Grantee;
    /// use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
    /// use std::time::{Duration, SystemTime};
    ///
    /// # fn main() {
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let mut indicator = Indicator::default();
    /// let leaving = Leaving::because(
    ///     &Grantee::named("@files"),
    ///     Why::Fetching,
    ///     Destination::at("alo.example").expect("a host that can be shown"),
    /// );
    /// let departing = indicator
    ///     .beginning(&EgressPolicy::Anywhere, leaving, now)
    ///     .expect("nothing forbids this");
    /// indicator.ended(departing);
    /// indicator.ended(departing); // the departure was spent above
    /// # }
    /// ```
    ///
    /// # Errors
    /// [`NotPermitted`], carrying the egress it refused, when the policy does
    /// not permit it. A refused egress is never shown, because nothing left.
    pub fn beginning(
        &mut self,
        policy: &EgressPolicy,
        leaving: Leaving,
        now: SystemTime,
    ) -> Result<Departing, NotPermitted> {
        if let Some(refused) = policy.refusal(&leaving) {
            return Err(refused);
        }
        let id = ShownId(self.next);
        self.next += 1;
        self.showing.push(Shown {
            id,
            at: now,
            leaving: leaving.clone(),
        });
        Ok(Departing::new(leaving, now, id))
    }

    /// Stop showing a departure that has finished.
    ///
    /// Takes the [`Departing`] rather than borrowing it, so one connection ends
    /// exactly one line. Answers whether that line was on this indicator: a
    /// departure from another indicator entirely is a programming mistake worth
    /// noticing rather than a silent no-op.
    pub fn ended(&mut self, departing: Departing) -> bool {
        let before = self.showing.len();
        self.showing.retain(|shown| shown.id != departing.shown());
        self.showing.len() < before
    }

    /// What is leaving right now, oldest first.
    #[must_use]
    pub fn showing(&self) -> &[Shown] {
        &self.showing
    }

    /// Whether nothing is leaving.
    ///
    /// The state a machine answering its own questions is in all day.
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.showing.is_empty()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::destination::Destination;
    use crate::leaving::Why;
    use crate::testing::in_english;
    use alo_capability::Grantee;
    use alo_models::{InferenceSource, Region};
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn mail() -> Grantee {
        Grantee::named("@mail")
    }

    fn fetching() -> Leaving {
        Leaving::because(
            &Grantee::named("@files"),
            Why::Fetching,
            Destination::at("alo.example").unwrap(),
        )
    }

    fn asking_alo() -> Leaving {
        Leaving::asking(
            &mail(),
            &InferenceSource::Hosted {
                provider: "alo".to_owned(),
                region: Region::Declared("the EU".to_owned()),
            },
        )
        .unwrap()
    }

    /// **The guarantee `CLAUDE.md` makes in public:** no agent-caused egress
    /// escapes the indicator. There is no way to hold a [`Departing`] that is
    /// not showing, because the only thing that makes one shows it first.
    #[test]
    fn nothing_is_permitted_to_leave_without_being_shown() {
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_alo(), noon())
            .unwrap();
        assert!(!indicator.is_quiet());
        assert_eq!(indicator.showing().len(), 1);
        let shown = indicator.showing().first().unwrap();
        assert_eq!(shown.id(), departing.shown());
        assert_eq!(shown.at(), noon());
        assert_eq!(
            shown.said(&in_english()).text(),
            "@mail is asking a question of alo, in the EU"
        );
    }

    /// **A refusal shows nothing, because nothing left.** An indicator that lit
    /// up for an egress the policy stopped would teach people to ignore it.
    #[test]
    fn an_egress_the_policy_refuses_never_appears_on_the_indicator() {
        let mut indicator = Indicator::default();
        let refused = indicator
            .beginning(&EgressPolicy::NothingLeaves, asking_alo(), noon())
            .unwrap_err();
        assert!(indicator.is_quiet());
        assert!(indicator.showing().is_empty());
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains("let nothing leave"),
            "{refused:?}"
        );
        assert_eq!(refused.leaving().agent(), &mail());
    }

    /// **The refusal carries what it refused**, so whatever records it can say
    /// what the agent tried rather than only that something was stopped.
    #[test]
    fn a_refused_egress_can_still_be_said_in_full() {
        let mut indicator = Indicator::default();
        let refused = indicator
            .beginning(&EgressPolicy::InTheBuilding, fetching(), noon())
            .unwrap_err();
        assert_eq!(
            refused.leaving().said(&in_english()).text(),
            "@files is fetching something from alo.example"
        );
        assert_eq!(
            refused.leaving().destination(),
            &Destination::at("alo.example").unwrap()
        );
    }

    /// A departure ends exactly one line, and the ones beside it stay.
    #[test]
    fn ending_one_departure_leaves_the_others_showing() {
        let mut indicator = Indicator::default();
        let first = indicator
            .beginning(&EgressPolicy::Anywhere, asking_alo(), noon())
            .unwrap();
        let second = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon() + hour())
            .unwrap();
        assert_eq!(indicator.showing().len(), 2);

        assert!(indicator.ended(first));
        assert_eq!(indicator.showing().len(), 1);
        assert_eq!(
            indicator.showing().first().map(Shown::id),
            Some(second.shown())
        );
        assert!(!indicator.is_quiet());

        assert!(indicator.ended(second));
        assert!(indicator.is_quiet());
    }

    /// Numbers are never reused, so a line ended late cannot take away one that
    /// began since.
    #[test]
    fn a_line_ended_late_cannot_take_away_one_that_began_since() {
        let mut indicator = Indicator::default();
        let first = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();
        assert!(indicator.ended(first));

        let second = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon() + hour())
            .unwrap();
        assert_ne!(second.shown(), ShownId(0));
        assert_eq!(indicator.showing().len(), 1);

        // A departure this indicator has never shown ends nothing.
        let mut elsewhere = Indicator::default();
        let stranger = elsewhere
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();
        assert!(!indicator.ended(stranger));
        assert_eq!(indicator.showing().len(), 1);
    }

    /// **Law 1's measurement, from inside.** A working day answered on this
    /// machine puts nothing on the indicator, because a question answered here
    /// never becomes an egress in the first place.
    #[test]
    fn a_day_answered_on_this_machine_leaves_the_indicator_quiet() {
        let mut indicator = Indicator::default();
        for _ in 0..8 {
            assert!(Leaving::asking(&mail(), &InferenceSource::ThisMachine).is_err());
        }
        assert!(indicator.is_quiet());

        // And the moment one question goes elsewhere, it is on the list.
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_alo(), noon())
            .unwrap();
        assert!(!indicator.is_quiet());
        indicator.ended(departing);
        assert!(indicator.is_quiet());
    }

    /// What the indicator shows can be handed to whatever draws it, and cannot
    /// be read back into an egress nothing decided about.
    #[test]
    fn what_is_showing_can_be_written_down_for_whatever_draws_it() {
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_alo(), noon())
            .unwrap();
        let written = serde_json::to_string(indicator.showing()).unwrap();
        assert!(written.contains("alo"), "{written}");
        assert!(written.contains("asking"), "{written}");
        assert_eq!(departing.shown().as_u64(), 0);
    }
}
