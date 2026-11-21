//! **The road a person installs an application by is said on the journal**:
//! which Dock slot the keyboard is on, which market card, and what became of
//! an installation (`alo_acting::said_in_the_market`).
//!
//! No walk had installed an application (the launch audit of 2026-11-21, item
//! 6). The market opens from one slot on the Dock and its cards are moved
//! through by the arrows, and a walk cannot read either off a picture, so this
//! says each by name — the Dock's slot by the English source word it draws
//! (an application by its own name), the market's card by the catalogue's
//! identifier — whenever it changes, the shape
//! `crate::settings_says_where_the_keyboard_is` has for Settings.

use alo_acting::said_in_the_market as said;

use crate::the_dock_as_a_reader_meets_it::{ItsName, as_read};
use crate::which_area_has_the_keyboard::AnArea;

/// Write one line on the journal, which is this shell's stderr.
pub(crate) fn say(line: &str) {
    let mut line = line.to_owned();
    line.push('\n');
    let _ = std::io::Write::write_all(&mut std::io::stderr(), line.as_bytes());
}

impl crate::Server {
    /// The name of the Dock slot the keyboard is on, or [`None`] while the
    /// keyboard is not on the Dock — counted the way Tab and Enter count, on
    /// the first display's Dock (`Self::the_docks_slots_as_drawn`' reason).
    pub(crate) fn where_the_keyboard_is_on_the_dock(&self) -> Option<String> {
        if self.which_area_has_the_keyboard != Some(AnArea::TheDock) {
            return None;
        }
        let which = self.which_slot_on_the_bar?;
        let dock = self.dock_as_drawn.values().next()?;
        let place = crate::the_bars_pressable_slots::the_one_at(&dock.places, which)?;
        let slot = as_read(place.what())?;
        Some(match slot.name {
            ItsName::ItsOwn(name) => name.to_owned(),
            ItsName::Said(word) => word.says().to_owned(),
        })
    }

    /// **After the keyboard moved on or off the Dock**: say where it is, if
    /// that changed, and forget it once it leaves.
    pub(crate) fn the_dock_says_where_the_keyboard_went(&mut self) {
        let now = self.where_the_keyboard_is_on_the_dock();
        if let Some(name) = &now
            && now != self.said_on_the_dock
        {
            say(&said::on_the_dock_the_keyboard_is_on(name));
        }
        self.said_on_the_dock = now;
    }

    /// The identifier of the market card the keyboard is on, or [`None`]
    /// while the market is closed.
    pub(crate) fn where_the_keyboard_is_in_the_market(&self) -> Option<String> {
        match self.market.shows() {
            crate::MarketShows::Nothing => None,
            crate::MarketShows::Browsing(browsing) => browsing
                .whichever_they_are_on()
                .map(|one| one.application().identifier().to_owned()),
        }
    }

    /// **After a key the market took, or its opening**: say which card the
    /// keyboard is on, if that changed, and forget it once the market closes.
    pub(crate) fn the_market_says_where_the_keyboard_went(&mut self) {
        let now = self.where_the_keyboard_is_in_the_market();
        if let Some(identifier) = &now
            && now != self.said_in_the_market
        {
            say(&said::in_the_market_the_keyboard_is_on(identifier));
        }
        self.said_in_the_market = now;
    }

    /// **The market opened**: say how many it lists, and the card it is on.
    pub(crate) fn the_market_says_it_opened(&mut self) {
        if let crate::MarketShows::Browsing(browsing) = self.market.shows() {
            say(&said::the_market_opened(browsing.how_many()));
        }
        self.said_in_the_market = None;
        self.the_market_says_where_the_keyboard_went();
    }

    /// **The host says an installation arrived**: said on the journal, and the
    /// window told. Every road that answers the market comes through here or
    /// [`Self::the_market_is_told_it_was_refused`], so none is left unsaid.
    pub(crate) fn the_market_is_told_it_arrived(&mut self, identifier: &str) {
        say(&said::an_installation_arrived(identifier));
        self.market.it_arrived(identifier);
    }

    /// **The host says an installation was not done**: said on the journal
    /// with why, in `alo-software`'s own terms, and the window told.
    pub(crate) fn the_market_is_told_it_was_refused(
        &mut self,
        identifier: &str,
        why: alo_software::Stopped,
    ) {
        say(&said::an_installation_was_not_done(
            identifier,
            &format!("{why:?}"),
        ));
        self.market.it_was_refused(identifier, why);
    }
}

#[cfg(test)]
#[path = "the_road_to_the_market_is_said_tests.rs"]
mod tests;
