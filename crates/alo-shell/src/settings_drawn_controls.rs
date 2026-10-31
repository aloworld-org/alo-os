//! The controls a Settings row is drawn with, each to its node in the frames:
//! a choice's ring, a switch, what a choice risks, a lead icon in its well, a
//! tag, a shortcut's keys, *Always on*, a segmented choice and the accent
//! colours.
//!
//! Each places itself and says where its targets are; `crate::settings_drawn_row`
//! decides where on the row it goes. The figures are `crate::settings_measures`
//! and the frames' own, named where they are used.

use smithay::utils::Rectangle;

use crate::painted_text::Shaped;
use crate::settings_canvas::Area;
use crate::settings_drawn_page::{Drawn, Laying, ThePageDrawn, shrunk, text};
use crate::settings_measures as m;
use crate::settings_page::{APart, ARisk, SwitchIs};
use crate::settings_type::TheType;
use crate::the_settings_icons::ASettingsIcon;

/// A choice's ring, and its dot when it is chosen: `550:170` and `550:184`.
///
/// Not chosen: a ring 1.5 thick at radius 9.25 of 22, in `text/muted`, on
/// `bg/surface`. Chosen: a ring 2 thick at radius 9 in the accent, and a dot
/// of radius 5 in the accent. The 18 under another is the same at 7.25, 7
/// and 4.
pub(crate) fn a_choice(
    out: &mut ThePageDrawn,
    laying: &Laying<'_>,
    (x, centre, side): (i32, i32, i32),
    chosen: bool,
) {
    let palette = laying.palette;
    let area = Rectangle::new((x, centre - side / 2).into(), (side, side).into());
    out.drawn.push(Drawn::Disc(area, palette.surface));
    if chosen {
        out.drawn
            .push(Drawn::Ring(shrunk(area, laying.px(1)), 20, palette.accent));
        let dot = if side > laying.px(m::A_NESTED_CHOICE) {
            6
        } else {
            5
        };
        out.drawn
            .push(Drawn::Disc(shrunk(area, laying.px(dot)), palette.accent));
    } else {
        out.drawn
            .push(Drawn::Ring(shrunk(area, laying.px(1)), 15, palette.muted));
    }
}

/// A switch: `550:256` off and `551:196` on.
///
/// Off: a pill 44 × 26 in `bg/cool` with a 1.5 edge in `text/muted` and a knob
/// of radius 10 at the start, in `text/muted`. On: the pill in the accent and
/// the knob in `bg/surface` at the end. *Not set* is drawn as off is, and its
/// word says which it is.
pub(crate) fn a_switch(
    out: &mut ThePageDrawn,
    laying: &Laying<'_>,
    x: i32,
    centre: i32,
    is: SwitchIs,
) {
    let palette = laying.palette;
    let (wide, tall) = (
        laying.px(m::A_SWITCH_IS_WIDE),
        laying.px(m::A_SWITCH_IS_TALL),
    );
    let track = Rectangle::new((x, centre - tall / 2).into(), (wide, tall).into());
    let knob = laying.px(2 * m::A_KNOB);
    let inset = (tall - knob) / 2;
    match is {
        SwitchIs::On => {
            out.drawn
                .push(Drawn::Fill(track, m::A_SWITCH_IS_TALL / 2, palette.accent));
            let at = Rectangle::new(
                (track.loc.x + wide - inset - knob, track.loc.y + inset).into(),
                (knob, knob).into(),
            );
            out.drawn.push(Drawn::Disc(at, palette.surface));
        }
        SwitchIs::Off | SwitchIs::NotSet => {
            out.drawn
                .push(Drawn::Fill(track, m::A_SWITCH_IS_TALL / 2, palette.cool));
            out.drawn.push(Drawn::Ring(track, 15, palette.muted));
            let at = Rectangle::new(
                (track.loc.x + inset, track.loc.y + inset).into(),
                (knob, knob).into(),
            );
            out.drawn.push(Drawn::Disc(at, palette.muted));
        }
    }
}

