//! **The keyboard a machine is typed on**, as frame `09a · Keyboard choice`
//! offers it: keyboards, the chosen one wearing the border and the word
//! *Selected*, and *Test your keyboard* under them.
//!
//! # What the design draws, and what it leaves to a rule
//!
//! `09a` draws three keyboards — each a name over the country it is named for
//! (`Belgian AZERTY` over `Belgium`) — then *Test your keyboard* over *Type
//! here to check letters and symbols.* It draws three because three fit; which
//! keyboards, and how a fourth is reached, it does not say. So the three are a
//! **window onto `alo_keyboards::the_keyboards_offered`** — the chosen
//! language's keyboards first, then every other keyboard alo OS offers any of
//! the 24 — moved through with the arrows, as the language sheet's window is.
//! That is how somebody who reads English reaches a German keyboard, which is
//! the owner's own example of 2026-10-28.
//!
//! The names are `xkeyboard-config`'s in the person's language
//! (`alo_keyboards::TheKeyboardNames`), not the design's sample names.
//!
//! # Where the ring goes
//!
//! Tab moves it from the keyboards to *Test your keyboard* to *Use this
//! keyboard* and back round. On the test row, letters a person types appear
//! in it — through the keyboard just chosen, because the compositor applies a
//! choice before the next key — which is the whole point of the row.

use alo_keyboards::Layout;

use crate::the_languages_offered::ROWS_AT_A_TIME;

/// The most a person can type into the test row before it stops taking
/// letters: enough for a word or two in any language, and short enough never
/// to wrap the row.
const AS_MUCH_AS_A_TEST_TAKES: usize = 40;

/// Where the focus ring is on this sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheRing {
    /// On the keyboard the cursor is on.
    OnTheKeyboards,
    /// On *Test your keyboard*, where letters typed appear.
    OnTheTest,
    /// On *Use this keyboard*.
    OnTheWayOn,
}

/// The keyboards, what is chosen, and what was typed to test it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheKeyboardPicker {
    /// Every keyboard offered, the chosen language's first.
    all: Vec<Layout>,
    /// Which one is chosen.
    chosen: usize,
    /// Where the cursor is among them.
    on: usize,
    /// Where the ring is.
    ring: TheRing,
    /// What was typed on the test row.
    typed: String,
    /// Whether a person chose a keyboard on this sheet, rather than it
    /// arriving chosen.
    touched: bool,
}

impl TheKeyboardPicker {
    /// The keyboards offered somebody reading `language`, with `chosen` chosen
    /// — or, where it is not among them, the first.
    pub(crate) fn offering(all: Vec<Layout>, chosen: Option<&Layout>) -> Self {
        let at = chosen
            .and_then(|it| all.iter().position(|one| one == it))
            .unwrap_or_default();
        Self {
            all,
            chosen: at,
            on: at,
            ring: TheRing::OnTheKeyboards,
            typed: String::new(),
            touched: false,
        }
    }

    /// The keyboard chosen.
    pub fn chosen(&self) -> Option<&Layout> {
        self.all.get(self.chosen)
    }

    /// Where the ring is.
    pub fn ring(&self) -> TheRing {
        self.ring
    }

    /// Whether a person chose a keyboard here, so it is applied before they
    /// test it.
    pub fn has_been_chosen(&self) -> bool {
        self.touched
    }

    /// What was typed on the test row.
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// Down one keyboard, stopping at the last.
    pub(crate) fn down(&mut self) {
        self.ring = TheRing::OnTheKeyboards;
        self.on = self
            .on
            .saturating_add(1)
            .min(self.all.len().saturating_sub(1));
    }

    /// Up one keyboard, stopping at the first.
    pub(crate) fn up(&mut self) {
        self.ring = TheRing::OnTheKeyboards;
        self.on = self.on.saturating_sub(1);
    }

