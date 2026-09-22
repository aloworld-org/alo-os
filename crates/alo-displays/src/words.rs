//! Every string this crate can say, and the English beside each one.
//!
//! They fall into five groups a person meets in five places: **what happened to
//! their screens** when they signed in, plugged something in, or asked for
//! night light somewhere the sun does not set; **what became of the windows**
//! on a screen that went; **what a screen that says nothing about itself is
//! called**; the **refusals**, which are mostly about a settings file they
//! edited by hand; and **the person's own file**.
//!
//! # What is deliberately not here
//!
//! **Anything we rent.** No sentence names the display protocol, the kernel's
//! side of a graphics card, the block of bytes a monitor describes itself with,
//! or what the compositor calls a socket internally. A person is told that a
//! screen does not say what it is, not what it failed to send.
//! `tests/every_sentence_here_is_collected.rs` holds that against the machine's
//! list of rented names.
//!
//! **A resolution.** Not one sentence here names a screen's pixels. alo OS
//! chooses a size and says what it chose; *3840 by 2160 at 175%* is a
//! specification, and the person wants to know whether their text will be
//! legible.
//!
//! # A screen is named the way its owner would name it
//!
//! `{display}` is filled with what the screen says about itself — *Dell
//! U2720Q*, which is printed on the front of it — or, for a screen that says
//! nothing, one of the `displays.screen.*` names: *Built-in screen*, *HDMI
//! socket 1*, *Socket 3*. It used to be filled with the socket's own name, and
//! `crate::plugged_into` is why it no longer is: `eDP-1` is what the kernel's
//! side of a graphics card calls a connector, and nobody bought a machine to
//! learn that.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What happened to a person's screens — [`crate::Note`].
// ---------------------------------------------------------------------------

/// The screens are where the person left them.
pub const AS_YOU_LEFT_THEM: Word = Word::saying(
    "displays.as-you-left-them",
    "Your screens are arranged the way you last left them",
)
.noting(
    "Shown in the screens section of Settings, and in the list of what happened when somebody \
     plugged a screen in. It is reassurance rather than news: this exact set of screens has been \
     used together before and alo OS has put each one back where it was.",
);

/// A screen this machine has never had plugged in.
pub const NEW_HERE: Word = Word::saying(
    "displays.new-here",
    "{display} has not been used with this machine before, so alo OS has put it beside your other \
     screens and chosen a size for it from how large it is",
)
.noting(
    "{display} names a screen. It is either what the screen says about itself, such as \"Dell \
     U2720Q\", which is never translated — or, for a screen that says nothing about itself, one \
     of the names under displays.screen.*, which is already translated by the time it arrives \
     here. \"alo OS\" is the product's name and is never translated. Size here means how large \
     everything is drawn on the screen, not the screen's own dimensions.",
);

/// A screen that can only be remembered by where it is plugged in.
pub const REMEMBERED_BY_ITS_SOCKET: Word = Word::saying(
    "displays.remembered-by-its-socket",
    "{display} does not say which screen it is, so alo OS remembers it by the socket it is \
     plugged into — another screen plugged into that socket will be set up the same way",
)
.noting(
    "{display} is one of the names under displays.screen.*, such as \"Built-in screen\" or \"HDMI \
     socket 1\", and arrives here already translated. Said once, when such a screen is first \
     used. Most built-in laptop screens are like this, and so are many screens behind an adapter. \
     The second clause is the honest consequence and should not be softened: alo OS genuinely \
     cannot tell one such screen from another.",
);

/// Two screens that describe themselves identically.
pub const TOLD_APART_BY_THEIR_SOCKETS: Word = Word::saying(
    "displays.told-apart-by-their-sockets",
    "Two of your screens say exactly the same thing about themselves, so alo OS tells them apart \
     by which socket each is plugged into — swapping their cables swaps the two",
)
.noting(
    "\"alo OS\" is the product's name and is never translated. Said when somebody has two \
     identical monitors that do not carry serial numbers. A socket is the place on the machine \
     the screen's cable goes into.",
);

/// The arrangement no longer fits the screens.
pub const DID_NOT_FIT: Word = Word::saying(
    "displays.did-not-fit",
    "Your screens have changed since you arranged them, so alo OS has put them side by side \
     again. Your arrangement is still here for when they change back.",
)
.noting(
    "\"alo OS\" is the product's name and is never translated. Said when a screen reports a \
     different shape than it did when the person arranged it — a projector renegotiating, or a \
     monitor switched to another input and back — and the saved arrangement would have drawn two \
     screens over each other. The second sentence matters: nothing the person chose has been \
     thrown away.",
);

