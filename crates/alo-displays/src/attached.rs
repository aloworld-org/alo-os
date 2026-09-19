//! The screens plugged in at this moment, laid out the way the person left
//! this set of them.
//!
//! This is where everything else in the crate meets the machine. An
//! [`Arrangement`] is what a person decided; [`Reported`] is what is actually
//! plugged in; [`Attached`] is the two put together, and the only thing in this
//! crate that changes when somebody pulls a cable out.
//!
//! # What is decided here, in the order it is decided
//!
//! 1. **Which screens these are** — over the whole set, so two identical
//!    monitors are told apart ([`crate::whoever_is_attached`]).
//! 2. **Whether this set has been arranged before.** If it has, and the
//!    arrangement still fits the screens as they report themselves now, it is
//!    used unchanged. *Docking at the office restores the office's layout*
//!    is this clause and nothing more.
//! 3. **Otherwise they are laid out side by side, left to right**, in the order
//!    the machine reported them, with the first as the main screen — and each
//!    at the size it was last given on this machine, or, for a screen nobody
//!    has ever sized, at a size worked out from its own glass
//!    ([`crate::Scale::worked_out_for`]).
//! 4. **What the machine can actually draw** is applied last
//!    ([`crate::Support`]), so an arrangement holds what the person chose and a
//!    screen shows the nearest thing to it that exists.
//!
//! # An arrangement that no longer fits is not forced on
//!
//! A screen whose resolution changed — a projector renegotiating, a monitor
//! switched to another input and back — can turn a saved arrangement into two
//! screens drawn over each other, which is a machine with a window nobody can
//! reach. So the saved places are checked against the pixels reported **now**,
//! and an arrangement that would overlap, or that has no place for a screen
//! that is plugged in, is set aside for one worked out, with
//! [`Note::DidNotFit`] said. The saved arrangement is not deleted: the next
//! time those screens report themselves as they did, it fits again.

use crate::arrangement::{Arrangement, NotArranged, Screens};
use crate::changes::Changes;
use crate::coming_and_going::{CameBack, Moved, NotAttached};
use crate::identity::{Identity, Socket};
use crate::notes::Note;
use crate::placed::{Placed, Position};
use crate::reported::{Reported, which_screens_these_are};
use crate::scale::{Rounded, Scale, Support};

/// One screen as it is plugged in now, in the place the arrangement gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnScreen {
    /// What the machine reports about it.
    reported: Reported,
    /// Which screen it is.
    identity: Identity,
    /// Where it sits and the size it was given.
    placed: Placed,
    /// The size that was given rounded to what the machine can draw, when
    /// those are not the same.
    rounded: Option<Rounded>,
}

impl OnScreen {
    /// What the machine reports about it.
    #[must_use]
    pub const fn reported(&self) -> &Reported {
        &self.reported
    }

    /// Which screen it is.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Which socket it is in.
    #[must_use]
    pub const fn socket(&self) -> &Socket {
        self.reported.socket()
    }

    /// Where it sits, and the size it was given.
    #[must_use]
    pub const fn placed(&self) -> Placed {
        self.placed
    }

    /// The size it is actually drawn at, which is the size it was given unless
    /// the machine cannot draw that one.
    #[must_use]
    pub const fn drawn_at(&self) -> Scale {
        match self.rounded {
            Some(rounded) => rounded.used(),
            None => self.placed.scale(),
        }
    }

    /// The size it was given and the size it is drawn at, when those are not
    /// the same.
    #[must_use]
    pub const fn rounded(&self) -> Option<Rounded> {
        self.rounded
    }
}

/// The screens plugged in at this moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attached {
    /// Every screen, in the order the machine reported them.
    on: Vec<OnScreen>,
    /// The arrangement in force, which is what would be remembered.
    arrangement: Arrangement,
    /// Screens that have been unplugged, and where what was on each is
    /// sitting: the screen that went, then the screen it went to.
    away: Vec<(Identity, Identity)>,
    /// What this machine can draw.
    support: Support,
    /// What a person is told about this set.
    notes: Vec<Note>,
}

impl Attached {
    /// The screens plugged in now, laid out.
    ///
    /// # Errors
    /// [`NotArranged::NoScreens`] when nothing is plugged in, which is a
    /// machine with nothing to draw on rather than an arrangement.
    pub fn now(
        reported: Vec<Reported>,
        remembered: &Changes,
        support: Support,
    ) -> Result<Self, NotArranged> {
        let (on, arrangement, notes) = settled(reported, remembered, support)?;
        Ok(Self {
            on,
            arrangement,
            away: Vec::new(),
            support,
            notes,
        })
    }

