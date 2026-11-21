//! **A line in the journal where every window's band is drawn, and where its
//! controls are**, so a walk on a booted machine can point at them.
//!
//! The walk's band section (`crates/alo-walking`, PR #1440) points at a band
//! and presses each of its controls on each kind of window. It finds them only
//! from what the screen and the journal say, never from geometry it assumed.
//! So the frame says, from the pictures it is about to paint, in the words
//! `alo_acting::said_of_a_band` declares once for both sides:
//!
//! - *a band is drawn* — once when a band first appears or moves, with where it
//!   is in the display's pixels;
//! - *a band is no longer drawn* — once when it goes, so a band that falls off
//!   the screen is a line a walk can fail on and not a line that never comes;
//! - *a band's controls are drawn* — once when they appear, or move, with the
//!   English source word of each and where it is.
//!
//! **Said only when it changes**, so a frame with nothing new costs nothing.
//! The lines name a kind of window, an application's own `app_id` or alo's own
//! window's fixed name, and numbers. Nothing the person typed or reads is in
//! them.
//!
//! What a press did is said by the one press road (`crate::a_band_is_pressed`).

use alo_acting::said_of_a_band::{OnTheScreen, TheKind};
use alo_put_aside::AlosOwnWindow;
use smithay::utils::{Physical, Rectangle};

use crate::the_bands_over_alos_windows::AlosBand;
use crate::which_band::ABand;
use crate::window_edge::OnTheEdge;
use crate::window_edge_picture::EdgePicture;

/// One band as it was last said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SaidOfOneBand {
    /// Whose band.
    pub(crate) band: ABand,
    /// The kind of window it hangs over, as the line names it.
    pub(crate) kind: TheKind,
    /// Its id on the line, kept so a band that has gone is named as it was.
    pub(crate) id: String,
    /// Where it was drawn.
    pub(crate) at: OnTheScreen,
    /// Its controls and where, as last said; empty while none were drawn.
    pub(crate) controls: Vec<(OnTheEdge, OnTheScreen)>,
}

/// A rectangle of the display's pixels, as the line says one.
const fn on_the_screen(of: Rectangle<i32, Physical>) -> OnTheScreen {
    OnTheScreen {
        x: of.loc.x,
        y: of.loc.y,
        w: of.size.w,
        h: of.size.h,
    }
}

/// The English source word for a control, as the line says it.
#[must_use]
pub(crate) const fn the_word_for(control: OnTheEdge) -> &'static str {
    match control {
        OnTheEdge::WindowActions => alo_acting::said_of_a_band::WINDOW_ACTIONS,
        OnTheEdge::Minimise => alo_acting::said_of_a_band::MINIMISE,
        OnTheEdge::Close => alo_acting::said_of_a_band::CLOSE,
    }
}

/// What changed between the bands as last said and the bands drawn now: the
/// lines to say, and the bands to remember.
#[must_use]
pub(crate) fn what_changed(
    before: &[SaidOfOneBand],
    now: Vec<SaidOfOneBand>,
) -> (Vec<String>, Vec<SaidOfOneBand>) {
    let mut lines = Vec::new();
    for band in &now {
        let (kind, id) = (band.kind, band.id.as_str());
        let was = before.iter().find(|it| it.band == band.band);
        if was.map(|it| it.at) != Some(band.at) {
            lines.push(alo_acting::said_of_a_band::a_band_is_drawn(
                kind, id, band.at,
            ));
        }
        let controls_were = was.map_or(&[][..], |it| it.controls.as_slice());
        if !band.controls.is_empty() && controls_were != band.controls.as_slice() {
            let named: Vec<(&str, OnTheScreen)> = band
                .controls
                .iter()
                .map(|(control, at)| (the_word_for(*control), *at))
                .collect();
            lines.push(alo_acting::said_of_a_band::a_bands_controls_are_drawn(
                kind, id, &named,
            ));
        }
    }
    for was in before {
        if !now.iter().any(|it| it.band == was.band) {
            lines.push(alo_acting::said_of_a_band::a_band_is_no_longer_drawn(
                was.kind, &was.id,
            ));
        }
    }
    (lines, now)
}

impl crate::Server {
    /// The kind of window `band` hangs over, and its id on the line: an
    /// application's own `app_id`, or alo's own window's fixed name.
    #[must_use]
    pub(crate) fn the_kind_and_id_of(&self, band: &ABand) -> (TheKind, String) {
        match band {
            ABand::App(surface) => (
                TheKind::Application,
                self.the_application_of(surface)
                    .map(|it| it.name().to_owned())
                    .unwrap_or_default(),
            ),
            ABand::Alos(AlosBand::Its(AlosOwnWindow::TheTerminal)) => {
                (TheKind::Terminal, "terminal".to_owned())
            }
            ABand::Alos(AlosBand::Its(AlosOwnWindow::Settings)) => {
                (TheKind::Settings, "settings".to_owned())
            }
            ABand::Alos(AlosBand::Its(AlosOwnWindow::TheRecord)) => {
                (TheKind::Record, "record".to_owned())
            }
            ABand::Alos(AlosBand::ATask(which)) => (TheKind::Task, format!("task {}", which.at())),
        }
    }

    /// **Say where every band this frame paints is, where that changed.**
    ///
    /// `pictures` is every band the frame is about to paint on the display a
    /// walk looks at: applications', alo's own windows' and the task windows'.
    pub(crate) fn say_where_the_bands_are<'a>(
        &mut self,
        pictures: impl IntoIterator<Item = &'a EdgePicture>,
    ) {
        let now: Vec<SaidOfOneBand> = pictures
            .into_iter()
            .filter_map(|picture| {
                let band = picture.whose()?.clone();
                let at = on_the_screen(picture.where_it_is_on_the_display()?);
                let controls = picture
                    .where_its_controls_are_on_the_display()
                    .iter()
                    .map(|(control, at)| (*control, on_the_screen(*at)))
                    .collect();
                let (kind, id) = self.the_kind_and_id_of(&band);
                Some(SaidOfOneBand {
                    band,
                    kind,
                    id,
                    at,
                    controls,
                })
            })
            .collect();
        let (lines, remembered) = what_changed(&self.where_the_bands_were_said, now);
        self.where_the_bands_were_said = remembered;
        for line in lines {
            eprintln!("{line}");
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_bands_say_where_they_are_tests.rs"]
mod tests;