/// What a choice risks: `550:246`, `bg/canvas`, 12 across and 10 down inside,
/// at radius 10, the warning sign 18, the heading in `status/warning` and the
/// sentence in `text/primary`.
pub(crate) fn a_risk(
    out: &mut ThePageDrawn,
    laying: &mut Laying<'_>,
    (x, y, wide): (i32, i32, i32),
    risk: &ARisk,
) -> i32 {
    let palette = laying.palette;
    let (across, down) = (laying.px(m::BESIDE_A_RISK), laying.px(m::ABOVE_A_RISK));
    let icon = laying.px(m::A_RISKS_ICON) + laying.px(m::AFTER_A_RISKS_ICON);
    let inside = wide - 2 * across - icon;
    let heading = laying.words(
        &risk.heading,
        TheType::Label,
        inside,
        (palette.canvas, palette.warning),
    );
    let sentence = laying.words(
        &risk.words,
        TheType::Body,
        inside,
        (palette.canvas, palette.ink),
    );
    let between = laying.px(m::UNDER_A_RISKS_HEADING);
    let tall = heading.height + between + sentence.height + 2 * down;
    let area = Rectangle::new((x, y).into(), (wide, tall).into());
    out.drawn
        .push(Drawn::Fill(area, m::A_SMALL_CORNER, palette.canvas));
    let sign = laying.px(m::A_RISKS_ICON);
    out.drawn.push(Drawn::Icon(
        ASettingsIcon::Warn,
        Rectangle::new((x + across, y + down).into(), (sign, sign).into()),
        palette.warning,
    ));
    let below = text(out, x + across + icon, y + down, heading, &risk.heading);
    text(
        out,
        x + across + icon,
        below + between,
        sentence,
        &risk.words,
    );
    y + tall
}

/// A lead icon in its well: `550:820`, 36 square in `bg/cool` at radius 10,
/// the icon 20 in its middle in `text/secondary`.
pub(crate) fn a_lead(
    out: &mut ThePageDrawn,
    laying: &Laying<'_>,
    (x, centre): (i32, i32),
    icon: ASettingsIcon,
) {
    let palette = laying.palette;
    let side = laying.px(m::A_LEADS_WELL);
    let well = Rectangle::new((x, centre - side / 2).into(), (side, side).into());
    out.drawn
        .push(Drawn::Fill(well, m::A_SMALL_CORNER, palette.cool));
    let inner = laying.px(m::A_CATEGORYS_ICON);
    out.drawn.push(Drawn::Icon(
        icon,
        Rectangle::new(
            (x + (side - inner) / 2, centre - inner / 2).into(),
            (inner, inner).into(),
        ),
        palette.secondary,
    ));
}

/// A tag beside a name: `550:951`, `bg/cool` at radius 6, 8 across and 2
/// down inside, Label/Small in `text/secondary`.
pub(crate) fn a_tag(
    out: &mut ThePageDrawn,
    laying: &mut Laying<'_>,
    (x, centre): (i32, i32),
    words: &str,
) {
    let palette = laying.palette;
    let shaped = laying
        .words(
            words,
            TheType::SmallLabel,
            laying.wide,
            (palette.cool, palette.secondary),
        )
        .cropped_to_its_ink();
    let (across, down) = (laying.px(8), laying.px(2));
    let area = Rectangle::new(
        (x, centre - shaped.height / 2 - down).into(),
        (shaped.width_of_box + 2 * across, shaped.height + 2 * down).into(),
    );
    out.drawn.push(Drawn::Fill(area, 6, palette.cool));
    text(out, x + across, area.loc.y + down, shaped, words);
}

/// What is at a row's end, measured before it is placed.
pub(crate) enum AnEndShaped {
    /// Words, ready to place.
    Words(Shaped, String),
    /// Keys, each shaped on its cap.
    Keys(Vec<(Shaped, String)>),
    /// *Always on*, shaped.
    AlwaysOn(Shaped, String),
    /// A picker's button, its label shaped.
    Picker(Shaped, String),
}

impl AnEndShaped {
    /// How wide it is.
    pub(crate) fn wide(&self, laying: &Laying<'_>) -> i32 {
        match self {
            Self::Words(shaped, _) => shaped.width_of_box,
            Self::Keys(keys) => {
                let caps: i32 = keys
                    .iter()
                    .map(|(shaped, _)| shaped.width_of_box + 2 * laying.px(m::INSIDE_A_KEY))
                    .sum();
                caps + laying.px(m::BETWEEN_KEYS) * (keys.len().saturating_sub(1) as i32)
            }
            Self::AlwaysOn(shaped, _) => {
                laying.px(m::A_LOCK) + laying.px(m::AFTER_A_NOTES_ICON) + shaped.width_of_box
            }
            Self::Picker(shaped, _) => {
                2 * laying.px(m::INSIDE_A_PICKER)
                    + shaped.width_of_box
                    + laying.px(m::BEFORE_A_PICKERS_ARROWS)
                    + laying.px(m::A_PICKERS_ARROWS)
            }
        }
    }

