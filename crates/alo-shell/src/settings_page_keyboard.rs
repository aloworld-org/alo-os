//! The Keyboard and shortcuts page — frame `551:870`: every shortcut, in one
//! list, each with its keys drawn as keys and a chevron to change it.
//!
//! Each key is `alo-shortcuts`' own name for it — *Super*, *Ctrl*, *A* — in
//! the person's language, one key to a cap, which is the design's drawing of
//! what `alo_shortcuts::Chord::shown` writes with `+` between them. An action
//! with no shortcut has no keys at its end.

use alo_strings::Strings;

use crate::settings_keepers::{Keeper, ShortcutsKept};
use crate::settings_page::{ABlock, ARow, ASection, AnEnd, TheKind, said};
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let section = &open.shortcuts;
    let mut blocks = Vec::new();
    if let Some(why) = section.not_read() {
        blocks.push(ABlock::Said(ShortcutsKept::not_read_said(why, strings)));
    }
    blocks.extend(section.told().map(|it| ABlock::Said(it.to_owned())));
    if section.at().is_none() {
        return blocks;
    }
    let rows = section
        .drawn()
        .bindings()
        .map(|binding| {
            let mut row = ARow::named(
                Some(SettingsRow::Shortcut(binding.action)),
                TheKind::AnOpeningRow,
                binding.action.said(strings).into_text(),
            );
            row.at_its_end = binding.chord.map(|chord| {
                let mut keys: Vec<String> = chord
                    .modifiers()
                    .held()
                    .map(|held| held.said(strings).into_text())
                    .collect();
                keys.push(chord.key().shown(strings));
                AnEnd::Keys(keys)
            });
            row
        })
        .collect();
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::SHORTCUTS)),
        rows,
        footnote: None,
    }));
    blocks
}
