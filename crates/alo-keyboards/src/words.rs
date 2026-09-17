//! Every string this crate can say, and the English beside each one.
//!
//! Three kinds: what each compose key is called, why a keyboard was not added
//! or removed, and what is said about a person's own file or the machine's
//! rented keyboard data.
//!
//! The shape is `alo-shortcuts`' and is copied rather than re-decided.
//!
//! # No sentence here names a layout, a keysym, XKB or an input-method framework
//!
//! Not `us(intl)`, not `dead_diaeresis`, not the framework's name. A person
//! chose a row that said *Ελληνικά* and the sentence they get back says
//! *that keyboard*; the row is marked beside it through
//! [`crate::Refused::keyboard`]. This is the plan's own constraint for these
//! crates, and it is also the difference between a product that rents its
//! parts and a product that is visibly made of them.
//!
//! What a sentence here does name is a **language, in its own language** —
//! `Ελληνικά`, `日本語` when somebody contributes it — because that is the one
//! word in the sentence a person definitely recognises.

use alo_strings::Vocabulary;

/// One string a crate can say.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Which key composes — [`crate::ComposeKey`]. Rows in a picker, so each is a
// name rather than a sentence.
// ---------------------------------------------------------------------------

/// [`crate::ComposeKey::None`].
pub const COMPOSE_NONE: Word = Word::saying("keyboards.compose.none", "No compose key").noting(
    "The row a person chooses when they do not want a compose key at all, which is what alo OS \
     ships. A compose key is one key that means \"the next few keys make one letter\". This row \
     is a choice and not a fault.",
);

/// [`crate::ComposeKey::Menu`].
pub const COMPOSE_MENU: Word = Word::saying("keyboards.compose.menu", "The Menu key").noting(
    "The key printed with a small list, between the right-hand Alt and Ctrl on most keyboards. \
     Named the way the keycap is printed in the person's part of the world, if it is printed \
     with a word there.",
);

/// [`crate::ComposeKey::CapsLock`].
pub const COMPOSE_CAPS_LOCK: Word = Word::saying(
    "keyboards.compose.caps-lock",
    "Caps Lock, which then stops locking capitals",
)
.noting(
    "The second clause matters and is not a warning to be dropped: choosing this row means \
             Caps Lock no longer locks capital letters. Caps Lock is often printed on the keycap \
             in English even on keyboards for other languages.",
);

/// [`crate::ComposeKey::RightCtrl`].
pub const COMPOSE_RIGHT_CTRL: Word = Word::saying(
    "keyboards.compose.right-ctrl",
    "The right-hand Ctrl key",
)
.noting(
    "Ctrl is printed on the keycap, and on a German keyboard it is printed Strg. \"Right-hand\" \
         because the left one is left alone.",
);

/// [`crate::ComposeKey::RightSuper`].
pub const COMPOSE_RIGHT_SUPER: Word = Word::saying(
    "keyboards.compose.right-super",
    "The right-hand key with the Windows logo on it",
)
.noting(
    "Called Super in software and printed with a logo rather than a word, so it is described by \
     what is printed on it. On a keyboard where that key is printed differently, describe it the \
     way it is printed there.",
);

/// [`crate::ComposeKey::ScrollLock`].
pub const COMPOSE_SCROLL_LOCK: Word = Word::saying("keyboards.compose.scroll-lock", "Scroll Lock")
    .noting(
        "A key on full-size keyboards that does nothing on a machine made this century, which is \
         what makes it a good compose key. Often printed in English on keyboards for other \
         languages.",
    );

// ---------------------------------------------------------------------------
// Why nothing changed — [`crate::Refused`]. Each says what did not happen,
// because a person reading it has just tried to change something.
// ---------------------------------------------------------------------------

/// [`crate::Refused::NotAKeyboardWeHave`].
pub const NOT_A_KEYBOARD_WE_HAVE: Word = Word::saying(
    "keyboards.refused.not-one-we-have",
    "this machine does not have that keyboard, so nothing has been added",
)
.noting(
    "Said when something asks for a keyboard that is not among the ones installed on the machine. \
     A person choosing from the list never meets it; it is what a settings file somebody edited \
     by hand, or an older release, can produce.",
);

