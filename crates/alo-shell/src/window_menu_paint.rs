//! What the window panel draws: its body, its placement pictures, its rows and
//! their words.
//!
//! Every figure is `11:51`'s, read with `get_design_context` on 2026-11-09 and
//! recorded in `docs/design/the-window-controls.md`; every colour is one of the
//! twelve roles, mapped from the design's literal where it has no variable
//! (the note's *Readings taken* says which).
//!
//! | part | figure |
//! |---|---|
//! | body | `bg/surface`, corner 16, line 1 `border/default`, shadow 0 / 16 blur 40 at 10 % of navy |
//! | heading *Window* | SemiBold 14, navy, at (22, 20) |
//! | a placement picture | 94 × 94, corner 10, `bg/canvas`; an outline 46 × 30 at (24, 16), corner 4, line 1; the placement inside, corner 2 |
//! | its label | Medium 12, navy, centred, at y 62 |
//! | a row | 304 × 40, corner 7; its mark 12 × 12 at (2, 12); its label Regular 13 at x 30 |
//!
//! **The choice in hand wears navy**: its placement is navy and the others are
//! `text/muted`, and the keyboard's choice also gets the navy ring — so which
//! choice is in hand is never told by colour alone.

use std::sync::OnceLock;

use alo_appearance::Role;
use alo_vector_shapes::AShape;
use cosmic_text::{Attrs, FontSystem, Metrics, Weight};
use smithay::utils::{Logical, Physical, Rectangle};

use crate::painted::{Inked, Washed};
use crate::window_menu::{A_PICTURE_IS, WhatARowDoes, WindowMenu};

/// What a person is doing to the panel, which is all that changes its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PointingAtTheMenu {
    /// The choice the pointer is over, if it is over one.
    pub hovered: Option<WhatARowDoes>,
    /// The choice the keyboard is on, if it is on one.
    pub focused: Option<WhatARowDoes>,
    /// Where Left or Right in hand would put the window, while it is.
    pub preview: Option<ThePreview>,
}

/// The outline of where a placement would put a window, before it is chosen —
/// `11:84`, *Left half · preview*.
///
/// **It never moves or resizes the real window**: it is drawn over the
/// screen, above every window, and is gone as soon as the choice is not in
/// hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThePreview {
    /// The destination, band included, in the band's own units on this
    /// display.
    pub outline: Rectangle<i32, Logical>,
    /// What its corner says: which half and that it is a preview, or that the
    /// window cannot fit there.
    pub says: alo_strings::Word,
}

/// The preview's line and corner: `11:90`, 2 wide, corner 15.
const THE_PREVIEWS_LINE: (f32, f32) = (2.0, 15.0);
/// Where the preview's words sit: 20 in from its left, 30 up from its foot.
const THE_PREVIEWS_WORDS: (i32, i32) = (20, 30);
/// The room round the preview's words, so they read over any window.
const ROUND_THE_PREVIEWS_WORDS: i32 = 6;

impl PointingAtTheMenu {
    /// The choice in hand: the keyboard's, or else the pointer's.
    #[must_use]
    pub const fn in_hand(self) -> Option<WhatARowDoes> {
        match self.focused {
            Some(it) => Some(it),
            None => self.hovered,
        }
    }
}

/// The panel's corner. `11:51`.
const THE_PANELS_CORNER: f32 = 16.0;
/// A placement picture's corner. `11:55`.
const A_PICTURES_CORNER: f32 = 10.0;
/// A row's corner. `11:68`.
const A_ROWS_CORNER: f32 = 7.0;
/// *Help with this window*'s corner. `11:76`.
const ALOS_ROWS_CORNER: f32 = 9.0;
/// Where *alo* starts across that row, and where its label does. `11:77`,
/// `11:78`; both are centred down the row's 52.
const ALOS_ROWS_WORDS: (i32, i32) = (12, 51);
/// The shadow: how far down, how soft, how strong. `11:51`'s effect.
const THE_SHADOW: (f32, f32, f32) = (16.0, 40.0, 0.10);
/// The outline inside a placement picture: where, and how big. `11:56`.
const THE_OUTLINE: (i32, i32, i32, i32) = (24, 16, 46, 30);
/// The placement inside that outline: Maximise `11:61`, 38 × 22 at (4, 4);
/// Left `11:57`, 18 × 22 at (4, 4); Right 18 × 22 at (24, 4).
const fn the_placement(does: WhatARowDoes) -> (i32, i32, i32, i32) {
    match does {
        WhatARowDoes::Left => (4, 4, 18, 22),
        WhatARowDoes::Right => (24, 4, 18, 22),
        _ => (4, 4, 38, 22),
    }
}
/// Where a picture's label sits down it. `11:58`.
const A_LABEL_SITS_AT: i32 = 62;
/// Where a row's mark is. `11:69`, inside its exported 13.4 box.
const A_ROWS_MARK: (i32, i32, i32) = (1, 11, 13);
/// Where a row's label starts across and down. `11:70`.
const A_ROWS_LABEL: (i32, i32) = (30, 10);

