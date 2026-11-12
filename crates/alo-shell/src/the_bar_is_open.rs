//! **Whether the alo Bar is open**, decided once for every road into it.
//!
//! *The owner's decision of 2026-11-11*: "Clicking 'Find, open, or ask…' on the
//! Dock should not open the same panel as Super+Space. It should allow user to
//! text inside it" — and Super+Space is *the same as clicking the field*: one
//! place to type, by mouse or keyboard. So a click on the field, `Super`+`Space`
//! and `Super`+`A`, which all do nothing but give the composer the keyboard
//! (`crate::a_press_on_the_dock`, `crate::a_chord_opens_the_bar`), open the
//! Dock's field, and the matches rise from it.
//!
//! **There is one face, so this is one question.** The centred panel was left
//! for *Help with this window* until that road moved into the window's own panel
//! (#1182); with nothing left to open it, it was removed
//! (`crate::what_the_bar_shows`), and *which face* stopped being a question.

impl crate::Server {
    /// Whether the Bar is open: whether the Dock's field has the keyboard.
    pub(crate) fn the_bar_is_open(&self) -> bool {
        self.the_composer.has_the_keyboard()
    }
}

#[cfg(test)]
#[path = "the_bar_is_open_tests.rs"]
mod tests;
