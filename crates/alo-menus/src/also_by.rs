//! Where else a person reaches what a context menu offers.
//!
//! A context menu is the fastest way to do a handful of things and it is never
//! the only way to do any of them. That is ADR 0009's standing rule read in the
//! direction it is usually forgotten: the ADR is about a machine that must be
//! whole with the agent switched off, and the same argument applies to a person
//! who drives the machine from the keyboard, or through a screen reader, or with
//! one hand — **a capability that lives in a right-click is a capability some
//! people do not have.**
//!
//! So every [`crate::Action`] answers this, the compiler holds it to answering,
//! and the answer is a **closed list of roads that exist**: a shortcut this
//! machine already binds, an application's own menus, the files window, or
//! Settings. There is no *somewhere else* variant, because a road nobody can
//! name is a road nobody has walked.
//!
//! # Nothing here is read to a person
//!
//! These are not rows on a screen: a menu shows what choosing an entry does, not
//! a lecture about where else it could have been done. The reader of an
//! [`AlsoBy`] is whoever adds an entry, in this repository, and the test that
//! walks them — so it keeps its English and has no words of its own, exactly as
//! `alo_by_hand::Finding` does for the same question about verbs.

use alo_shortcuts::Action as Shortcut;

/// Where a person reaches a menu entry's action without the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlsoBy {
    /// A keyboard shortcut this machine already binds — named as
    /// `alo-shortcuts` names it, so an entry pointing at a shortcut that does
    /// not exist is not a thing anybody can write.
    AShortcut(Shortcut),
    /// The menus at the top of the application's own window.
    TheApplicationsOwnMenus,
    /// The files window, where a person manages their own files.
    TheFilesWindow,
    /// Settings.
    Settings,
}

impl AlsoBy {
    /// The shortcut, when the other road is one.
    #[must_use]
    pub const fn shortcut(self) -> Option<Shortcut> {
        match self {
            Self::AShortcut(shortcut) => Some(shortcut),
            Self::TheApplicationsOwnMenus | Self::TheFilesWindow | Self::Settings => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A road that is a shortcut names one, and the others name none.
    #[test]
    fn a_road_that_is_a_shortcut_names_which() {
        assert_eq!(
            AlsoBy::AShortcut(Shortcut::SnapLeft).shortcut(),
            Some(Shortcut::SnapLeft)
        );
        assert_eq!(AlsoBy::TheFilesWindow.shortcut(), None);
        assert_eq!(AlsoBy::Settings.shortcut(), None);
        assert_eq!(AlsoBy::TheApplicationsOwnMenus.shortcut(), None);
    }
}