/// The expand mark beside Full screen, read once.
fn the_full_screen_mark() -> Option<&'static AShape> {
    /// Read once, or not at all.
    static MARK: OnceLock<Option<AShape>> = OnceLock::new();
    MARK.get_or_init(|| {
        AShape::read(include_str!("../icons/the-window-controls/full-screen.svg")).ok()
    })
    .as_ref()
}

/// A role as three bytes.
pub(crate) fn rgb(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// The panel, drawn: its rasters, back to front, and its words.
#[must_use]
pub(crate) fn the_panel_drawn(
    menu: &WindowMenu,
    pointing: PointingAtTheMenu,
    fonts: &mut FontSystem,
    scale: u32,
    output: (i32, i32),
) -> (Vec<Washed>, Vec<Inked>) {
    let fraction = f64::from(scale.max(1)) / 100.0;
    let px = |it: i32| scaled(it, scale);
    let mut rasters = Vec::new();
    rasters.extend(the_body(in_pixels(menu.panel, scale), fraction));
    for divider in std::iter::once(menu.divider).chain(menu.before_alo) {
        rasters.extend(a_rounded_fill(
            in_pixels(divider, scale),
            0.0,
            rgb(Role::BorderDefault),
        ));
    }
    let in_hand = pointing.in_hand();
    for row in &menu.rows {
        let area = in_pixels(row.row, scale);
        let held = in_hand == Some(row.does);
        let alos = row.does == WhatARowDoes::HelpWithThisWindow;
        let ground = if alos {
            // **Teal never decorates an idle entry** (§7): the row is the
            // panel's own quiet ground, as `11:76` draws it.
            if held {
                rgb(Role::BgCool)
            } else {
                rgb(Role::BgCanvas)
            }
        } else if row.does.is_a_picture() {
            if held {
                rgb(Role::BgCool)
            } else {
                rgb(Role::BgCanvas)
            }
        } else if held {
            rgb(Role::BgCool)
        } else {
            rgb(Role::BgSurface)
        };
        let corner = if alos {
            ALOS_ROWS_CORNER
        } else if row.does.is_a_picture() {
            A_PICTURES_CORNER
        } else {
            A_ROWS_CORNER
        };
        rasters.extend(a_rounded_fill(
            area,
            as_f32(f64::from(corner) * fraction),
            ground,
        ));
        if row.does.is_a_picture() {
            rasters.extend(the_placement_picture(
                row.row, row.does, held, scale, fraction,
            ));
        } else if !alos
            // The word *alo* stands where the other rows have a mark.
            && let Some(mark) = the_full_screen_mark()
        {
            let at = Rectangle::new(
                (row.row.loc.x + A_ROWS_MARK.0, row.row.loc.y + A_ROWS_MARK.1).into(),
                (A_ROWS_MARK.2, A_ROWS_MARK.2).into(),
            );
            rasters.extend(a_mark(in_pixels(at, scale), mark, rgb(Role::TextPrimary)));
        }
        if pointing.focused == Some(row.does) {
            rasters.extend(a_ring(area, as_f32(f64::from(corner) * fraction), fraction));
        }
    }
    // **The preview goes under the panel**, which stays readable over it.
    let mut words = Vec::new();
    let mut under = Vec::new();
    if let Some(preview) = pointing.preview {
        under.extend(a_ring_of(
            in_pixels(preview.outline, scale),
            as_f32(f64::from(THE_PREVIEWS_LINE.1) * fraction),
            as_f32(f64::from(THE_PREVIEWS_LINE.0) * fraction).max(1.0),
            rgb(Role::TextPrimary),
        ));
        let shaped = in_a_weight(
            fonts,
            preview.says.says(),
            Weight(400),
            metrics(12.0, 16.0, fraction),
            px((preview.outline.size.w - 2 * THE_PREVIEWS_WORDS.0).max(1)),
            rgb(Role::BgSurface),
            rgb(Role::TextMuted),
        )
        .cropped_to_its_ink();
        let across = px(preview.outline.loc.x + THE_PREVIEWS_WORDS.0);
        let down = px(preview.outline.loc.y + preview.outline.size.h - THE_PREVIEWS_WORDS.1);
        let room = px(ROUND_THE_PREVIEWS_WORDS);
        under.extend(a_rounded_fill(
            Rectangle::new(
                (across - room, down - room).into(),
                (shaped.width + 2 * room, shaped.height + 2 * room).into(),
            ),
            as_f32(4.0 * fraction),
            rgb(Role::BgSurface),
        ));
        words.extend(shaped.placed(across, down, output.1));
    }
    rasters.splice(0..0, under);

    let navy = rgb(Role::TextPrimary);
    let heading = crate::painted_text::sentence(
        fonts,
        alo_access::words::THE_WINDOW_PANEL.says(),
        px(crate::window_menu::A_ROW_IS_WIDE),
        metrics(14.0, 19.0, fraction),
        rgb(Role::BgSurface),
        navy,
    )
    .cropped_to_its_ink();
    words.extend(heading.placed(px(menu.heading.x), px(menu.heading.y), output.1));
    for row in &menu.rows {
        let held = in_hand == Some(row.does);
        let label = row.does.its_own_word().says();
        if row.does.is_a_picture() {
            let ground = if held {
                rgb(Role::BgCool)
            } else {
                rgb(Role::BgCanvas)
            };
            let shaped = in_a_weight(
                fonts,
                label,
                the_labels_weight(row.does),
                metrics(12.0, 16.0, fraction),
                px(A_PICTURE_IS),
                ground,
                navy,
            )
            .cropped_to_its_ink();
            let across = px(row.row.loc.x) + (px(A_PICTURE_IS) - shaped.width).max(0) / 2;
            words.extend(shaped.placed(across, px(row.row.loc.y + A_LABEL_SITS_AT), output.1));
        } else if row.does == WhatARowDoes::HelpWithThisWindow {
            words.extend(the_alo_rows_words(
                fonts, row.row, label, held, scale, fraction, output,
            ));
        } else {
            let ground = if held {
                rgb(Role::BgCool)
            } else {
                rgb(Role::BgSurface)
            };
            let shaped = in_a_weight(
                fonts,
                label,
                the_labels_weight(row.does),
                metrics(13.0, 18.0, fraction),
                px(row.row.size.w - A_ROWS_LABEL.0),
                ground,
                navy,
            )
            .cropped_to_its_ink();
            words.extend(shaped.placed(
                px(row.row.loc.x + A_ROWS_LABEL.0),
                px(row.row.loc.y + A_ROWS_LABEL.1),
                output.1,
            ));
        }
    }
    (rasters, words)
}

/// *Help with this window*'s words: **alo** in SemiBold 15, then the label in
/// Regular 13, both navy and centred down the row — `11:76`, which draws the
/// assistant's name where the other rows have a mark, and no teal: teal is for
/// alo working, never an idle entry.
fn the_alo_rows_words(
    fonts: &mut FontSystem,
    row: Rectangle<i32, Logical>,
    label: &str,
    held: bool,
    scale: u32,
    fraction: f64,
    output: (i32, i32),
) -> Vec<Inked> {
    let px = |it: i32| scaled(it, scale);
    let ground = if held {
        rgb(Role::BgCool)
    } else {
        rgb(Role::BgCanvas)
    };
    let navy = rgb(Role::TextPrimary);
    let mut words = Vec::new();
    for (text, weight, (size, line), across) in [
        // The assistant's name, which is never translated.
        ("alo", Weight::SEMIBOLD, (15.0, 20.0), ALOS_ROWS_WORDS.0),
        (label, Weight::NORMAL, (13.0, 18.0), ALOS_ROWS_WORDS.1),
    ] {
        let shaped = in_a_weight(
            fonts,
            text,
            weight,
            metrics(size, line, fraction),
            px((row.size.w - across).max(1)),
            ground,
            navy,
        )
        .cropped_to_its_ink();
        let down = px(row.loc.y) + (px(row.size.h) - shaped.height).max(0) / 2;
        words.extend(shaped.placed(px(row.loc.x + across), down, output.1));
    }
    words
}

/// The weight a choice's label is drawn at: **Medium** under a placement
/// picture and **Regular** on a row, as `11:56` and `11:68` give them.
///
/// **Each is a weight a bundled Manrope file declares exactly**, because
/// cosmic-text keeps the family asked for only among faces of exactly the
/// weight asked for (`docs/quirks.md`). Until `fonts/Manrope-Medium.ttf`
/// was bundled, Medium had no Manrope face, and the only face at 500 was Geist
/// Mono Medium — so a booted machine drew *Left*, *Maximise* and *Right* in
/// the terminal's monospace beside *Full screen* in Manrope (2026-11-09).
/// `a_choices_label_is_drawn_in_manrope` shapes each label through the
/// shell's own fonts and reads back the face.
pub(crate) const fn the_labels_weight(does: WhatARowDoes) -> Weight {
    if does.is_a_picture() {
        Weight::MEDIUM
    } else {
        Weight::NORMAL
    }
}

/// Text in alo's own face at another weight than its labels'.
pub(crate) fn in_a_weight(
    fonts: &mut FontSystem,
    text: &str,
    weight: Weight,
    metrics: Metrics,
    width: i32,
    ground: [u8; 3],
    ink: [u8; 3],
) -> crate::painted_text::Shaped {
    let attrs: Attrs<'_> = crate::painted_text::alos_own_text().weight(weight);
    crate::painted_text::sentence_in_runs(
        fonts,
        &[(text, attrs.clone())],
        &attrs,
        width,
        metrics,
        ground,
        ink,
    )
}

/// Type at a size and line height, at the display's scale.
pub(crate) fn metrics(size: f32, line: f32, fraction: f64) -> Metrics {
    let fraction = as_f32(fraction);
    Metrics::new(size * fraction, line * fraction)
}

/// The panel's body: its shadow, its line and its white, as one raster that
/// reaches past the panel by the shadow's softness.
fn the_body(panel: Rectangle<i32, Physical>, fraction: f64) -> Option<Washed> {
    let (down, soft, strength) = THE_SHADOW;
    let (down, soft) = (
        as_f32(f64::from(down) * fraction),
        as_f32(f64::from(soft) * fraction),
    );
    let corner = as_f32(f64::from(THE_PANELS_CORNER) * fraction);
    let line = as_f32(fraction).max(1.0);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a shadow's reach in pixels is well inside an i32"
    )]
    let reach = soft.ceil() as i32;
    let area = Rectangle::<i32, Physical>::new(
        (panel.loc.x - reach, panel.loc.y - reach).into(),
        (
            panel.size.w + 2 * reach,
            panel.size.h + 2 * reach + as_i32(down),
        )
            .into(),
    );
    let (l, t) = (as_f32_i(panel.loc.x), as_f32_i(panel.loc.y));
    let (w, h) = (as_f32_i(panel.size.w), as_f32_i(panel.size.h));
    let navy = rgb(Role::TextPrimary);
    let edging = rgb(Role::BorderDefault);
    let white = rgb(Role::BgSurface);
    crate::a_window_raised::raster(area, |x, y| {
        let d = away(x - l, y - t, w, h, corner);
        if d < 0.5 {
            let inside = (0.5 - d).clamp(0.0, 1.0);
            let within = (0.5 - (d + line)).clamp(0.0, 1.0);
            let colour = if within > 0.5 { white } else { edging };
            return (colour, inside);
        }
        let shadow = away(x - l, y - t - down, w, h, corner).max(0.0);
        let fall = (1.0 - shadow / soft.max(1.0)).clamp(0.0, 1.0);
        (navy, strength * fall * fall)
    })
}

