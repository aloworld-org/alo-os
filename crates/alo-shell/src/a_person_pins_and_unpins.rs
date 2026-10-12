//! Pinning something to this person's Dock, and taking it off again.
//!
//! `docs/features.md` [v0.01] promises it: *after that the person owns the
//! list: additions, removals, reordering and a list they deliberately emptied
//! all survive a restart and an update.* The surviving was built first —
//! `alo_dock::Changes` keeps the list and `dock.toml` carries it — and **there
//! was no way to make a change for it to survive.** Measured 2026-10-12: every
//! `Holding::pin` and `unpin_entry` call outside a test was a fixture, and the
//! only production write was the defaults.
//!
//! So a saved list existed that nothing but a fresh machine ever wrote. This is
//! the half that makes the promise true.
//!
//! # Why the write is owed rather than done
//!
//! `crates/alo-shell` keeps no settings file of its own — ADR 0038, and
//! `tests/settings_source.rs` refuses any shipped file here that even names
//! where settings are kept. So these answer **what is owed to the disk** and
//! the desktop that owns the folder writes it, which is the road
//! `its_dock_is_filled_from_what_they_kept` already takes for the defaults.
//!
//! # And why it is owed *immediately*
//!
//! Not at sign-out. A machine loses power, a session is killed, a person
//! closes the lid — and a pin they made an hour ago should not be the thing
//! that is lost. The list is small and the write replaces one short file.

use alo_dock::ADockEntry;
use alo_dock::window::AppId;

use crate::server::Server;

impl Server {
    /// Keep this application on the Dock when it closes.
    ///
    /// Answers **what is owed to the disk**, or [`None`] when nothing changed —
    /// which is what pinning something already pinned does. `Holding::pin_entry`
    /// is where that rule lives: *pinning something already pinned changes
    /// nothing and does not move it*, because an icon that jumped because
    /// somebody pinned it twice is the instability it exists to prevent.
    pub fn this_application_is_pinned(&mut self, app: &AppId) -> Option<Vec<ADockEntry>> {
        self.this_entry_is_pinned(ADockEntry::Application(app.clone()))
    }

    /// Keep this entry on the Dock, of either kind.
    pub fn this_entry_is_pinned(&mut self, entry: ADockEntry) -> Option<Vec<ADockEntry>> {
        if self.what_the_dock_is_pinned_to.is_entry_pinned(&entry) {
            return None;
        }
        self.what_the_dock_is_pinned_to.pin_entry(entry);
        let now = self.what_is_pinned_now();
        Some(self.these_pins_are_owed(now))
    }

    /// Stop keeping this entry on the Dock.
    ///
    /// **Either kind**, which is the owner's ruling of 2026-10-12 carried to
    /// the one place a person acts on it: *native describes the implementation;
    /// it does not mean permanent.* A Settings entry comes off exactly as an
    /// application does.
    ///
    /// **An unpin that empties the Dock is owed a write like any other.** That
    /// is the clause the whole saved shape exists for: an empty list a person
    /// made is a choice and stays one, and refusing to write it here would make
    /// the defaults come back at the next sign-in.
    pub fn this_entry_is_unpinned(&mut self, entry: &ADockEntry) -> Option<Vec<ADockEntry>> {
        if !self.what_the_dock_is_pinned_to.unpin_entry(entry) {
            return None;
        }
        let now = self.what_is_pinned_now();
        Some(self.these_pins_are_owed(now))
    }

    /// Take the pinned list a person changed, if one is waiting.
    ///
    /// **Taken rather than read**, so one change causes one write however many
    /// frames pass before the desktop asks. That is `start_what_was_asked_for`'s
    /// shape and the capability model's rule in the same sentence: *one
    /// approval causes exactly one execution.*
    pub fn take_the_pins_owed_to_the_disk(&mut self) -> Option<Vec<ADockEntry>> {
        self.pins_owed_to_the_disk.take()
    }

    /// Remember that this list is owed to the disk.
    ///
    /// **The last one wins**, which is right because each is the whole list
    /// rather than a change to it: two unpins before a frame is drawn leave one
    /// write of what the Dock holds after both.
    fn these_pins_are_owed(&mut self, pins: Vec<ADockEntry>) -> Vec<ADockEntry> {
        self.pins_owed_to_the_disk = Some(pins.clone());
        pins
    }

    /// The list as it stands, for the disk.
    fn what_is_pinned_now(&self) -> Vec<ADockEntry> {
        self.what_the_dock_is_pinned_to
            .each_pinned()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
#[path = "a_person_pins_and_unpins_tests.rs"]
mod tests;
