//! Opening the window menu, moving through it, choosing from it, and copying
//! the title out of it.
//!
//! # What this is for
//!
//! The owner's direction of 2026-10-10: *layout and text selection are
//! progress, but the menu is unfinished until a person can open and use it.
//! Connect the edge's menu action, pointer and touch activation, keyboard
//! access, focus handling, dismissal, accessible names and actual menu
//! commands.* And: *exporting a module does not establish a production caller.*
//!
//! `crate::window_menu` lays the menu out. This decides what a press, a touch
//! or a key **means**, which is state that outlives a frame and therefore
//! cannot live in a layout — the shape `alo_dock::Revealing` has for the edge
//! and `crate::window_edge_tooltip::ShowingTheTitle` has for the tooltip.
//!
//! # Three roads in, and they are the same menu
//!
//! A pointer press on the edge's menu control, a finger on the same place, and
//! `Alt+Space` all arrive here. **The keyboard road opens with a row already
//! focused and the pointer road does not**, which is the one difference and it
//! is deliberate: a person who pressed a key has no pointer to hover with and
//! would otherwise have to press Down before anything was reachable, while a
//! person who clicked would see a row highlighted under a pointer that is not
//! on it.
//!
//! # Dismissal is a road out, not an absence
//!
//! Escape closes it, choosing closes it, and a press outside closes it — and
//! all three go through [`TheWindowMenu::dismissed`] so there is one place that
//! decides what closing does to the selection. A menu that forgot which window
//! it was for while staying open would offer somebody else's actions.
//!
//! # Copying the title is the clipboard's, not a string here
//!
//! [`TheWindowMenu::copy_the_title`] answers an `alo_clipboard::Offer` and the
//! thing that can hand the bytes over. It does **not** reach into a clipboard
//! itself: one application owns the selection at a time and `alo-clipboard`
//! holds that rule, so a shell that wrote into it directly would be a second
//! owner of a thing designed to have one.

use alo_clipboard::{CouldNotGive, Gives, Kind, Offer};
use alo_menus::Action;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};

/// Which road opened the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenedBy {
    /// A pointer pressed the menu control.
    APointer,
    /// A finger touched it.
    ATouch,
    /// The keyboard, by `Alt+Space` or by the control's own key.
    TheKeyboard,
}

impl OpenedBy {
    /// Whether a row should be focused the moment the menu opens.
    ///
    /// **Only for the keyboard.** A person who pressed a key has no pointer to
    /// hover with, so a menu that opened with nothing focused would need a Down
    /// press before anything was reachable. A person who clicked would instead
    /// see a row lit under a pointer that is not on it.
    #[must_use]
    pub const fn focuses_a_row(self) -> bool {
        matches!(self, Self::TheKeyboard)
    }
}

/// Which part of the title a person has selected, as byte offsets into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    /// Where the selection began.
    pub from: usize,
    /// Where it has reached.
    pub to: usize,
}

impl Selection {
    /// Whether anything is selected.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.from == self.to
    }
}

/// The window menu, open or not, and what a person has done in it.
#[derive(Debug, Clone, Default)]
pub struct TheWindowMenu {
    /// Whether it is open at all.
    ///
    /// **Separate from [`Self::window`]**, because a test opens one without a
    /// surface and a surface is the only thing production has to say *open* by.
    /// One flag rather than inferring it from the window keeps the two readings
    /// from drifting when a surface goes away under an open menu.
    opened: bool,
    /// The window it is open for, if it is open.
    window: Option<WlSurface>,
    /// Where it was opened, in logical pixels.
    at: Point<i32, Logical>,
    /// Which row the keyboard is on, if any.
    focused: Option<usize>,
    /// What of the title is selected.
    selection: Selection,
    /// The whole title of the window it is open for.
    title: String,
}

impl TheWindowMenu {
    /// Nothing is open.
    #[must_use]
    pub fn closed() -> Self {
        Self::default()
    }

    /// Open it for `window`, at `at`, by `road`.
    ///
    /// `title` is the application's own untruncated string, held so a selection
    /// and a copy mean the same text the menu drew.
    ///
    /// **Opening it for a second window replaces the first.** One menu is open
    /// at a time, as one is on every desktop a person arrives from, and a menu
    /// that kept the old window's actions while showing the new one's title
    /// would act on something nobody was looking at.
    pub fn open(
        &mut self,
        window: &WlSurface,
        title: &str,
        at: Point<i32, Logical>,
        road: OpenedBy,
        rows: usize,
    ) {
        self.window = Some(window.clone());
        self.opened = true;
        self.at = at;
        self.title = title.to_owned();
        self.selection = Selection::default();
        self.focused = (road.focuses_a_row() && rows > 0).then_some(0);
    }

    /// Open it with no window, for a test.
    ///
    /// **`#[cfg(test)]`, so production keeps one way in.** A `WlSurface` needs a
    /// client on a socket, which a unit test has none of, and the state this
    /// type holds is worth exercising without one. A menu opened in production
    /// with no window would be a menu that could act on nothing, so the real
    /// `open` requires one and this cannot be reached from outside a test.
    #[cfg(test)]
    pub(crate) fn opened_without_a_surface(
        &mut self,
        title: &str,
        at: Point<i32, Logical>,
        road: OpenedBy,
        rows: usize,
    ) {
        self.window = None;
        self.opened = true;
        self.at = at;
        self.title = title.to_owned();
        self.selection = Selection::default();
        self.focused = (road.focuses_a_row() && rows > 0).then_some(0);
    }

    /// Whether it is open, and for which window.
    #[must_use]
    pub const fn open_for(&self) -> Option<&WlSurface> {
        self.window.as_ref()
    }

