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

use crate::Server;
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
        let _ = (asked, may);
        (Pictured::CouldNotDraw, Vec::new())
    }
}

/// A PNG of a frame `wide` by `tall` whose rows are tightly packed blue, green,
/// red and a byte nobody reads, top to bottom — [`None`] for bytes that are not
/// that frame.
pub(crate) fn a_png_of(wide: u32, tall: u32, bgr0: &[u8]) -> Option<Vec<u8>> {
    let _ = (wide, tall, bgr0);
    None
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "picturing_the_window_in_front_tests.rs"]
mod tests;
