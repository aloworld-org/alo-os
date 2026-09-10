//! The light itself: dark, or lit for this many things.
//!
//! This is the whole of what a person answers by glancing at their machine, and
//! the plan's acceptance for this task is about where the answer comes from
//! rather than about what it looks like: *the indicator is drawn from
//! `alo_egress::Indicator` and nothing else*, and *it cannot be drawn from a
//! value that was not a departure.*
//!
//! # There is one door, and it is a real indicator
//!
//! [`Lamp::of`] takes `&alo_egress::Indicator` and reads it. There is no
//! constructor here that takes a count, a boolean or a state somebody named —
//! not a private one this crate could reach past either, because the field is
//! an `Option<NonZeroUsize>` filled in one function. That is deliberate and it
//! is the point of the type:
//!
//! - `Indicator::beginning` is the only way to obtain permission to open a
//!   connection, and it puts the egress on the list **before** it hands that
//!   permission back. A line on the indicator is therefore something that
//!   really left.
//! - An egress a policy refused never reaches the list at all, because nothing
//!   left. A light that lit for the connections a rule stopped would teach
//!   people to ignore the one thing on the screen that matters.
//! - So a lit lamp is a departure, and there is no arithmetic anywhere in this
//!   crate that could produce one without a departure behind it.
//!
//! `alo-overlay`'s `Quiet` has a second constructor from a
//! count, for a shell that was **told** what a daemon's indicator holds over
//! `alo-protocol`. This deliberately does not, and the difference is the
//! plan's: what is drawn on the screen is the machine's own indicator, and a
//! machine whose light could be lit from a number would have a second answer to
//! *has anything left this machine*. Drawing one from another machine's report
//! is a different feature, and it will be a different type with the report in
//! its name.
//!
//! # A number, not a boolean
//!
//! The lamp knows how many, because the surface next to it is the list and the
//! two must not be able to disagree — a light that said *something is leaving*
//! beside three lines would be a light nobody trusts. [`Lamp::is_dark`] is the
//! glance; [`Lamp::how_many`] is what the list is long enough for.

use std::num::NonZeroUsize;

use alo_egress::Indicator;
use alo_strings::{Counting, Filling, Said, Strings};

use crate::words::{self, Counted, Word};

/// The egress indicator's light at one moment.
///
/// Dark when nothing is leaving this machine — the state a machine answering
/// its own questions is in all day — and lit for as many things as are leaving
/// when something is.
///
/// ```
/// use alo_capability::Grantee;
/// use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
/// use alo_indicator::Lamp;
/// use std::time::{Duration, SystemTime};
///
/// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
/// let mut indicator = Indicator::default();
/// assert!(Lamp::of(&indicator).is_dark());
///
/// let departing = indicator.beginning(
///     &EgressPolicy::Anywhere,
///     Leaving::because(
///         &Grantee::named("@files"),
///         Why::Fetching,
///         Destination::at("alo.example").expect("a host that can be shown"),
///     ),
///     now,
/// )?;
/// assert_eq!(Lamp::of(&indicator).how_many(), 1);
///
/// indicator.ended(departing);
/// assert!(Lamp::of(&indicator).is_dark());
/// # Ok::<(), alo_egress::NotPermitted>(())
/// ```
///
/// A lamp cannot be lit from a number somebody made up, and this is what says
/// so — there is no such constructor to call:
///
/// ```compile_fail
/// let lit = alo_indicator::Lamp::of_how_many(3);
/// ```
///
/// nor can one be assembled from its parts, because it has none that are
/// reachable:
///
/// ```compile_fail
/// let lit = alo_indicator::Lamp { lit: std::num::NonZeroUsize::new(3) };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lamp {
    /// How many things are leaving, or [`None`] while the light is dark.
    ///
    /// Filled by [`Lamp::of`] and by nothing else, which is the whole
    /// guarantee this type makes.
    lit: Option<NonZeroUsize>,
}

impl Lamp {
    /// What this machine's indicator is showing, right now.
    ///
    /// The only way to a [`Lamp`].
    #[must_use]
    pub fn of(indicator: &Indicator) -> Self {
        Self {
            lit: if indicator.is_quiet() {
                None
            } else {
                NonZeroUsize::new(indicator.showing().len())
            },
        }
    }

    /// Whether the light is dark: nothing is leaving this machine.
    #[must_use]
    pub const fn is_dark(self) -> bool {
        self.lit.is_none()
    }

    /// Whether the light is lit: something is.
    #[must_use]
    pub const fn is_lit(self) -> bool {
        self.lit.is_some()
    }

    /// How many things are leaving, which is zero while it is dark.
    #[must_use]
    pub const fn how_many(self) -> usize {
        match self.lit {
            None => 0,
            Some(how_many) => how_many.get(),
        }
    }

    /// The plain string this crate declares for it, where it has one.
    #[must_use]
    pub const fn word(self) -> Option<Word> {
        match self.lit {
            None => Some(words::NOTHING_IS_LEAVING),
            Some(_) => None,
        }
    }

    /// The countable string this crate declares for it, where it has one.
    #[must_use]
    pub const fn counted(self) -> Option<Counted> {
        match self.lit {
            None => None,
            Some(_) => Some(words::HOW_MANY_ARE_LEAVING),
        }
    }

