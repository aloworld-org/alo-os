//! One display's desktops: the row of them, which one a person is on, and every
//! change a person makes to it.
//!
//! **A display always has at least one desktop.** A person who has never heard
//! of virtual desktops has one, uses it, and never learns the word; adding a
//! second is what makes the row visible. So this type is made with one
//! ([`crate::Desktops::plug_in`]), removing the last one is refused
//! ([`Refused::TheLastDesktop`]), and there is no state in which a display has
//! nowhere to put a window.
//!
//! **Removing a desktop never closes a window.** Its windows go to the
//! neighbour — the desktop after it, or the one before it when it was last —
//! and a person who removes the desktop they were on lands on that same
//! neighbour, beside the windows they had. They arrive **floating**: the
//! neighbour's division is an arrangement that person made, and dropping three
//! windows into it would rearrange windows they were not looking at. Dividing
//! them again is one drag each, which is a hand's work rather than a surprise.
//!
//! # Where a window is
//!
//! On exactly one desktop, or on every desktop at once. Those are the two
//! states, and [`OnADisplay::put_on`] and [`OnADisplay::on_every_desktop`] are
//! the two roads between them. A window on every desktop has **no share in any
//! desktop's division**: a share belongs to one desktop's arrangement, and a
//! window that is everywhere cannot be in one arrangement and not another.
//!
//! The three surfaces of [`crate::Always`] are on every desktop too, and are
//! not windows a person moves: every road that could move one refuses with
//! [`Refused::APromise`]. `always.rs` says why that is structural here rather
//! than a rule somebody has to remember.
//!
//! # Nothing here draws, and nothing here divides
//!
//! The division on each desktop is `alo-dividing`'s, reached through
//! [`crate::Desktop::division_mut`]. Divide a desktop's windows after putting
//! them on it: this crate holds which desktop a window is on, and that crate
//! holds where on the screen it is.

use std::collections::BTreeSet;

use alo_dividing::{Area, WindowId};

use crate::always::{Always, Promises};
use crate::desktop::{Desktop, DesktopId};
use crate::naming::Name;
use crate::position::Position;
use crate::refusing::Refused;
use crate::switching::Switch;

/// The most desktops one display can have.
///
/// Sixteen: more than anybody has been seen to use, and few enough that the row
/// is still a row a person can read and count along. The cap is why *add a
/// desktop* cannot be held down into a list nothing can draw, and why a
/// [`Position`] is always a number a person could count to.
pub const MOST_DESKTOPS: usize = 16;

/// A person's desktops on one display.
///
/// Deliberately not `Clone`, for the reason [`Desktop`] is not: a copy would be
/// a second row of desktops claiming the same windows.
#[derive(Debug, PartialEq, Eq)]
pub struct OnADisplay {
    /// The display's area, in logical units, which every desktop's division is
    /// of.
    display: Area,
    /// The desktops, in the person's order. Never empty.
    desktops: Vec<Desktop>,
    /// The one the person is on.
    current: DesktopId,
    /// The person's windows that are on every desktop of this display.
    everywhere: BTreeSet<WindowId>,
    /// The three surfaces that are on every desktop and cannot be moved.
    promises: Promises,
}

impl OnADisplay {
    /// One display, with one desktop and its three promises.
    pub(crate) fn of(display: Area, promises: Promises) -> Self {
        let first = Desktop::of(display);
        let current = first.id();
        Self {
            display,
            desktops: vec![first],
            current,
            everywhere: BTreeSet::new(),
            promises,
        }
    }

    /// The display's area, in logical units.
    #[must_use]
    pub const fn display(&self) -> Area {
        self.display
    }

    /// Which window is which of the three that are on every desktop.
    #[must_use]
    pub const fn promises(&self) -> Promises {
        self.promises
    }

