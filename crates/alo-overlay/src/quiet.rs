//! Whether anything is leaving this machine, on the line beneath the other
//! two.
//!
//! The third of the three readings the overlay shows at rest, and the one law
//! 1 is about: *every network egress an agent causes is visible at the moment
//! it happens.* `alo-egress` decides and shows; this is that indicator's
//! answer put where a person actually looks — inside the thing they opened
//! with the agent's key, rather than in a panel they would have to go and
//! find.
//!
//! # It is drawn from the indicator and from nothing else
//!
//! [`Quiet::of`] takes `&alo_egress::Indicator` and reads it. There is no
//! constructor here that takes a number a caller made up, and that is the
//! point rather than an accident: `Indicator::beginning` is the only way to
//! obtain permission to open a connection and it puts the egress on the list
//! *before* it hands that permission back, so a line on the indicator is
//! something that really left. An overlay that could be handed a count would
//! be a second, quieter answer to *has anything left this machine*, and a
//! person watching for that must not have two places to look.
//!
//! **A refused egress therefore shows nothing**, because nothing left. An
//! indicator that lit up for the connections a policy stopped would teach
//! people to ignore it.
//!
//! # This is the reading, not the indicator on a screen
//!
//! The plan's task 9 is the indicator itself: drawn from
//! `alo_egress::Indicator`, dark for a local answer, lit while a provider
//! answers. This is a line of text in an overlay that is at rest, and it
//! claims no pixels. What both have in common is where the number comes from,
//! and that is deliberate — one source, so the light and the line cannot
//! disagree.

use std::num::NonZeroUsize;

use alo_egress::Indicator;
use alo_strings::{Counting, Filling, Said, Strings};

use crate::words::{self, Counted, Word};

/// What is leaving this machine at this moment.
///
/// Two cases, matching the two things a person wants to know: **nothing**,
/// which is the state a machine answering its own questions is in all day, and
/// **this many**, counted in the reader's own language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quiet {
    /// Nothing at all — neither an agent's egress nor an errand alo OS is
    /// running on its own.
    NothingIsLeaving,
    /// This many connections are open at this moment.
    ThisMany(NonZeroUsize),
}

impl Quiet {
    /// What this machine's indicator is showing.
    #[must_use]
    pub fn of(indicator: &Indicator) -> Self {
        if indicator.is_quiet() {
            return Self::NothingIsLeaving;
        }
        Self::of_how_many(indicator.showing().len())
    }

    /// The same, from a count somebody has already made.
    ///
    /// Not a way around [`Quiet::of`]: what it is for is a shell that has been
    /// **told** what the indicator holds — over `alo-protocol`, from a daemon
    /// in another process — rather than one making a number up. The honest
    /// reading of a machine whose indicator is out of reach is still a reading
    /// of that indicator's own count.
    #[must_use]
    pub fn of_how_many(how_many: usize) -> Self {
        NonZeroUsize::new(how_many).map_or(Self::NothingIsLeaving, Self::ThisMany)
    }

    /// How many, as a number.
    #[must_use]
    pub const fn how_many(self) -> usize {
        match self {
            Self::NothingIsLeaving => 0,
            Self::ThisMany(how_many) => how_many.get(),
        }
    }

    /// Whether nothing is leaving.
    #[must_use]
    pub const fn is_quiet(self) -> bool {
        matches!(self, Self::NothingIsLeaving)
    }

    /// The plain string this crate declares for it, where it has one.
    #[must_use]
    pub const fn word(self) -> Option<Word> {
        match self {
            Self::NothingIsLeaving => Some(words::NOTHING_IS_LEAVING),
            Self::ThisMany(_) => None,
        }
    }

    /// The countable string this crate declares for it, where it has one.
    #[must_use]
    pub const fn counted(self) -> Option<Counted> {
        match self {
            Self::NothingIsLeaving => None,
            Self::ThisMany(_) => Some(words::HOW_MANY_ARE_LEAVING),
        }
    }