    /// **A window of `rows` keyboards** (`crate::a_window_onto_a_list`); the
    /// test row is drawn after it, and is not in it.
    pub(crate) fn window_of(
        &self,
        rows: usize,
    ) -> crate::a_window_onto_a_list::AWindow<'_, Layout> {
        crate::a_window_onto_a_list::a_window(&self.all, self.on, Some(self.chosen), rows)
    }

    /// Tab: keyboards, then the test, then the way on, and round.
    pub(crate) fn ring_crosses(&mut self) {
        self.ring = match self.ring {
            TheRing::OnTheKeyboards => TheRing::OnTheTest,
            TheRing::OnTheTest => TheRing::OnTheWayOn,
            TheRing::OnTheWayOn => TheRing::OnTheKeyboards,
        };
    }

    /// Enter on a keyboard: it is chosen, and the ring goes to the test, so
    /// the next thing a person does is try it.
    pub(crate) fn choose(&mut self) {
        if self.on < self.all.len() {
            self.chosen = self.on;
            self.touched = true;
        }
        self.typed.clear();
        self.ring = TheRing::OnTheTest;
    }

    /// A letter typed while the ring is on the test row; nothing elsewhere.
    pub(crate) fn typed_a(&mut self, letter: char) {
        if self.ring == TheRing::OnTheTest
            && !letter.is_control()
            && self.typed.chars().count() < AS_MUCH_AS_A_TEST_TAKES
        {
            self.typed.push(letter);
        }
    }

    /// Erase on the test row.
    pub(crate) fn erased(&mut self) {
        if self.ring == TheRing::OnTheTest {
            self.typed.pop();
        }
    }

    /// The first keyboard in the window, derived from the cursor.
    fn first_in_the_window(&self) -> usize {
        let last_start = self.all.len().saturating_sub(ROWS_AT_A_TIME);
        self.on.saturating_sub(ROWS_AT_A_TIME / 2).min(last_start)
    }

    /// The keyboards drawn now.
    pub fn window(&self) -> &[Layout] {
        let from = self.first_in_the_window();
        let to = from.saturating_add(ROWS_AT_A_TIME).min(self.all.len());
        self.all.get(from..to).unwrap_or_default()
    }

    /// Where in the window the cursor is.
    pub fn cursor_in_the_window(&self) -> Option<usize> {
        self.on.checked_sub(self.first_in_the_window())
    }

    /// Where in the window the chosen keyboard is, while it is in it.
    pub fn chosen_in_the_window(&self) -> Option<usize> {
        let within = self.chosen.checked_sub(self.first_in_the_window())?;
        (within < ROWS_AT_A_TIME).then_some(within)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn offered_in_english() -> TheKeyboardPicker {
        TheKeyboardPicker::offering(
            alo_keyboards::the_keyboards_offered(&alo_strings::Language::written("en").unwrap()),
            None,
        )
    }

    /// **An English reader can choose a German keyboard**, with the arrows.
    #[test]
    fn an_english_reader_can_choose_a_german_keyboard() {
        let mut picker = offered_in_english();
        assert_eq!(picker.chosen().map(Layout::name), Some("us"));
        let german = Layout::named("de").unwrap();
        for _ in 0..200 {
            if picker.window().get(picker.cursor_in_the_window().unwrap()) == Some(&german) {
                break;
            }
            picker.down();
        }
        picker.choose();
        assert_eq!(picker.chosen(), Some(&german));
        assert_eq!(picker.ring(), TheRing::OnTheTest);
    }

    /// **The test row takes letters only while the ring is on it**, and a
    /// new choice starts the test again.
    #[test]
    fn the_test_row_takes_letters_only_when_it_has_the_ring() {
        let mut picker = offered_in_english();
        picker.typed_a('x');
        assert_eq!(picker.typed(), "", "letters reached the test from the list");
        picker.ring_crosses();
        for letter in "äöü".chars() {
            picker.typed_a(letter);
        }
        picker.erased();
        assert_eq!(picker.typed(), "äö");
        picker.ring_crosses();
        picker.typed_a('y');
        assert_eq!(
            picker.typed(),
            "äö",
            "letters reached the test from the way on"
        );
        picker.ring_crosses();
        picker.choose();
        assert_eq!(picker.typed(), "");
    }

    /// **A keyboard chosen before is chosen again** when the sheet reopens.
    #[test]
    fn a_keyboard_chosen_before_is_chosen_again() {
        let german = Layout::named("de").unwrap();
        let picker = TheKeyboardPicker::offering(
            alo_keyboards::the_keyboards_offered(&alo_strings::Language::written("en").unwrap()),
            Some(&german),
        );
        assert_eq!(picker.chosen(), Some(&german));
        assert!(picker.chosen_in_the_window().is_some());
    }
}
