//! **How a model is told to work in an application's window**: read it first,
//! then type or press through the verbs — never through a command.
//!
//! Walked on a booted machine on 2026-11-14 (`gpt-4o-mini` at OpenAI, full
//! access, VSCodium open with an `Untitled-1` editor): asked to *type hello
//! from alo into the new file that is open*, the model called
//! `accessible.type_text` and `accessible.read_window` with arguments they do
//! not take, five times, and then sent the run door `xdotool type 'hello from
//! alo'` three times. Every one came back as exit 127.
//!
//! The verbs were on the list it was shown; what nothing told it was **the
//! order** — the field's name comes from `accessible.read_window`'s answer —
//! nor that the run door is not a way to type into a window. This paragraph
//! says both, after the verbs and only when the registry has the two verbs it
//! names, so a machine without the accessibility fallback is never told of
//! verbs it does not have.
//!
//! # A sentence, not a refusal
//!
//! The run door at full access still runs `xdotool` if a person has it and the
//! model sends it. Refusing commands by name there would take away something
//! the person chose (ADR 0149, law 5), and it would be a list nobody could keep
//! complete. Steering the model costs the person nothing.
//!
//! **No grade is about this text**, as none is about the verbs' own words: a
//! grade names the instructions above them by digest (ADR 0034), and this is
//! after them.

use alo_capability::Verbs;

/// The verb that reads what an application's windows show —
/// `alo_adapters::fallback_verbs::READ_WINDOW`, which `alo-turn`'s tests hold
/// this to. Named here rather than depended on: this crate reads a registry and
/// names verbs, and declares none.
pub const READ_WINDOW: &str = "accessible.read_window";

/// The verb that types one approved text into one field.
pub const TYPE_TEXT: &str = "accessible.type_text";

/// The verb that presses one control.
pub const ACTIVATE_CONTROL: &str = "accessible.activate_control";

/// The verb that scrolls a page (2026-11-17).
pub const SCROLL_PAGE: &str = "accessible.scroll_page";

/// The verb that scrolls one named thing onto the screen (2026-11-17).
pub const SCROLL_TO: &str = "accessible.scroll_to";

/// The verb that moves a slider or a number box (2026-11-17).
pub const SET_VALUE: &str = "accessible.set_value";

/// The verb that chooses one option of a drop-down (2026-11-17).
pub const CHOOSE_OPTION: &str = "accessible.choose_option";

/// The verb that selects one item of a list without opening it (2026-11-17).
pub const SELECT_ITEM: &str = "accessible.select_item";

/// The verb that opens one item of a list, as a double-click would
/// (2026-11-17).
pub const OPEN_ITEM: &str = "accessible.open_item";

/// The verb that opens a path in an application the person names —
/// `alo_applications::OPEN_FILE_IN_APPLICATION`. Named so that opening an item
/// a window shows is told apart from it: asked to *open report.pdf in Dolphin*,
/// a model sent this with a guessed path in every try (`alo-every-try` runs
/// 44093922179 and 44145644681, 0 of 10 at both levels), where the file was in
/// the window Dolphin showed.
pub const OPEN_FILE_IN_APPLICATION: &str = "open_file_in_application";

