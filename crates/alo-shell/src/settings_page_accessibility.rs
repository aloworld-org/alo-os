//! The Accessibility page — frame `551:1038`: the text size, as a slider, and
//! the pointer's size, as a segmented choice.
//!
//! **Text size makes an existing promise reachable**: `alo_appearance::TextScale`
//! has been kept and drawn by every surface since it was written, from 75 to
//! 300 per cent, and until this page nothing in Settings could change it.
//! Left and Right move it 25 per cent at a time.
//!
//! **The pointer's size is drawn and not built.** `docs/features.md` promises
//! *cursor size*, and nothing on this computer draws the pointer larger yet;
//! choosing *Large* or *Larger* says so and changes nothing, and *Default* is
//! marked because it is what the pointer is.

use alo_strings::Strings;

use crate::settings_keepers::{AppearanceKept, Keeper};
use crate::settings_page::{ABlock, APart, ARow, ASection, AnEnd, TheKind, said};
use crate::settings_window::{Open, PointerSize, SettingsRow};
use crate::the_settings_icons::ASettingsIcon;
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let mut blocks: Vec<ABlock> = open.told.iter().cloned().map(ABlock::Said).collect();
    let appearance = &open.appearance;
    if let Some(why) = appearance.not_read() {
        blocks.push(ABlock::Said(AppearanceKept::not_read_said(why, strings)));
    }
    if appearance.at().is_some() {
        let percent = appearance.drawn().text().as_percent();
        let mut row = ARow::named(
            Some(SettingsRow::TextSize),
            TheKind::ASlider(percent),
            said(strings, w::TEXT_SIZE),
        );
        row.lines.push(said(strings, w::TEXT_SIZE_IS));
        // **The percentage sits at the slider's end**, where `551:1123`
        // draws it.
        row.at_its_end = Some(AnEnd::Words(w::as_a_percentage(strings, percent)));
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::TEXT)),
            rows: vec![row],
            footnote: Some(said(strings, w::LARGER_TEXT_NOTE)),
        }));
    }
    let mut pointer = ARow::named(None, TheKind::ASegmented, said(strings, w::POINTER_SIZE));
    pointer.lead = Some(ASettingsIcon::Pointer);
    pointer.parts = PointerSize::ALL
        .into_iter()
        .map(|size| APart {
            label: said(
                strings,
                match size {
                    PointerSize::Default => w::DEFAULT,
                    PointerSize::Large => w::LARGE,
                    PointerSize::Larger => w::LARGER,
                },
            ),
            row: SettingsRow::PointerSize(size),
            chosen: size == PointerSize::Default,
            colour: None,
        })
        .collect();
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::POINTER)),
        rows: vec![pointer],
        footnote: None,
    }));
    blocks
}
