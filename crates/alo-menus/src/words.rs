//! Every string this crate can say, and the English beside each one.
//!
//! Fourteen rows and one refusal. A context menu is nearly all words — it is a
//! list of sentences and nothing else — so this file is most of what a person
//! meets of this crate, and every one of them is translated before it is shown.
//!
//! The shape is `alo-shortcuts`' and is copied rather than re-decided.
//!
//! # The row that reaches the agent says what it does
//!
//! [`ASK_THE_AGENT_ABOUT_THIS`] is the only entry that offers anything to the
//! agent, and its English says so in the row itself rather than in a note
//! somebody could have left out. A person choosing it has read what it does;
//! that is the whole of *no menu entry sends anything to the agent without the
//! person choosing the entry that says so*.
//!
//! # No sentence here names a file
//!
//! Not *Rename “march.pdf”* but *Rename*. The menu is drawn over the thing it is
//! about and a person can see which one it is; a row that repeated the name
//! would be a gap to fill, in twenty-four languages, for a word they are
//! already looking at.

use alo_strings::Vocabulary;

/// One string a crate can say.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The rows — [`crate::Action`]. Each is what choosing it does, in the fewest
// words that are still a sentence a person can act on.
// ---------------------------------------------------------------------------

/// [`crate::Action::Open`].
pub const OPEN: Word = Word::saying("menus.action.open", "Open")
    .noting("A row in the menu that appears when a person right-clicks a file: open the file.");

/// [`crate::Action::OpenWith`].
pub const OPEN_WITH: Word = Word::saying("menus.action.open-with", "Open with…").noting(
    "A row in a file's menu: choose which application opens this file, rather than the one that \
     usually does. The three dots mean that choosing it asks something before anything happens.",
);

/// [`crate::Action::Rename`].
pub const RENAME: Word = Word::saying("menus.action.rename", "Rename")
    .noting("A row in a file's menu: give the file another name.");

/// [`crate::Action::MoveToTheWastebasket`].
pub const MOVE_TO_THE_WASTEBASKET: Word = Word::saying(
    "menus.action.move-to-the-wastebasket",
    "Move to wastebasket",
)
.noting(
    "A row in a file's menu: put the file in the wastebasket, where it can still be got back. \
         Translators: the container deleted files wait in — bin, trash, recycle bin, whatever it \
         is called where this language is spoken.",
);

/// [`crate::Action::Copy`].
pub const COPY: Word = Word::saying("menus.action.copy", "Copy")
    .noting("A row in a menu: copy what is under the pointer, so it can be pasted somewhere else.");

/// [`crate::Action::Cut`].
pub const CUT: Word = Word::saying("menus.action.cut", "Cut").noting(
    "A row in a menu: take what is under the pointer to be moved somewhere else. It stays where \
     it is until it is pasted.",
);

/// [`crate::Action::Paste`].
pub const PASTE: Word = Word::saying("menus.action.paste", "Paste")
    .noting("A row in a menu: put what was last copied or cut here.");

/// [`crate::Action::SelectAll`].
pub const SELECT_ALL: Word = Word::saying("menus.action.select-all", "Select all")
    .noting("A row in a menu over text: select everything there is to select here.");

/// [`crate::Action::CloseTheWindow`].
pub const CLOSE_THE_WINDOW: Word = Word::saying("menus.action.close-the-window", "Close")
    .noting("A row in a window's menu: close the window.");

/// [`crate::Action::TheLeftHalf`].
pub const THE_LEFT_HALF: Word = Word::saying("menus.action.the-left-half", "Left half of screen")
    .noting(
        "A row in a window's menu: give this window the left half of the screen, with whatever is \
         beside it taking the right half.",
    );

/// [`crate::Action::TheRightHalf`].
pub const THE_RIGHT_HALF: Word =
    Word::saying("menus.action.the-right-half", "Right half of screen")
        .noting("The other half — see the note on menus.action.the-left-half.");

/// [`crate::Action::ChangeTheBackground`].
pub const CHANGE_THE_BACKGROUND: Word = Word::saying(
    "menus.action.change-the-background",
    "Change desktop background",
)
.noting(
    "A row in the menu that appears when a person right-clicks the desktop itself: change the \
     picture behind everything.",
);

/// [`crate::Action::OpenSettings`].
pub const OPEN_SETTINGS: Word = Word::saying("menus.action.open-settings", "Settings").noting(
    "A row in the desktop's menu: open the machine's settings. Translators: the same word this \
     language uses for the settings application everywhere else in alo OS.",
);

/// [`crate::Action::AskTheAgentAboutThis`].
pub const ASK_THE_AGENT_ABOUT_THIS: Word = Word::saying(
    "menus.action.ask-the-agent-about-this",
    "Ask the agent about this",
)
.noting(
    "The one row in any menu that involves the assistant built into the machine, and it is \
         only ever shown over a file or over text a person has selected. Choosing it offers that \
         one thing to the agent for one question; it does not hand over the folder it is in, and \
         nothing is offered unless this row is chosen. On a machine with no agent the row is not \
         there at all.",
);

// ---------------------------------------------------------------------------
// Why nothing happened — [`crate::NotChosen`].
// ---------------------------------------------------------------------------

/// [`crate::NotChosen::NotOnThisMenu`].
pub const NOT_ON_THIS_MENU: Word = Word::saying(
    "menus.refused.not-on-this-menu",
    "that is not something this menu offers, so nothing has happened",
)
.noting(
    "Said when the thing a menu was opened over has changed — a file deleted, a window closed — \
     between the menu appearing and a person choosing a row in it. alo OS does the action to \
     nothing rather than to whatever is there now.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 15] = [
    OPEN,
    OPEN_WITH,
    RENAME,
    MOVE_TO_THE_WASTEBASKET,
    COPY,
    CUT,
    PASTE,
    SELECT_ALL,
    CLOSE_THE_WINDOW,
    THE_LEFT_HALF,
    THE_RIGHT_HALF,
    CHANGE_THE_BACKGROUND,
    OPEN_SETTINGS,
    ASK_THE_AGENT_ABOUT_THIS,
    NOT_ON_THIS_MENU,
];

/// Why this crate's own words could not be declared.
///
/// Nothing in the list above can cause one; the tests at the bottom of this file
/// are what say so.
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
pub fn menu_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "menus", "{}", word.named());
        }
    }

    /// No two words share a key, and the whole list declares.
    #[test]
    fn the_whole_list_declares_and_no_two_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let vocabulary = menu_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        let again = declare_into(&mut menu_words().unwrap()).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// Every word carries a note, and none has a gap: nothing here is filled in.
    #[test]
    fn every_word_carries_a_note_and_names_nothing() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{}",
                word.named()
            );
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{}",
                word.named()
            );
        }
    }
}
