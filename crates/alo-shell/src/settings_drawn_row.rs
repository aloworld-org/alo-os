//! One row of a Settings card, laid out: what is at its start, its words, and
//! what is at its end — from the frames' rows (`550:169`, `550:191`, `550:219`,
//! `550:241`, `550:819`, `551:332`, `551:940`).
//!
//! # Where things sit on a row
//!
//! - **A choice keeps its words at the top**, its ring level with its name:
//!   the frames draw those rows `items-start`.
//! - **Every other row is centred in its least height** — a row that opens
//!   something, one that only says something, one with a lead icon or a
//!   segmented choice — as the frames draw them `items-center`.
//! - What is at the end — a switch with its word, keys, *Always on*, a
//!   chevron, a segmented choice or a button — takes its room first, so the
//!   words wrap in what is left.

use smithay::utils::Rectangle;

use crate::settings_drawn_controls::{
    ASegmented, AnEndShaped, a_choice, a_lead, a_risk, a_switch, a_tag, the_swatches,
};
use crate::settings_drawn_page::{ATargetDrawn, Drawn, Laying, ThePageDrawn, moved_down, text};
use crate::settings_measures as m;
use crate::settings_page::{ARow, AnEnd, TheKind};
use crate::settings_type::TheType;
use crate::the_settings_icons::ASettingsIcon;

