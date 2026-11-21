//! **Every window's band is drawn by one function**, whatever kind of window
//! it hangs over.
//!
//! Until 2026-11-21 four files drew a band, one per kind of window:
//! `every_windows_edge` for applications, `an_alo_windows_edge` for Settings
//! and the record, `the_terminals_edge` and `a_task_windows_edge`. Each laid
//! the band out, asked whether it was revealed, worked out what the pointer
//! was on and which panel was open, and chose the title, in its own words. A
//! fix to one of them reached that kind only: #1138 hid the controls at rest
//! on applications and wrote *always shown* into the three others, which
//! stood for six days.
//!
//! So [`crate::Server::the_band_drawn`] answers all of it for any [`ABand`].
//! What differs between kinds is said once, in the exhaustive matches below:
//! where the window is, its title and its application's name, whether it has
//! the keyboard, and whether the band draws the card under it. A new kind of
//! window does not compile until it says each of those. The files that place
//! a band in a frame's layers (`every_windows_edge`, `a_task_windows_edge`,
//! `the_terminals_edge`, `settings_edge`, `record_edge`) only ask this, so
//! a band goes where its window goes in the order windows are drawn.

use std::time::Instant;

use alo_put_aside::AlosOwnWindow;
use smithay::utils::{Logical, Rectangle, Size};

use crate::Server;
use crate::an_edge_in_its_displays_units::{
    a_point_in_the_edges_units, a_window_in_the_edges_units,
};
use crate::the_bands_over_alos_windows::AlosBand;
use crate::which_band::ABand;
use crate::which_window_is_in_front::AKindOfWindow;
use crate::window_edge::{WindowEdge, edge_of};
use crate::window_edge_look::WhichLook;
use crate::window_edge_picture::{EdgePicture, OverTheEdge, TheWindow};
use crate::window_menu::{HowTheWindowIs, WhatARowDoes, WindowMenu, the_menu_of};
use crate::window_menu_paint::{PointingAtTheMenu, ThePreview};

/// **alo's name, as a task window's band says whose it is.** A name, so it
/// is never translated, as the mark on a task's card is drawn
/// (`crate::task_card_raster`).
const ALOS_NAME: &str = "alo";

/// What a band says: its window's title and its application's name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WhatTheBandSays {
    /// The window's title.
    pub(crate) title: Option<String>,
    /// The application's name, in quieter type after the title.
    pub(crate) application: Option<String>,
}

impl Server {
    /// Where `band`'s window is, in the band's units at `scale`. [`None`]
    /// for one of alo's windows that is not drawn on the display now.
    #[must_use]
    pub(crate) fn where_the_bands_window_is(
        &self,
        band: &ABand,
        scale: u32,
    ) -> Option<Rectangle<i32, Logical>> {
        match band {
            ABand::App(surface) => Some(self.the_window_a_band_hangs_from(surface, scale)),
            ABand::Alos(which) => self
                .alos_windows_with_a_band()
                .into_iter()
                .find(|(it, _)| it == which)
                .map(|(_, window)| a_window_in_the_edges_units(window, scale)),
        }
    }

    /// **What the band over `band`'s window says**, for every kind.
    #[must_use]
    pub(crate) fn what_the_band_says(
        &self,
        band: &ABand,
        says: &alo_strings::Strings,
    ) -> WhatTheBandSays {
        let say = |word: alo_strings::Word| {
            Some(
                says.say(&word.key(), &alo_strings::Filling::nothing())
                    .text()
                    .to_owned(),
            )
        };
        match band {
            ABand::App(surface) => WhatTheBandSays {
                title: crate::the_name_on_the_band::the_title_to_show(&self.the_name_of(surface))
                    .map(str::to_owned),
                application: self.the_application_on_the_band(surface),
            },
            // **Their title is their name, so it is said once** — the owner,
            // 2026-11-21: *"Once: \"Settings\""*.
            ABand::Alos(AlosBand::Its(AlosOwnWindow::TheTerminal)) => WhatTheBandSays {
                title: say(alo_terminal::words::TERMINAL),
                application: None,
            },
            ABand::Alos(AlosBand::Its(AlosOwnWindow::Settings)) => WhatTheBandSays {
                title: say(crate::the_settings_words::SETTINGS),
                application: None,
            },
            ABand::Alos(AlosBand::Its(AlosOwnWindow::TheRecord)) => WhatTheBandSays {
                title: Some(crate::the_records_words::TheRecordsSentences::of(says).title),
                application: None,
            },
            // **The person's request, then alo**: the owner, 2026-11-21,
            // *"Request · alo"*. The request is the person's own words, never
            // translated, as the task window draws them at its top.
            ABand::Alos(AlosBand::ATask(which)) => WhatTheBandSays {
                title: self
                    .tasks()
                    .the(*which)
                    .map(|task| task.in_the_panel().request)
                    .filter(|request| !request.trim().is_empty())
                    .or_else(|| say(alo_tasks::words::A_TASK)),
                application: Some(ALOS_NAME.to_owned()),
            },
        }
    }