    /// The set these screens make, which is what an arrangement for them is
    /// remembered under.
    #[must_use]
    pub fn screens(&self) -> Screens {
        self.arrangement.screens()
    }

    /// The arrangement in force, which is what [`Changes::remember`] would be
    /// handed.
    #[must_use]
    pub const fn arrangement(&self) -> &Arrangement {
        &self.arrangement
    }

    /// The screen a new window opens on.
    #[must_use]
    pub const fn main_screen(&self) -> &Identity {
        self.arrangement.main_screen()
    }

    /// Every screen, in the order the machine reported them.
    pub fn each(&self) -> impl Iterator<Item = &OnScreen> {
        self.on.iter()
    }

    /// One screen, if it is plugged in.
    #[must_use]
    pub fn on(&self, screen: &Identity) -> Option<&OnScreen> {
        self.on.iter().find(|held| &held.identity == screen)
    }

    /// The screen in this socket, if there is one.
    #[must_use]
    pub fn in_socket(&self, socket: &Socket) -> Option<&OnScreen> {
        self.on.iter().find(|held| held.reported.socket() == socket)
    }

    /// What a person is told about how these screens were set up.
    #[must_use]
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// Every screen that has been unplugged and whose windows are sitting
    /// somewhere else: the screen that went, then the one they are on.
    pub fn windows_away(&self) -> impl Iterator<Item = (&Identity, &Identity)> {
        self.away.iter().map(|(from, onto)| (from, onto))
    }

    /// A screen has been unplugged. Says where what was on it belongs.
    ///
    /// The whole set is laid out again afterwards, because the screens
    /// remaining are a different set and may be one the person has arranged.
    ///
    /// # Errors
    /// [`NotAttached::NotPluggedIn`] for a socket with no screen in it,
    /// [`NotAttached::NothingRemains`] when it is the only screen the machine
    /// has — what is on it would have nowhere to go, so nothing is changed and
    /// the screen stays attached — and [`NotAttached::TheyDoNotArrange`] for
    /// what is left.
    pub fn unplugged(
        &mut self,
        socket: &Socket,
        remembered: &Changes,
    ) -> Result<Moved, NotAttached> {
        let gone = self
            .in_socket(socket)
            .ok_or(NotAttached::NotPluggedIn)?
            .identity
            .clone();
        if self.on.len() < 2 {
            return Err(NotAttached::NothingRemains);
        }
        let remaining: Vec<Reported> = self
            .on
            .iter()
            .filter(|held| held.reported.socket() != socket)
            .map(|held| held.reported.clone())
            .collect();
        self.resettle(remaining, remembered)?;

        let onto = self.arrangement.main_screen().clone();
        let carrying: Vec<Identity> = self
            .away
            .iter()
            .filter(|(_, sitting)| sitting == &gone)
            .map(|(from, _)| from.clone())
            .collect();
        for (_, sitting) in &mut self.away {
            if *sitting == gone {
                *sitting = onto.clone();
            }
        }
        self.away.push((gone.clone(), onto.clone()));
        Ok(Moved::of(gone, onto, carrying))
    }

    /// A screen has been plugged in. Says what goes back to it.
    ///
    /// # Errors
    /// [`NotAttached::AlreadyPluggedIn`] for a socket that already has a screen
    /// in it, and [`NotAttached::TheyDoNotArrange`] for the set this makes.
    pub fn plugged_in(
        &mut self,
        arriving: Reported,
        remembered: &Changes,
    ) -> Result<CameBack, NotAttached> {
        if self.in_socket(arriving.socket()).is_some() {
            return Err(NotAttached::AlreadyPluggedIn);
        }
        let mut reported: Vec<Reported> =
            self.on.iter().map(|held| held.reported.clone()).collect();
        reported.push(arriving);
        self.resettle(reported, remembered)?;

        let Some(back) = self.on.last().map(|held| held.identity.clone()) else {
            return Err(NotAttached::TheyDoNotArrange(NotArranged::NoScreens));
        };
        let were_on = self
            .away
            .iter()
            .position(|(from, _)| from == &back)
            .map(|at| self.away.remove(at).1)
            .filter(|sitting| self.on(sitting).is_some());
        Ok(CameBack::of(back, were_on))
    }

    /// These screens, laid out again, keeping what is away from home.
    fn resettle(
        &mut self,
        reported: Vec<Reported>,
        remembered: &Changes,
    ) -> Result<(), NotAttached> {
        let (on, arrangement, notes) =
            settled(reported, remembered, self.support).map_err(NotAttached::TheyDoNotArrange)?;
        self.on = on;
        self.arrangement = arrangement;
        self.notes = notes;
        Ok(())
    }
}

