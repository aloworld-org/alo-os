//! How a person has laid one set of screens out, and the set itself.
//!
//! An [`Arrangement`] is every screen of one set with its place: where it sits,
//! how large it draws, and which single one of them is the main screen. A
//! [`Screens`] is the set alone, and it is what an arrangement is remembered
//! under — docking at the office restores the office's arrangement because the
//! office's screens are a different set from home's, not because anything asked
//! where the machine is.
//!
//! # Exactly one main screen, and the file is checked for it
//!
//! Everything a shell does with *the* screen — where a window opens, where a
//! surface with nowhere else to go lands, where the windows of a screen that
//! was unplugged end up — is one screen or it is a shrug. So an arrangement
//! with none, or with two, is not an arrangement: it is refused, in words a
//! person can act on, whether it arrives from a settings panel or out of a file
//! they edited by hand.
//!
//! # Nothing here knows how large a screen is
//!
//! A place holds a size, not a rectangle. How much room a screen actually takes
//! depends on the pixels it reports at the moment it is plugged in, which is
//! not a thing a file can hold: a screen whose resolution changed has moved.
//! So an arrangement is checked for the things a file can be wrong about — a
//! screen named twice, no main screen, two — and [`crate::Attached`] is where it
//! meets the machine and the screens are checked for overlapping.

use serde::{Deserialize, Serialize};

use alo_strings::{Filling, Said, Strings, Word};

use crate::identity::{Identity, Panel, Socket};
use crate::placed::{Placed, Position};
use crate::scale::Scale;
use crate::unreadable::NotRead;
use crate::words;

/// Why this is not an arrangement of screens.
///
/// There is no `Display`: the only road to words is [`NotArranged::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotArranged {
    /// No screens at all.
    NoScreens,
    /// One screen written down twice, which is a file that disagrees with
    /// itself about where that screen is.
    TheSameScreenTwice(Identity),
    /// No screen marked as the main one.
    NoMainScreen,
    /// More than one screen marked as the main one.
    MoreThanOneMainScreen,
    /// A row that is neither a screen that says what it is nor a socket.
    HalfAScreen,
}

impl NotArranged {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match *self {
            Self::NoScreens => words::NO_SCREENS,
            Self::TheSameScreenTwice(_) => words::THE_SAME_SCREEN_TWICE,
            Self::NoMainScreen => words::NO_MAIN_SCREEN,
            Self::MoreThanOneMainScreen => words::MORE_THAN_ONE_MAIN_SCREEN,
            Self::HalfAScreen => words::HALF_A_SCREEN,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::TheSameScreenTwice(identity) => {
                Filling::of("display", identity.as_a_person_reads_it())
            }
            Self::NoScreens
            | Self::NoMainScreen
            | Self::MoreThanOneMainScreen
            | Self::HalfAScreen => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// One set of screens, which is what an arrangement is remembered under.
///
/// Sorted, so that plugging the same three screens in in a different order is
/// the same set. Two of them are equal when they hold the same screens, and
/// nothing else about the arrangement is in here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Screens {
    /// The screens, sorted.
    of: Vec<Identity>,
}

impl Screens {
    /// The set these screens make.
    ///
    /// # Errors
    /// [`NotArranged::NoScreens`] for none at all, and
    /// [`NotArranged::TheSameScreenTwice`] for one screen given twice.
    pub fn of(screens: impl IntoIterator<Item = Identity>) -> Result<Self, NotArranged> {
        let mut of: Vec<Identity> = screens.into_iter().collect();
        if of.is_empty() {
            return Err(NotArranged::NoScreens);
        }
        of.sort();
        if let Some(twice) = of.windows(2).find_map(|pair| match pair {
            [one, next] if one == next => Some(one.clone()),
            _ => None,
        }) {
            return Err(NotArranged::TheSameScreenTwice(twice));
        }
        Ok(Self { of })
    }

    /// The screens, sorted.
    pub fn each(&self) -> impl Iterator<Item = &Identity> {
        self.of.iter()
    }

    /// How many there are.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.of.len()
    }

    /// Whether this screen is one of them.
    #[must_use]
    pub fn holds(&self, screen: &Identity) -> bool {
        self.of.contains(screen)
    }
}

/// How a person has laid one set of screens out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct Arrangement {
    /// Every screen and its place, in the order they were given.
    places: Vec<(Identity, Placed)>,
    /// The one a new window opens on.
    main: Identity,
}

impl Arrangement {
    /// These screens, laid out like this.
    ///
    /// # Errors
    /// [`NotArranged`] for no screens, one screen written down twice, no main
    /// screen, or more than one.
    pub fn of(places: Vec<(Identity, Placed)>) -> Result<Self, NotArranged> {
        if places.is_empty() {
            return Err(NotArranged::NoScreens);
        }
        let _set = Screens::of(places.iter().map(|(identity, _)| identity.clone()))?;
        let mut main = None;
        for (identity, placed) in &places {
            if placed.is_the_main_screen() {
                if main.is_some() {
                    return Err(NotArranged::MoreThanOneMainScreen);
                }
                main = Some(identity.clone());
            }
        }
        let Some(main) = main else {
            return Err(NotArranged::NoMainScreen);
        };
        Ok(Self { places, main })
    }

