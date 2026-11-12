//! **Settings, as a screen reader is told about it: each page as it is drawn.**
//!
//! The owner's decision of 2026-11-12, *"1. Live, keep names"*. Until then a
//! reader was told Settings by a fixed list — the window, *what can be
//! changed*, and nine switches of which eight are drawn on no page — so a
//! person on Settings → Default apps heard nothing of that page.
//!
//! Now the description **is the page**: [`crate::settings_page::the_page`], the
//! one list the drawing and the keyboard already read, is read a third time
//! here, so a row cannot be drawn and not heard, or heard and not drawn. Every
//! word is one the page draws, said in the person's language.
//!
//! # What is read, in order
//!
//! - **the window**, named as the side column is headed — *Settings*;
//! - **the side column**, a list of the nine pages as tabs, the page shown
//!   selected, each with its group as description and its place (*6 of 9*);
//! - **the page's heading**, at level 1, and the sentence under it;
//! - each **section**: its heading at level 2, its rows, and its footnote; a
//!   sentence the page says after a change is announced as it appears;
//! - each **row** by its drawn name, with what is drawn under or beside it as
//!   description: a choice as a radio button, chosen or not, with its place
//!   among its section's choices (*2 of 3*); a switch, on or off; a row of
//!   side-by-side choices as a list of radio buttons; text size as a slider; a
//!   row that opens something, or a button at a row's end, as a button;
//! - **the list a row opened**, while one is open: its search field, then its
//!   matches and the whole list as radio buttons;
//! - **the form that adds a provider**, while it is open, in place of the
//!   page: its sentences, then its four fields — the key as a password field.
//!
//! # Where the keyboard is
//!
//! While Settings is the window in front, it is put in front on the tree
//! (`crate::access_sheet_in_front`), and the control the keyboard is on is
//! focused — the row on the page, the entry in an open list, the field in the
//! form, or the page shown in the side column — so a reader is told each time
//! it moves (`crate::access_events`).
//!
//! # A press is a click
//!
//! Every page, row, choice, button and entry carries
//! [`InSettings`], carried out through the door a click takes
//! (`crate::a_reader_presses_in_settings`).
//!
//! # The names it had
//!
//! Settings' own node keeps the name the fixed list gave it, and a row that is
//! an accessibility setting keeps that setting's old name; a setting no page
//! draws yet is listed on Settings' node (`crate::settings_old_names`).

use alo_access::{Role, State, Surface};
use alo_setting_up::a_provider_is_filled_in::{AProviderBeingAddded, WhichField};
use alo_strings::Strings;

use crate::access_action::{InSettings, Pressed};
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_attributes::its_place;
use crate::access_roles;
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::settings_categories::SettingsCategory;
use crate::settings_page::{ABlock, ARow, ASection, AnEnd, SwitchIs, TheKind, said};
use crate::settings_window::{Open, SettingsRow, SettingsWindow};
use crate::the_settings_words as w;

/// What Settings shows a reader now.
#[derive(Clone, Copy)]
pub(crate) struct SettingsShows<'a> {
    /// The window, open.
    pub(crate) window: &'a SettingsWindow,
    /// The form adding a provider, while it is the page, and why the last try
    /// was not saved.
    pub(crate) filling_in: Option<(
        &'a AProviderBeingAddded,
        Option<crate::a_provider_is_added_in_settings::WhyNotSaved>,
    )>,
    /// Whether Settings is the window in front, taking the keys.
    pub(crate) takes_the_keys: bool,
    /// The moment, for the grants and pairings in force.
    pub(crate) now: std::time::SystemTime,
}

impl crate::Server {
    /// What Settings shows a reader now, at `now` — [`None`] while it is not
    /// open on the Place being looked at, or is in the right-hand panel.
    pub(crate) fn settings_as_a_reader_meets_it(
        &self,
        now: std::time::SystemTime,
    ) -> Option<SettingsShows<'_>> {
        if !self.settings_is_open_here() {
            return None;
        }
        Some(SettingsShows {
            window: &self.settings,
            filling_in: self
                .a_provider_is_being_filled_in()
                .map(|form| (form, self.why_the_provider_was_not_saved())),
            takes_the_keys: self.settings_is_taking_the_keys(),
            now,
        })
    }
}

/// The heading level of the page's own heading.
const THE_PAGES_LEVEL: u8 = 1;
/// A section's heading level.
const A_SECTIONS_LEVEL: u8 = 2;

