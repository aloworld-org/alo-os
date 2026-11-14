//! Everything publishing says, in the English it is written in.
//!
//! Each is a key a translator renders, with a note saying what it is for. The
//! sentence the person approves, the line they read afterwards and every
//! refusal are here, so the warning the owner asked for — *it goes to the
//! internet, and Put back cannot recall it* — is written once and read by every
//! surface that tells a person about a publication.

use alo_strings::{Vocabulary, VocabularyError, WordError};

pub use alo_strings::Word;

/// The name of the argument naming the folder.
pub const FOLDER: &str = "folder";

/// The name of the argument naming the GitHub account.
pub const ACCOUNT: &str = "account";

/// What the verb is for, in the list of what alo can do.
pub const PURPOSE: Word = Word::saying(
    "publishing.verb.purpose",
    "publish a folder to the person's own GitHub Pages site, where anybody on the internet can \
     read it",
)
.noting(
    "What alo's ability to publish a folder is for, shown in a list of what it can do. GitHub \
     Pages is GitHub's free service that puts a folder of web pages on the internet at the \
     address <account>.github.io; keep 'GitHub Pages' as it is.",
);

/// The argument naming the folder.
pub const THE_FOLDER: Word = Word::saying(
    "publishing.verb.folder",
    "the folder to publish, whole, by its full path",
)
.noting("Describes one part of the request: which folder on this computer is published.");

/// The argument naming the account.
pub const THE_ACCOUNT: Word = Word::saying(
    "publishing.verb.account",
    "the GitHub account whose site it goes to, by its name on GitHub",
)
.noting(
    "Describes one part of the request: whose GitHub Pages site. The name is the account's name \
     on GitHub, like 'anna'.",
);

/// The sentence the person approves — and, at full access, the account of
/// what was done without asking.
pub const SENTENCE: Word = Word::saying(
    "publishing.verb.sentence",
    "publish the folder {folder} to the internet as your GitHub Pages site {account}.github.io, \
     in place of what is there — anybody can read it, and Put back can't recall it",
)
.noting(
    "The sentence a person approves before alo publishes a folder. {folder} is the folder's path \
     and {account} the GitHub account's name; neither is translated, and '.github.io' after it is \
     part of the address. 'Put back' is the button that undoes what alo did on this computer; use \
     the words your translation uses for it. The warning is the point of the sentence: once \
     published, the folder is on the internet and nothing on this computer can take it back. \
     Lower case at the start, because it is shown after other words.",
);

/// The sentence of the declaration a model is offered, which names no account
/// because none has been looked up yet — said only where such a call is
/// written down before `gh` was asked, at a level that refuses it.
pub const OFFERED_SENTENCE: Word = Word::saying(
    "publishing.verb.offered-sentence",
    "publish the folder {folder} to the internet as the GitHub Pages site of the account you are \
     signed in to GitHub with, in place of what is there — anybody can read it, and Put back \
     can't recall it",
)
.noting(
    "The same request as the sentence a person approves, written down before alo asked which \
     GitHub account they are signed in with, so it names no account. {folder} is the folder's \
     path and is not translated. 'Put back' is the button that undoes what alo did on this \
     computer; use the words your translation uses for it. Lower case at the start, because it \
     is shown after other words.",
);

/// What every publication changes, on the card under the sentence.
pub const CHANGES: Word = Word::saying(
    "publishing.verb.changes",
    "Puts the files in the folder on the internet as your GitHub Pages site, in place of what \
     was there, and makes the site's repository on GitHub first if you have none.",
)
.noting(
    "Shown on the card a person answers before alo publishes a folder, beside a tick. A \
     repository is the place on GitHub where a project's files are kept. Held to what the \
     publishing does by a test.",
);

/// What every publication leaves alone.
pub const LEAVES: Word = Word::saying(
    "publishing.verb.leaves",
    "Nothing on this computer is changed, and the site's earlier versions stay in its history on \
     GitHub.",
)
.noting(
    "Shown on the same card, beside the line above. True: alo prepares what it sends somewhere \
     of its own and never writes into the folder. Held to that by a test.",
);

/// The line the person reads after alo published without asking, at full
/// access (the owner's decision of 2026-11-13).
pub const PUBLISHED_WITHOUT_ASKING: Word = Word::saying(
    "publishing.published-without-asking",
    "Published without asking you first, because you chose full access in Settings → Running \
     code: the folder {folder} is on the internet now, as your GitHub Pages site at {address}. \
     Put back can't recall it.",
)
.noting(
    "Shown after alo published a folder without asking, because the person chose full access. \
     {folder} is the folder's path and {address} the web address it can be read at; neither is \
     translated. 'Settings → Running code' names a page as Settings shows it, and 'Put back' the \
     button that undoes what alo did on this computer; use the words your translation uses there.",
);

/// The line the person reads after they said yes and it was published.
pub const PUBLISHED_AFTER_YOUR_OK: Word = Word::saying(
    "publishing.published-after-your-ok",
    "Published: it is on the internet now, as your GitHub Pages site at {address}. Put back \
     can't recall it.",
)
.noting(
    "Shown after the person said yes and alo published a folder. {address} is the web address \
     it can be read at, and is not translated. 'Put back' is the button that undoes what alo did \
     on this computer; use the words your translation uses for it.",
);

