//! **Writing in VSCodium at *ask each time*** — Demo 2's second road at the
//! default level: the person sets the level and gives alo VSCodium in
//! Settings, as a person would, then asks alo to write in a file, and approves
//! each change on its own card.
//!
//! The proof the owner asked for on 2026-11-16: the same request the walk at
//! full access makes, carried through one approval per change. It stands on
//! three things that landed that day: the shell carries an approval off its
//! frame loop (#1309), the agent's turn goes on after each answer (#1310), and
//! Settings gives alo one application by name (#1307). The card shows the
//! text alo will type (#1308).
//!
//! # What it does, and what each check is
//!
//! 1. **The level**, in Settings → *Running code*: *Ask me each time*, the row
//!    [`ASK_EACH_TIME`] counts to from `alo_capability::HowCodeRuns::ALL`.
//! 2. **The grant**, in Settings → *What alo can use* → *Give alo an
//!    application*: VSCodium, which a fresh machine lists last (a test holds
//!    that, from the shipped names ordered as `alo-picking` orders them), for
//!    a day — the second length the list offers.
//!    Then the home folder, where the file is, for a day: opening a file at
//!    *ask each time* needs the file granted as well as the application.
//! 3. **The file is made by the person**, as at full access, and checked on
//!    the journal before alo is asked anything.
//! 4. **The ask**, in the alo bar.
//! 5. **A card to open the file**, which types nothing: the shell's own
//!    sentence (`alo_acting::said_on_the_log::a_card_is_up`) with nought
//!    characters. Tab moves to *Allow once* and Enter chooses it.
//! 6. **The file opens in alo's own copy**, as at full access.
//! 7. **A card to type, showing the whole text**: the same sentence, with the
//!    text's own count of characters — the card carries all of it, never cut.
//!    Tab, Enter.
//! 8. **The text is typed and saved**, and **the file on disk holds it**, read
//!    back by the person through the journal.
//!
//! # What it cannot show
//!
//! Which window has the keyboard when the second card comes up is the
//! shell's: the walk presses Tab and Enter believing it is the task window,
//! because the window alo opens a file in is not brought in front (#1272). A
//! run where those keys went to VSCodium stops at *alo types the text*.

use std::time::Duration;

use alo_acting::said_on_the_log::A_CARD_IS_UP;
use alo_acting::{Opened, Typed};
use alo_capability::HowCodeRuns;

use crate::keys::Chord;
use crate::settling::Settling;
use crate::step::{Since, Walk};
use crate::writing_in_vscodium::{opened_in_vscodium, saved_in_vscodium, typed_into_vscodium};

/// The file the person makes for this road, beside the one written at full
/// access, so what this road writes is not mistaken for that.
pub const THE_FILE_AT_EACH_ASK: &str = "asked.txt";

/// What alo is asked to write here: plain words, different from the first.
pub const THE_TEXT_AT_EACH_ASK: &str = "Written by alo after each yes";

/// The journal tag the person's check of this file is written under.
pub const THE_TAG_AT_EACH_ASK: &str = "alo-walk-the-file-asked";

/// The journal tag the person's check that this file was made is written
/// under.
pub const THE_FILE_IS_MADE_AT_EACH_ASK: &str = "alo-walk-the-file-asked-is-made";

/// The row of *Ask me each time* on Settings' *Running code* page, from
/// nought, in the order the page lists the levels.
pub const ASK_EACH_TIME: usize = match HowCodeRuns::ALL {
    [_, HowCodeRuns::AskingEachTime, _] => 1,
    _ => 0,
};

/// How many rows down from the first row of *What alo can use* *Give alo an
/// application* is, with nothing given yet: *Give alo a folder* is above it.
///
/// **Counted from the top**, because the page's end moves with what is
/// installed: since #1342 it ends with one switch per application, and a count
/// up from End landed among them (run 44034387104, pictures 86 and 91). Above
/// these rows is only what is already given, which this section decides.
/// `alo-shell`'s
/// `the_rows_that_give_are_counted_from_the_top_whatever_is_installed`
/// presses these keys in the real Settings window.
pub const DOWN_TO_GIVE_AN_APPLICATION: usize = 1;