    /// Whether the person's typing goes to `band`'s window.
    #[must_use]
    pub(crate) fn has_the_bands_window_the_keyboard(&self, band: &ABand) -> bool {
        match band {
            ABand::App(surface) => self.is_the_window_typing_goes_to(surface),
            ABand::Alos(AlosBand::Its(AlosOwnWindow::TheTerminal)) => {
                self.the_terminal_is_taking_the_keys()
            }
            ABand::Alos(AlosBand::Its(AlosOwnWindow::Settings)) => {
                self.settings_is_taking_the_keys()
            }
            ABand::Alos(AlosBand::Its(AlosOwnWindow::TheRecord)) => {
                self.the_record_is_taking_the_keys()
            }
            ABand::Alos(AlosBand::ATask(which)) => {
                self.the_kind_in_front() == AKindOfWindow::TaskWindows
                    && self.tasks().the_one_in_front() == Some(*which)
            }
        }
    }

    /// How `band`'s window is — maximised, full screen — as its panel reads it.
    #[must_use]
    pub(crate) fn how_the_bands_window_is(&self, band: &ABand) -> HowTheWindowIs {
        match band {
            ABand::App(surface) => self.how_this_window_is(surface),
            ABand::Alos(which) => self.how_alos_window_is(*which),
        }
    }

    /// Where Left or Right in hand would put `band`'s window, drawn without
    /// moving anything.
    #[must_use]
    pub(crate) fn the_preview_on_the_band(
        &self,
        band: &ABand,
        in_hand: Option<WhatARowDoes>,
        scale: u32,
    ) -> Option<ThePreview> {
        match band {
            ABand::App(surface) => self.the_preview_of(surface, in_hand, scale),
            ABand::Alos(which) => self.the_preview_of_alos(*which, in_hand, scale),
        }
    }

    /// `band`'s band as it is drawn at `now`, and how shown its controls are:
    /// laid out with them **while any of them is still to be seen**, so a
    /// fade out has something to draw. What a press reaches is asked of the
    /// reveal itself (`crate::a_band_is_pressed`), so a control fading out is
    /// drawn and is not pressed.
    #[must_use]
    pub(crate) fn the_band_as_drawn(
        &self,
        band: &ABand,
        window: Rectangle<i32, Logical>,
        room: Size<i32, Logical>,
        now: Instant,
    ) -> (WindowEdge, u16) {
        match band {
            ABand::App(surface) => {
                let how_shown = self.how_shown_its_controls_are(surface, now);
                let edge = edge_of(
                    window,
                    self.the_decorations_of(band),
                    how_shown > 0 || self.is_this_windows_edge_revealed(surface),
                    room,
                );
                (edge, how_shown)
            }
            ABand::Alos(which) => self.alos_band_as_drawn(*which, window, room, now),
        }
    }