/// The screens are not the ones the machine went to sleep with.
pub const THE_DESK_CHANGED: Word = Word::saying(
    "displays.the-desk-changed",
    "Your screens have changed since this machine went to sleep, so alo OS has set up the ones in \
     front of you now — the arrangement you made at the other desk is still here for when you are \
     back at it",
)
.noting(
    "\"alo OS\" is the product's name and is never translated. Said once, when a machine that was \
     asleep is woken and the screens plugged into it are not the ones it went to sleep with: a \
     laptop closed at home and opened at the office is the everyday case. It is not a fault and \
     should not read like one — it says why the screens are laid out differently than they were a \
     moment ago. The last clause matters: nothing the person arranged at another desk has been \
     thrown away.",
);

/// The sun does not set today, so night light has nothing to start at.
pub const THE_SUN_DOES_NOT_SET: Word = Word::saying(
    "displays.the-sun-does-not-set",
    "the sun does not set where you are today, so your screens are not being warmed — they will \
     be again on the first evening it does",
)
.noting(
    "Said to somebody inside a polar circle in summer, whose night light follows the sun rather \
     than a schedule. It is not a fault and should not read like one: it says what is happening \
     and when it will change. \"Warmed\" means made more orange, the way a lamp is warmer than \
     daylight, and not made hot.",
);

/// The sun does not rise today, so night light has nothing to stop at.
pub const THE_SUN_DOES_NOT_RISE: Word = Word::saying(
    "displays.the-sun-does-not-rise",
    "the sun does not rise where you are today, so your screens are being warmed all day — they \
     will stop on the first morning it does",
)
.noting(
    "The same person as the sentence above, six months later: a polar winter. \"Warmed\" means \
     made more orange, the way a lamp is warmer than daylight, and not made hot.",
);

/// A size the machine could not draw exactly.
pub const SIZE_ROUNDED: Word = Word::saying(
    "displays.size-rounded",
    "{display} cannot draw at {asked}, so alo OS is using {used}, the nearest size it can",
)
.noting(
    "{display} names a screen: what it says about itself, which is never translated, or one of \
     the already-translated names under displays.screen.*. {asked} and {used} are percentages as \
     this machine writes them, such as \"150%\". \"alo OS\" is the product's name and is never \
     translated. The size is how large everything on the screen is drawn.",
);

// ---------------------------------------------------------------------------
// What became of the windows — [`crate::Moved`] and [`crate::CameBack`].
// ---------------------------------------------------------------------------

/// A screen went, and what was on it is elsewhere.
pub const WINDOWS_MOVED: Word = Word::saying(
    "displays.windows-moved",
    "{display} was unplugged, so what was open on it is now on {other}",
)
.noting(
    "{display} and {other} each name a screen: what that screen says about itself, which is never \
     translated, or one of the already-translated names under displays.screen.*. Said the moment \
     a cable comes out. Nothing has been closed — the windows have moved — and the sentence \
     should not sound like a warning.",
);

/// A screen came back, and what was on it went back to it.
pub const WINDOWS_CAME_BACK: Word = Word::saying(
    "displays.windows-came-back",
    "{display} is back, so what was open on it before has gone back to it",
)
.noting(
    "{display} names a screen: what it says about itself, which is never translated, or one of \
     the already-translated names under displays.screen.*. Said when a screen that was unplugged \
     is plugged in again during the same session.",
);

// ---------------------------------------------------------------------------
// What a screen that says nothing about itself is called —
// [`crate::PluggedInto`]. Each of these goes into the `{display}` gap of a
// sentence above, in place of a make and a model, so each is written as a
// **name** rather than as a clause. `crate::plugged_into` is why they exist.
// ---------------------------------------------------------------------------

/// The machine's own screen.
pub const SCREEN_BUILT_IN: Word = Word::saying("displays.screen.built-in", "Built-in screen")
    .noting(
        "The name alo OS gives a laptop's or an all-in-one's own screen, used wherever a screen \
         is named — \"Built-in screen was unplugged\", \"what was open on it is now on Built-in \
         screen\". It is a name rather than a description, so capitalise it the way a name is \
         capitalised in your language and keep it short enough to sit inside another sentence. It \
         is used for a screen that does not say what make and model it is, which nearly every \
         built-in screen does not.",
    );

