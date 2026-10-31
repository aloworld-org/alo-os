//! The list a picker opens, drawn over the page — frame `551:841`: 360 across
//! and at most 480 down, `bg/surface` at radius 18 with the floating shadow,
//! 8 inside; the search field at its top with the keyboard's outline round it;
//! what matches under a heading counting them; and the whole list under its own.
//!
//! **It hangs from its button**: its right edge on the button's, 8 under it,
//! and never past the window's foot — it is shorter there, and its entries
//! scroll inside it to keep the keyboard's in view.

use cosmic_text::FontSystem;
use smithay::utils::Rectangle;

use crate::desktop_look::Measure;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_drawn_page::SettingsText;
use crate::settings_measures as m;
use crate::settings_palette::SettingsPalette;
use crate::settings_picker::{APickerOpen, AnEntry, SettingsPicker};
use crate::settings_type::{TheType, written};
use crate::settings_window::Open;
use crate::the_settings_icons::ASettingsIcon;

/// What drawing the list needs at hand.
pub(crate) struct TheListsRoom<'a> {
    /// The fonts.
    pub(crate) fonts: &'a mut FontSystem,
    /// The person's text size on this display.
    pub(crate) measure: Measure,
    /// The colours.
    pub(crate) palette: SettingsPalette,
    /// The words.
    pub(crate) strings: &'a alo_strings::Strings,
    /// The button it hangs from.
    pub(crate) button: Area,
    /// The window, which it stays inside.
    pub(crate) window: Area,
    /// The floating shadow, at this display's scale.
    pub(crate) shadow: AShadow,
}

/// The list as drawn: where it is, every entry's place and what it says.
pub(crate) struct TheListDrawn {
    /// The list itself.
    pub(crate) area: Area,
    /// Every entry in view, where it is drawn.
    pub(crate) entries: Vec<(AnEntry, Area)>,
    /// Every piece of text drawn.
    pub(crate) texts: Vec<SettingsText>,
}

