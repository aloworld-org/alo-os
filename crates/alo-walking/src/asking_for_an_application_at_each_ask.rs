//! **alo asks for an application nobody gave it, at *ask each time*** — the
//! sheet, its *This task*, the action asked again on its own card, and the
//! grant taken back when the task closes (the owner's decisions of 2026-11-16,
//! `docs/design/when-alo-asks-for-an-application.md`).
//!
//! It sets *ask each time* itself, so it stands on no other section, and asks
//! for **Papers**, which nothing gives alo: the VSCodium section at *ask each
//! time* gives VSCodium, so the two never share a grant. It walks last, after
//! the every-application section, and so only where that section does
//! (`crate::in_every_shipped_application::EVERY_APPLICATION`).
//!
//! # What it does, and what each check is
//!
//! 0. **The level**, in Settings → *Running code*: *Ask me each time*, the row
//!    [`ASK_EACH_TIME`] counts to. Harmless where it is already chosen.
//! 1. **The ask**, in the alo bar: *Open Papers.*
//! 2. **The sheet goes up** for Papers: the shell's own sentence
//!    (`alo_acting::said_on_the_log::the_sheet_asks_for`), and a frame kept
//!    while it is up.
//! 3. **Enter grants *This task*** — the length the keyboard starts on.
//! 4. **The same action is asked again, on its own card**: the card's
//!    sentence with nought characters of text. Tab moves to *Allow once* and
//!    Enter chooses it.
//! 5. **Papers opens**: the turn's own sentence for `open_application`, done.
//! 6. **The task closes, and the grant goes with it**: the shell's sentence for
//!    a task's grant taken back (`alo_acting::said_on_the_log::a_tasks_grant_went`).
//!
//! # What it cannot show
//!
//! Which window has the keyboard once Papers opens is the shell's: Papers may
//! take it, or the task window may keep it. So the walk presses Alt+F4 twice,
//! looking after each, and either order ends with the task closed; the check
//! is the journal's line, not the order. A run where neither press reached the
//! task stops at *the grant goes with the task*, with the frames to say why.

use std::time::Duration;

use alo_acting::said_on_the_log::{A_CARD_IS_UP, A_SHEET_ASKS_FOR, A_TASKS_GRANT_WENT};
use alo_capability::HowCodeRuns;

use crate::in_every_shipped_application::{PAPERS, acted, done_in};
use crate::keys::Chord;
use crate::settling::Settling;
use crate::step::{Since, Walk};

/// The row of *Ask me each time* on Settings' *Running code* page, from
/// nought, in the order the page lists the levels.
pub const ASK_EACH_TIME: usize = match HowCodeRuns::ALL {
    [_, HowCodeRuns::AskingEachTime, _] => 1,
    _ => 0,
};

/// *Running code*, counted down from the first page in Settings' side column
/// (`alo_shell::SettingsCategory::ALL`: alo and AI, What alo can use,
/// Running code).
pub const DOWN_TO_RUNNING_CODE: usize = 2;

/// How long alo is given to say its last words once Papers is open.
const THE_TURN_ENDS: Duration = Duration::from_secs(45);

/// What the person asks.
pub const THE_ASK_FOR_AN_APPLICATION: &str = "Open Papers.";

/// What the card's sentence says after its start, for a change that types
/// nothing.
pub const TYPING_NOTHING: &str = " 0 characters of text to type";

/// Then ask alo to open Papers, which nothing has given it, let it use Papers
/// for this task on the sheet, approve the opening on its card, and close the
/// task.
#[must_use]
pub fn for_this_task(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let minutes = |it: u64| Duration::from_secs(it * 60);
    let (tab, ret, esc) = (Chord::key("tab"), Chord::key("ret"), Chord::key("esc"));
    let close = Chord::with("alt", "f4");
    let (home, down) = (Chord::key("home"), Chord::key("down"));
    walk.press(esc.clone())
        // **The level**: Running code, Ask me each time.
        .press(Chord::with("meta_l", "i"))
        .look("settings-for-the-sheets-level", look.at_most(minutes(1)))
        .press(home)
        .press_times(&down, DOWN_TO_RUNNING_CODE)
        .press(tab.clone())
        .press_times(&down, ASK_EACH_TIME)
        .look("on-ask-each-time-for-the-sheet", look)
        .press(ret.clone())
        .press(esc)
        .look("ask-each-time-for-the-sheet", look)
        // **The ask**, and from here every check reads from this mark.
        .press(Chord::with("meta_l", "a"))
        .type_text(THE_ASK_FOR_AN_APPLICATION)
        .look("the-ask-for-papers-typed", look)
        .mark()
        .press(ret.clone())
        // **The sheet goes up**, for Papers, and a frame is kept of it.
        .expect_line_saying(
            "the sheet asks to let alo use Papers",
            A_SHEET_ASKS_FOR,
            PAPERS,
            Since::Mark,
            minutes(5),
        )
        .look("the-sheet-asks-for-papers", look.at_most(minutes(1)))
        // **This task**, where the keyboard starts: Enter grants it.
        .press(ret.clone())
        .look("papers-given-for-this-task", look)
        // **The same action, asked again, on its own card.**
        .expect_line_saying(
            "the opening is asked again, on its own card",
            A_CARD_IS_UP,
            TYPING_NOTHING,
            Since::Mark,
            minutes(3),
        )
        .look("the-card-to-open-papers", look.at_most(minutes(1)))
        .press(tab)
        .press(ret)
        .expect_line_saying(
            "after its approval alo opens Papers",
            &acted("open_application", PAPERS),
            &done_in(PAPERS),
            Since::Mark,
            minutes(3),
        )
        .look("papers-opened-for-this-task", look.at_most(minutes(1)))
        // **alo finishes its turn**: a running task asks before it closes, so
        // the walk gives the model its last words first.
        .wait(THE_TURN_ENDS)
        // **The task closes, and the grant goes with it.** Two presses, for
        // this file's header's reason.
        .mark()
        .press(close.clone())
        .look("after-the-first-close", look)
        .press(close)
        .look("after-the-second-close", look)
        .expect_line_saying(
            "closing the task takes back the grant made for it",
            A_TASKS_GRANT_WENT,
            "",
            Since::Mark,
            minutes(1),
        )
}

#[cfg(test)]
#[path = "asking_for_an_application_at_each_ask_tests.rs"]
mod tests;