    /// The set of screens this is remembered under.
    ///
    /// Cannot fail: an arrangement that could not make a set is not one that
    /// could be built.
    #[must_use]
    pub fn screens(&self) -> Screens {
        Screens {
            of: {
                let mut of: Vec<Identity> = self
                    .places
                    .iter()
                    .map(|(identity, _)| identity.clone())
                    .collect();
                of.sort();
                of
            },
        }
    }

    /// Where this screen is, if it is one of them.
    #[must_use]
    pub fn placed(&self, screen: &Identity) -> Option<Placed> {
        self.places
            .iter()
            .find(|(identity, _)| identity == screen)
            .map(|(_, placed)| *placed)
    }

    /// The screen a new window opens on.
    #[must_use]
    pub const fn main_screen(&self) -> &Identity {
        &self.main
    }

    /// Every screen and its place, in the order they were given.
    pub fn places(&self) -> impl Iterator<Item = (&Identity, Placed)> {
        self.places
            .iter()
            .map(|(identity, placed)| (identity, *placed))
    }

    /// How many screens are in it.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.places.len()
    }
}

/// One screen's row, as a settings file holds it.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct WrittenScreen {
    /// Who made the screen, for one that says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    make: Option<String>,
    /// What they call it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    /// Its own number, where it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    serial: Option<String>,
    /// The socket it is in, for a screen that says nothing about itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    socket: Option<String>,
    /// Where its top left corner is.
    at: Position,
    /// How large everything on it is drawn, in per cent.
    scale: Scale,
    /// Whether it is the main screen. Absent for every screen that is not.
    #[serde(default, skip_serializing_if = "is_not_the_main_screen")]
    main: bool,
}

/// Whether this screen is not the main one, so that the key is left out.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "the shape serde's skip_serializing_if takes"
)]
fn is_not_the_main_screen(main: &bool) -> bool {
    !*main
}

/// An arrangement, as a settings file holds it.
#[derive(Serialize, Deserialize)]
struct Written {
    /// Every screen of the set, with its place.
    screens: Vec<WrittenScreen>,
}

