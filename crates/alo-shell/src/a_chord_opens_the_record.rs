//! **The chord that opens the record, and the one opener a row in Settings
//! uses too.**
//!
//! `docs/features.md` promises the record and *afterwards, ask what it did*.
//! `crate::RecordWindow` was built, routed and tested on 2026-10-17, and
//! nothing on a running machine opened it: `alo_shortcuts::Action` had no
//! action for it, Settings had no row, and the Dock had no entry. Demo 1
//! stopped at *open the record* for exactly that reason on 2026-10-27, and the
//! owner chose this road the same day: **Super+R, and a row in Settings.**
//!
//! This makes an existing promise true rather than adding one, because the
//! record and a plain road to it (ADR 0009) were both already promised.
//!
//! # It needs no agent
//!
//! It reads the record through `RecordWindow::opened_by_hand`, which takes
//! where the record is and nothing else. A machine with no model, no provider
//! and no agent opens it the same way.
//!
//! # Where the record is, is told
//!
//! `Server::recounting` is set by the desktop, for `settings_places`' reason:
//! the description naming the record's path is a file, and this crate opens
//! none. A session never told opens nothing and says so on the error stream —
//! never an empty window, which would read as *alo did nothing*.

use alo_recounting::{NotRecounted, Recounting};
use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::RecordOpened;

impl crate::Server {
    /// Where this machine keeps its record, or why it could not say.
    ///
    /// Told once by the binary that read the machine's description, like
    /// [`crate::Server::the_settings_places_are`].
    pub fn the_record_is_kept(&mut self, recounting: Result<Recounting, NotRecounted>) {
        self.recounting = Some(recounting);
    }

    /// The record window this session owns, for the frame that draws it.
    pub(crate) const fn the_record_window(&self) -> &crate::RecordWindow {
        &self.record
    }

    /// The record window as a frame draws it, while it is open, in the
    /// person's own look.
    ///
    /// **[`None`] while it is shut**, so a frame with no record pays nothing.
    pub(crate) fn the_records_frame<'a>(
        &'a self,
        strings: &'a alo_strings::Strings,
        look: crate::DesktopLook,
    ) -> Option<crate::RecordFrame<'a>> {
        self.record.is_open().then_some(crate::RecordFrame {
            window: self.the_record_window(),
            strings,
            look: crate::RecordLook {
                scheme: look.scheme(),
                scale: look.scale(),
                reading: look.reading(),
                contrast: look.contrast(),
            },
            // **The person's formats, and now.** `crate::record_clock` says
            // why the window decides neither.
            clock: crate::RecordClock {
                formats: self.the_formats(),
                now: std::time::SystemTime::now(),
            },
        })
    }

    /// Whether the record window is open, and so takes the keyboard.
    pub(crate) fn the_record_is_taking_the_keys(&self) -> bool {
        self.record.is_open()
    }

    /// Open the record window, from a chord or from the row in Settings.
    ///
    /// **Settings is closed first**, because Settings is painted above the
    /// record and takes the keys before it: both open would be a person typing
    /// into a window they cannot see.
    ///
    /// Answers [`None`] when nobody told this session where the record is.
    pub(crate) fn the_record_is_opened(&mut self) -> Option<RecordOpened> {
        let Some(recounting) = self.recounting.clone() else {
            eprintln!(
                "alo-shell: the record was asked for and this session was never told where it is"
            );
            return None;
        };
        self.settings.close();
        let opened = match recounting {
            Ok(recounting) => self.record.opened_by_hand(&recounting),
            Err(why) => self.record.could_not_be_found(why),
        };
        if let RecordOpened::NowhereToShow(why) = &opened {
            eprintln!("alo-shell: the record could not be shown: {why:?}");
        }
        Some(opened)
    }

    /// Carry out the record's chord, if this is it.
    ///
    /// **Taken whether or not anything opened**, so the chord never falls
    /// through to an application: a person who pressed it asked for the
    /// record, and what they get instead is a sentence on the error stream.
    pub(crate) fn a_chord_opens_the_record(&mut self, shortcuts: &Shortcuts, chord: Chord) -> bool {
        if shortcuts.action_for(chord) != Some(Action::SeeWhatAloDid) {
            return false;
        }
        let _ = self.the_record_is_opened();
        true
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on a window showing the wrong thing is the failure being reported"
)]
#[path = "a_chord_opens_the_record_tests.rs"]
mod tests;
