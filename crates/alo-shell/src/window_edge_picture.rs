//! One window's band, laid out in owned pixels and ready for a frame.
//!
//! Three layers, because they are drawn at three moments
//! (`crate::each_window_with_its_edge`):
//!
//! | layer | what | when |
//! |---|---|---|
//! | behind | the line and shadow round the window, the band | before the window |
//! | over | the three controls, faded as far as they have got | after the window |
//! | on top | an open window panel | after **every** window |
//!
//! And the words: the title and the application's name on the band, a tooltip,
//! and the panel's words, which go with the panel.
//!
//! # The title and the application's name
//!
//! `docs/design/the-window-controls.md`: **The next chapter · Notes** — the
//! title in SemiBold 14, then 24 units, then the application's name in Regular
//! 11, quieter. Both are always drawn. The title is cut with an ellipsis to fit
//! the room [`WindowEdge::title_room`] leaves, which is the same whether or not
//! the controls are shown, so **revealing them never moves the title**. Where
//! the window's title *is* its application's name, it is drawn once.

use smithay::{
    backend::renderer::Frame,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Physical, Rectangle},
};

use crate::painted::{self, Inked, Solid, Washed};
use crate::window_edge::{BEFORE_THE_APPLICATIONS_NAME, THE_APPLICATIONS_NAME_DROPS, WindowEdge};
use crate::window_edge_look::WhichLook;
use crate::window_edge_paint::HowTheControlsLook;
use crate::window_edge_title::FittedTitle;
use crate::{RenderError, painted_text};

/// What only the window knows: its name, its application's, how it looks.
#[derive(Debug, Clone, Copy, Default)]
pub struct TheWindow<'a> {
    /// What the application called this window, if it said.
    pub title: Option<&'a str>,
    /// What this machine calls the application, if it said — the same name
    /// the Dock shows for it.
    pub application: Option<&'a str>,
    /// The typing window's look, or every other's.
    pub look: WhichLook,
    /// The window's own rectangle, for the line and shadow round it.
    pub body: Option<Rectangle<i32, Logical>>,
    /// The band is out over a full-screen window, so its ground is drawn
    /// **over** the window rather than under it
    /// (`crate::the_title_strip_drawn::the_strip_over_full_screen`).
    pub laid_over: bool,
    /// How its controls look now.
    pub controls: HowTheControlsLook,
}

/// What may be drawn over the band, which the host decides.
#[derive(Default)]
pub struct OverTheEdge<'a> {
    /// This window's open panel, and what is pointed at in it.
    pub menu: Option<(
        &'a crate::window_menu::WindowMenu,
        crate::window_menu_paint::PointingAtTheMenu,
    )>,
    /// What is typed in that panel's field, *alo · Ask about this window…*,
    /// once anything is (`crate::window_field_paint`).
    pub field: Option<crate::window_field_paint::TheFieldAsTyped<'a>>,
    /// Why the whole title, or a control's name, is being shown.
    pub showing: Option<crate::window_edge_tooltip::Because>,
    /// alo, while it is at work in this window: the teal edge and the band
    /// with Stop (`crate::alos_band_in_a_window`).
    pub alo: Option<crate::alos_band_in_a_window::AlosBand<'a>>,
}

/// One window's band, drawn.
pub struct EdgePicture {
    /// Under the window.
    behind: Vec<Washed>,
    /// Over the window.
    over: Vec<Washed>,
    /// Over every window.
    on_top: Vec<Washed>,
    /// The words over every window.
    on_top_inked: Vec<Inked>,
    /// Which window this is, once the caller says.
    window: Option<WlSurface>,
    /// Flat fills over the window: a tooltip's.
    solids: Vec<Solid>,
    /// Words over the window.
    inked: Vec<Inked>,
    /// The whole title, for whoever says it aloud.
    whole: String,
}

/// The title's type: SemiBold 14 on 19. `11:32`.
const THE_TITLE_IS: (f32, f32) = (14.0, 19.0);

/// The application's name: Regular 11 on 15. `11:33`.
const THE_APPLICATIONS_NAME_IS: (f32, f32) = (11.0, 15.0);

/// The smallest a title is drawn, in pixels, below which it is left out
/// rather than drawn as a smear.
pub(crate) const THE_SMALLEST_TITLE_IS: f32 = 8.0;

