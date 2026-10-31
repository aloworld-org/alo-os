//! The Folders alo can use page — frame `550:741`: the folders given to alo,
//! each with *Take back*; the button that gives it another; the record of what
//! it did; and the computers paired with this one.
//!
//! **While a folder is being picked the picker is the whole page**, because a
//! person walking their own folders
//! with the grants underneath would be moving one ring through two lists, and
//! the key that opens a folder would be the key that takes one back a row
//! away.
//!
//! **Paired computers are here**, under a heading of their own, because they
//! were in the old window's one list of things granted and the redesign draws
//! them nowhere. `docs/design/the-settings-window.md` asks the owner.

use std::time::SystemTime;

use alo_capability::Reach;
use alo_changing::Row;
use alo_strings::{Filling, Strings};

use crate::settings_page::{ABlock, ANote, ARow, ASection, AnAction, TheKind, said};
use crate::settings_type::as_a_name;
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_icons::ASettingsIcon;
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings, now: SystemTime) -> Vec<ABlock> {
    let section = &open.granted;
    if let Some(picking) = section.picking() {
        let mut blocks = vec![ABlock::Said(picking.asking(strings).into_text())];
        blocks.extend(picking.told().map(|it| ABlock::Said(it.to_owned())));
        let rows = picking
            .rows()
            .into_iter()
            .map(|row| {
                let words = picking.said(&row, strings);
                ARow::named(Some(SettingsRow::Picking(row)), TheKind::Plain, words)
            })
            .collect();
        blocks.push(ABlock::Section(ASection {
            heading: None,
            rows,
            footnote: None,
        }));
        return blocks;
    }

    let mut blocks: Vec<ABlock> = section
        .told()
        .iter()
        .map(|it| ABlock::Said(it.clone()))
        .collect();
    if let Some(nothing) = section.nothing_granted(now) {
        blocks.extend(nothing.said(strings).map(|it| ABlock::Said(it.into_text())));
    }
    let take_back = said(strings, w::TAKE_BACK);
    let (mut given, mut paired) = (Vec::new(), Vec::new());
    for row in section.rows(now) {
        match &row {
            Row::Grant(seen) => {
                let over = seen.over();
                let title = match over {
                    Reach::Folder(path) | Reach::File(path) => path.file_name().map_or_else(
                        || seen.to().to_owned(),
                        |it| it.to_string_lossy().into_owned(),
                    ),
                    Reach::Application(_) | Reach::Facility(_) => seen.to().to_owned(),
                };
                let mut drawn = ARow::named(None, TheKind::Plain, title);
                drawn.lead = Some(ASettingsIcon::Folder);
                drawn.lines.push(over.said(strings).into_text());
                if seen.lasts().is_until_removed() {
                    let mut note = ANote::said(
                        as_a_name(
                            &strings
                                .say(
                                    &crate::the_pickers_words::IT_LASTS_UNTIL_REMOVED.key(),
                                    &Filling::nothing(),
                                )
                                .into_text(),
                        ),
                        false,
                    );
                    note.icon = ASettingsIcon::History14;
                    drawn.note = Some(note);
                }
                drawn.button = Some((take_back.clone(), SettingsRow::Granted(row.clone())));
                given.push(drawn);
            }
            Row::Pairing(seen) => {
                let name = seen
                    .called()
                    .unwrap_or_else(|| seen.machine().as_str())
                    .to_owned();
                let mut drawn = ARow::named(None, TheKind::Plain, name);
                drawn.lines.extend(section.may(seen).into_iter().map(|may| {
                    strings
                        .say(&may.word().key(), &Filling::nothing())
                        .into_text()
                }));
                drawn.button = Some((take_back.clone(), SettingsRow::Granted(row.clone())));
                paired.push(drawn);
            }
        }
    }
    if !given.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::GIVEN_TO_ALO)),
            rows: given,
            footnote: Some(said(strings, w::TAKING_BACK_STOPS_IT)),
        }));
    }
    // **Always offered when it can be**: a row that appeared only once
    // somebody had a grant would be a door that opens after you are through
    // it.
    if section.a_folder_can_be_given() {
        blocks.push(ABlock::AnAction(AnAction {
            label: strings
                .say(
                    &crate::the_pickers_words::GRANT_A_FOLDER.key(),
                    &Filling::nothing(),
                )
                .into_text(),
            row: SettingsRow::GrantAFolder,
            beside: said(strings, w::YOU_CHOOSE_FOLDER_AND_HOW_LONG),
        }));
    }
    let mut record = ARow::named(
        Some(SettingsRow::SeeWhatAloDid),
        TheKind::AnOpeningRow,
        strings
            .say(
                &alo_shortcuts::Action::SeeWhatAloDid.word().key(),
                &Filling::nothing(),
            )
            .into_text(),
    );
    record.lead = Some(ASettingsIcon::History20);
    record
        .lines
        .push(said(strings, w::EVERY_ACTION_AND_REFUSAL));
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::WHAT_ALO_DID)),
        rows: vec![record],
        footnote: None,
    }));
    if !paired.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::PAIRED_MACHINES)),
            rows: paired,
            footnote: None,
        }));
    }
    blocks
}
