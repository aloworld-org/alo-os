//! The Accessibility page — frame `551:1038`: the text size, as a slider, and
//! the pointer's size and shape, as segmented choices.
//!
//! **Text size makes an existing promise reachable**: `alo_appearance::TextScale`
//! has been kept and drawn by every surface since it was written, from 75 to
//! 300 per cent, and until this page nothing in Settings could change it.
//! Left and Right move it 25 per cent at a time.
//!
//! **The pointer's size makes the promise of *cursor size* true**
//! (`docs/features.md`): *Large* is half as large again and *Larger* twice as
//! large, kept in `appearance.toml` and drawn by alo's own pointer from the next
//! frame, and told to applications as `XCURSOR_SIZE`. Its shape sits beside it
//! (`docs/design/the-settings-window.md`). Both are offered only where the
//! appearance was read, because that is where they are kept.

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
    if appearance.at().is_none() {
        blocks.extend(the_motion(open, strings));
        return blocks;
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
            chosen: size == appearance.drawn().pointer_size(),
            colour: None,
        })
        .collect();
    let mut rows = vec![pointer];
    // **Which arrow**, beside how large: the kite the owner chose on 2026-11-06,
    // and the familiar arrow kept as a choice. Not drawn in `551:1038`, which has
    // one row here; it is the design's own segmented choice, in the same
    // section (`docs/design/the-settings-window.md`).
    {
        let chosen = appearance.drawn().pointer_shape();
        let mut shape = ARow::named(None, TheKind::ASegmented, said(strings, w::POINTER_SHAPE));
        shape.lead = Some(ASettingsIcon::Pointer);
        shape.parts = alo_appearance::PointerShape::ALL
            .into_iter()
            .map(|each| APart {
                label: said(
                    strings,
                    match each {
                        alo_appearance::PointerShape::Kite => w::KITE,
                        alo_appearance::PointerShape::Classic => w::CLASSIC,
                    },
                ),
                row: SettingsRow::PointerShape(each),
                chosen: each == chosen,
                colour: None,
            })
            .collect();
        rows.push(shape);
    }
    // **What colour**: navy as it ships, white with a navy edge for dark
    // backgrounds, or black — the owner's yes of 2026-11-06. Never deep teal,
    // which is the agent's. Also not drawn in `551:1038`; the same segmented
    // choice, in the same section (`docs/design/the-settings-window.md`).
    {
        let chosen = appearance.drawn().pointer_colour();
        let mut colour = ARow::named(None, TheKind::ASegmented, said(strings, w::POINTER_COLOUR));
        colour.lead = Some(ASettingsIcon::Pointer);
        colour.parts = alo_appearance::PointerColour::ALL
            .into_iter()
            .map(|each| APart {
                label: said(
                    strings,
                    match each {
                        alo_appearance::PointerColour::Navy => w::NAVY,
                        alo_appearance::PointerColour::White => w::WHITE,
                        alo_appearance::PointerColour::Black => w::BLACK,
                    },
                ),
                row: SettingsRow::PointerColour(each),
                chosen: each == chosen,
                colour: None,
            })
            .collect();
        rows.push(colour);
    }
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::POINTER)),
        rows,
        footnote: None,
    }));
    blocks.extend(the_motion(open, strings));
    blocks
}

/// **Motion**: the *Reduced motion* switch, the owner's yes of 2026-11-07.
///
/// Not drawn in `551:1038`, which has Text and Pointer only, so it is a third
/// section in the design's own switch row (`550:256`, `551:196`) —
/// `docs/design/the-settings-window.md`, question 9. Kept in the person's
/// `access.toml`, so offered only where that file can be kept, and off as this
/// computer comes.
fn the_motion(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let access = &open.access;
    let mut blocks = Vec::new();
    if let Some(why) = access.not_read() {
        blocks.push(ABlock::Said(
            crate::settings_keepers::AccessKept::not_read_said(why, strings),
        ));
    }
    blocks.extend(access.told().map(|it| ABlock::Said(it.to_owned())));
    if access.at().is_none() || access.not_read().is_some() {
        return blocks;
    }
    let is = if access.drawn().has(alo_access::Setting::ReducedMotion) {
        crate::settings_page::SwitchIs::On
    } else {
        crate::settings_page::SwitchIs::Off
    };
    let mut row = ARow::named(
        Some(SettingsRow::ReducedMotion),
        TheKind::ASwitch(is),
        said(strings, w::REDUCED_MOTION),
    );
    row.lines.push(said(strings, w::REDUCED_MOTION_IS));
    row.at_its_end = Some(AnEnd::Words(crate::settings_page::switch_said(is, strings)));
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::MOTION)),
        rows: vec![row],
        footnote: None,
    }));
    blocks.push(the_applications(open, strings));
    blocks
}

/// **Applications**: *Apps describe themselves to alo and screen readers*, the
/// owner's decision of 2026-11-11 — on as this computer comes, with what it
/// costs said under it, so a person can turn it off knowing what they give up.
///
/// Drawn only where *Motion* is, because it is kept in the same `access.toml`;
/// a fourth section in the design's own switch row, as Motion is the third
/// (`docs/design/the-settings-window.md`, question 10).
fn the_applications(open: &Open, strings: &Strings) -> ABlock {
    let is = if open.access.drawn().apps_describe_themselves() {
        crate::settings_page::SwitchIs::On
    } else {
        crate::settings_page::SwitchIs::Off
    };
    let mut row = ARow::named(
        Some(SettingsRow::AppsDescribeThemselves),
        TheKind::ASwitch(is),
        said(strings, w::APPS_DESCRIBE_THEMSELVES),
    );
    row.lines
        .push(said(strings, w::APPS_DESCRIBE_THEMSELVES_IS));
    row.lines
        .push(said(strings, w::APPS_DESCRIBE_THEMSELVES_LATER));
    row.at_its_end = Some(AnEnd::Words(crate::settings_page::switch_said(is, strings)));
    ABlock::Section(ASection {
        heading: Some(said(strings, w::APPLICATIONS)),
        rows: vec![row],
        footnote: None,
    })
}