/// A screen in a DisplayPort socket.
pub const SCREEN_AT_DISPLAYPORT: Word = Word::saying(
    "displays.screen.at-displayport",
    "DisplayPort socket {number}",
)
.noting(
    "The name alo OS gives a screen that does not say what make and model it is, by the socket it \
     is plugged into. \"DisplayPort\" is printed beside that socket on the machine itself and is \
     never translated. {number} is a plain whole number counted from one, which tells two \
     DisplayPort sockets apart. It is a name and sits inside other sentences, as in \"DisplayPort \
     socket 2 was unplugged\".",
);

/// A screen in an HDMI socket.
pub const SCREEN_AT_HDMI: Word = Word::saying("displays.screen.at-hdmi", "HDMI socket {number}")
    .noting(
        "The name alo OS gives a screen that does not say what make and model it is, by the \
         socket it is plugged into. \"HDMI\" is printed beside that socket on the machine itself \
         and is never translated. {number} is a plain whole number counted from one, which tells \
         two HDMI sockets apart. It is a name and sits inside other sentences.",
    );

/// A screen in a DVI socket.
pub const SCREEN_AT_DVI: Word = Word::saying("displays.screen.at-dvi", "DVI socket {number}")
    .noting(
        "The name alo OS gives a screen that does not say what make and model it is, by the \
         socket it is plugged into. \"DVI\" is printed beside that socket on the machine itself \
         and is never translated. {number} is a plain whole number counted from one. It is a name \
         and sits inside other sentences.",
    );

/// A screen in a VGA socket.
pub const SCREEN_AT_VGA: Word = Word::saying("displays.screen.at-vga", "VGA socket {number}")
    .noting(
        "The name alo OS gives a screen that does not say what make and model it is, by the \
         socket it is plugged into. \"VGA\" is printed beside that socket on the machine itself \
         and is never translated. {number} is a plain whole number counted from one. It is a name \
         and sits inside other sentences.",
    );

/// A screen in a socket alo OS has no word for.
pub const SCREEN_AT_ANOTHER_SOCKET: Word =
    Word::saying("displays.screen.at-another-socket", "Socket {number}").noting(
        "The name alo OS gives a screen that does not say what make and model it is and is \
         plugged into a kind of socket alo OS has no word for — an older or unusual one. {number} \
         is a plain whole number counted from one, which is all there is to tell two of them \
         apart. It is a name and sits inside other sentences.",
    );

/// A screen alo OS can say nothing more about than that it is another one.
pub const SCREEN_SOMEWHERE_ELSE: Word =
    Word::saying("displays.screen.somewhere-else", "Another screen").noting(
        "The name alo OS gives a screen that does not say what make and model it is and whose \
         socket alo OS cannot name or number either — the last resort, and rare. Two such screens \
         would read alike, which is admitted rather than hidden. It is a name and sits inside \
         other sentences, as in \"Another screen was unplugged\".",
    );

// ---------------------------------------------------------------------------
// The refusals — [`crate::IdentityError`], [`crate::ScaleError`],
// [`crate::NotATime`], [`crate::NotAStretch`], [`crate::NotNightly`],
// [`crate::WarmthError`], [`crate::NowhereOnEarth`], [`crate::NotArranged`]
// and [`crate::NotAttached`].
// ---------------------------------------------------------------------------

/// Nothing at all was offered as a screen's name.
pub const NOT_A_NAME: Word = Word::saying(
    "displays.not-a-name",
    "name the screen — it is what the screen says about itself, or the socket it is plugged into",
)
.noting(
    "An instruction rather than a report: shown beside an empty box in Settings, and when a \
     settings file a person edited has an empty name in it. A socket is the place on the machine \
     the screen's cable goes into.",
);

/// A name with a space at one end.
pub const NAME_SPACED: Word = Word::saying(
    "displays.name-spaced",
    "{name} begins or ends with a space, so it would never match the screen it looks like — take \
     the space out",
)
.noting(
    "{name} is text as the person typed it, shown inside quotation marks that alo OS adds, so \
     that the space they cannot otherwise see is visible. It is never translated.",
);