/// [`crate::Refused::AlreadyOneOfYours`].
pub const ALREADY_ONE_OF_YOURS: Word = Word::saying(
    "keyboards.refused.already-yours",
    "that keyboard is already one of yours, so nothing has been added",
)
.noting("Said when a person adds a keyboard they already have. Nothing is wrong.");

/// [`crate::Refused::NotOneOfYours`].
pub const NOT_ONE_OF_YOURS: Word = Word::saying(
    "keyboards.refused.not-yours",
    "that keyboard is not one of yours, so nothing has been removed",
)
.noting("Said when something asks to remove a keyboard the person does not have.");

/// [`crate::Refused::TheLastKeyboard`].
pub const THE_LAST_KEYBOARD: Word = Word::saying(
    "keyboards.refused.the-last-one",
    "this is your only keyboard and you need one to type, so it has been kept — add another \
     first, then remove this one",
)
.noting(
    "Said when a person tries to remove their last keyboard. The last clause is the way on and is \
     not optional: a refusal that does not say how to get what somebody wanted is a wall.",
);

/// [`crate::Refused::NoKeyboardForThisLanguage`].
pub const NO_KEYBOARD_FOR_THIS_LANGUAGE: Word = Word::saying(
    "keyboards.refused.no-keyboard-for-this-language",
    "alo OS does not yet know which keyboard {language} is typed on, so nothing has been added — \
     you can still add the keyboard yourself from the list",
)
.noting(
    "{language} is the name of a language written in that language itself — Ελληνικά, Gaeilge — \
     and is never translated. \"alo OS\" is the product's name and is never translated. The last \
     clause matters: the whole list of keyboards is still there, and this is alo OS admitting it \
     has no suggestion rather than refusing to help.",
);

/// [`crate::Refused::TypedOnAKeyboard`].
pub const TYPED_ON_A_KEYBOARD: Word = Word::saying(
    "keyboards.refused.typed-on-a-keyboard",
    "{language} is typed on a keyboard rather than by choosing characters as you type, so nothing \
     has been added — add the keyboard for it instead",
)
.noting(
    "{language} is the name of a language written in that language itself and is never \
     translated. Said when a person looks for Greek or Bulgarian among the ways of typing Chinese, \
     Japanese and Korean. \"Choosing characters as you type\" is what those three need and is \
     described rather than named, because the name of the software that does it is not something \
     a person should have to learn.",
);

/// [`crate::Refused::AlreadyTyping`].
pub const ALREADY_TYPING: Word = Word::saying(
    "keyboards.refused.already-typing",
    "you can already type that way, so nothing has been added",
)
.noting("Said when a person adds a way of typing they have already added. Nothing is wrong.");

// ---------------------------------------------------------------------------
// What is said about the machine's own keyboard data.
// ---------------------------------------------------------------------------

/// [`crate::NotRented`] — the machine's list of keyboards.
pub const KEYBOARDS_NOT_THERE: Word = Word::saying(
    "keyboards.machine.not-there",
    "the list of keyboards this machine has could not be read from {path}, so no keyboard can be \
     added — the keyboard you are typing on is unchanged",
)
.noting(
    "{path} is a file on this machine and is never translated. A fault in the machine rather than \
     anything the person did: the keyboard data is part of alo OS and should always be there. The \
     last clause is what they need to know — typing still works.",
);

/// [`crate::NotComposed`] — the table that turns a sequence of keys into a letter.
pub const COMPOSING_NOT_THERE: Word = Word::saying(
    "keyboards.machine.nothing-composes",
    "the table that turns a sequence of keys into one letter could not be read from {path}, so \
     accents typed with a dead key or a compose key will not appear — the letters printed on your \
     keyboard are unaffected",
)
.noting(
    "{path} is a file on this machine and is never translated. A dead key is a key that makes no \
     letter on its own and changes the next one. The last clause tells a person what still works, \
     which is most of their keyboard.",
);