    /// Place it with its right edge at `right`, centred on `centre`.
    pub(crate) fn placed(
        self,
        out: &mut ThePageDrawn,
        laying: &Laying<'_>,
        right: i32,
        centre: i32,
    ) {
        let palette = laying.palette;
        let at = right - self.wide(laying);
        match self {
            Self::Words(shaped, words) => {
                let tall = shaped.height;
                text(out, at, centre - tall / 2, shaped, &words);
            }
            Self::Keys(keys) => {
                let mut x = at;
                let tall = laying.px(m::A_KEY_IS_TALL);
                for (shaped, words) in keys {
                    let wide = shaped.width_of_box + 2 * laying.px(m::INSIDE_A_KEY);
                    let cap = Rectangle::new((x, centre - tall / 2).into(), (wide, tall).into());
                    out.drawn
                        .push(Drawn::Fill(cap, m::A_KEYS_CORNER, palette.cool));
                    out.drawn
                        .push(Drawn::Border(cap, m::A_KEYS_CORNER, palette.edge));
                    let label = shaped.height;
                    text(
                        out,
                        x + laying.px(m::INSIDE_A_KEY),
                        centre - label / 2,
                        shaped,
                        &words,
                    );
                    x += wide + laying.px(m::BETWEEN_KEYS);
                }
            }
            Self::Picker(shaped, words) => {
                let wide = self_wide_of_a_picker(laying, &shaped);
                let tall = laying.px(m::A_TARGET);
                let button = Rectangle::new((at, centre - tall / 2).into(), (wide, tall).into());
                out.drawn
                    .push(Drawn::Fill(button, m::A_CARDS_CORNER, palette.surface));
                out.drawn
                    .push(Drawn::Border(button, m::A_CARDS_CORNER, palette.edge));
                let label = shaped.height;
                let label_wide = shaped.width_of_box;
                text(
                    out,
                    at + laying.px(m::INSIDE_A_PICKER),
                    centre - label / 2,
                    shaped,
                    &words,
                );
                let arrows = laying.px(m::A_PICKERS_ARROWS);
                out.drawn.push(Drawn::Icon(
                    ASettingsIcon::ChevronUpDown,
                    Rectangle::new(
                        (
                            at + laying.px(m::INSIDE_A_PICKER)
                                + label_wide
                                + laying.px(m::BEFORE_A_PICKERS_ARROWS),
                            centre - arrows / 2,
                        )
                            .into(),
                        (arrows, arrows).into(),
                    ),
                    palette.muted,
                ));
            }
            Self::AlwaysOn(shaped, words) => {
                let lock = laying.px(m::A_LOCK);
                out.drawn.push(Drawn::Icon(
                    ASettingsIcon::Lock,
                    Rectangle::new((at, centre - lock / 2).into(), (lock, lock).into()),
                    palette.positive,
                ));
                let tall = shaped.height;
                text(
                    out,
                    at + lock + laying.px(m::AFTER_A_NOTES_ICON),
                    centre - tall / 2,
                    shaped,
                    &words,
                );
            }
        }
    }
}

/// How wide a picker's button is around its shaped label.
fn self_wide_of_a_picker(laying: &Laying<'_>, shaped: &Shaped) -> i32 {
    2 * laying.px(m::INSIDE_A_PICKER)
        + shaped.width_of_box
        + laying.px(m::BEFORE_A_PICKERS_ARROWS)
        + laying.px(m::A_PICKERS_ARROWS)
}

/// A segmented choice's parts, shaped: each label, and whether it is chosen.
pub(crate) struct ASegmented {
    /// Each part's label, shaped on its ground, and its width.
    parts: Vec<(Shaped, i32)>,
}

impl ASegmented {
    /// The parts of `551:336`: each 48 tall, 16 across inside — and the chosen
    /// one 14 across with a check of 16 and 6 before its label — in a `bg/cool`
    /// box 4 inside at radius 12, 4 between them.
    pub(crate) fn of(laying: &mut Laying<'_>, parts: &[APart]) -> Self {
        let palette = laying.palette;
        let shaped = parts
            .iter()
            .map(|part| {
                let (ground, ink) = if part.chosen {
                    (palette.surface, palette.ink)
                } else {
                    (palette.cool, palette.secondary)
                };
                let shaped = laying
                    .words(&part.label, TheType::Label, laying.wide, (ground, ink))
                    .cropped_to_its_ink();
                let wide = shaped.width_of_box
                    + if part.chosen {
                        2 * laying.px(m::INSIDE_A_CHOSEN_SEGMENT)
                            + laying.px(m::A_SEGMENTS_CHECK)
                            + laying.px(m::AFTER_A_SEGMENTS_CHECK)
                    } else {
                        2 * laying.px(m::INSIDE_A_SEGMENT)
                    };
                (shaped, wide)
            })
            .collect();
        Self { parts: shaped }
    }

