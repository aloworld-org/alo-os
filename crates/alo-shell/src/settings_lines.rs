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
//!   `alo_dock::Hiding::said`;
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
use alo_choosing::MayUseAnother;

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
        terminal(open, strings),
        how_code_runs(open, strings),
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

    // **The one row that is always here**, because every other row in this
    // section exists only for something the person already has — which is why
    // a fresh machine offered *not at all* and nothing else, and why this
    // section could say truthfully that adding a provider was reachable from
    // nowhere. A row that appeared once somebody had a provider would be a
    // door that opens after you are through it.
    if section.a_provider_can_be_added() {
        lines.push(LineWords {
            row: Some(SettingsRow::AddAProvider),
            parts: vec![
                strings
                    .say(
                        &crate::the_providers_words::ADDING_A_PROVIDER.key(),
                        &Filling::nothing(),
                    )
                    .into_text(),
            ],
            // **Never ticked.** The mark means *this is what your settings say
            // now*, and an action is not a state: ticking it would say a
            // person had chosen to be adding a provider.
            chosen: false,
        });
    }

    // **And, beneath the four, what alo may do when their choice is not
    // available** — `docs/features.md`: *Unless the person says otherwise, in
    // Settings*, the owner's instruction of 2026-10-18.
    //
    // Only for somebody who chose a provider or another machine: there is
    // nothing to cover for otherwise, and a row offered where it changes
    // nothing is the advertisement ADR 0009 rejects. **Absent rather than
    // drawn and disabled**, which is this section's rule for the four above.
    //
    // Neither row is ticked for a person who has not said. *Absent is stronger
    // than defaulted*: a tick for somebody who has never seen this would be a
    // default dressed as their choice.
    if section.another_is_a_question_at_all() {
        let said_already = section.may_use_another();
        for said in MayUseAnother::BOTH {
            lines.push(LineWords {
                chosen: said_already == Some(said),
                row: Some(SettingsRow::UsingAnother(said)),
                parts: vec![said.said(strings).into_text()],
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
        alo_dock::Hiding::ALL
            .into_iter()
            .map(|hiding| LineWords {
                row: Some(SettingsRow::Hiding(hiding)),
                parts: vec![hiding.said(strings).into_text()],
                chosen: section.drawn().hiding() == hiding,
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

/// The terminal: whether alo may read what a person types in it, and whether a
/// place to return to is kept before each command.
///
/// **Two rows, and three reasons there may be none of either** — no folder, a
/// shell alo ships nothing for, or a session that did not say which shell. The
/// first two say so above the rows that are not there; the third says nothing,
/// because there is nothing true to say about a shell nobody named.
///
/// **The sentences above come from the first row only**, because all three of
/// those reasons are about the shell and the folder and are therefore the same
/// for both. Saying them twice would tell a person with no folder, twice, that
/// there is nowhere to add anything.
fn terminal(open: &Open, strings: &Strings) -> SectionWords {
    let section = &open.shell_integration;
    let keeping = &open.a_place_to_return_to;
    let mut lines = Vec::new();
    if section.offers_a_row() {
        lines.push(LineWords {
            row: Some(SettingsRow::ShellIntegration),
            parts: section.row(strings),
            chosen: section.is_already_there(),
        });
    }
    if keeping.offers_a_row() {
        lines.push(LineWords {
            row: Some(SettingsRow::APlaceToReturnTo),
            parts: keeping.row(strings),
            chosen: keeping.is_already_there(),
        });
    }
    SectionWords {
        section: SettingsSection::Terminal,
        said: section.said(strings),
        lines,
    }
}

/// What has been granted to what.
fn granted(open: &Open, strings: &Strings, now: SystemTime) -> SectionWords {
    let section = &open.granted;

    // **While a picker is open it is the whole section.** A person walking
    // their own folders with the grants they already have underneath would be
    // moving one ring through two unrelated lists, and the key that opens a
    // folder would be the key that revokes a grant a row away.
    //
    // This and `Open::rows` are two readers of one question, held to each
    // other by `every_setting_the_machine_has_is_in_one_window` — *a row is
    // acted on that is not drawn, or drawn and not acted on*.
    if let Some(picking) = section.picking() {
        let mut said = vec![picking.asking(strings).into_text()];
        said.extend(picking.told().map(str::to_owned));
        return SectionWords {
            section: SettingsSection::Granted,
            said,
            lines: picking
                .rows()
                .into_iter()
                .map(|row| LineWords {
                    parts: vec![picking.said(&row, strings)],
                    row: Some(SettingsRow::Picking(row)),
                    chosen: false,
                })
                .collect(),
        };
    }

    let mut said: Vec<String> = section.told().to_vec();
    if let Some(nothing) = section.nothing_granted(now) {
        said.extend(nothing.said(strings).map(alo_strings::Said::into_text));
    }
    // **Above the list, and the only row here that is not something they
    // already have.** `AddAProvider` makes the same argument in the answering
    // section: a row that appeared once somebody had a grant would be a door
    // that opens after you are through it.
    let mut lines = Vec::new();
    if section.a_folder_can_be_given() {
        lines.push(LineWords {
            row: Some(SettingsRow::GrantAFolder),
            parts: vec![
                strings
                    .say(
                        &crate::the_pickers_words::GRANT_A_FOLDER.key(),
                        &Filling::nothing(),
                    )
                    .into_text(),
            ],
            chosen: false,
        });
    }
    lines.extend(section.rows(now).into_iter().map(|row| {
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
    }));
    SectionWords {
        section: SettingsSection::Granted,
        said,
        lines,
    }
}

/// How code the agent writes or fetches runs: ADR 0064's three levels.
///
/// **The three levels, each with its own cost, and then what every level
/// keeps.** The names and the costs are `alo_capability::HowCodeRuns`'s
/// answers, as everything else here is its crate's: nothing is worded in this
/// file.
///
/// **A person who has not chosen has no row marked**, which is why `chosen` is
/// compared against an `Option` rather than a value — and why there is no
/// shipped default to fall back on. `crate::how_code_runs_in_settings` holds
/// that argument.
///
/// **The three always-on protections are a line with no row**, like the four
/// ways of answering above: they are not a choice and must not read as one.
/// They are drawn at all because ADR 0064 keeps them on *at full trust too*,
/// and somebody reading only the cost of full trust would not know what they
/// still have.
///
/// **Nothing is said above the rows yet when the file will not read.**
/// `alo_remembering::NotRemembered` has no `said`, so there is no sentence in
/// the person's own language for why — and inventing one here would be
/// hardcoded English in the crate that draws. The section knows it did not
/// read; the words are owed by the crate that owns the file.
fn how_code_runs(open: &Open, strings: &Strings) -> SectionWords {
    let section = &open.how_code_runs;
    let chosen = section.chosen();
    let mut lines: Vec<LineWords> = alo_capability::HowCodeRuns::ALL
        .into_iter()
        .map(|level| {
            let mut parts = vec![
                strings
                    .say(&level.the_word().key(), &Filling::nothing())
                    .into_text(),
            ];
            parts.extend(
                level
                    .what_it_costs()
                    .map(|cost| strings.say(&cost.key(), &Filling::nothing()).into_text()),
            );
            LineWords {
                row: Some(SettingsRow::HowCodeRuns(level)),
                parts,
                chosen: chosen == Some(level),
            }
        })
        .collect();

    lines.push(LineWords {
        row: None,
        parts: alo_capability::HowCodeRuns::what_stays_on(
            alo_capability::HowCodeRuns::InASealedBox,
        )
        .into_iter()
        .map(|stays| {
            strings
                .say(&stays.the_word().key(), &Filling::nothing())
                .into_text()
        })
        .collect(),
        chosen: false,
    });

    SectionWords {
        section: SettingsSection::HowCodeRuns,
        said: Vec::new(),
        lines,
    }
}