    /// The panel over `band` as the frame draws it, with what the pointer and
    /// the keyboard are on and the preview of the choice in hand — [`None`]
    /// while its panel is shut.
    #[must_use]
    pub(crate) fn the_panel_drawn_over_the_band(
        &self,
        band: &ABand,
        edge: &WindowEdge,
        (room, scale): (Size<i32, Logical>, u32),
    ) -> Option<(WindowMenu, PointingAtTheMenu)> {
        let in_the_menu = self.where_the_keyboard_is_on_this_band(band).the_menu()?;
        let menu = the_menu_of(edge, self.how_the_bands_window_is(band), room)?;
        let hovered = self
            .where_the_pointer_is_in_logical_pixels()
            .map(|at| a_point_in_the_edges_units(at, scale))
            .and_then(|at| menu.what_is_under(at))
            .map(|row| row.does);
        let focused = in_the_menu.on();
        let preview = self.the_preview_on_the_band(band, focused.or(hovered), scale);
        Some((
            menu,
            PointingAtTheMenu {
                hovered,
                focused,
                preview,
            },
        ))
    }

    /// Whether alo wears its border round this application's window now, so
    /// a band drawn for it draws the border too.
    #[must_use]
    pub(crate) fn alo_wears_its_border_round(
        &self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> bool {
        self.alo_in_this_window(surface).is_some_and(|here| {
            here.wears == crate::alo_in_a_window::Wears::TheBorder
                && here.how != crate::alo_in_a_window::HowAloIsHere::WaitingForThePerson
        })
    }

    /// **The band over `band`'s window, drawn**, for a display `output`
    /// pixels big at `scale` per cent — or [`None`] where the window is not
    /// drawn, or its band would not be wholly on the display.
    ///
    /// The pointer is told where it is once a frame, before any band is
    /// drawn (`Server::the_edges_were_told_where_the_pointer_is`); this reads
    /// what that left.
    pub(crate) fn the_band_drawn(
        &self,
        band: &ABand,
        says: &alo_strings::Strings,
        fonts: &mut cosmic_text::FontSystem,
        (output, scale): ((i32, i32), u32),
    ) -> Option<EdgePicture> {
        let window = self.where_the_bands_window_is(band, scale)?;
        let room = crate::window_edge::the_usable_output(Size::from(output), scale);
        let (edge, how_shown) = self.the_band_as_drawn(band, window, room, Instant::now());
        let keyboard = self.where_the_keyboard_is_on_this_band(band);
        let pointing =
            self.what_this_edge_is_pointing_at_on(&edge, keyboard.what_the_painter_shows(), scale);
        let panel = self.the_panel_drawn_over_the_band(band, &edge, (room, scale));
        let saying = self.what_the_band_says(band, says);
        let look = WhichLook::of(self.has_the_bands_window_the_keyboard(band));
        let showing =
            crate::every_windows_edge::the_reason_to_show_a_tooltip(self, &edge, keyboard, scale);
        // **What only an application's band carries**: the line and shadow
        // round its window (alo's own windows draw their own), the field typed
        // in its panel, its band laid over full screen, and alo at work in it.
        let (body, laid_over, field, alo) = match band {
            ABand::App(surface) => {
                let laid_over = self.is_its_band_out_over_full_screen(surface);
                (
                    (!laid_over).then_some(window),
                    laid_over,
                    self.what_is_typed_in_the_window_field(surface),
                    self.alo_in_this_window(surface),
                )
            }
            ABand::Alos(_) => (None, false, None, None),
        };
        let card = edge.region.merge(window);
        let stop_pointed_at = crate::alos_band_in_a_window::the_band_of(window)
            .zip(self.where_the_pointer_is_in_logical_pixels())
            .is_some_and(|(foot, at)| foot.stop.contains(a_point_in_the_edges_units(at, scale)));
        let picture = EdgePicture::of(
            &edge,
            OverTheEdge {
                menu: panel.as_ref().map(|(menu, pointing)| (menu, *pointing)),
                field,
                showing,
                alo: alo
                    .as_ref()
                    .map(|here| crate::alos_band_in_a_window::AlosBand {
                        here,
                        card,
                        window,
                        stop_pointed_at,
                    }),
            },
            TheWindow {
                title: saying.title.as_deref(),
                application: saying.application.as_deref(),
                look,
                body,
                laid_over,
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
        )?;
        Some(match band {
            ABand::App(surface) => picture.on_the_window(surface.clone()),
            ABand::Alos(_) => picture,
        })
    }
}

#[cfg(all(test, target_os = "linux"))]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_band_is_drawn_tests.rs"]
mod tests;
