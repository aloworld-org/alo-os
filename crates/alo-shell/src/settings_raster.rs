//! Settings laid out and rasterised for one output: the window the owner's
//! redesign of 2026-10-31 draws, with a side column of categories and the page
//! the person chose beside it.
//!
//! # What is drawn
//!
//! While Settings is closed, **nothing**. While it is open, one window — a
//! rounded panel with the floating shadow, frame `550:98` — holding:
//!
//! - **the side column** on the canvas colour: *Settings*, the line under it,
//!   the two groups of categories, and the footer at its foot. The page being
//!   shown is the category on the accent's ground (`crate::settings_palette`);
//! - **the page** beside it: its heading, its introduction, and its sections as
//!   `crate::settings_drawn_page` lays them out — and it **scrolls**, so the
//!   row the keyboard is on is always in view, with the start of its section
//!   above it where both fit.
//!
//! Where it goes and how large is a rule of the display (`crate::settings_frame`),
//! and every figure goes through the person's text size and the display's
//! scale together, so a 200 per cent screen draws the same window sharper.
//!
//! # Where the keyboard is, is drawn
//!
//! A navy outline (`text/primary`, as the task window's is) round the category
//! while the keyboard is in the side column, and round the row or button it is
//! on in the page — doubled while Settings waits for a chord. Nothing is
//! dimmed: a row is drawn only for something a person can do, or says in words
//! that it is not on this computer yet.

use std::time::SystemTime;

use alo_appearance::{Colour, Scheme, TextScale};
use alo_strings::{Direction, Strings};
use smithay::utils::Rectangle;

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_categories::SettingsCategory;
use crate::settings_drawn_page::{ATargetDrawn, Laying, SettingsText, laid_out, painted};
use crate::settings_frame::TheFrame;
use crate::settings_measures as m;
use crate::settings_page::said;
use crate::settings_palette::SettingsPalette;
use crate::settings_type::{TheType, written};
use crate::{Contrast, RenderError, SettingsRow, SettingsWindow, WindowControlLabels};

/// How Settings looks, as the person's appearance and language decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text size, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides which side the side column
    /// is on.
    pub reading: Direction,
    /// The design's palette, or the one high contrast decides
    /// (`crate::access_contrast`).
    pub contrast: Contrast,
    /// The person's accent, which selections and switches are drawn in.
    pub accent: Colour,
}

/// Settings for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SettingsPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The window, while Settings is open.
    pub(crate) panel: Option<Area>,
    /// The window and everything on it, as one raster.
    pub(crate) washed: Option<Washed>,
    /// Every category, where it is drawn, in the side column's order.
    pub(crate) categories: Vec<(SettingsCategory, Area)>,
    /// Every target on the page that is in view, where it is drawn.
    pub(crate) targets: Vec<ATargetDrawn>,
    /// Where the keyboard's outline is drawn round, when it is anywhere.
    pub(crate) focused: Option<Area>,
    /// Every piece of text drawn, where it is drawn.
    pub(crate) texts: Vec<SettingsText>,
    /// Whether any of the page is out of view above or below.
    pub(crate) out_of_view: bool,
    /// The list a picker opened, while it is open.
    pub(crate) list: Option<Area>,
    /// Every entry of that list in view, where it is drawn.
    pub(crate) entries: Vec<(crate::SettingsRow, Area)>,
}

impl SettingsPicture {
    /// Nothing drawn, for an output of `size`.
    const fn empty(size: (i32, i32)) -> Self {
        Self {
            size,
            panel: None,
            washed: None,
            categories: Vec::new(),
            targets: Vec::new(),
            focused: None,
            texts: Vec::new(),
            out_of_view: false,
            list: None,
            entries: Vec::new(),
        }
    }

    /// Whether nothing at all is drawn.
    pub(crate) const fn is_empty(&self) -> bool {
        self.washed.is_none()
    }
}