impl EdgePicture {
    /// This band, drawn, or [`None`] where its region is not wholly on the
    /// display — a band half off the screen is not drawn half.
    #[must_use]
    pub fn of(
        edge: &WindowEdge,
        over: OverTheEdge<'_>,
        window: TheWindow<'_>,
        fonts: &mut cosmic_text::FontSystem,
        output: (i32, i32),
        scale: u32,
    ) -> Option<Self> {
        let whole_output = Rectangle::from_size((output.0, output.1).into());
        let region = into_pixels(edge.region, scale);
        if whole_output.intersection(region) != Some(region) {
            return None;
        }
        let as_a_fraction = f64::from(scale.max(1)) / 100.0;
        let colours = window.look.colours();

        let mut behind = Vec::new();
        if let Some(body) = window.body {
            behind.extend(crate::a_window_raised::round_the_window(
                into_pixels(body, scale),
                as_a_fraction,
                window.look,
                output,
            ));
        }
        let mut over_the_window = Vec::new();
        if window.laid_over {
            over_the_window.extend(crate::the_title_strip_drawn::the_strip_over_full_screen(
                into_pixels(edge.strip, scale),
                as_a_fraction,
                window.look,
                output,
            ));
        } else {
            behind.extend(crate::the_title_strip_drawn::the_strip(
                into_pixels(edge.strip, scale),
                as_a_fraction,
                window.look,
                output,
            ));
        }
        over_the_window.extend(crate::window_edge_paint::the_controls_drawn(
            edge,
            &window.controls,
            scale,
        ));

        let mut inked = Vec::new();
        let mut solids = Vec::new();
        let mut whole = String::new();
        if let Some(at) = edge.title {
            let room = edge.the_room_a_title_has();
            let (title, application) = what_the_band_says(window.title, window.application);
            let fraction = as_f32(as_a_fraction);
            let title_metrics = cosmic_text::Metrics::new(THE_TITLE_IS.0, THE_TITLE_IS.1);
            let name_metrics =
                cosmic_text::Metrics::new(THE_APPLICATIONS_NAME_IS.0, THE_APPLICATIONS_NAME_IS.1);
            let regular = painted_text::alos_own_text().weight(cosmic_text::Weight(400));
            // **The application's name keeps its room and the title gives way**,
            // because it is short and says whose window this is; with no room
            // for both, the name goes before the title is cut to nothing.
            let name_wide = application.map_or(0, |name| {
                painted_text::how_wide_as(fonts, name, name_metrics, &regular)
            });
            let title_room = if application.is_some() && title.is_some() {
                room - BEFORE_THE_APPLICATIONS_NAME - name_wide
            } else {
                room
            };
            let (title_room, application) = if title.is_some() && title_room < name_wide {
                (room, None)
            } else {
                (title_room, application)
            };
            let fitted = title.map(|title| {
                crate::window_edge_title::fitted(fonts, title, title_room.max(0), title_metrics)
            });
            whole = fitted
                .as_ref()
                .map_or_else(String::new, |it| it.whole.clone());
            let mut title_ends = at.x;
            if let Some(FittedTitle { shown, .. }) = fitted.as_ref()
                && !shown.is_empty()
                && THE_TITLE_IS.0 * fraction >= THE_SMALLEST_TITLE_IS
            {
                let shaped = painted_text::sentence(
                    fonts,
                    shown,
                    scaled(title_room.max(0), scale),
                    cosmic_text::Metrics::new(THE_TITLE_IS.0 * fraction, THE_TITLE_IS.1 * fraction),
                    colours.strip,
                    colours.ink,
                )
                .cropped_to_its_ink();
                title_ends = at.x + painted_text::how_wide(fonts, shown, title_metrics);
                let placed = into_pixels(Rectangle::new(at, (title_room, 0).into()), scale);
                inked.extend(shaped.placed(placed.loc.x, placed.loc.y, output.1));
            }
            if let Some(name) = application
                && THE_APPLICATIONS_NAME_IS.0 * fraction >= THE_SMALLEST_TITLE_IS
            {
                let from = if title.is_some() {
                    title_ends + BEFORE_THE_APPLICATIONS_NAME
                } else {
                    at.x
                };
                let quiet = alo_appearance::Role::TextMuted.colour();
                let shaped = painted_text::sentence_in_runs(
                    fonts,
                    &[(name, regular.clone())],
                    &regular,
                    scaled(name_wide.max(1), scale),
                    cosmic_text::Metrics::new(
                        THE_APPLICATIONS_NAME_IS.0 * fraction,
                        THE_APPLICATIONS_NAME_IS.1 * fraction,
                    ),
                    colours.strip,
                    [quiet.red(), quiet.green(), quiet.blue()],
                )
                .cropped_to_its_ink();
                inked.extend(shaped.placed(
                    scaled(from, scale),
                    scaled(at.y + THE_APPLICATIONS_NAME_DROPS, scale),
                    output.1,
                ));
            }
            // **A tooltip: a control's name while it is pointed at or the
            // keyboard is on it, or else the whole of a cut title.**
            let tooltip = over.showing.and_then(|because| {
                let control = window
                    .controls
                    .pointing
                    .hovered
                    .or(window.controls.pointing.focused);
                match control {
                    Some(does) => crate::window_edge_tooltip::the_name_of_a_control(
                        edge,
                        does,
                        crate::window_edge_reading::what_it_is_called(does).says(),
                        because,
                        fonts,
                        output,
                    ),
                    None => fitted.as_ref().and_then(|fitted| {
                        crate::window_edge_tooltip::tooltip_of(edge, fitted, because, fonts, output)
                    }),
                }
            });
            if let Some(shown) = tooltip {
                draw_a_tooltip(&shown, fonts, scale, output, &mut solids, &mut inked);
            }
        }

        // **alo's presence, over the window and under any panel**: drawn after
        // the controls, so a panel opened on this window still covers it.
        if let Some(alo) = over.alo {
            let (rasters, words) = crate::alos_band_in_a_window::drawn(alo, fonts, scale, output);
            over_the_window.extend(rasters);
            inked.extend(words);
        }

        let (on_top, on_top_inked) = over.menu.map_or_else(
            || (Vec::new(), Vec::new()),
            |(menu, pointing)| {
                crate::window_menu_paint::the_panel_drawn(
                    menu, pointing, over.field, fonts, scale, output,
                )
            },
        );

        Some(Self {
            behind,
            over: over_the_window,
            on_top,
            on_top_inked,
            window: None,
            solids,
            inked,
            whole,
        })
    }