/// A placement picture's outline and the placement in it.
fn the_placement_picture(
    picture: Rectangle<i32, Logical>,
    does: WhatARowDoes,
    held: bool,
    scale: u32,
    fraction: f64,
) -> Vec<Washed> {
    let outline = Rectangle::<i32, Logical>::new(
        (picture.loc.x + THE_OUTLINE.0, picture.loc.y + THE_OUTLINE.1).into(),
        (THE_OUTLINE.2, THE_OUTLINE.3).into(),
    );
    let (x, y, w, h) = the_placement(does);
    let placement = Rectangle::<i32, Logical>::new(
        (outline.loc.x + x, outline.loc.y + y).into(),
        (w, h).into(),
    );
    let fill = if held {
        rgb(Role::TextPrimary)
    } else {
        rgb(Role::TextMuted)
    };
    let mut drawn = Vec::new();
    drawn.extend(a_ring_of(
        in_pixels(outline, scale),
        as_f32(4.0 * fraction),
        as_f32(fraction).max(1.0),
        rgb(Role::TextMuted),
    ));
    drawn.extend(a_rounded_fill(
        in_pixels(placement, scale),
        as_f32(2.0 * fraction),
        fill,
    ));
    drawn
}

/// A filled rounded rectangle.
pub(crate) fn a_rounded_fill(
    area: Rectangle<i32, Physical>,
    corner: f32,
    colour: [u8; 3],
) -> Option<Washed> {
    let (l, t) = (as_f32_i(area.loc.x), as_f32_i(area.loc.y));
    let (w, h) = (as_f32_i(area.size.w), as_f32_i(area.size.h));
    crate::a_window_raised::raster(area, |x, y| {
        let d = away(x - l, y - t, w, h, corner);
        (colour, (0.5 - d).clamp(0.0, 1.0))
    })
}