/// The provider form on screen, and what the machine answered about it.
///
/// **One value rather than two arguments**, because a refusal without a form is
/// not a state this surface can be in: the sentence is *about* what is on the
/// screen, and a form that has closed takes its answer with it.
#[derive(Debug, Clone, Copy)]
pub struct AFormOnScreen<'a> {
    /// What a person has typed so far.
    pub form: &'a alo_setting_up::a_provider_is_filled_in::AProviderBeingAddded,
    /// Why the last attempt to save it did not happen, when one did not.
    pub not_saved: Option<crate::a_provider_is_added_in_settings::WhyNotSaved>,
}

/// Lay Settings out for an output of `size` drawn at `display`, and
/// rasterise it.
///
/// # Errors
/// [`RenderError::SettingsScene`] when Settings is open and the output has no
/// room for its window, or the person's accent is not one they may choose.
#[expect(
    clippy::too_many_arguments,
    reason = "each is a different thing about one frame of Settings, and a struct would be one call"
)]
pub(crate) fn picture(
    window: &SettingsWindow,
    strings: &Strings,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: SettingsLook,
    now: SystemTime,
    // **The provider a person is filling in, when they are.** Handed in
    // rather than read off the window, because the form lives on the `Server`.
    filling_in: Option<AFormOnScreen<'_>>,
    display: DisplayScale,
) -> Result<SettingsPicture, RenderError> {
    let mut picture = SettingsPicture::empty(size);
    let Some(open) = window.open() else {
        return Ok(picture);
    };
    let palette = SettingsPalette::of(look).map_err(|_| RenderError::SettingsScene)?;
    let measure = Measure::of(look.scale).on(display);
    let factor = measure.factor();
    let design = |pixels: i32| -> u32 {
        let it = (pixels.max(0) as f32 / factor).floor();
        it as u32
    };
    let frame =
        TheFrame::on(design(size.0), design(size.1)).map_err(|_| RenderError::SettingsScene)?;
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));

    let window_area = Rectangle::new(
        (px(frame.left), px(frame.top)).into(),
        (px(frame.wide), px(frame.tall)).into(),
    );
    picture.panel = Some(window_area);
    let raised = crate::how_a_panel_is_raised::HowItIsRaised::FLOATING.at(f64::from(factor));
    let reach = raised.reach();
    let screen: Area = Rectangle::new((0, 0).into(), size.into());
    let canvas_area = Rectangle::new(
        (window_area.loc.x - reach, window_area.loc.y - reach).into(),
        (
            window_area.size.w + 2 * reach,
            window_area.size.h + 2 * reach,
        )
            .into(),
    )
    .intersection(screen)
    .ok_or(RenderError::SettingsScene)?;
    let mut canvas = Canvas::over(canvas_area);
    // **For a reader of a right-to-left language the whole window is turned
    // round**: the side column on the right, every row's mark at its right-hand
    // end. Words are placed turned and drawn as they were shaped.
    if look.reading == Direction::RightToLeft {
        canvas.turned_round(window_area);
    }
    let corner = px(m::THE_WINDOWS_CORNER) as f32;
    canvas.shadow(
        window_area,
        corner,
        AShadow {
            falls: raised.falls(),
            blurs: raised.blurs(),
            spreads: raised.spreads(),
            darkness: raised.darkness(),
        },
    );
    canvas.fill(window_area, corner, palette.surface, 1.0);

    // **The side column**, on the canvas colour, inside the window's own
    // rounded shape: filled as the window is and cut where it ends.
    let line = px(m::A_HAIRLINE);
    let side_wide = px(frame.side);
    let side = Rectangle::new(
        (window_area.loc.x + line, window_area.loc.y + line).into(),
        (side_wide, window_area.size.h - 2 * line).into(),
    );
    canvas.clipped_to(side);
    canvas.fill(window_area, corner, palette.canvas, 1.0);
    canvas.fill(
        Rectangle::new(
            (side.loc.x + side.size.w - line, side.loc.y).into(),
            (line, side.size.h).into(),
        ),
        0.0,
        palette.edge,
        1.0,
    );
    the_side_column(
        &mut canvas,
        &mut picture,
        labels,
        (side, measure),
        (palette, strings),
        open,
    );
    canvas.unclipped();

    // **The page**, in its own view, scrolled to keep the keyboard in it.
    let view = Rectangle::new(
        (side.loc.x + side.size.w, window_area.loc.y + line).into(),
        (
            window_area.loc.x + window_area.size.w - line - (side.loc.x + side.size.w),
            window_area.size.h - 2 * line,
        )
            .into(),
    );
    let page = filling_in.map_or_else(
        || crate::settings_page::the_page(open, open.category, strings, now),
        |on_screen| {
            crate::settings_page::the_form_page(
                crate::a_provider_being_added_is_drawn::the_form(
                    on_screen.form,
                    on_screen.not_saved,
                    strings,
                ),
                strings,
            )
        },
    );
    let mut laying = Laying {
        fonts: &mut labels.fonts,
        measure,
        palette,
        left: window_area.loc.x + px(frame.the_column_starts()),
        wide: px(frame.column),
        above: px(frame.above_the_content),
    };
    let laid = laid_out(&page, &mut laying);
    let focused_row = window.focused(now);
    let target = focused_row
        .as_ref()
        .and_then(|row| laid.targets.iter().find(|it| it.row == *row));
    let scrolled = scrolled_to(target, laid.tall, view.size.h);
    let top = view.loc.y - scrolled;
    picture.out_of_view = scrolled > 0 || laid.tall - scrolled > view.size.h;
    canvas.clipped_to(view);
    painted(&laid.drawn, &mut canvas, top, factor);
    let moved = |area: Area| Rectangle::new((area.loc.x, area.loc.y + top).into(), area.size);
    for drawn in &laid.targets {
        let area = moved(drawn.area);
        if area.intersection(view).is_some() {
            picture.targets.push(ATargetDrawn {
                area,
                ..drawn.clone()
            });
        }
    }
    for text in &laid.texts {
        let area = moved(text.area);
        if area.intersection(view).is_some() {
            picture.texts.push(SettingsText {
                area,
                ..text.clone()
            });
        }
    }
    if let Some(target) = target {
        let area = moved(target.area);
        a_focus_ring(
            &mut canvas,
            area,
            m::A_CARDS_CORNER,
            measure,
            palette,
            window.is_waiting_for_a_chord(),
        );
        picture.focused = Some(area);
    }
    canvas.unclipped();

    // **The list a picker opened, over the page**, hanging from its button.
    if let Some(picker) = &open.picker {
        let row = SettingsRow::OpenThePicker(picker.which);
        if let Some(anchor) = laid.targets.iter().find(|it| it.row == row) {
            let area = moved(anchor.area);
            let (right, centre) = (
                area.loc.x + area.size.w - px(m::BESIDE_A_ROW),
                area.loc.y + area.size.h / 2,
            );
            let button = Rectangle::new(
                (right, centre - px(m::A_TARGET) / 2).into(),
                (0, px(m::A_TARGET)).into(),
            );
            let list = crate::settings_drawn_picker::the_list(
                &mut canvas,
                open,
                picker,
                crate::settings_drawn_picker::TheListsRoom {
                    fonts: &mut labels.fonts,
                    measure,
                    palette,
                    strings,
                    button,
                    window: window_area,
                    shadow: AShadow {
                        falls: raised.falls(),
                        blurs: raised.blurs(),
                        spreads: raised.spreads(),
                        darkness: raised.darkness(),
                    },
                },
            );
            picture.list = Some(list.area);
            picture.texts.extend(list.texts);
            picture.entries = list
                .entries
                .into_iter()
                .map(|(entry, area)| (entry.row, area))
                .collect();
        }
    }
    canvas.border(window_area, corner, line as f32, palette.edge);

    if look.reading == Direction::RightToLeft {
        turned_round(window_area, &mut picture);
    }
    picture.washed = Some(canvas.washed());
    Ok(picture)
}

