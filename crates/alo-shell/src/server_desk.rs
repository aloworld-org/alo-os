//! What is on each display: how it is divided, and the desktops a person has
//! on it.
//!
//! # This holds and shows; it decides nothing
//!
//! How a display is divided is `alo-dividing`'s answer, which desktop a swipe
//! moves to is `alo-desktops`', and what a division looks like when a display
//! comes back is `alo_dividing::Remembered::restored`'s. Every one of those
//! crates already holds its own state and its own lifecycle. What was missing
//! was somewhere for a **session** to keep them across windows opening and
//! closing and displays arriving and leaving, and that is all this is.
//!
//! A second answer to any of those questions would be the fault the shell
//! plan's constraint is about, one level up from the one it names: two layout
//! deciders is bad because a window would be in two places, and two desktop
//! deciders is bad because a person's swipe would go two ways.
//!
//! # A display that leaves is remembered, and one that returns is restored
//!
//! Unplugging hands back what was on it — nothing is closed and nothing is
//! moved, which is `alo-desktops`' own rule — and the division is written into
//! the remembered set before it goes. When the display returns, the division
//! comes back **under the window numbers that are open now**: an application
//! that is not running is left out and its share collapses onto what is left,
//! so coming back with one of two applications gives that one the whole display
//! rather than half of it and a hole. That is `Remembered::restored`'s
//! behaviour and this file only hands it the windows.

use std::collections::BTreeMap;

use alo_desktops::{Desktops, DisplayId, NotADisplay, Promises, Switch};
use alo_dividing::{
    Area, Division, HeldBy,
    remembering::{Divisions, Remembered},
};

/// Everything a session holds about its displays.
///
/// Empty until a display arrives: a session with no display has no desktops and
/// no division, which is a real state rather than a missing one — it is what a
/// machine is between the sign-in screen ending and the first frame.
#[derive(Debug, Default)]
pub(crate) struct Desk {
    /// The desktops, as `alo-desktops` holds them.
    desktops: Desktops,
    /// How each display is divided now.
    dividing: BTreeMap<DisplayId, Division>,
    /// What each display's division was when it last went away, keyed the way
    /// `alo-dividing` keys them.
    remembered: Divisions,
}

impl Desk {
    /// Nothing on any display yet.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// A display has arrived: its desktops begin and its division is restored.
    ///
    /// `named` is what `alo-dividing` remembers this display by — a screen's
    /// own written form, so the same screen returning finds what it left.
    /// `open` answers which window a remembered share is held by now, and a
    /// share whose application is not running is left out.
    ///
    /// The division is `None` when nothing was remembered for this display, or
    /// when none of what it remembered is open: a display with no division is
    /// one undivided, not one with an empty tree.
    ///
    /// # Errors
    /// [`NotADisplay::AlreadyThere`] for a display already here. Plugging the
    /// same display in twice is a bug in the caller, not a thing to smooth
    /// over: the second call would take the first's desktops away.
    pub(crate) fn display_arrived(
        &mut self,
        display: DisplayId,
        named: &str,
        area: Area,
        promises: Promises,
        open: &impl Fn(&HeldBy) -> Option<alo_dividing::Window>,
    ) -> Result<(), NotADisplay> {
        self.desktops.plug_in(display, area, promises)?;
        if let Some(division) = self
            .remembered
            .on(named)
            .and_then(|remembered| restored(remembered, area, open))
        {
            self.dividing.insert(display, division);
        }
        Ok(())
    }

    /// A display has gone: its division is remembered and its desktops handed
    /// back.
    ///
    /// Nothing is closed and nothing is moved — what happens to the windows
    /// that were on it is the caller's, which is `alo-desktops`' own rule.
    ///
    /// # Errors
    /// [`NotADisplay::Unknown`] for a display that was not here.
    pub(crate) fn display_left(
        &mut self,
        display: DisplayId,
        named: &str,
        who: &impl Fn(alo_dividing::WindowId) -> Option<HeldBy>,
    ) -> Result<alo_desktops::OnADisplay, NotADisplay> {
        if let Some(division) = self.dividing.remove(&display)
            && let Some(remembered) = Remembered::of(&division, who)
        {
            self.remembered.remember(named, remembered);
        }
        self.desktops.unplug(display)
    }

