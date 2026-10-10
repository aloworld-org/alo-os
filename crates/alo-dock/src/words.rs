//! Every string this crate can say, and the English beside each one.
//!
//! They fall into groups a person meets in different places: whether the dock
//! gives way, which is a list in Settings; what the dock is doing with its names,
//! which is a line under that list; the two ways a screen can fail to be one,
//! read by whoever is looking at a machine that reported something impossible;
//! and what is said about the person's own file.
//!
//! **The four edge names left with the edges.** ADR 0076 fixed the dock along the
//! bottom, so `dock.edge.bottom`, `.left`, `.right` and `.top` name nothing a
//! person picks between, and `dock.labels.beside` described a placement that no
//! longer happens. Removing a key is not the rewording ADR 0068 governs — there
//! is no new meaning under an old key — but the vocabulary snapshot is regenerated
//! in the same change, because a snapshot that still lists them is a claim that
//! something says them.
//!
//! The shape is `alo-appearance`'s, one crate on: constants, `alo_strings::Word`,
//! and a test at the bottom putting every key back through `Key::named`.
//!
//! # The one that matters is the one nobody plans for
//!
//! [`NAMES_BESIDE`] is the sentence a person reads when they want to know where
//! their dock's names are. It says **both** ways of reaching one — resting a
//! pointer on the icon, and arriving at it with the keyboard — and a
//! translation that mentioned only the pointer would read, to somebody who does
//! not use one, as though the names were gone.
//!
//! **A harder one stood here until 2026-10-10.** `dock.labels.gave-way` was
//! read by somebody who had just made their text bigger for a reason and
//! watched the names in their dock disappear, and the half that mattered was
//! not *why* but **the name is still there**. The owner removed the row of
//! names from the bar, so nothing disappears and the sentence has no occasion;
//! the key is gone from this file. The care it needed has not gone anywhere —
//! it moved into [`NAMES_BESIDE`], which is now the only thing this crate says
//! about where a name is.
//!
//! # What is deliberately not here
//!
//! **How a number is written.** The percentage and the two screen measurements
//! arrive as plain whole numbers with no sign or separator on them, exactly as
//! `alo_appearance::TextScale`'s two refusals do, because how a number is
//! written belongs to the region rather than to the language. Where the percent
//! sign goes *is* the translator's, so it is inside the sentence.

use alo_strings::Vocabulary;

/// One string a crate can say. Re-exported so this crate's own files name it as
/// `crate::words::Word`, as `alo-appearance` does.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Whether the dock gives way — [`crate::Hiding`]. A list a person picks one row
// from, and since ADR 0076 the only thing in the dock's panel they pick.
// ---------------------------------------------------------------------------

/// What [`crate::Hiding::Never`] is called.
pub const ALWAYS_SHOWN: Word = Word::saying("dock.hiding.never", "Always shown").noting(
    "One of two rows in a list a person picks from, for whether the dock gives way to a window \
     that wants the room it is in. This is the row a fresh machine is on. It describes what the \
     dock does from now on, not an instruction to show it this moment.",
);

/// What [`crate::Hiding::WhenAWindowNeedsTheRoom`] is called.
pub const GIVES_WAY_TO_A_WINDOW: Word = Word::saying(
    "dock.hiding.when-a-window-needs-the-room",
    "Gives way when a window needs the room",
)
.noting(
    "The other of the two rows. The dock goes off the screen while a window wants the space it \
     sits in, and comes back when none does. It is not closed, removed or emptied, and the \
     sentence must not suggest the dock is gone for good — a person choosing this is lending the \
     room, not giving it up.",
);

// ---------------------------------------------------------------------------
// What the dock is doing with its names — [`crate::Labels`]. One line in the
// dock's panel, so that turning the text up shows what it did.
// ---------------------------------------------------------------------------

/// Where a dock's names are, on every edge.
///
/// **Restored 2026-10-04 with the measurement that makes it possible**, for a
/// dock down a side: 200 logical pixels of usable text width, in a tooltip.
///
/// **And it is the only placement since 2026-10-10.** `dock.labels.under` and
/// `dock.labels.gave-way` were declared beside it and are gone: the owner
/// removed the row of names from a horizontal dock's bar, so a name is a
/// tooltip whichever edge the dock is on and neither of those sentences can be
/// said. Two keys therefore left this crate's vocabulary — said here because a
/// translator who has already worked on them should know they went, and why.
pub const NAMES_BESIDE: Word = Word::saying(
    "dock.labels.beside",
    "each icon shows its name beside it when you point at it or reach it by keyboard",
)
.noting(
    "The dock is down the side of the screen, so a name cannot sit under a picture — it opens \
     beside the icon, toward the middle of the screen, and goes away again. Shown in the dock's \
     settings so somebody can see where the names went after moving the dock to a side. \
     **Say both ways of reaching it**: resting a pointer on the icon and arriving at it with the \
     keyboard show the same name, and a sentence that mentioned only the pointer would read, to \
     somebody who does not use one, as though the names were gone.",
);