/// One row of a card, from its top; answer its bottom.
pub(crate) fn a_row(
    out: &mut ThePageDrawn,
    laying: &mut Laying<'_>,
    top: i32,
    row: &ARow,
    section_starts: i32,
) -> i32 {
    let palette = laying.palette;
    let line = laying.px(m::A_HAIRLINE);
    let nested = matches!(
        row.kind,
        TheKind::ANestedChoice { .. } | TheKind::AnAddingRow
    );
    let (before, beside, above, gap) = if nested {
        (
            laying.px(m::A_NESTED_ROW_IN),
            laying.px(m::BESIDE_A_ROW),
            laying.px(m::ABOVE_A_NESTED_ROW),
            laying.px(m::INSIDE_A_NESTED_ROW),
        )
    } else {
        (
            laying.px(m::BESIDE_A_ROW),
            laying.px(m::BESIDE_A_ROW),
            laying.px(m::ABOVE_A_ROW),
            laying.px(m::INSIDE_A_ROW),
        )
    };
    let inside_left = laying.left + line;
    let inside_wide = laying.wide - 2 * line;
    let mark_at = inside_left + before;

    // What is at the row's start.
    let mark = match (row.kind, row.lead) {
        (_, Some(_)) => laying.px(m::A_LEADS_WELL),
        (TheKind::AChoice { .. }, None) => laying.px(m::A_CHOICE),
        (TheKind::ANestedChoice { .. } | TheKind::AnAddingRow, None) => {
            laying.px(m::A_NESTED_CHOICE)
        }
        _ => 0,
    };
    let x = if mark > 0 {
        mark_at + mark + gap
    } else {
        mark_at
    };

    // What is at its end, measured first so the words know their room.
    let end = row.at_its_end.as_ref().map(|end| match end {
        AnEnd::Words(words) => AnEndShaped::Words(
            laying
                .words(
                    words,
                    TheType::Label,
                    inside_wide / 3,
                    (palette.surface, palette.muted),
                )
                .cropped_to_its_ink(),
            words.clone(),
        ),
        AnEnd::Keys(keys) => AnEndShaped::Keys(
            keys.iter()
                .map(|key| {
                    (
                        laying
                            .words(
                                key,
                                TheType::Label,
                                inside_wide / 3,
                                (palette.cool, palette.ink),
                            )
                            .cropped_to_its_ink(),
                        key.clone(),
                    )
                })
                .collect(),
        ),
        AnEnd::Picker(words) => AnEndShaped::Picker(
            laying
                .words(
                    words,
                    TheType::Body,
                    inside_wide / 2,
                    (palette.surface, palette.ink),
                )
                .cropped_to_its_ink(),
            words.clone(),
        ),
        AnEnd::AlwaysOn(words) => AnEndShaped::AlwaysOn(
            laying
                .words(
                    words,
                    TheType::Label,
                    inside_wide / 3,
                    (palette.surface, palette.positive),
                )
                .cropped_to_its_ink(),
            words.clone(),
        ),
    });
    let segmented = (row.kind == TheKind::ASegmented).then(|| ASegmented::of(laying, &row.parts));
    let button = row.button.as_ref().map(|(label, _)| {
        let shaped = laying
            .words(
                label,
                TheType::Label,
                inside_wide / 3,
                (palette.surface, palette.ink),
            )
            .cropped_to_its_ink();
        let wide = shaped.width_of_box + 2 * laying.px(m::BESIDE_A_BUTTONS_LABEL);
        (shaped, wide)
    });
    let opens = matches!(row.kind, TheKind::AnOpeningRow | TheKind::AnAddingRow);
    let chevron = laying.px(m::A_CHEVRON);
    let switch = laying.px(m::A_SWITCH_IS_WIDE);
    let mut pieces = Vec::new();
    if opens {
        pieces.push(chevron);
    }
    if let TheKind::ASwitch(_) = row.kind {
        pieces.push(switch);
    }
    if let Some(end) = &end {
        pieces.push(
            end.wide(laying)
                + if matches!(row.kind, TheKind::ASwitch(_)) {
                    laying.px(m::BEFORE_A_SWITCH) - gap
                } else {
                    0
                },
        );
    }
    if let Some(segmented) = &segmented {
        pieces.push(segmented.wide(laying));
    }
    if let Some((_, wide)) = &button {
        pieces.push(*wide);
    }
    let end_wide: i32 = pieces.iter().sum::<i32>() + gap * pieces.len() as i32;
    let right = inside_left + inside_wide - beside - end_wide;
    let words_wide = (right - x).max(laying.px(48));

    // The words.
    let (first_drawn, first_text) = (out.drawn.len(), out.texts.len());
    let mut y = top + above;
    let title = laying.words(
        &row.title,
        TheType::Name,
        words_wide,
        (palette.surface, palette.ink),
    );
    let (title_tall, title_ink) = (title.height, title.width);
    y = text(out, x, y, title, &row.title);
    if let Some(tag) = &row.tag {
        a_tag(
            out,
            laying,
            (x + title_ink + laying.px(8), top + above + title_tall / 2),
            tag,
        );
    }
    for line_words in &row.lines {
        let shaped = laying.words(
            line_words,
            TheType::Body,
            words_wide,
            (palette.surface, palette.muted),
        );
        y = text(
            out,
            x,
            y + laying.px(m::UNDER_A_ROWS_TITLE),
            shaped,
            line_words,
        );
    }
    if let Some(note) = &row.note {
        let icon = laying.px(m::A_NOTES_ICON) + laying.px(m::AFTER_A_NOTES_ICON);
        let ink = if note.warns {
            palette.warning
        } else {
            palette.muted
        };
        let style = if note.in_mono {
            TheType::Mono
        } else {
            TheType::SmallLabel
        };
        let shaped = laying.words(
            &note.words,
            style,
            words_wide - icon,
            (palette.surface, ink),
        );
        let note_at = y + laying.px(m::UNDER_A_ROWS_TITLE);
        let side = laying.px(m::A_NOTES_ICON);
        out.drawn.push(Drawn::Icon(
            note.icon,
            Rectangle::new(
                (x, note_at + (shaped.height - side) / 2).into(),
                (side, side).into(),
            ),
            ink,
        ));
        y = text(out, x + icon, note_at, shaped, &note.words);
    }
    if let Some(risk) = &row.risk {
        y = a_risk(
            out,
            laying,
            (x, y + laying.px(m::UNDER_A_ROWS_TITLE), words_wide),
            risk,
        );
    }
    let mut part_areas = Vec::new();
    if row.kind == TheKind::ASwatches {
        let (areas, bottom) = the_swatches(
            out,
            laying,
            (
                x,
                y + laying.px(m::UNDER_A_ROWS_TITLE) + laying.px(m::ABOVE_THE_SWATCHES),
            ),
            &row.parts,
        );
        part_areas = areas;
        y = bottom;
    }
    let least = laying.px(if matches!(row.kind, TheKind::AnAddingRow) {
        m::AN_OPENING_ROW_AT_LEAST
    } else {
        m::A_ROW_AT_LEAST
    });
    let segmented_tall = segmented.as_ref().map_or(0, |_| {
        laying.px(m::A_TARGET) + 2 * laying.px(m::INSIDE_A_SEGMENTED)
    });
    let button_tall = button.as_ref().map_or(0, |_| laying.px(m::A_TARGET));
    let bottom = (y + above)
        .max(top + least)
        .max(top + 2 * above + segmented_tall.max(button_tall));
    // **A row that is not a choice is centred in its height**, which the
    // frames draw `items-center`; a choice keeps its words at the top.
    let centred = !matches!(
        row.kind,
        TheKind::AChoice { .. }
            | TheKind::ANestedChoice { .. }
            | TheKind::ASwitch(_)
            | TheKind::ASwatches
    );
    let lower = if centred {
        ((bottom - top) - (y - top) - above) / 2
    } else {
        0
    };
    if lower > 0 {
        moved_down(out, (first_drawn, first_text), lower);
        for area in &mut part_areas {
            area.loc.y += lower;
        }
    }
    let centre_of_title = top + above + lower.max(0) + title_tall / 2;
    let centre_of_row = top + (bottom - top) / 2;
    let ends_at = if centred {
        centre_of_row
    } else {
        centre_of_title
    };

    // The start.
    match (row.kind, row.lead) {
        (_, Some(icon)) => a_lead(out, laying, (mark_at, centre_of_row), icon),
        (TheKind::AChoice { chosen } | TheKind::ANestedChoice { chosen }, None) => {
            a_choice(out, laying, (mark_at, centre_of_title, mark), chosen);
        }
        (TheKind::AnAddingRow, None) => {
            // **The plus, 16 in the 18 a ring would take** (`550:220`).
            let side = laying.px(16);
            out.drawn.push(Drawn::Icon(
                ASettingsIcon::Plus,
                Rectangle::new(
                    (mark_at + (mark - side) / 2, centre_of_title - side / 2).into(),
                    (side, side).into(),
                ),
                palette.secondary,
            ));
        }
        _ => {}
    }

    // The end, right to left.
    let mut at = inside_left + inside_wide - beside;
    if opens {
        at -= chevron;
        out.drawn.push(Drawn::Icon(
            ASettingsIcon::ChevronRight,
            Rectangle::new(
                (at, centre_of_row - chevron / 2).into(),
                (chevron, chevron).into(),
            ),
            palette.muted,
        ));
        at -= gap;
    }
    if let TheKind::ASwitch(is) = row.kind {
        at -= switch;
        a_switch(out, laying, at, ends_at, is);
        at -= laying.px(m::BEFORE_A_SWITCH);
    }
    if let Some(end) = end {
        let wide = end.wide(laying);
        end.placed(out, laying, at, ends_at);
        at -= wide + gap;
    }
    if let Some(segmented) = segmented {
        let wide = segmented.wide(laying);
        part_areas = segmented.placed(out, laying, (at, centre_of_row), &row.parts);
        at -= wide + gap;
    }
    let mut button_area = None;
    if let (Some((shaped, wide)), Some((label, target))) = (button, &row.button) {
        at -= wide;
        let tall = laying.px(m::A_TARGET);
        let button_top = if centred {
            centre_of_row - tall / 2
        } else {
            top + above
        };
        let area = Rectangle::new((at, button_top).into(), (wide, tall).into());
        out.drawn
            .push(Drawn::Fill(area, m::A_CARDS_CORNER, palette.surface));
        out.drawn
            .push(Drawn::Border(area, m::A_CARDS_CORNER, palette.edge));
        let (label_wide, label_tall) = (shaped.width_of_box, shaped.height);
        text(
            out,
            at + (wide - label_wide) / 2,
            button_top + (tall - label_tall) / 2,
            shaped,
            label,
        );
        button_area = Some((area, target.clone()));
    }

    // Its targets, in the keyboard's order: the row, its parts, its button.
    if let Some(target) = &row.row {
        out.targets.push(ATargetDrawn {
            row: target.clone(),
            area: Rectangle::new(
                (inside_left, top).into(),
                (inside_wide, bottom - top).into(),
            ),
            section_starts,
        });
    }
    for (part, area) in row.parts.iter().zip(part_areas) {
        out.targets.push(ATargetDrawn {
            row: part.row.clone(),
            area,
            section_starts,
        });
    }
    if let Some((area, target)) = button_area {
        out.targets.push(ATargetDrawn {
            row: target,
            area,
            section_starts,
        });
    }
    bottom
}
