//! The Region and time page — frame `551:661`: where this computer is, and its
//! time zone, each a row with a button that opens its list
//! (`crate::settings_picker`).
//!
//! **What the frame draws and this page does not, yet.** The time zone's line
//! — *It is 14:32 here now* — is a clock, and this window draws no clock: the
//! status area does. The zone is named by its city, as `alo_formats` names it;
//! the frame's *Central European Time* is a name nothing on this machine holds.

use alo_strings::Strings;

use crate::settings_page::{ABlock, ARow, ASection, AnEnd, TheKind, said};
use crate::settings_picker::SettingsPicker;
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let mut blocks: Vec<ABlock> = open
        .formats
        .said(strings)
        .into_iter()
        .chain(open.time_zone.said())
        .map(ABlock::Said)
        .collect();
    let mut rows = Vec::new();
    if open.formats.is_offered() {
        let now = open.formats.drawn().region();
        let mut row = ARow::named(
            Some(SettingsRow::OpenThePicker(SettingsPicker::Region)),
            TheKind::Plain,
            said(strings, w::WHERE_THIS_COMPUTER_IS),
        );
        row.lines.push(said(strings, w::CHOOSE_A_COUNTRY));
        row.at_its_end = Some(AnEnd::Picker(open.formats.called(now, strings)));
        rows.push(row);
    }
    if !open.time_zone.choices().is_empty() {
        let mut row = ARow::named(
            Some(SettingsRow::OpenThePicker(SettingsPicker::TimeZone)),
            TheKind::Plain,
            said(strings, w::TIME_ZONE),
        );
        row.at_its_end = Some(AnEnd::Picker(
            open.time_zone
                .now()
                .map(|zone| open.time_zone.called(zone))
                .unwrap_or_default(),
        ));
        rows.push(row);
    }
    if !rows.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::WHERE_THIS_COMPUTER_IS)),
            rows,
            footnote: None,
        }));
    }
    blocks
}
