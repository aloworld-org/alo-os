//! **The window you go to comes in front** — the owner's decision of
//! 2026-10-31.
//!
//! Three kinds of window share the screen: the applications' own, alo's
//! terminal, and alo's task windows. Until 2026-10-31 they were drawn in one
//! fixed order — applications, then the terminal over them, then every task
//! window over both — and nothing changed it. So a shortcut from a task window
//! to the terminal or an application sent the keyboard to a window the task
//! window still covered: the person typed into something they could not see.
//!
//! Now the kind a person goes to comes to the front of the other two:
//!
//! - **a task window**, by a click on it, by asking alo, or by Super+Tab or
//!   Alt+Tab reaching it;
//! - **the terminal**, by Ctrl+Alt+T, a click on it, or Super+Tab or Alt+Tab;
//! - **an application**, by a click on it, by the Dock, or by Super+Tab or
//!   Alt+Tab.
//!
//! The windows left behind stay open, drawn behind, **one Super+Tab away**:
//! the next-window shortcuts walk the applications, then the terminal, then
//! every open task window (`crate::a_chord_goes_round_every_window`). That is
//! what keeps `docs/features.md`'s *no window is buried behind another where
//! the person cannot find it* true now that a task window can be behind.
//!
//! # One order for drawing, pressing and typing
//!
//! [`WhichIsInFront`] is the one answer three things read: the painter
//! (`crate::scene_drawing`) draws the three kinds bottom to top in it, a press
//! goes to the topmost kind under the pointer ([`crate::Server::the_kind_under`]),
//! and the keyboard goes to the kind in front. Three readers of one value
//! cannot disagree about which window is on top.
//!
//! **Order between kinds, not inside one.** The applications keep the order
//! the compositor already gives them, and task windows keep theirs — the last
//! opened in front, never reordered (`crate::the_tasks_a_person_opened`).
//!
//! **Only a person's own road moves it.** An agent's verb that activates an
//! application (`crate::an_application_verb_is_carried_out`) does not bring it
//! in front of the task window the person is reading.

use smithay::utils::{Logical, Physical, Point};

/// One of the three kinds of window on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AKindOfWindow {
    /// The applications' own windows.
    Applications,
    /// alo's own terminal.
    TheTerminal,
    /// alo's task windows.
    TaskWindows,
    /// Settings, a window on the canvas since ADR 0145's third stage.
    Settings,
    /// The record window, on the canvas since ADR 0145's fourth stage.
    TheRecord,
}

/// How many kinds of window there are.
const HOW_MANY: usize = 5;

/// The kinds, bottom to top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhichIsInFront {
    /// Bottom first, the kind in front last.
    bottom_to_top: [AKindOfWindow; HOW_MANY],
}

impl Default for WhichIsInFront {
    fn default() -> Self {
        Self::AS_IT_WAS
    }
}

impl WhichIsInFront {
    /// **The order there was before 2026-10-31**: applications, the terminal
    /// over them, and task windows over both — and the record and Settings over
    /// all of them, in the order they were drawn before they joined the canvas.
    /// A machine where nobody has gone anywhere yet draws exactly what it drew.
    pub(crate) const AS_IT_WAS: Self = Self {
        bottom_to_top: [
            AKindOfWindow::Applications,
            AKindOfWindow::TheTerminal,
            AKindOfWindow::TaskWindows,
            AKindOfWindow::TheRecord,
            AKindOfWindow::Settings,
        ],
    };

    /// The kinds, bottom to top: the order they are painted in.
    pub(crate) const fn bottom_to_top(self) -> [AKindOfWindow; HOW_MANY] {
        self.bottom_to_top
    }

    /// The kinds, top to bottom: the order a press asks them in.
    pub(crate) fn top_to_bottom(self) -> [AKindOfWindow; HOW_MANY] {
        let mut order = self.bottom_to_top;
        order.reverse();
        order
    }

    /// This kind comes in front of the others, which keep their order behind
    /// it.
    pub(crate) fn comes_to_the_front(&mut self, kind: AKindOfWindow) {
        let others = self
            .bottom_to_top
            .into_iter()
            .filter(|other| *other != kind);
        let mut order = [kind; HOW_MANY];
        for (slot, other) in order.iter_mut().zip(others) {
            *slot = other;
        }
        self.bottom_to_top = order;
    }

    /// Whether the task windows are in front of both other kinds.
    ///
    /// **Only the tests ask now.** The painter asked it to paint task windows
    /// in front above the Dock until they became windows on the canvas (ADR
    /// 0145), and a window on the canvas is under the Dock wherever it is in
    /// the order.
    ///
    /// **Settings is left out of the question**: it is open far less often
    /// than it is shut, and a shut window is not in front of anything.
    #[cfg(test)]
    pub(crate) fn the_task_windows_are_in_front(self) -> bool {
        self.top_to_bottom()
            .into_iter()
            .find(|kind| !matches!(kind, AKindOfWindow::Settings | AKindOfWindow::TheRecord))
            == Some(AKindOfWindow::TaskWindows)
    }
}

