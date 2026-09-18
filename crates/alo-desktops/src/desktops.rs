//! Every display's desktops, and the person's chords that move between them.
//!
//! **Desktops are per display, and independently so.** The laptop's own screen
//! and an external one have their own desktops, their own order, their own
//! names and their own current desktop; switching on one changes nothing on the
//! other. That is what `docs/features.md` promises for a division at v0.5 —
//! *splitting works on an external display independently of the laptop's own* —
//! carried up to the desktop a division is on, and the test for it is that two
//! displays' rows never move together.
//!
//! **The chords are the person's, not a display's.** A person has one keyboard,
//! and *next desktop* means the next desktop of the screen they are on. So
//! [`DesktopChords`] is held here once, and the display is named where the
//! switch is carried out ([`crate::OnADisplay::switch`]).
//!
//! # A display that goes, and what task 2 will do with it
//!
//! [`Desktops::unplug`] hands back the whole of a display's desktops rather than
//! dropping them, because a display that is unplugged comes back. What is not
//! here is keeping them: remembering an arrangement across a session needs the
//! stable identity of a display that `alo-displays` gives and a
//! [`crate::DisplayId`] deliberately is not, and that is task 2 of this crate's
//! plan. Until then this type holds a session's desktops and hands them over
//! whole, which is what whoever remembers them needs to be given.

use std::collections::BTreeMap;

use alo_dividing::Area;

use crate::always::Promises;
use crate::chords::DesktopChords;
use crate::display::{DisplayId, NotADisplay};
use crate::on_a_display::OnADisplay;

/// A person's desktops, on every display they have.
#[derive(Debug, PartialEq, Eq)]
pub struct Desktops {
    /// Each display's own row of desktops.
    displays: BTreeMap<DisplayId, OnADisplay>,
    /// The chords that move between them, which are the person's and not a
    /// display's.
    chords: DesktopChords,
}

impl Desktops {
    /// No displays yet, with these chords — what a shell has before the
    /// compositor has told it about a screen.
    #[must_use]
    pub fn with(chords: DesktopChords) -> Self {
        Self {
            displays: BTreeMap::new(),
            chords,
        }
    }

    /// A display has appeared: one desktop on it, and its three promises.
    ///
    /// # Errors
    /// [`NotADisplay::AlreadyThere`] when that display already has desktops.
    /// Plugging it in again would discard them, and a compositor that says a
    /// screen appeared twice is a bug this crate will not paper over by losing
    /// somebody's windows.
    pub fn plug_in(
        &mut self,
        display: DisplayId,
        area: Area,
        promises: Promises,
    ) -> Result<&mut OnADisplay, NotADisplay> {
        if self.displays.contains_key(&display) {
            return Err(NotADisplay::AlreadyThere(display));
        }
        Ok(self
            .displays
            .entry(display)
            .or_insert_with(|| OnADisplay::of(area, promises)))
    }

    /// A display has gone: its desktops, handed back whole.
    ///
    /// Nothing is closed and nothing is moved. What the caller does with them is
    /// the caller's — task 2 of this crate's plan is what keeps them to restore
    /// when the display returns.
    ///
    /// # Errors
    /// [`NotADisplay::Unknown`] when no desktops are held for that display.
    pub fn unplug(&mut self, display: DisplayId) -> Result<OnADisplay, NotADisplay> {
        self.displays
            .remove(&display)
            .ok_or(NotADisplay::Unknown(display))
    }

    /// One display's desktops.
    #[must_use]
    pub fn on(&self, display: DisplayId) -> Option<&OnADisplay> {
        self.displays.get(&display)
    }

    /// One display's desktops, to change.
    pub fn on_mut(&mut self, display: DisplayId) -> Option<&mut OnADisplay> {
        self.displays.get_mut(&display)
    }

    /// Every display that has desktops, in the order the compositor numbered
    /// them.
    pub fn displays(&self) -> impl Iterator<Item = DisplayId> + '_ {
        self.displays.keys().copied()
    }

    /// The chords that move between desktops.
    #[must_use]
    pub const fn chords(&self) -> &DesktopChords {
        &self.chords
    }

    /// The chords that move between desktops, to change.
    pub const fn chords_mut(&mut self) -> &mut DesktopChords {
        &mut self.chords
    }
}

