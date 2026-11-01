//! One window edge, laid out in owned pixels and ready for a frame.
//!
//! `crate::window_edge` says where everything is, `crate::window_edge_paint`
//! says what colour, and `crate::window_edge_title` says what text. This is
//! the three of them turned into the pixels a frame takes — the same shape
//! every other native surface in this crate has, so the edge paints the way
//! the sign-in screen and the egress indicator already do rather than
//! inventing a second road into a frame.
//!
//! # Logical units become output pixels exactly once
//!
//! `window_edge` works in logical units, which is what the design is drawn in
//! and what a person's scale setting multiplies. `crate::painted` works in
//! output pixels, which is what a frame takes. **The conversion happens here
//! and nowhere else**, which is the contract's *apply display scaling once*.
//!
//! # What it refuses
//!
//! A picture whose region falls outside the output is not drawn at all. The
//! edge sits above its window, so a window at the very top of a display has an
//! edge partly or wholly off-screen — and a half-drawn edge is worse than
//! none, because a person would reach for a control that is not where it
//! looks.

use smithay::{
    backend::renderer::Frame,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Physical, Rectangle},
};

use crate::painted::{self, Inked, Solid, Washed};
use crate::window_edge::WindowEdge;
use crate::window_edge_look::WhichLook;
use crate::window_edge_paint::Pointing;
use crate::window_edge_title::FittedTitle;
use crate::{RenderError, painted_text};

/// The window an edge belongs to, as the edge needs to know it.
///
/// **Four facts that are the window's rather than the edge's layout**, gathered
/// so `EdgePicture::of` takes one thing for them rather than four arguments a
/// call site has to keep in order.
#[derive(Debug, Clone, Copy, Default)]
pub struct TheWindow<'a> {
    /// The application's own title, never translated. [`None`] for an edge
    /// that carries none.
    pub title: Option<&'a str>,
    /// Whether this is the window the person's typing goes to.
    pub look: WhichLook,
    /// The application's icon, where this machine knows one.
    pub icon: Option<&'a alo_dock::artwork::APicture>,
    /// The window's own rectangle, in the edge's units, where alo draws the
    /// line and the shadow round it — every application's window. [`None`]
    /// for a window that draws its own body, as alo's task windows do.
    pub body: Option<Rectangle<i32, Logical>>,
}

/// What is drawn **over** one window's edge this frame.
///
/// Two things that are not the edge and belong to it: the menu it opened, and
/// the reason the whole title would be shown. They travel together because
/// they are both *this window, right now* rather than its layout, and because
/// `EdgePicture::of` had grown more arguments than anybody can read at a call
/// site — clippy said so, and it was right.
#[derive(Default)]
pub struct OverTheEdge<'a> {
    /// This window's menu, where one is open.
    pub menu: Option<(
        &'a crate::window_menu::WindowMenu,
        crate::window_menu_paint::PointingAtTheMenu,
    )>,
    /// Why the whole title would be shown, where there is a reason.
    ///
    /// A reason, not an instruction: `window_edge_tooltip::tooltip_of` refuses
    /// a title that was never cut, because one that fits is already readable.
    pub showing: Option<crate::window_edge_tooltip::Because>,
}

/// A window's edge, in output pixels.
pub struct EdgePicture {
    /// What is drawn **before** the window: the line and shadow round it, and
    /// the strip with its rounded corners.
    behind: Vec<Washed>,
    /// What is drawn after the window and before the flat shapes: its icon.
    over: Vec<Washed>,
    /// The window this edge is on, where the caller said; the painter draws
    /// each window's edge with that window rather than all of them above all
    /// of them.
    window: Option<WlSurface>,
    /// The flat shapes: the grip, any highlight and focus outline.
    solids: Vec<Solid>,
    /// The title, where there is one and it fits.
    inked: Vec<Inked>,
    /// The whole title, for whoever says it aloud, whatever was drawn.
    whole: String,
}