    /// What the light reads as, in the language the person reads.
    ///
    /// The whole of what the control says: a light with no name is a light that
    /// does not exist for somebody using a screen reader, and law 1 is not a
    /// promise alo OS keeps for people who can see the light and breaks for
    /// people who cannot.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self.lit {
            None => strings.say(&words::NOTHING_IS_LEAVING.key(), &Filling::nothing()),
            Some(how_many) => strings.count(
                &words::HOW_MANY_ARE_LEAVING.key(),
                // Saturating rather than casting: a machine with more open
                // connections than a `u64` can hold does not exist, and a cast
                // that wrapped would light the lamp for a number that was not
                // the number of departures.
                &Counting::of(u64::try_from(how_many.get()).unwrap_or(u64::MAX)),
                &Filling::nothing(),
            ),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{fetching, in_english, noon};
    use alo_egress::{Destination, EgressPolicy, Errand, OnItsOwn};

    /// **A machine with nothing leaving has a dark lamp**, which is law 1's
    /// measurement seen from the light a person glances at.
    #[test]
    fn a_machine_with_nothing_leaving_is_dark() {
        let lamp = Lamp::of(&Indicator::default());
        assert!(lamp.is_dark());
        assert!(!lamp.is_lit());
        assert_eq!(lamp.how_many(), 0);

        let said = lamp.said(&in_english());
        assert!(!said.is_a_bug(), "the reading is not declared");
        assert_eq!(said.text(), "Nothing is leaving this machine right now");
    }

    /// **The lamp is lit the moment something is permitted to leave**, and dark
    /// again the moment it ends. There is no state in which a connection has
    /// permission and the light is still dark, because the permission and the
    /// line are one act inside `alo-egress`.
    #[test]
    fn the_lamp_is_lit_while_something_is_leaving_and_dark_after() {
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();
        let lamp = Lamp::of(&indicator);
        assert!(lamp.is_lit());
        assert_eq!(lamp.how_many(), 1);
        assert_eq!(
            lamp.said(&in_english()).text(),
            "One thing is leaving this machine right now"
        );

        assert!(indicator.ended(departing));
        assert!(Lamp::of(&indicator).is_dark());
    }

    /// **What alo OS does on its own lights it too.** A lamp that counted only
    /// what agents caused would be dark while the machine downloaded a model,
    /// which is the comfortable answer and the false one.
    #[test]
    fn what_the_machine_does_on_its_own_lights_it_as_well() {
        let mut indicator = Indicator::default();
        let underway = indicator.beginning_on_its_own(
            OnItsOwn::for_(
                Errand::FetchingAModel,
                Destination::at("models.alo.example").unwrap(),
            ),
            noon(),
        );
        assert_eq!(Lamp::of(&indicator).how_many(), 1);

        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();
        assert_eq!(
            Lamp::of(&indicator).said(&in_english()).text(),
            "2 things are leaving this machine right now"
        );

        assert!(indicator.ended(departing));
        assert!(indicator.ended_on_its_own(underway));
        assert!(Lamp::of(&indicator).is_dark());
    }

    /// **An egress the policy refused leaves the lamp dark.** Nothing left, so
    /// there is nothing to light it with — this is the acceptance's *it cannot
    /// be drawn from a value that was not a departure*, at the one place a
    /// refusal could plausibly have leaked into the light.
    #[test]
    fn an_egress_the_policy_refused_leaves_the_lamp_dark() {
        let mut indicator = Indicator::default();
        let refused = indicator.beginning(&EgressPolicy::NothingLeaves, fetching(), noon());
        assert!(
            refused.is_err(),
            "the policy permitted it, so this proves nothing"
        );
        let lamp = Lamp::of(&indicator);
        assert!(lamp.is_dark());
        assert_eq!(lamp.how_many(), 0);
        assert_eq!(
            lamp.said(&in_english()).text(),
            "Nothing is leaving this machine right now"
        );
    }

    /// Each state reaches the kind of string it is declared as, and neither
    /// reaches the other's.
    #[test]
    fn each_state_names_the_kind_of_string_it_is() {
        let dark = Lamp::of(&Indicator::default());
        assert_eq!(dark.word(), Some(words::NOTHING_IS_LEAVING));
        assert_eq!(dark.counted(), None);

        let mut indicator = Indicator::default();
        drop(indicator.beginning(&EgressPolicy::Anywhere, fetching(), noon()));
        let lit = Lamp::of(&indicator);
        assert_eq!(lit.word(), None);
        assert_eq!(lit.counted(), Some(words::HOW_MANY_ARE_LEAVING));
    }

    /// **The light reads in the language the person reads**, and says so — the
    /// whole of what declaring these through `alo-strings` buys.
    #[test]
    fn the_light_reads_in_the_language_the_person_reads() {
        let strings = crate::testing::translated(&[(
            words::NOTHING_IS_LEAVING,
            "Zurzeit verlässt nichts dieses Gerät",
        )]);
        let said = Lamp::of(&Indicator::default()).said(&strings);
        assert!(said.is_translated());
        assert_eq!(said.text(), "Zurzeit verlässt nichts dieses Gerät");
    }
}
