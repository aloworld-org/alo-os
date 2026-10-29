//! What the first-start sheets say, in words somebody can translate.
//!
//! Read from the renders of `183:10675` and `186:10737`. Each is the design's
//! own wording; none is invented here, and none is written into the drawing.
//!
//! # Why a machine says this at all
//!
//! `alo_greeting::makes_an_account` has been able to make a machine's first
//! account since 2026-10-10 — refused when the store is not empty, written to
//! the disk before anybody is signed in, on the uid `agentd.toml` names rather
//! than one a surface chose. **Nothing had ever called it.** A machine with no
//! accounts showed one sentence saying *make an account* and offered nowhere
//! to make one, which is what stood between alo OS and a computer somebody
//! could use.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// The line at the top right of every sheet.
pub const STAGE: Word = Word::saying("shell.first-start.stage", "alo OS").noting(
    "The product's name, at the top right of every first-start sheet. A name, so leave it.",
);

/// The small line above the heading on the name sheet.
pub const NAME_EYEBROW: Word = Word::saying("shell.first-start.name.eyebrow", "make it yours")
    .noting(
        "A short line above the heading, telling a person which part of setting up they are in. \
         Lower case, like every quiet label in alo OS; use whatever your language does for one.",
    );

/// The name sheet's heading.
pub const NAME_TITLE: Word =
    Word::saying("shell.first-start.name.title", "What should we call you?").noting(
        "The heading of the sheet where a person gives their name, the first thing alo OS asks \
         anybody. A question, asked warmly.",
    );

/// What the name is for.
pub const NAME_INTRODUCTION: Word = Word::saying(
    "shell.first-start.name.introduction",
    "This name appears when you sign in. You can change it later.",
)
.noting(
    "Under the heading on the name sheet. It says where the name shows and that it is not final.",
);

/// The row a person types their name into.
pub const NAME_CHOICE: Word = Word::saying("shell.first-start.name.choice", "Your name")
    .noting("The label above the field a person types their name into.");

/// What that row says before anything is typed.
pub const NAME_HINT: Word = Word::saying("shell.first-start.name.hint", "Type your name")
    .noting("Inside the empty name field, telling a person what to type.");

/// What making an account here means.
pub const NAME_CONSEQUENCE: Word = Word::saying(
    "shell.first-start.name.consequence",
    "Your account is created on this computer. No online account is required.",
)
.noting(
    "Under the choices on the name sheet. It is a promise, and it is true: the account is kept in \
     a file on this machine and is not sent anywhere.",
);

/// The PIN sheet's small line.
pub const PIN_EYEBROW: Word = Word::saying("shell.first-start.pin.eyebrow", "Optional · PIN")
    .noting(
        "A short line above the heading on the PIN sheet. The sentence case is this phrase's own; \
         it is not a label that should be upper-cased.",
    );

/// The PIN sheet's heading.
pub const PIN_TITLE: Word = Word::saying("shell.first-start.pin.title", "Add a PIN?")
    .noting("The heading of the sheet where a person chooses a PIN. A question, and declinable.");

/// What a PIN is for.
pub const PIN_INTRODUCTION: Word = Word::saying(
    "shell.first-start.pin.introduction",
    "Use a PIN to sign in on this computer. Keep your password for the situations that require it.",
)
.noting("Under the heading on the PIN sheet, saying what a PIN is for and that it does not replace a password.");

/// The row a new PIN is typed into.
pub const PIN_CHOICE: Word = Word::saying("shell.first-start.pin.choice", "New PIN")
    .noting("The label above the field a person types a new PIN into.");

/// What that row says before anything is typed.
pub const PIN_HINT: Word = Word::saying("shell.first-start.pin.hint", "Enter your PIN")
    .noting("Inside the empty new-PIN field. A PIN is a short secret of digits; use whatever your language calls that.");

/// What a PIN means for this machine.
pub const PIN_CONSEQUENCE: Word = Word::saying(
    "shell.first-start.pin.consequence",
    "A PIN signs you in on this computer only.",
)
.noting(
    "Under the choices on the PIN sheet. It says the PIN is this machine's and goes nowhere else.",
);

/// The action that goes on to the next sheet.
pub const CONTINUE: Word = Word::saying("shell.first-start.continue", "Continue")
    .noting("The action that accepts what is on this sheet and goes on to the next.");

/// The action that makes the account.
pub const SAVE: Word = Word::saying("shell.first-start.save", "Save PIN").noting(
    "The action on the PIN sheet that keeps the PIN and makes the account on this machine.",
);