/// **The paragraph a model is told, for this registry** — empty when it does
/// not declare both [`READ_WINDOW`] and [`TYPE_TEXT`].
///
/// **Scrolling is told only where both scrolls are declared**: a read answers
/// only what is on the screen, and nothing else tells a model that what it did
/// not see may be further down.
#[must_use]
pub fn working_in_an_application(verbs: &Verbs) -> String {
    if verbs.of(READ_WINDOW).is_none() || verbs.of(TYPE_TEXT).is_none() {
        return String::new();
    }
    let pressing = if verbs.of(ACTIVATE_CONTROL).is_some() {
        format!(", or press one with {ACTIVATE_CONTROL}")
    } else {
        String::new()
    };
    let scrolling = if verbs.of(SCROLL_PAGE).is_some() && verbs.of(SCROLL_TO).is_some() {
        format!(
            " {READ_WINDOW} answers only what is on the screen. Whenever the thing to bring onto \
             the screen has a name — a heading, a file, a button — send {SCROLL_TO} with that \
             name, once, even when the person says \"scroll down until\" it shows: it scrolls as \
             far as it takes. Send {SCROLL_PAGE} only to read on when nothing ahead has a name \
             you know. Never type, and never send keys such as Page Down, to scroll or to find \
             something: {TYPE_TEXT} never brings anything onto the screen, and it would type \
             into the page and save it. Then send {READ_WINDOW} again."
        )
    } else {
        String::new()
    };
    let setting = if verbs.of(SET_VALUE).is_some() {
        format!(
            " Move a slider or a number box to a whole number with {SET_VALUE}, never by \
             typing into it."
        )
    } else {
        String::new()
    };
    let choosing = if verbs.of(CHOOSE_OPTION).is_some() && verbs.of(SELECT_ITEM).is_some() {
        format!(
            " Choose an option of a drop-down with {CHOOSE_OPTION}, naming the drop-down as \
             that answer showed it and the option as the person said it; select an item of a \
             list, such as a file, with {SELECT_ITEM}, which never opens it."
        )
    } else {
        String::new()
    };
    let opening = if verbs.of(OPEN_ITEM).is_some() {
        let not_by_its_path = if verbs.of(OPEN_FILE_IN_APPLICATION).is_some() {
            format!(
                ", never with {OPEN_FILE_IN_APPLICATION}, which is for a path to open in an \
                 application that does not show it"
            )
        } else {
            String::new()
        };
        format!(
            " When the person asks to open something in a window that lists it — a file in a \
             file manager, an item of a list — read that window, then open it there as a \
             double-click would with {OPEN_ITEM}, naming it as that answer showed \
             it{not_by_its_path}; only when the person asks to open it."
        )
    } else {
        String::new()
    };
    format!(
        "\n\nTo work in an application's window, send {READ_WINDOW} first: its answer names \
         the window's fields, documents and controls. Then type into one with {TYPE_TEXT}, \
         giving its name exactly as that answer showed it{pressing}.{scrolling}{setting}{choosing}{opening} Never \
         send a run line \
         that types, clicks or presses keys (xdotool, ydotool, wtype or anything like them): \
         {TYPE_TEXT} is how alo types into a window, and it shows the person the whole text \
         before anything is typed."
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::the_verbs;
    use alo_capability::{Arg, Effect, Requires, Takes, Verb};
    use alo_strings::Word;

    /// A verb of the given name, reading one application.
    fn named(name: &str, effect: Effect) -> Verb {
        Verb::checked(
            name,
            Word::saying("testing.working.purpose", "work in a window"),
            effect,
            vec![Arg::taking(
                "application",
                Word::saying("testing.working.application", "the application"),
                Takes::Application,
            )],
            Requires::grants_over(["application"]),
            Word::saying("testing.working.sentence", "work in {application}"),
        )
        .unwrap()
    }

    /// **A machine without the two verbs is told nothing about them.**
    #[test]
    fn a_registry_without_the_fallback_is_told_nothing() {
        assert_eq!(working_in_an_application(&the_verbs()), "");
        let mut only_reading = the_verbs();
        only_reading
            .declare(named(READ_WINDOW, Effect::Read))
            .unwrap();
        assert_eq!(working_in_an_application(&only_reading), "");
    }

    /// **With both, the model is told the order and that a command is not
    /// how a window is typed into** — the two things the walk of 2026-11-14
    /// showed it did not know.
    #[test]
    fn with_both_verbs_the_order_and_the_command_road_are_told() {
        let mut verbs = the_verbs();
        verbs.declare(named(READ_WINDOW, Effect::Read)).unwrap();
        verbs.declare(named(TYPE_TEXT, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        let reading = told.find(READ_WINDOW).unwrap();
        let typing = told.find(TYPE_TEXT).unwrap();
        assert!(reading < typing, "{told}");
        assert!(told.contains("Never send a run line"), "{told}");
        assert!(told.contains("xdotool"), "{told}");
        assert!(!told.contains(ACTIVATE_CONTROL), "{told}");

        verbs
            .declare(named(ACTIVATE_CONTROL, Effect::Change))
            .unwrap();
        assert!(working_in_an_application(&verbs).contains(ACTIVATE_CONTROL));
        assert!(!working_in_an_application(&verbs).contains(SET_VALUE));
        verbs.declare(named(SET_VALUE, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        assert!(
            told.contains(SET_VALUE) && told.contains("never by typing"),
            "{told}"
        );
        assert!(!working_in_an_application(&verbs).contains(CHOOSE_OPTION));

        // **Choosing is told only where both choosing verbs are there.**
        verbs.declare(named(CHOOSE_OPTION, Effect::Change)).unwrap();
        assert!(!working_in_an_application(&verbs).contains(CHOOSE_OPTION));
        verbs.declare(named(SELECT_ITEM, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        assert!(
            told.contains(CHOOSE_OPTION) && told.contains(SELECT_ITEM),
            "{told}"
        );
        assert!(told.contains("never opens it"), "{told}");
    }

    /// **Scrolling is told only where both scrolls are offered**, and then it
    /// names both and says to read again after.
    #[test]
    fn scrolling_is_told_only_where_both_scrolls_are_offered() {
        let mut verbs = the_verbs();
        verbs.declare(named(READ_WINDOW, Effect::Read)).unwrap();
        verbs.declare(named(TYPE_TEXT, Effect::Change)).unwrap();
        verbs.declare(named(SCROLL_PAGE, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        assert!(!told.contains(SCROLL_PAGE), "{told}");
        verbs.declare(named(SCROLL_TO, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        assert!(
            told.contains(SCROLL_PAGE) && told.contains(SCROLL_TO),
            "{told}"
        );
        assert!(told.contains("only what is on the screen"), "{told}");
    }

    /// **Opening an item a window shows is told apart from opening a path**:
    /// the model is told to open it where the window lists it, with
    /// [`OPEN_ITEM`], and never with [`OPEN_FILE_IN_APPLICATION`] — which is
    /// named only where the machine offers it, and the paragraph names no
    /// opening at all without [`OPEN_ITEM`].
    #[test]
    fn an_item_a_window_shows_is_opened_there_and_not_by_its_path() {
        let mut verbs = the_verbs();
        verbs.declare(named(READ_WINDOW, Effect::Read)).unwrap();
        verbs.declare(named(TYPE_TEXT, Effect::Change)).unwrap();
        verbs
            .declare(named(OPEN_FILE_IN_APPLICATION, Effect::Change))
            .unwrap();
        let told = working_in_an_application(&verbs);
        assert!(!told.contains(OPEN_ITEM), "{told}");
        assert!(!told.contains(OPEN_FILE_IN_APPLICATION), "{told}");

        verbs.declare(named(OPEN_ITEM, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        assert!(told.contains("file manager"), "{told}");
        assert!(
            told.contains(&format!("never with {OPEN_FILE_IN_APPLICATION}")),
            "{told}"
        );
        let opening = told.find(OPEN_ITEM).unwrap();
        let not_by_path = told.find(OPEN_FILE_IN_APPLICATION).unwrap();
        assert!(opening < not_by_path, "{told}");

        // Without the path verb, it is not named.
        let mut without = the_verbs();
        without.declare(named(READ_WINDOW, Effect::Read)).unwrap();
        without.declare(named(TYPE_TEXT, Effect::Change)).unwrap();
        without.declare(named(OPEN_ITEM, Effect::Change)).unwrap();
        let told = working_in_an_application(&without);
        assert!(told.contains(OPEN_ITEM), "{told}");
        assert!(!told.contains(OPEN_FILE_IN_APPLICATION), "{told}");
    }
}