impl crate::Server {
    /// The order the three kinds of window are in now.
    pub(crate) const fn which_is_in_front(&self) -> WhichIsInFront {
        self.which_is_in_front
    }

    /// **The person went to a window of this kind**, so it comes in front,
    /// and the keyboard leaves whatever it no longer belongs to.
    ///
    /// Going to an application or the terminal lets go of a task window's
    /// keyboard, because the task window is now behind. Going to a task
    /// window is `Server::this_task_took_the_keyboard`'s, which calls this.
    pub(crate) fn the_person_went_to(&mut self, kind: AKindOfWindow) {
        self.which_is_in_front.comes_to_the_front(kind);
        // An application started earlier does not take them away from here
        // when its window comes (`crate::a_window_the_person_opened`).
        self.the_person_waits_for_nothing();
        if kind != AKindOfWindow::TaskWindows {
            self.the_task_gave_the_keyboard_up();
        }
    }

    /// **The kind in front that has something to show**: the task windows
    /// only while one is open, the terminal only while its shell runs, and the
    /// applications otherwise.
    ///
    /// A kind with nothing open is not in front of anything, so closing the
    /// last task window leaves the terminal or the applications in front, in
    /// the order the person left them.
    pub(crate) fn the_kind_in_front(&self) -> AKindOfWindow {
        self.which_is_in_front
            .top_to_bottom()
            .into_iter()
            .find(|kind| match kind {
                AKindOfWindow::TaskWindows => self.tasks().the_one_in_front().is_some(),
                AKindOfWindow::TheTerminal => self
                    .alos_own_terminal()
                    .is_some_and(|terminal| !terminal.has_finished()),
                // **Settings while it is open and on the Place being looked
                // at** (ADR 0145): one on another Place is out of sight.
                AKindOfWindow::Settings => self.settings_is_open_here(),
                AKindOfWindow::TheRecord => self.the_record_is_open_here(),
                AKindOfWindow::Applications => true,
            })
            .unwrap_or(AKindOfWindow::Applications)
    }

    /// **The person went to this application's window**: it is activated, and
    /// the applications come in front.
    ///
    /// The road a person's own choice takes — the Dock, its window list, and
    /// the next-window shortcuts. An agent's verb calls
    /// `Server::activate_window` directly and moves nothing in front.
    ///
    /// # Errors
    /// As `Server::activate_window`; nothing moves in front when it refuses.
    pub(crate) fn the_person_goes_to_this_window(
        &mut self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> Result<(), crate::WindowActivationError> {
        self.activate_window(surface)?;
        self.the_person_went_to(AKindOfWindow::Applications);
        Ok(())
    }

    /// **The topmost kind of window under the pointer at `location`**, in the
    /// order they are drawn, or [`None`] where none is.
    ///
    /// Each kind is asked by the hit test its own presses already use: the
    /// task windows and the terminal in the display's pixels, the pointer's
    /// position truncated to the pixel it is in as `crate::pointer` does, and
    /// the applications on the plane.
    pub(crate) fn the_kind_under(&self, location: Point<f64, Logical>) -> Option<AKindOfWindow> {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the pointer is clamped to the output, which is well inside i32"
        )]
        let (x, y) = (location.x as i32, location.y as i32);
        let on_the_screen = Point::<i32, Physical>::from((x, y));
        let on_an_edge = Point::<i32, Logical>::from((x, y));
        self.which_is_in_front
            .top_to_bottom()
            .into_iter()
            .find(|kind| match kind {
                AKindOfWindow::TaskWindows => {
                    self.the_task_window_under(on_the_screen).is_some()
                        || self.a_task_windows_edge_is_under(on_an_edge)
                }
                AKindOfWindow::TheTerminal => {
                    self.the_terminal_is_under(on_the_screen)
                        || self.the_terminals_edge_is_under(on_an_edge)
                }
                AKindOfWindow::Settings => {
                    self.settings_is_under(on_the_screen)
                        || self.the_settings_edge_is_under(on_an_edge)
                }
                AKindOfWindow::TheRecord => {
                    self.the_record_is_under(on_the_screen)
                        || self.the_records_edge_is_under(on_an_edge)
                }
                AKindOfWindow::Applications => {
                    self.the_window_under(location).is_some()
                        || self.the_edge_under(location).is_some()
                        || self.the_name_under(location).is_some()
                }
            })
    }
}

#[cfg(test)]
#[path = "which_window_is_in_front_tests.rs"]
mod tests;