/// A size no screen can be set to.
pub const SIZE_OUT_OF_RANGE: Word = Word::saying(
    "displays.size-out-of-range",
    "{asked} is not a size a screen can be set to — choose between {least} and {most}",
)
.noting(
    "{asked}, {least} and {most} are percentages as this machine writes them, such as \"150%\", \
     and are never translated. The size is how large everything on the screen is drawn.",
);

/// A piece of text that is not a time of day.
pub const NOT_A_TIME: Word = Word::saying(
    "displays.not-a-time",
    "{name} is not a time of day — write it as 22:00, on a twenty-four hour clock",
)
.noting(
    "{name} is text as the person typed it, shown inside quotation marks that alo OS adds, so \
     that a space or a character they cannot otherwise see is visible. It is never translated. \
     The example is written the way a settings file holds a time, which is the same in every \
     language; how a time is *shown* to a person elsewhere belongs to their region rather than \
     to this sentence.",
);

/// A schedule that begins and ends at the same moment.
pub const BEGINS_AND_ENDS_AT_ONCE: Word = Word::saying(
    "displays.begins-and-ends-at-once",
    "night light cannot begin and end at the same moment — choose two different times",
)
.noting(
    "Shown beside two time boxes in Settings, and read when a settings file somebody edited has \
     the same time in both. Night light is the setting that makes a screen warmer — more orange \
     — in the evening.",
);

/// Both a schedule and the sun.
pub const A_SCHEDULE_OR_THE_SUN: Word = Word::saying(
    "displays.a-schedule-or-the-sun",
    "night light follows a schedule or the sun, and this asks for both",
)
.noting(
    "Almost always a settings file somebody edited by hand and left the old lines in. Night \
     light is the setting that makes a screen warmer — more orange — in the evening; \"the sun\" \
     here means from sunset to sunrise where the person said they are.",
);

/// A number that is not a warmth.
pub const NOT_A_WARMTH: Word = Word::saying(
    "displays.not-a-warmth",
    "{asked} is not a warmth a screen can be drawn at — choose between {least} and {most}",
)
.noting(
    "{asked}, {least} and {most} are colour temperatures as this machine writes them, such as \
     \"3400 K\", and are never translated. A smaller number is a warmer, more orange screen, \
     which is the opposite of what the word warm suggests about a number — the sentence \
     deliberately does not explain that, because the box the person is typing into is a slider \
     with both ends labelled.",
);

/// A number that is not a latitude.
pub const NOT_A_LATITUDE: Word = Word::saying(
    "displays.not-a-latitude",
    "{asked} is not a latitude — a latitude runs from -90 in the far south to 90 in the far north",
)
.noting(
    "{asked} is a number as the person typed it and is never translated. A latitude is how far \
     north or south somewhere is; alo OS asks for it so that it can work out sunset without \
     asking anybody else where the person lives.",
);

/// A number that is not a longitude.
pub const NOT_A_LONGITUDE: Word = Word::saying(
    "displays.not-a-longitude",
    "{asked} is not a longitude — a longitude runs from -180 in the far west to 180 in the far \
     east",
)
.noting(
    "{asked} is a number as the person typed it and is never translated. A longitude is how far \
     east or west somewhere is, counted from the line through Greenwich.",
);

/// An arrangement with no screens in it.
pub const NO_SCREENS: Word = Word::saying(
    "displays.no-screens",
    "there are no screens here, so there is nothing to arrange",
)
.noting(
    "Read when a settings file holds an arrangement with nothing in it, or when nothing at all is \
     plugged into the machine.",
);

/// One screen written down twice.
pub const THE_SAME_SCREEN_TWICE: Word = Word::saying(
    "displays.the-same-screen-twice",
    "{display} is in this arrangement twice, so alo OS cannot tell where you meant to put it",
)
.noting(
    "{display} names a screen: what it says about itself, which is never translated, or one of \
     the already-translated names under displays.screen.*. \"alo OS\" is the product's name and \
     is never translated. Almost always a settings file somebody edited by hand and copied a \
     block in.",
);

/// An arrangement with no main screen.
pub const NO_MAIN_SCREEN: Word = Word::saying(
    "displays.no-main-screen",
    "one of your screens has to be the main one, and this arrangement names none",
)
.noting(
    "The main screen is the one a new window opens on. Read when a settings file somebody edited \
     has lost the line that marks it.",
);

