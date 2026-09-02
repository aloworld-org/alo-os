//! Every string this crate can say, and the English beside each one.
//!
//! A person opens the shortcuts panel and reads almost nothing but this file:
//! what each shortcut does, what the keys are called, and the three sentences
//! that come back when a combination cannot be a shortcut. `CLAUDE.md` calls
//! hardcoded English a bug, and this is where the file half's answer to that
//! (item 9b) reaches the keyboard.
//!
//! The shape is `alo-files`' and is copied rather than re-decided: constants
//! under one area, `alo_strings::Word` because these are literals in this file,
//! and a test at the bottom putting every one of them back through
//! `Key::named`.
//!
//! # What is not here, and why that is the interesting half
//!
//! **A key that prints a character is not a string.** [`crate::Key::Q`] is
//! shown as `Q`, [`crate::Key::Digit7`] as `7`, [`crate::Key::Comma`] as `,`,
//! and none of the forty-one keys like them is in the list below. That is this
//! crate's own doctrine held to: a key is *the one printed on the person's own
//! keyboard*, so on a French keyboard `Super+Q` is the key marked Q — and a
//! translator who rendered `Q` as `Й` would be naming a **position** on a
//! keyboard, which is the model [`crate::key`] exists to reject. `F1` is the
//! same: it is marked F1 on every keyboard in the union.
//!
//! What *is* here is the sixteen keys that print a word — Space, Enter, Page
//! Up, the four arrows — because those are exactly the ones whose printing
//! changes with the keyboard: a German keyboard says *Entf* where an English one
//! says Delete, and *Bild ↑* where it says Page Up. So the split is not
//! tidiness. It is the difference between what a keyboard prints everywhere and
//! what it prints in one country, and only the second is a translator's.
//!
//! The cost of the other choice is what settles it: declaring all sixty-nine
//! would put forty-one rows reading `A`, `B`, `C` in front of a translator, and
//! would make `alo_strings::Strings::unanswered` — *what a release note has to
//! count* — report forty-one strings nobody should ever translate.
//!
//! # A note is part of the string
//!
//! A translator works alone, in a language nobody here reads, with no keyboard
//! panel in front of them. *Left* is an arrow key and not a direction, *the
//! agent* is the one built into alo OS and not a person, and *next window* and
//! *next application* are two different shortcuts that one word in some
//! languages would collapse into one. Where the sentence cannot be translated
//! from its own words, the note says so.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Lifted into `alo-strings` by item 9d, when `alo-appearance` would have been
/// the third crate to write the same four fields. Re-exported here because this
/// crate's own files, and the tests that read its list, name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What a shortcut does — [`crate::Action`]. These are rows in a list, read by
// somebody looking for the one they want to change.
// ---------------------------------------------------------------------------

/// What [`crate::Action::TheAgent`] does.
pub const THE_AGENT: Word = Word::saying("shortcuts.action.the-agent", "Ask the agent").noting(
    "The agent is the assistant built into alo OS, not a person. Every string here is a row \
         in a list of shortcuts and names what the key does.",
);

/// What [`crate::Action::Launcher`] does.
pub const LAUNCHER: Word = Word::saying("shortcuts.action.launcher", "Open the launcher")
    .noting("The launcher is where a person finds and starts an application.");

/// What [`crate::Action::CloseWindow`] does.
pub const CLOSE_WINDOW: Word = Word::saying("shortcuts.action.close-window", "Close the window");

/// What [`crate::Action::MinimiseWindow`] does.
pub const MINIMISE_WINDOW: Word =
    Word::saying("shortcuts.action.minimise-window", "Minimise the window")
        .noting("The window is put out of the way without being closed.");

/// What [`crate::Action::MaximiseWindow`] does.
pub const MAXIMISE_WINDOW: Word = Word::saying(
    "shortcuts.action.maximise-window",
    "Maximise the window, or put it back",
)
.noting(
    "One shortcut does both: it makes the window fill the screen, and pressing it again puts the \
     window back to the size it was.",
);

/// What [`crate::Action::SnapLeft`] does.
pub const SNAP_LEFT: Word = Word::saying(
    "shortcuts.action.snap-left",
    "Put the window on the left half",
)
.noting("The window is made to fill exactly the left half of the screen.");

/// What [`crate::Action::SnapRight`] does.
pub const SNAP_RIGHT: Word = Word::saying(
    "shortcuts.action.snap-right",
    "Put the window on the right half",
)
.noting("The window is made to fill exactly the right half of the screen.");

