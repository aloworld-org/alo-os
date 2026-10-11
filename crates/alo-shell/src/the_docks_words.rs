//! Everything on the Dock that a person reads, said once a frame.
//!
//! **The drawing path may not word a sentence.** `tests/desktop_source.rs` holds
//! that no file named `desktop_*` or `dock_*` names `Word::`, `.say(` or
//! `Filling::`: every sentence a person meets comes from `alo-saying`, and a
//! raster that looked one up would be a second place deciding what a person
//! reads.
//!
//! So the lane says them and hands the results over, exactly as it does for a
//! window's control labels.
//!
//! # One bundle rather than five fields
//!
//! `DesktopFrame` carried `the_overflows_heading` alone for a day, and the Dock
//! then gained a composer with two strings in it and two command labels. Five
//! borrowed `&str` fields on a frame is five chances to set four of them, and
//! the one that is forgotten is a blank where a word should be — which draws
//! fine and says nothing.
//!
//! A struct cannot be half-filled: this type's one constructor builds
//! all of them or none, from one `Strings`, in one place.

use alo_strings::{Filling, Strings};

/// What the Dock says, already said.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheDocksWords {
    /// The heading over the list the overflow control opens.
    pub heading: String,
    /// What the composer invites a person to do.
    pub prompt: String,
    /// The key that opens the composer, shown at its end.
    pub key: String,
    /// The control that lays every open window out at once.
    pub show_all: String,
    /// The control that opens what a person did before.
    pub history: String,
}

impl TheDocksWords {
    /// Say all of them, in this person's language.
    ///
    /// **All five or none**, which is the whole reason this is a type: a frame
    /// with four of them set and one blank draws a control with no label on it
    /// and refuses nothing.
    #[must_use]
    pub fn said(strings: &Strings) -> Self {
        let say =
            |word: &alo_strings::Word| strings.say(&word.key(), &Filling::nothing()).into_text();
        Self {
            heading: say(&alo_dock::words::MORE_OPEN_APPS),
            prompt: say(&alo_dock::words::FIND_OPEN_OR_ASK),
            key: say(&alo_dock::words::THE_COMPOSERS_KEY),
            show_all: say(&alo_dock::words::SHOW_ALL),
            history: say(&alo_dock::words::HISTORY),
        }
    }
}
