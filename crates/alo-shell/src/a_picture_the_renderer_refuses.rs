//! **A picture the renderer refuses ends nothing but that picture.**
//!
//! # What happened
//!
//! Measured on a booted machine on 2026-11-12 (qemu q35, virtio-vga, no
//! graphics card, so pixman draws): Super+M on Firefox ended the desktop, twice,
//! with `Render(Submission("The requested operation is not supported"))`.
//! systemd started it again, and every window the person had open was gone. The
//! same keys on Dolphin put it aside as a live miniature.
//!
//! # Why pixman refused it
//!
//! Pixman draws a scaled picture through a transform held in 16.16 fixed
//! point, whose entries cannot reach 32 768. Smithay builds it as *move by
//! minus the destination's corner, then scale by source over destination*, so
//! one entry is the destination's corner times how many times smaller the
//! picture is drawn; past the limit pixman will not build the transform, and
//! smithay returns `PixmanError::Unsupported` — that sentence. A miniature is
//! a whole window drawn some forty pixels wide **at the right-hand edge of the
//! screen**: a buffer 1 400 wide drawn 40 wide at x = 1 224 is 1 224 × 35 ≈
//! 42 800, and refused; Dolphin's 760 at the same place is about 23 000, and
//! drawn. `a_picture_the_renderer_refuses_tests` measures both on pixman every
//! time it runs. The measurement the miniature landed with drew into a column
//! 112 pixels wide at x = 35, where the product never is, so it passed.
//!
//! # What happens instead
//!
//! The painter (`crate::scene_drawing`) notes each refusal here and draws the
//! rest of the frame. **One window's picture is that window's problem**:
//!
//! - **a window on the canvas** is drawn without the picture that was refused,
//!   in each frame that cannot draw it, and the log says so once;
//! - **a window put aside** shows its application's mark on a blank card from
//!   the next frame on — what a private window shows — and the log says so
//!   once. It is tried again the next time it is put aside;
//! - **an application's pointer** is drawn without its picture.
//!
//! **On the desktop a person signs into**, which hands the painter somewhere
//! to note them (`crate::direct_desktop`). The nested lane and the offscreen
//! checks hand it nowhere, and there a refusal is raised as it always was —
//! `the_nested_fixtures` holds that a truncated buffer is refused.
//!
//! The person is not shown a notice: nothing they had is lost or closed, and
//! the card they see says which application the window is.

use std::cell::RefCell;
use std::fmt::Display;

use smithay::reexports::wayland_server::{Resource as _, protocol::wl_surface::WlSurface};

/// Where a refused picture was going to be drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhereItWasGoing {
    /// A window, or one of its popups, on the canvas.
    OnTheCanvas,
    /// A window put aside, drawn small in its square in the panel.
    InTheMiniature,
    /// An application's own pointer.
    AsThePointer,
}

/// One picture the renderer would not draw.
#[derive(Debug, Clone)]
pub(crate) struct ARefusal {
    /// The window's root surface — or the popup's, or the pointer's.
    pub(crate) surface: WlSurface,
    /// Where it was going.
    pub(crate) going: WhereItWasGoing,
    /// The renderer's own words for why.
    pub(crate) why: String,
}

/// **The pictures the renderer refused while one frame was painted**, handed
/// to the painter by whoever will act on them after the frame.
///
/// A cell because the painter is handed its layers by shared reference, and
/// what it adds here is the one thing a frame tells back.
#[derive(Debug, Default)]
pub(crate) struct WhatTheRendererRefused(RefCell<Vec<ARefusal>>);

impl WhatTheRendererRefused {
    /// The renderer refused `surface`'s picture, going `going`, for `why`.
    pub(crate) fn refused(&self, surface: &WlSurface, going: WhereItWasGoing, why: &dyn Display) {
        self.0.borrow_mut().push(ARefusal {
            surface: surface.clone(),
            going,
            why: why.to_string(),
        });
    }

    /// Every refusal noted so far, leaving none.
    pub(crate) fn taken(&self) -> Vec<ARefusal> {
        self.0.take()
    }
}

/// Note a refusal where something will act on it — the desktop a person is
/// working on — and **raise it where nothing will**, as every refusal was
/// raised before: the nested lane and the offscreen checks are handed no
/// collector, and a check that a bad buffer is refused still reads the
/// refusal. Never silent either way.
///
/// # Errors
/// `why`, where there is nothing to note it in.
pub(crate) fn noted(
    refused: Option<&WhatTheRendererRefused>,
    surface: &WlSurface,
    going: WhereItWasGoing,
    why: crate::RenderError,
) -> Result<(), crate::RenderError> {
    match refused {
        Some(refused) => {
            refused.refused(surface, going, &why);
            Ok(())
        }
        None => Err(why),
    }
}

