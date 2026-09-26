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
    /// The number each window is known by outside this crate.
    numbers: crate::window_number::Numbers,
    /// How big each display is, as it was when it arrived.
    areas: BTreeMap<DisplayId, Area>,
    /// What each display is remembered by, as it arrived under.
    ///
    /// Kept so a display can be let go of without the caller having to hand
    /// back the name it plugged it in with. A departure that used a different
    /// name would remember the arrangement where nothing looks for it.
    names: BTreeMap<DisplayId, String>,
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
        self.areas.insert(display, area);
        self.names.insert(display, named.to_owned());
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
        self.areas.remove(&display);
        self.names.remove(&display);
        self.desktops.unplug(display)
    }

    /// How this display is divided now, or [`None`] where nothing has divided
    /// it.
    pub(crate) fn dividing(&self, display: DisplayId) -> Option<&Division> {
        self.dividing.get(&display)
    }

    /// The division to change, for a caller `alo-dividing` has given an answer
    /// to.
    ///
    /// Borrowed rather than handed back, because **a `Division` is not
    /// `Clone`**: it carries an identity and a count of its changes, and two
    /// copies would both claim to be the one a proposal was made against.
    pub(crate) fn dividing_mut(&mut self, display: DisplayId) -> Option<&mut Division> {
        self.dividing.get_mut(&display)
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

    /// The number this window is known by where a division holds it.
    pub(crate) fn number_of(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> alo_dividing::WindowId {
        alo_dividing::WindowId::from_compositor(self.numbers.of(surface))
    }

    /// The number this window already has, or [`None`] where it has never had
    /// one.
    ///
    /// For finding a window a division holds without giving a number to
    /// something that was never one here.
    pub(crate) fn number_given_to(
        &self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> Option<u64> {
        self.numbers.given_to(surface)
    }

    /// These are the windows that are open now; the rest have closed.
    ///
    /// **A division is kept true to what is open, once a frame.** A window that
    /// closed loses its share and the rest of the tree collapses onto it, which
    /// is `alo_dividing::Division::close`'s own behaviour — and its number is
    /// never given to anything else, so a share cannot come to belong to a
    /// different window. That is the whole reason `crate::window_number` hands
    /// out a number and never takes one back.
    pub(crate) fn windows_are_now<'a>(
        &mut self,
        open: impl Iterator<
            Item = &'a smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
        >,
    ) {
        let alive: std::collections::HashSet<u32> =
            open.map(crate::window_number::Numbers::identity).collect();
        for number in self.numbers.keep_only(&alive) {
            let window = alo_dividing::WindowId::from_compositor(number);
            for division in self.dividing.values_mut() {
                // A division that does not hold it says so, and that is not a
                // failure: a window can close on a display nobody divided.
                let _ = division.close(window);
            }
        }
    }

    /// Every display here, in the order `alo-desktops` holds them.
    pub(crate) fn displays(&self) -> impl Iterator<Item = DisplayId> + '_ {
        self.desktops.displays()
    }

    /// The area of a display, as it was when it arrived.
    pub(crate) fn area_of(&self, display: DisplayId) -> Option<Area> {
        self.areas.get(&display).copied()
    }

    /// What this display is remembered by, or [`None`] where it is not here.
    pub(crate) fn name_of(&self, display: DisplayId) -> Option<&str> {
        self.names.get(&display).map(String::as_str)
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