/// How far the page is scrolled for the keyboard's row to be in view: as
/// little as keeps it whole, and the start of its section with it when both
/// fit. Never past the page's end.
fn scrolled_to(target: Option<&ATargetDrawn>, tall: i32, view: i32) -> i32 {
    let most = (tall - view).max(0);
    let Some(target) = target else {
        return 0;
    };
    let (from, to) = (target.area.loc.y, target.area.loc.y + target.area.size.h);
    let mut scrolled = (to - view).max(0);
    if from < scrolled {
        scrolled = from;
    }
    if to - target.section_starts <= view && target.section_starts < scrolled {
        scrolled = target.section_starts;
    }
    scrolled.clamp(0, most)
}

/// The keyboard's outline on `area`'s own edge, doubled while Settings waits
/// for a chord.
///
/// **On the edge, not round it.** Until 2026-11-01 the outline stood two units
/// off the row and two thick, so it was eight units wider and taller than what
/// it marked: on the side column it reached past the chosen category's own
/// pill to the column's edge, the first thing a person saw on opening Settings
/// and the one thing the design (`551:*`) does not draw. It is the same navy,
/// the same thickness, and still on every row the keyboard reaches; it is no
/// bigger than the row.
fn a_focus_ring(
    canvas: &mut Canvas,
    area: Area,
    corner: u32,
    measure: Measure,
    palette: SettingsPalette,
    doubled: bool,
) {
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let thick = px(m::THE_FOCUS_RING);
    let inner = px(corner);
    canvas.border(area, inner as f32, thick as f32, palette.ink);
    if doubled {
        // A second ring one ring's width inside the first, so waiting for a
        // chord still reads differently from an ordinary focus.
        let gap = 2 * thick;
        let inside = Rectangle::new(
            (area.loc.x + gap, area.loc.y + gap).into(),
            (area.size.w - 2 * gap, area.size.h - 2 * gap).into(),
        );
        canvas.border(
            inside,
            (inner - gap).max(0) as f32,
            thick as f32,
            palette.ink,
        );
    }
}

