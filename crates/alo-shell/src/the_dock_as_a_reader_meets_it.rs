//! **What each control on the Dock is called when a screen reader reaches it,
//! and whether pressing it does anything.**
//!
//! The owner's decision of 2026-10-28: every Dock button has a name a screen
//! reader announces. Until then the tree said *the dock*, *an application* and
//! *open something* and nothing per control, so somebody using a reader was
//! told a Dock existed and never what was on it.
//!
//! # Named by the words it is drawn with
//!
//! ADR 0089's rule, which `alo_access::Surface::read_aloud` keeps for the
//! approval's two answers: what a reader hears is the string a sighted person
//! is shown, so the two cannot drift and a translator moves both at once.
//! Settings, Applications, *Show all* and History are the words
//! `crate::the_docks_words` draws; the composer is its own prompt; *Show more
//! apps* is the overflow's name, which the design gives as both its spoken name
//! and its tooltip. An application is called by its own name, untranslated,
//! which is what the Dock draws for it (`alo_dock::AppId::name`).
//!
//! # A press is offered only where a click does something
//!
//! Each control's press is `crate::a_press_on_the_dock::what_this_slot_does`,
//! the same answer a pointer and the keyboard get. **History opens nothing
//! yet**, so it is named and announced but carries no action: a reader told it
//! can press something that then does nothing would be told something untrue.
//! The divider is a line rather than a control, and is not in the tree at all.

use alo_access::Role;
use alo_dock::a_dock_entry::ANativeAction;
use alo_dock::places::WhatIsHere;
use alo_dock::words;
use alo_strings::Word;

use crate::a_press_on_the_dock::{WhatAPressOnTheDockDoes, what_this_slot_does};
use crate::access_action::Pressed;

/// What a control on the Dock is called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItsName<'a> {
    /// An application's own name, which nobody translates.
    ItsOwn(&'a str),
    /// A word from the vocabulary, said in the person's language.
    Said(Word),
}

/// One control on the Dock, as a reader is told about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ASlotAsRead<'a> {
    /// What kind of thing it is.
    pub(crate) role: Role,
    /// What it is called.
    pub(crate) name: ItsName<'a>,
    /// What pressing it asks the shell to do, where a click does anything.
    pub(crate) press: Option<Pressed>,
}

/// `what` as a reader is told about it, or [`None`] for the divider, which is
/// a line and not a control.
#[must_use]
pub(crate) fn as_read(what: &WhatIsHere) -> Option<ASlotAsRead<'_>> {
    let (role, name) = match what {
        WhatIsHere::TheDivider => return None,
        WhatIsHere::Application(app) => (Role::ListItem, ItsName::ItsOwn(app.name())),
        WhatIsHere::ANativeAction(ANativeAction::Settings) => {
            (Role::Button, ItsName::Said(words::SETTINGS_ON_THE_DOCK))
        }
        WhatIsHere::ANativeAction(ANativeAction::Applications) => {
            (Role::Button, ItsName::Said(words::APPLICATIONS_ON_THE_DOCK))
        }
        WhatIsHere::TheOverflow => (Role::Button, ItsName::Said(words::SHOW_MORE_OPEN_APPS)),
        WhatIsHere::TheComposer => (Role::Entry, ItsName::Said(words::FIND_OPEN_OR_ASK)),
        WhatIsHere::ShowAll => (Role::Button, ItsName::Said(words::SHOW_ALL)),
        WhatIsHere::History => (Role::Button, ItsName::Said(words::HISTORY)),
    };
    let press = (what_this_slot_does(what) != WhatAPressOnTheDockDoes::Nothing)
        .then(|| Pressed::OnTheDock(what.clone()));
    Some(ASlotAsRead { role, name, press })
}

impl crate::Server {
    /// What is on the Dock as it was last drawn, in the order it is drawn.
    ///
    /// **One Dock, the first display's by name.** Every display draws the same
    /// Dock from the same contents; what can differ is how many applications
    /// fit before the rest go under *Show more apps*. A reader is told one
    /// Dock rather than one per display, because a person has one Dock, and
    /// the first by name is the same one every frame rather than whichever
    /// display happened to draw last. Empty before any Dock has been drawn.
    #[must_use]
    pub(crate) fn the_docks_slots_as_drawn(&self) -> Vec<WhatIsHere> {
        self.dock_as_drawn
            .values()
            .next()
            .map(|dock| {
                dock.places
                    .each()
                    .iter()
                    .map(|place| place.what().clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether any Dock drawn now holds `what`.
    ///
    /// What a reader's press is held to, as a click is held to landing on a
    /// slot: a press heard on a Dock that has since changed reaches nothing
    /// rather than opening an application that is no longer there.
    #[must_use]
    pub(crate) fn a_dock_drawn_now_holds(&self, what: &WhatIsHere) -> bool {
        self.dock_as_drawn
            .values()
            .any(|dock| dock.places.each().iter().any(|place| place.what() == what))
    }
}

#[cfg(test)]
#[path = "the_dock_as_a_reader_meets_it_tests.rs"]
mod tests;