    /// The same picture, said to belong to this window.
    #[must_use]
    pub(crate) fn on_the_window(mut self, surface: WlSurface) -> Self {
        self.window = Some(surface);
        self
    }

    /// Which window this belongs to, if the caller said.
    pub(crate) const fn the_window(&self) -> Option<&WlSurface> {
        self.window.as_ref()
    }

    /// What is drawn under the window.
    pub(crate) fn behind(&self) -> &[Washed] {
        &self.behind
    }

    /// What is drawn over the window.
    pub(crate) fn over(&self) -> &[Washed] {
        &self.over
    }

    /// What is drawn over every window.
    pub(crate) fn on_top(&self) -> &[Washed] {
        &self.on_top
    }

    /// The whole title, untruncated, for whoever says it aloud.
    #[must_use]
    pub fn the_whole_title(&self) -> &str {
        &self.whole
    }

    /// The flat fills and the words, for a test that reads what was drawn.
    #[cfg(test)]
    pub(crate) fn flat(&self) -> (&[Solid], &[Inked]) {
        (&self.solids, &self.inked)
    }

    /// The panel's words, for a test that reads them.
    #[cfg(test)]
    pub(crate) fn on_top_words(&self) -> &[Inked] {
        &self.on_top_inked
    }

    /// Draw the fills and the words that go over the window.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        painted::paint(frame, &self.solids, &self.inked)
    }

    /// Draw the words that go over every window: the panel's.
    pub(crate) fn paint_on_top(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        painted::paint(frame, &[], &self.on_top_inked)
    }
}

/// What the band says: the title and the application's name, **never the
/// same word twice** — where the application named its window nothing, or
/// named it its own name, the name alone.
#[must_use]
pub(crate) fn what_the_band_says<'a>(
    title: Option<&'a str>,
    application: Option<&'a str>,
) -> (Option<&'a str>, Option<&'a str>) {
    let title = title.filter(|it| !it.trim().is_empty());
    let application = application.filter(|it| !it.trim().is_empty());
    match (title, application) {
        (Some(title), Some(name)) => match without_its_applications_name(title, name) {
            Some(shorter) => (Some(shorter), Some(name)),
            None => (None, Some(name)),
        },
        (title, application) => (title, application),
    }
}

