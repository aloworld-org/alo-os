//! Settings, as one place: every setting this machine has today, in one
//! window, each section reading and writing through the crate that owns it.
//!
//! `ROADMAP.md` v0.5: *Settings, as one place … not a scattering of dialogues
//! a person has to know the name of.* The sections, in the order drawn:
//!
//! 1. **what answers questions** — the four ways alo OS runs, as
//!    `alo-setting-up` offers them, and the choices the person's own
//!    `settings.toml` holds, changed through `alo_choosing::Choosing`
//!    (`crate::settings_answering`);
//! 2. **appearance** — the accent, through `alo_appearance::keeping`;
//! 3. **the dock** — its edge, through `alo_dock::keeping`;
//! 4. **shortcuts** — each action's chord, through `alo_shortcuts::keeping`;
//! 5. **what has been granted to what** — the grants and the pairings in one
//!    list, revoked with `alo_changing::Changing::revoked`
//!    (`crate::settings_granted`).
//!
//! # This window decides nothing
//!
//! Every value it shows is a crate's, every change goes through that crate,
//! and every sentence is that crate's. A change is made on a copy and only a
//! copy the owning crate kept replaces what is drawn, so what a person sees
//! and what the machine enforces cannot disagree. `tests/settings_source.rs`
//! holds these files to writing no file and naming no place of their own.
//!
//! # Only what the machine has is here
//!
//! A row is something a person can act on: choose it, rebind it, revoke it.
//! Nothing is drawn disabled. Terracotta is not among the accents, because
//! `alo_appearance::Accent` cannot be terracotta. A setting the machine has
//! no road to change from a window yet — a background picture, a text scale,
//! adding a provider — is absent rather than greyed out.
//!
//! # It is a person's, and it is reached by hand
//!
//! [`SettingsWindow::opened_by_hand`] takes where things are kept and a moment,
//! and nothing else: no turn, no model and no agent is involved or reachable
//! from these files, so nothing here is a road an agent could take to change a
//! setting, grant anything or keep a grant it was about to lose.

use std::time::SystemTime;

use alo_appearance::{Accent, Appearance};
use alo_changing::{Door, Row};
use alo_dock::{Dock, Edge};
use alo_nearby::Pairings;
use alo_remembering::NotRemembered;
use alo_shortcuts::{Action, Chord, Key, Modifiers, Shortcuts};
use alo_strings::Strings;

use crate::settings_answering::{AnsweringSection, SettingsAnswered, SettingsChoice};
use crate::settings_granted::{GrantedSection, SettingsRevoked};
use crate::settings_keepers::{AppearanceKept, DockKept, ShortcutsKept};
use crate::settings_kept::{KeptSection, SettingsKept};
use crate::{SettingsKey, SettingsPlaces};

/// The sections of Settings, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingsSection {
    /// What answers this person's questions.
    Answering,
    /// How the machine looks.
    Appearance,
    /// Which edge the dock is on.
    Dock,
    /// The shortcuts.
    Shortcuts,
    /// What has been granted to what.
    Granted,
}

/// One row a person can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsRow {
    /// A way to answer questions: Enter chooses it.
    Answer(SettingsChoice),
    /// An accent: Enter chooses it.
    Accent(Accent),
    /// An edge for the dock: Enter moves the dock there.
    Edge(Edge),
    /// An action's shortcut: Enter waits for a chord, Backspace puts it back.
    Shortcut(Action),
    /// A grant or a pairing: Enter revokes it.
    Granted(Row),
}

impl SettingsRow {
    /// The section the row is in.
    #[must_use]
    pub const fn section(&self) -> SettingsSection {
        match self {
            Self::Answer(_) => SettingsSection::Answering,
            Self::Accent(_) => SettingsSection::Appearance,
            Self::Edge(_) => SettingsSection::Dock,
            Self::Shortcut(_) => SettingsSection::Shortcuts,
            Self::Granted(_) => SettingsSection::Granted,
        }
    }
}

/// One key press as Settings receives it: what the key means, and the chord it
/// makes, for the moment Settings is waiting for one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsPress {
    /// What the key means.
    pub key: SettingsKey,
    /// The modifiers held and the key pressed, when the key is one a shortcut
    /// can be made of.
    pub chord: Option<(Modifiers, Key)>,
}