/// What [`crate::Action::NextWindow`] does.
pub const NEXT_WINDOW: Word = Word::saying("shortcuts.action.next-window", "Next window").noting(
    "A window is one window; an application may have several open at once. These two shortcuts \
     and the two about applications are four different things, and a language with one word for \
     both needs four rows a person can still tell apart.",
);

/// What [`crate::Action::PreviousWindow`] does.
pub const PREVIOUS_WINDOW: Word =
    Word::saying("shortcuts.action.previous-window", "Previous window")
        .noting("The window before this one, in the order they were last used.");

/// What [`crate::Action::NextApplication`] does.
pub const NEXT_APPLICATION: Word =
    Word::saying("shortcuts.action.next-application", "Next application").noting(
        "An application, not one of its windows — see the note on \
         shortcuts.action.next-window.",
    );

/// What [`crate::Action::PreviousApplication`] does.
pub const PREVIOUS_APPLICATION: Word = Word::saying(
    "shortcuts.action.previous-application",
    "Previous application",
)
.noting("An application, not one of its windows.");

// ---------------------------------------------------------------------------
// What is held down — [`crate::Modifier`]. Four words, and three of them are
// printed differently on the keyboards this will run on.
// ---------------------------------------------------------------------------

/// What [`crate::Modifier::Super`] is called.
pub const SUPER: Word = Word::saying("shortcuts.modifier.super", "Super").noting(
    "The key between Ctrl and Alt. Software calls it Super; the keyboards most of your readers \
     have print a Windows logo and no word at all. Name it whatever a person in your language \
     would call that key.",
);

/// What [`crate::Modifier::Ctrl`] is called.
pub const CTRL: Word = Word::saying("shortcuts.modifier.ctrl", "Ctrl").noting(
    "The control key, printed Ctrl on English keyboards and Strg on German ones. Write what is \
     printed on the keyboards your readers have.",
);

/// What [`crate::Modifier::Alt`] is called.
pub const ALT: Word = Word::saying("shortcuts.modifier.alt", "Alt")
    .noting("Printed Alt on almost every keyboard; leave it alone unless yours says otherwise.");

/// What [`crate::Modifier::Shift`] is called.
pub const SHIFT: Word = Word::saying("shortcuts.modifier.shift", "Shift").noting(
    "The key that makes a letter a capital, printed as an upward arrow on many keyboards and as \
     Maj on French ones. Write the word your readers use for it.",
);

// ---------------------------------------------------------------------------
// The sixteen keys that print a word rather than a character — [`crate::Key`].
// The other forty-one print what they are called and are not here; the module
// documentation says why.
// ---------------------------------------------------------------------------

/// What [`crate::Key::Space`] is called.
pub const SPACE: Word = Word::saying("shortcuts.key.space", "Space")
    .noting("The long key at the bottom, which types a space. It is usually printed with nothing.");

/// What [`crate::Key::Tab`] is called.
pub const TAB: Word = Word::saying("shortcuts.key.tab", "Tab").noting(
    "The key that moves to the next field or column, printed with two arrows on many \
             keyboards.",
);

/// What [`crate::Key::Enter`] is called.
pub const ENTER: Word = Word::saying("shortcuts.key.enter", "Enter").noting(
    "The large key that confirms something or starts a new line. Called Return, Intro or Invio \
     depending on the keyboard.",
);

/// What [`crate::Key::Escape`] is called.
pub const ESCAPE: Word = Word::saying("shortcuts.key.escape", "Escape")
    .noting("The key at the top left that cancels or backs out. Usually printed Esc.");

/// What [`crate::Key::Backspace`] is called.
pub const BACKSPACE: Word = Word::saying("shortcuts.key.backspace", "Backspace")
    .noting("The key that removes the character *before* the cursor. Not the same key as Delete.");

/// What [`crate::Key::Delete`] is called.
pub const DELETE: Word = Word::saying("shortcuts.key.delete", "Delete").noting(
    "The key that removes the character *after* the cursor, printed Entf on German keyboards and \
     Supr on Spanish ones. Not the same key as Backspace.",
);

/// What [`crate::Key::Insert`] is called.
pub const INSERT: Word = Word::saying("shortcuts.key.insert", "Insert")
    .noting("The key beside Delete, printed Einfg on German keyboards.");

/// What [`crate::Key::Home`] is called.
pub const HOME: Word = Word::saying("shortcuts.key.home", "Home").noting(
    "The key that goes to the beginning of a line or a document — not the person's home folder, \
     and not a house. Printed Pos1 on German keyboards.",
);

