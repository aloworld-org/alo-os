//! **One picture of the window in front, of that window alone** — the screen
//! road's picture ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §1) and `docs/contracts/application-door.md`, *`picture-of`*.
//!
//! What it does, in order, and nothing else:
//!
//! 1. asks the one question typing asks too
//!    (`crate::the_window_in_front_alo_may_reach`): the window that holds the
//!    keyboard now, if it is that application's and its process is one alo
//!    may reach at the person's setting;
//! 2. draws **that window and its own menus alone**, at its own size and
//!    without its shadow, with the painter every frame is drawn with
//!    (`crate::scene_drawing::paint`) on the processor — so nothing in front of
//!    it or behind it, no pointer and none of alo's own surfaces is in it;
//! 3. hands back a PNG of it.
//!
//! **On the processor, whatever the display draws with.** A picture is one act
//! at a time and never a stream, the renderer costs nothing to hold, and a
//! machine with no graphics card must take it just the same (ADR 0104).
//!
//! **Nothing is kept.** The pixels and the PNG belong to the one answer; no
//! picture is taken between acts and none is remembered (ADR 0147 §1, *at the
//! moment of the act, and never kept*).
//!
//! **Logical size, at zoom one.** On a display drawn at twice its size the
//! window is drawn at its logical size — the size a person reads it at — which
//! is also the smaller picture to send anywhere.

use alo_acting::{Pictured, ToPicture};
use image::ImageEncoder as _;
use smithay::utils::Size;

use crate::Server;
use crate::direct_target::ScenePainter as _;
use crate::the_window_in_front_alo_may_reach::NotReached;
use crate::typing_into_the_window_in_front::MayTypeInto;

impl Server {
    /// One picture of the window in front, if it is a window of the
    /// application `asked` names and `may` lets alo reach it — the answer, and
    /// the PNG's bytes, which are empty for every refusal.
    pub fn picture_of_the_window_in_front(
        &mut self,
        asked: &ToPicture,
        may: &mut dyn MayTypeInto,
    ) -> (Pictured, Vec<u8>) {
        let identifier = asked.application.identifier();
        let root = match self.the_window_in_front_alo_may_reach(identifier, may) {
            Ok((root, _)) => root,
            Err(NotReached::NotInFront) => return (Pictured::NotInFront, Vec::new()),
            Err(NotReached::NotAlosCopy) => return (Pictured::NotAlosCopy, Vec::new()),
        };
        // **alo's eyes are on this window**: it wears the task at work for the
        // turn, as it does when alo types into it (`crate::the_window_alo_acts_in`).
        let _ = self.alo_acts_in(&root, identifier);
        let popups = self.popup_surfaces();
        match the_picture_of(&root, &popups) {
            Some((png, wide, tall)) => (
                Pictured::Picture {
                    bytes: png.len(),
                    wide,
                    tall,
                },
                png,
            ),
            None => (Pictured::CouldNotDraw, Vec::new()),
        }
    }
}

/// `root` and its own menus drawn alone, at the window's own size and place,
/// and made a PNG: the bytes and the size, or [`None`] where any step could not
/// be done — never a picture of something else.
fn the_picture_of(
    root: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    popups: &[crate::Popup],
) -> Option<(Vec<u8>, u32, u32)> {
    // **The window as a person sees it**: its committed geometry, so a shadow
    // a client draws round itself is not in the picture.
    let window = crate::where_a_window_is(root);
    if window.size.w <= 0 || window.size.h <= 0 {
        return None;
    }
    let camera = alo_canvas::Camera::new().looking_at(alo_canvas::At {
        x: window.loc.x,
        y: window.loc.y,
    })?;
    let mut painter = crate::software_scanout::SoftwarePainter::new().ok()?;
    let (pixels, _drawn) = painter
        .paint(
            Size::from((window.size.w, window.size.h)),
            std::slice::from_ref(root),
            popups,
            &crate::Cursor::Default,
            crate::scene_native::NativeLayers::nothing(),
            camera,
        )
        .ok()?;
    let (wide, tall) = pixels.size();
    let png = a_png_of(wide, tall, pixels.pixels())?;
    (png.len() <= Pictured::THE_MOST).then_some((png, wide, tall))
}

/// A PNG of a frame `wide` by `tall` whose rows are tightly packed blue, green,
/// red and a byte nobody reads, top to bottom — [`None`] for bytes that are not
/// that frame.
pub(crate) fn a_png_of(wide: u32, tall: u32, bgr0: &[u8]) -> Option<Vec<u8>> {
    let pixels = usize::try_from(wide)
        .ok()?
        .checked_mul(usize::try_from(tall).ok()?)?;
    if pixels == 0 || bgr0.len() != pixels.checked_mul(4)? {
        return None;
    }
    let rgb: Vec<u8> = bgr0
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[blue, green, red, _]| [red, green, blue])
        .collect();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&rgb, wide, tall, image::ExtendedColorType::Rgb8)
        .ok()?;
    Some(png)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "picturing_the_window_in_front_tests.rs"]
mod tests;
