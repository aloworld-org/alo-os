//! **The window panel's Left, Maximise, Right and Restore on alo's own
//! windows** — the terminal and each task window.
//!
//! The owner, 2026-11-14: *"Yes add that"* to Window actions on the terminal,
//! *"Yes, it too"* for the alo chat window, *"Terminal first"*. An
//! application's window is placed by telling its client a mode
//! (`crate::window_mode`); alo's windows have no client, and each keeps its
//! own rectangle on the plane — the terminal in
//! `Server::the_terminal_on_the_plane`, a task window in
//! `crate::the_tasks_a_person_opened::TheTasks`. Each lays itself out at
//! whatever size that rectangle is, every frame, so placing one is setting it.
//!
//! # The same rooms and the same halves as every window
//!
//! **Nothing here decides where a window goes.** Maximise is the room
//! `crate::the_room_a_window_fills` gives any maximised window on the display
//! showing most of it — the work area less the band at its top — and Left and
//! Right are `crate::a_window_on_one_side::the_half`, which asks
//! `alo-dividing`, so a terminal on the left and an application on the left
//! are on the same half (`tests/one_layout_decider.rs` holds that one file
//! builds a share; this reads one).
//!
//! # Restore
//!
//! The rectangle it had before the first placement is kept until Restore puts
//! it back, so Left, then Maximise, then Restore returns it to where it was
//! before Left — *Restore returns the size and place it had*.
//!
//! # Not yet
//!
//! **Full screen and Help with this window** are not offered on alo's windows
//! (`HowTheWindowIs::full_screen_is_not_offered`, and Help needs the window
//! field an application's panel has): a choice that does not work yet is not
//! drawn. Settings and the record lay their pages out at one size, so nothing
//! here places them.

use alo_dividing::area::Size as AtLeast;
use alo_dividing::{Window, WindowId};
use alo_put_aside::AlosOwnWindow;
use smithay::utils::{Logical, Physical, Rectangle};

use crate::a_window_on_one_side::{OneSide, TheHalf, the_half, the_words_of};
use crate::the_bands_over_alos_windows::AlosBand;
use crate::the_room_a_window_fills::{ADisplaysRoom, the_room_of};
use crate::window_menu::WhatARowDoes;

/// Where one of alo's windows was before the panel first placed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhereItWas {
    /// Which window.
    which: AlosBand,
    /// Its rectangle on the plane before.
    before: Rectangle<i32, Logical>,
}

/// A rectangle on the plane, as the task model keeps one — the plane's units,
/// in the type the model was written with (`crate::task_windows_on_the_plane`).
fn as_the_tasks_keep_it(window: Rectangle<i32, Logical>) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (window.loc.x, window.loc.y).into(),
        (window.size.w, window.size.h).into(),
    )
}

/// The same, back.
fn as_the_plane_holds_it(window: Rectangle<i32, Physical>) -> Rectangle<i32, Logical> {
    Rectangle::new(
        (window.loc.x, window.loc.y).into(),
        (window.size.w, window.size.h).into(),
    )
}

/// How a person names this window in the journal.
const fn its_name(which: AlosBand) -> &'static str {
    match which {
        AlosBand::Its(AlosOwnWindow::TheTerminal) => "the terminal",
        AlosBand::Its(AlosOwnWindow::Settings) => "Settings",
        AlosBand::Its(AlosOwnWindow::TheRecord) => "the record",
        AlosBand::ATask(_) => "a task window",
    }
}

impl crate::Server {
    /// Where this window is on the plane, if it is open and the panel can
    /// place it.
    #[must_use]
    pub(crate) fn alos_window_on_the_plane(
        &self,
        which: AlosBand,
    ) -> Option<Rectangle<i32, Logical>> {
        match which {
            AlosBand::Its(AlosOwnWindow::TheTerminal) => {
                self.the_terminal_on_the_plane.map(|(window, _)| window)
            }
            AlosBand::Its(AlosOwnWindow::Settings | AlosOwnWindow::TheRecord) => None,
            AlosBand::ATask(task) => self
                .tasks()
                .each_open_window()
                .into_iter()
                .find(|(it, _)| *it == task)
                .map(|(_, window)| as_the_plane_holds_it(window)),
        }
    }