    /// How this display is divided now, or [`None`] where nothing has divided
    /// it.
    pub(crate) fn dividing(&self, display: DisplayId) -> Option<&Division> {
        self.dividing.get(&display)
    }

    /// Divide a display that nothing had divided, or replace what divides it.
    ///
    /// Taking the whole division rather than a change to it, because what a
    /// division *is* is `alo-dividing`'s to say and a shell that edited one
    /// would be deciding a layout.
    pub(crate) fn divide(&mut self, display: DisplayId, division: Division) {
        self.dividing.insert(display, division);
    }

    /// The desktops on this display.
    pub(crate) fn desktops(&self, display: DisplayId) -> Option<&alo_desktops::OnADisplay> {
        self.desktops.on(display)
    }

    /// Move to another desktop, as `alo-desktops` decides which that is.
    ///
    /// The switch is handed over whole: which desktop is next, whether there is
    /// one, and what a wrap does are that crate's answers and never this
    /// file's.
    ///
    /// # Errors
    /// [`NotADisplay::Unknown`] for a display that is not here; the crate's own
    /// refusal for a switch it will not make.
    pub(crate) fn switch(
        &mut self,
        display: DisplayId,
        switch: Switch,
    ) -> Result<Result<alo_desktops::DesktopId, alo_desktops::Refused>, NotADisplay> {
        let on = self
            .desktops
            .on_mut(display)
            .ok_or(NotADisplay::Unknown(display))?;
        Ok(on.switch(switch))
    }

    /// Whether any display is here at all.
    pub(crate) fn is_empty(&self) -> bool {
        self.desktops.displays().next().is_none()
    }
}

/// A remembered division given back for the windows open now.
fn restored(
    remembered: &alo_dividing::remembering::OnADisplay,
    area: Area,
    open: &impl Fn(&HeldBy) -> Option<alo_dividing::Window>,
) -> Option<Division> {
    remembered.division.restored(area, open)
}

impl crate::Server {
    /// A display has arrived: its desktops begin and its division is restored.
    ///
    /// See [`Desk::display_arrived`]. `named` is what `alo-dividing` remembers
    /// this screen by, so the same screen returning finds what it left.
    ///
    /// # Errors
    /// [`NotADisplay::AlreadyThere`] for a display already here.
    pub fn display_arrived(
        &mut self,
        display: DisplayId,
        named: &str,
        area: Area,
        promises: Promises,
        open: &impl Fn(&HeldBy) -> Option<alo_dividing::Window>,
    ) -> Result<(), NotADisplay> {
        self.desk
            .display_arrived(display, named, area, promises, open)
    }

    /// A display has gone: its division is remembered and its desktops handed
    /// back.
    ///
    /// Nothing is closed and nothing is moved; what happens to the windows that
    /// were on it is the caller's.
    ///
    /// # Errors
    /// [`NotADisplay::Unknown`] for a display that was not here.
    pub fn display_left(
        &mut self,
        display: DisplayId,
        named: &str,
        who: &impl Fn(alo_dividing::WindowId) -> Option<HeldBy>,
    ) -> Result<alo_desktops::OnADisplay, NotADisplay> {
        self.desk.display_left(display, named, who)
    }

    /// How this display is divided now, for whatever draws it.
    #[must_use]
    pub fn dividing(&self, display: DisplayId) -> Option<&Division> {
        self.desk.dividing(display)
    }

    /// Divide a display, with a division `alo-dividing` decided.
    pub fn divide(&mut self, display: DisplayId, division: Division) {
        self.desk.divide(display, division);
    }

    /// The desktops on this display.
    #[must_use]
    pub fn desktops_on(&self, display: DisplayId) -> Option<&alo_desktops::OnADisplay> {
        self.desk.desktops(display)
    }

    /// Move to another desktop, as `alo-desktops` decides which that is.
    ///
    /// # Errors
    /// [`NotADisplay::Unknown`] for a display that is not here.
    pub fn switch_desktop(
        &mut self,
        display: DisplayId,
        switch: Switch,
    ) -> Result<Result<alo_desktops::DesktopId, alo_desktops::Refused>, NotADisplay> {
        self.desk.switch(display, switch)
    }

    /// Whether this session has any display at all.
    #[must_use]
    pub fn has_no_display(&self) -> bool {
        self.desk.is_empty()
    }
}

#[cfg(test)]
#[path = "server_desk_tests.rs"]
mod tests;
