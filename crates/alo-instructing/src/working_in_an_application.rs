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

/// **The paragraph a model is told, for this registry** — empty when it does
/// not declare both [`READ_WINDOW`] and [`TYPE_TEXT`].
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
    format!(
        "\n\nTo work in an application's window, send {READ_WINDOW} first: its answer names \
         the window's fields, documents and controls. Then type into one with {TYPE_TEXT}, \
         giving its name exactly as that answer showed it{pressing}. Never send a run line \
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
    }
}