// ---------------------------------------------------------------------------
// Why something is not a screen — [`crate::ScreenError`]. Read by whoever is
// looking at a machine that reported an impossible display.
// ---------------------------------------------------------------------------

/// A screen with no width or no height.
pub const NOT_A_SCREEN: Word = Word::saying(
    "dock.screen.not-a-screen",
    "a screen has a width and a height — {width} by {height} is not one",
)
.noting(
    "{width} and {height} are plain whole numbers with nothing on them, and one of the two is \
     zero. \"by\" joins two measurements, as in \"1366 by 768\"; use whatever your language puts \
     between the two numbers of a size.",
);

/// A screen too small for a dock to sit on without taking it.
pub const SCREEN_TOO_SMALL: Word = Word::saying(
    "dock.screen.too-small",
    "{width} by {height} is smaller than alo OS lays out for — a screen needs at least {least} \
     each way, or the dock would take the screen rather than sit on it",
)
.noting(
    "All three gaps are plain whole numbers. \"by\" joins two measurements — see the note on \
     dock.screen.not-a-screen. \"alo OS\" is the name of the system and is never translated.",
);

// ---------------------------------------------------------------------------
// The person's own file, `dock.toml` — [`crate::FileNotRead`] and
// [`crate::FileNotWritten`]. Each names the file, because a sentence that does
// not is one a person cannot act on, and each says where the dock is instead.
// ---------------------------------------------------------------------------

/// The disk would not give the file up.
pub const KEPT_NOT_READ: Word = Word::saying(
    "dock.kept.not-read",
    "your dock settings at {path} could not be read, so the dock is where alo OS puts it",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk or a permission rather than \
     anything a person typed. \"alo OS\" is the product's name and is never translated.",
);

/// The file is there and is not dock settings.
pub const KEPT_NOT_UNDERSTOOD: Word = Word::saying(
    "dock.kept.not-understood",
    "your dock settings at {path} are not settings alo OS can read, so nothing in the file has \
     been used and the dock is where alo OS puts it",
)
.noting(
    "{path} is a file on this machine and is never translated. The important clause is that \
     nothing in the file was used: alo OS did not take the half it understood. Said of a value \
     that is not one of the ones offered, or a file with no format number at the top.",
);

/// The file stopped being settings at a line.
pub const KEPT_NOT_UNDERSTOOD_AT: Word = Word::saying(
    "dock.kept.not-understood-at",
    "your dock settings at {path} stop making sense at line {line}, so nothing in the file has \
     been used and the dock is where alo OS puts it",
)
.noting(
    "{path} is a file on this machine and is never translated. {line} is a plain whole number, \
     counted from one the way a text editor counts lines.",
);

/// The file says it is a shape this alo OS does not read.
pub const KEPT_ANOTHER_FORMAT: Word = Word::saying(
    "dock.kept.another-format",
    "your dock settings at {path} were written for a different alo OS than this one, so nothing \
     in the file has been used and the dock is where alo OS puts it",
)
.noting(
    "{path} is a file on this machine and is never translated. Most often a newer alo OS wrote \
     the file; reading it part-way would put the dock somewhere the person did not choose.",
);

/// The file names something that is not a dock setting.
pub const KEPT_UNKNOWN_KEY: Word = Word::saying(
    "dock.kept.unknown-key",
    "your dock settings at {path} say {key}, which is not something alo OS can change about the \
     dock, so nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and {key} is a word as it was typed into it; neither is \
     translated. The key is named because it is what a person has to find in the file to fix it.",
);

/// The disk would not take the changed file.
pub const KEPT_NOT_WRITTEN: Word = Word::saying(
    "dock.kept.not-written",
    "your dock settings at {path} could not be written, so nothing about the dock has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A full disk or a folder the person \
     cannot write. The second clause is what they act on: the file is exactly as it was.",
);

/// The change could not be written as a file this alo OS reads back.
pub const KEPT_NOT_EXPRESSIBLE: Word = Word::saying(
    "dock.kept.not-expressible",
    "this alo OS could not write that change into dock settings at {path} it can read back again, \
     so nothing about the dock has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A fault in alo OS rather than \
     anything the person did, said plainly: the change was refused before the file was touched.",
);