    /// Where it was opened.
    #[must_use]
    pub const fn at(&self) -> Point<i32, Logical> {
        self.at
    }

    /// Which row the keyboard is on.
    #[must_use]
    pub const fn focused(&self) -> Option<usize> {
        self.focused
    }

    /// What of the title is selected.
    #[must_use]
    pub const fn selection(&self) -> Selection {
        self.selection
    }

    /// Close it, forgetting the window, the focus and the selection.
    ///
    /// **One road out for all three ways of closing** — Escape, choosing, and a
    /// press outside — so there is one place that decides what closing does.
    pub fn dismissed(&mut self) {
        *self = Self::closed();
    }

    /// Move the keyboard to the next row, wrapping.
    ///
    /// **Wraps rather than stopping**, which is what every menu a person has
    /// used does: pressing Down at the bottom reaches the top, so the last row
    /// is one press from the first rather than `rows - 1` presses.
    pub const fn the_keyboard_went_down(&mut self, rows: usize) {
        if rows == 0 || !self.opened {
            return;
        }
        self.focused = Some(match self.focused {
            Some(which) if which + 1 < rows => which + 1,
            Some(_) => 0,
            None => 0,
        });
    }

    /// Move the keyboard to the previous row, wrapping.
    pub const fn the_keyboard_went_up(&mut self, rows: usize) {
        if rows == 0 || !self.opened {
            return;
        }
        self.focused = Some(match self.focused {
            Some(0) | None => rows - 1,
            Some(which) => which - 1,
        });
    }

    /// A pointer or a finger moved onto a row.
    ///
    /// **Hovering focuses, which is what makes the two roads one menu**: a
    /// person who opened it with a pointer and then reached for the keyboard
    /// carries on from the row they were pointing at, rather than from the top.
    pub const fn the_pointer_is_on(&mut self, row: usize) {
        if self.opened {
            self.focused = Some(row);
        }
    }

    /// Begin a selection of the title at `byte`.
    pub const fn selecting_from(&mut self, byte: usize) {
        self.selection = Selection {
            from: byte,
            to: byte,
        };
    }

    /// Extend the selection to `byte`.
    pub const fn selecting_to(&mut self, byte: usize) {
        self.selection.to = byte;
    }

    /// Select the whole title.
    ///
    /// What a person means by pressing the title once, or by `Ctrl+A` in it —
    /// and the common case, because the reason they opened the menu was to read
    /// or copy all of it.
    pub const fn select_the_whole_title(&mut self) {
        self.selection = Selection {
            from: 0,
            to: self.title.len(),
        };
    }

    /// What is selected, as text.
    ///
    /// Both ends are moved to a character boundary before slicing, for
    /// `crate::window_menu`'s reason: a byte offset from a pointer is not one,
    /// and slicing on it would panic on every script that needs more than one
    /// byte a letter.
    #[must_use]
    pub fn selected(&self) -> &str {
        let (from, to) = if self.selection.from <= self.selection.to {
            (self.selection.from, self.selection.to)
        } else {
            (self.selection.to, self.selection.from)
        };
        let end = floor(&self.title, to.min(self.title.len()));
        let start = floor(&self.title, from.min(end));
        self.title.get(start..end).unwrap_or("")
    }

    /// What to put on the clipboard, and what can hand it over.
    ///
    /// **The whole title when nothing is selected**, because a person who opened
    /// the menu and chose *copy* without dragging meant the title, and copying
    /// nothing would be a command that silently did nothing.
    ///
    /// Answers [`None`] only when there is no title at all — an application that
    /// named itself nothing — because an offer of an empty string is an offer a
    /// paste cannot tell from a failure.
    ///
    /// # Errors
    ///
    /// None here: the refusal is the clipboard's, when it checks that the form
    /// asked for is one this offer made.
    #[must_use]
    pub fn copy_the_title(&self) -> Option<(Offer, TheTitleAsText)> {
        if !self.opened {
            return None;
        }
        let text = if self.selection.is_empty() {
            self.title.clone()
        } else {
            self.selected().to_owned()
        };
        if text.is_empty() {
            return None;
        }
        let offer = Offer::copied(vec![Kind::text()]).ok()?;
        Some((offer, TheTitleAsText(text)))
    }
}

/// The copied title, able to hand itself over in the one form it offered.
///
/// A type rather than a closure because `alo_clipboard::Clipboard::taken` holds
/// the owner for as long as the selection lasts, and a name in a backtrace is
/// worth more than brevity when a paste fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheTitleAsText(String);

impl Gives for TheTitleAsText {
    fn give(&mut self, form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
        // **Checked rather than assumed**, although `Clipboard` checks first:
        // this type is handed over as a `Box<dyn Gives>` and a later caller
        // could ask for a form it never offered. Answering the title's bytes
        // for an image request would be worse than refusing.
        if *form == Kind::text() {
            Ok(self.0.clone().into_bytes())
        } else {
            Err(CouldNotGive::TheApplicationDidNot)
        }
    }
}

/// What choosing the focused row does, if a row is focused.
#[must_use]
pub fn chosen_from(menu: &TheWindowMenu, rows: &[Action]) -> Option<Action> {
    rows.get(menu.focused()?).copied()
}

/// Move `at` back to the nearest character boundary at or before it.
fn floor(whole: &str, at: usize) -> usize {
    let mut at = at.min(whole.len());
    while at > 0 && !whole.is_char_boundary(at) {
        at -= 1;
    }
    at
}

#[cfg(test)]
#[path = "window_menu_open_tests.rs"]
mod tests;