impl EdgePicture {
    /// Lay this edge out for an output of `output` pixels, at `scale`
    /// hundredths.
    ///
    /// `window` says what the edge needs to know of its window: the
    /// application's own title, never translated here and [`None`] for an
    /// edge that carries none; which look it wears; its icon; and its body,
    /// where alo draws the line and shadow round it.
    ///
    /// Returns [`None`] when the edge would not be wholly on the output.
    #[must_use]
    /// `menu` is this window's menu where one is open, and nothing where
    /// none is. **It belongs to this picture rather than to one of its
    /// own**: it is this window's menu, it is drawn over this window's
    /// edge, and two pictures would be two things a caller had to keep in
    /// the same order to get the stacking right.
    pub fn of(
        edge: &WindowEdge,
        pointing: Pointing,
        over: OverTheEdge<'_>,
        window: TheWindow<'_>,
        fonts: &mut cosmic_text::FontSystem,
        output: (i32, i32),
        scale: u32,
    ) -> Option<Self> {
        let title = window.title;
        let whole_output = Rectangle::from_size((output.0, output.1).into());
        let region = into_pixels(edge.region, scale);
        if whole_output.intersection(region) != Some(region) {
            return None;
        }
        let as_a_fraction = f64::from(scale.max(1)) / 100.0;
        let colours = window.look.colours();

        // **Behind the window: its line and shadow, then the strip.** The
        // strip after the shadow, so a focused window's shadow never falls
        // across the strip above it.
        let mut behind = Vec::new();
        if let Some(body) = window.body {
            behind.extend(crate::a_window_raised::round_the_window(
                into_pixels(body, scale),
                as_a_fraction,
                window.look,
                output,
            ));
        }
        if let Some(strip) = edge.strip {
            behind.extend(crate::the_title_strip_drawn::the_strip(
                into_pixels(strip, scale),
                as_a_fraction,
                window.look,
                output,
            ));
        }
        // **The application's own icon**, scaled to the 16 × 16 the design
        // gives it at this display's scale, so a dense display draws it
        // sharper rather than smaller.
        let mut drawn_icon = Vec::new();
        if let (Some(at), Some(icon)) = (edge.icon, window.icon) {
            let area = into_pixels(at, scale);
            if let (Ok(across), Ok(down)) = (u32::try_from(area.size.w), u32::try_from(area.size.h))
                && across > 0
                && down > 0
            {
                drawn_icon.push(Washed {
                    area,
                    pixels: icon.scaled_to(across, down).pixels().to_vec(),
                });
            }
        }

        // **The menu after the edge**, because it is drawn over it: these
        // are filled in order and the last one wins where they overlap.
        let mut solids: Vec<Solid> = crate::window_edge_paint::solids(edge, pointing, window.look)
            .into_iter()
            // **No icon to show, the grip's mark instead**, so the Move target
            // is never an empty square a person cannot tell is there.
            .chain(if edge.icon.is_some() && drawn_icon.is_empty() {
                crate::window_edge_paint::the_grip_mark(edge)
            } else {
                Vec::new()
            })
            .chain(
                over.menu
                    .into_iter()
                    .flat_map(|(menu, at)| crate::window_menu_paint::solids(menu, at)),
            )
            .map(|(area, colour)| Solid {
                area: into_pixels(area, scale),
                colour: [colour.red(), colour.green(), colour.blue()],
            })
            .collect();

        // The title, fitted to what the drag region leaves after the grip and
        // then inked. `fitted` is what decides how much of it is drawn; this
        // only places what came back.
        let mut inked = Vec::new();
        let mut whole = String::new();
        if let (Some(title), Some(at)) = (title, edge.title) {
            // Asked of the edge rather than recomputed: the title no longer
            // begins at the drag region's leading edge, so the old
            // `drag.size.w - THE_GRIP_IS.0` over-estimated it by the Move
            // target and let a long title reach the controls.
            let room = edge.the_room_a_title_has();
            let metrics = cosmic_text::Metrics::new(THE_LABEL_IS, THE_LINE_IS);
            let was_fitted = crate::window_edge_title::fitted(fonts, title, room, metrics);
            let FittedTitle {
                ref shown,
                whole: ref all,
                ..
            } = was_fitted;
            whole = all.clone();
            if !shown.is_empty() {
                // **On the strip's own ground, in the look's ink**: navy on the
                // focused window's white, muted on every other's cream.
                let shaped = painted_text::sentence(
                    fonts,
                    shown,
                    scaled(room, scale),
                    metrics,
                    colours.strip,
                    colours.ink,
                );
                let placed =
                    into_pixels(Rectangle::new(at, (room, THE_LINE_IS as i32).into()), scale);
                inked.extend(shaped.placed(placed.loc.x, placed.loc.y, output.1));
            }
            // **The whole title, for somebody who can see it.** Asked only
            // where a reason was given and the title was actually cut -
            // `tooltip_of` answers `None` for a title that fits, because one
            // that fits is already readable.
            //
            // Wired here rather than left exported: `tooltip_of` had no
            // caller at all until 2026-10-12, which is a tooltip nobody could
            // ever see.
            if let Some(because) = over.showing
                && let Some(shown) = crate::window_edge_tooltip::tooltip_of(
                    edge,
                    &was_fitted,
                    because,
                    fonts,
                    output,
                )
            {
                let ground = alo_appearance::Role::BgSurface.colour();
                let ink = alo_appearance::Role::TextPrimary.colour();
                let edging = alo_appearance::Role::BorderDefault.colour();
                solids.push(Solid {
                    area: into_pixels(shown.panel, scale),
                    colour: [edging.red(), edging.green(), edging.blue()],
                });
                if let Some(inside) = shrunk(shown.panel) {
                    solids.push(Solid {
                        area: into_pixels(inside, scale),
                        colour: [ground.red(), ground.green(), ground.blue()],
                    });
                }
                let metrics = cosmic_text::Metrics::new(THE_LABEL_IS, THE_LINE_IS);
                for (line, at) in &shown.lines {
                    let shaped = painted_text::sentence(
                        fonts,
                        line,
                        scaled(crate::window_edge_tooltip::AT_MOST_WIDE, scale),
                        metrics,
                        [ground.red(), ground.green(), ground.blue()],
                        [ink.red(), ink.green(), ink.blue()],
                    );
                    let placed = into_pixels(
                        Rectangle::new(
                            *at,
                            (crate::window_edge_tooltip::AT_MOST_WIDE, THE_LINE_IS as i32).into(),
                        ),
                        scale,
                    );
                    inked.extend(shaped.placed(placed.loc.x, placed.loc.y, output.1));
                }
            }
        }
        Some(Self {
            behind,
            over: drawn_icon,
            window: None,
            solids,
            inked,
            whole,
        })
    }

