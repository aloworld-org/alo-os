//! **`accessible.type_text`**: one approved text typed into one field of an
//! application, read back, and saved only if it reads back exactly —
//! [ADR 0153](../../../docs/decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md).
//! A change, so it runs only after one approval of *type the text below into
//! “{name}” in {application}, and save it*, with the whole text beneath it.
//!
//! The order, and why each step is where it is:
//!
//! 1. `alo_adapters::Typing::of` — the grants asked again **now**, the
//!    application installed, no adapter of its own; refused before any tree.
//! 2. The tree opened for this one typing, and the field found again in it as
//!    the windows are now (`Typing::the_field`): one field of that name,
//!    never a password field, never a terminal.
//! 3. **Whose window it is**, asked of the compositor, which alone can tell
//!    alo's own copy from the person's window: on the direct road with
//!    `may-type-into`, which sends nothing; on the keyboard road by
//!    `type-into` itself. At the default only alo's copy is typed into.
//! 4. The text put in: through the tree where the field takes it
//!    (`EditableText`), otherwise as key presses the compositor delivers to
//!    that window alone — and for keys, only if the field has the keyboard.
//! 5. **Read back**, for up to [`READ_BACK_FOR`]: the approved text must be in
//!    the field exactly. Then `Ctrl+S`; otherwise nothing is saved and the
//!    person is told why.
//!
//! **One approval types once.** Nothing here loops over texts or fields: the
//! settings *each block on its own* and *everything for one request* (ADR 0143
//! Q1) are not built, and this is ADR 0143's default.
//!
//! **What is recorded.** A refusal before anything was typed is a refusal. Once
//! anything was typed it is a typing that **ran** — the change happened under
//! the approval — with what followed said beside it: saved, or not saved and
//! why. The record keeps the whole text, as every text argument is kept.

use std::time::{Duration, Instant};

use alo_acting::{ToType, Typed};
use alo_adapters::typing_words as words;
use alo_adapters::{NotWalked, TheField, TreeFault, Typing};
use alo_applications::Application;
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;
use alo_strings::Strings;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// How long a typed text is given to appear in the tree before it is read
/// back as not there: an application describes a change a moment after it
/// makes it.
pub const READ_BACK_FOR: Duration = Duration::from_secs(2);

/// How long between two reads back.
const BETWEEN_READS: Duration = Duration::from_millis(100);

/// One text alo typed into an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ATextTyped {
    /// The application.
    application: String,
    /// The field or document, by its name.
    name: String,
    /// How many characters were typed.
    characters: usize,
    /// Whether it was put in through the tree rather than as keys.
    through_the_tree: bool,
    /// Whether it read back exactly and was saved.
    saved: bool,
}

impl ATextTyped {
    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The field or document, by its name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// How many characters were typed.
    #[must_use]
    pub const fn characters(&self) -> usize {
        self.characters
    }

    /// Whether it was put in through the tree rather than as key presses.
    #[must_use]
    pub const fn through_the_tree(&self) -> bool {
        self.through_the_tree
    }

    /// Whether it read back exactly and was saved.
    #[must_use]
    pub const fn saved(&self) -> bool {
        self.saved
    }
}

/// The words for what the compositor answered, when it was not a yes.
fn the_word_for(typed: Typed) -> (alo_strings::Word, Option<String>) {
    match typed {
        Typed::NotInFront => (words::NOT_IN_FRONT, None),
        Typed::NotAlosCopy => (words::NOT_ALOS_COPY, None),
        Typed::CannotType(character) => (
            words::CANNOT_TYPE,
            Some(format!("U+{:04X}", u32::from(character))),
        ),
        Typed::Typed | Typed::AskedToSave | Typed::MayType | Typed::NoKeyboard => {
            (words::NO_KEYBOARD, None)
        }
    }
}

