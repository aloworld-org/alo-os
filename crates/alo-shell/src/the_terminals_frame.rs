//! The terminal window's own body: a white panel with rounded corners, a
//! hairline round it and a shadow under it.
//!
//! # What was wrong, in the words of the boot walk
//!
//! Until 2026-10-28 the terminal's text sat straight on the desktop. Its window
//! was painted in `bg/canvas` — the very colour of the canvas behind it — with
//! nothing round it, so on a booted machine there was no window to see: only
//! characters floating over the wallpaper, unlike every task window beside it.
//!
//! # What the design draws, and where each figure came from
//!
//! The body is `479:29012`, *Terminal only / 01 · Quiet light*, 1040×800.
//! Asked of the live file on 2026-10-28 for that node:
//!
//! | | token | here |
//! |---|---|---|
//! | the panel | `bg/surface` `#ffffff` | [`Role::BgSurface`] |
//! | the hairline | `border/default` `#e7ebef` | [`Role::BorderDefault`] |
//! | the ink | `text/primary` `#102a43` | [`Role::TextPrimary`] |
//! | its thickness | `alo_dock::measures::THE_DOCKS_EDGE`, 1 | through the display's scale |
//! | the corners | `radius/lg` 18 | [`HowItIsRaised::FLOATING`], through the display's scale |
//! | the shadow | `Elevation/Floating` | the same, through the display's scale |
//!
//! and the render of the same node shows the transcript on the panel's own
//! white, not on a ground of its own. `docs/design/the-terminal-window.md`
//! records the measurement.
//!
//! # Drawn by the Dock's own rasteriser, at this display's scale
//!
//! The corners and the shadow are partial coverage, which a flat rectangle
//! cannot be, so the body is a raster from `crate::dock_surface::a_panel` —
//! the one that draws the Dock — handed the floating look **at this display's
//! scale**. On a display at 200% the corner is 36 pixels and the shadow falls
//! 32, which is the same panel drawn sharper rather than a smaller one.
//!
//! # The dark appearance is not measured
//!
//! The design draws light and dark pairs of the terminal and only the light one
//! has been read. The dark ground and ink are the ones this terminal already
//! drew, and the hairline is `border/default` in both, as
//! `crate::desktop_look::DesktopPalette::edge` already draws it round every
//! other surface.

use alo_appearance::{Role, Scheme, Token};
use smithay::utils::{Physical, Rectangle};

use crate::dock_surface::{APanelsShapes, InPrimary, a_panel};
use crate::how_a_panel_is_raised::HowItIsRaised;
use crate::painted::Washed;

/// The three colours the terminal's window is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheTerminalsColours {
    /// The panel, which the transcript shares.
    pub(crate) ground: [u8; 3],
    /// What a program's text is drawn in, unless it asks for another colour.
    pub(crate) ink: [u8; 3],
    /// The hairline round the panel.
    pub(crate) hairline: [u8; 3],
}

impl TheTerminalsColours {
    /// The terminal's colours in this appearance.
    pub(crate) fn of(scheme: Scheme) -> Self {
        let (ground, ink) = match scheme {
            Scheme::Light => (Role::BgSurface.colour(), Role::TextPrimary.colour()),
            Scheme::Dark => (Token::Charcoal.colour(), Token::Cream.colour()),
        };
        let hairline = Role::BorderDefault.colour();
        Self {
            ground: [ground.red(), ground.green(), ground.blue()],
            ink: [ink.red(), ink.green(), ink.blue()],
            hairline: [hairline.red(), hairline.green(), hairline.blue()],
        }
    }
}

/// The window's body: its rounded panel, the hairline round it and the shadow
/// under it, as one raster on a display `room` big at `scale`.
///
/// **The Dock's rasteriser with the floating look**, so a change to how a
/// corner or a shadow is drawn reaches this panel and the Dock together. The
/// raster reaches past the window by the shadow's room and is kept on the
/// display, so a window near an edge loses the shadow off the screen rather
/// than the frame.
///
/// [`None`] for a window with no area, which is a window that cannot be drawn
/// rather than a failure.
pub(crate) fn the_body(
    window: Rectangle<i32, Physical>,
    room: (i32, i32),
    scale: f64,
    colours: TheTerminalsColours,
) -> Option<Washed> {
    a_panel(
        window,
        colours.ground,
        colours.hairline,
        room,
        HowItIsRaised::FLOATING.at(scale),
        APanelsShapes {
            circles: &[],
            circle: colours.ground,
            marks: &[],
            mark: colours.ground,
            wells: &[],
            well_is: colours.ground,
            rings: &[],
            ring_is: colours.ground,
            in_primary: InPrimary::NOTHING,
        },
    )
}

#[cfg(test)]
#[path = "the_terminals_frame_tests.rs"]
mod tests;
