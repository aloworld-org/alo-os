//! The Running code page — frame `550:874`: ADR 0064's three levels, each with
//! what it means, and what straight away risks; then the three protections
//! that stay on at every level.
//!
//! **A person who has never chosen has none of the three chosen**, though the
//! frame draws the sealed box chosen: `crate::how_code_runs_in_settings` holds
//! why *nobody has chosen* is a state of its own and not the sealed box. The
//! sealed box carries the frame's *Default* tag, which is true of it whether or
//! not anybody has chosen.
//!
//! **The three protections are rows with nothing to choose**, each marked
//! *Always on* with a lock: they are not a choice and must not read as one.

use alo_capability::{AlwaysOn, HowCodeRuns};
use alo_strings::{Filling, Strings};

use crate::settings_page::{ABlock, ARisk, ARow, ASection, AnEnd, TheKind, said};
use crate::settings_type::as_a_name;
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_icons::ASettingsIcon;
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let in_words = |word: alo_strings::Word| -> String {
        as_a_name(&strings.say(&word.key(), &Filling::nothing()).into_text())
    };
    let chosen = open.how_code_runs.chosen();
    let levels = HowCodeRuns::ALL
        .into_iter()
        .map(|level| {
            let mut row = ARow::named(
                Some(SettingsRow::HowCodeRuns(level)),
                TheKind::AChoice {
                    chosen: chosen == Some(level),
                },
                in_words(level.the_word()),
            );
            match (level, level.what_it_costs()) {
                (HowCodeRuns::InASealedBox, _) => {
                    row.tag = Some(said(strings, w::DEFAULT));
                    row.lines.push(said(strings, w::SEALED_BOX_IS));
                }
                (_, Some(cost)) => {
                    row.risk = Some(ARisk {
                        heading: said(strings, w::WHAT_IT_RISKS),
                        words: in_words(cost),
                    });
                }
                (_, None) => row.lines.push(said(strings, w::ASKING_IS)),
            }
            row
        })
        .collect();
    let always_on = said(strings, w::ALWAYS_ON);
    let stays_on = AlwaysOn::ALL
        .into_iter()
        .map(|it| {
            let mut row = ARow::named(None, TheKind::Plain, in_words(it.the_word()));
            row.lead = Some(match it {
                AlwaysOn::TheRecord => ASettingsIcon::History20,
                AlwaysOn::TheSnapshotBeforeEachRun => ASettingsIcon::Undo,
                AlwaysOn::TheEgressIndicator => ASettingsIcon::Egress,
            });
            row.at_its_end = Some(AnEnd::AlwaysOn(always_on.clone()));
            row
        })
        .collect();
    vec![
        ABlock::Section(ASection {
            heading: Some(said(strings, w::ALO_RUNS_CODE)),
            rows: levels,
            footnote: None,
        }),
        ABlock::Section(ASection {
            heading: Some(said(strings, w::STAYS_ON)),
            rows: stays_on,
            footnote: Some(said(strings, w::THEY_PROTECT_YOU)),
        }),
    ]
}