    /// How wide it is, its box included.
    pub(crate) fn wide(&self, laying: &Laying<'_>) -> i32 {
        let inside: i32 = self.parts.iter().map(|(_, wide)| *wide).sum();
        inside
            + 2 * laying.px(m::INSIDE_A_SEGMENTED)
            + laying.px(m::INSIDE_A_SEGMENTED) * (self.parts.len().saturating_sub(1) as i32)
    }

    /// Place it with its right edge at `right`, centred on `centre`; answer
    /// each part's area, in order.
    pub(crate) fn placed(
        self,
        out: &mut ThePageDrawn,
        laying: &Laying<'_>,
        (right, centre): (i32, i32),
        parts: &[APart],
    ) -> Vec<Area> {
        let palette = laying.palette;
        let pad = laying.px(m::INSIDE_A_SEGMENTED);
        let tall = laying.px(m::A_TARGET) + 2 * pad;
        let wide = self.wide(laying);
        let frame = Rectangle::new(
            (right - wide, centre - tall / 2).into(),
            (wide, tall).into(),
        );
        out.drawn
            .push(Drawn::Fill(frame, m::A_CARDS_CORNER, palette.cool));
        let mut x = frame.loc.x + pad;
        let mut areas = Vec::new();
        for ((shaped, part_wide), part) in self.parts.into_iter().zip(parts) {
            let area = Rectangle::new(
                (x, frame.loc.y + pad).into(),
                (part_wide, laying.px(m::A_TARGET)).into(),
            );
            let mut label_x = x + laying.px(m::INSIDE_A_SEGMENT);
            if part.chosen {
                out.drawn
                    .push(Drawn::Fill(area, m::A_SEGMENTS_CORNER, palette.surface));
                let check = laying.px(m::A_SEGMENTS_CHECK);
                label_x = x + laying.px(m::INSIDE_A_CHOSEN_SEGMENT);
                out.drawn.push(Drawn::Icon(
                    ASettingsIcon::Check,
                    Rectangle::new((label_x, centre - check / 2).into(), (check, check).into()),
                    palette.accent,
                ));
                label_x += check + laying.px(m::AFTER_A_SEGMENTS_CHECK);
            }
            let label = shaped.height;
            text(out, label_x, centre - label / 2, shaped, &part.label);
            areas.push(area);
            x += part_wide + pad;
        }
        areas
    }
}