/// The file a change would replace is there and did not read, so it was kept.
pub const KEPT_NOT_REPLACED: Word = Word::saying(
    "dock.kept.not-replaced",
    "your dock settings at {path} could not be read, so that change has not been written over them and \
     nothing about the dock has been changed — correct the file, or put the dock back as alo OS ships it",
)
.noting(
    "{path} is a file on this machine and is never translated. Said when a person changes \
     something in Settings while the file, most often one they edited by hand, does not read: alo \
     OS keeps their file rather than replacing it, so a typing mistake in it is not lost. The last \
     clause gives the two ways on — mend the file in an editor, or deliberately put this section \
     back as it ships, which does replace the file. \"alo OS\" is the product's name and is never \
     translated.",
);

// ---------------------------------------------------------------------------
// What a screen reader reads out for one icon — [`crate::announcing`]. One
// sentence, built from an application's name and its counts, so that a person
// who cannot see the Dock is told the same three facts a person looking at it
// can see: which application, how many windows, and what state they are in.
// ---------------------------------------------------------------------------

/// An application with nothing open.
pub const ANNOUNCED_CLOSED: Word = Word::saying("dock.announced.closed", "{app}, not open").noting(
    "Read aloud when the reader reaches an application's icon in the dock. {app} is the \
     application's own name as this machine reports it and is never translated. It is on the dock \
     because the person pinned it there, so \"not open\" describes it rather than warns about it.",
);

/// An application with windows, none of them put aside.
pub const ANNOUNCED_OPEN: Word = Word::saying(
    "dock.announced.open",
    "{app}, {windows} open",
)
.noting(
    "Read aloud for an icon whose application has windows. {app} is the application's own name and \
     is never translated. {windows} is a plain whole number counted on this machine. English \
     leaves the word \"windows\" to the plural form; if your language counts differently, the \
     count and the word are yours to arrange.",
);

/// An application with windows, some of them put aside.
pub const ANNOUNCED_SOME_PUT_ASIDE: Word = Word::saying(
    "dock.announced.some-put-aside",
    "{app}, {windows} open, {aside} minimised",
)
.noting(
    "Read aloud for an icon some of whose windows the person has put aside. Both numbers are \
     plain whole numbers counted on this machine, and {aside} is never larger than {windows}. \
     \"Minimised\" is the word a screen-reader user will expect from other systems, which is why \
     it is used here although the rest of alo OS says \"put aside\".",
);

/// Said after the application's name when one of its windows has the keyboard.
pub const ANNOUNCED_FOCUSED: Word = Word::saying("dock.announced.focused", "focused").noting(
    "Added to what is read for an icon when one of that application's windows currently has the \
     keyboard. A single word, joined to the sentence by whoever reads it out, so that a language \
     which would rather inflect the whole sentence can translate the sentences above instead.",
);

/// Where a chosen window sits, read out so somebody who cannot see the canvas
/// knows where they would be taken.
pub const ANNOUNCED_WHERE: Word = Word::saying(
    "dock.announced.where",
    "on the canvas, {across} across and {down} down",
)
.noting(
    "Read for a window preview, so that a person who cannot see the canvas knows where choosing \
     it would take them. Both numbers are plain whole numbers of canvas units and may be \
     negative, because the canvas extends in every direction from its origin. They are positions, \
     not distances from the person.",
);

/// Where the dock goes: the bottom edge of the screen.
pub const ON_THE_BOTTOM: Word = Word::saying("dock.edge.bottom", "Along the bottom").noting(
    "One of four rows a person picks from, for which edge of the screen the dock sits on. **The \
     one a fresh machine uses**, so this is the row that will already be chosen when somebody \
     first opens the setting. A direction rather than a place: in a language written \
     right-to-left the dock is still along the bottom, and nothing about this row mirrors.",
);

/// Where the dock goes: down the left side of the screen.
pub const DOWN_THE_LEFT: Word = Word::saying("dock.edge.left", "Down the left side").noting(
    "One of four rows a person picks from, for which edge of the screen the dock sits on. **Left \
     is the physical side of the screen and does not mirror with the reading direction** — a \
     person reading right-to-left who picks this gets the dock on their left, which is what they \
     asked for. Translate it as the side, never as *the leading side* or *the start*.",
);

/// Where the dock goes: down the right side of the screen.
pub const DOWN_THE_RIGHT: Word = Word::saying("dock.edge.right", "Down the right side").noting(
    "One of four rows a person picks from, for which edge of the screen the dock sits on. The \
     same rule as the left row: **the physical side, which does not mirror with the reading \
     direction.**",
);