    /// The line a person reads, in the language they read it in.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self {
            Self::NothingIsLeaving => {
                strings.say(&words::NOTHING_IS_LEAVING.key(), &Filling::nothing())
            }
            Self::ThisMany(how_many) => strings.count(
                &words::HOW_MANY_ARE_LEAVING.key(),
                // Saturating rather than casting, for `crate::granted`'s
                // reason: a machine with more open connections than a `u64`
                // can hold does not exist.
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
    use crate::testing::in_english;
    use alo_capability::Grantee;
    use alo_egress::{Destination, EgressPolicy, Errand, Leaving, OnItsOwn, Why};
    use std::time::{Duration, SystemTime};

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// An agent fetching something from somewhere else.
    fn fetching() -> Leaving {
        Leaving::because(
            &Grantee::named("@files"),
            Why::Fetching,
            Destination::at("alo.example").unwrap(),
        )
    }

    /// **A machine answering its own questions is quiet**, which is law 1's
    /// measurement seen from the line a person reads.
    #[test]
    fn a_machine_with_nothing_leaving_says_nothing_is_leaving() {
        let quiet = Quiet::of(&Indicator::default());
        assert_eq!(quiet, Quiet::NothingIsLeaving);
        assert!(quiet.is_quiet());

        let said = quiet.said(&in_english());
        assert!(!said.is_a_bug(), "the reading is not declared");
        assert_eq!(said.text(), "nothing is leaving this machine");
    }

    /// One connection is one, and it is on the line the moment it is
    /// permitted — there is no state in which something has permission and
    /// the overlay is still saying the machine is quiet.
    #[test]
    fn something_leaving_is_on_the_line_the_moment_it_is_permitted() {
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();
        let quiet = Quiet::of(&indicator);
        assert_eq!(quiet.how_many(), 1);
        assert!(!quiet.is_quiet());
        assert_eq!(
            quiet.said(&in_english()).text(),
            "one thing is leaving this machine right now"
        );

        // And it goes off the line when the connection ends.
        assert!(indicator.ended(departing));
        assert_eq!(Quiet::of(&indicator), Quiet::NothingIsLeaving);
    }

    /// **What alo OS does on its own is counted too.** A line that counted
    /// only what agents caused would be quiet while the machine downloaded a
    /// model, which is the comfortable answer and the false one.
    #[test]
    fn what_the_machine_does_on_its_own_is_counted_as_well() {
        let mut indicator = Indicator::default();
        let underway = indicator.beginning_on_its_own(
            OnItsOwn::for_(
                Errand::FetchingAModel,
                Destination::at("models.alo.example").unwrap(),
            ),
            noon(),
        );
        assert_eq!(Quiet::of(&indicator).how_many(), 1);
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();
        assert_eq!(
            Quiet::of(&indicator).said(&in_english()).text(),
            "2 things are leaving this machine right now"
        );

        assert!(indicator.ended(departing));
        assert!(indicator.ended_on_its_own(underway));
        assert!(Quiet::of(&indicator).is_quiet());
    }

    /// **An egress the policy refused never reaches this line.** Nothing
    /// left, so there is nothing to show — an overlay that reported the
    /// connections a rule stopped would teach people to ignore the one thing
    /// on it that matters.
    #[test]
    fn an_egress_the_policy_refused_leaves_the_line_quiet() {
        let mut indicator = Indicator::default();
        let refused = indicator.beginning(&EgressPolicy::NothingLeaves, fetching(), noon());
        assert!(
            refused.is_err(),
            "the policy permitted it, so this proves nothing"
        );
        assert_eq!(Quiet::of(&indicator), Quiet::NothingIsLeaving);
        assert_eq!(
            Quiet::of(&indicator).said(&in_english()).text(),
            "nothing is leaving this machine"
        );
    }

    /// Each case reaches the kind of string it is declared as, and neither
    /// reaches the other's.
    #[test]
    fn each_case_names_the_kind_of_string_it_is() {
        assert_eq!(
            Quiet::NothingIsLeaving.word(),
            Some(words::NOTHING_IS_LEAVING)
        );
        assert_eq!(Quiet::NothingIsLeaving.counted(), None);
        let one = Quiet::of_how_many(1);
        assert_eq!(one.word(), None);
        assert_eq!(one.counted(), Some(words::HOW_MANY_ARE_LEAVING));
    }
}