impl ReadAloudTree {
    /// The same tree, with Settings' description given way to Settings as it
    /// is drawn now. `shows` is [`None`] while Settings is not open on the
    /// Place being looked at, and the tree is left as it was: described, and
    /// not shown.
    #[must_use]
    pub(crate) fn with_settings(
        mut self,
        strings: &Strings,
        shows: Option<SettingsShows<'_>>,
    ) -> Self {
        let Some(shows) = shows else {
            return self;
        };
        let Some(open) = shows.window.open() else {
            return self;
        };
        let Some(at) = self.the_surfaces_node(Surface::Settings) else {
            return self;
        };
        let window = self.emptied(at);
        let mut attributes = vec![(
            alo_access::THE_OLD_NAME_ATTRIBUTE.to_owned(),
            said(strings, alo_access::words::SETTINGS),
        )];
        attributes.extend(crate::settings_old_names::the_settings_not_drawn_yet(
            open, strings, shows.now,
        ));
        if let Some(node) = self.node_mut(window) {
            let states = access_roles::words_of(State::ReadOnly, None);
            node.name = said(strings, w::SETTINGS);
            node.states = if shows.takes_the_keys {
                access_roles::the_window_in_front(states)
            } else {
                states
            };
            node.attributes = attributes;
        }
        let mut reading = Reading {
            tree: &mut self,
            strings,
            focused: shows.window.focused(shows.now),
            keyboard: None,
        };
        let shown_page = reading.the_side_column(window, open);
        if let Some((form, not_saved)) = shows.filling_in {
            reading.the_form(window, form, not_saved);
        } else {
            let page = crate::settings_page::the_page(open, open.category, strings, shows.now);
            if open.picker.is_some() {
                reading.focused = None;
            }
            reading.the_page(window, &page);
            reading.the_list_open(window, open);
        }
        let keyboard = reading.keyboard.unwrap_or(shown_page);
        if shows.takes_the_keys {
            if let Some(node) = self.node_mut(keyboard) {
                node.states = access_roles::with_the_keyboard(node.states);
            }
            self.put_in_front(ASheetInFront {
                which: TheSheet::Settings,
                at: window,
                keyboard,
            });
        }
        self
    }
}

/// One reading of Settings in progress.
struct Reading<'t, 's> {
    /// The tree being added to.
    tree: &'t mut ReadAloudTree,
    /// The person's vocabulary.
    strings: &'s Strings,
    /// The row the keyboard is on, on the page.
    focused: Option<SettingsRow>,
    /// The node the keyboard is on, once found.
    keyboard: Option<usize>,
}

impl Reading<'_, '_> {
    /// Put a node under `parent`.
    fn push(&mut self, parent: usize, node: Node) -> usize {
        self.tree.push(parent, node)
    }

    /// Put a node under `parent` that a person acts on as `row`, and note
    /// whether the keyboard is on it.
    fn push_a_target(&mut self, parent: usize, row: &SettingsRow, node: Node) -> usize {
        let at = self.push(
            parent,
            Node {
                press: Some(Pressed::InSettings(InSettings::ARow(row.clone()))),
                ..node
            },
        );
        if self.keyboard.is_none() && self.focused.as_ref() == Some(row) {
            self.keyboard = Some(at);
        }
        at
    }

    /// The side column: every page, the one shown selected. Answers the node
    /// of the page shown, which is where the keyboard is while it is in the
    /// side column.
    fn the_side_column(&mut self, window: usize, open: &Open) -> usize {
        let list = self.push(
            window,
            Node {
                role: access_roles::number_of(Role::List),
                name: said(self.strings, alo_access::words::WHAT_CAN_BE_CHANGED),
                description: said(self.strings, w::EVERYTHING_IN_ONE_PLACE),
                states: access_roles::words_of(State::ReadOnly, None),
                ..Node::default()
            },
        );
        let mut shown = list;
        let all = SettingsCategory::ALL.len();
        for (place, category) in SettingsCategory::ALL.into_iter().enumerate() {
            let is_shown = category == open.category;
            let at = self.push(
                list,
                Node {
                    role: access_roles::number_of(Role::Tab),
                    name: category.said(self.strings),
                    description: said(self.strings, category.group().word()),
                    states: access_roles::words_of(State::OneOfAFew, Some(is_shown)),
                    press: Some(Pressed::InSettings(InSettings::APage(category))),
                    attributes: its_place(place, all),
                    ..Node::default()
                },
            );
            if is_shown {
                shown = at;
            }
        }
        shown
    }

    /// The page: its heading and sentence, then its blocks.
    fn the_page(&mut self, window: usize, page: &crate::settings_page::APage) {
        self.a_heading(window, &page.heading, THE_PAGES_LEVEL);
        self.a_line(window, &page.introduction, false);
        for block in &page.blocks {
            match block {
                ABlock::Said(words) => self.a_line(window, words, true),
                ABlock::Section(section) => self.a_section(window, section),
                ABlock::AnAction(action) => {
                    self.push_a_target(
                        window,
                        &action.row,
                        Node {
                            role: access_roles::number_of(Role::Button),
                            name: action.label.clone(),
                            description: action.beside.clone(),
                            states: access_roles::words_of(State::CanBeUsed, None),
                            ..Node::default()
                        },
                    );
                }
            }
        }
    }

