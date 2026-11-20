//! **A card comes in front of a window alo just opened.**
//!
//! Main's walk at 71932e07 (run 44161055698, picture 92): at *ask each time*,
//! alo opened `asked.txt` in its own copy of VSCodium after the person's
//! approval, and that copy came in front with the keyboard
//! (`crate::a_file_alo_opened_comes_in_front`). The card asking to type into it
//! then rose in the task window **behind** the copy, where the person could not
//! see what they were asked to approve — and their Tab and Enter went to the
//! editor.
//!
//! # The one case a card takes the keys
//!
//! *A question arriving never moves the keyboard* (the owner, 2026-10-26;
//! `crate::the_tasks_seat`), so a person typing in a document cannot approve a
//! card with two ordinary keys. That stands. What this adds is the one case
//! where the keys were not the person's to lose: **alo's own opening took them
//! from this task**, and nothing has moved them since. Then the card's task
//! window comes in front and takes them back, which is where the person was.
//! **The owner allowed it on 2026-11-20, in these words: *"Yes, allow the
//! narrow exception."*** It is recorded beside the rule in
//! `docs/design/the-ask-alo-task-window.md`. All four must hold:
//!
//! - the keys alo's opening took came from **this** task;
//! - the seat's focus is **still on the window alo opened** — the person has
//!   not gone anywhere since;
//! - **no task holds the keys** now;
//! - the **applications are in front**, not the terminal, Settings or the
//!   record.
//!
//! What alo took is remembered only **until that task's next answer**, a card
//! or words, so a card much later in a long turn does not reach back for it.
//!
//! # Why typing still reaches the window
//!
//! A task window holding the keys intercepts them at the seat and leaves the
//! seat's focus where it was (`crate::a_chord_closes_alos_own_window`). Typing
//! goes to the window holding the seat's focus
//! (`crate::the_window_in_front_alo_may_reach`), so after the person approves,
//! the text still reaches the copy alo opened, as it did before.
//!
//! # Why not in front without the keys
//!
//! Keys go to a task window only while one holds them; otherwise the seat's
//! focused window gets them. A card drawn in front without the keys would send
//! the person's typing into the copy it covers — a window they cannot see, the
//! fault the owner's *the window you go to comes in front* (2026-10-31) fixed.

use alo_dock::WindowId;

use crate::the_tasks_a_person_opened::WhichTask;
use crate::which_window_is_in_front::AKindOfWindow;

/// The keys alo's opening of a file took: from which task, to which window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AloTookTheKeys {
    /// The task that held them.
    pub(crate) from: WhichTask,
    /// The window alo opened, which took them.
    pub(crate) to: WindowId,
}

/// **Whether a card arriving in `card` takes the keys back**: this module's
/// four conditions, read from what alo `took`, the window the seat's focus is
/// on, whether a task holds the keys, and whether the applications are in
/// front.
pub(crate) fn the_card_takes_the_keys_back(
    took: Option<AloTookTheKeys>,
    card: WhichTask,
    seat: Option<WindowId>,
    a_task_holds_the_keys: bool,
    the_applications_are_in_front: bool,
) -> bool {
    took.is_some_and(|took| {
        took.from == card
            && seat == Some(took.to)
            && !a_task_holds_the_keys
            && the_applications_are_in_front
    })
}

impl crate::Server {
    /// **alo's opening is about to take the keys** to `window`: remember the
    /// task that held them, if one did.
    pub(crate) fn alo_takes_the_keys_to(&mut self, window: WindowId) {
        self.alo_took_the_keys = self
            .the_task_with_the_keyboard
            .map(|from| AloTookTheKeys { from, to: window });
    }

    /// **An answer arrived in `which`**: what alo took from it is spent, and
    /// where the answer is a card and this module's four conditions hold, the
    /// task takes the keys back and comes in front.
    pub(crate) fn a_card_may_take_back_what_alo_took(&mut self, which: WhichTask, a_card: bool) {
        let Some(took) = self.alo_took_the_keys.filter(|took| took.from == which) else {
            return;
        };
        self.alo_took_the_keys = None;
        let seat = self
            .surfaces
            .keyboard_root()
            .map(|root| crate::window_number::the_window_id_of(&root));
        if a_card
            && the_card_takes_the_keys_back(
                Some(took),
                which,
                seat,
                self.the_task_with_the_keyboard.is_some(),
                self.the_kind_in_front() == AKindOfWindow::Applications,
            )
        {
            self.this_task_took_the_keyboard(which);
            eprintln!(
                "alo-shell: the card came in front of the window alo opened, with the keys it took"
            );
        }
    }
}

#[cfg(test)]
#[path = "the_card_comes_in_front_of_what_alo_opened_tests.rs"]
mod tests;