/// The line along the bottom of every sheet.
pub const FOOTER: Word = Word::saying(
    "shell.first-start.footer",
    "Local setup. No online account required.",
)
.noting("A quiet line at the bottom right of every first-start sheet, repeating that nothing here needs an account elsewhere.");

/// The small line above the heading on the language sheet.
pub const LANGUAGE_EYEBROW: Word =
    Word::saying("shell.first-start.language.eyebrow", "feel at home").noting(
        "A short line above the heading on the screen where a person picks the language alo OS \
         speaks to them in. Lower case, like every quiet label in alo OS; use whatever your \
         language does for one. Do not translate it as an instruction - it describes the step, \
         it does not ask for anything.",
    );

/// The heading on the language sheet.
pub const LANGUAGE_TITLE: Word =
    Word::saying("shell.first-start.language.title", "Choose your language.")
        .noting("The heading of the screen where a person picks the language alo OS speaks.");

/// The way on from the language sheet.
pub const USE_THIS_LANGUAGE: Word =
    Word::saying("shell.first-start.language.use", "Use this language").noting(
        "The button that accepts the language the person has picked. It names the thing it does \
         rather than saying OK, so somebody reading only the buttons can tell what will happen.",
    );

/// The small line above the heading on the region sheet, frame `09c`.
pub const REGION_EYEBROW: Word = Word::saying("shell.first-start.region.eyebrow", "local formats")
    .noting(
        "A short line above the heading on the screen where a person says which country or \
         region they are setting the computer up in. Lower case, like every quiet label in alo \
         OS. It names the step: the formats of dates, times and numbers used locally.",
    );

/// The region sheet's heading.
pub const REGION_TITLE: Word = Word::saying(
    "shell.first-start.region.title",
    "Where are you setting up?",
)
.noting(
    "The heading of the screen where a person says which country or region they are in. A \
     question about place, asked of somebody setting up a new computer.",
);

/// What a region is for.
pub const REGION_INTRODUCTION: Word = Word::saying(
    "shell.first-start.region.introduction",
    "Your region sets default formats for dates and numbers.",
)
.noting(
    "Under the heading on the region screen. It says what the choice changes: how dates, times \
     and numbers are written. It does not change the language.",
);

/// Under the region a person has chosen.
pub const SELECTED_REGION: Word =
    Word::saying("shell.first-start.region.selected", "Selected region").noting(
        "The second line of the row showing the country or region currently chosen, under its \
         name.",
    );

/// The row that opens every region.
pub const OTHER_REGION: Word = Word::saying("shell.first-start.region.other", "Other region")
    .noting("A row that opens the list of every country and region, to pick a different one.");

/// What that row does.
pub const OTHER_REGION_HINT: Word = Word::saying(
    "shell.first-start.region.other.hint",
    "Choose another country or region.",
)
.noting("The second line of the row that opens every country and region.");

/// The way on from the region sheet.
pub const USE_THIS_REGION: Word = Word::saying("shell.first-start.region.use", "Use this region")
    .noting(
        "The button that accepts the country or region chosen. It names what it does rather \
         than saying OK.",
    );

/// The overview's heading, frame `09`.
pub const OVERVIEW_TITLE: Word = Word::saying(
    "shell.first-start.overview.title",
    "Start with your language.",
)
.noting(
    "The heading of the first screen a new computer shows, which has three rows: the language, \
     the region and the keyboard. Each row opens a list to choose from.",
);

/// What the overview is for.
pub const OVERVIEW_INTRODUCTION: Word = Word::saying(
    "shell.first-start.overview.introduction",
    "Choose the language and keyboard you feel comfortable using.",
)
.noting("Under the heading on the first screen, saying what its three rows are for.");

/// The overview's language row.
pub const ROW_LANGUAGE: Word = Word::saying("shell.first-start.overview.language", "Language")
    .noting("A row label on the first screen; the language currently chosen is written under it.");

/// The overview's region row.
pub const ROW_REGION: Word = Word::saying("shell.first-start.overview.region", "Region").noting(
    "A row label on the first screen; the country or region currently chosen is written under \
     it. It decides how dates and numbers are written.",
);

/// The overview's keyboard row.
pub const ROW_KEYBOARD: Word = Word::saying("shell.first-start.overview.keyboard", "Keyboard")
    .noting(
        "A row label on the first screen; the keyboard layout currently chosen is written under \
         it.",
    );