/// What [`crate::Key::End`] is called.
pub const END: Word = Word::saying("shortcuts.key.end", "End")
    .noting("The key that goes to the end of a line or a document, opposite the Home key.");

/// What [`crate::Key::PageUp`] is called.
pub const PAGE_UP: Word = Word::saying("shortcuts.key.page-up", "Page Up")
    .noting("The key that moves a whole screen upwards, printed Bild ↑ on German keyboards.");

/// What [`crate::Key::PageDown`] is called.
pub const PAGE_DOWN: Word = Word::saying("shortcuts.key.page-down", "Page Down")
    .noting("The key that moves a whole screen downwards, printed Bild ↓ on German keyboards.");

/// What [`crate::Key::Left`] is called.
pub const LEFT: Word = Word::saying("shortcuts.key.left", "Left").noting(
    "The arrow key pointing left, not the word for the direction on its own. If your language \
     would be unclear, say arrow: this is a row in a list of keys.",
);

/// What [`crate::Key::Right`] is called.
pub const RIGHT: Word = Word::saying("shortcuts.key.right", "Right")
    .noting("The arrow key pointing right — see the note on shortcuts.key.left.");

/// What [`crate::Key::Up`] is called.
pub const UP: Word = Word::saying("shortcuts.key.up", "Up")
    .noting("The arrow key pointing up — see the note on shortcuts.key.left.");

/// What [`crate::Key::Down`] is called.
pub const DOWN: Word = Word::saying("shortcuts.key.down", "Down")
    .noting("The arrow key pointing down — see the note on shortcuts.key.left.");

/// What [`crate::Key::Print`] is called.
pub const PRINT: Word = Word::saying("shortcuts.key.print", "Print").noting(
    "The key that takes a picture of the screen, printed PrtScn on English keyboards and Druck on \
     German ones. It does not print anything on paper.",
);

// ---------------------------------------------------------------------------
// Why a combination cannot be a shortcut — [`crate::ChordError`]. Read by
// somebody in the middle of setting one, so each says what to press instead.
// ---------------------------------------------------------------------------

/// Nothing was held down.
pub const NOTHING_HELD: Word = Word::saying(
    "shortcuts.chord.nothing-held",
    "hold Super, Ctrl or Alt as well — a shortcut on {key} by itself would take that key away \
     from everything you type",
)
.noting(
    "{key} is the key that was pressed, already in your language. Super, Ctrl and Alt are the \
     three keys held down to make a shortcut: call them what you called them in \
     shortcuts.modifier.super, shortcuts.modifier.ctrl and shortcuts.modifier.alt.",
);

/// Only Shift was held down, which leaves the key printing a character.
pub const SHIFT_IS_NOT_ENOUGH: Word = Word::saying(
    "shortcuts.chord.shift-is-not-enough",
    "Shift only changes which character {key} prints — hold Super, Ctrl or Alt as well",
)
.noting(
    "{key} is the key that was pressed. Shift is shortcuts.modifier.shift and the other three are \
     the modifier strings; use the same names here. The point is that Shift and 2 is @, which a \
     person types on purpose.",
);

/// `Ctrl+C`, which every application copies with.
pub const THE_CLIPBOARD_COPIES: Word = Word::saying(
    "shortcuts.chord.the-clipboard-copies",
    "Ctrl+C is how copying works in every application — pick another key",
)
.noting(
    "Ctrl is shortcuts.modifier.ctrl and C is the letter key, which is not translated. This is \
     about copying text and files, which alo OS promises works in every application.",
);

/// `Ctrl+X`, which every application cuts with.
pub const THE_CLIPBOARD_CUTS: Word = Word::saying(
    "shortcuts.chord.the-clipboard-cuts",
    "Ctrl+X is how cutting works in every application — pick another key",
)
.noting(
    "Ctrl is shortcuts.modifier.ctrl and X is the letter key, which is not translated. Cutting is \
     moving something to the clipboard and removing it from where it was.",
);

/// `Ctrl+V`, which every application pastes with.
pub const THE_CLIPBOARD_PASTES: Word = Word::saying(
    "shortcuts.chord.the-clipboard-pastes",
    "Ctrl+V is how pasting works in every application — pick another key",
)
.noting(
    "Ctrl is shortcuts.modifier.ctrl and V is the letter key, which is not translated. Pasting is \
     putting what is on the clipboard where the cursor is.",
);

// ---------------------------------------------------------------------------
// Two actions wanting the same keys — [`crate::Taken`] and [`crate::Clash`].
// ---------------------------------------------------------------------------