/// Draw the open list onto `canvas`.
pub(crate) fn the_list(
    canvas: &mut Canvas,
    open: &Open,
    picker: &APickerOpen,
    room: TheListsRoom<'_>,
) -> TheListDrawn {
    let TheListsRoom {
        fonts,
        measure,
        palette,
        strings,
        button,
        window,
        shadow,
    } = room;
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let wide = px(m::A_LIST_IS_WIDE).min(window.size.w);
    let top = button.loc.y + button.size.h + px(m::UNDER_A_PICKER);
    let foot = window.loc.y + window.size.h - px(m::INSIDE_A_ROW);
    let tall = px(m::A_LIST_AT_MOST_TALL).min((foot - top).max(px(m::A_TARGET) * 3));
    let left = (button.loc.x + button.size.w - wide).max(window.loc.x);
    let area = Rectangle::new((left, top).into(), (wide, tall).into());
    let corner = px(m::THE_WINDOWS_CORNER) as f32;
    canvas.shadow(area, corner, shadow);
    canvas.fill(area, corner, palette.surface, 1.0);
    canvas.border(area, corner, px(m::A_HAIRLINE) as f32, palette.edge);

    let mut drawn = TheListDrawn {
        area,
        entries: Vec::new(),
        texts: Vec::new(),
    };
    let inside = px(m::INSIDE_A_LIST);
    let (x, inner) = (left + inside, wide - 2 * inside);

    // **The search**, with the keyboard's outline: it has the keyboard for as
    // long as the list is open.
    let search = Rectangle::new((x, top + inside).into(), (inner, px(m::A_TARGET)).into());
    canvas.fill(search, px(m::A_CARDS_CORNER) as f32, palette.cool, 1.0);
    canvas.border(
        search,
        px(m::A_CARDS_CORNER) as f32,
        px(m::A_SEARCHS_OUTLINE) as f32,
        palette.ink,
    );
    let sign = px(m::A_SEARCHS_SIGN);
    let centre = search.loc.y + search.size.h / 2;
    ASettingsIcon::Search.drawn(
        canvas,
        Rectangle::new(
            (search.loc.x + px(m::INSIDE_A_SEARCH), centre - sign / 2).into(),
            (sign, sign).into(),
        ),
        palette.muted,
    );
    let typed_at = search.loc.x + px(m::INSIDE_A_SEARCH) + sign + px(m::AFTER_A_SEARCHS_SIGN);
    let typed_room = search.loc.x + search.size.w - typed_at - px(m::INSIDE_A_SEARCH);
    let mut caret_at = typed_at;
    if !picker.typed.is_empty() {
        let shaped = written(
            fonts,
            &picker.typed,
            TheType::Body,
            typed_room,
            measure,
            (palette.cool, palette.ink),
        )
        .cropped_to_its_ink();
        caret_at += shaped.width_of_box + px(m::AFTER_A_SEARCHS_SIGN);
        canvas.words(typed_at, centre - shaped.height / 2, &shaped);
    }
    let caret = px(TheType::SmallLabel.line()) + px(2);
    canvas.fill(
        Rectangle::new(
            (caret_at, centre - caret / 2).into(),
            (px(2).max(1), caret).into(),
        ),
        0.0,
        palette.ink,
        1.0,
    );

    // **The entries, in their own view under the search**, scrolled to keep
    // the keyboard's in it.
    let view = Rectangle::new(
        (x, search.loc.y + search.size.h + px(m::BETWEEN_LIST_ITEMS)).into(),
        (
            inner,
            top + tall - inside - (search.loc.y + search.size.h + px(m::BETWEEN_LIST_ITEMS)),
        )
            .into(),
    );
    let every = open.every_entry(picker.which, strings);
    let matches = open.the_matches(picker.which, &picker.typed, strings);
    let entry_tall = px(m::A_TARGET);
    let heading_tall =
        px(m::ABOVE_A_LISTS_HEADING) + px(TheType::Label.line()) + px(m::BELOW_A_LISTS_HEADING);
    let step = entry_tall + px(m::BETWEEN_LIST_ITEMS);
    // Where each entry, in the keyboard's order, and each heading would be.
    let mut placed: Vec<(Option<AnEntry>, Option<String>, i32)> = Vec::new();
    let mut y = 0;
    if !matches.is_empty() {
        let heading = crate::the_settings_words::how_many_match(strings, matches.len());
        placed.push((None, Some(heading), y));
        y += heading_tall;
        for entry in matches {
            placed.push((Some(entry), None, y));
            y += step;
        }
    }
    let all = crate::settings_page::said(
        strings,
        match picker.which {
            SettingsPicker::Region => crate::the_settings_words::ALL_COUNTRIES,
            SettingsPicker::TimeZone => crate::the_settings_words::ALL_TIME_ZONES,
        },
    );
    placed.push((None, Some(all), y));
    y += heading_tall;
    for entry in every {
        placed.push((Some(entry), None, y));
        y += step;
    }
    let focused_at = placed
        .iter()
        .filter(|(entry, _, _)| entry.is_some())
        .nth(picker.focus)
        .map(|(_, _, at)| *at);
    let scrolled = match focused_at {
        Some(at) if at + entry_tall > view.size.h => at + entry_tall - view.size.h,
        _ => 0,
    }
    .min((y - view.size.h).max(0));

    canvas.clipped_to(view);
    let mut entry_number = 0;
    for (entry, heading, at) in placed {
        let row_top = view.loc.y + at - scrolled;
        if row_top > view.loc.y + view.size.h || row_top + step < view.loc.y {
            if entry.is_some() {
                entry_number += 1;
            }
            continue;
        }
        if let Some(heading) = heading {
            let shaped = written(
                fonts,
                &heading,
                TheType::Label,
                inner,
                measure,
                (palette.surface, palette.secondary),
            )
            .cropped_to_its_ink();
            let words_at = (
                x + px(m::INSIDE_AN_ENTRY),
                row_top + px(m::ABOVE_A_LISTS_HEADING),
            );
            drawn.texts.push(SettingsText {
                text: heading.clone(),
                area: Rectangle::new(words_at.into(), (shaped.width_of_box, shaped.height).into()),
            });
            canvas.words(words_at.0, words_at.1, &shaped);
            continue;
        }
        let Some(entry) = entry else { continue };
        let row = Rectangle::new((x, row_top).into(), (inner, entry_tall).into());
        let ground = if entry.chosen {
            palette.chosen_on_surface
        } else {
            palette.surface
        };
        if entry.chosen {
            canvas.fill(row, px(m::A_SMALL_CORNER) as f32, ground, 1.0);
            let check = px(m::AN_ENTRYS_CHECK);
            ASettingsIcon::Check.drawn(
                canvas,
                Rectangle::new(
                    (
                        row.loc.x + row.size.w - px(m::INSIDE_AN_ENTRY) - check,
                        row_top + (entry_tall - check) / 2,
                    )
                        .into(),
                    (check, check).into(),
                ),
                palette.accent,
            );
        }
        let style = if entry.chosen {
            TheType::Name
        } else {
            TheType::Body
        };
        let shaped = written(
            fonts,
            &entry.label,
            style,
            inner - 2 * px(m::INSIDE_AN_ENTRY) - px(m::AN_ENTRYS_CHECK),
            measure,
            (ground, palette.ink),
        )
        .cropped_to_its_ink();
        let words_at = (
            x + px(m::INSIDE_AN_ENTRY),
            row_top + (entry_tall - shaped.height) / 2,
        );
        drawn.texts.push(SettingsText {
            text: entry.label.clone(),
            area: Rectangle::new(words_at.into(), (shaped.width_of_box, shaped.height).into()),
        });
        canvas.words(words_at.0, words_at.1, &shaped);
        // **The keyboard's outline**, inside the entry so the list's clip
        // never cuts it.
        if entry_number == picker.focus {
            let thick = px(m::THE_FOCUS_RING);
            canvas.border(row, px(m::A_SMALL_CORNER) as f32, thick as f32, palette.ink);
        }
        drawn.entries.push((entry, row));
        entry_number += 1;
    }
    canvas.unclipped();
    drawn
}
