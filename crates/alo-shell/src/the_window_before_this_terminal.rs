//! **The compositor's answer to `front`**: the window the person was working in
//! just before the terminal they are typing in
//! (`docs/contracts/terminal-commands.md`, *front*), and to `alo-tree`: where
//! this shell's own tree is on the accessibility bus.
//!
//! *This terminal* is whatever has the keyboard now, and it must be a terminal:
//! alo's own, or an application window whose identifier is one of the
//! terminals [`alo_capability::A_PERSONS_OWN`] lists. Anything else having the
//! keyboard means the command did not come from a person typing in a terminal —
//! a script a timer started, say — so *the window before this terminal* means
//! nothing and the answer is [`InFront::None`].
//!
//! The answer is then the most recent other holder of the keyboard that is
//! still open, from `crate::who_had_the_keyboard`. **Never another window
//! instead**: alo's own terminal and a window with no identifier are
//! [`InFront::Unnamed`], refused rather than passed over for an older window the
//! person did not mean.

use alo_acting::{InFront, WhereAlosTreeIs};
use smithay::utils::IsAlive as _;

use crate::which_window_is_in_front::AKindOfWindow;
use crate::who_had_the_keyboard::HadTheKeyboard;

impl crate::Server {
    /// The window the person was working in before this terminal.
    pub(crate) fn the_window_before_this_terminal(&mut self) -> InFront {
        self.surfaces.keyboard_was_with.forget(|window| !window.alive());
        let this = if self.the_kind_in_front() == AKindOfWindow::TheTerminal {
            HadTheKeyboard::AlosOwnTerminal
        } else {
            match self.surfaces.keyboard_root() {
                Some(window)
                    if self
                        .the_application_of(&window)
                        .is_some_and(|app| alo_capability::is_a_persons_own(app.name())) =>
                {
                    HadTheKeyboard::Window(window)
                }
                _ => return InFront::None,
            }
        };
        match self.surfaces.keyboard_was_with.latest_other_than(&this) {
            None => InFront::None,
            Some(HadTheKeyboard::AlosOwnTerminal) => InFront::Unnamed,
            Some(HadTheKeyboard::Window(window)) => {
                let Some(application) = self.the_application_of(window) else {
                    return InFront::Unnamed;
                };
                let title = match self.the_name_of(window) {
                    crate::FrameName::Given(title) => title,
                    _ => String::new(),
                };
                InFront::window(
                    crate::window_number::the_window_id_of(window).number(),
                    application.name(),
                    &title,
                )
            }
        }
    }

    /// Where this shell's own tree is on the accessibility bus, as the desktop
    /// told it when the tree was published.
    pub(crate) fn where_alos_tree_is(&self) -> WhereAlosTreeIs {
        self.alos_tree
            .as_deref()
            .map_or(WhereAlosTreeIs::Nowhere, WhereAlosTreeIs::at)
    }

    /// The desktop published this shell's tree, answering as `name` on the
    /// accessibility bus — or published none.
    pub fn the_shells_tree_answers_as(&mut self, name: Option<&str>) {
        self.alos_tree = name.map(str::to_owned);
    }
}