/// An arrangement with more than one main screen.
pub const MORE_THAN_ONE_MAIN_SCREEN: Word = Word::saying(
    "displays.more-than-one-main-screen",
    "only one of your screens can be the main one, and this arrangement names more than one",
)
.noting("The main screen is the one a new window opens on.");

/// A row that names neither a screen nor a socket.
pub const HALF_A_SCREEN: Word = Word::saying(
    "displays.half-a-screen",
    "one screen in this arrangement is written down as neither what a screen says about itself \
     nor the socket it is plugged into, so alo OS cannot tell which screen it is",
)
.noting(
    "\"alo OS\" is the product's name and is never translated. Read when a settings file somebody \
     edited has half of a screen's description in it, or has both a description and a socket.",
);

/// No screen is in that socket.
pub const NOT_PLUGGED_IN: Word = Word::saying(
    "displays.not-plugged-in",
    "there is no screen plugged in there",
)
.noting("A socket is the place on the machine a screen's cable goes into.");

/// A screen is already in that socket.
pub const ALREADY_PLUGGED_IN: Word = Word::saying(
    "displays.already-plugged-in",
    "there is already a screen plugged in there",
)
.noting("A socket is the place on the machine a screen's cable goes into.");

/// It is the only screen the machine has.
pub const NOTHING_REMAINS: Word = Word::saying(
    "displays.nothing-remains",
    "that is the only screen this machine has, so what is open on it has nowhere to go",
)
.noting(
    "Read when something asks alo OS to treat the last remaining screen as unplugged. Nothing is \
     changed and nothing is closed.",
);

// ---------------------------------------------------------------------------
// The person's own file, `displays.toml` — [`crate::FileNotRead`] and
// [`crate::FileNotWritten`]. Each names the file, and each says what the
// machine does instead.
// ---------------------------------------------------------------------------

/// The disk would not give the file up.
pub const KEPT_NOT_READ: Word = Word::saying(
    "displays.kept.not-read",
    "your screen arrangements at {path} could not be read, so your screens have been laid out \
     side by side",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk or a permission rather than \
     anything a person typed.",
);

/// The file is there and is not arrangements.
pub const KEPT_NOT_UNDERSTOOD: Word = Word::saying(
    "displays.kept.not-understood",
    "your screen arrangements at {path} are not arrangements alo OS can read, so nothing in the \
     file has been used and your screens have been laid out side by side",
)
.noting(
    "{path} is a file on this machine and is never translated. \"alo OS\" is the product's name \
     and is never translated. The important clause is that nothing in the file was used: alo OS \
     did not take the half it understood.",
);

/// The file stopped being arrangements at a line.
pub const KEPT_NOT_UNDERSTOOD_AT: Word = Word::saying(
    "displays.kept.not-understood-at",
    "your screen arrangements at {path} stop making sense at line {line}, so nothing in the file \
     has been used and your screens have been laid out side by side",
)
.noting(
    "{path} is a file on this machine and is never translated. {line} is a plain whole number, \
     counted from one the way a text editor counts lines.",
);

/// The file says it is a shape this alo OS does not read.
pub const KEPT_ANOTHER_FORMAT: Word = Word::saying(
    "displays.kept.another-format",
    "your screen arrangements at {path} were written for a different alo OS than this one, so \
     nothing in the file has been used and your screens have been laid out side by side",
)
.noting(
    "{path} is a file on this machine and is never translated. \"alo OS\" is the product's name \
     and is never translated. Most often a newer alo OS wrote the file.",
);

/// The file names something that is not part of an arrangement.
pub const KEPT_UNKNOWN_KEY: Word = Word::saying(
    "displays.kept.unknown-key",
    "your screen arrangements at {path} say {key}, which is not something alo OS knows about \
     screens, so nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and {key} is a word as it was typed into it; neither is \
     translated. The key is named because it is what a person has to find in the file to fix it.",
);

/// The disk would not take the changed file.
pub const KEPT_NOT_WRITTEN: Word = Word::saying(
    "displays.kept.not-written",
    "your screen arrangements at {path} could not be written, so nothing about your screens has \
     been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A full disk or a folder the person \
     cannot write. The second clause is what they act on: the file is exactly as it was.",
);