    /// A heading, at `level`.
    fn a_heading(&mut self, parent: usize, words: &str, level: u8) {
        self.push(
            parent,
            Node {
                role: access_roles::number_of(Role::Heading),
                name: words.to_owned(),
                states: access_roles::words_of(State::ReadOnly, None),
                attributes: vec![crate::access_attributes::a_heading_at(level)],
                ..Node::default()
            },
        );
    }

    /// A sentence the page says; `announced` where it appeared after a change.
    fn a_line(&mut self, parent: usize, words: &str, announced: bool) {
        if words.trim().is_empty() {
            return;
        }
        let state = if announced {
            State::AnnouncedWhenItChanges
        } else {
            State::ReadOnly
        };
        self.push(
            parent,
            Node {
                role: access_roles::number_of(Role::Label),
                name: words.to_owned(),
                states: access_roles::words_of(state, None),
                announced,
                ..Node::default()
            },
        );
    }

    /// One section: its heading, its rows, its footnote.
    fn a_section(&mut self, window: usize, section: &ASection) {
        if let Some(heading) = &section.heading {
            self.a_heading(window, heading, A_SECTIONS_LEVEL);
        }
        let places = the_places_of_the_choices(&section.rows);
        for (row, place) in section.rows.iter().zip(places) {
            self.a_row(window, row, place);
        }
        if let Some(footnote) = &section.footnote {
            self.a_line(window, footnote, false);
        }
    }

    /// One row, and its button.
    fn a_row(&mut self, window: usize, row: &ARow, place: Option<(usize, usize)>) {
        let description = what_is_drawn_with(row);
        let (role, state, set) = match row.kind {
            TheKind::AChoice { chosen } | TheKind::ANestedChoice { chosen } => {
                (Role::RadioButton, State::OnOrOff, Some(chosen))
            }
            TheKind::ASwitch(is) => (Role::Switch, State::OnOrOff, Some(is == SwitchIs::On)),
            TheKind::AnAddingRow | TheKind::AnOpeningRow => (Role::Button, State::CanBeUsed, None),
            TheKind::ASlider(_) => (Role::Slider, State::CanBeUsed, None),
            TheKind::ASegmented | TheKind::ASwatches => (Role::List, State::ReadOnly, None),
            TheKind::Plain if row.row.is_some() => (Role::Button, State::CanBeUsed, None),
            TheKind::Plain => (Role::Label, State::ReadOnly, None),
        };
        let mut attributes = place.map_or_else(Vec::new, |(at, of)| its_place(at, of));
        attributes.extend(
            row.row
                .as_ref()
                .and_then(|it| it.its_old_name(self.strings)),
        );
        let node = Node {
            role: access_roles::number_of(role),
            name: row.title.clone(),
            description,
            states: access_roles::words_of(state, set),
            attributes,
            ..Node::default()
        };
        let at = match &row.row {
            Some(target) => self.push_a_target(window, target, node),
            None => self.push(window, node),
        };
        let parts = row.parts.len();
        for (place, part) in row.parts.iter().enumerate() {
            self.push_a_target(
                at,
                &part.row,
                Node {
                    role: access_roles::number_of(Role::RadioButton),
                    name: part.label.clone(),
                    states: access_roles::words_of(State::OnOrOff, Some(part.chosen)),
                    attributes: its_place(place, parts),
                    ..Node::default()
                },
            );
        }
        if let Some((label, target)) = &row.button {
            self.push_a_target(
                window,
                target,
                Node {
                    role: access_roles::number_of(Role::Button),
                    name: label.clone(),
                    states: access_roles::words_of(State::CanBeUsed, None),
                    ..Node::default()
                },
            );
        }
    }