impl TryFrom<Written> for Arrangement {
    type Error = NotRead;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        let mut places = Vec::with_capacity(written.screens.len());
        for screen in written.screens {
            let identity = match (screen.make, screen.model, screen.serial, screen.socket) {
                (Some(make), Some(model), serial, None) => Identity::Panel(
                    Panel::of(&make, &model, serial.as_deref())
                        .map_err(|refused| NotRead::about(refused.word()))?,
                ),
                (None, None, None, Some(socket)) => Identity::Socket(
                    Socket::named(&socket).map_err(|refused| NotRead::about(refused.word()))?,
                ),
                _ => return Err(NotRead::about(NotArranged::HalfAScreen.word())),
            };
            let placed = Placed::at(screen.at, screen.scale);
            places.push((
                identity,
                if screen.main {
                    placed.as_the_main_screen()
                } else {
                    placed
                },
            ));
        }
        Self::of(places).map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<Arrangement> for Written {
    fn from(arrangement: Arrangement) -> Self {
        Self {
            screens: arrangement
                .places
                .into_iter()
                .map(|(identity, placed)| {
                    let (make, model, serial, socket) = match identity {
                        Identity::Panel(panel) => (
                            Some(panel.make().to_owned()),
                            Some(panel.model().to_owned()),
                            panel.serial().map(str::to_owned),
                            None,
                        ),
                        Identity::Socket(socket) => {
                            (None, None, None, Some(socket.name().to_owned()))
                        }
                    };
                    WrittenScreen {
                        make,
                        model,
                        serial,
                        socket,
                        at: placed.position(),
                        scale: placed.scale(),
                        main: placed.is_the_main_screen(),
                    }
                })
                .collect(),
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
    use crate::testing::{in_english, the_laptop, the_office_screen};

    /// A place that is not the main screen, at this corner.
    fn beside(across: i32) -> Placed {
        Placed::at(Position::at(across, 0), Scale::a_hundred())
    }

    /// An arrangement holds each screen's place and one main screen, and the
    /// set it is remembered under does not care what order they came in.
    #[test]
    fn an_arrangement_holds_a_place_each_and_one_main_screen() {
        let laptop = the_laptop();
        let office = the_office_screen();
        let arrangement = Arrangement::of(vec![
            (laptop.clone(), beside(0).as_the_main_screen()),
            (office.clone(), beside(-1920)),
        ])
        .unwrap();
        assert_eq!(arrangement.main_screen(), &laptop);
        assert_eq!(arrangement.how_many(), 2);
        assert_eq!(
            arrangement.placed(&office).unwrap().position(),
            Position::at(-1920, 0)
        );

        let other_way_round = Arrangement::of(vec![
            (office, beside(-1920)),
            (laptop, beside(0).as_the_main_screen()),
        ])
        .unwrap();
        assert_eq!(arrangement.screens(), other_way_round.screens());
    }

    /// **An arrangement with no main screen, or two, is refused** — and so is
    /// one with no screens or the same screen twice. Each says which.
    #[test]
    fn an_arrangement_that_could_not_be_used_is_refused_and_says_which() {
        let strings = in_english();
        let laptop = the_laptop();
        let office = the_office_screen();

        assert_eq!(Arrangement::of(Vec::new()), Err(NotArranged::NoScreens));
        assert_eq!(
            Arrangement::of(vec![(laptop.clone(), beside(0))]),
            Err(NotArranged::NoMainScreen)
        );
        assert_eq!(
            Arrangement::of(vec![
                (laptop.clone(), beside(0).as_the_main_screen()),
                (office.clone(), beside(1920).as_the_main_screen()),
            ]),
            Err(NotArranged::MoreThanOneMainScreen)
        );
        assert_eq!(
            Arrangement::of(vec![
                (laptop.clone(), beside(0).as_the_main_screen()),
                (laptop.clone(), beside(1920)),
            ]),
            Err(NotArranged::TheSameScreenTwice(laptop))
        );

        for refused in [
            NotArranged::NoScreens,
            NotArranged::NoMainScreen,
            NotArranged::MoreThanOneMainScreen,
            NotArranged::TheSameScreenTwice(office),
            NotArranged::HalfAScreen,
        ] {
            let said = refused.said(&strings);
            assert!(words::EVERY_WORD.contains(&refused.word()), "{refused:?}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            assert!(!said.is_a_bug(), "{refused:?}: {said}");
        }
    }

    /// **An arrangement survives being written down and read back**, screen
    /// descriptions, sockets, places, sizes and the main screen alike.
    #[test]
    fn an_arrangement_survives_being_written_down() {
        let arrangement = Arrangement::of(vec![
            (the_laptop(), beside(0).as_the_main_screen()),
            (
                the_office_screen(),
                Placed::at(Position::at(-2560, -120), Scale::per_cent(150).unwrap()),
            ),
        ])
        .unwrap();
        let written = serde_json::to_string(&arrangement).unwrap();
        assert_eq!(
            serde_json::from_str::<Arrangement>(&written).unwrap(),
            arrangement
        );
        assert!(written.contains("\"socket\":\"eDP-1\""), "{written}");
        assert!(written.contains("\"model\":\"U2720Q\""), "{written}");
        assert!(written.contains("\"main\":true"), "{written}");
        assert_eq!(written.matches("\"main\"").count(), 1, "{written}");
    }

    /// **A file that names a screen as neither a description nor a socket is
    /// refused**, and so is one with no main screen — both write the key of the
    /// sentence a person would be shown.
    #[test]
    fn a_file_that_is_not_an_arrangement_is_refused_with_a_key() {
        let half = r#"{"screens":[{"make":"Dell","at":[0,0],"scale":100,"main":true}]}"#;
        let refused = serde_json::from_str::<Arrangement>(half).unwrap_err();
        assert!(
            refused.to_string().contains("displays.half-a-screen"),
            "{refused}"
        );

        let leaderless = r#"{"screens":[{"socket":"eDP-1","at":[0,0],"scale":100}]}"#;
        let refused = serde_json::from_str::<Arrangement>(leaderless).unwrap_err();
        assert!(
            refused.to_string().contains("displays.no-main-screen"),
            "{refused}"
        );

        let both = r#"{"screens":[{"socket":"eDP-1","make":"Dell","model":"U2720Q","at":[0,0],"scale":100,"main":true}]}"#;
        let refused = serde_json::from_str::<Arrangement>(both).unwrap_err();
        assert!(
            refused.to_string().contains("displays.half-a-screen"),
            "{refused}"
        );
    }

    /// A set is the screens and nothing else, and a screen given twice is
    /// refused there too.
    #[test]
    fn a_set_is_the_screens_and_nothing_else() {
        let laptop = the_laptop();
        let office = the_office_screen();
        let set = Screens::of([laptop.clone(), office.clone()]).unwrap();
        assert_eq!(set.how_many(), 2);
        assert!(set.holds(&laptop));
        assert_eq!(
            Screens::of([office.clone(), laptop]).unwrap(),
            set,
            "the order they were plugged in is not part of the set"
        );
        assert_eq!(
            Screens::of([office.clone(), office.clone()]),
            Err(NotArranged::TheSameScreenTwice(office))
        );
        assert_eq!(Screens::of([]), Err(NotArranged::NoScreens));
    }
}