/// These screens, laid out: every one in its place, the arrangement in force,
/// and what a person is told about it.
fn settled(
    reported: Vec<Reported>,
    remembered: &Changes,
    support: Support,
) -> Result<(Vec<OnScreen>, Arrangement, Vec<Note>), NotArranged> {
    let identities = which_screens_these_are(&reported);
    let screens = Screens::of(identities.iter().cloned())?;
    let mut notes = Vec::new();
    if two_of_them_say_the_same(&reported) {
        notes.push(Note::ToldApartByTheirSockets);
    }

    let left = remembered.for_screens(&screens);
    let places = match left.and_then(|left| still_fits(left, &reported, &identities, support)) {
        Some(places) => {
            notes.push(Note::AsYouLeftThem);
            places
        }
        None => {
            if left.is_some() {
                notes.push(Note::DidNotFit);
            }
            for (identity, screen) in identities.iter().zip(&reported) {
                if remembered.the_size_last_chosen_for(identity).is_none() {
                    notes.push(Note::NewHere(identity.clone()));
                    if screen.says().is_none() {
                        notes.push(Note::RememberedByItsSocket(identity.clone()));
                    }
                }
            }
            side_by_side(&reported, &identities, remembered, support)
        }
    };

    let arrangement = Arrangement::of(places.clone())?;
    let mut on = Vec::with_capacity(reported.len());
    for (screen, (identity, placed)) in reported.into_iter().zip(places) {
        let rounded = Rounded::of(placed.scale(), support);
        if let Some(rounded) = rounded {
            notes.push(Note::SizeRounded {
                display: identity.clone(),
                rounded,
            });
        }
        on.push(OnScreen {
            reported: screen,
            identity,
            placed,
            rounded,
        });
    }
    Ok((on, arrangement, notes))
}

/// Whether two of these screens say exactly the same thing about themselves.
fn two_of_them_say_the_same(reported: &[Reported]) -> bool {
    let mut rest = reported;
    while let Some((first, others)) = rest.split_first() {
        if first.says().is_some() && others.iter().any(|other| other.says() == first.says()) {
            return true;
        }
        rest = others;
    }
    false
}

/// This arrangement, if it still fits the screens as they report themselves
/// now: a place for every one of them, and no two of them drawn over each
/// other.
fn still_fits(
    left: &Arrangement,
    reported: &[Reported],
    identities: &[Identity],
    support: Support,
) -> Option<Vec<(Identity, Placed)>> {
    let mut places = Vec::with_capacity(identities.len());
    for identity in identities {
        places.push((identity.clone(), left.placed(identity)?));
    }
    let drawn: Vec<(Placed, crate::reported::Resolution)> = places
        .iter()
        .zip(reported)
        .map(|((_, placed), screen)| {
            (
                placed.at_this_size(support.nearest(placed.scale())),
                screen.pixels(),
            )
        })
        .collect();
    let mut rest = drawn.as_slice();
    while let Some((first, others)) = rest.split_first() {
        if others
            .iter()
            .any(|other| first.0.overlaps(first.1, other.0, other.1))
        {
            return None;
        }
        rest = others;
    }
    Some(places)
}