/// The small line above the heading on the keyboard sheet, frame `09a`.
pub const KEYBOARD_EYEBROW: Word =
    Word::saying("shell.first-start.keyboard.eyebrow", "type comfortably").noting(
        "A short line above the heading on the screen where a person picks their keyboard \
         layout. Lower case, like every quiet label in alo OS. It describes the step.",
    );

/// The keyboard sheet's heading.
pub const KEYBOARD_TITLE: Word =
    Word::saying("shell.first-start.keyboard.title", "Choose your keyboard.")
        .noting("The heading of the screen where a person picks their keyboard layout.");

/// What the keyboard sheet asks.
pub const KEYBOARD_INTRODUCTION: Word = Word::saying(
    "shell.first-start.keyboard.introduction",
    "Use the layout printed on your keyboard.",
)
.noting(
    "Under the heading on the keyboard screen: pick the layout that matches the letters printed \
     on the keys in front of you.",
);

/// The row where a keyboard is tried.
pub const TEST_YOUR_KEYBOARD: Word =
    Word::saying("shell.first-start.keyboard.test", "Test your keyboard").noting(
        "A row on the keyboard screen where a person types to check the layout they chose. What \
         they type appears in it.",
    );

/// What that row says before anything is typed.
pub const TEST_HINT: Word = Word::saying(
    "shell.first-start.keyboard.test.hint",
    "Type here to check letters and symbols.",
)
.noting("The second line of the test row, before anything has been typed in it.");

/// The way on from the keyboard sheet.
pub const USE_THIS_KEYBOARD: Word =
    Word::saying("shell.first-start.keyboard.use", "Use this keyboard").noting(
        "The button that accepts the keyboard layout chosen. It names what it does rather than \
         saying OK.",
    );

/// The way back from a sheet that has one.
pub const BACK: Word = Word::saying("shell.first-start.back", "Back")
    .noting("The button that returns to the previous step of setting up. It changes nothing.");

/// What is appended to a chosen row, beside its border.
pub const SELECTED: Word = Word::saying("shell.first-start.selected", "Selected").noting(
    "Appended to the title of the row a person has chosen, after a middle dot. It exists so that \
     a choice is never shown by colour alone - somebody who cannot see the border reads this \
     word. Keep it a single word if your language has one.",
);
/// Every string these sheets draw.
pub const FIRST_START_WORDS: [Word; 38] = [
    STAGE,
    NAME_EYEBROW,
    NAME_TITLE,
    NAME_INTRODUCTION,
    NAME_CHOICE,
    NAME_HINT,
    NAME_CONSEQUENCE,
    PIN_EYEBROW,
    PIN_TITLE,
    PIN_INTRODUCTION,
    PIN_CHOICE,
    PIN_HINT,
    PIN_CONSEQUENCE,
    CONTINUE,
    SAVE,
    LANGUAGE_EYEBROW,
    LANGUAGE_TITLE,
    USE_THIS_LANGUAGE,
    BACK,
    SELECTED,
    REGION_EYEBROW,
    REGION_TITLE,
    REGION_INTRODUCTION,
    SELECTED_REGION,
    OTHER_REGION,
    OTHER_REGION_HINT,
    USE_THIS_REGION,
    OVERVIEW_TITLE,
    OVERVIEW_INTRODUCTION,
    ROW_LANGUAGE,
    ROW_REGION,
    ROW_KEYBOARD,
    KEYBOARD_EYEBROW,
    KEYBOARD_TITLE,
    KEYBOARD_INTRODUCTION,
    TEST_YOUR_KEYBOARD,
    TEST_HINT,
    USE_THIS_KEYBOARD,
];

/// Invalid declaration or collision while registering this wording.
#[derive(Debug, thiserror::Error)]
pub enum FirstStartWordsError {
    /// Invalid source phrase or translator note.
    #[error(transparent)]
    Word(#[from] WordError),
    /// An existing declaration already owns this key.
    #[error(transparent)]
    Vocabulary(#[from] VocabularyError),
}

/// Add these sheets' wording atomically, before any translation is loaded.
///
/// # Errors
/// [`FirstStartWordsError`] for a malformed phrase or a key already taken.
pub fn declare_first_start_words(vocabulary: &mut Vocabulary) -> Result<(), FirstStartWordsError> {
    let mut prepared = vocabulary.clone();
    for word in FIRST_START_WORDS {
        prepared.says(word.phrase()?)?;
    }
    prepared.says(FOOTER.phrase()?)?;
    *vocabulary = prepared;
    Ok(())
}
