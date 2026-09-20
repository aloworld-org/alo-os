//! Every string this crate can say, and the English beside each one.
//!
//! The groups, in the order this file declares them.
//!
//! **One sentence for each portal** — what it lets an application do, in the
//! words a person reads beside it wherever this machine lists the portals it
//! answers. Each says what the application may do and, where a reader could
//! reasonably worry about more, what it may not.
//!
//! **One refusal of a request this crate judged**: [`NOTHING_GRANTED`], for an
//! application nobody has granted anything. The other judged refusals are
//! `alo-capability`'s own ([`alo_capability::NotAllowed`]), because that is
//! the crate that decides, and a second sentence about the same refusal written
//! here could come to disagree with it.
//!
//! **The refusals of something that was never a request** — an identifier that
//! is not one, a path where there should be none, a relative path. They are
//! read in the record by whoever is looking at what an application tried, and
//! each names what a well-formed request would have carried.
//!
//! **What the portal backend answered** — [`crate::Outcome`]. Each is written
//! into the record of what applications asked (`crate::recording`), so whoever
//! reads that record reads why an application was answered as it was: a
//! secret handed over, a file opened, or one of the ways a request on the bus
//! was answered with the portal's refusal.
//!
//! **What applications were answered, read back** — [`crate::KeptAnswer::said`]
//! and `ReadBack::said`. Most of what the answers file holds is said with the
//! words above, and with `alo-capability`'s and `alo-applications'` own, so the
//! backend and the file cannot be two accounts of one answer. These are only
//! what the file needs and a live answer does not: who asked, where the name
//! is all the file holds; the clauses standing in for what the file does not
//! keep (a file's path, a file's kind, `alo-opening`'s finding); whether the
//! file still reaches all the way back; and how many of its lines did not read.

use alo_strings::{Key, Plural, PluralError, Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// One string this crate can say about a number of things.
///
/// Separate from [`Word`] because a countable string is declared and looked up
/// differently: two English sentences rather than one, and the reader's own
/// language decides which of *its* forms is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counted {
    /// What names it.
    named: &'static str,
    /// The gap the number goes in.
    number: &'static str,
    /// What it says about one thing.
    one: &'static str,
    /// What it says about any other number of things.
    other: &'static str,
    /// What a translator needs to know.
    note: &'static str,
}

impl Counted {
    /// What names it.
    #[must_use]
    pub fn key(&self) -> Key {
        Key::unchecked(self.named)
    }

    /// The name this is declared under.
    #[must_use]
    pub const fn named(&self) -> &'static str {
        self.named
    }
}

// ---------------------------------------------------------------------------
// What each portal lets an application do — [`crate::Portal`].
// ---------------------------------------------------------------------------

/// The file chooser, and the documents it hands over.
pub const FILE_CHOOSER: Word = Word::saying(
    "portals.portal.file-chooser",
    "Lets an application open or save a file you choose, and reach that file and nothing beside it",
)
.noting(
    "A portal is how an application on alo OS asks for something outside itself. The file is \
     the one the person picked; the application is not given the folder it is in.",
);

/// Open-with and default applications.
pub const OPEN_WITH: Word = Word::saying(
    "portals.portal.open-with",
    "Lets an application open a file you have granted it in the application that opens that kind \
     of file",
);

/// Notifications.
pub const NOTIFICATIONS: Word = Word::saying(
    "portals.portal.notifications",
    "Lets an application send you notifications",
);

/// Printing.
pub const PRINT: Word = Word::saying(
    "portals.portal.print",
    "Lets an application print a document you have granted it",
);

/// One picture of the screen.
pub const SCREENSHOT: Word = Word::saying(
    "portals.portal.screenshot",
    "Lets an application take one picture of the screen",
)
.noting("Once — recording or sharing the screen is a different portal and a different grant.");

/// The screen, continuously.
pub const SCREEN_CAPTURE: Word = Word::saying(
    "portals.portal.screen-capture",
    "Lets an application record or share the screen for as long as the grant lasts",
);

/// The camera.
pub const CAMERA: Word = Word::saying(
    "portals.portal.camera",
    "Lets an application use the camera",
);

/// The microphone.
pub const MICROPHONE: Word = Word::saying(
    "portals.portal.microphone",
    "Lets an application use the microphone",
);

