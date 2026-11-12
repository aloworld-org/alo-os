//! **A window put aside is asked to draw again four times a second**, so its
//! miniature in the right-hand panel stays live — the owner's choice of *live*
//! over a still picture in the approved design `27:2` (2026-11-12) — at a cost
//! bounded on a machine with no graphics card.
//!
//! # Why it has to be asked
//!
//! An application draws when the compositor says its last picture was shown:
//! the Wayland frame callback. A hidden window is sent none
//! (`crate::window_minimize`), so it stops drawing the moment it is put aside,
//! and its miniature would be the last picture it drew — a still. Asking it
//! here is what makes the miniature live.
//!
//! # Why four times a second and not every frame
//!
//! **Every frame would make a window put aside cost what a window on the
//! screen costs**: the application would lay out and paint its whole window
//! sixty times a second to be shown 44 pixels wide. On a machine whose only
//! renderer is the processor (ADR 0104: no graphics card is required) that is
//! the application's work, not the compositor's, and it is the larger of the
//! two. Four a second keeps a video, a download's progress or a build's output
//! visibly moving in the corner of an eye, and is a fifteenth of what a shown
//! window costs the application. The compositor's own share — drawing each
//! miniature from the window's buffer, every frame it composes — is measured
//! by `put_aside_windows_draw_again_tests`, every time the tests run.
//!
//! **Not a setting.** Nothing was offered here, so nothing is withdrawn
//! (law 5's second paragraph); the rate is a figure the owner can change.
//!
//! # Who is asked
//!
//! Only windows **in the panel and not private**: a private window has no
//! miniature (`crate::a_window_in_miniature`), so asking it to draw would
//! spend its owner's processor on a picture nobody is shown. A window hidden
//! by its own request rather than put aside is not in the panel and is not
//! asked either.

use std::time::{Duration, Instant};

use alo_put_aside::Panel;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::{
    SurfaceAttributes, TraversalAction, with_surface_tree_downward,
};

/// How often a window put aside is asked to draw again: four times a second.
pub(crate) const THEY_DRAW_AGAIN_EVERY: Duration = Duration::from_millis(250);

/// Whether it is time to ask again: never asked, or asked at least
/// [`THEY_DRAW_AGAIN_EVERY`] ago.
#[must_use]
pub(crate) fn it_is_time(last: Option<Instant>, now: Instant) -> bool {
    last.is_none_or(|last| now.saturating_duration_since(last) >= THEY_DRAW_AGAIN_EVERY)
}

/// Tell every surface of `root` that its last picture was shown, at `time`
/// milliseconds: what asks a Wayland application to draw its next one.
fn asked_to_draw(root: &WlSurface, time: u32) {
    with_surface_tree_downward(
        root,
        (),
        |_, _, ()| TraversalAction::DoChildren(()),
        |_, states, ()| {
            for callback in states
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, ()| true,
    );
}

impl crate::Server {
    /// **Ask every window put aside to draw again, if it is time.** Answers how
    /// many were asked — none on a frame between two asks.
    pub fn the_put_aside_windows_draw_again(
        &mut self,
        panel: Option<&Panel>,
        now: Instant,
    ) -> usize {
        if !it_is_time(self.put_aside_last_asked_to_draw, now) {
            return 0;
        }
        let Some(panel) = panel else {
            return 0;
        };
        let asked: Vec<WlSurface> = panel
            .previews()
            .iter()
            .filter(|preview| crate::a_window_in_miniature::it_may_be_drawn_small(preview))
            .filter_map(|preview| self.the_put_aside_window_numbered(preview.window()))
            .collect();
        self.put_aside_last_asked_to_draw = Some(now);
        // A wrapping millisecond clock from this shell's first ask, which is
        // all a frame callback carries.
        static FIRST: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let since = now.saturating_duration_since(*FIRST.get_or_init(|| now));
        let time =
            u32::try_from(since.as_millis() % (u128::from(u32::MAX) + 1)).unwrap_or_default();
        for root in &asked {
            asked_to_draw(root, time);
        }
        asked.len()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "put_aside_windows_draw_again_tests.rs"]
mod tests;
