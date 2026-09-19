//! Everything a person has changed about their screens, which is the only part
//! written down: every arrangement they have made, and night light.
//!
//! # An arrangement is per set of screens; night light is not
//!
//! Where a screen sits is a fact about the set it is in, so it is remembered
//! per set. When the evening starts is not: the clock and the sun are the same
//! at both desks, and somebody who warms their screens at ten does not stop
//! wanting that when they plug a monitor in. So there is one night light for
//! the machine, and [`Changes::night_light`] is absent until the person has
//! touched it at all.
//!
//! The shape `alo-appearance`, `alo-dock` and `alo-sleeping` keep, for the same
//! reason: what alo OS ships lives in the running release and the file holds
//! the difference. Here the difference is the whole of it — alo OS ships **no**
//! arrangement, because it has never seen anybody's screens, and a machine
//! nobody has arranged has no `displays.toml` at all ([`crate::keeping`]).
//!
//! # One arrangement per set of screens, newest last
//!
//! Remembering the same set again replaces the arrangement for it, because two
//! rows for one set is a file that disagrees with itself about where somebody's
//! screens are. The order is the order they were last remembered in, which is
//! what makes *the last size chosen for this screen* answerable.
//!
//! # A size follows the screen; a place follows the set
//!
//! [`Changes::for_screens`] answers with the arrangement for exactly this set,
//! and nothing else: the office's layout is the office's. But a screen that is
//! new to *this* set is very often not new to the machine — plugging a fourth
//! screen in makes a set nobody has arranged while three of the four are
//! screens somebody has sized. So [`Changes::the_size_last_chosen_for`] answers
//! across every arrangement, newest first, and [`crate::Attached`] uses it
//! before it works a size out from the glass. How large a screen should draw is
//! a fact about that screen; where it sits is a fact about the set it is in.

use serde::{Deserialize, Serialize};

use crate::arrangement::{Arrangement, Screens};
use crate::identity::Identity;
use crate::night_light::NightLight;
use crate::scale::Scale;

/// Everything a person has changed about their screens.
///
/// This is what `displays.toml` holds and nothing else ([`crate::keeping`]): a
/// machine nobody has arranged and who has never asked for night light has no
/// file at all.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub struct Changes {
    /// One arrangement per set of screens, oldest first.
    arrangements: Vec<Arrangement>,
    /// Night light, where the person has touched it at all.
    ///
    /// One for the machine rather than one per set of screens: the sun and the
    /// clock are the same at both desks, and somebody who warms their screens
    /// in the evening does not stop wanting that when they plug a monitor in.
    night_light: Option<NightLight>,
}

impl Changes {
    /// Nothing arranged yet.
    #[must_use]
    pub fn untouched() -> Self {
        Self::default()
    }

    /// Whether nothing has been changed at all, which is what a fresh machine
    /// has, and what a missing file reads as.
    #[must_use]
    pub fn is_untouched(&self) -> bool {
        self.arrangements.is_empty() && self.night_light.is_none()
    }

    /// Night light as the person set it, if they have touched it at all.
    ///
    /// Nothing is [`crate::NightLight::as_shipped`], which is off — answered as
    /// an absence rather than as that value, because *the person has not asked*
    /// and *the person asked for what the release ships* are different things
    /// to a release that later ships something else.
    #[must_use]
    pub const fn night_light(&self) -> Option<NightLight> {
        self.night_light
    }

    /// Night light, set to this.
    pub const fn set_night_light(&mut self, night_light: NightLight) {
        self.night_light = Some(night_light);
    }

    /// Night light, back to what the release ships.
    ///
    /// Says whether there was anything to put back.
    pub const fn forget_night_light(&mut self) -> bool {
        let had = self.night_light.is_some();
        self.night_light = None;
        had
    }

    /// Remember this arrangement, replacing any for the same set of screens.
    pub fn remember(&mut self, arrangement: Arrangement) {
        let screens = arrangement.screens();
        self.arrangements.retain(|held| held.screens() != screens);
        self.arrangements.push(arrangement);
    }

    /// Forget the arrangement for this set of screens, which puts it back to a
    /// set alo OS has never been asked about.
    ///
    /// Says whether there was anything to forget.
    pub fn forget(&mut self, screens: &Screens) -> bool {
        let before = self.arrangements.len();
        self.arrangements.retain(|held| &held.screens() != screens);
        self.arrangements.len() != before
    }

    /// Forget every arrangement.
    pub fn forget_everything(&mut self) {
        *self = Self::untouched();
    }