/// The clipboard.
pub const CLIPBOARD: Word = Word::saying(
    "portals.portal.clipboard",
    "Lets an application read what you copied, and copy something for you",
);

/// The trash.
pub const TRASH: Word = Word::saying(
    "portals.portal.trash",
    "Lets an application move a file you have granted it to the trash",
)
.noting("To the trash, from which it can be taken back — not deleted.");

/// The desktop background.
pub const WALLPAPER: Word = Word::saying(
    "portals.portal.wallpaper",
    "Lets an application set the desktop background",
);

/// Appearance settings.
pub const SETTINGS: Word = Word::saying(
    "portals.portal.settings",
    "Lets an application read your appearance settings, such as light or dark, so it can match \
     them",
)
.noting("Only appearance: not any other setting on the machine, and never changing one.");

/// Keeping the machine awake.
pub const INHIBIT: Word = Word::saying(
    "portals.portal.inhibit",
    "Lets an application keep the machine awake, for example during a presentation",
);

/// The network monitor.
pub const NETWORK_MONITOR: Word = Word::saying(
    "portals.portal.network-monitor",
    "Lets an application know whether the machine is connected to a network, and never what is \
     sent over it",
);

/// The power-profile monitor.
pub const POWER_PROFILE_MONITOR: Word = Word::saying(
    "portals.portal.power-profile-monitor",
    "Lets an application know the power profile, so it can use less power while you are saving it",
);

/// The Secret portal.
pub const SECRET: Word = Word::saying(
    "portals.portal.secret",
    "Lets an application keep its own passwords in your keyring, and read back only its own",
)
.noting(
    "The keyring is the one place on the machine where a person's passwords are kept, locked with \
     their sign-in. An application reaches the passwords it stored and never another \
     application's.",
);

// ---------------------------------------------------------------------------
// A request judged and refused — [`crate::Refused`].
// ---------------------------------------------------------------------------

/// An application that holds no grant at all.
pub const NOTHING_GRANTED: Word = Word::saying(
    "portals.refused.nothing-granted",
    "{application} has not been granted anything on this machine, so what it asked for was \
     refused without asking you",
)
.noting(
    "{application} is the identifier of an application, like org.gnome.Cheese, and is never \
     translated; it is an application, not a person. \"Without asking you\" is literal: no \
     question was put on the screen, because an application nobody has granted anything is \
     refused before a question is.",
);

// ---------------------------------------------------------------------------
// Something that was never a request — [`crate::NotARequest`].
// ---------------------------------------------------------------------------

/// No application was named.
pub const NO_APPLICATION: Word = Word::saying(
    "portals.not-a-request.no-application",
    "A request has to say which application is asking, and this one said none",
);

/// The application's identifier is not one.
pub const NOT_AN_IDENTIFIER: Word = Word::saying(
    "portals.not-a-request.not-an-identifier",
    "A request names an application by an identifier like org.gnome.Cheese, with no spaces in it",
)
.noting("org.gnome.Cheese is an example identifier and is not translated.");

/// A portal over a path, asked without one.
pub const NEEDS_A_PATH: Word = Word::saying(
    "portals.not-a-request.needs-a-path",
    "This portal is about a file, so a request to it has to say which file",
);

/// A portal that is not over a path, asked with one.
pub const NOT_OVER_A_PATH: Word = Word::saying(
    "portals.not-a-request.not-over-a-path",
    "This portal is not about a file, so a request to it names none",
);

/// A path that is not a full path.
pub const NOT_A_FULL_PATH: Word = Word::saying(
    "portals.not-a-request.not-a-full-path",
    "A request names a file by its full path, so it means the same thing wherever it is read",
);

/// A path with `..` in it.
pub const COULD_LEAD_ELSEWHERE: Word = Word::saying(
    "portals.not-a-request.could-lead-elsewhere",
    "A path with .. in it can lead somewhere else, so a request names a file by its own path",
)
.noting("\"..\" is how a path says \"the folder above\" and is never translated.");

// ---------------------------------------------------------------------------
// What the portal backend answered — [`crate::Outcome`] and [`crate::Unanswered`].
// ---------------------------------------------------------------------------