    /// The list a row opened, while one is open: its search, its matches and
    /// the whole list — the keyboard on the entry it is on.
    fn the_list_open(&mut self, window: usize, open: &Open) {
        let Some(picker) = &open.picker else {
            return;
        };
        let search = self.push(
            window,
            Node {
                role: access_roles::number_of(Role::Entry),
                name: said(self.strings, w::SEARCH_THE_LIST),
                states: access_roles::words_of(State::CanBeUsed, None),
                ..Node::default()
            },
        );
        let matches = open.the_matches(picker.which, &picker.typed, self.strings);
        let every = open.every_entry(picker.which, self.strings);
        let whole = said(
            self.strings,
            match picker.which {
                crate::SettingsPicker::Region => w::ALL_COUNTRIES,
                crate::SettingsPicker::TimeZone => w::ALL_TIME_ZONES,
            },
        );
        let mut counted = 0;
        let mut lists = Vec::new();
        if !matches.is_empty() {
            lists.push((w::how_many_match(self.strings, matches.len()), matches));
        }
        lists.push((whole, every));
        for (heading, entries) in lists {
            let list = self.push(
                window,
                Node {
                    role: access_roles::number_of(Role::List),
                    name: heading,
                    states: access_roles::words_of(State::ReadOnly, None),
                    ..Node::default()
                },
            );
            let all = entries.len();
            for (place, entry) in entries.into_iter().enumerate() {
                let at = self.push(
                    list,
                    Node {
                        role: access_roles::number_of(Role::RadioButton),
                        name: entry.label,
                        states: access_roles::words_of(State::OnOrOff, Some(entry.chosen)),
                        press: Some(Pressed::InSettings(InSettings::AnEntry(entry.row))),
                        attributes: its_place(place, all),
                        ..Node::default()
                    },
                );
                if counted == picker.focus {
                    self.keyboard = Some(at);
                }
                counted += 1;
            }
        }
        // **The search has the keyboard while nothing in the list can**: the
        // drawing rings it for as long as the list is open.
        if self.keyboard.is_none() {
            self.keyboard = Some(search);
        }
    }

    /// The form that adds a provider, in place of the page.
    fn the_form(
        &mut self,
        window: usize,
        form: &AProviderBeingAddded,
        not_saved: Option<crate::a_provider_is_added_in_settings::WhyNotSaved>,
    ) {
        let category = SettingsCategory::AloAndAi;
        self.a_heading(window, &category.said(self.strings), THE_PAGES_LEVEL);
        self.a_line(window, &said(self.strings, category.is()), false);
        let words = crate::a_provider_being_added_is_drawn::the_form(form, not_saved, self.strings);
        for line in &words.said {
            self.a_line(window, line, true);
        }
        let which = form.which_field();
        for field in WhichField::EVERY {
            // **A field's contents never cross**: no `Text` is served on this
            // tree, and the key is a password field, which no reader reads.
            let role = if field.is_a_secret() {
                Role::PasswordEntry
            } else {
                Role::Entry
            };
            let at = self.push(
                window,
                Node {
                    role: access_roles::number_of(role),
                    name: crate::a_provider_being_added_is_drawn::label(field, self.strings),
                    states: access_roles::words_of(State::CanBeUsed, None),
                    ..Node::default()
                },
            );
            if field == which {
                self.keyboard = Some(at);
            }
        }
    }
}

/// Each row's place among the choices beside it, where it is one: the
/// choices of a section in one run, the smaller choices under one of them in
/// a run of their own.
fn the_places_of_the_choices(rows: &[ARow]) -> Vec<Option<(usize, usize)>> {
    let mut places = vec![None; rows.len()];
    let is_a_choice = |row: &ARow| matches!(row.kind, TheKind::AChoice { .. });
    let choices: Vec<usize> = (0..rows.len())
        .filter(|at| rows.get(*at).is_some_and(is_a_choice))
        .collect();
    for (place, at) in choices.iter().enumerate() {
        if let Some(it) = places.get_mut(*at) {
            *it = Some((place, choices.len()));
        }
    }
    let mut at = 0;
    while at < rows.len() {
        let nested = |row: &ARow| matches!(row.kind, TheKind::ANestedChoice { .. });
        let run = rows
            .get(at..)
            .map_or(0, |rest| rest.iter().take_while(|row| nested(row)).count());
        for place in 0..run {
            if let Some(it) = places.get_mut(at + place) {
                *it = Some((place, run));
            }
        }
        at += run.max(1);
    }
    places
}

/// **Everything drawn with a row besides its name**, in the order it is
/// drawn: its tag, the lines under it, what is at its end, its note and what
/// it risks. A switch's *On* or *Off* is left out, because the switch already
/// says which it is; *not set*, which is neither, is kept.
fn what_is_drawn_with(row: &ARow) -> String {
    let mut said: Vec<String> = Vec::new();
    said.extend(row.tag.iter().cloned());
    said.extend(row.lines.iter().cloned());
    let a_switch_that_says_which =
        matches!(row.kind, TheKind::ASwitch(SwitchIs::On | SwitchIs::Off));
    match &row.at_its_end {
        Some(AnEnd::Words(words)) if !a_switch_that_says_which => said.push(words.clone()),
        Some(AnEnd::Words(_)) | None => {}
        Some(AnEnd::Keys(keys)) => said.push(crate::access_attributes::keys_together(keys)),
        Some(AnEnd::AlwaysOn(words) | AnEnd::Picker(words)) => said.push(words.clone()),
    }
    if let Some(note) = &row.note {
        said.push(note.words.clone());
    }
    if let Some(risk) = &row.risk {
        said.push(risk.heading.clone());
        said.push(risk.words.clone());
    }
    said.retain(|line| !line.trim().is_empty());
    crate::access_attributes::told_together(&said)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "settings_as_read_tests.rs"]
mod tests;