/// The keyboard's navy ring round a choice, 2 wide.
fn a_ring(area: Rectangle<i32, Physical>, corner: f32, fraction: f64) -> Option<Washed> {
    a_ring_of(
        area,
        corner,
        as_f32(2.0 * fraction).max(1.0),
        rgb(Role::TextPrimary),
    )
}

/// A line `width` wide just inside a rounded rectangle.
pub(crate) fn a_ring_of(
    area: Rectangle<i32, Physical>,
    corner: f32,
    width: f32,
    colour: [u8; 3],
) -> Option<Washed> {
    let (l, t) = (as_f32_i(area.loc.x), as_f32_i(area.loc.y));
    let (w, h) = (as_f32_i(area.size.w), as_f32_i(area.size.h));
    crate::a_window_raised::raster(area, |x, y| {
        let d = away(x - l, y - t, w, h, corner);
        let on = (0.5 - d).clamp(0.0, 1.0) * (d + width + 0.5).clamp(0.0, 1.0);
        (colour, on)
    })
}

/// A vector mark inked into its box.
pub(crate) fn a_mark(at: Rectangle<i32, Physical>, shape: &AShape, ink: [u8; 3]) -> Option<Washed> {
    let (Ok(wide), Ok(tall)) = (u32::try_from(at.size.w), u32::try_from(at.size.h)) else {
        return None;
    };
    if wide == 0 || tall == 0 {
        return None;
    }
    let pixels = shape
        .coverage(wide, tall)
        .into_iter()
        .flat_map(|covered| [ink[0], ink[1], ink[2], covered])
        .collect();
    Some(Washed { area: at, pixels }.premultiplied())
}