/// The change could not be written as a file this alo OS reads back.
pub const KEPT_NOT_EXPRESSIBLE: Word = Word::saying(
    "displays.kept.not-expressible",
    "this alo OS could not write that change into your screen arrangements at {path} in a way it \
     can read back again, so nothing about your screens has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. \"alo OS\" is the product's name \
     and is never translated. A fault in alo OS rather than anything the person did, said \
     plainly: the change was refused before the file was touched.",
);

/// The file a change would replace is there and did not read, so it was kept.
pub const KEPT_NOT_REPLACED: Word = Word::saying(
    "displays.kept.not-replaced",
    "your screen arrangements at {path} could not be read, so that change has not been written \
     over them and nothing about your screens has been changed — correct the file, or put your \
     screen arrangements back as alo OS ships them",
)
.noting(
    "{path} is a file on this machine and is never translated. \"alo OS\" is the product's name \
     and is never translated. Said when a person changes something in Settings while the file, \
     most often one they edited by hand, does not read: alo OS keeps their file rather than \
     replacing it, so a typing mistake in it is not lost. The last clause gives the two ways on.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 43] = [
    AS_YOU_LEFT_THEM,
    NEW_HERE,
    REMEMBERED_BY_ITS_SOCKET,
    TOLD_APART_BY_THEIR_SOCKETS,
    DID_NOT_FIT,
    THE_DESK_CHANGED,
    THE_SUN_DOES_NOT_SET,
    THE_SUN_DOES_NOT_RISE,
    SIZE_ROUNDED,
    WINDOWS_MOVED,
    WINDOWS_CAME_BACK,
    SCREEN_BUILT_IN,
    SCREEN_AT_DISPLAYPORT,
    SCREEN_AT_HDMI,
    SCREEN_AT_DVI,
    SCREEN_AT_VGA,
    SCREEN_AT_ANOTHER_SOCKET,
    SCREEN_SOMEWHERE_ELSE,
    NOT_A_NAME,
    NAME_SPACED,
    SIZE_OUT_OF_RANGE,
    NOT_A_TIME,
    BEGINS_AND_ENDS_AT_ONCE,
    A_SCHEDULE_OR_THE_SUN,
    NOT_A_WARMTH,
    NOT_A_LATITUDE,
    NOT_A_LONGITUDE,
    NO_SCREENS,
    THE_SAME_SCREEN_TWICE,
    NO_MAIN_SCREEN,
    MORE_THAN_ONE_MAIN_SCREEN,
    HALF_A_SCREEN,
    NOT_PLUGGED_IN,
    ALREADY_PLUGGED_IN,
    NOTHING_REMAINS,
    KEPT_NOT_READ,
    KEPT_NOT_UNDERSTOOD,
    KEPT_NOT_UNDERSTOOD_AT,
    KEPT_ANOTHER_FORMAT,
    KEPT_UNKNOWN_KEY,
    KEPT_NOT_WRITTEN,
    KEPT_NOT_EXPRESSIBLE,
    KEPT_NOT_REPLACED,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn display_words() -> Result<Vocabulary, WordsError> {
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
    use std::collections::BTreeSet;

    use super::*;

    /// What we ship is held to the rule everybody else is held to.
    #[test]
    fn every_key_is_a_key_under_this_crates_area() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "displays", "{}", word.named());
        }
    }

    /// The list declares once, each key once, and a second declaration
    /// replaces nothing.
    #[test]
    fn the_whole_list_declares_once() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let mut vocabulary = display_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// Every word carries a note: none of these can be translated from its own
    /// words alone.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Only a screen's own name, a socket's number, a size, a file's place, a
    /// line and a key are ever filled in.** No sentence here has room for a
    /// resolution, a window's title or anything else that was on somebody's
    /// screen.
    ///
    /// `number` is the socket's own, which is how two HDMI sockets are told
    /// apart; it is the one gap added since this list was written, and it is on
    /// it rather than the list being loosened.
    #[test]
    fn nothing_but_a_screen_a_size_a_path_a_line_or_a_key_is_filled_in() {
        let allowed: BTreeSet<&str> = [
            "display", "other", "name", "number", "asked", "used", "least", "most", "path", "line",
            "key",
        ]
        .into();
        for word in EVERY_WORD {
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(allowed.contains(gap.as_str()), "{}: {gap}", word.named());
            }
        }
    }
}