    /// How many desktops this display has — at least one, and at most
    /// [`MOST_DESKTOPS`].
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.desktops.len()
    }

    /// Every desktop, in the person's order, with the number they read it as.
    pub fn in_order(&self) -> impl Iterator<Item = (Position, &Desktop)> + '_ {
        self.desktops
            .iter()
            .enumerate()
            .filter_map(|(index, desktop)| Position::at(index).map(|at| (at, desktop)))
    }

    /// One desktop, by identity.
    #[must_use]
    pub fn desktop(&self, desktop: DesktopId) -> Option<&Desktop> {
        self.desktops.iter().find(|held| held.id() == desktop)
    }

    /// One desktop, by identity, to change — its division, above all.
    pub fn desktop_mut(&mut self, desktop: DesktopId) -> Option<&mut Desktop> {
        self.desktops.iter_mut().find(|held| held.id() == desktop)
    }

    /// The desktop a person counts as this number.
    #[must_use]
    pub fn at(&self, at: Position) -> Option<&Desktop> {
        self.desktops.get(at.index())
    }

    /// Where a desktop is in the person's order.
    #[must_use]
    pub fn position_of(&self, desktop: DesktopId) -> Option<Position> {
        self.desktops
            .iter()
            .position(|held| held.id() == desktop)
            .and_then(Position::at)
    }

    /// The desktop the person is on.
    #[must_use]
    pub const fn current(&self) -> DesktopId {
        self.current
    }

    /// Add a desktop at the end of the row.
    ///
    /// The person stays where they are: adding a desktop is not going to it, and
    /// a system that moved them would make *add* unusable for preparing one.
    ///
    /// # Errors
    /// [`Refused::TooManyDesktops`] at [`MOST_DESKTOPS`].
    pub fn add(&mut self) -> Result<DesktopId, Refused> {
        if self.desktops.len() >= MOST_DESKTOPS {
            return Err(Refused::TooManyDesktops);
        }
        let desktop = Desktop::of(self.display);
        let id = desktop.id();
        self.desktops.push(desktop);
        Ok(id)
    }

    /// Remove a desktop, giving its windows to its neighbour.
    ///
    /// **No window is closed.** The neighbour that takes them is the desktop
    /// after this one, or the one before it when this was the last; its identity
    /// is the answer, so the shell can show where the windows went. A person
    /// removing the desktop they were on lands on that neighbour.
    ///
    /// # Errors
    /// - [`Refused::NoSuchDesktop`] when this display has no such desktop;
    /// - [`Refused::TheLastDesktop`] when it is the only one.
    pub fn remove(&mut self, desktop: DesktopId) -> Result<DesktopId, Refused> {
        let index = self
            .desktops
            .iter()
            .position(|held| held.id() == desktop)
            .ok_or(Refused::NoSuchDesktop(desktop))?;
        if self.desktops.len() == 1 {
            return Err(Refused::TheLastDesktop);
        }
        let neighbour = if index + 1 < self.desktops.len() {
            index + 1
        } else {
            index - 1
        };
        let windows = self
            .desktops
            .get_mut(index)
            .ok_or(Refused::NoSuchDesktop(desktop))?
            .emptied();
        let taking = self
            .desktops
            .get_mut(neighbour)
            .ok_or(Refused::NoSuchDesktop(desktop))?;
        let took = taking.id();
        for window in windows {
            taking.take(window);
        }
        self.desktops.remove(index);
        if self.current == desktop {
            self.current = took;
        }
        Ok(took)
    }

    /// Call a desktop something, or take its name away with `None`.
    ///
    /// # Errors
    /// - [`Refused::NoSuchDesktop`] when this display has no such desktop;
    /// - [`Refused::NameIsTaken`], naming the desktop that already has it.
    pub fn call_it(&mut self, desktop: DesktopId, name: Option<Name>) -> Result<(), Refused> {
        if self.desktop(desktop).is_none() {
            return Err(Refused::NoSuchDesktop(desktop));
        }
        if let Some(name) = &name
            && let Some(already) = self
                .desktops
                .iter()
                .find(|held| held.id() != desktop && held.name().is_some_and(|held| held == name))
        {
            return Err(Refused::NameIsTaken(already.id()));
        }
        self.desktop_mut(desktop)
            .ok_or(Refused::NoSuchDesktop(desktop))?
            .call_it(name);
        Ok(())
    }

    /// Move a desktop to another place in the person's order.
    ///
    /// The rest close up behind it and open up in front of it, which is what
    /// dragging one along a row does. Nobody's windows move and nobody's current
    /// desktop changes: a reorder is about the row, not about what is on it.
    ///
    /// # Errors
    /// - [`Refused::NoSuchDesktop`] when this display has no such desktop;
    /// - [`Refused::NoSuchPosition`] beyond the desktops there are.
    pub fn reorder(&mut self, desktop: DesktopId, to: Position) -> Result<(), Refused> {
        let index = self
            .desktops
            .iter()
            .position(|held| held.id() == desktop)
            .ok_or(Refused::NoSuchDesktop(desktop))?;
        if to.index() >= self.desktops.len() {
            return Err(Refused::NoSuchPosition(to));
        }
        let moved = self.desktops.remove(index);
        self.desktops.insert(to.index(), moved);
        Ok(())
    }

    /// Go to another desktop.
    ///
    /// The one road there, whether a gesture or a chord asked
    /// ([`crate::Switch`]). Asking for the desktop a person is already on is not
    /// a refusal: it is where they are, and it is the answer.
    ///
    /// # Errors
    /// - [`Refused::NoDesktopThatWay`] past either end of the row, because
    ///   switching does not wrap;
    /// - [`Refused::NoSuchPosition`] for a number this display has no desktop
    ///   at.
    pub fn switch(&mut self, switch: Switch) -> Result<DesktopId, Refused> {
        let arriving = match switch {
            Switch::Next | Switch::Previous => {
                let going = self
                    .position_of(self.current)
                    .and_then(|at| at.along(matches!(switch, Switch::Next)))
                    .ok_or(Refused::NoDesktopThatWay(switch))?;
                self.at(going).ok_or(Refused::NoDesktopThatWay(switch))?
            }
            Switch::To(at) => self.at(at).ok_or(Refused::NoSuchPosition(at))?,
        };
        self.current = arriving.id();
        Ok(self.current)
    }

    /// Put a window on one desktop, and only that one.
    ///
    /// It leaves wherever it was — another desktop, or every desktop — and its
    /// share of that desktop's division with it.
    ///
    /// # Errors
    /// - [`Refused::APromise`] for one of the three that are on every desktop;
    /// - [`Refused::NoSuchDesktop`] when this display has no such desktop.
    pub fn put_on(&mut self, desktop: DesktopId, window: WindowId) -> Result<(), Refused> {
        if let Some(promise) = self.promises.which(window) {
            return Err(Refused::APromise(promise));
        }
        if self.desktop(desktop).is_none() {
            return Err(Refused::NoSuchDesktop(desktop));
        }
        self.let_go_of(window);
        self.desktop_mut(desktop)
            .ok_or(Refused::NoSuchDesktop(desktop))?
            .take(window);
        Ok(())
    }

    /// Put a window on every desktop of this display at once.
    ///
    /// It leaves the desktop it was on, and its share of that desktop's
    /// division: a window that is everywhere is in nobody's arrangement. Asking
    /// twice changes nothing.
    ///
    /// # Errors
    /// [`Refused::APromise`] for one of the three that are already on every
    /// desktop and are not a person's window to pin.
    pub fn on_every_desktop(&mut self, window: WindowId) -> Result<(), Refused> {
        if let Some(promise) = self.promises.which(window) {
            return Err(Refused::APromise(promise));
        }
        self.let_go_of(window);
        self.everywhere.insert(window);
        Ok(())
    }

    /// Whether this window is on every desktop of this display.
    ///
    /// True for the three promises, which are on every desktop by construction.
    #[must_use]
    pub fn is_everywhere(&self, window: WindowId) -> bool {
        self.promises.which(window).is_some() || self.everywhere.contains(&window)
    }

    /// Which desktop a window is on, when it is on exactly one.
    ///
    /// `None` for a window on every desktop, for a promise, and for one this
    /// display has never been told about — [`OnADisplay::is_everywhere`] is what
    /// tells the first two apart from the third.
    #[must_use]
    pub fn where_is(&self, window: WindowId) -> Option<DesktopId> {
        self.desktops
            .iter()
            .find(|desktop| desktop.holds(window))
            .map(Desktop::id)
    }

    /// A window has closed: take it off whatever it was on.
    ///
    /// # Errors
    /// - [`Refused::APromise`] for one of the three that are on every desktop —
    ///   they are alo OS's own surfaces, and this is not the road that ends one;
    /// - [`Refused::NoSuchWindow`] when it was on no desktop of this display.
    pub fn close(&mut self, window: WindowId) -> Result<(), Refused> {
        if let Some(promise) = self.promises.which(window) {
            return Err(Refused::APromise(promise));
        }
        if self.let_go_of(window) {
            return Ok(());
        }
        Err(Refused::NoSuchWindow(window))
    }

    /// Everything a person sees on one desktop: its own windows, the windows on
    /// every desktop, and the three promises.
    ///
    /// In the order the compositor numbered them, and each once. `None` when
    /// this display has no such desktop.
    #[must_use]
    pub fn windows_on(&self, desktop: DesktopId) -> Option<Vec<WindowId>> {
        let held = self.desktop(desktop)?;
        let mut windows: BTreeSet<WindowId> = held.windows().collect();
        windows.extend(self.everywhere.iter().copied());
        windows.extend(self.promises.every_surface());
        Some(windows.into_iter().collect())
    }

    /// The window one of the three promises is drawn in on this desktop.
    ///
    /// The same window on every desktop this display has, and that is the whole
    /// of the answer: there is no road in this crate by which a desktop shows
    /// one of them and another does not. `None` only for a desktop this display
    /// does not have, which is not a desktop anybody is looking at.
    #[must_use]
    pub fn promise_on(&self, desktop: DesktopId, always: Always) -> Option<WindowId> {
        self.desktop(desktop)
            .map(|_| self.promises.the_surface(always))
    }

    /// Take a window off every desktop and out of every division, and say
    /// whether it was anywhere.
    fn let_go_of(&mut self, window: WindowId) -> bool {
        let was_everywhere = self.everywhere.remove(&window);
        let mut was_on_one = false;
        for desktop in &mut self.desktops {
            was_on_one |= desktop.let_go(window);
        }
        was_everywhere || was_on_one
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_display, window};

    /// A display starts with one desktop, the person is on it, and adding one
    /// leaves them where they are.
    #[test]
    fn a_display_starts_with_one_desktop_and_the_person_on_it() {
        let mut display = a_display();
        assert_eq!(display.how_many(), 1);
        let first = display.current();
        assert_eq!(display.at(Position::first()).map(Desktop::id), Some(first));
        assert_eq!(display.position_of(first), Some(Position::first()));

        let second = display.add().unwrap();
        assert_eq!(display.how_many(), 2);
        assert_eq!(display.current(), first, "adding one does not go to it");
        assert_eq!(display.position_of(second).map(Position::number), Some(2));
        assert_eq!(
            display
                .in_order()
                .map(|(at, desktop)| (at.number(), desktop.id()))
                .collect::<Vec<_>>(),
            vec![(1, first), (2, second)]
        );
    }

    /// The cap is a refusal, not a silent stop, and the row is unchanged by it.
    #[test]
    fn a_seventeenth_desktop_is_refused() {
        let mut display = a_display();
        while display.how_many() < MOST_DESKTOPS {
            display.add().unwrap();
        }
        assert_eq!(display.add(), Err(Refused::TooManyDesktops));
        assert_eq!(display.how_many(), MOST_DESKTOPS);
    }

    /// The only desktop cannot be removed, and neither can one this display does
    /// not have.
    #[test]
    fn the_only_desktop_stays() {
        let mut display = a_display();
        let only = display.current();
        assert_eq!(display.remove(only), Err(Refused::TheLastDesktop));
        assert_eq!(display.how_many(), 1);

        let elsewhere = a_display().current();
        assert_eq!(
            display.remove(elsewhere),
            Err(Refused::NoSuchDesktop(elsewhere))
        );
    }

    /// Reordering moves the desktop and nothing else; a place there is no
    /// desktop at is refused.
    #[test]
    fn reordering_moves_the_desktop_and_nothing_else() {
        let mut display = a_display();
        let first = display.current();
        let second = display.add().unwrap();
        let third = display.add().unwrap();
        display.put_on(first, window(1)).unwrap();

        display.reorder(third, Position::first()).unwrap();
        assert_eq!(
            display.in_order().map(|(_, d)| d.id()).collect::<Vec<_>>(),
            vec![third, first, second]
        );
        assert_eq!(display.current(), first, "a reorder moves nobody");
        assert_eq!(display.where_is(window(1)), Some(first));

        let beyond = Position::numbered(4).unwrap();
        assert_eq!(
            display.reorder(third, beyond),
            Err(Refused::NoSuchPosition(beyond))
        );
        assert_eq!(
            display.in_order().map(|(_, d)| d.id()).collect::<Vec<_>>(),
            vec![third, first, second]
        );
    }

    /// Two desktops cannot share a name, and taking a name away frees it.
    #[test]
    fn two_desktops_cannot_share_a_name() {
        let mut display = a_display();
        let first = display.current();
        let second = display.add().unwrap();
        let post = Name::given("Post").unwrap();
        display.call_it(first, Some(post.clone())).unwrap();
        assert_eq!(
            display.call_it(second, Some(post.clone())),
            Err(Refused::NameIsTaken(first))
        );
        // The same desktop keeping its own name is not a clash.
        display.call_it(first, Some(post.clone())).unwrap();
        // And once the first gives it up, the second may have it.
        display.call_it(first, None).unwrap();
        display.call_it(second, Some(post)).unwrap();
        assert_eq!(
            display
                .desktop(second)
                .and_then(Desktop::name)
                .map(Name::as_str),
            Some("Post")
        );

        let elsewhere = a_display().current();
        assert_eq!(
            display.call_it(elsewhere, None),
            Err(Refused::NoSuchDesktop(elsewhere))
        );
    }

    /// Switching does not wrap at either end, and a desktop that is not there
    /// is refused by number.
    #[test]
    fn switching_stops_at_the_ends_of_the_row() {
        let mut display = a_display();
        let first = display.current();
        let second = display.add().unwrap();

        assert_eq!(
            display.switch(Switch::Previous),
            Err(Refused::NoDesktopThatWay(Switch::Previous))
        );
        assert_eq!(display.switch(Switch::Next), Ok(second));
        assert_eq!(
            display.switch(Switch::Next),
            Err(Refused::NoDesktopThatWay(Switch::Next))
        );
        assert_eq!(display.current(), second, "a refused switch moves nobody");
        assert_eq!(display.switch(Switch::Previous), Ok(first));

        let fourth = Position::numbered(4).unwrap();
        assert_eq!(
            display.switch(Switch::To(fourth)),
            Err(Refused::NoSuchPosition(fourth))
        );
        // The desktop a person is already on is where they are, not a refusal.
        assert_eq!(display.switch(Switch::To(Position::first())), Ok(first));
    }

    /// A window is on one desktop or on every one, and moving it off a desktop
    /// takes its share of that desktop's division with it.
    #[test]
    fn a_window_is_on_one_desktop_or_on_every_one() {
        let mut display = a_display();
        let first = display.current();
        let second = display.add().unwrap();
        display.put_on(first, window(1)).unwrap();
        display.put_on(first, window(2)).unwrap();
        assert_eq!(display.where_is(window(1)), Some(first));

        display.put_on(second, window(1)).unwrap();
        assert_eq!(display.where_is(window(1)), Some(second));
        assert!(!display.desktop(first).unwrap().holds(window(1)));

        display.on_every_desktop(window(1)).unwrap();
        assert!(display.is_everywhere(window(1)));
        assert_eq!(display.where_is(window(1)), None);
        // Twice changes nothing.
        display.on_every_desktop(window(1)).unwrap();
        assert!(display.windows_on(second).unwrap().contains(&window(1)));
        assert!(display.windows_on(first).unwrap().contains(&window(1)));

        // And back onto one desktop, which takes it off every other.
        display.put_on(first, window(1)).unwrap();
        assert!(!display.is_everywhere(window(1)));
        assert!(!display.windows_on(second).unwrap().contains(&window(1)));

        let elsewhere = a_display().current();
        assert_eq!(
            display.put_on(elsewhere, window(3)),
            Err(Refused::NoSuchDesktop(elsewhere))
        );
        assert_eq!(display.windows_on(elsewhere), None);
    }

    /// A window that closed is taken off whatever it was on; one that was on
    /// nothing is refused.
    #[test]
    fn a_closed_window_leaves_and_one_that_was_never_here_is_refused() {
        let mut display = a_display();
        let first = display.current();
        display.put_on(first, window(1)).unwrap();
        display.on_every_desktop(window(2)).unwrap();

        display.close(window(1)).unwrap();
        assert_eq!(display.where_is(window(1)), None);
        display.close(window(2)).unwrap();
        assert!(!display.is_everywhere(window(2)));
        assert_eq!(
            display.close(window(1)),
            Err(Refused::NoSuchWindow(window(1)))
        );
    }
}