    /// Put this window here on the plane, on the Place it is on. Answers
    /// whether it was there to put.
    fn alos_window_is_put(&mut self, which: AlosBand, window: Rectangle<i32, Logical>) -> bool {
        match which {
            AlosBand::Its(AlosOwnWindow::TheTerminal) => match self.the_terminal_on_the_plane {
                Some((_, place)) => {
                    self.the_terminal_on_the_plane = Some((window, place));
                    true
                }
                None => false,
            },
            AlosBand::Its(AlosOwnWindow::Settings | AlosOwnWindow::TheRecord) => false,
            AlosBand::ATask(task) => self
                .tasks_mut()
                .the_window_is_now(task, as_the_tasks_keep_it(window)),
        }
    }

    /// The room of the display showing most of this window.
    fn the_room_of_alos_window(&self, which: AlosBand) -> Option<ADisplaysRoom> {
        let window = self.alos_window_on_the_plane(which)?;
        let rooms: Vec<ADisplaysRoom> = self
            .surfaces
            .rooms
            .iter()
            .map(|(_, room, _)| *room)
            .collect();
        the_room_of(&rooms, window)
    }

    /// How small this window can be, as `alo-dividing` is told it: a task
    /// window no smaller than its layout holds (`alo_tasks::layout`), at this
    /// display's scale; the terminal any size, because it lays its grid out
    /// in whatever room it has.
    fn how_small_alos_window_can_be(&self, which: AlosBand) -> Window {
        let id = WindowId::from_compositor(0);
        match which {
            AlosBand::ATask(_) => {
                let scale = crate::display_scale::DisplayScale::of_per_cent(
                    self.the_scale_edges_are_measured_at,
                );
                let least = |it: u32| u32::try_from(scale.px(it)).unwrap_or(0);
                Window::at_least(
                    id,
                    AtLeast::of(
                        least(alo_tasks::layout::the_least_width()),
                        least(alo_tasks::layout::the_least_height()),
                    ),
                )
            }
            AlosBand::Its(_) => Window::any_size(id),
        }
    }

    /// Whether the panel has placed this window — maximised or on a half —
    /// so its middle picture reads Restore.
    #[must_use]
    pub(crate) fn is_alos_window_placed(&self, which: AlosBand) -> bool {
        self.alos_windows_placed.iter().any(|it| it.which == which)
    }

    /// Where this side would put this window, as the preview shows it.
    #[must_use]
    pub(crate) fn where_this_side_would_put_alos(
        &self,
        which: AlosBand,
        side: OneSide,
    ) -> Option<TheHalf> {
        let room = self.the_room_of_alos_window(which)?;
        the_half(room, side, self.how_small_alos_window_can_be(which))
    }