/// The line the person reads when the site already held exactly this folder.
pub const NOTHING_NEW: Word = Word::saying(
    "publishing.nothing-new",
    "Nothing new was published: your GitHub Pages site at {address} already has exactly the \
     files in {folder}.",
)
.noting(
    "Shown when alo was asked to publish a folder and the site already held exactly those \
     files, so nothing was sent. {address} and {folder} are not translated.",
);

/// The message the commit on GitHub carries.
pub const THE_COMMIT_SAYS: Word =
    Word::saying("publishing.commit-message", "Published from alo OS").noting(
        "The short note GitHub keeps beside each version of the person's site that alo \
         published, readable by anybody who looks at the site's history. 'alo OS' is a name.",
    );

/// A publication that did not happen, around why.
pub const NOTHING_WAS_PUBLISHED: Word = Word::saying(
    "publishing.not.published",
    "Nothing was published: {why}.",
)
.noting(
    "Shown when alo was asked to publish a folder and did not. {why} is one of the clauses below, \
     each starting in lower case with no full stop of its own.",
);

/// Why: git is not on this computer.
pub const BECAUSE_THERE_IS_NO_GIT: Word = Word::saying(
    "publishing.not.no-git",
    "git, the program that sends it, is not on this computer",
)
.noting("Follows 'Nothing was published:'. 'git' is a program's name and is not translated.");

/// Why: GitHub's own command is not on this computer.
pub const BECAUSE_THERE_IS_NO_GH: Word = Word::saying(
    "publishing.not.no-gh",
    "GitHub's own command, gh, is not on this computer",
)
.noting("Follows 'Nothing was published:'. 'gh' is a program's name and is not translated.");

/// Why: the person is signed in to GitHub as somebody else.
pub const BECAUSE_SIGNED_IN_AS_ANOTHER: Word = Word::saying(
    "publishing.not.another-account",
    "you are signed in to GitHub as {signed_in}, not {account}, so it would have gone to \
     somebody else's site",
)
.noting(
    "Follows 'Nothing was published:'. {signed_in} and {account} are GitHub account names and \
     are not translated.",
);

/// Why: GitHub could not be reached, or would not answer.
pub const BECAUSE_GITHUB_DID_NOT_ANSWER: Word = Word::saying(
    "publishing.not.no-answer",
    "GitHub could not be reached, or would not say who is signed in; it said: {said}",
)
.noting(
    "Follows 'Nothing was published:'. {said} is what GitHub's own command printed, in English, \
     and is not translated.",
);

/// Why: the site's repository could not be made.
pub const BECAUSE_THERE_IS_NO_REPOSITORY: Word = Word::saying(
    "publishing.not.no-repository",
    "GitHub would not make the repository {account}.github.io; it said: {said}",
)
.noting(
    "Follows 'Nothing was published:'. A repository is the place on GitHub where a project's \
     files are kept. {account} is an account name and {said} what GitHub's own command printed; \
     neither is translated.",
);

/// Why: there was nowhere to prepare it.
pub const BECAUSE_THERE_IS_NO_ROOM: Word = Word::saying(
    "publishing.not.no-room",
    "this computer could not make a place of its own to prepare it in",
)
.noting(
    "Follows 'Nothing was published:'. alo prepares what it sends in a temporary place of its \
     own, and could not make one — usually because the disk is full.",
);

/// Why: git could not prepare the files.
pub const BECAUSE_IT_COULD_NOT_BE_PREPARED: Word = Word::saying(
    "publishing.not.not-prepared",
    "git could not prepare the files to send; it said: {said}",
)
.noting(
    "Follows 'Nothing was published:'. {said} is what git printed, in English, and is not \
     translated.",
);

/// Why: the folder has nothing in it.
pub const BECAUSE_THE_FOLDER_IS_EMPTY: Word = Word::saying(
    "publishing.not.empty",
    "the folder has nothing in it to publish",
)
.noting("Follows 'Nothing was published:'.");

/// Why: GitHub refused the new version.
pub const BECAUSE_GITHUB_REFUSED_IT: Word = Word::saying(
    "publishing.not.refused",
    "GitHub refused it, which usually means the site was changed somewhere else while alo was \
     publishing — asking again publishes on top of that change; it said: {said}",
)
.noting(
    "Follows 'Nothing was published:'. {said} is what git printed, in English, and is not \
     translated.",
);

/// A publication stopped before it said how it ended, around why — the one
/// case where nobody here knows whether the site changed, and says so.
pub const IT_IS_NOT_KNOWN: Word = Word::saying(
    "publishing.not-known",
    "It is not known whether it was published: {why}. Look at your site to see.",
)
.noting(
    "Shown when alo was publishing a folder and was stopped before it could tell. {why} is one of \
     the two clauses below, starting in lower case with no full stop of its own.",
);