/// These screens side by side, left to right, in the order the machine
/// reported them, the first one the main screen.
fn side_by_side(
    reported: &[Reported],
    identities: &[Identity],
    remembered: &Changes,
    support: Support,
) -> Vec<(Identity, Placed)> {
    let mut across = 0_i32;
    let mut places = Vec::with_capacity(identities.len());
    for (at, (identity, screen)) in identities.iter().zip(reported).enumerate() {
        let chosen = remembered
            .the_size_last_chosen_for(identity)
            .unwrap_or_else(|| Scale::worked_out_for(screen.pixels(), screen.millimetres()));
        let placed = Placed::at(Position::at(across, 0), chosen);
        across = across
            .saturating_add_unsigned(support.nearest(chosen).laid_out(screen.pixels().width()));
        places.push((
            identity.clone(),
            if at == 0 {
                placed.as_the_main_screen()
            } else {
                placed
            },
        ));
    }
    places
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{
        a_reported_home_screen, a_reported_laptop, a_reported_office_screen, an_arrangement,
        the_home_screen, the_laptop, the_office_screen,
    };

    /// **A set nobody has arranged is laid out side by side, the first screen
    /// the main one**, and each screen at a size worked out from its own glass
    /// rather than at 100%.
    #[test]
    fn a_set_nobody_has_arranged_is_laid_out_side_by_side() {
        let attached = Attached::now(
            vec![a_reported_laptop(), a_reported_office_screen()],
            &Changes::untouched(),
            Support::Fractional,
        )
        .unwrap();

        assert_eq!(attached.main_screen(), &the_laptop());
        let laptop = attached.on(&the_laptop()).unwrap();
        assert_eq!(laptop.placed().position(), Position::the_origin());
        assert_eq!(laptop.placed().scale(), Scale::per_cent(175).unwrap());

        let office = attached.on(&the_office_screen()).unwrap();
        assert_eq!(office.placed().scale(), Scale::per_cent(175).unwrap());
        assert_eq!(
            office.placed().position(),
            Position::at(
                i32::try_from(Scale::per_cent(175).unwrap().laid_out(1920)).unwrap(),
                0,
            ),
            "beside the laptop, at the width the laptop takes once it is drawn"
        );
        assert!(attached.notes().contains(&Note::NewHere(the_laptop())));
        assert!(
            attached
                .notes()
                .contains(&Note::RememberedByItsSocket(the_laptop())),
            "the laptop's own panel says nothing about itself, and the person is told"
        );
    }

    /// **An arrangement is remembered per set of screens**, and this is the
    /// clause that decides it: the same laptop with two different second
    /// screens is two arrangements, each restored on its own.
    #[test]
    fn each_set_of_screens_is_restored_on_its_own() {
        let mut remembered = Changes::untouched();
        remembered.remember(an_arrangement(&[
            (the_laptop(), 0, 175),
            (the_office_screen(), -2560, 150),
        ]));
        remembered.remember(an_arrangement(&[
            (the_laptop(), 0, 175),
            (the_home_screen(), 1920, 100),
        ]));

        let office = Attached::now(
            vec![a_reported_laptop(), a_reported_office_screen()],
            &remembered,
            Support::Fractional,
        )
        .unwrap();
        assert!(office.notes().contains(&Note::AsYouLeftThem));
        assert_eq!(
            office.on(&the_office_screen()).unwrap().placed().position(),
            Position::at(-2560, 0)
        );

        let home = Attached::now(
            vec![a_reported_laptop(), a_reported_home_screen()],
            &remembered,
            Support::Fractional,
        )
        .unwrap();
        assert!(home.notes().contains(&Note::AsYouLeftThem));
        assert_eq!(
            home.on(&the_home_screen()).unwrap().placed().position(),
            Position::at(1920, 0)
        );
    }

    /// **An arrangement that would draw two screens over each other is set
    /// aside**, and the person is told rather than handed a window they cannot
    /// reach. The arrangement itself is kept for the day it fits again.
    #[test]
    fn an_arrangement_that_no_longer_fits_is_set_aside_and_said() {
        let mut remembered = Changes::untouched();
        let office = an_arrangement(&[(the_laptop(), 0, 100), (the_office_screen(), 1000, 100)]);
        remembered.remember(office.clone());

        let attached = Attached::now(
            vec![a_reported_laptop(), a_reported_office_screen()],
            &remembered,
            Support::Fractional,
        )
        .unwrap();
        assert!(attached.notes().contains(&Note::DidNotFit));
        assert!(!attached.notes().contains(&Note::AsYouLeftThem));
        assert_eq!(
            attached.on(&the_laptop()).unwrap().placed().position(),
            Position::the_origin()
        );
        assert_eq!(
            remembered.for_screens(&office.screens()),
            Some(&office),
            "what the person chose is not thrown away because a screen changed size"
        );
    }

    /// **A size the machine cannot draw is rounded, and the person is told
    /// which two sizes are involved.**
    #[test]
    fn a_size_the_machine_cannot_draw_is_rounded_and_said() {
        let mut remembered = Changes::untouched();
        remembered.remember(an_arrangement(&[
            (the_laptop(), 0, 150),
            (the_office_screen(), 2560, 100),
        ]));
        let attached = Attached::now(
            vec![a_reported_laptop(), a_reported_office_screen()],
            &remembered,
            Support::WholeMultiplesOnly,
        )
        .unwrap();
        let laptop = attached.on(&the_laptop()).unwrap();
        assert_eq!(laptop.placed().scale(), Scale::per_cent(150).unwrap());
        assert_eq!(laptop.drawn_at(), Scale::per_cent(200).unwrap());
        assert!(
            attached
                .notes()
                .iter()
                .any(|note| matches!(note, Note::SizeRounded { .. }))
        );

        let office = attached.on(&the_office_screen()).unwrap();
        assert_eq!(office.rounded(), None);
        assert_eq!(office.drawn_at(), Scale::a_hundred());
    }

    /// **A machine with nothing plugged into it is not an arrangement.**
    #[test]
    fn a_machine_with_no_screens_is_refused() {
        assert_eq!(
            Attached::now(Vec::new(), &Changes::untouched(), Support::Fractional),
            Err(NotArranged::NoScreens)
        );
    }
}