impl Default for Desktops {
    /// No displays, and the chords alo OS ships — or none of them, if the
    /// shipped list could not be built, which [`DesktopChords::as_shipped`]'s
    /// own test says cannot happen.
    fn default() -> Self {
        Self::with(DesktopChords::as_shipped().unwrap_or_else(|_| DesktopChords::none()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{display, promises, second_promises};
    use crate::{Refused, Switch};

    /// A display appears, is found again, and appearing twice is refused rather
    /// than discarding what it had.
    #[test]
    fn a_display_appears_once() {
        let mut desktops = Desktops::default();
        let laptop = DisplayId::from_compositor(1);
        assert!(desktops.on(laptop).is_none());
        let added = desktops
            .plug_in(laptop, display(), promises())
            .unwrap()
            .add()
            .unwrap();
        assert_eq!(desktops.on(laptop).unwrap().how_many(), 2);
        assert_eq!(
            desktops.plug_in(laptop, display(), promises()).unwrap_err(),
            NotADisplay::AlreadyThere(laptop)
        );
        assert_eq!(desktops.on(laptop).unwrap().how_many(), 2);
        assert!(desktops.on(laptop).unwrap().desktop(added).is_some());
        assert_eq!(desktops.displays().collect::<Vec<_>>(), vec![laptop]);
    }

    /// A display that goes hands its desktops back whole, and going twice is
    /// refused.
    #[test]
    fn a_display_that_goes_hands_its_desktops_back() {
        let mut desktops = Desktops::default();
        let external = DisplayId::from_compositor(2);
        desktops
            .plug_in(external, display(), promises())
            .unwrap()
            .add()
            .unwrap();
        let had = desktops.unplug(external).unwrap();
        assert_eq!(had.how_many(), 2);
        assert!(desktops.on(external).is_none());
        assert_eq!(
            desktops.unplug(external).unwrap_err(),
            NotADisplay::Unknown(external)
        );
        assert!(desktops.displays().next().is_none());
    }

    /// **Two displays divide their desktops independently**: adding, switching
    /// and reordering on one changes nothing on the other.
    #[test]
    fn two_displays_keep_their_own_desktops() {
        let mut desktops = Desktops::default();
        let laptop = DisplayId::from_compositor(1);
        let external = DisplayId::from_compositor(2);
        desktops.plug_in(laptop, display(), promises()).unwrap();
        desktops
            .plug_in(external, display(), second_promises())
            .unwrap();

        let on_laptop = desktops.on(laptop).unwrap().current();
        desktops.on_mut(laptop).unwrap().add().unwrap();
        let second = desktops
            .on_mut(laptop)
            .unwrap()
            .switch(Switch::Next)
            .unwrap();

        assert_eq!(desktops.on(laptop).unwrap().how_many(), 2);
        assert_eq!(desktops.on(laptop).unwrap().current(), second);
        assert_ne!(second, on_laptop);

        assert_eq!(desktops.on(external).unwrap().how_many(), 1);
        assert_eq!(
            desktops.on_mut(external).unwrap().switch(Switch::Next),
            Err(Refused::NoDesktopThatWay(Switch::Next)),
            "the external screen has one desktop of its own"
        );

        // And a display nobody plugged in is nobody's desktops.
        assert!(desktops.on(DisplayId::from_compositor(3)).is_none());
        assert!(desktops.on_mut(DisplayId::from_compositor(3)).is_none());
    }

    /// The chords are the person's and outlive any one display.
    #[test]
    fn the_chords_belong_to_the_person() {
        let mut desktops = Desktops::default();
        assert_eq!(desktops.chords().bound().count(), 11);
        assert!(desktops.chords_mut().unbind(Switch::Next));
        assert_eq!(desktops.chords().bound().count(), 10);
        assert_eq!(
            Desktops::with(DesktopChords::none())
                .chords()
                .bound()
                .count(),
            0
        );
    }
}
