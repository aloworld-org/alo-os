//! Each section of Settings as the words drawn for it, in the order a person
//! reads them.
//!
//! Nothing here is worded. Every piece of text is a crate's answer:
//!
//! - **what answers questions**: each of the four `alo_setting_up::Offered`
//!   has its name and its one line; under it, a brought model's name, a
//!   provider's model and the provider's name, or a paired machine's identity —
//!   the person's own data, never translated;
//! - **appearance** and **the dock**: `alo_appearance::Accent::said` and
//!   `alo_dock::Edge::said`;
//! - **shortcuts**: `alo_shortcuts::Action::said`, and the chord as
//!   `alo_shortcuts::Chord::shown` writes it — nothing beside an action with no
//!   shortcut;
//! - **what has been granted to what**: `alo_granted::Seen::said` for a grant,
//!   and for a pairing the machine's name or identity with what the pairing
//!   permits, as `alo_nearby` words it.
//!
//! Above a section's rows go the sentences it has to say — its file did not
//! read, a change was refused, a revocation came back — in the owning crate's
//! words. Which row is chosen is a mark, never a word: no crate declares one,
//! and a word invented here would be the drawing crate deciding.

use std::time::SystemTime;

use alo_changing::Row;
use alo_setting_up::Offered;
use alo_strings::{Filling, Strings};

use crate::settings_answering::SettingsChoice;
use crate::settings_keepers::{AppearanceKept, DockKept, Keeper, ShortcutsKept};
use crate::settings_window::{Open, SettingsRow, SettingsSection};

/// One line of a section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LineWords {
    /// The row a person acts on, or [`None`] for a line that names one of the
    /// four ways alo OS runs above the choices under it.
    pub(crate) row: Option<SettingsRow>,
    /// The pieces of text, in reading order, each drawn whole.
    pub(crate) parts: Vec<String>,
    /// Whether this is what the person's settings are now.
    pub(crate) chosen: bool,
}

/// One section's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SectionWords {
    /// Which section.
    pub(crate) section: SettingsSection,
    /// What the section says above its rows.
    pub(crate) said: Vec<String>,
    /// Its lines.
    pub(crate) lines: Vec<LineWords>,
}

/// Every section's words, in the order drawn. A section with nothing to say
/// and no line is left out.
pub(crate) fn sections(open: &Open, strings: &Strings, now: SystemTime) -> Vec<SectionWords> {
    let mut drawn = vec![
        answering(open, strings, now),
        appearance(open, strings),
        dock(open, strings),
        shortcuts(open, strings),
        granted(open, strings, now),
    ];
    drawn.retain(|section| !section.said.is_empty() || !section.lines.is_empty());
    drawn
}

/// What answers questions.
fn answering(open: &Open, strings: &Strings, now: SystemTime) -> SectionWords {
    let section = &open.answering;
    let mut said = Vec::new();
    if let Some(why) = section.not_read() {
        said.push(why.said(strings).into_text());
    }
    said.extend(section.told().map(str::to_owned));
    let chosen_way = section.chosen_way();
    let chosen = section.chosen();
    let mut lines = Vec::new();
    for (offered, choices) in section.offered(&open.pairings(), now) {
        let head = vec![
            offered.called(strings).into_text(),
            offered.described(strings).into_text(),
        ];
        if offered == Offered::NotAtAll {
            lines.push(LineWords {
                row: Some(SettingsRow::Answer(SettingsChoice::NotAtAll)),
                parts: head,
                chosen: chosen_way == Some(offered),
            });
            continue;
        }
        lines.push(LineWords {
            row: None,
            parts: head,
            chosen: chosen_way == Some(offered),
        });
        for choice in choices {
            let parts = match &choice {
                SettingsChoice::Brought(name) | SettingsChoice::Paired(name) => vec![name.clone()],
                SettingsChoice::Provider { provider, model } => {
                    vec![model.clone(), provider.clone()]
                }
                SettingsChoice::NotAtAll => Vec::new(),
            };
            lines.push(LineWords {
                chosen: chosen.as_ref() == Some(&choice),
                row: Some(SettingsRow::Answer(choice)),
                parts,
            });
        }
    }
    SectionWords {
        section: SettingsSection::Answering,
        said,
        lines,
    }
}

/// Appearance.
fn appearance(open: &Open, strings: &Strings) -> SectionWords {
    let section = &open.appearance;
    let mut said = Vec::new();
    if let Some(why) = section.not_read() {
        said.push(AppearanceKept::not_read_said(why, strings));
    }
    said.extend(section.told().map(str::to_owned));
    let lines = if section.at().is_some() {
        alo_appearance::Accent::ALL
            .into_iter()
            .map(|accent| LineWords {
                row: Some(SettingsRow::Accent(accent)),
                parts: vec![accent.said(strings).into_text()],
                chosen: section.drawn().accent() == accent,
            })
            .collect()
    } else {
        Vec::new()
    };
    SectionWords {
        section: SettingsSection::Appearance,
        said,
        lines,
    }
}

/// The dock.
fn dock(open: &Open, strings: &Strings) -> SectionWords {
    let section = &open.dock;
    let mut said = Vec::new();
    if let Some(why) = section.not_read() {
        said.push(DockKept::not_read_said(why, strings));
    }
    said.extend(section.told().map(str::to_owned));
    let lines = if section.at().is_some() {
        alo_dock::Edge::ALL
            .into_iter()
            .map(|edge| LineWords {
                row: Some(SettingsRow::Edge(edge)),
                parts: vec![edge.said(strings).into_text()],
                chosen: section.drawn().edge() == edge,
            })
            .collect()
    } else {
        Vec::new()
    };
    SectionWords {
        section: SettingsSection::Dock,
        said,
        lines,
    }
}

/// Shortcuts.
fn shortcuts(open: &Open, strings: &Strings) -> SectionWords {
    let section = &open.shortcuts;
    let mut said = Vec::new();
    if let Some(why) = section.not_read() {
        said.push(ShortcutsKept::not_read_said(why, strings));
    }
    said.extend(section.told().map(str::to_owned));
    let lines = if section.at().is_some() {
        section
            .drawn()
            .bindings()
            .map(|binding| {
                let mut parts = vec![binding.action.said(strings).into_text()];
                parts.extend(binding.chord.map(|chord| chord.shown(strings)));
                LineWords {
                    row: Some(SettingsRow::Shortcut(binding.action)),
                    parts,
                    chosen: false,
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    SectionWords {
        section: SettingsSection::Shortcuts,
        said,
        lines,
    }
}

/// What has been granted to what.
fn granted(open: &Open, strings: &Strings, now: SystemTime) -> SectionWords {
    let section = &open.granted;
    let mut said: Vec<String> = section.told().to_vec();
    if let Some(nothing) = section.nothing_granted(now) {
        said.extend(nothing.said(strings).map(alo_strings::Said::into_text));
    }
    let lines = section
        .rows(now)
        .into_iter()
        .map(|row| {
            let parts = match &row {
                Row::Grant(seen) => vec![seen.said(strings).into_text()],
                Row::Pairing(seen) => {
                    let mut parts = vec![
                        seen.called()
                            .unwrap_or_else(|| seen.machine().as_str())
                            .to_owned(),
                    ];
                    parts.extend(section.may(seen).into_iter().map(|may| {
                        strings
                            .say(&may.word().key(), &Filling::nothing())
                            .into_text()
                    }));
                    parts
                }
            };
            LineWords {
                row: Some(SettingsRow::Granted(row)),
                parts,
                chosen: false,
            }
        })
        .collect();
    SectionWords {
        section: SettingsSection::Granted,
        said,
        lines,
    }
}
