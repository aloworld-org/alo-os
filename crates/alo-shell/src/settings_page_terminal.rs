//! The Terminal page — frame `551:98`: the things alo can do in a person's
//! terminal, each a switch with what it changes; then the row that opens
//! Running code.
//!
//! **The first two switches stay on from here.** Each adds a line to the
//! person's shell startup file, and `crate::settings_shell_integration` has
//! no road to take one out — the file is the person's own, and writing it
//! back as alo ships it would throw away whatever else is in it. Pressing a
//! switch that is on says, in the terminal crate's own words, that it is
//! already there.
//!
//! **The third turns either way in one press**: letting alo act without asking
//! in the terminal (the owner's decision of 2026-11-06), which is a line in a
//! file of alo's and not the person's startup file, so there is nothing of
//! theirs to lose by taking it out. `crate::alo_in_the_terminal_in_settings`
//! keeps it.

use alo_strings::Strings;

use crate::settings_categories::SettingsCategory;
use crate::settings_page::{
    ABlock, ANote, ARisk, ARow, ASection, AnEnd, SwitchIs, TheKind, said, switch_said,
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
    rows.push(acting_without_asking(open, strings));
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::IN_THE_TERMINAL)),
        rows,
        footnote: None,
    }));
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

/// The switch that lets what the person asks alo for in the terminal run
/// without the OK, with what it costs beneath it.
///
/// **The design has no frame for it** — `551:98` was drawn with two switches,
/// before the owner's decision of 2026-11-06 — so it is built from the two
/// pieces the design already has: this page's own switch row (`551:168`) and
/// the Running code page's *What it risks* box (`550:874`), which is where the
/// design says a cost in plain words goes. `docs/design/the-settings-window.md`
/// records the departure.
///
/// **The cost is shown whichever way the switch is**, because it is what a
/// person needs to read *before* switching it on; drawn only once it was on, it
/// would arrive after the decision it informs.
fn acting_without_asking(open: &Open, strings: &Strings) -> ARow {
    let answer = open.alo_in_the_terminal.is();
    let is = if answer.acts_without_asking() {
        SwitchIs::On
    } else {
        SwitchIs::Off
    };
    let mut drawn = ARow::named(
        Some(SettingsRow::ActWithoutAsking),
        TheKind::ASwitch(is),
        said(strings, w::ACT_WITHOUT_ASKING),
    );
    drawn.lines.push(said(strings, w::ACT_WITHOUT_ASKING_IS));
    if let Some(cost) = alo_capability::AloInTheTerminal::ActsWithoutAsking.what_it_costs() {
        drawn.risk = Some(ARisk {
            heading: said(strings, w::WHAT_IT_RISKS),
            // **Not drawn as a name**: it begins with *alo*, which is never
            // written with a capital.
            words: said(strings, cost),
        });
    }
    drawn.at_its_end = Some(AnEnd::Words(switch_said(is, strings)));
    drawn
}

#[cfg(test)]
#[path = "settings_page_terminal_tests.rs"]
mod tests;