/// An application was handed its own secret.
pub const SECRET_HANDED_OVER: Word = Word::saying(
    "portals.answered.secret-handed-over",
    "{application} was given its own secret from your keyring",
)
.noting(
    "{application} is the identifier of an application, like org.gnome.Fractal, and is never \
     translated. The secret is the one the keyring keeps for that application alone.",
);

/// A file is opened in the application that opens its kind.
///
/// Written down before that application is asked, so it says the file was
/// handed to it; when it then does not open it, [`NOT_OPENED`] follows.
pub const OPENED_IN: Word = Word::saying(
    "portals.answered.opened-in",
    "{application} had a file it was granted handed to {opener} to open",
)
.noting(
    "{application} and {opener} are identifiers of applications, like org.gnome.Geary and \
     org.gnome.Papers, and are never translated. When {opener} did not open it, a separate \
     line saying so follows this one.",
);

/// The caller is not a sandboxed application.
pub const NOT_IDENTIFIED: Word = Word::saying(
    "portals.unanswered.not-identified",
    "A request came from a program that is not a sandboxed application, so no application could \
     be named and it was refused",
)
.noting(
    "Applications on alo OS are sandboxed, and the sandbox is what says which application is \
     asking. A program running outside one cannot be told apart from any other.",
);

/// The grants could not be read.
pub const GRANTS_UNREAD: Word = Word::saying(
    "portals.unanswered.grants-unread",
    "The grants on this machine could not be read, so the request was refused",
);

/// What is installed, declared or chosen could not be read.
pub const APPLICATIONS_UNREAD: Word = Word::saying(
    "portals.unanswered.applications-unread",
    "Which application opens a kind of file could not be read, so nothing was opened",
);

/// The keyring would not answer.
pub const KEYRING_UNAVAILABLE: Word = Word::saying(
    "portals.unanswered.keyring-unavailable",
    "Your keyring could not be reached or is locked, so the application was not given its secret",
)
.noting("The keyring is never unlocked on an application's behalf.");

/// The secret could not be handed over.
pub const NOT_WRITTEN: Word = Word::saying(
    "portals.unanswered.not-written",
    "The application stopped listening before its secret could be handed over",
);

/// What was handed over is not a file.
pub const NOT_A_FILE: Word = Word::saying(
    "portals.unanswered.not-a-file",
    "An application can ask for a file to be opened, and what it handed over was not a file",
);

/// The opener did not open the file.
pub const NOT_OPENED: Word = Word::saying(
    "portals.unanswered.not-opened",
    "{opener} did not open the file when it was asked to, so nothing was opened",
)
.noting(
    "{opener} is the identifier of an application, like org.gnome.Papers, and is never translated.",
);

/// A link or a folder, which nothing here decides yet.
pub const NOT_DECIDED_HERE: Word = Word::saying(
    "portals.unanswered.not-decided-here",
    "This machine does not yet decide what opens a web link or a folder, so nothing was opened",
);

/// A request whose handle token is not one.
pub const NOT_A_TOKEN: Word = Word::saying(
    "portals.unanswered.not-a-token",
    "A request names itself with letters, digits and underscores, and this one did not",
);

/// An application read the appearance settings.
pub const APPEARANCE_READ: Word = Word::saying(
    "portals.answered.appearance-read",
    "{application} read your appearance settings, such as light or dark",
)
.noting(
    "{application} is the identifier of an application, like io.gitlab.news_flash.NewsFlash, and \
     is never translated. Appearance settings are light or dark and the accent colour, nothing \
     else.",
);

/// An application was sent appearance settings that changed.
pub const APPEARANCE_SENT: Word = Word::saying(
    "portals.answered.appearance-sent",
    "{application} was told your appearance settings changed, such as from light to dark",
)
.noting(
    "{application} is the identifier of an application, like io.gitlab.news_flash.NewsFlash, and \
     is never translated. Only an application allowed to read the settings at that moment is told.",
);

/// A setting that is not shared with applications.
pub const NO_SUCH_SETTING: Word = Word::saying(
    "portals.unanswered.no-such-setting",
    "An application asked for a setting this machine does not share with applications, and was \
     told there is none",
)
.noting("Only appearance settings are shared, and only with an application granted them.");