/// The chord somebody just pressed is already doing something else.
pub const TAKEN: Word = Word::saying(
    "shortcuts.clash.taken",
    "{chord} is already {action} — change that one first, or use another key",
)
.noting(
    "{chord} is a key combination such as Super+Left, already written in your language. {action} \
     is one of the shortcuts.action strings — a phrase like \"Close the window\" — so word the \
     sentence so that it reads naturally around one of them.",
);

/// One chord is set to do more than one thing.
pub const CLASH: Word = Word::saying(
    "shortcuts.clash.more-than-one-thing",
    "{chord} is set to do more than one thing — change one of them",
)
.noting(
    "{chord} is a key combination. What it is set to do is shown beside this sentence, one thing \
     to a row, so this sentence deliberately does not list them.",
);

/// Every string this crate can say, in the order a translator meets them: what
/// the shortcuts do, what is held down, the keys that print a word, why a
/// combination was refused, and what one chord wanted twice says.
pub const EVERY_WORD: [Word; 38] = [
    THE_AGENT,
    LAUNCHER,
    CLOSE_WINDOW,
    MINIMISE_WINDOW,
    MAXIMISE_WINDOW,
    SNAP_LEFT,
    SNAP_RIGHT,
    NEXT_WINDOW,
    PREVIOUS_WINDOW,
    NEXT_APPLICATION,
    PREVIOUS_APPLICATION,
    SUPER,
    CTRL,
    ALT,
    SHIFT,
    SPACE,
    TAB,
    ENTER,
    ESCAPE,
    BACKSPACE,
    DELETE,
    INSERT,
    HOME,
    END,
    PAGE_UP,
    PAGE_DOWN,
    LEFT,
    RIGHT,
    UP,
    DOWN,
    PRINT,
    NOTHING_HELD,
    SHIFT_IS_NOT_ENOUGH,
    THE_CLIPBOARD_COPIES,
    THE_CLIPBOARD_CUTS,
    THE_CLIPBOARD_PASTES,
    TAKEN,
    CLASH,
];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the list above — the tests at the bottom of this
/// file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the shell with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
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
pub fn shortcut_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The shell has one vocabulary and every crate adds its own to it, which is
/// what the area at the front of a key is for.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
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

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true, and it is
    /// the same shape as this crate putting every shipped binding back through
    /// `Chord::checked`.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}: {}",
                word.named(),
                alo_strings::Key::named(word.named()).unwrap_err()
            );
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "shortcuts", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    /// Nothing here counts anything, so the vocabulary holds no plurals.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = shortcut_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = shortcut_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The three sentences a person is refused with name what was pressed**,
    /// and the two about a clash name the chord. A refusal with nothing in it
    /// would leave somebody pressing keys to find out which one was wrong.
    #[test]
    fn the_sentences_that_name_something_have_a_gap_for_it() {
        for (word, gap) in [
            (NOTHING_HELD, "key"),
            (SHIFT_IS_NOT_ENOUGH, "key"),
            (TAKEN, "chord"),
            (CLASH, "chord"),
        ] {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().has(gap), "{}", word.named());
        }
        assert!(TAKEN.says().contains("{action}"));
    }

    /// A label has nothing to fill in, and a label with a gap in it would be a
    /// row in a picker with `{}` printed in it.
    #[test]
    fn a_label_names_nothing() {
        for word in [THE_AGENT, SUPER, PAGE_UP, THE_CLIPBOARD_COPIES] {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **The words that cannot be translated from their own words carry a
    /// note.** All four modifiers do, because three of them are printed
    /// differently on the keyboards this will run on and the fourth says so;
    /// every key that prints a word does, because that is the whole reason it
    /// is a string at all.
    #[test]
    fn the_ones_a_translator_cannot_guess_carry_a_note() {
        for word in [
            SUPER,
            CTRL,
            ALT,
            SHIFT,
            SPACE,
            TAB,
            ENTER,
            ESCAPE,
            BACKSPACE,
            DELETE,
            INSERT,
            HOME,
            END,
            PAGE_UP,
            PAGE_DOWN,
            LEFT,
            RIGHT,
            UP,
            DOWN,
            PRINT,
            THE_AGENT,
            NEXT_WINDOW,
            NOTHING_HELD,
            TAKEN,
            CLASH,
        ] {
            assert!(word.note().is_some(), "{}", word.named());
        }
        assert!(
            HOME.note()
                .is_some_and(|note| note.contains("not the person's home folder")),
        );
    }
}
