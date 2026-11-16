//! **What the desktop says on its log when it carries out alo's acts** —
//! opening a file, typing, saving — declared once, here, so the desktop that
//! writes a sentence and the walk that reads it for (`crates/alo-walking`)
//! cannot drift apart.
//!
//! The log is the journal, and on a walk machine the serial port carries it,
//! so these are the sentences a walk can check that an act happened, and what
//! the desktop answered. **They never carry the text alo typed**: only how many
//! bytes it was, because a text may be anything the person asked for, and the
//! journal is read by more than this machine's person.

use crate::{Opened, ToType, Typed};

/// The words every sentence about opening a file begins with.
pub const A_FILE_WAS_OPENED: &str = "alo asked to open a file in";

/// The words every sentence about typing, saving, or asking whether it may,
/// begins with.
pub const ALO_TYPED: &str = "alo asked to type into";

/// The sentence for one file alo asked to open in `application`, and what the
/// desktop answered.
#[must_use]
pub fn a_file_was_opened(application: &str, answered: Opened) -> String {
    format!(
        "{A_FILE_WAS_OPENED} {application}, and it was answered {}",
        answered.written()
    )
}

/// The sentence for one typing alo asked of the window in front: which kind
/// (`text`, `save`, `check`), the application, how many bytes, and what the
/// desktop answered.
#[must_use]
pub fn alo_typed(asked: &ToType, answered: Typed) -> String {
    let (what, bytes) = match asked {
        ToType::Text { bytes, .. } => ("text", *bytes),
        ToType::Save { .. } => ("save", 0),
        ToType::Check { .. } => ("check", 0),
    };
    format!(
        "{ALO_TYPED} {}: {what}, {bytes} bytes, and it was answered {}",
        asked.application().identifier(),
        answered.written()
    )
}

/// The words every sentence about the field alo was asked to type into
/// begins with.
pub const THE_FIELD_ASKED_FOR: &str = "alo was asked to type into the field";

/// **The field a typing names, and whether it was found** — the name exactly
/// as the call gave it, quoted, so the first walk past the keyboard shows
/// whether a model named the file (`hello.txt`) or something the editor does
/// not show (`Document`). A field's name is the name an application shows for
/// it, usually a file's; the text itself is never said.
#[must_use]
pub fn the_field_asked_for(application: &str, name: &str, found: bool) -> String {
    format!(
        "{THE_FIELD_ASKED_FOR} {name:?} in {application}, and it was {}",
        if found { "found" } else { "not found" }
    )
}

/// The words the sentence begins with when a window starts to wear alo's
/// presence, because alo acted in it for a task at work.
pub const ALO_WEARS_A_WINDOW: &str = "alo is at work in a window of";

/// The words the sentence begins with when that presence comes off: the turn
/// ended, or the window went.
pub const ALOS_WORK_IS_OVER: &str = "alo's work is over in a window of";

/// The sentence for a window of `application` starting to wear alo's
/// presence. Its application only, never its title, which may name the
/// person's document.
#[must_use]
pub fn alo_wears_a_window_of(application: &str) -> String {
    format!("{ALO_WEARS_A_WINDOW} {application}")
}

/// The sentence for alo's presence coming off a window of `application`.
#[must_use]
pub fn alos_work_is_over_in_a_window_of(application: &str) -> String {
    format!("{ALOS_WORK_IS_OVER} {application}")
}

/// The words every sentence about one of alo's acts in an application begins
/// with, said by the turn that carried it out.
pub const ALO_IN_AN_APPLICATION: &str = "alo in an application:";

/// What became of one act of alo's in an application, as the turn that
/// carried it out says it: done, or not done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InAnApplication {
    /// It was carried out.
    Done,
    /// A read was carried out and the windows showed some text.
    ReadWithText,
    /// A read was carried out and the windows showed no text.
    ReadWithNoText,
    /// A press was asked for and the application did not answer.
    Unanswered,
    /// A text was typed and read back exactly, and saved.
    TypedAndSaved,
    /// A text was typed and not saved.
    TypedAndNotSaved,
    /// It was refused, or the machine could not carry it out.
    NotDone,
}

impl InAnApplication {
    /// The words a walk reads for.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::ReadWithText => "done, and it showed text",
            Self::ReadWithNoText => "done, and it showed no text",
            Self::Unanswered => "done, and the application did not answer",
            Self::TypedAndSaved => "done, and saved",
            Self::TypedAndNotSaved => "done, and not saved",
            Self::NotDone => "not done",
        }
    }
}

/// The sentence for one act of alo's in `application`, by its verb's name.
/// **Never the text typed, the control's name or a window's title**, which may
/// name the person's document: only the verb, the application and the outcome.
#[must_use]
pub fn alo_in_an_application(verb: &str, application: &str, became: InAnApplication) -> String {
    format!(
        "{ALO_IN_AN_APPLICATION} {verb} in {application}, {}",
        became.written()
    )
}