/// The separators an application puts between its window's title and its own
/// name: KDE's em dash (*Home — Dolphin*), an en dash, a hyphen, a bar, a dot.
const BEFORE_AN_APPLICATIONS_NAME: [&str; 5] = [" — ", " – ", " - ", " | ", " · "];

/// A title with its application's name taken off its end, where the
/// application put it there — or [`None`] where nothing is left, because the
/// title *was* the name.
///
/// **The quiet name stays and the title's copy goes**, so a Dolphin window
/// reads *Home · Dolphin*, which is the design's own shape (*The next chapter ·
/// Notes*), rather than *Home — Dolphin · Dolphin*, which the boot walk of
/// 2026-11-09 showed. Not the other way round, because the quiet name is the
/// one place every band says whose window it is, in the same words the Dock
/// uses; a title that ends in the name is one application's habit.
///
/// Only an exact name after one of [`BEFORE_AN_APPLICATIONS_NAME`]: a title
/// that merely mentions the application keeps every word.
fn without_its_applications_name<'a>(title: &'a str, name: &str) -> Option<&'a str> {
    let (title, name) = (title.trim(), name.trim());
    if title == name {
        return None;
    }
    let shorter = BEFORE_AN_APPLICATIONS_NAME.iter().find_map(|between| {
        title.strip_suffix(name).and_then(|before| {
            before
                .strip_suffix(between)
                .or_else(|| (before.trim() == between.trim()).then_some(""))
        })
    });
    match shorter.map(str::trim) {
        Some("") => None,
        Some(shorter) => Some(shorter),
        None => Some(title),
    }
}

/// A tooltip's panel and words.
fn draw_a_tooltip(
    shown: &crate::window_edge_tooltip::Tooltip,
    fonts: &mut cosmic_text::FontSystem,
    scale: u32,
    output: (i32, i32),
    solids: &mut Vec<Solid>,
    inked: &mut Vec<Inked>,
) {
    let ground = alo_appearance::Role::BgSurface.colour();
    let ink = alo_appearance::Role::TextPrimary.colour();
    let edging = alo_appearance::Role::BorderDefault.colour();
    solids.push(Solid {
        area: crate::painted::drawn_at(into_pixels(shown.panel, scale)),
        colour: [edging.red(), edging.green(), edging.blue()],
    });
    if let Some(inside) = shrunk(shown.panel) {
        solids.push(Solid {
            area: crate::painted::drawn_at(into_pixels(inside, scale)),
            colour: [ground.red(), ground.green(), ground.blue()],
        });
    }
    let fraction = as_f32(f64::from(scale.max(1)) / 100.0);
    let metrics = cosmic_text::Metrics::new(12.0 * fraction, 16.0 * fraction);
    for (line, at) in &shown.lines {
        let shaped = painted_text::sentence(
            fonts,
            line,
            scaled(crate::window_edge_tooltip::AT_MOST_WIDE, scale),
            metrics,
            [ground.red(), ground.green(), ground.blue()],
            [ink.red(), ink.green(), ink.blue()],
        )
        .cropped_to_its_ink();
        let placed = into_pixels(Rectangle::new(*at, (1, 1).into()), scale);
        inked.extend(shaped.placed(placed.loc.x, placed.loc.y, output.1));
    }
}

/// A scale as an `f32`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a scale of a few hundredths to a few hundred per cent is well inside an f32"
)]
fn as_f32(it: f64) -> f32 {
    it as f32
}

/// A logical length in the display's pixels.
fn scaled(length: i32, scale: u32) -> i32 {
    let scale = i64::from(scale.max(1));
    i32::try_from(i64::from(length) * scale / 100).unwrap_or(i32::MAX)
}

/// A logical rectangle in the display's pixels, scaled once.
fn into_pixels(of: Rectangle<i32, Logical>, scale: u32) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (scaled(of.loc.x, scale), scaled(of.loc.y, scale)).into(),
        (scaled(of.size.w, scale), scaled(of.size.h, scale)).into(),
    )
}

/// A rectangle one unit in on every side, for a tooltip's line.
fn shrunk(of: Rectangle<i32, Logical>) -> Option<Rectangle<i32, Logical>> {
    let (w, h) = (of.size.w - 2, of.size.h - 2);
    (w > 0 && h > 0).then(|| Rectangle::new((of.loc.x + 1, of.loc.y + 1).into(), (w, h).into()))
}

#[cfg(test)]
#[path = "window_edge_picture_tests.rs"]
mod tests;