    /// The same picture, said to be the edge of `surface`.
    ///
    /// **So the painter can draw it with its own window**: a window in front
    /// then covers the edge of the one behind it, as it covers the rest of it,
    /// rather than every edge being drawn over every window.
    #[must_use]
    pub(crate) fn on_the_window(mut self, surface: WlSurface) -> Self {
        self.window = Some(surface);
        self
    }

    /// The window this edge is on, where it was said.
    pub(crate) const fn the_window(&self) -> Option<&WlSurface> {
        self.window.as_ref()
    }

    /// What is drawn before the window: its line, its shadow and its strip.
    pub(crate) fn behind(&self) -> &[Washed] {
        &self.behind
    }

    /// What is drawn after the window, before the flat shapes: its icon.
    pub(crate) fn over(&self) -> &[Washed] {
        &self.over
    }

    /// The whole title, however little of it was drawn.
    ///
    /// **What a reader is given.** The contract: *preserve the full title for
    /// accessibility.* Anything announcing the drawn string would make a
    /// long-titled window unfindable by name.
    #[must_use]
    pub fn the_whole_title(&self) -> &str {
        &self.whole
    }

    /// Draw it: the shapes, then the text, in that order.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        painted::paint(frame, &self.solids, &self.inked)
    }
}

/// The design's `Label/Medium` size.
const THE_LABEL_IS: f32 = 13.0;

/// Its line height.
const THE_LINE_IS: f32 = 18.0;

/// One logical length in output pixels, at `scale` hundredths.
fn scaled(length: i32, scale: u32) -> i32 {
    let scale = i64::from(scale.max(1));
    i32::try_from(i64::from(length) * scale / 100).unwrap_or(i32::MAX)
}

/// One logical rectangle in output pixels.
///
/// **The only place the conversion happens**, so drawing and hit testing
/// cannot round differently — the contract asks for one authoritative
/// calculation and this is where its units change.
fn into_pixels(of: Rectangle<i32, Logical>, scale: u32) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (scaled(of.loc.x, scale), scaled(of.loc.y, scale)).into(),
        (scaled(of.size.w, scale), scaled(of.size.h, scale)).into(),
    )
}

#[cfg(test)]
#[path = "window_edge_picture_tests.rs"]
mod tests;

/// A rectangle shrunk by one unit on every side, for a panel's border.
fn shrunk(of: Rectangle<i32, Logical>) -> Option<Rectangle<i32, Logical>> {
    let (w, h) = (of.size.w - 2, of.size.h - 2);
    (w > 0 && h > 0).then(|| Rectangle::new((of.loc.x + 1, of.loc.y + 1).into(), (w, h).into()))
}