/// Where the dock goes: along the top edge of the screen.
pub const ALONG_THE_TOP: Word = Word::saying("dock.edge.top", "Along the top").noting(
    "One of four rows a person picks from, for which edge of the screen the dock sits on. \
     Distinguish it from the bottom row in languages where *top* and *above* are one word: this \
     is the edge the dock sits on, not something being above something else.",
);

/// What the four rows above are a choice about.
///
/// The setting's own name, for the heading of the list rather than a row in it.
/// A person reads *Where the dock goes* and then four places.
pub const WHERE_THE_DOCK_GOES: Word = Word::saying("dock.edge", "Where the dock goes").noting(
    "The heading over the four rows that name an edge of the screen. **Not a question** — alo's \
     settings name what a thing is rather than asking, so this is a label and not *Where should \
     the dock go?*",
);

/// The control that opens the overflow list: its name for a screen reader, and
/// the tooltip a person sees on hover and on keyboard focus.
///
/// **One string for both**, by the owner's ruling of 2026-10-10: *keep its
/// accessible name and tooltip: "Show more open apps".* They are the same
/// sentence because they answer the same question, and two strings would drift.
pub const SHOW_MORE_OPEN_APPS: Word = Word::saying("dock.overflow.show", "Show more open apps")
    .noting(
        "The name of the one control at the end of the dock, which opens a list of the \
         applications there was no room for. **Open**, not installed: it is about windows a \
         person has right now, not about everything on the machine. It is an instruction a \
         person acts on — a screen reader reads it and a tooltip shows it — so keep it short \
         enough to sit in a tooltip beside an icon.",
    );

/// The heading over the overflow list.
///
/// What the list is, rather than what pressing the control does. A person has
/// already pressed it by the time they read this.
pub const MORE_OPEN_APPS: Word = Word::saying("dock.overflow.heading", "More open apps").noting(
    "The heading at the top of the list that opens when somebody presses the dock's last \
     control. **A label and not an instruction** — the companion to `dock.overflow.show`, which \
     is the instruction, and the two must stay distinguishable in translation: one names a thing \
     and the other says what to do.",
);

/// Keys this crate used to declare and never will again.
///
/// **The owner's ruling of 2026-10-10**, on removing the row of names from the
/// bar: *Removing the unused vocabulary is fine. Retire `dock.labels.under` and
/// `dock.labels.gave-way` through the normal translation process. Preserve
/// their history in Git and never reuse those keys for different meanings.*
///
/// **Never reuse** is the half a list can hold, so this is that list and the
/// test below is what makes it true. The danger is not that somebody
/// resurrects the old sentence — it is that `dock.labels.under` looks like a
/// free, descriptive name to whoever next needs a key about a label, and a
/// translator with the old phrase in their memory would be handed a new meaning
/// under a name they have already answered.
///
/// `git log crates/alo-dock/src/words.rs` is where the sentences went; nothing
/// is lost, and this file is not the place to keep a copy of them.
pub const RETIRED: [&str; 2] = ["dock.labels.under", "dock.labels.gave-way"];