    /// How this exact set of screens was last laid out, if it ever was.
    #[must_use]
    pub fn for_screens(&self, screens: &Screens) -> Option<&Arrangement> {
        self.arrangements
            .iter()
            .find(|held| &held.screens() == screens)
    }

    /// The size this screen was last given, in any arrangement it appears in.
    ///
    /// Newest first, so the size a person chose most recently is the one a set
    /// they have never arranged starts from.
    #[must_use]
    pub fn the_size_last_chosen_for(&self, screen: &Identity) -> Option<Scale> {
        self.arrangements
            .iter()
            .rev()
            .find_map(|held| held.placed(screen).map(|placed| placed.scale()))
    }

    /// Every arrangement, oldest first.
    pub fn each(&self) -> impl Iterator<Item = &Arrangement> {
        self.arrangements.iter()
    }

    /// How many sets of screens have been arranged.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.arrangements.len()
    }
}

/// Changes as a settings file holds them: a machine nobody has changed writes
/// no keys at all.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Written {
    /// Every arrangement.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    arrangements: Vec<Arrangement>,
    /// Night light, where the person has touched it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    night_light: Option<NightLight>,
}

impl From<Written> for Changes {
    /// Normalising, because a file is a thing a person can edit: a set of
    /// screens arranged twice means what the file says last, which is the only
    /// reading that does not throw one of the two away at random. It is
    /// `alo-appearance`'s answer to the same question about a display named
    /// twice, and for the same reason.
    fn from(written: Written) -> Self {
        let mut changes = Self::untouched();
        for arrangement in written.arrangements {
            changes.remember(arrangement);
        }
        changes.night_light = written.night_light;
        changes
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self {
            arrangements: changes.arrangements,
            night_light: changes.night_light,
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
    use crate::placed::{Placed, Position};
    use crate::testing::{an_arrangement, the_home_screen, the_laptop, the_office_screen};
    use crate::warmth::Warmth;

    /// **One arrangement per set of screens**, replaced rather than added to
    /// when the same set is arranged again — and forgotten on request.
    #[test]
    fn one_arrangement_per_set_of_screens() {
        let mut changes = Changes::untouched();
        assert!(changes.is_untouched());

        let office = an_arrangement(&[(the_laptop(), 0, 100), (the_office_screen(), -2560, 150)]);
        let set = office.screens();
        changes.remember(office);
        assert_eq!(changes.how_many(), 1);

        let moved = an_arrangement(&[(the_laptop(), 0, 100), (the_office_screen(), 1920, 150)]);
        changes.remember(moved);
        assert_eq!(changes.how_many(), 1, "the same set, arranged again");
        assert_eq!(
            changes
                .for_screens(&set)
                .unwrap()
                .placed(&the_office_screen())
                .unwrap()
                .position(),
            Position::at(1920, 0)
        );

        assert!(changes.forget(&set));
        assert!(!changes.forget(&set));
        assert!(changes.is_untouched());
    }

    /// **Two sets of screens are two arrangements**, and asking for one never
    /// answers with the other — which is the whole of *docking at the office
    /// restores the office*.
    #[test]
    fn two_sets_of_screens_are_two_arrangements() {
        let mut changes = Changes::untouched();
        let office = an_arrangement(&[(the_laptop(), 0, 100), (the_office_screen(), -2560, 150)]);
        let home = an_arrangement(&[(the_laptop(), 0, 100), (the_home_screen(), 1920, 100)]);
        let office_set = office.screens();
        let home_set = home.screens();
        changes.remember(office);
        changes.remember(home);
        assert_eq!(changes.how_many(), 2);
        assert_ne!(office_set, home_set);
        assert!(
            changes
                .for_screens(&office_set)
                .unwrap()
                .placed(&the_home_screen())
                .is_none()
        );
        assert_eq!(
            changes
                .for_screens(&home_set)
                .unwrap()
                .placed(&the_home_screen())
                .unwrap()
                .position(),
            Position::at(1920, 0)
        );
    }

    /// **A size follows the screen.** The size last chosen for a screen is
    /// found in whichever arrangement it was chosen in, newest first.
    #[test]
    fn the_size_last_chosen_for_a_screen_is_found_in_any_arrangement() {
        let mut changes = Changes::untouched();
        changes.remember(an_arrangement(&[
            (the_laptop(), 0, 200),
            (the_office_screen(), -2560, 150),
        ]));
        assert_eq!(
            changes.the_size_last_chosen_for(&the_office_screen()),
            Some(Scale::per_cent(150).unwrap())
        );
        assert_eq!(
            changes.the_size_last_chosen_for(&the_laptop()),
            Some(Scale::per_cent(200).unwrap())
        );
        assert_eq!(changes.the_size_last_chosen_for(&the_home_screen()), None);

        changes.remember(an_arrangement(&[
            (the_laptop(), 0, 175),
            (the_home_screen(), 1920, 100),
        ]));
        assert_eq!(
            changes.the_size_last_chosen_for(&the_laptop()),
            Some(Scale::per_cent(175).unwrap()),
            "the most recent choice, not the first"
        );
    }

    /// **A set arranged twice in a hand-edited file means what the file says
    /// last**, rather than one of the two at random.
    #[test]
    fn a_set_arranged_twice_in_a_file_means_what_it_says_last() {
        let first = an_arrangement(&[(the_laptop(), 0, 100), (the_office_screen(), -2560, 100)]);
        let second = an_arrangement(&[(the_laptop(), 0, 100), (the_office_screen(), 2560, 100)]);
        let set = first.screens();
        let written = Written {
            arrangements: vec![first, second],
            night_light: None,
        };
        let changes = Changes::from(written);
        assert_eq!(changes.how_many(), 1);
        assert_eq!(
            changes
                .for_screens(&set)
                .unwrap()
                .placed(&the_office_screen())
                .unwrap()
                .position(),
            Position::at(2560, 0)
        );
    }

    /// Changes survive being written down and read back, and a main screen at
    /// a size stays exactly where it was put.
    #[test]
    fn changes_survive_being_written_down() {
        let mut changes = Changes::untouched();
        changes.remember(an_arrangement(&[
            (the_laptop(), 0, 175),
            (the_office_screen(), -2560, 150),
        ]));
        let written = serde_json::to_string(&changes).unwrap();
        assert_eq!(serde_json::from_str::<Changes>(&written).unwrap(), changes);
        assert_eq!(
            serde_json::to_string(&Changes::untouched()).unwrap(),
            "{}",
            "a machine nobody has arranged writes nothing"
        );
    }

    /// **Night light is one setting for the machine**, remembered whatever
    /// screens are plugged in and absent until the person has touched it — so
    /// that a later release changing what it ships reaches everybody who never
    /// asked.
    #[test]
    fn night_light_is_one_setting_for_the_machine_and_is_absent_until_asked_for() {
        let mut changes = Changes::untouched();
        assert_eq!(changes.night_light(), None);
        assert!(changes.is_untouched());

        let asked = NightLight::as_shipped().at_this_warmth(Warmth::kelvin(2700).unwrap());
        changes.set_night_light(asked);
        assert_eq!(changes.night_light(), Some(asked));
        assert!(
            !changes.is_untouched(),
            "a machine with night light set is not an untouched one"
        );

        changes.remember(an_arrangement(&[
            (the_laptop(), 0, 175),
            (the_office_screen(), -2560, 150),
        ]));
        assert_eq!(
            changes.night_light(),
            Some(asked),
            "arranging screens says nothing about night light"
        );
        assert!(
            changes.forget(
                &an_arrangement(&[(the_laptop(), 0, 175), (the_office_screen(), -2560, 150),])
                    .screens()
            )
        );
        assert_eq!(
            changes.night_light(),
            Some(asked),
            "and forgetting an arrangement says nothing about it either"
        );

        assert!(changes.forget_night_light());
        assert!(!changes.forget_night_light());
        assert!(changes.is_untouched());
    }

    /// Night light survives being written down beside an arrangement, and the
    /// two are read back independently.
    #[test]
    fn night_light_and_an_arrangement_are_written_down_together() {
        let mut changes = Changes::untouched();
        changes.remember(an_arrangement(&[
            (the_laptop(), 0, 175),
            (the_office_screen(), -2560, 150),
        ]));
        changes.set_night_light(
            NightLight::as_shipped().at_this_warmth(Warmth::kelvin(2700).unwrap()),
        );
        let written = serde_json::to_string(&changes).unwrap();
        assert!(written.contains("night-light"), "{written}");
        assert_eq!(serde_json::from_str::<Changes>(&written).unwrap(), changes);
    }

    /// A place a test puts a screen at, so the shape above stays readable.
    #[test]
    fn a_place_is_a_corner_and_a_size() {
        let placed = Placed::at(Position::at(-2560, 0), Scale::per_cent(150).unwrap());
        assert_eq!(placed.position().across(), -2560);
        assert_eq!(placed.scale(), Scale::per_cent(150).unwrap());
    }
}
