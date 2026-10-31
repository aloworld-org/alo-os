//! The Terminal page — frame `551:98`: the two things alo can do in a person's
//! terminal, each a switch with what it changes and the file it adds a line
//! to; then the row that opens Running code.
//!
//! **A switch that is on stays on from here.** Each adds a line to the
//! person's shell startup file, and `crate::settings_shell_integration` has
//! no road to take one out — the file is the person's own, and writing it
//! back as alo ships it would throw away whatever else is in it. Pressing a
//! switch that is on says, in the terminal crate's own words, that it is
//! already there.

use alo_strings::Strings;

use crate::settings_categories::SettingsCategory;
use crate::settings_page::{
    ABlock, ANote, ARow, ASection, AnEnd, SwitchIs, TheKind, said, switch_said,
};
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_icons::ASettingsIcon;
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let mut blocks: Vec<ABlock> = open
        .shell_integration
        .said(strings)
        .into_iter()
        .map(ABlock::Said)
        .collect();
    let mut rows = Vec::new();
    for (section, row) in [
        (&open.shell_integration, SettingsRow::ShellIntegration),
        (&open.a_place_to_return_to, SettingsRow::APlaceToReturnTo),
    ] {
        if !section.offers_a_row() {
            continue;
        }
        let is = if section.is_already_there() {
            SwitchIs::On
        } else {
            SwitchIs::Off
        };
        let mut parts = section
            .row(strings)
            .into_iter()
            .map(|it| crate::settings_type::as_a_name(&it));
        let mut drawn = ARow::named(
            Some(row),
            TheKind::ASwitch(is),
            parts.next().unwrap_or_default(),
        );
        drawn.lines.extend(parts.next());
        if let Some(file) = parts.next() {
            let mut note = ANote::said(file, false);
            note.icon = ASettingsIcon::File;
            note.in_mono = true;
            drawn.note = Some(note);
        }
        drawn.at_its_end = Some(AnEnd::Words(switch_said(is, strings)));
        rows.push(drawn);
    }
    if !rows.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::IN_THE_TERMINAL)),
            rows,
            footnote: None,
        }));
    }
    let mut related = ARow::named(
        Some(SettingsRow::GoTo(SettingsCategory::RunningCode)),
        TheKind::AnOpeningRow,
        said(strings, w::HOW_ALO_RUNS_CODE),
    );
    related.lead = Some(ASettingsIcon::Code);
    related.lines.push(said(strings, w::HOW_ALO_RUNS_CODE_IS));
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::RELATED)),
        rows: vec![related],
        footnote: None,
    }));
    blocks
}