/// Every string this crate can say, in the order a translator meets them: the
/// two answers about whether it gives way, where the dock keeps its names, the
/// two refusals, what is said about the person's own file, what a reader is
/// told about an icon, and then where the dock goes.
///
/// **The last five arrived on 2026-10-10** with the setting that offers them,
/// and two left the same day — see [`RETIRED`]. They are at the end rather than
/// beside the other settings' words because this list's order is the order a
/// translator meets them in, and a translator who has already done this file
/// should find the new ones together rather than hunting a diff.
pub const EVERY_WORD: [Word; 25] = [
    ALWAYS_SHOWN,
    GIVES_WAY_TO_A_WINDOW,
    NAMES_BESIDE,
    NOT_A_SCREEN,
    SCREEN_TOO_SMALL,
    KEPT_NOT_READ,
    KEPT_NOT_UNDERSTOOD,
    KEPT_NOT_UNDERSTOOD_AT,
    KEPT_ANOTHER_FORMAT,
    KEPT_UNKNOWN_KEY,
    KEPT_NOT_WRITTEN,
    KEPT_NOT_EXPRESSIBLE,
    KEPT_NOT_REPLACED,
    ANNOUNCED_CLOSED,
    ANNOUNCED_OPEN,
    ANNOUNCED_SOME_PUT_ASIDE,
    ANNOUNCED_FOCUSED,
    ANNOUNCED_WHERE,
    WHERE_THE_DOCK_GOES,
    ON_THE_BOTTOM,
    DOWN_THE_LEFT,
    DOWN_THE_RIGHT,
    ALONG_THE_TOP,
    SHOW_MORE_OPEN_APPS,
    MORE_OPEN_APPS,
];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the list above — the tests at the bottom of this
/// file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the shell with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that already
/// holds one of these keys.
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
pub fn dock_words() -> Result<Vocabulary, WordsError> {
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
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what lets
    /// one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "dock", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it. Nothing here counts anything, so the vocabulary holds no
    /// plurals.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = dock_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = dock_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **A row in a picker is a name, and a name has nothing to fill in.** A row
    /// with `{}` printed in the middle of it is the failure a person choosing
    /// what their dock does would see first.
    ///
    /// This used to be the four edge names; they went with the edges, and the two
    /// rows about giving way are what a person now picks between.
    #[test]
    fn every_row_a_person_picks_names_nothing() {
        for word in [ALWAYS_SHOWN, GIVES_WAY_TO_A_WINDOW] {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{word:?}"
            );
        }
    }

    /// Every string that is about a particular value names it. A sentence saying
    /// only that something was wrong leaves whoever is reading it to guess which
    /// number was the problem.
    #[test]
    fn the_strings_that_are_about_something_have_a_gap_for_it() {
        for (word, gaps) in [
            (NOT_A_SCREEN, &["width", "height"][..]),
            (SCREEN_TOO_SMALL, &["width", "height", "least"][..]),
            (KEPT_NOT_READ, &["path"][..]),
            (KEPT_NOT_UNDERSTOOD, &["path"][..]),
            (KEPT_NOT_UNDERSTOOD_AT, &["path", "line"][..]),
            (KEPT_ANOTHER_FORMAT, &["path"][..]),
            (KEPT_UNKNOWN_KEY, &["path", "key"][..]),
            (KEPT_NOT_WRITTEN, &["path"][..]),
            (KEPT_NOT_EXPRESSIBLE, &["path"][..]),
            (KEPT_NOT_REPLACED, &["path"][..]),
        ] {
            let phrase = word.phrase().unwrap();
            for gap in gaps {
                assert!(phrase.source().has(gap), "{} wants {gap}", word.named());
            }
        }
    }

    /// **Both ways of reaching a name are in [`NAMES_BESIDE`]'s string**, not
    /// added beside it by a shell — so a translator is handed them and a
    /// checked translation cannot lose one without somebody deciding to.
    ///
    /// This tested `NAMES_GAVE_WAY`'s reassurance until 2026-10-10, when that
    /// key left with the row of names it described. The rule it was an example
    /// of is the same, and this is the string it now applies to.
    #[test]
    fn the_string_about_names_says_both_ways_of_reaching_one() {
        assert!(NAMES_BESIDE.says().contains("point at it"));
        assert!(NAMES_BESIDE.says().contains("keyboard"));
        assert!(
            NAMES_BESIDE
                .note()
                .is_some_and(|note| note.contains("Say both ways of reaching it")),
            "the note tells a translator which half matters"
        );
    }

    /// Every string here carries a note, because a word with no note is where a
    /// translation goes wrong quietly.
    #[test]
    fn every_word_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }
}

#[cfg(test)]
mod retired_keys {
    use super::{EVERY_WORD, RETIRED};

    /// **A retired key is never declared again**, which is the half of the
    /// owner's ruling a test can hold.
    ///
    /// The other half — *preserve their history* — is Git's, and needs nothing
    /// from this file.
    #[test]
    fn nothing_this_crate_says_uses_a_retired_key() {
        for retired in RETIRED {
            for word in EVERY_WORD {
                assert_ne!(
                    word.key().as_str(),
                    retired,
                    "`{retired}` is declared again. It was retired on 2026-10-10 and a \
                     translator who answered it then would be handed a new meaning under a name \
                     they have already seen — which is why the owner's ruling says never reuse \
                     them rather than merely remove them"
                );
            }
        }
    }

    /// **And a retired key is a key**, so a typo in the list above cannot make
    /// the check vacuous by naming something no key could ever be.
    #[test]
    fn every_retired_key_is_one_this_crate_could_have_said() {
        for retired in RETIRED {
            assert!(
                alo_strings::Key::named(retired).is_ok(),
                "`{retired}` is not a key this crate could ever have declared, so guarding \
                 against its reuse guards nothing"
            );
            assert!(
                retired.starts_with("dock."),
                "`{retired}` is not this crate's to retire"
            );
        }
    }
}
