//! **Settings says on the journal where its keyboard went, by the name it
//! draws.**
//!
//! Main's walk at 9ced5c40 (run 44851000278) counted five Downs to *Add a
//! provider*, landed on *alo* (picture 54) — a provider added earlier brings a
//! row and its *Say where it runs* button — and saved nothing it meant to. A walk
//! cannot read a row's name off a picture, so after each key Settings takes,
//! this says the name of the row or field the keyboard is on, whenever it
//! changes: `alo-shell: in Settings the keyboard is on "Add a provider"`
//! (`alo_acting::said_in_settings`).
//!
//! **The name is the one a screen reader is told** — read from the same tree
//! (`crate::settings_as_read`), which names each row by the word the page draws
//! and each field of the form by its label. So it is never what a person typed
//! into a field, and it is said on this machine's own journal and nowhere else.

use crate::access_sheet_in_front::TheSheet;

impl crate::Server {
    /// The name of the row or field Settings' keyboard is on, as a reader is
    /// told it — [`None`] while Settings is not the sheet in front.
    pub(crate) fn where_the_keyboard_is_in_settings(
        &self,
        strings: &alo_strings::Strings,
    ) -> Option<String> {
        let tree = self.read_aloud_with_the_frames_open(
            strings,
            &[alo_access::Surface::Desktop],
            &alo_access::TurnedOn::nothing(),
        );
        let sheet = tree.in_front()?;
        if sheet.which != TheSheet::Settings {
            return None;
        }
        tree.nodes()
            .get(sheet.keyboard)
            .map(|node| node.name.clone())
    }

    /// **After a key Settings took**: say where its keyboard is, if that
    /// changed — and forget it once Settings is no longer in front, so opening
    /// it again says its first row afresh.
    pub(crate) fn settings_says_where_the_keyboard_went(&mut self, strings: &alo_strings::Strings) {
        let now = self.where_the_keyboard_is_in_settings(strings);
        if let Some(name) = &now
            && now != self.said_in_settings
        {
            // **The whole line is `alo_acting`'s**: a Settings file words
            // nothing of its own (`tests/settings_source.rs`), so it is written
            // as declared there, with no format of this file's.
            let mut line = alo_acting::said_in_settings::the_keyboard_is_on(name);
            line.push('\n');
            let _ = std::io::Write::write_all(&mut std::io::stderr(), line.as_bytes());
        }
        self.said_in_settings = now;
    }
}