/// How many rows down from the first row *Give alo a folder* is once VSCodium
/// is given: VSCodium's row is above it.
pub const DOWN_TO_GIVE_A_FOLDER: usize = 1;

/// *Running code*, counted down from the first page in Settings' side column
/// (`alo_shell::SettingsCategory::ALL`: alo and AI, What alo can use,
/// Running code).
const DOWN_TO_RUNNING_CODE: usize = 2;

/// *What alo can use*, counted down from the first page.
const DOWN_TO_WHAT_ALO_CAN_USE: usize = 1;

/// *A day*, counted down from the first length the list offers (an hour, a
/// day, a month, until I remove it): long enough for the rest of the walk, and
/// not for ever.
const DOWN_TO_A_DAY: usize = 1;

/// The line the person types to make the file and say so on the journal,
/// beginning with a space for `crate::writing_in_vscodium::making_the_file`'s
/// reason.
#[must_use]
pub fn making_the_file_at_each_ask() -> String {
    format!(
        " touch ~/{THE_FILE_AT_EACH_ASK} && systemd-cat -t {THE_FILE_IS_MADE_AT_EACH_ASK} ls \
         ~/{THE_FILE_AT_EACH_ASK}"
    )
}

/// The person's request, worded as the request at full access.
#[must_use]
pub fn the_ask_at_each_time() -> String {
    format!(
        "Open the file {THE_FILE_AT_EACH_ASK} in my home folder in VSCodium, type exactly this \
         text into it: {THE_TEXT_AT_EACH_ASK} and then save it."
    )
}

/// What the line saying a card is up says after its start, for a card with
/// this many characters of text.
fn showing(characters: usize) -> String {
    format!(" {characters} characters of text to type")
}

