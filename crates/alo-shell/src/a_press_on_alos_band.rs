//! **A press on the band alo draws across a window it is working in**
//! (`crate::alos_band_in_a_window`): Stop stops that work; anywhere else on
//! the band is the band's, and reaches nothing behind it.
//!
//! # Stop stops; it does not undo
//!
//! Pressing Stop is the task window's own Stop (`crate::the_person_stops_a_task`,
//! ADR 0139): the window's band says *Stopping…* at once, alo asks the machine
//! for nothing more, a change waiting for the person's OK lapses unanswered, and
//! the task window says what was changed before it — which stays changed. That
//! report is where the person reads the resulting state; the band goes when the
//! work has ended, because the turn it marked is over.
//!
//! # Only the window in front of the pointer
//!
//! The band is drawn over its window's content, so a press there is one the
//! application would otherwise have had. It is the band's only where its window
//! is the one under the pointer: a window lying over the band's corner keeps
//! its own presses.

use smithay::backend::input::ButtonState;

use crate::alo_in_a_window::HowAloIsHere;

/// What a press on alo's band did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnAlosBand {
    /// It was on Stop, and the work was asked to stop.
    Stopped,
    /// It was on Stop while stopping already, which does nothing twice.
    AlreadyStopping,
    /// It was on the band and not on Stop: the band's, and nothing.
    OnTheBand,
}

impl crate::Server {
    /// A press that may be on alo's band across a window. Answers whether the
    /// band took it, so nothing else does.
    pub(crate) fn a_press_on_alos_band(&mut self, state: ButtonState) -> bool {
        if state != ButtonState::Pressed {
            return false;
        }
        self.what_a_press_on_alos_band_did().is_some()
    }

    /// Where a press now lands on alo's band, and what it did there.
    pub(crate) fn what_a_press_on_alos_band_did(&mut self) -> Option<OnAlosBand> {
        let location = self.surfaces.pointer.as_ref().map(|it| it.location)?;
        let in_front = self.the_window_under(location)?;
        let here = self.alo_in_this_window(&in_front)?;
        let at = self.where_the_pointer_is_in_logical_pixels()?;
        let scale = self.the_scale_an_edge_is_measured_at();
        let at = crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale);
        let window = self.the_window_a_band_hangs_from(&in_front, scale);
        let band = crate::alos_band_in_a_window::the_band_of(window)?;
        if !band.band.contains(at) {
            return None;
        }
        if !band.stop.contains(at) {
            return Some(OnAlosBand::OnTheBand);
        }
        if here.how == HowAloIsHere::Stopping {
            return Some(OnAlosBand::AlreadyStopping);
        }
        Some(if self.the_person_stops_a_task(here.which) {
            OnAlosBand::Stopped
        } else {
            OnAlosBand::AlreadyStopping
        })
    }
}