/// The appearance settings could not be read.
pub const APPEARANCE_UNREAD: Word = Word::saying(
    "portals.unanswered.appearance-unread",
    "Your appearance settings could not be read, so the application was not told them",
);

/// An application read how far this machine reaches.
pub const NETWORK_READ: Word = Word::saying(
    "portals.answered.network-read",
    "{application} read whether this machine is connected, and whether the connection is metered",
)
.noting(
    "{application} is the identifier of an application, like io.gitlab.news_flash.NewsFlash, and \
     is never translated. It is told whether anything is reached and whether the connection costs \
     money to use — never which network this machine is on, nor its name.",
);

/// How far this machine reaches could not be read.
pub const NETWORK_UNREAD: Word = Word::saying(
    "portals.unanswered.network-unread",
    "This machine could not tell whether it is connected, so the application was not told",
)
.noting(
    "Said when the service that manages networks did not answer. The application is refused rather \
     than told the machine is connected, which would have it keep retrying something that cannot \
     work.",
);

// ---------------------------------------------------------------------------
// What applications were answered, read back — [`crate::KeptAnswer::said`] and
// `ReadBack::said`.
// ---------------------------------------------------------------------------

/// Which application asked, beside the moment it asked.
pub const ASKED_BY: Word = Word::saying("portals.read-back.asked-by", "Asked by {application}")
    .noting(
        "{application} is the identifier of an application, like org.gnome.Fractal, which is never \
     translated, or the words for an application that could not be named. The moment it asked is \
     shown beside this, written the way the reader's region writes dates, and is not part of the \
     sentence.",
    );

/// Put where an application's identifier goes, when none was named.
pub const NOBODY_NAMED: Word = Word::saying(
    "portals.read-back.nobody-named",
    "an application that could not be named",
)
.noting(
    "Put into another sentence where an application's identifier would go. A program with no \
     sandbox, or one that had stopped before it could be named: no name is invented for it.",
);

/// Put where a refusal names what an application asked for, when that was a
/// file and the answers file keeps no path.
pub const WHAT_IT_ASKED_FOR: Word =
    Word::saying("portals.read-back.what-it-asked-for", "what it asked for").noting(
        "Put into a sentence refusing an application, where what it asked for would be named. The \
     record of what applications were answered keeps no file names, so the sentence cannot say \
     which file.",
    );

/// Put where a sentence about nothing opening a file names its kind, which the
/// answers file does not keep.
pub const THAT_KIND_OF_FILE: Word =
    Word::saying("portals.read-back.that-kind-of-file", "that kind of file").noting(
        "Put into the sentence saying nothing on this machine opens a kind of file, where the kind \
     would be named. The record of what applications were answered keeps no kinds of file.",
    );

/// The file was not a kind anything opens, read back without saying which.
pub const NOT_A_KIND_ANYTHING_OPENS: Word = Word::saying(
    "portals.read-back.not-a-kind-anything-opens",
    "The file was not a kind anything opens — a program, empty, damaged, protected with a \
     password or not recognised — so nothing was opened",
)
.noting(
    "Read back from the record of what applications were answered, which keeps that the file was \
     one of these and not which one. Opening a file never runs a program.",
);

/// The file would not be read, read back.
pub const FILE_NOT_READ: Word = Word::saying(
    "portals.read-back.file-not-read",
    "The file could not be read, so nothing was opened",
);

/// The answers file still holds everything it was ever given.
pub const WHOLE: Word = Word::saying(
    "portals.read-back.whole",
    "Nothing has been removed from what applications on this machine were answered",
)
.noting(
    "Shown above the list of what applications asked for and were told, so that a request missing \
     from it is one that was never made.",
);

/// The answers file was shortened, and does not reach back to the first answer.
pub const SHORTENED: Word = Word::saying(
    "portals.read-back.shortened",
    "What applications on this machine were answered does not go all the way back — older \
     answers were removed under how long this machine keeps its record",
)
.noting(
    "Shown above the list, with the moment it now starts at beside it, written the way the \
     reader's region writes dates, and the rule. Without this sentence, a month in which no \
     application asked anything and a month the list no longer reaches would read the same.",
);

