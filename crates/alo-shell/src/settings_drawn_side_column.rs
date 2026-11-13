//! **The side column of Settings, drawn**: *Settings*, the line under it, the
//! two groups of categories and the footer — and, where they do not all fit,
//! **scrolled**.
//!
//! # Why it scrolls
//!
//! The owner, 2026-11-13, asked whether a tenth category may make the column
//! taller than a 1280 × 800 screen leaves it: *"The side column scrolls"*. So
//! on a screen where every entry does not fit, the column scrolls, and **nothing
//! in it is hidden or made smaller to fit**. On a screen where it does fit —
//! the design's 1440 × 960 at the ordinary text size — it is drawn exactly as
//! before, with the footer at the column's foot.
//!
//! # How far it is scrolled
//!
//! Laid out first as though nothing were scrolled, then drawn moved up by
//! [`scrolled_for`]: where the wheel left it, and — after the keyboard or a
//! press moved to a category — **as little further as brings the chosen
//! category whole into view**, so Up, Down, Home and End never choose a page
//! the person cannot see. Never past the column's end. Where the column runs
//! over, the footer follows the last category rather than sitting at the foot,
//! because a footer pinned there would be drawn over a category.
//!
//! A category scrolled under the column's edge is recorded only as far as it is
//! seen (`SettingsPicture::categories`), so a press can never reach one through
//! the window's edge.

use alo_strings::Strings;
use smithay::utils::Rectangle;

use crate::WindowControlLabels;
use crate::desktop_look::Measure;
use crate::painted_text::Shaped;
use crate::settings_canvas::{Area, Canvas};
use crate::settings_categories::SettingsCategory;
use crate::settings_drawn_page::SettingsText;
use crate::settings_measures as m;
use crate::settings_page::said;
use crate::settings_palette::SettingsPalette;
use crate::settings_raster::SettingsPicture;
use crate::settings_side_column_scrolls::TheSideColumnsScroll;
use crate::settings_type::{TheType, written};

/// The side column as one frame drew it: what a wheel over it needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheSideColumnDrawn {
    /// The column's view, in the picture's pixels.
    pub(crate) view: Area,
    /// How far it was drawn scrolled, in the picture's pixels.
    pub(crate) scrolled: i32,
    /// The furthest it can scroll: nothing where everything fits.
    pub(crate) most: i32,
    /// How far one notch of a wheel moves it: one category and the gap after
    /// it, in the picture's pixels.
    pub(crate) a_notch: i32,
}

impl TheSideColumnDrawn {
    /// Whether anything in the column is out of view, so a wheel has
    /// something to move.
    pub(crate) const fn scrolls(&self) -> bool {
        self.most > 0
    }
}

/// How far the column is scrolled: where `scroll` left it, and — while it
/// follows the keyboard — as little further as keeps `chosen` (its top and
/// bottom, measured from the column's top) whole in a view `view` tall. Never
/// below nothing, and never past `content`'s end.
pub(crate) fn scrolled_for(
    scroll: TheSideColumnsScroll,
    content: i32,
    view: i32,
    chosen: Option<(i32, i32)>,
) -> i32 {
    let most = (content - view).max(0);
    let mut scrolled = scroll.scrolled.clamp(0, most);
    if scroll.follows
        && let Some((top, bottom)) = chosen
    {
        if bottom - scrolled > view {
            scrolled = bottom - view;
        }
        if top < scrolled {
            scrolled = top;
        }
    }
    scrolled.clamp(0, most)
}

/// A piece of the column's words, shaped, and where it goes before scrolling.
struct Placed {
    /// Its left edge.
    x: i32,
    /// Its top, as though nothing were scrolled.
    y: i32,
    /// The words, shaped.
    shaped: Shaped,
    /// The words, as the picture records them.
    text: String,
}

/// One category, where it goes before scrolling.
struct ACategoryPlaced {
    /// Which.
    category: SettingsCategory,
    /// Its whole row.
    item: Area,
    /// Whether its page is shown.
    chosen: bool,
    /// Its name.
    name: Placed,
}