/// The side column's contents: the heading, the two groups of categories, and
/// the footer at its foot.
fn the_side_column(
    canvas: &mut Canvas,
    picture: &mut SettingsPicture,
    labels: &mut WindowControlLabels,
    (side, measure): (Area, Measure),
    (palette, strings): (SettingsPalette, &Strings),
    open: &crate::settings_window::Open,
) {
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    let fonts = &mut labels.fonts;
    let line = px(m::A_HAIRLINE);
    let left = side.loc.x + px(m::INSIDE_THE_SIDE_COLUMN);
    let wide = side.size.w - line - 2 * px(m::INSIDE_THE_SIDE_COLUMN);
    let inside_heading = px(m::INSIDE_THE_HEADING);
    let beside = px(m::BESIDE_A_CATEGORY);
    let mut y = side.loc.y + px(m::INSIDE_THE_SIDE_COLUMN);
    let mut put = |canvas: &mut Canvas,
                   fonts: &mut cosmic_text::FontSystem,
                   words: String,
                   style: TheType,
                   (x, y, width): (i32, i32, i32),
                   colours: ([u8; 3], [u8; 3])|
     -> i32 {
        let shaped = written(fonts, &words, style, width, measure, colours).cropped_to_its_ink();
        let tall = shaped.height;
        picture.texts.push(SettingsText {
            text: words,
            area: Rectangle::new((x, y).into(), (shaped.width_of_box, tall).into()),
        });
        canvas.words(x, y, &shaped);
        tall
    };

    let heading_x = left + inside_heading;
    let heading_wide = wide - 2 * inside_heading;
    let tall = put(
        canvas,
        fonts,
        said(strings, crate::the_settings_words::SETTINGS),
        TheType::Title,
        (heading_x, y + inside_heading, heading_wide),
        (palette.canvas, palette.ink),
    );
    let under_at = y + inside_heading + tall + px(m::UNDER_THE_TITLE);
    let under = put(
        canvas,
        fonts,
        said(strings, crate::the_settings_words::EVERYTHING_IN_ONE_PLACE),
        TheType::Body,
        (heading_x, under_at, heading_wide),
        (palette.canvas, palette.muted),
    );
    y = under_at + under + inside_heading + px(m::BETWEEN_SIDE_ITEMS);

    let mut group = None;
    let mut chosen_at = None;
    for category in SettingsCategory::ALL {
        if group != Some(category.group()) {
            group = Some(category.group());
            let tall = put(
                canvas,
                fonts,
                said(strings, category.group().word()),
                TheType::SmallLabel,
                (left + beside, y + px(m::ABOVE_A_GROUP), wide - 2 * beside),
                (palette.canvas, palette.muted),
            );
            y += px(m::ABOVE_A_GROUP) + tall + px(m::BELOW_A_GROUP) + px(m::BETWEEN_SIDE_ITEMS);
        }
        let item = Rectangle::new((left, y).into(), (wide, px(m::A_TARGET)).into());
        let chosen = category == open.category;
        let (style, ground, ink) = if chosen {
            canvas.fill(item, px(m::A_SMALL_CORNER) as f32, palette.chosen, 1.0);
            chosen_at = Some(item);
            (TheType::Name, palette.chosen, palette.ink)
        } else {
            (TheType::Body, palette.canvas, palette.secondary)
        };
        // **The icon in the accent when its page is shown**, `550:106`, and in
        // `text/secondary` otherwise.
        let side = px(m::A_CATEGORYS_ICON);
        category.icon().drawn(
            canvas,
            Rectangle::new(
                (left + beside, y + (item.size.h - side) / 2).into(),
                (side, side).into(),
            ),
            if chosen {
                palette.accent
            } else {
                palette.secondary
            },
        );
        let before = beside + px(m::A_CATEGORYS_ICON) + px(m::AFTER_A_CATEGORYS_ICON);
        let line_tall = px(style.line());
        put(
            canvas,
            fonts,
            category.said(strings),
            style,
            (
                left + before,
                y + (item.size.h - line_tall) / 2,
                wide - before - beside,
            ),
            (ground, ink),
        );
        picture.categories.push((category, item));
        y += item.size.h + px(m::BETWEEN_SIDE_ITEMS);
    }

    let footer_tall = px(TheType::SmallLabel.line());
    let bottom = side.loc.y + side.size.h - px(m::INSIDE_THE_SIDE_COLUMN) - px(m::BELOW_THE_FOOTER);
    put(
        canvas,
        fonts,
        said(strings, crate::the_settings_words::YOURS_TO_CHANGE),
        TheType::SmallLabel,
        (
            left + beside,
            (bottom - footer_tall).max(y),
            wide - 2 * beside,
        ),
        (palette.canvas, palette.muted),
    );
    // **The keyboard's outline last**, so nothing in the column covers it.
    if let Some(item) = chosen_at.filter(|_| !open.in_the_page) {
        a_focus_ring(canvas, item, m::A_SMALL_CORNER, measure, palette, false);
        picture.focused = Some(item);
    }
}

/// The record of where each thing is drawn, turned round the window for a
/// reader of a right-to-left language — the canvas drew it turned already
/// (`crate::settings_canvas::Canvas::turned_round`), so a test reads where it
/// really is.
fn turned_round(window: Area, picture: &mut SettingsPicture) {
    let (left, wide) = (window.loc.x, window.size.w);
    let flip = |area: Area| -> Area {
        Rectangle::new(
            (2 * left + wide - area.loc.x - area.size.w, area.loc.y).into(),
            area.size,
        )
    };
    picture.focused = picture.focused.map(flip);
    for (_, area) in &mut picture.categories {
        *area = flip(*area);
    }
    for target in &mut picture.targets {
        target.area = flip(target.area);
    }
    picture.list = picture.list.map(flip);
    for (_, area) in &mut picture.entries {
        *area = flip(*area);
    }
    for text in &mut picture.texts {
        text.area = flip(text.area);
    }
}

#[cfg(test)]
#[path = "settings_raster_tests.rs"]
mod tests;
