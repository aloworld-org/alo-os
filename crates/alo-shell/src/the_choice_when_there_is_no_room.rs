//! **Where a new window goes when there is no room for it in view**, as the
//! person chose in Settings → Appearance → *When there's no room for a new
//! window*.
//!
//! The owner, 2026-11-07: *glide to empty space* is how this computer comes, and
//! *open in front, slightly offset* — the rule before that day — stays a choice
//! (`alo_appearance::WhenThereIsNoRoom`). Told to this seat when the desktop
//! stands up and again whenever Settings keeps a new answer, so the next window
//! opens the way the person just chose.

impl crate::Server {
    /// Where a new window goes when there is no room, as the person last said.
    ///
    /// **Told rather than read**, for the reason `Server::the_window_titles_are`
    /// gives: the desktop reads the file at sign-in and says what it holds;
    /// Settings says it again when the person changes it.
    pub fn when_there_is_no_room_is(&mut self, choice: alo_appearance::WhenThereIsNoRoom) {
        self.when_there_is_no_room = choice;
    }

    /// Where a new window goes when there is no room for it in view.
    #[must_use]
    pub const fn when_there_is_no_room(&self) -> alo_appearance::WhenThereIsNoRoom {
        self.when_there_is_no_room
    }
}