// ---------------------------------------------------------------------------
// What is said about the person's own file — the shape `alo-shortcuts` keeps,
// about this crate's file.
// ---------------------------------------------------------------------------

/// The file is there and could not be opened.
pub const KEPT_NOT_READ: Word = Word::saying(
    "keyboards.kept.not-read",
    "your keyboard settings at {path} could not be read, so you are typing on the keyboard alo OS \
     ships",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk or a permission rather than \
     anything a person typed. \"alo OS\" is the product's name and is never translated.",
);

/// The file is there and is not keyboard settings.
pub const KEPT_NOT_UNDERSTOOD: Word = Word::saying(
    "keyboards.kept.not-understood",
    "your keyboard settings at {path} are not settings alo OS can read, so nothing in the file has \
     been used and you are typing on the keyboard alo OS ships",
)
.noting(
    "{path} is a file on this machine and is never translated. The important clause is that \
     nothing in the file was used: alo OS did not take the half it understood.",
);

/// The file stopped being settings at a line.
pub const KEPT_NOT_UNDERSTOOD_AT: Word = Word::saying(
    "keyboards.kept.not-understood-at",
    "your keyboard settings at {path} stop making sense at line {line}, so nothing in the file has \
     been used and you are typing on the keyboard alo OS ships",
)
.noting(
    "{path} is a file on this machine and is never translated. {line} is a plain whole number, \
     counted from one the way a text editor counts lines.",
);

/// The file says it is a shape this alo OS does not read.
pub const KEPT_ANOTHER_FORMAT: Word = Word::saying(
    "keyboards.kept.another-format",
    "your keyboard settings at {path} were written for a different alo OS than this one, so \
     nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and is never translated. Most often a newer alo OS wrote the \
     file; reading it part-way could leave a person with a keyboard they cannot type on.",
);

/// The file names something that is not a keyboard setting.
pub const KEPT_UNKNOWN_KEY: Word = Word::saying(
    "keyboards.kept.unknown-key",
    "your keyboard settings at {path} say {key}, which is not something alo OS can change about \
     keyboards, so nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and {key} is a word as it was typed into it; neither is \
     translated. {key} is a name in the file, not a key on the keyboard — it is named because it \
     is what a person has to find in the file to fix it.",
);

/// The disk would not take the changed file.
pub const KEPT_NOT_WRITTEN: Word = Word::saying(
    "keyboards.kept.not-written",
    "your keyboard settings at {path} could not be written, so nothing about your keyboards has \
     changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A full disk or a folder the person \
     cannot write. The second clause is what they act on: the file is exactly as it was.",
);

/// The change could not be written as a file this alo OS reads back.
pub const KEPT_NOT_EXPRESSIBLE: Word = Word::saying(
    "keyboards.kept.not-expressible",
    "this alo OS could not write that change into keyboard settings at {path} it can read back \
     again, so nothing about your keyboards has changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A fault in alo OS rather than \
     anything the person did, said plainly: the change was refused before the file was touched.",
);

/// The file a change would replace is there and did not read, so it was kept.
pub const KEPT_NOT_REPLACED: Word = Word::saying(
    "keyboards.kept.not-replaced",
    "your keyboard settings at {path} could not be read, so that change has not been written over \
     them and nothing about your keyboards has changed — correct the file, or put keyboards back \
     as alo OS ships them",
)
.noting(
    "{path} is a file on this machine and is never translated. Said when a person changes \
     something in Settings while the file, most often one they edited by hand, does not read: alo \
     OS keeps their file rather than replacing it, so a typing mistake in it is not lost. The last \
     clause gives the two ways on. \"alo OS\" is the product's name and is never translated.",
);

// ---------------------------------------------------------------------------
// Switching keyboards.
// ---------------------------------------------------------------------------