/// What a key press reaches outside the window, handed in for that press.
#[derive(Clone, Copy)]
pub struct SettingsDoors<'a> {
    /// The vocabulary the person reads, which words every refusal.
    pub strings: &'a Strings,
    /// The daemon's door, which hears a grant's revocation and revokes a
    /// pairing.
    pub daemon: &'a dyn Door,
    /// The moment of the press, for the grants and pairings in force.
    pub now: SystemTime,
}

/// What a key press did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsDid {
    /// A section kept in the person's folder was changed, or was not.
    Kept(SettingsSection, SettingsKept),
    /// What answers questions was chosen, or was not.
    Answered(SettingsAnswered),
    /// A row of the one list was revoked, or was not.
    Revoked(SettingsRevoked),
    /// Settings waits for the chord to bind the focused action to.
    WaitingForAChord,
    /// Settings stopped waiting for a chord without binding one.
    StoppedWaiting,
    /// The window closed.
    Closed,
}

/// What opening Settings read, for the host's log: every file of the
/// machine's own that did not read. A section whose own file did not read says
/// so in the window instead.
#[derive(Debug)]
pub struct SettingsOpened {
    /// The grants and pairings files that are there and did not read.
    pub not_remembered: Vec<NotRemembered>,
}

/// Settings, while it is open.
#[derive(Debug)]
pub(crate) struct Open {
    /// What answers questions.
    pub(crate) answering: AnsweringSection,
    /// Appearance.
    pub(crate) appearance: KeptSection<AppearanceKept>,
    /// The dock.
    pub(crate) dock: KeptSection<DockKept>,
    /// Shortcuts.
    pub(crate) shortcuts: KeptSection<ShortcutsKept>,
    /// Grants and pairings.
    pub(crate) granted: GrantedSection,
    /// Which row has the focus, counted from the first.
    pub(crate) focus: usize,
    /// Whether Settings waits for a chord for the focused shortcut.
    pub(crate) waiting_for_a_chord: bool,
}

/// The Settings window.
#[derive(Debug, Default)]
pub struct SettingsWindow {
    /// The sections, while the window is open.
    open: Option<Open>,
}

impl SettingsWindow {
    /// The window, closed.
    #[must_use]
    pub fn closed() -> Self {
        Self::default()
    }

    /// A person opened Settings, without asking any agent anything.
    ///
    /// Every section is read once, here, from where `places` says it is kept.
    /// Reopening an open window reads everything again.
    pub fn opened_by_hand(&mut self, places: &SettingsPlaces, now: SystemTime) -> SettingsOpened {
        let (granted, not_remembered) =
            GrantedSection::at_sign_in(places.grants(), places.pairings(), now);
        self.open = Some(Open {
            answering: AnsweringSection::at_sign_in(places.choosing()),
            appearance: KeptSection::at_sign_in(places.kept(alo_appearance::keeping::THE_FILE)),
            dock: KeptSection::at_sign_in(places.kept(alo_dock::keeping::THE_FILE)),
            shortcuts: KeptSection::at_sign_in(places.kept(alo_shortcuts::keeping::THE_FILE)),
            granted,
            focus: 0,
            waiting_for_a_chord: false,
        });
        SettingsOpened { not_remembered }
    }

    /// Close the window. Nothing is written by closing it.
    pub fn close(&mut self) {
        self.open = None;
    }

    /// Whether the window is open, so the host routes keys here.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Settings, while it is open.
    pub(crate) const fn open(&self) -> Option<&Open> {
        self.open.as_ref()
    }

    /// How the machine looks as Settings holds it — what the compositor draws
    /// from the moment a change is kept, because the two are one process.
    #[must_use]
    pub fn appearance(&self) -> Option<&Appearance> {
        self.open.as_ref().map(|open| open.appearance.drawn())
    }

    /// Where the dock is as Settings holds it.
    #[must_use]
    pub fn dock(&self) -> Option<&Dock> {
        self.open.as_ref().map(|open| open.dock.drawn())
    }

