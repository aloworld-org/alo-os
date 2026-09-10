//! The whole indicator at one moment: the light, and the lines under it.
//!
//! A [`Lamp`] answers *is anything leaving* at a glance. `alo-egress` says of
//! itself that the indicator is **a list a person can read — who, where to and
//! why — rather than a light that is on or off**, and this is that list beside
//! that light, in one value the compositor is handed.
//!
//! # One value, so the light and the list cannot disagree
//!
//! Both halves are taken from one `alo_egress::Indicator` in one call, and
//! there is no way to build a [`Drawn`] with a lamp that came from one moment
//! and lines that came from another. A light saying *nothing is leaving* over
//! three readable lines would be worse than no indicator at all: it would be an
//! indicator a person had learned to disbelieve.
//!
//! # It is a copy, taken at a moment, and that is deliberate
//!
//! [`Drawn::of`] clones the lines rather than borrowing the indicator. A
//! compositor that held a borrow would hold the machine's egress list open
//! while it drew, and the thing that adds to that list is the thing that
//! permits a connection — so drawing would be in the way of leaving. What the
//! compositor gets instead is a picture of one moment, which is what it draws
//! anyway.
//!
//! Nothing here reads the clock, as everywhere else in this repository: each
//! line carries the moment it began, decided by whoever permitted it, so what a
//! person saw on the screen and what the record says cannot disagree about when
//! it happened.
//!
//! # What is not here
//!
//! No size, no place, no colour, no icon. *Dark* and *lit* are states, not
//! pixels: which pixels say so is the compositor's, and `docs/features.md` puts
//! the indicator's home in the dock's status area at v0.5, which is a decision
//! about furniture that this value does not need to know.

use alo_egress::{Indicator, Shown};
use alo_strings::{Said, Strings};

use crate::lamp::Lamp;

/// The egress indicator, as it stands at one moment.
///
/// Made only by [`Drawn::of`] from a real `alo_egress::Indicator`, which is the
/// plan's *drawn from `alo_egress::Indicator` and nothing else*. The lines are
/// `alo_egress::Shown`s, which that crate builds and nothing outside it can:
/// they do not deserialise either, so an egress read back off a disk cannot
/// become one.
///
/// ```
/// use alo_capability::Grantee;
/// use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
/// use alo_indicator::Drawn;
/// use std::time::{Duration, SystemTime};
///
/// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
/// let mut indicator = Indicator::default();
/// assert!(Drawn::of(&indicator).lamp().is_dark());
/// assert!(Drawn::of(&indicator).lines().is_empty());
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
/// let drawn = Drawn::of(&indicator);
/// assert!(drawn.lamp().is_lit());
/// assert_eq!(drawn.lines().len(), 1);
///
/// indicator.ended(departing);
/// assert!(Drawn::of(&indicator).lamp().is_dark());
/// # Ok::<(), alo_egress::NotPermitted>(())
/// ```
///
/// There is no other way to one. A picture cannot be assembled out of lines a
/// caller is holding:
///
/// ```compile_fail
/// let drawn = alo_indicator::Drawn::of_lines(Vec::new());
/// ```
///
/// nor out of its fields, which are this crate's:
///
/// ```compile_fail
/// let drawn = alo_indicator::Drawn { lamp: todo!(), lines: Vec::new() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    /// The light, from the same moment as the lines.
    lamp: Lamp,
    /// Every line the indicator was showing, oldest first.
    lines: Vec<Shown>,
}

impl Drawn {
    /// What this machine's indicator stands at, right now.
    ///
    /// The only way to a [`Drawn`].
    #[must_use]
    pub fn of(indicator: &Indicator) -> Self {
        Self {
            lamp: Lamp::of(indicator),
            lines: indicator.showing().to_vec(),
        }
    }

    /// The light.
    #[must_use]
    pub fn lamp(&self) -> Lamp {
        self.lamp
    }

