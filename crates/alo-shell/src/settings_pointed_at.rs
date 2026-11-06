//! **What a press on Settings points at**, as Settings understands it: a
//! category in the side column, a row on the page, or an entry in a list a
//! picker opened.
//!
//! # A press moves the keyboard, and a key does the rest
//!
//! Settings is operated whole from the keyboard (`crate::settings_keys`), and
//! a press does not get a second road into it. It moves **where the keyboard
//! is** to what was pressed — exactly where Up, Down and Tab would have taken
//! it — and then, for a row or an entry, the same **Choose** press Enter makes
//! is carried out through `crate::a_key_reaches_settings`. So a click and a
//! key cannot disagree about what a row does, and every refusal a row can say
//! is said the same way.
//!
//! - **A category**: the page is shown, with the keyboard on the category, as
//!   Up and Down choose a page. Nothing is chosen.
//! - **A row**: the keyboard goes to it in the page, and it is chosen.
//! - **An entry of an open list**: the keyboard goes to it, and it is chosen,
//!   which closes the list as Enter does.
//! - **Anywhere else on the window while a list is open**: the list closes, as
//!   Escape closes it, and Settings stays open.
//!
//! Waiting for a chord stops at any press: a person who clicked elsewhere has
//! not pressed the chord they were asked for.

use std::time::SystemTime;

use crate::SettingsRow;
use crate::settings_categories::SettingsCategory;
use crate::settings_window::SettingsWindow;

/// What a press on Settings was on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PointedAt {
    /// A category in the side column.
    ACategory(SettingsCategory),
    /// A row on the page.
    ARow(SettingsRow),
    /// An entry in the list a picker opened.
    AnEntry(SettingsRow),
    /// The window, outside the list a picker opened.
    OutsideTheList,
    /// The window, and nothing on it a person can act on.
    TheWindow,
}

/// Whether the press is then carried out as Enter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThenChoose {
    /// It is: the keyboard is on a row or an entry.
    Yes,
    /// It is not: the keyboard moved, or nothing happened.
    No,
}

impl SettingsWindow {
    /// The keyboard goes to what a press was on, and whether that press is
    /// then carried out as Enter. Nothing happens while the window is shut.
    pub(crate) fn pointed_at(&mut self, what: &PointedAt, now: SystemTime) -> ThenChoose {
        let Some(open) = self.open_mut() else {
            return ThenChoose::No;
        };
        open.waiting_for_a_chord = false;
        match what {
            PointedAt::ACategory(category) => {
                open.picker = None;
                if open.category != *category {
                    open.told = None;
                }
                open.category = *category;
                open.in_the_page = false;
                open.focus = 0;
                ThenChoose::No
            }
            PointedAt::ARow(row) => {
                open.picker = None;
                let rows = open.rows_on(open.category, now);
                match rows.iter().position(|it| it == row) {
                    Some(at) => {
                        open.in_the_page = true;
                        open.focus = at;
                        ThenChoose::Yes
                    }
                    None => ThenChoose::No,
                }
            }
            PointedAt::AnEntry(row) => {
                let Some(picker) = open.picker.as_mut() else {
                    return ThenChoose::No;
                };
                match picker.shown.iter().position(|it| it == row) {
                    Some(at) => {
                        picker.focus = at;
                        ThenChoose::Yes
                    }
                    None => ThenChoose::No,
                }
            }
            PointedAt::OutsideTheList => {
                open.picker = None;
                ThenChoose::No
            }
            PointedAt::TheWindow => ThenChoose::No,
        }
    }
}

#[cfg(test)]
#[path = "settings_pointed_at_tests.rs"]
mod tests;
