//! The Default apps page — frame `25:2`: *Web links* and *Email addresses*,
//! each a card of the installed applications that open that kind, the one
//! that opens it now ringed, and the footnote saying every link opened is in
//! the record (`docs/design/default-apps-and-open-with.md`).
//!
//! **A section with nothing installed says so in a plain line** rather than
//! drawing an empty card, and no row is ever greyed — the owner's approval of
//! 2026-11-12 as the coordinator relayed it.
//!
//! **What each row's line says** is the frame's: the browser opening links now
//! says so, any other says it is installed, and the mail application opening
//! addresses now says what it does with one.

use alo_applications::LinkKind;
use alo_strings::Strings;

use crate::settings_keepers::LinksKept;
use crate::settings_page::{ABlock, ARow, ASection, AnApplicationsMark, TheKind, said};
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let section = &open.default_apps;
    let mut blocks = Vec::new();
    if let Some(why) = section.kept().not_read() {
        blocks.push(ABlock::Said(
            <LinksKept as crate::settings_keepers::Keeper>::not_read_said(why, strings),
        ));
    }
    blocks.extend(section.kept().told().map(|it| ABlock::Said(it.to_owned())));
    blocks.extend(section.told().map(|it| ABlock::Said(it.to_owned())));
    if !section.is_offered() {
        return blocks;
    }
    let mut sections: Vec<ASection> = LinkKind::EVERY
        .into_iter()
        .map(|kind| a_section(open, kind, strings))
        .collect();
    // **The footnote is under the last card**, as `25:51` sits under `25:43`;
    // where that section has no card its plain line is its footnote, so the
    // record's sentence goes in a section of its own beneath.
    let record = said(strings, w::EVERY_LINK_IS_RECORDED);
    match sections.last_mut() {
        Some(last) if !last.rows.is_empty() => last.footnote = Some(record),
        _ => sections.push(ASection {
            heading: None,
            rows: Vec::new(),
            footnote: Some(record),
        }),
    }
    blocks.extend(sections.into_iter().map(ABlock::Section));
    blocks
}

/// One kind's section: its heading and a card of its handlers, or the line
/// saying there are none.
fn a_section(open: &Open, kind: LinkKind, strings: &Strings) -> ASection {
    let section = &open.default_apps;
    let (heading, none) = match kind {
        LinkKind::Web => (w::WEB_LINKS, w::NO_APP_OPENS_WEB_LINKS),
        LinkKind::Mail => (w::EMAIL_ADDRESSES, w::NO_APP_OPENS_EMAIL_ADDRESSES),
    };
    let now = section.opens_now(kind);
    let rows: Vec<ARow> = section
        .could_open(kind)
        .into_iter()
        .map(|application| {
            let identifier = application.identifier().to_owned();
            let chosen = now.as_deref() == Some(identifier.as_str());
            let name = application
                .name()
                .map(|called| called.shown(strings))
                .filter(|called| !called.trim().is_empty())
                .unwrap_or_else(|| identifier.clone());
            let line = match (kind, chosen) {
                (LinkKind::Web, true) => w::OPENS_WEB_LINKS_NOW,
                (LinkKind::Mail, true) => w::OPENS_A_NEW_MESSAGE,
                (_, false) => w::INSTALLED,
            };
            let letter = name.chars().next().map(String::from).unwrap_or_default();
            let mut row = ARow::named(
                Some(SettingsRow::DefaultFor(kind, identifier.clone())),
                TheKind::AChoice { chosen },
                name,
            );
            row.lines.push(said(strings, line));
            row.application = Some(AnApplicationsMark {
                picture: section.picture_of(&identifier),
                letter,
            });
            row
        })
        .collect();
    let footnote = rows.is_empty().then(|| said(strings, none));
    ASection {
        heading: Some(said(strings, heading)),
        rows,
        footnote,
    }
}