/// Lines of the answers file that did not read, counted.
pub const LINES_NOT_READ: Counted = Counted {
    named: "portals.read-back.lines-not-read",
    number: "lines",
    one: "One line of what applications were answered could not be read, and is kept as it was",
    other: "{lines} lines of what applications were answered could not be read, and are kept as \
            they were",
    note: "Shown beside everything that did read, never in place of it. A line that could not be \
           read is usually one the machine stopped writing when it lost power; it is kept rather \
           than removed, because nobody can say what it held.",
};

/// The words above that are put into another sentence rather than shown on
/// their own, so they begin as a clause does rather than as a sentence.
pub const CLAUSES: [Word; 3] = [NOBODY_NAMED, WHAT_IT_ASKED_FOR, THAT_KIND_OF_FILE];

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 48] = [
    FILE_CHOOSER,
    OPEN_WITH,
    NOTIFICATIONS,
    PRINT,
    SCREENSHOT,
    SCREEN_CAPTURE,
    CAMERA,
    MICROPHONE,
    CLIPBOARD,
    TRASH,
    WALLPAPER,
    SETTINGS,
    INHIBIT,
    NETWORK_MONITOR,
    POWER_PROFILE_MONITOR,
    SECRET,
    NOTHING_GRANTED,
    NO_APPLICATION,
    NOT_AN_IDENTIFIER,
    NEEDS_A_PATH,
    NOT_OVER_A_PATH,
    NOT_A_FULL_PATH,
    COULD_LEAD_ELSEWHERE,
    SECRET_HANDED_OVER,
    OPENED_IN,
    NOT_IDENTIFIED,
    GRANTS_UNREAD,
    APPLICATIONS_UNREAD,
    KEYRING_UNAVAILABLE,
    NOT_WRITTEN,
    NOT_A_FILE,
    NOT_OPENED,
    NOT_DECIDED_HERE,
    NOT_A_TOKEN,
    APPEARANCE_READ,
    APPEARANCE_SENT,
    NO_SUCH_SETTING,
    APPEARANCE_UNREAD,
    NETWORK_READ,
    NETWORK_UNREAD,
    ASKED_BY,
    NOBODY_NAMED,
    WHAT_IT_ASKED_FOR,
    THAT_KIND_OF_FILE,
    NOT_A_KIND_ANYTHING_OPENS,
    FILE_NOT_READ,
    WHOLE,
    SHORTENED,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// say so. It keeps its English and its `Display`: whoever reads it is fixing
/// the list.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A countable string that could not be declared.
    #[error(transparent)]
    Counting(#[from] PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn portal_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// `alo-saying` is the one place that calls all of these.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    vocabulary.counts(
        Plural::counting(
            LINES_NOT_READ.key(),
            LINES_NOT_READ.number,
            LINES_NOT_READ.one,
            LINES_NOT_READ.other,
        )?
        .noting(LINES_NOT_READ.note)?,
    )?;
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

    /// Every key is a key, in this crate's area, and named once.
    #[test]
    fn every_key_is_a_key_in_this_crates_area_once() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "portals", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The list declares, and a vocabulary that has it already keeps its own.
    #[test]
    fn the_whole_list_declares_once() {
        let mut vocabulary = portal_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
        assert_eq!(vocabulary.counted().count(), 1);
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Nothing here calls an application an agent.** An application asking
    /// through a portal is not an AI assistant, and the sentences it is judged
    /// by must not read as though it were.
    #[test]
    fn nothing_here_calls_an_application_an_agent() {
        for word in EVERY_WORD {
            assert!(
                !word.says().to_lowercase().contains("agent"),
                "{}",
                word.named()
            );
        }
    }

    /// Each is a sentence a person reads on its own, so it begins like one —
    /// except the clauses, which are only ever put inside another sentence.
    #[test]
    fn every_string_begins_a_sentence() {
        let clauses: BTreeSet<&str> = CLAUSES.iter().map(|word| word.named()).collect();
        for word in EVERY_WORD {
            if clauses.contains(word.named()) {
                let first = word.says().chars().next().unwrap();
                assert!(first.is_lowercase(), "{} is a clause", word.named());
                continue;
            }
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase() || first == '{',
                "{} does not begin a sentence",
                word.named()
            );
        }
    }
}