/// How far a point is outside a `w × h` rectangle at the origin with rounded
/// corners, negative inside.
fn away(px: f32, py: f32, w: f32, h: f32, corner: f32) -> f32 {
    let corner = corner.min(w / 2.0).min(h / 2.0).max(0.0);
    let (cx, cy) = (w / 2.0, h / 2.0);
    let qx = (px - cx).abs() - (cx - corner);
    let qy = (py - cy).abs() - (cy - corner);
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - corner
}

/// A length at the display's scale.
pub(crate) fn scaled(length: i32, scale: u32) -> i32 {
    i32::try_from(i64::from(length) * i64::from(scale.max(1)) / 100).unwrap_or(i32::MAX)
}

/// A logical rectangle in the display's pixels.
pub(crate) fn in_pixels(of: Rectangle<i32, Logical>, scale: u32) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (scaled(of.loc.x, scale), scaled(of.loc.y, scale)).into(),
        (scaled(of.size.w, scale), scaled(of.size.h, scale)).into(),
    )
}

/// A scale as an `f32`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a display's scale and a few hundred units at it are well inside an f32"
)]
pub(crate) fn as_f32(it: f64) -> f32 {
    it as f32
}

/// A pixel count as an `f32`.
#[expect(
    clippy::cast_precision_loss,
    reason = "a position on a display is far inside an f32's exact range"
)]
fn as_f32_i(it: i32) -> f32 {
    it as f32
}

/// A length in pixels, rounded up.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a shadow's offset in pixels is well inside an i32"
)]
fn as_i32(it: f32) -> i32 {
    it.ceil() as i32
}

#[cfg(test)]
#[path = "window_menu_paint_tests.rs"]
mod tests;