/// The side column's contents inside `side`, scrolled as `open` says, and
/// recorded on `picture`.
pub(crate) fn the_side_column(
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
    let shape = |fonts: &mut cosmic_text::FontSystem,
                 words: String,
                 style: TheType,
                 (x, y, width): (i32, i32, i32),
                 colours: ([u8; 3], [u8; 3])|
     -> Placed {
        let shaped = written(fonts, &words, style, width, measure, colours).cropped_to_its_ink();
        Placed {
            x,
            y,
            shaped,
            text: words,
        }
    };

    // **Laid out first, as though nothing were scrolled.**
    let mut words: Vec<Placed> = Vec::new();
    let mut y = side.loc.y + px(m::INSIDE_THE_SIDE_COLUMN);
    let heading_x = left + inside_heading;
    let heading_wide = wide - 2 * inside_heading;
    let title = shape(
        fonts,
        said(strings, crate::the_settings_words::SETTINGS),
        TheType::Title,
        (heading_x, y + inside_heading, heading_wide),
        (palette.canvas, palette.ink),
    );
    let under_at = y + inside_heading + title.shaped.height + px(m::UNDER_THE_TITLE);
    words.push(title);
    let under = shape(
        fonts,
        said(strings, crate::the_settings_words::EVERYTHING_IN_ONE_PLACE),
        TheType::Body,
        (heading_x, under_at, heading_wide),
        (palette.canvas, palette.muted),
    );
    y = under_at + under.shaped.height + inside_heading + px(m::BETWEEN_SIDE_ITEMS);
    words.push(under);

    let mut categories: Vec<ACategoryPlaced> = Vec::new();
    let mut group = None;
    for category in SettingsCategory::ALL {
        if group != Some(category.group()) {
            group = Some(category.group());
            let label = shape(
                fonts,
                said(strings, category.group().word()),
                TheType::SmallLabel,
                (left + beside, y + px(m::ABOVE_A_GROUP), wide - 2 * beside),
                (palette.canvas, palette.muted),
            );
            y += px(m::ABOVE_A_GROUP)
                + label.shaped.height
                + px(m::BELOW_A_GROUP)
                + px(m::BETWEEN_SIDE_ITEMS);
            words.push(label);
        }
        let item = Rectangle::new((left, y).into(), (wide, px(m::A_TARGET)).into());
        let chosen = category == open.category;
        let (style, ground, ink) = if chosen {
            (TheType::Name, palette.chosen, palette.ink)
        } else {
            (TheType::Body, palette.canvas, palette.secondary)
        };
        let before = beside + px(m::A_CATEGORYS_ICON) + px(m::AFTER_A_CATEGORYS_ICON);
        let line_tall = px(style.line());
        let name = shape(
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
        categories.push(ACategoryPlaced {
            category,
            item,
            chosen,
            name,
        });
        y += item.size.h + px(m::BETWEEN_SIDE_ITEMS);
    }

    // **The footer at the column's foot where everything fits**, and after the
    // last category where it does not.
    let footer_tall = px(TheType::SmallLabel.line());
    let foot = side.loc.y + side.size.h - px(m::INSIDE_THE_SIDE_COLUMN) - px(m::BELOW_THE_FOOTER);
    let footer = shape(
        fonts,
        said(strings, crate::the_settings_words::YOURS_TO_CHANGE),
        TheType::SmallLabel,
        (left + beside, (foot - footer_tall).max(y), wide - 2 * beside),
        (palette.canvas, palette.muted),
    );
    let content_ends = footer.y
        + footer_tall
        + px(m::BELOW_THE_FOOTER)
        + px(m::INSIDE_THE_SIDE_COLUMN);
    words.push(footer);

    // **Then scrolled.**
    let content = content_ends - side.loc.y;
    let chosen = categories.iter().find(|it| it.chosen).map(|it| {
        let top = it.item.loc.y - side.loc.y;
        (top, top + it.item.size.h)
    });
    let scrolled = scrolled_for(open.side_column, content, side.size.h, chosen);
    let moved = |area: Area| Rectangle::new((area.loc.x, area.loc.y - scrolled).into(), area.size);
    for placed in &words {
        put(canvas, picture, placed, (side, scrolled));
    }
    let mut chosen_at = None;
    for placed in &categories {
        let item = moved(placed.item);
        let Some(seen) = item.intersection(side) else {
            continue;
        };
        if placed.chosen {
            canvas.fill(item, px(m::A_SMALL_CORNER) as f32, palette.chosen, 1.0);
            chosen_at = Some(item);
        }
        // **The icon in the accent when its page is shown**, `550:106`, and in
        // `text/secondary` otherwise.
        let icon = px(m::A_CATEGORYS_ICON);
        placed.category.icon().drawn(
            canvas,
            Rectangle::new(
                (left + beside, item.loc.y + (item.size.h - icon) / 2).into(),
                (icon, icon).into(),
            ),
            if placed.chosen {
                palette.accent
            } else {
                palette.secondary
            },
        );
        put(canvas, picture, &placed.name, (side, scrolled));
        picture.categories.push((placed.category, seen));
    }
    // **The keyboard's outline last**, so nothing in the column covers it.
    if let Some(item) = chosen_at.filter(|_| !open.in_the_page) {
        crate::settings_raster::a_focus_ring(canvas, item, m::A_SMALL_CORNER, measure, palette, false);
        picture.focused = Some(item);
    }
    picture.side_column = Some(TheSideColumnDrawn {
        view: side,
        scrolled,
        most: (content - side.size.h).max(0),
        a_notch: px(m::A_TARGET + m::BETWEEN_SIDE_ITEMS),
    });
}

/// Draw `placed`, moved up by `scrolled`, where any of it is in `side`, and
/// record it on `picture`.
fn put(
    canvas: &mut Canvas,
    picture: &mut SettingsPicture,
    placed: &Placed,
    (side, scrolled): (Area, i32),
) {
    let area = Rectangle::new(
        (placed.x, placed.y - scrolled).into(),
        (placed.shaped.width_of_box, placed.shaped.height).into(),
    );
    if area.intersection(side).is_some() {
        canvas.words(area.loc.x, area.loc.y, &placed.shaped);
        picture.texts.push(SettingsText {
            text: placed.text.clone(),
            area,
        });
    }
}

#[cfg(test)]
#[path = "settings_drawn_side_column_tests.rs"]
mod tests;
