//! What a preview's row says at the top of it.
//!
//! Part of task 8 of `docs/autonomy/putting-a-window-aside.md`: a private window shows a
//! neutral *Preview hidden* surface with a safe identifying name.
//!
//! # This is the only way to get a preview's heading, and that is the guarantee
//!
//! `Preview` used to offer `called()`, which returned the window's title. It no longer does.
//! A public accessor returning the title is all a surface needs to leak a private window, and
//! no amount of documentation beside it prevents the one caller who draws the obvious field.
//!
//! So the title is **unreachable** and this is what a surface gets instead. For an ordinary
//! window it carries the title; for a private one it carries the safe name and does not carry
//! the title at all — not privately, not behind a method, not at all. A surface that wants to
//! head a row has exactly one thing it can ask for, and that thing is already correct.
//!
//! **This is the same correction as `Zoom` and `Camera`**, made properly the first time
//! rather than after a peer measured it: a rule held by a type not having the value beats a
//! rule held by a caller not asking for it.
//!
//! # Nothing here is a translated string
//!
//! *Preview hidden* is a **case**, not a message. The words belong to whoever draws and are
//! externalised there, because this crate cannot know the person's language and a crate that
//! shipped English here would be the hardcoded-English bug the standing rules name. What
//! travels from here is *which of two things to say* and the name to say it with.

use crate::a_safe_name::SafeName;

/// What to write at the top of a preview's row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Headline<'a> {
    /// An ordinary window: its own title.
    ItsTitle(&'a str),
    /// A private window: *Preview hidden*, with this name to tell it from another.
    ///
    /// The words *Preview hidden* are the drawer's, translated there. This carries the case
    /// and the name.
    PreviewHidden(&'a SafeName),
}

impl<'a> Headline<'a> {
    /// The text to draw, whichever case this is.
    ///
    /// For a private window this is the **safe name and never the title**, which is the whole
    /// reason a surface asks here rather than reaching for a field.
    #[must_use]
    pub fn text(self) -> &'a str {
        match self {
            Self::ItsTitle(title) => title,
            Self::PreviewHidden(name) => name.as_str(),
        }
    }

    /// Whether the row is a hidden one, so a surface knows to draw the neutral treatment.
    #[must_use]
    pub const fn is_hidden(self) -> bool {
        matches!(self, Self::PreviewHidden(_))
    }
}