    /// The preview of the choice in hand on this window, where it is Left or
    /// Right — exactly as an application's is drawn
    /// (`Server::the_preview_of`).
    pub(crate) fn the_preview_of_alos(
        &self,
        which: AlosBand,
        in_hand: Option<WhatARowDoes>,
        scale: u32,
    ) -> Option<crate::window_menu_paint::ThePreview> {
        let side = in_hand?.the_side()?;
        let half = self.where_this_side_would_put_alos(which, side)?;
        let drawn =
            crate::the_terminal_on_the_plane::on_the_screen(half.outline, self.the_camera());
        Some(crate::window_menu_paint::ThePreview {
            outline: crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
                drawn, scale,
            ),
            says: the_words_of(side, half.share),
        })
    }

    /// Do what a choice in the panel says to this window. Answers whether it
    /// was done; a refusal changes nothing.
    pub(crate) fn place_alos_window(&mut self, which: AlosBand, does: WhatARowDoes) -> bool {
        let Some(now) = self.alos_window_on_the_plane(which) else {
            return false;
        };
        let to = match does {
            WhatARowDoes::Maximise => self
                .the_room_of_alos_window(which)
                .and_then(ADisplaysRoom::maximised)
                .map(|area| Rectangle::new(area.at.into(), area.size.into())),
            WhatARowDoes::Left | WhatARowDoes::Right => does.the_side().and_then(|side| {
                let share = self
                    .where_this_side_would_put_alos(which, side)?
                    .share
                    .ok()?;
                Some(Rectangle::new(share.at.into(), share.size.into()))
            }),
            WhatARowDoes::Restore => {
                let before = self
                    .alos_windows_placed
                    .iter()
                    .find(|it| it.which == which)
                    .map(|it| it.before);
                let put = before.is_some_and(|before| self.alos_window_is_put(which, before));
                if put {
                    self.alos_windows_placed.retain(|it| it.which != which);
                    eprintln!("window panel: {} restored", its_name(which));
                }
                return put;
            }
            WhatARowDoes::FullScreen
            | WhatARowDoes::ExitFullScreen
            | WhatARowDoes::HelpWithThisWindow => None,
        };
        let Some(to) = to else {
            return false;
        };
        if !self.alos_window_is_put(which, to) {
            return false;
        }
        // **Where it was before the first placement**, kept through every
        // placement after it until Restore.
        if !self.is_alos_window_placed(which) {
            self.alos_windows_placed
                .push(WhereItWas { which, before: now });
        }
        self.say_where_the_panel_put(which, does, to);
        true
    }

    /// Forget where a window was once it has closed, so a terminal opened
    /// again, or a task numbered the same, does not read Restore.
    pub(crate) fn only_alos_windows_still_open_are_placed(&mut self) {
        let open: Vec<AlosBand> = self
            .alos_windows_placed
            .iter()
            .map(|it| it.which)
            .filter(|which| match which {
                AlosBand::Its(AlosOwnWindow::TheTerminal) => {
                    self.the_terminal_on_the_plane.is_some()
                }
                AlosBand::Its(AlosOwnWindow::Settings | AlosOwnWindow::TheRecord) => false,
                AlosBand::ATask(task) => self.tasks().is_open(*task),
            })
            .collect();
        self.alos_windows_placed
            .retain(|it| open.contains(&it.which));
    }

    /// **Say in the journal where the panel put it**, and, for Maximise,
    /// whether it fills the work area of its display — what the KVM walk
    /// reads (`crates/alo-walking`), since a walk has no other way to ask a
    /// booted machine where a window is.
    fn say_where_the_panel_put(
        &self,
        which: AlosBand,
        does: WhatARowDoes,
        to: Rectangle<i32, Logical>,
    ) {
        let whose = its_name(which);
        let fills = does == WhatARowDoes::Maximise
            && self
                .the_room_of_alos_window(which)
                .and_then(ADisplaysRoom::maximised)
                .is_some_and(|area| {
                    self.alos_window_on_the_plane(which)
                        == Some(Rectangle::new(area.at.into(), area.size.into()))
                });
        eprintln!(
            "{}",
            what_the_panel_says(whose, does, fills, (to.size.w, to.size.h))
        );
    }
}

/// The line the journal is told where the panel put one of alo's windows.
#[must_use]
pub(crate) fn what_the_panel_says(
    whose: &str,
    does: WhatARowDoes,
    fills: bool,
    (wide, tall): (i32, i32),
) -> String {
    if fills {
        format!("window panel: {whose} maximised, filling the work area ({wide} x {tall})")
    } else {
        format!("window panel: {whose} placed {does:?} at {wide} x {tall}")
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "alos_window_placed_tests.rs"]
mod tests;