/// The text size slider: `551:1116`, at least 48 tall — a small *A* in
/// Body/Medium, a track 4 tall in `bg/cool` filled in the accent up to a knob of
/// radius 11 in `bg/surface` with a 2 accent edge, a large *A* in
/// Heading/Large, and the percentage — 16 between each. The track is 360
/// across at most. Answer the slider's bottom.
pub(crate) fn a_slider(
    out: &mut ThePageDrawn,
    laying: &mut Laying<'_>,
    (x, y, wide): (i32, i32, i32),
    (percent, (least, most)): (u16, (u16, u16)),
    the_percentage: Option<AnEndShaped>,
) -> i32 {
    let palette = laying.palette;
    let gap = laying.px(m::INSIDE_A_ROW);
    let tall = laying.px(m::A_TARGET);
    let centre = y + tall / 2;
    let letter = crate::the_settings_words::THE_LETTER;
    let small = laying
        .words(
            letter,
            TheType::Body,
            wide,
            (palette.surface, palette.secondary),
        )
        .cropped_to_its_ink();
    let large = laying
        .words(
            letter,
            TheType::PageHeading,
            wide,
            (palette.surface, palette.secondary),
        )
        .cropped_to_its_ink();
    let end = the_percentage
        .as_ref()
        .map_or(0, |it| it.wide(laying) + gap);
    let track_wide = (wide - small.width_of_box - large.width_of_box - 2 * gap - end - gap)
        .min(laying.px(m::A_SLIDER_AT_MOST))
        .max(laying.px(m::A_TARGET));
    let (small_tall, large_tall) = (small.height, large.height);
    let mut at = x;
    let small_wide = small.width_of_box;
    text(out, at, centre - small_tall / 2, small, letter);
    at += small_wide + gap;
    let track = Rectangle::new(
        (at, centre - laying.px(2)).into(),
        (track_wide, laying.px(4)).into(),
    );
    out.drawn.push(Drawn::Fill(track, 2, palette.cool));
    let share =
        f32::from(percent.saturating_sub(least)) / f32::from(most.saturating_sub(least).max(1));
    let knob_x = at + (share * track_wide as f32) as i32;
    out.drawn.push(Drawn::Fill(
        Rectangle::new(track.loc, ((knob_x - at).max(0), track.size.h).into()),
        2,
        palette.accent,
    ));
    let knob = laying.px(2 * m::A_SLIDERS_KNOB);
    let knob_area = Rectangle::new(
        (knob_x - knob / 2, centre - knob / 2).into(),
        (knob, knob).into(),
    );
    out.drawn.push(Drawn::Disc(knob_area, palette.surface));
    out.drawn.push(Drawn::Ring(knob_area, 20, palette.accent));
    at += track_wide + gap;
    let large_wide = large.width_of_box;
    text(out, at, centre - large_tall / 2, large, letter);
    at += large_wide + gap;
    if let Some(percentage) = the_percentage {
        let wide = percentage.wide(laying);
        percentage.placed(out, laying, at + wide, centre);
    }
    y + tall
}

/// The accent colours: `551:307`, each 72 across with 16 between, a target 48
/// square holding a disc of radius 17 in the colour — and round the chosen one
/// a ring of radius 21, 2 thick, in the same colour, with a white check of 18
/// on the disc — and its name 6 under it. Answer each target's area and the
/// bottom.
pub(crate) fn the_swatches(
    out: &mut ThePageDrawn,
    laying: &mut Laying<'_>,
    (x, y): (i32, i32),
    parts: &[APart],
) -> (Vec<Area>, i32) {
    let palette = laying.palette;
    let (item, between, target) = (
        laying.px(m::A_SWATCH_IS_WIDE),
        laying.px(m::BETWEEN_SWATCHES),
        laying.px(m::A_TARGET),
    );
    let mut areas = Vec::new();
    let mut bottom = y;
    for (which, part) in parts.iter().enumerate() {
        let left = x + (item + between) * which as i32;
        let area = Rectangle::new(
            (left + (item - target) / 2, y).into(),
            (target, target).into(),
        );
        let colour = part.colour.map_or(
            palette.accent,
            |(light, dark)| {
                if palette.dark { dark } else { light }
            },
        );
        let centre = (area.loc.x + target / 2, area.loc.y + target / 2);
        let disc = laying.px(2 * m::A_SWATCH);
        out.drawn.push(Drawn::Disc(
            Rectangle::new(
                (centre.0 - disc / 2, centre.1 - disc / 2).into(),
                (disc, disc).into(),
            ),
            colour,
        ));
        if part.chosen {
            let ring = laying.px(2 * m::A_CHOSEN_SWATCHS_RING + 2);
            out.drawn.push(Drawn::Ring(
                Rectangle::new(
                    (centre.0 - ring / 2, centre.1 - ring / 2).into(),
                    (ring, ring).into(),
                ),
                20,
                colour,
            ));
            let check = laying.px(m::A_SWATCHS_CHECK);
            out.drawn.push(Drawn::Icon(
                ASettingsIcon::Check,
                Rectangle::new(
                    (centre.0 - check / 2, centre.1 - check / 2).into(),
                    (check, check).into(),
                ),
                palette.surface,
            ));
        }
        let ink = if part.chosen {
            palette.ink
        } else {
            palette.secondary
        };
        // **SemiBold where the frame draws the chosen name Bold**: this
        // machine bundles Manrope at Regular and SemiBold, and a Bold asked
        // for is drawn in whatever face answers instead.
        let style = TheType::Label;
        let shaped = laying
            .words(&part.label, style, item, (palette.surface, ink))
            .cropped_to_its_ink();
        let label_at = (
            left + (item - shaped.width_of_box) / 2,
            y + target + laying.px(m::UNDER_A_SWATCH),
        );
        bottom = bottom.max(text(out, label_at.0, label_at.1, shaped, &part.label));
        areas.push(area);
    }
    (areas, bottom)
}