/// Type one approved text, once.
pub(crate) fn typing_into_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<ATextTyped, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    let typing = match Typing::of(
        authorised,
        machine.the_adapters(),
        grants,
        machine.the_applications_installed(),
        strings,
    ) {
        Ok(typing) => typing,
        Err(refused) => return refusing(refused),
    };
    let Ok(application) = Application::identified(typing.application()) else {
        return refusing(typing.refused(words::NO_KEYBOARD, None, strings));
    };

    // Opened now, for this one typing, and dropped when it ends.
    let tree = match machine.the_windows_on_the_screen() {
        Some(windows) => windows.opening(),
        None => Err(TreeFault::NotThere),
    };
    let tree = match tree {
        Ok(tree) => tree,
        Err(fault) => {
            let said = NotWalked::Tree(fault).said(typing.application(), None, strings);
            return refusing(Refused::worded_elsewhere(call, said));
        }
    };
    let field = match typing.the_field(tree.as_ref(), strings) {
        Ok(field) => field,
        Err(refused) => return refusing(refused),
    };

    let Some(door) = machine.at_the_compositors_door() else {
        return refusing(typing.refused(words::NO_KEYBOARD, None, strings));
    };

    // Steps 3 and 4: whose window, then the text in.
    if field.takes_text {
        let may = asking(
            &mut *door,
            ToType::Check {
                application: application.clone(),
            },
            "",
        );
        if may != Typed::MayType {
            let (word, more) = the_word_for(may);
            return refusing(typing.refused(
                word,
                more.map(|value| (words::CHARACTER, value)),
                strings,
            ));
        }
        if let Err(refused) = typing.put_in(tree.as_ref(), &field, strings) {
            return refusing(refused);
        }
    } else {
        if !field.has_the_keyboard {
            return refusing(typing.refused(words::NOT_IN_FRONT, None, strings));
        }
        let typed = asking(
            &mut *door,
            ToType::Text {
                application: application.clone(),
                bytes: typing.text().len(),
            },
            typing.text(),
        );
        if typed != Typed::Typed {
            let (word, more) = the_word_for(typed);
            return refusing(typing.refused(
                word,
                more.map(|value| (words::CHARACTER, value)),
                strings,
            ));
        }
    }

    // Step 5: read back, then save — or say why not.
    let read_back = read_back(&typing, tree.as_ref(), &field, strings);
    let (saved, told) = match read_back {
        Err(refused) => (false, Some(refused.said(strings))),
        Ok(()) => {
            let saving = asking(&mut *door, ToType::Save { application }, "");
            if saving == Typed::AskedToSave {
                (true, None)
            } else {
                let (word, more) = the_word_for(saving);
                let not_saved =
                    typing.refused(word, more.map(|value| (words::CHARACTER, value)), strings);
                (false, Some(not_saved.said(strings)))
            }
        }
    };
    drop(tree);

    let typed = ATextTyped {
        application: typing.application().to_owned(),
        name: typing.name().to_owned(),
        characters: typing.text().chars().count(),
        through_the_tree: field.takes_text,
        saved,
    };
    let entry = Entry::ran(&typing.into_authorised(), strings);
    let entry = match told {
        Some(said) => entry.telling(&[said]),
        None => entry,
    };
    (entry, Ok(typed))
}

/// What the compositor answered one typing — a door that could not be reached
/// answers as one with no keyboard, and nothing is typed.
fn asking(door: &mut dyn crate::TheDoorTheCompositorServes, asked: ToType, text: &str) -> Typed {
    door.typing(&asked, text).unwrap_or(Typed::NoKeyboard)
}

/// Read the field back until the text is there, for at most
/// [`READ_BACK_FOR`].
fn read_back(
    typing: &Typing,
    tree: &dyn alo_adapters::AccessibilityTree,
    field: &TheField,
    strings: &Strings,
) -> Result<(), Refused> {
    let until = Instant::now() + READ_BACK_FOR;
    loop {
        match typing.read_back(tree, field, strings) {
            Ok(()) => return Ok(()),
            Err(refused) if Instant::now() >= until => return Err(refused),
            Err(_) => std::thread::sleep(BETWEEN_READS),
        }
    }
}
