//! **alo's terminal's window edge**: the strip above it with its icon, its
//! title and Close, in the focused or unfocused look — the edge every
//! application's window has.
//!
//! # What the design draws, and what is drawn
//!
//! The approved frame `557:148`, *Window · Terminal · not focused*, puts the
//! same edge over the terminal as over Dolphin (`554:56`): a 32-tall strip in
//! a 44 band, the icon at (14, 20), *Terminal* at (38, 19), and Minimise,
//! Maximise and Close on the trailing walk. A walk on a booted machine
//! (`8263ddcb`, 2026-11-01) showed the terminal on the canvas with none of it.
//!
//! **Drawn:** the strip, the icon (`crate::the_terminals_icon`), the title,
//! Minimise and Close, in the focused look while the terminal is the window the
//! keys go to and the unfocused one otherwise (`crate::window_edge_look`).
//! Minimise puts the terminal in the right-hand panel, still running
//! (`crate::alos_own_window_goes_aside`, design `27:2`).
//!
//! **And Window actions** since the owner's *"Yes add that"* of 2026-11-14:
//! its panel places the terminal as it places any window
//! (`crate::alos_windows_panel`, `crate::alos_window_placed`).
//!
//! # The application edge, reused
//!
//! Laid out by `crate::window_edge::edge_of`, painted by
//! `crate::window_edge_picture::EdgePicture`, pressed through
//! `crate::window_edge_pointing` — the very code every application's edge goes
//! through. **The window it is laid out from is where the camera draws the
//! terminal** (`Server::where_the_terminal_is`, worked out each frame by
//! `crate::the_terminal_on_the_plane::on_the_screen`), so the edge pans and
//! zooms with it, as #1084 made every application's do. Its title and
//! controls keep their size round a window of whatever size the zoom makes it.
//!
//! # Revealed as every band is
//!
//! No controls at rest: they appear as the pointer enters the band and go as
//! it leaves, and *Always show window controls* keeps them shown
//! (`crate::the_bands_over_alos_windows`). Until 2026-11-14 this edge was laid
//! out revealed and drawn shown whatever the pointer did, *because it carries
//! the only Close this window has* — a reading the owner overruled on seeing a
//! booted machine, recorded so in `docs/design/the-terminal-window.md`.
//!
//! # A press on Close
//!
//! `Server::the_person_closes_the_terminal` (`crate::closing_the_terminal`):
//! at once when the shell is idle, after a question when a command is running.

use alo_put_aside::AlosOwnWindow;
use smithay::utils::{Physical, Rectangle, Size};

use crate::Server;
use crate::an_edge_in_its_displays_units::{
    a_point_in_the_edges_units, a_window_in_the_edges_units,
};
use crate::the_bands_over_alos_windows::AlosBand;
use crate::which_band::ABand;
use crate::window_edge_picture::{EdgePicture, OverTheEdge};
use crate::window_edge_pointing::pointing_at;

/// The terminal's band, as `crate::the_bands_over_alos_windows` knows it.
const THE_TERMINAL: AlosBand = AlosBand::Its(AlosOwnWindow::TheTerminal);

impl Server {
    /// Where the terminal is drawn on the display now. [`None`] with no
    /// terminal open, and **with it in the right-hand panel**, whatever the
    /// last frame drew (`crate::alos_own_window_goes_aside`).
    pub(crate) fn where_the_terminal_is_seen(&self) -> Option<Rectangle<i32, Physical>> {
        self.alos_own_terminal.as_ref()?;
        if self.this_alo_window_is_aside(AlosOwnWindow::TheTerminal) {
            return None;
        }
        self.where_the_terminal_is
    }

    /// The terminal's edge as a picture for this display: `output` its size in
    /// pixels and `scale` its scale in per cent, as
    /// `crate::every_windows_edge` takes them.
    pub(crate) fn the_terminals_edge_picture(
        &self,
        fonts: &mut cosmic_text::FontSystem,
        says: &alo_strings::Strings,
        output: (i32, i32),
        scale: u32,
    ) -> Option<EdgePicture> {
        let room = crate::window_edge::the_usable_output(Size::from(output), scale);
        let window = self.where_the_terminal_is_seen()?;
        let (edge, how_shown) = self.alos_band_as_drawn(
            THE_TERMINAL,
            a_window_in_the_edges_units(window, scale),
            room,
            std::time::Instant::now(),
        );
        let pointer = self.where_the_pointer_is_in_logical_pixels();
        let keyboard = self.where_the_keyboard_is_on_this_band(&ABand::Alos(THE_TERMINAL));
        let pointing = pointing_at(
            &edge,
            pointer.map(|at| a_point_in_the_edges_units(at, scale)),
            keyboard.what_the_painter_shows(),
        );
        let title = says.say(
            &alo_terminal::words::TERMINAL.key(),
            &alo_strings::Filling::nothing(),
        );
        let look = crate::window_edge_look::WhichLook::of(self.the_terminal_is_taking_the_keys());
        // **Its window panel, while it is open** (`crate::alos_windows_panel`),
        // drawn above every window as an application's is.
        let (panel, pointing_at_it) = self
            .the_panel_drawn_over(THE_TERMINAL, &edge, (room, scale))
            .map_or((None, None), |(menu, pointing)| {
                (Some(menu), Some(pointing))
            });
        EdgePicture::of(
            &edge,
            OverTheEdge {
                menu: panel.as_ref().zip(pointing_at_it),
                field: None,
                showing: None,
                alo: None,
            },
            // **No body from the edge**: the terminal draws its own panel,
            // hairline and shadow (`crate::the_terminals_frame`), in the same
            // look as this strip.
            crate::window_edge_picture::TheWindow {
                title: Some(title.text()),
                application: None,
                look,
                body: None,
                laid_over: false,
                controls: crate::window_edge_paint::HowTheControlsLook {
                    pointing,
                    open: keyboard.the_menu().is_some(),
                    how_shown,
                    look,
                },
            },
            fonts,
            output,
            scale,
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_terminals_edge_tests.rs"]
mod tests;
