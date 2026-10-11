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

use cosmic_text::FontSystem;

use alo_appearance::{Colour, DisplayId};
use alo_displays::{Position, Warming};
use alo_dock::Dock;
use smithay::backend::renderer::Frame;

use crate::RenderError;
use crate::desktop_look::DesktopLook;
use crate::dock_raster::DockPicture;
use crate::screen_background::ScreenBackground;
use crate::screens::{ScreenPlace, Screens};

/// One screen, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenPicture {
    /// The name the screen is known by.
    pub(crate) name: DisplayId,
    /// Its corner on the desk.
    pub(crate) at: Position,
    /// The room it was laid out for.
    pub(crate) size: crate::TheRoom,
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
    pub const fn size(&self) -> crate::TheRoom {
        self.size
    }

    /// Paint this screen: its background first, then its dock over it.
    ///
    /// # Errors
    /// Every refusal the painter makes for a shape outside the frame.
    pub fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.background.solids, &[])?;
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
    fonts: &mut FontSystem,
) -> Result<ScreenPicture, RenderError> {
    // One dock, drawn for this screen's own size. It used to be cloned and
    // moved to this screen's edge first; ADR 0076 fixed the dock to the bottom
    // edge of every screen, so there is nothing to override and no copy to make.
    // `across_and_along` rather than a silent conversion: `dock_raster` still
    // takes a pair, and this is the point at which the value stops saying what
    // it is. Three such points exist in this function and each says so.
    // **No applications on a screen's own picture, and that is this picture's
    // subject rather than an omission.** A `ScreenPlace` knows a display's size,
    // scale and where it sits on the desk; it does not know what is open, and a
    // preview of *which screen is which* answers where the dock is rather than
    // what is in it. The empty slice draws the band alone, which is what this
    // drew when the argument was a count of `0`.
    let mut drawn = crate::dock_raster::picture(
        dock,
        look,
        place.room().across_and_along(),
        &[],
        // **And nothing in *More apps* either**, for the same reason: this
        // picture's subject is where the screens are, not what is on them. The
        // list is closed below in any case, so an overview here would be built
        // and thrown away.
        &[],
        // **A preview of an arrangement, where nothing is open and so
        // nothing can have overflowed.** Not a placeholder: a Dock holding
        // nothing has no control to open a list with, and `picture` draws
        // none for an empty overflow whatever this says.
        // **A preview of an arrangement says nothing.** It is a small picture of
        // several screens, drawn to show where they are relative to each other;
        // the words on a Dock are for the Dock a person uses, and this is not
        // it. Empty rather than invented, for `crate::the_docks_words`' reason.
        (
            &crate::what_the_dock_has_open::NOTHING,
            &crate::the_docks_words::TheDocksWords::default(),
        ),
        fonts,
    )?;
    let warming = place.warming();
    for solid in &mut drawn.solids {
        solid.colour = warm(solid.colour, warming);
    }
    // **And the surface, which is where the Dock's own colour is since
    // 2026-10-11.** It was only the solids until then, and the solids were the
    // band and the accent; now they are neither, so a night light that warmed
    // only them would warm nothing a person can see. Found by
    // `night_light_warms_the_background_and_the_dock_on_every_screen`, which
    // read the Dock's ground as pure white on a warmed screen.
    //
    // **The alpha is not touched.** Warming is about the light a screen emits,
    // and a shadow's transparency is not a colour.
    if let Some(surface) = drawn.surface.as_mut() {
        let (pixels, _) = surface.pixels.as_chunks_mut::<4>();
        for pixel in pixels {
            let mut was = [0_u8; 3];
            if let Some(colour) = pixel.get(..3) {
                was.copy_from_slice(colour);
            }
            let now = warm(was, warming);
            if let Some(into) = pixel.get_mut(..3) {
                into.copy_from_slice(&now);
            }
        }
    }
    let background =
        ScreenBackground::prepare(place.wearing().background(), warming, place.room())?;
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
    fonts: &mut FontSystem,
) -> Result<Vec<ScreenPicture>, RenderError> {
    screens
        .each()
        .map(|place| picture(place, dock, look, fonts))
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