/// The line the log is told for one refusal.
fn what_the_log_is_told(going: WhereItWasGoing, number: Option<u64>, why: &dyn Display) -> String {
    let which = number.map_or_else(
        || "a surface".to_owned(),
        |number| format!("window {number}"),
    );
    let so = match going {
        WhereItWasGoing::OnTheCanvas => {
            "so it is left out of the frames that cannot draw it and the rest of the desktop is \
             drawn"
        }
        WhereItWasGoing::InTheMiniature => {
            "so its square in the panel shows its application's mark instead of a miniature"
        }
        WhereItWasGoing::AsThePointer => "so the pointer is drawn without its picture",
    };
    format!("alo-shell: the renderer refused to draw {which}'s picture ({why}), {so}")
}

/// **What the server remembers of the refusals**: which windows put aside are
/// drawn by their mark, and which refusals the log has heard of.
#[derive(Debug, Default)]
pub(crate) struct TheRefusedPictures {
    /// Windows put aside, by number, whose miniature the renderer refused.
    by_their_mark: Vec<u64>,
    /// Each refusal the log was told of: the surface, where it was going, and
    /// the window's number where it had one.
    said: Vec<(WlSurface, WhereItWasGoing, Option<u64>)>,
}

impl TheRefusedPictures {
    /// Remember `refusal`, of the window numbered `number` if it has one, and
    /// answer **the line to put in the log, the first time only**.
    pub(crate) fn remembered(&mut self, refusal: &ARefusal, number: Option<u64>) -> Option<String> {
        if refusal.going == WhereItWasGoing::InTheMiniature
            && let Some(number) = number
            && !self.by_their_mark.contains(&number)
        {
            self.by_their_mark.push(number);
        }
        let heard = self
            .said
            .iter()
            .any(|(surface, going, _)| *surface == refusal.surface && *going == refusal.going);
        if heard {
            return None;
        }
        self.said
            .push((refusal.surface.clone(), refusal.going, number));
        Some(what_the_log_is_told(refusal.going, number, &refusal.why))
    }

    /// Whether the window numbered `number`, put aside, is drawn by its mark.
    pub(crate) fn is_drawn_by_its_mark(&self, number: u64) -> bool {
        self.by_their_mark.contains(&number)
    }

    /// Forget every window no longer put aside — so one put aside again is
    /// tried again — and every surface that has gone.
    pub(crate) fn forget_all_but(&mut self, still_aside: &[u64]) {
        self.by_their_mark
            .retain(|number| still_aside.contains(number));
        self.said.retain(|(surface, going, number)| {
            surface.is_alive()
                && (*going != WhereItWasGoing::InTheMiniature
                    || number.is_some_and(|number| still_aside.contains(&number)))
        });
    }
}

impl crate::Server {
    /// **The renderer refused these pictures in the frame just drawn**: say
    /// so in the log, once each, and draw each window put aside among them by
    /// its mark from now on.
    pub(crate) fn the_renderer_refused(&mut self, refusals: &[ARefusal]) {
        let still_aside: Vec<u64> = self
            .minimized_surfaces()
            .filter_map(crate::window_number::Numbers::given_to)
            .collect();
        self.refused_pictures.forget_all_but(&still_aside);
        for refusal in refusals {
            let number = if refusal.surface.is_alive() {
                crate::window_number::Numbers::given_to(&refusal.surface)
            } else {
                None
            };
            if let Some(line) = self.refused_pictures.remembered(refusal, number) {
                eprintln!("{line}");
            }
        }
    }

    /// **The squares of the windows put aside whose miniature the renderer
    /// refused**, each the application's mark on a blank card, laid out where
    /// the panel was last drawn. Painted over the panel, so each covers the
    /// card its miniature would have been drawn into.
    pub(crate) fn the_miniatures_drawn_by_their_mark(
        &self,
        panel: &alo_put_aside::Panel,
        palette: &crate::desktop_look::DesktopPalette,
        fonts: &mut cosmic_text::FontSystem,
    ) -> Vec<crate::painted::Inked> {
        let Some(drawn) = self.panel_as_drawn.as_ref() else {
            return Vec::new();
        };
        drawn
            .the_windows_slots()
            .filter(|(id, _)| self.refused_pictures.is_drawn_by_its_mark(id.number()))
            .filter_map(|(id, slot)| {
                let preview = panel
                    .previews()
                    .iter()
                    .find(|preview| preview.window() == id)?;
                Some(crate::a_miniatures_card::a_windows_square_with_its_mark(
                    slot,
                    preview,
                    &self.what_each_application_draws,
                    palette,
                    fonts,
                ))
            })
            .collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_picture_the_renderer_refuses_tests.rs"]
mod tests;