/// Why: it ran too long.
pub const BECAUSE_IT_TOOK_TOO_LONG: Word = Word::saying(
    "publishing.not.too-long",
    "it ran longer than alo lets one thing run, and was stopped before it finished",
)
.noting("Follows 'It is not known whether it was published:'.");

/// Why: it was stopped.
pub const BECAUSE_IT_WAS_STOPPED: Word = Word::saying(
    "publishing.not.stopped",
    "something stopped it before it finished",
)
.noting(
    "Follows 'It is not known whether it was published:', when something outside alo ended it.",
);

/// Why: git would not start.
pub const BECAUSE_IT_DID_NOT_START: Word = Word::saying(
    "publishing.not.not-started",
    "alo could not start git on this computer",
)
.noting("Follows 'Nothing was published:'. 'git' is a program's name and is not translated.");

/// Why: it ended in a way nothing here expected.
pub const BECAUSE_IT_ENDED_WITH: Word = Word::saying(
    "publishing.not.ended-with",
    "it ended with exit status {status}; it said: {said}",
)
.noting(
    "Follows 'Nothing was published:'. {status} is a number a program gives as it ends, and \
     {said} what it printed, in English; neither is translated.",
);

/// Why: the folder's name cannot be put in a command.
pub const BECAUSE_THE_NAME_CANNOT_BE_CARRIED: Word = Word::saying(
    "publishing.not.name-cannot-be-carried",
    "the folder's name has a line break in it, or is too long, for a command to carry",
)
.noting("Follows 'Nothing was published:'.");

/// Why: what was asked is not a folder.
pub const BECAUSE_IT_IS_NOT_A_FOLDER: Word =
    Word::saying("publishing.not.not-a-folder", "{folder} is not a folder")
        .noting("Follows 'Nothing was published:'. {folder} is a path and is not translated.");

/// Why: what was asked is not a GitHub account's name.
pub const BECAUSE_IT_IS_NOT_AN_ACCOUNT: Word = Word::saying(
    "publishing.not.not-an-account",
    "{account} is not a name GitHub gives an account, which has only letters, numbers and single \
     hyphens, at most 39 of them",
)
.noting(
    "Follows 'Nothing was published:'. {account} is what was asked for, and is not translated.",
);

/// At the sealed box: what publishing needs, and where to choose it.
pub const THE_SEALED_BOX_CANNOT_PUBLISH: Word = Word::saying(
    "publishing.sealed-box",
    "Nothing was published. Publishing runs git and GitHub's own command, gh, as you, and the \
     sealed box can't run programs on this computer yet. Choose 'ask each time' in Settings → \
     Running code to publish after you approve it, or 'full access' to let alo publish without \
     asking.",
)
.noting(
    "Shown when alo was asked to publish a folder and the person's setting for running code is \
     the sealed box, the strictest of three. 'ask each time', 'full access' and 'Settings → \
     Running code' are named as Settings shows them; use the words your translation uses there.",
);

/// An authority that is not a publication, which only a mistake in alo can
/// cause.
pub const COULD_NOT_PUBLISH_HERE: Word = Word::saying(
    "publishing.not.here",
    "alo could not publish from this computer, because something in alo itself went wrong",
)
.noting(
    "Follows 'Nothing was published:'. Shown only when alo contradicted itself; the service log \
     says what.",
);

/// Every word this crate says.
pub const EVERY_WORD: [Word; 31] = [
    IT_IS_NOT_KNOWN,
    PURPOSE,
    THE_FOLDER,
    THE_ACCOUNT,
    SENTENCE,
    OFFERED_SENTENCE,
    CHANGES,
    LEAVES,
    PUBLISHED_WITHOUT_ASKING,
    PUBLISHED_AFTER_YOUR_OK,
    NOTHING_NEW,
    THE_COMMIT_SAYS,
    NOTHING_WAS_PUBLISHED,
    BECAUSE_THERE_IS_NO_GIT,
    BECAUSE_THERE_IS_NO_GH,
    BECAUSE_SIGNED_IN_AS_ANOTHER,
    BECAUSE_GITHUB_DID_NOT_ANSWER,
    BECAUSE_THERE_IS_NO_REPOSITORY,
    BECAUSE_THERE_IS_NO_ROOM,
    BECAUSE_IT_COULD_NOT_BE_PREPARED,
    BECAUSE_THE_FOLDER_IS_EMPTY,
    BECAUSE_GITHUB_REFUSED_IT,
    BECAUSE_IT_TOOK_TOO_LONG,
    BECAUSE_IT_WAS_STOPPED,
    BECAUSE_IT_DID_NOT_START,
    BECAUSE_IT_ENDED_WITH,
    BECAUSE_THE_NAME_CANNOT_BE_CARRIED,
    BECAUSE_IT_IS_NOT_A_FOLDER,
    BECAUSE_IT_IS_NOT_AN_ACCOUNT,
    THE_SEALED_BOX_CANNOT_PUBLISH,
    COULD_NOT_PUBLISH_HERE,
];

/// Why this crate's words could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate says, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn publishing_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate says into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] for a key the vocabulary already holds.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}