    /// The shortcuts as Settings holds them.
    #[must_use]
    pub fn shortcuts(&self) -> Option<&Shortcuts> {
        self.open.as_ref().map(|open| open.shortcuts.drawn())
    }

    /// Every row a person can act on, in the order drawn. Empty while closed.
    #[must_use]
    pub fn rows(&self, now: SystemTime) -> Vec<SettingsRow> {
        self.open
            .as_ref()
            .map(|open| open.rows(now))
            .unwrap_or_default()
    }

    /// The row with the focus, while the window is open and has any row.
    #[must_use]
    pub fn focused(&self, now: SystemTime) -> Option<SettingsRow> {
        let open = self.open.as_ref()?;
        let rows = open.rows(now);
        rows.get(open.focus.min(rows.len().saturating_sub(1)))
            .cloned()
    }

    /// Whether Settings waits for a chord.
    #[must_use]
    pub fn is_waiting_for_a_chord(&self) -> bool {
        self.open
            .as_ref()
            .is_some_and(|open| open.waiting_for_a_chord)
    }

    /// One key press. Nothing happens while the window is closed.
    pub fn pressed(
        &mut self,
        press: SettingsPress,
        doors: SettingsDoors<'_>,
    ) -> Option<SettingsDid> {
        let open = self.open.as_mut()?;
        if open.waiting_for_a_chord {
            return Some(open.chord_pressed(press, doors));
        }
        let rows = open.rows(doors.now);
        let last = rows.len().saturating_sub(1);
        open.focus = open.focus.min(last);
        let focused = rows.get(open.focus).cloned();
        match press.key {
            SettingsKey::Previous => open.focus = open.focus.saturating_sub(1),
            SettingsKey::Next => open.focus = (open.focus + 1).min(last),
            SettingsKey::First => open.focus = 0,
            SettingsKey::Last => open.focus = last,
            SettingsKey::NextSection => open.focus = next_section(&rows, open.focus, true),
            SettingsKey::PreviousSection => open.focus = next_section(&rows, open.focus, false),
            SettingsKey::Choose => return focused.map(|row| open.chosen(&row, doors)),
            SettingsKey::PutBackAsShipped => {
                return focused.and_then(|row| open.put_back_as_shipped(&row, doors.strings));
            }
            SettingsKey::PutBackThisOne => {
                if let Some(SettingsRow::Shortcut(action)) = focused {
                    let kept = open.shortcuts.changed(
                        |shortcuts| {
                            shortcuts.reset(action);
                            Ok(())
                        },
                        doors.strings,
                    );
                    return Some(SettingsDid::Kept(SettingsSection::Shortcuts, kept));
                }
            }
            SettingsKey::Close => {
                self.open = None;
                return Some(SettingsDid::Closed);
            }
            SettingsKey::Nothing => {}
        }
        None
    }
}

impl Open {
    /// Every row, in the order drawn.
    pub(crate) fn rows(&self, now: SystemTime) -> Vec<SettingsRow> {
        let mut rows = Vec::new();
        let pairings = self.pairings();
        for (_, choices) in self.answering.offered(&pairings, now) {
            rows.extend(choices.into_iter().map(SettingsRow::Answer));
        }
        if self.appearance.at().is_some() {
            rows.extend(Accent::ALL.into_iter().map(SettingsRow::Accent));
        }
        if self.dock.at().is_some() {
            rows.extend(Edge::ALL.into_iter().map(SettingsRow::Edge));
        }
        if self.shortcuts.at().is_some() {
            rows.extend(
                self.shortcuts
                    .drawn()
                    .bindings()
                    .map(|binding| SettingsRow::Shortcut(binding.action)),
            );
        }
        rows.extend(self.granted.rows(now).into_iter().map(SettingsRow::Granted));
        rows
    }

    /// The pairings as read, or none when they did not read.
    pub(crate) fn pairings(&self) -> Pairings {
        self.granted.pairings()
    }

