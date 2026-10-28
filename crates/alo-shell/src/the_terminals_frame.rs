//! The terminal window's own body: a white panel with a hairline round it.
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
//! | its thickness | `alo_terminal::measures::A_BORDER`, 1 | through the display's scale |
//!
//! and the render of the same node shows the transcript on the panel's own
//! white, not on a ground of its own. `docs/design/the-terminal-window.md`
//! records the measurement.
//!
//! # What the design also draws, and this does not — yet
//!
//! **The corners and the shadow.** The same node carries `radius/lg` 18 and
//! the `Elevation/Floating` shadow. Both are partial coverage, which a
//! [`Solid`] cannot be, and the one rasteriser that draws them —
//! `crate::dock_surface::a_panel` — is fixed to the Dock's own shadow and takes
//! no display scale. So the body is square-cornered and flat until that
//! rasteriser can draw this panel at this display's scale; this is a named gap
//! and not a choice of look.
//!
//! # The dark appearance is not measured
//!
//! The design draws light and dark pairs of the terminal and only the light one
//! has been read. The dark ground and ink are the ones this terminal already
//! drew, and the hairline is `border/default` in both, as
//! `crate::desktop_look::DesktopPalette::edge` already draws it round every
//! other surface.

use alo_appearance::{Role, Scheme, Token};
use alo_terminal::measures;
use smithay::utils::{Physical, Rectangle};

use crate::painted::Solid;

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

/// How thick the hairline is on a display at this scale, in whole pixels.
///
/// **Never thinner than one pixel**, because a hairline rounded to nothing on a
/// display scaled below one is a window with no edge — the fault this file
/// exists to end.
fn how_thick_the_hairline_is(scale: f64) -> i32 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a one-unit logical measure through a display's scale is a handful of pixels"
    )]
    let thick = (f64::from(measures::A_BORDER) * scale).round() as i32;
    thick.max(1)
}

/// The window's body, as flat shapes painted back to front: the hairline, then
/// the panel inside it.
///
/// **Two rectangles, not a stroke**, which is how `crate::window_edge_paint`
/// draws the hairline round an edge: the whole window in the hairline's colour,
/// then the panel one hairline in from every side.
///
/// A window too small to have anything inside its hairline is drawn as the
/// hairline alone rather than as a panel that overruns it, and a window with no
/// area is drawn as nothing.
pub(crate) fn the_body(
    window: Rectangle<i32, Physical>,
    scale: f64,
    colours: TheTerminalsColours,
) -> Vec<Solid> {
    if window.size.w <= 0 || window.size.h <= 0 {
        return Vec::new();
    }
    let thick = how_thick_the_hairline_is(scale);
    let mut body = vec![Solid {
        area: window,
        colour: colours.hairline,
    }];
    let (inside_w, inside_h) = (window.size.w - thick * 2, window.size.h - thick * 2);
    if inside_w > 0 && inside_h > 0 {
        body.push(Solid {
            area: Rectangle::new(
                (window.loc.x + thick, window.loc.y + thick).into(),
                (inside_w, inside_h).into(),
            ),
            colour: colours.ground,
        });
    }
    body
}

#[cfg(test)]
#[path = "the_terminals_frame_tests.rs"]
mod tests;
