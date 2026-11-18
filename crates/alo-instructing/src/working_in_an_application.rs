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
/// The verb that opens one thing's right-click menu (2026-11-17).
pub const OPEN_MENU: &str = "accessible.open_menu";

/// The verb that moves a slider or a number box (2026-11-17).
pub const SET_VALUE: &str = "accessible.set_value";

/// The verb that chooses one option of a drop-down (2026-11-17).
pub const CHOOSE_OPTION: &str = "accessible.choose_option";

/// The verb that selects one item of a list without opening it (2026-11-17).
pub const SELECT_ITEM: &str = "accessible.select_item";

/// The verb that opens one item of a list, as a double-click would
/// (2026-11-17).
pub const OPEN_ITEM: &str = "accessible.open_item";

/// The verb that presses one key of a closed list (2026-11-17).
pub const PRESS_KEY: &str = "accessible.press_key";

/// The verb that types into a field and saves nothing (2026-11-17).
pub const TYPE_WITHOUT_SAVING: &str = "accessible.type_without_saving";

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
    // **One act, one verb**, each said with what it is for (2026-11-17): told
    // the acts as a run of sentences around `type_text`, `gpt-4o-mini` sent
    // `type_text` with an empty text to scroll to a heading and to open a
    // file's menu, so each verb is now named beside the one act it does.
    //
    // **Where typing that saves nothing is offered, `type_text` is told as the
    // typing that saves** (2026-11-17): told "a field or a document", the model
    // sent `type_text` into a web page's search box in 17 of 20 tries.
    let mut acts = vec![if verbs.of(TYPE_WITHOUT_SAVING).is_some() {
        format!(
            "to put text into a document open in an editor and save it, {TYPE_TEXT}, only ever \
             with the text to type"
        )
    } else {
        format!(
            "to put text into a field or a document, {TYPE_TEXT}, only ever with the text to type"
        )
    }];
    // **A named control is pressed by its name, never reached by keys**: told
    // keys and controls side by side, the model sent Tab or an arrow to reach
    // the Reload button in 19 of 20 tries.
    if verbs.of(ACTIVATE_CONTROL).is_some() {
        acts.push(format!(
            "to press a button or another control the window names, {ACTIVATE_CONTROL} with that \
             name, never by sending keys to reach it"
        ));
    }
    if verbs.of(OPEN_MENU).is_some() {
        acts.push(format!(
            "to open the menu a right-click opens on one thing, as for renaming or deleting a \
             file in a list, {OPEN_MENU}, then {READ_WINDOW} again to see its items"
        ));
    }
    if verbs.of(SCROLL_PAGE).is_some() && verbs.of(SCROLL_TO).is_some() {
        acts.push(format!(
            "to bring a thing with a name — a heading, a file, a button — onto the screen, \
             {SCROLL_TO} with that name, once, even when the person says \"scroll down until\" \
             it shows, since {READ_WINDOW} answers only what is on the screen; and {SCROLL_PAGE} \
             only to read on when nothing ahead has a name you know; then {READ_WINDOW} again"
        ));
    }
    if verbs.of(SET_VALUE).is_some() {
        acts.push(format!(
            "to move a slider or a number box to a whole number, {SET_VALUE}, never by typing into it"
        ));
    }
    // **Choosing, as #1353 told it**, only where both choosing verbs are there.
    if verbs.of(CHOOSE_OPTION).is_some() && verbs.of(SELECT_ITEM).is_some() {
        acts.push(format!(
            "to choose an option of a drop-down, {CHOOSE_OPTION}, naming the drop-down as that \
             answer showed it and the option as the person said it; to select an item of a list, \
             such as a file, {SELECT_ITEM}, which never opens it"
        ));
    }
    // **Opening an item, as #1361 told it.**
    if verbs.of(OPEN_ITEM).is_some() {
        acts.push(format!(
            "to open an item, such as a file in a folder, as a double-click would, {OPEN_ITEM} \
             with its name, only when the person asks to open it"
        ));
    }
    // **A key, and typing that saves nothing** (2026-11-17), each beside its
    // one act like the rest.
    if verbs.of(TYPE_WITHOUT_SAVING).is_some() {
        acts.push(format!(
            "to type into a field — a search box, a form, an address or a message — \
             {TYPE_WITHOUT_SAVING}, which saves nothing"
        ));
    }
    if verbs.of(PRESS_KEY).is_some() {
        acts.push(format!(
            "to press Enter, Escape, Tab or an arrow where the keyboard is, only when the person \
             names that key, {PRESS_KEY}"
        ));
    }
    format!(
        "\n\nTo work in an application's window, send {READ_WINDOW} first: its answer names \
         the window's fields, documents and controls. Then act on one by its name exactly as \
         that answer showed it, with the one verb for that act: {}. Never send a run line that \
         types, clicks or presses keys (xdotool, ydotool, wtype or anything like them): \
         {TYPE_TEXT} is how alo types into a window, and it shows the person the whole text \
         before anything is typed.",
        acts.join("; ")
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
        assert!(!told.contains("Use an application's named verb"), "{told}");
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
        assert!(!working_in_an_application(&verbs).contains(OPEN_MENU));

        verbs.declare(named(OPEN_MENU, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        assert!(told.contains(OPEN_MENU), "{told}");
        assert!(told.contains("Never send a run line"), "{told}");
        assert!(!working_in_an_application(&verbs).contains(PRESS_KEY));

        verbs.declare(named(PRESS_KEY, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        assert!(told.contains(PRESS_KEY), "{told}");
        assert!(told.contains("Never send a run line"), "{told}");
        assert!(!told.contains(TYPE_WITHOUT_SAVING), "{told}");

        verbs
            .declare(named(TYPE_WITHOUT_SAVING, Effect::Change))
            .unwrap();
        let told = working_in_an_application(&verbs);
        assert!(told.contains(TYPE_WITHOUT_SAVING), "{told}");
        assert!(told.contains("Never send a run line"), "{told}");
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
    /// **Where typing that saves nothing is offered, `type_text` is told as the
    /// typing that saves**, a key only as one the person names, and a control
    /// by its name rather than by keys — and no act runs words apart.
    #[test]
    fn typing_keys_and_controls_are_each_told_apart() {
        let mut verbs = the_verbs();
        for (name, effect) in [
            (READ_WINDOW, Effect::Read),
            (TYPE_TEXT, Effect::Change),
            (ACTIVATE_CONTROL, Effect::Change),
        ] {
            verbs.declare(named(name, effect)).unwrap();
        }
        let before = working_in_an_application(&verbs);
        assert!(before.contains("into a field or a document"), "{before}");
        assert!(before.contains("never by sending keys"), "{before}");
        verbs
            .declare(named(TYPE_WITHOUT_SAVING, Effect::Change))
            .unwrap();
        verbs.declare(named(PRESS_KEY, Effect::Change)).unwrap();
        let told = working_in_an_application(&verbs);
        assert!(!told.contains("into a field or a document"), "{told}");
        assert!(told.contains("open in an editor and save it"), "{told}");
        assert!(told.contains("a search box"), "{told}");
        assert!(
            told.contains("only when the person names that key"),
            "{told}"
        );
        assert!(!told.contains("  "), "two spaces run words apart: {told}");
    }
}