    /// Enter on a row.
    fn chosen(&mut self, row: &SettingsRow, doors: SettingsDoors<'_>) -> SettingsDid {
        match row {
            SettingsRow::Answer(choice) => {
                let pairings = self.pairings();
                SettingsDid::Answered(self.answering.choose(
                    choice,
                    &pairings,
                    doors.now,
                    doors.strings,
                ))
            }
            SettingsRow::Accent(accent) => {
                let accent = *accent;
                let kept = self.appearance.changed(
                    |appearance| {
                        appearance.set_accent(accent);
                        Ok(())
                    },
                    doors.strings,
                );
                SettingsDid::Kept(SettingsSection::Appearance, kept)
            }
            SettingsRow::Edge(edge) => {
                let edge = *edge;
                let kept = self.dock.changed(
                    |dock| {
                        dock.set_edge(edge);
                        Ok(())
                    },
                    doors.strings,
                );
                SettingsDid::Kept(SettingsSection::Dock, kept)
            }
            SettingsRow::Shortcut(_) => {
                self.waiting_for_a_chord = true;
                SettingsDid::WaitingForAChord
            }
            SettingsRow::Granted(row) => SettingsDid::Revoked(self.granted.revoked(
                row,
                doors.daemon,
                doors.now,
                doors.strings,
            )),
        }
    }

    /// Delete on a row: its section put back as shipped, when its file did
    /// not read.
    fn put_back_as_shipped(&mut self, row: &SettingsRow, strings: &Strings) -> Option<SettingsDid> {
        let section = row.section();
        let kept = match section {
            SettingsSection::Appearance => self.appearance.put_back_as_shipped(strings),
            SettingsSection::Dock => self.dock.put_back_as_shipped(strings),
            SettingsSection::Shortcuts => self.shortcuts.put_back_as_shipped(strings),
            SettingsSection::Answering | SettingsSection::Granted => return None,
        };
        Some(SettingsDid::Kept(section, kept))
    }

    /// A key while Settings waits for a chord: Escape stops waiting, Backspace
    /// on its own leaves the action with no shortcut, and anything else is the
    /// chord — refused in `alo-shortcuts`' words when it cannot be one, or
    /// when another action has it.
    fn chord_pressed(&mut self, press: SettingsPress, doors: SettingsDoors<'_>) -> SettingsDid {
        let Some(SettingsRow::Shortcut(action)) = self.rows(doors.now).get(self.focus).cloned()
        else {
            self.waiting_for_a_chord = false;
            return SettingsDid::StoppedWaiting;
        };
        let Some((held, key)) = press.chord else {
            if press.key == SettingsKey::Close {
                self.waiting_for_a_chord = false;
                return SettingsDid::StoppedWaiting;
            }
            return SettingsDid::WaitingForAChord;
        };
        if held.is_empty() && key == Key::Escape {
            self.waiting_for_a_chord = false;
            return SettingsDid::StoppedWaiting;
        }
        self.waiting_for_a_chord = false;
        let strings = doors.strings;
        let kept = if held.is_empty() && key == Key::Backspace {
            self.shortcuts.changed(
                |shortcuts| {
                    shortcuts.unbind(action);
                    Ok(())
                },
                strings,
            )
        } else {
            self.shortcuts.changed(
                |shortcuts| {
                    let chord = Chord::checked(held, key)
                        .map_err(|refused| refused.said(strings).into_text())?;
                    shortcuts
                        .bind(action, chord)
                        .map_err(|taken| taken.said(strings).into_text())
                },
                strings,
            )
        };
        SettingsDid::Kept(SettingsSection::Shortcuts, kept)
    }
}

/// The first row of the next section after `from`, or of the one before.
fn next_section(rows: &[SettingsRow], from: usize, forward: bool) -> usize {
    let Some(here) = rows.get(from).map(SettingsRow::section) else {
        return from;
    };
    if forward {
        rows.iter()
            .enumerate()
            .skip(from)
            .find(|(_, row)| row.section() > here)
            .map_or(from, |(at, _)| at)
    } else {
        let Some(before) = rows
            .iter()
            .take(from)
            .rev()
            .map(SettingsRow::section)
            .find(|section| *section < here)
        else {
            return 0;
        };
        rows.iter()
            .position(|row| row.section() == before)
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[path = "settings_window_tests.rs"]
mod tests;