/// The words the sentence begins with when the sheet asks the person to let
/// alo use an application (`docs/design/when-alo-asks-for-an-application.md`).
pub const A_SHEET_ASKS_FOR: &str = "the sheet asks the person to let alo use";

/// The sentence for the sheet asking about `application`, by its identifier.
/// **Never who asked or what for**: the application is all a walk needs.
#[must_use]
pub fn the_sheet_asks_for(application: &str) -> String {
    format!("{A_SHEET_ASKS_FOR} {application}")
}

/// The sentence when a grant made for *This task* is taken back because its
/// task ended — its window closed, or it was stopped.
pub const A_TASKS_GRANT_WENT: &str = "a grant made for a task that ended was taken back";

/// The sentence for a task's grant taken back.
#[must_use]
pub const fn a_tasks_grant_went() -> &'static str {
    A_TASKS_GRANT_WENT
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    /// **The sheet's sentence names the application, and only it.**
    #[test]
    fn the_sheet_names_the_application_it_asks_for() {
        let said = super::the_sheet_asks_for("org.gnome.Papers");
        assert!(said.starts_with(super::A_SHEET_ASKS_FOR), "{said}");
        assert!(said.ends_with(" org.gnome.Papers"), "{said}");
        assert_ne!(said, super::the_sheet_asks_for("org.mozilla.firefox"));
    }

    use alo_applications::Application;

    use super::*;

    /// **A sentence begins with the words a walk looks for, then the
    /// application, and ends with the answer's own word** — so a walk that
    /// reads `phrase` and `and_says` reads exactly what the desktop wrote.
    ///
    /// What would have to be true for this to pass while broken: the desktop
    /// writing its log with its own `format!` rather than these functions. The
    /// desktop and the shell call these by name, which `cargo` checks.
    #[test]
    fn each_sentence_names_the_application_and_the_answer() {
        let codium = Application::identified("com.vscodium.codium").unwrap();
        let opened = a_file_was_opened(codium.identifier(), Opened::InAlosCopy);
        assert!(opened.starts_with("alo asked to open a file in com.vscodium.codium,"));
        assert!(opened.ends_with("opened-in-alos-copy"));

        let typed = alo_typed(
            &ToType::Text {
                application: codium.clone(),
                bytes: 12,
            },
            Typed::Typed,
        );
        assert_eq!(
            typed,
            "alo asked to type into com.vscodium.codium: text, 12 bytes, and it was answered typed"
        );
        let saved = alo_typed(
            &ToType::Save {
                application: codium,
            },
            Typed::AskedToSave,
        );
        assert!(saved.ends_with(": save, 0 bytes, and it was answered asked-to-save"));

        // The field, quoted as the call gave it, and whether it was there.
        assert_eq!(
            the_field_asked_for("com.vscodium.codium", "Document", false),
            "alo was asked to type into the field \"Document\" in com.vscodium.codium, and it \
             was not found"
        );
        assert!(
            the_field_asked_for("com.vscodium.codium", "hello.txt", true).ends_with("was found")
        );
    }

    /// **The window alo works in is said by its application, when its presence
    /// goes on and when it comes off** — never by its title, which may name the
    /// person's document — so a walk can check the edge is drawn after the
    /// first line and gone after the second.
    ///
    /// What would have to be true for this to pass while broken: the shell
    /// writing either line with its own `format!`, which a search finds.
    #[test]
    fn the_presence_is_said_by_its_application_on_and_off() {
        let on = alo_wears_a_window_of("com.vscodium.codium");
        let off = alos_work_is_over_in_a_window_of("com.vscodium.codium");
        assert_eq!(on, "alo is at work in a window of com.vscodium.codium");
        assert_eq!(off, "alo's work is over in a window of com.vscodium.codium");
        assert!(on.starts_with(ALO_WEARS_A_WINDOW));
        assert!(off.starts_with(ALOS_WORK_IS_OVER));
        // Neither phrase is found inside the other's line.
        assert!(!off.contains(ALO_WEARS_A_WINDOW));
        assert!(!on.contains(ALOS_WORK_IS_OVER));
    }

    /// **An act in an application is said by its verb, its application and
    /// what became of it**, and an act not done never reads as one done.
    #[test]
    fn an_act_in_an_application_says_its_verb_its_application_and_its_outcome() {
        assert_eq!(
            alo_in_an_application(
                "accessible.read_window",
                "org.kde.dolphin",
                InAnApplication::ReadWithText
            ),
            "alo in an application: accessible.read_window in org.kde.dolphin, done, and it \
             showed text"
        );
        let not =
            alo_in_an_application("open_application", "org.kde.ark", InAnApplication::NotDone);
        assert!(not.ends_with(", not done"), "{not}");
        assert!(!not.contains(", done"), "{not}");
    }
}
