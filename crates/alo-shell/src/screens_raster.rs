//! The desk: one picture per screen, each laid out for its own room, wearing
//! its own background, with its own dock on its own edge, and warmed by its own
//! night light.
//!
//! # One picture per screen, and nothing stretched across them
//!
//! Every screen is laid out on its own, at its own size, from
//! `crate::screens::ScreenPlace`. A dock laid out once for the desk and cut
//! into pieces is the bug that gives a 4K monitor the dock a 1366-wide laptop
//! needed; `alo_dock::Dock::layout_on` is asked per screen for exactly that
//! reason, and so is the background.
//!
//! # The edge comes from what the screen wears
//!
//! `alo_dock::Dock` holds **one** edge for the machine today, so every screen's
//! dock is on the same edge — but the edge each picture is drawn on is read
//! from that screen's `alo_displays::Wearing`, which is the one place a
//! per-screen edge will appear when `alo-dock` decides one. Nothing here picks
//! an edge; a screen whose wearing names a different edge gets a dock on it
//! without another line changing.
//!
//! # Warming is applied to what is drawn, not decided here
//!
//! The background's pixels and the dock's own colours are multiplied by the
//! `alo_displays::Warming` that screen wears, through the same
//! `Warming::applied_to` every other warmed colour in the shell goes through.
//! At neutral warmth — which is a machine with night light off — that is the
//! identity, and a screen is drawn exactly as it was.

use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_appearance::{Colour, DisplayId};
use alo_displays::{Position, Warming};
use alo_dock::Dock;
use smithay::backend::renderer::Frame;

use crate::RenderError;
use crate::desktop_look::DesktopLook;
use crate::dock_raster::DockPicture;
use crate::screen_background::ScreenBackground;
use crate::screens::{ScreenPlace, Screens};

/// Where the images this machine shipped are installed.
fn shipped_wallpapers() -> PathBuf {
    PathBuf::from("/usr/share/alo/wallpapers")
}

/// One screen, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenPicture {
    /// The name the screen is known by.
    pub(crate) name: DisplayId,
    /// Its corner on the desk.
    pub(crate) at: Position,
    /// The room it was laid out for.
    pub(crate) size: (i32, i32),
    /// What is behind its windows.
    pub(crate) background: ScreenBackground,
    /// Its dock, on its own edge, laid out for its own room and already warmed.
    pub(crate) dock: DockPicture,
}

impl ScreenPicture {
    /// The name of the screen this picture is for.
    #[must_use]
    pub const fn name(&self) -> &DisplayId {
        &self.name
    }

    /// Its corner on the desk, as the arrangement places it.
    #[must_use]
    pub const fn at(&self) -> Position {
        self.at
    }

    /// The room it was laid out for.
    #[must_use]
    pub const fn size(&self) -> (i32, i32) {
        self.size
    }

    /// Paint this screen: its background first, then its dock over it.
    ///
    /// # Errors
    /// Every refusal the painter makes for a shape outside the frame.
    pub fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.background.solids, &self.background.inked)?;
        crate::painted::paint(frame, &self.dock.solids, &[])
    }
}

/// Lay one screen out and rasterise it.
///
/// # Errors
/// Every refusal `crate::dock_raster` makes for this screen's room, every
/// refusal `crate::screen_background` makes for its chosen background, and
/// [`RenderError::AccentRefused`] for a look whose accent is not one a person
/// can choose.
pub(crate) fn picture(
    place: &ScreenPlace,
    dock: &Dock,
    look: DesktopLook,
    running: Duration,
    shipped: &Path,
) -> Result<ScreenPicture, RenderError> {
    let mut on_this_screen = dock.clone();
    on_this_screen.set_edge(place.edge());
    let mut drawn = crate::dock_raster::picture(&on_this_screen, look, place.room())?;
    let warming = place.warming();
    for solid in &mut drawn.solids {
        solid.colour = warm(solid.colour, warming);
    }
    let background = ScreenBackground::prepare(
        place.wearing().background(),
        warming,
        place.room(),
        running,
        shipped,
    )?;
    Ok(ScreenPicture {
        name: place.name().clone(),
        at: place.at(),
        size: place.room(),
        background,
        dock: drawn,
    })
}

/// Lay every attached screen out, in the order the machine reported them.
///
/// # Errors
/// The first refusal any one screen makes. A desk is refused whole rather than
/// drawn with one screen missing, because a screen that silently drew nothing
/// is a screen a person would call broken.
pub fn desk(
    screens: &Screens,
    dock: &Dock,
    look: DesktopLook,
    running: Duration,
) -> Result<Vec<ScreenPicture>, RenderError> {
    within(screens, dock, look, running, &shipped_wallpapers())
}

/// Every screen, resolving shipped images beneath an explicit root.
pub(crate) fn within(
    screens: &Screens,
    dock: &Dock,
    look: DesktopLook,
    running: Duration,
    shipped: &Path,
) -> Result<Vec<ScreenPicture>, RenderError> {
    screens
        .each()
        .map(|place| picture(place, dock, look, running, shipped))
        .collect()
}

/// One painted colour as this screen's night light shows it.
fn warm(colour: [u8; 3], warming: Warming) -> [u8; 3] {
    let [red, green, blue] = colour;
    let shown = warming.applied_to(Colour::of(red, green, blue));
    [shown.red(), shown.green(), shown.blue()]
}

#[cfg(test)]
#[path = "screens_raster_tests.rs"]
mod tests;