    /// Every line, oldest first — empty exactly when the lamp is dark.
    #[must_use]
    pub fn lines(&self) -> &[Shown] {
        &self.lines
    }

    /// What the light reads as, in the language the person reads.
    #[must_use]
    pub fn lamp_said(&self, strings: &Strings) -> Said {
        self.lamp.said(strings)
    }

    /// Every line, in the language the person reads, oldest first.
    ///
    /// The sentences are `alo-egress`'s own — *@mail is asking a question of
    /// alo, in the EU* — because whoever decided the egress is who knows what
    /// it was. This crate does not word a line and could not: it never sees a
    /// destination or a reason, only the [`Shown`] that carries both.
    #[must_use]
    pub fn lines_said(&self, strings: &Strings) -> Vec<Said> {
        self.lines
            .iter()
            .map(|line| line.said(strings))
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{asking_a_provider, fetching, in_english, noon};
    use alo_egress::EgressPolicy;

    /// **The light and the list come from one moment**, so a dark lamp has no
    /// lines under it and a lit one has as many as it says.
    #[test]
    fn the_light_and_the_list_agree_about_the_same_moment() {
        let mut indicator = Indicator::default();
        let dark = Drawn::of(&indicator);
        assert!(dark.lamp().is_dark());
        assert!(dark.lines().is_empty());

        let first = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();
        let second = indicator
            .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
            .unwrap();
        let lit = Drawn::of(&indicator);
        assert_eq!(lit.lamp().how_many(), 2);
        assert_eq!(lit.lines().len(), lit.lamp().how_many());

        assert!(indicator.ended(first));
        assert!(indicator.ended(second));
        assert!(Drawn::of(&indicator).lamp().is_dark());
        assert!(Drawn::of(&indicator).lines().is_empty());
    }

    /// **A picture is a moment, not a window onto the machine.** One taken
    /// before a departure does not change when the departure happens — which is
    /// what lets a compositor draw it without holding the list that permitting
    /// a connection has to add to.
    #[test]
    fn a_picture_taken_earlier_does_not_change_underneath_whoever_is_drawing_it() {
        let mut indicator = Indicator::default();
        let before = Drawn::of(&indicator);
        drop(
            indicator
                .beginning(&EgressPolicy::Anywhere, fetching(), noon())
                .unwrap(),
        );
        assert!(before.lamp().is_dark());
        assert!(before.lines().is_empty());
        assert!(Drawn::of(&indicator).lamp().is_lit());
    }

    /// **Every line is worded by whoever decided the egress**, and reads in the
    /// person's own language. This crate never writes one: it does not see a
    /// destination or a reason at all.
    #[test]
    fn every_line_is_worded_by_the_crate_that_decided_the_egress() {
        let mut indicator = Indicator::default();
        drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
        let drawn = Drawn::of(&indicator);

        let strings = in_english();
        let lines = drawn.lines_said(&strings);
        assert_eq!(lines.len(), 1);
        let line = lines.first().unwrap();
        assert!(!line.is_a_bug(), "{line}");
        assert_eq!(line.text(), "@mail is asking a question of alo, in the EU");
        assert_eq!(
            drawn.lamp_said(&strings).text(),
            "One thing is leaving this machine right now"
        );
    }

    /// **An egress the policy refused is not on the list and not in the
    /// light.** The refusal path of the whole surface: nothing left, so there
    /// is nothing to draw.
    #[test]
    fn an_egress_the_policy_refused_is_drawn_nowhere() {
        let mut indicator = Indicator::default();
        assert!(
            indicator
                .beginning(&EgressPolicy::NothingLeaves, asking_a_provider(), noon())
                .is_err(),
            "the policy permitted it, so this proves nothing"
        );
        let drawn = Drawn::of(&indicator);
        assert!(drawn.lamp().is_dark());
        assert!(drawn.lines().is_empty());
        assert!(drawn.lines_said(&in_english()).is_empty());
    }
}
