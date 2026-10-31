//! The Appearance page — frame `551:214`: the accent colours under *Look*, and
//! the Dock — which edge it sits on, and what it does when a window needs the
//! room.
//!
//! **Where the Dock sits is a promise made true here.** `docs/features.md`
//! promises *the person decides where it goes — bottom, left, right or top,
//! chosen in Settings*, restored by the owner the day after ADR 0076 withdrew
//! it; `alo_dock::Dock::set_edge` and the drawing both honour the choice, and
//! until this page nothing in Settings could make it.
//!
//! **Light and dark is not on this page yet.** The frame draws it first; it
//! arrives with the change that draws Settings dark, so that choosing *Dark*
//! is never a choice the window it is made in cannot show.

use alo_appearance::{Accent, Scheme};
use alo_dock::{Edge, Hiding};
use alo_strings::Strings;

use crate::settings_keepers::{AppearanceKept, DockKept, Keeper};
use crate::settings_page::{ABlock, APart, ARow, ASection, TheKind, said};
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let mut blocks = Vec::new();
    let appearance = &open.appearance;
    if let Some(why) = appearance.not_read() {
        blocks.push(ABlock::Said(AppearanceKept::not_read_said(why, strings)));
    }
    blocks.extend(appearance.told().map(|it| ABlock::Said(it.to_owned())));
    if appearance.at().is_some() {
        let mut accent = ARow::named(None, TheKind::ASwatches, said(strings, w::ACCENT_COLOUR));
        accent.lines.push(said(strings, w::ACCENT_COLOUR_IS));
        accent.parts = Accent::ALL
            .into_iter()
            .map(|it| APart {
                label: it.said(strings).into_text(),
                row: SettingsRow::Accent(it),
                chosen: appearance.drawn().accent() == it,
                colour: Some({
                    let colour = it.on(Scheme::Light);
                    [colour.red(), colour.green(), colour.blue()]
                }),
            })
            .collect();
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::LOOK)),
            rows: vec![accent],
            footnote: Some(said(strings, w::THE_ACCENT_NOTE)),
        }));
    }

    let dock = &open.dock;
    if let Some(why) = dock.not_read() {
        blocks.push(ABlock::Said(DockKept::not_read_said(why, strings)));
    }
    blocks.extend(dock.told().map(|it| ABlock::Said(it.to_owned())));
    if dock.at().is_some() {
        let mut edge = ARow::named(
            None,
            TheKind::ASegmented,
            said(strings, w::WHERE_THE_DOCK_SITS),
        );
        edge.parts = Edge::EVERY
            .into_iter()
            .map(|it| APart {
                label: said(
                    strings,
                    match it {
                        Edge::Bottom => w::BOTTOM,
                        Edge::Left => w::LEFT,
                        Edge::Right => w::RIGHT,
                        Edge::Top => w::TOP,
                    },
                ),
                row: SettingsRow::Edge(it),
                chosen: dock.drawn().edge() == it,
                colour: None,
            })
            .collect();
        let mut rows = vec![edge];
        rows.extend(Hiding::ALL.into_iter().map(|hiding| {
            let mut row = ARow::named(
                Some(SettingsRow::Hiding(hiding)),
                TheKind::AChoice {
                    chosen: dock.drawn().hiding() == hiding,
                },
                hiding.said(strings).into_text(),
            );
            row.lines.push(said(
                strings,
                match hiding {
                    Hiding::Never => w::ALWAYS_SHOWN_IS,
                    Hiding::WhenAWindowNeedsTheRoom => w::GIVES_WAY_IS,
                },
            ));
            row
        }));
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::THE_DOCK)),
            rows,
            footnote: None,
        }));
    }
    blocks
}