/// Then choose *ask each time*, give alo VSCodium, and write in it, approving
/// each change.
#[must_use]
pub fn at_each_ask(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let minutes = |it: u64| Duration::from_secs(it * 60);
    let settings = Chord::with("meta_l", "i");
    let terminal = Chord::holding(&["ctrl", "alt"], "t");
    let (home, down, end) = (Chord::key("home"), Chord::key("down"), Chord::key("end"));
    let (tab, ret, esc) = (Chord::key("tab"), Chord::key("ret"), Chord::key("esc"));
    walk.press(esc.clone())
        // **The level**: Running code, Ask me each time.
        .press(settings.clone())
        .look("settings-for-the-level", look.at_most(minutes(1)))
        .press(home.clone())
        .press_times(&down, DOWN_TO_RUNNING_CODE)
        .look("running-code", look)
        .press(tab.clone())
        .press_times(&down, ASK_EACH_TIME)
        .look("on-ask-each-time", look)
        .press(ret.clone())
        .look("ask-each-time-chosen", look)
        .press(esc.clone())
        .look("settings-closed-on-ask-each-time", look)
        // **The grant**: What alo can use, Give alo an application,
        // VSCodium, a day.
        .press(settings.clone())
        .look("settings-for-the-application", look.at_most(minutes(1)))
        .press(home.clone())
        .press_times(&down, DOWN_TO_WHAT_ALO_CAN_USE)
        .look("what-alo-can-use", look)
        .press(tab.clone())
        .press(home.clone())
        .press_times(&down, DOWN_TO_GIVE_AN_APPLICATION)
        .look("on-give-alo-an-application", look)
        .press(ret.clone())
        .look("the-applications", look)
        .press(end.clone())
        .look("on-vscodium", look)
        .press(ret.clone())
        .look("how-long", look)
        .press(home.clone())
        .press_times(&down, DOWN_TO_A_DAY)
        .look("on-a-day", look)
        .press(ret.clone())
        .look("vscodium-given", look)
        // **And the home folder, where the file is**: at *ask each time*
        // opening a file needs the file granted as well as the application
        // (`alo_turn::opening_a_file_in_an_application`, step 2). The first
        // merge walk of this section (run 43955643824) gave VSCodium alone, and
        // the turn ended in words after one step, with no card. The picker
        // opens in the home folder, and its first row picks the folder it is in.
        //
        // **From Settings opened afresh**, as the application's row was: a
        // button in the page draws no focus ring, so moving between two of
        // them changes nothing on the screen (run 43996140707: pictures 76 and
        // 77 are the same file). Settings opens on *alo and AI* with the ring
        // on its side column (picture 69), and that ring leaving is the change
        // the look on the row sees.
        .press(esc.clone())
        .look("settings-closed-before-the-folder", look)
        .press(settings)
        .look("settings-for-the-folder", look.at_most(minutes(1)))
        .press(home.clone())
        .press_times(&down, DOWN_TO_WHAT_ALO_CAN_USE)
        .look("what-alo-can-use-for-the-folder", look)
        .press(tab.clone())
        .press(home.clone())
        .press_times(&down, DOWN_TO_GIVE_A_FOLDER)
        .look("on-give-alo-a-folder", look)
        .press(ret.clone())
        .look("the-folder-picker-at-home", look)
        .press(home.clone())
        .press(ret.clone())
        .look("how-long-for-the-home-folder", look)
        .press(home)
        .press_times(&down, DOWN_TO_A_DAY)
        .look("on-a-day-for-the-home-folder", look)
        .press(ret.clone())
        .look("the-home-folder-given", look)
        .press(esc)
        .look("settings-closed-with-vscodium-given", look)
        // **The file, made by the person and checked before alo is asked.**
        .press(terminal.clone())
        .look("the-terminal-for-the-asked-file", look.at_most(minutes(1)))
        .type_text(&making_the_file_at_each_ask())
        .mark()
        .press(ret.clone())
        .look("the-asked-file-made", look)
        .expect_line_saying(
            "the person made the file for asking each time",
            THE_FILE_IS_MADE_AT_EACH_ASK,
            THE_FILE_AT_EACH_ASK,
            Since::Mark,
            minutes(1),
        )
        // **The ask**, and from here every check reads from this mark.
        .press(Chord::with("meta_l", "a"))
        .type_text(&the_ask_at_each_time())
        .look("the-ask-at-each-time-typed", look)
        .mark()
        .press(ret.clone())
        .expect_line_saying(
            "a card asks to open the file, typing nothing",
            A_CARD_IS_UP,
            &showing(0),
            Since::Mark,
            minutes(5),
        )
        .look("the-card-to-open", look.at_most(minutes(1)))
        // **A fresh mark before the approval**, so the typing card is not
        // judged on this card's line (run 44069105568); before rather than
        // after the opening is seen, because the typing card follows the
        // opening within one read of the log.
        .mark()
        .press(tab.clone())
        .press(ret.clone())
        .expect_line_saying(
            "after one approval alo opens the file in its own copy of VSCodium",
            &opened_in_vscodium(),
            Opened::InAlosCopy.written(),
            Since::Mark,
            minutes(3),
        )
        .expect_line_saying(
            "a card asks to type, showing the whole text",
            A_CARD_IS_UP,
            &showing(THE_TEXT_AT_EACH_ASK.chars().count()),
            Since::Mark,
            minutes(5),
        )
        .look("the-card-shows-the-text", look.at_most(minutes(1)))
        .press(tab)
        .press(ret.clone())
        .expect_line_saying(
            "after the second approval alo types the text",
            &typed_into_vscodium(),
            &Typed::Typed.written(),
            Since::Mark,
            minutes(3),
        )
        .expect_line_saying(
            "and saves it, after the text read back exactly",
            &saved_in_vscodium(),
            &Typed::AskedToSave.written(),
            Since::Mark,
            minutes(2),
        )
        .look("vscodium-after-asking-each-time", look.at_most(minutes(1)))
        // **The file on disk**, read back by the person.
        .press(terminal)
        .look("the-terminal-for-the-asked-check", look.at_most(minutes(1)))
        .type_text(&format!(
            " systemd-cat -t {THE_TAG_AT_EACH_ASK} cat ~/{THE_FILE_AT_EACH_ASK}"
        ))
        .mark()
        .press(ret)
        .expect_line_saying(
            "the file holds what alo typed after each approval",
            THE_TAG_AT_EACH_ASK,
            THE_TEXT_AT_EACH_ASK,
            Since::Mark,
            minutes(1),
        )
        .look("the-asked-file-checked", look.at_most(minutes(1)))
}

#[cfg(test)]
#[path = "writing_in_vscodium_at_each_ask_tests.rs"]
mod tests;