/// Something else is on the chord that switches keyboards.
pub const SWITCH_SHADOWED: Word = Word::saying(
    "keyboards.switch.shadowed",
    "{chord} switches between your keyboards, and you have also given it to {action}, so it will \
     do that instead — change one of them to switch keyboards with it again",
)
.noting(
    "{chord} is a combination of keys, written the way this machine writes them, such as \
     Alt+Space. {action} is the name of something the system does, taken from the list of \
     shortcuts and already translated when it arrives here.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 24] = [
    COMPOSE_NONE,
    COMPOSE_MENU,
    COMPOSE_CAPS_LOCK,
    COMPOSE_RIGHT_CTRL,
    COMPOSE_RIGHT_SUPER,
    COMPOSE_SCROLL_LOCK,
    NOT_A_KEYBOARD_WE_HAVE,
    ALREADY_ONE_OF_YOURS,
    NOT_ONE_OF_YOURS,
    THE_LAST_KEYBOARD,
    NO_KEYBOARD_FOR_THIS_LANGUAGE,
    TYPED_ON_A_KEYBOARD,
    ALREADY_TYPING,
    KEYBOARDS_NOT_THERE,
    COMPOSING_NOT_THERE,
    KEPT_NOT_READ,
    KEPT_NOT_UNDERSTOOD,
    KEPT_NOT_UNDERSTOOD_AT,
    KEPT_ANOTHER_FORMAT,
    KEPT_UNKNOWN_KEY,
    KEPT_NOT_WRITTEN,
    KEPT_NOT_EXPRESSIBLE,
    KEPT_NOT_REPLACED,
    SWITCH_SHADOWED,
];

/// Why this crate's own words could not be declared.
///
/// Nothing in the list above can cause one; the tests at the bottom of this
/// file are what say so.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn keyboard_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every key written here is a key by the rule every other key is held to.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "keyboards", "{}", word.named());
        }
    }

    /// No two words share a key, and the whole list declares.
    #[test]
    fn the_whole_list_declares_and_no_two_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let vocabulary = keyboard_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        let again = declare_into(&mut keyboard_words().unwrap()).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// Every word carries a note, and only the ones that name something have a
    /// gap in them.
    #[test]
    fn every_word_carries_a_note_and_the_gaps_are_the_ones_meant() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{}",
                word.named()
            );
        }
        for (word, gap) in [
            (NO_KEYBOARD_FOR_THIS_LANGUAGE, "language"),
            (TYPED_ON_A_KEYBOARD, "language"),
            (KEYBOARDS_NOT_THERE, "path"),
            (COMPOSING_NOT_THERE, "path"),
            (KEPT_NOT_UNDERSTOOD_AT, "line"),
            (KEPT_UNKNOWN_KEY, "key"),
            (SWITCH_SHADOWED, "chord"),
            (SWITCH_SHADOWED, "action"),
        ] {
            assert!(word.phrase().unwrap().source().has(gap), "{}", word.named());
        }
        for word in [
            COMPOSE_NONE,
            COMPOSE_MENU,
            THE_LAST_KEYBOARD,
            ALREADY_ONE_OF_YOURS,
            NOT_A_KEYBOARD_WE_HAVE,
        ] {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{}",
                word.named()
            );
        }
    }

    /// **No sentence this crate can say names a rented part.** Not a layout
    /// code, not a keysym, not XKB, not the input-method framework — the plan's
    /// own constraint, held here over every string at once rather than in the
    /// one file that happens to have been reviewed.
    #[test]
    fn nothing_this_crate_says_names_a_rented_part() {
        for word in EVERY_WORD {
            let said = word.says().to_ascii_lowercase();
            for named in [
                "xkb",
                "keysym",
                "ibus",
                "mozc",
                "libpinyin",
                "hangul",
                "chewing",
                "xkeyboard",
                "multi_key",
                "dead_",
                "libinput",
                "us(intl)",
                "grp:",
                "compose:",
            ] {
                assert!(!said.contains(named), "{} says {named}", word.named());
            }
        }
    }
}
