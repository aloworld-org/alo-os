//! **Why a folder was not published**, each in words a person can act on.
//!
//! No `Display`, like every refusal a person reads in this workspace: the road
//! to words is [`NotPublished::said`]. What a tool printed is carried as it
//! printed it — in English, untranslated, and cut to its last lines — because
//! it is the one part of the reason nobody here wrote.

use std::path::{Path, PathBuf};

use alo_capability::ItDidNotStart;
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why a folder was not published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotPublished {
    /// What was asked is not a folder.
    NotAFolder(PathBuf),
    /// What was asked is not a name GitHub gives an account.
    NotAnAccount(String),
    /// The folder's name cannot be carried in a command.
    TheNameCannotBeCarried,
    /// `git` is not on this computer.
    NoGit,
    /// GitHub's own `gh` is not on this computer.
    NoGh,
    /// Nobody is signed in to GitHub.
    NotSignedIn,
    /// The person is signed in to GitHub as somebody other than the account
    /// the sentence named.
    SignedInAsAnother {
        /// Who they are signed in as.
        signed_in: String,
        /// Whose site the sentence named.
        account: String,
    },
    /// GitHub could not be reached, or would not say who is signed in.
    GitHubDidNotAnswer {
        /// What `gh` said.
        said: String,
    },
    /// GitHub would not make the site's repository.
    NoRepository {
        /// Whose site.
        account: String,
        /// What `gh` said.
        said: String,
    },
    /// There was nowhere to prepare it.
    NoRoom,
    /// `git` could not prepare the files.
    NotPrepared {
        /// What `git` said.
        said: String,
    },
    /// The folder has nothing in it.
    Empty,
    /// GitHub refused the new version.
    Refused {
        /// What `git` said.
        said: String,
    },
    /// It ran too long and was stopped.
    TookTooLong,
    /// Something outside it stopped it.
    WasStopped,
    /// `git` would not start.
    DidNotStart(ItDidNotStart),
    /// It ended in a way nothing here expected.
    EndedWith {
        /// How it ended.
        status: i32,
        /// What it said.
        said: String,
    },
    /// An authority that is not a publication, or a machine with nothing to
    /// publish with — alo contradicting itself, said in English for the log.
    NotHere(String),
}

impl NotPublished {
    /// Why, in one sentence: *Nothing was published: …* — or, for a run
    /// stopped before it could say how it ended, *It is not known whether it
    /// was published: …*, because the site may have changed before the stop
    /// and saying *nothing* would be a claim nobody checked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let why = self.why(strings);
        let frame = match self {
            Self::TookTooLong | Self::WasStopped => words::IT_IS_NOT_KNOWN,
            _ => words::NOTHING_WAS_PUBLISHED,
        };
        strings.say(&frame.key(), &Filling::nothing().and_said("why", &why))
    }

    /// The clause that goes after *Nothing was published:*.
    fn why(&self, strings: &Strings) -> Said {
        match self {
            Self::NotAFolder(folder) => strings.say(
                &words::BECAUSE_IT_IS_NOT_A_FOLDER.key(),
                &Filling::of(words::FOLDER, shown(folder)),
            ),
            Self::NotAnAccount(account) => strings.say(
                &words::BECAUSE_IT_IS_NOT_AN_ACCOUNT.key(),
                &Filling::of(words::ACCOUNT, account.clone()),
            ),
            Self::TheNameCannotBeCarried => {
                plain(strings, words::BECAUSE_THE_NAME_CANNOT_BE_CARRIED)
            }
            Self::NoGit => plain(strings, words::BECAUSE_THERE_IS_NO_GIT),
            Self::NoGh => plain(strings, words::BECAUSE_THERE_IS_NO_GH),
            // **The command road's own sentence**, so *not signed in* and how
            // to sign in are said one way wherever a person meets them.
            Self::NotSignedIn => plain(strings, alo_capability::words::NOT_SIGNED_IN_TO_GITHUB),
            Self::SignedInAsAnother { signed_in, account } => strings.say(
                &words::BECAUSE_SIGNED_IN_AS_ANOTHER.key(),
                &Filling::of("signed_in", signed_in.clone()).and(words::ACCOUNT, account.clone()),
            ),
            Self::GitHubDidNotAnswer { said } => {
                with_what_it_said(strings, words::BECAUSE_GITHUB_DID_NOT_ANSWER, said)
            }
            Self::NoRepository { account, said } => strings.say(
                &words::BECAUSE_THERE_IS_NO_REPOSITORY.key(),
                &Filling::of(words::ACCOUNT, account.clone()).and("said", said.clone()),
            ),
            Self::NoRoom => plain(strings, words::BECAUSE_THERE_IS_NO_ROOM),
            Self::NotPrepared { said } => {
                with_what_it_said(strings, words::BECAUSE_IT_COULD_NOT_BE_PREPARED, said)
            }
            Self::Empty => plain(strings, words::BECAUSE_THE_FOLDER_IS_EMPTY),
            Self::Refused { said } => {
                with_what_it_said(strings, words::BECAUSE_GITHUB_REFUSED_IT, said)
            }
            Self::TookTooLong => plain(strings, words::BECAUSE_IT_TOOK_TOO_LONG),
            Self::WasStopped => plain(strings, words::BECAUSE_IT_WAS_STOPPED),
            Self::DidNotStart(_) => plain(strings, words::BECAUSE_IT_DID_NOT_START),
            Self::EndedWith { status, said } => strings.say(
                &words::BECAUSE_IT_ENDED_WITH.key(),
                &Filling::of("status", status.to_string()).and("said", said.clone()),
            ),
            Self::NotHere(_) => plain(strings, words::COULD_NOT_PUBLISH_HERE),
        }
    }
}

/// A clause with nothing to fill.
fn plain(strings: &Strings, word: words::Word) -> Said {
    strings.say(&word.key(), &Filling::nothing())
}

/// A clause carrying what a tool said.
fn with_what_it_said(strings: &Strings, word: words::Word, said: &str) -> Said {
    strings.say(&word.key(), &Filling::of("said", said.to_owned()))
}

/// A path as a person reads it.
fn shown(folder: &Path) -> String {
    folder.display().to_string()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::NotPublished;
    use alo_strings::{Strings, Vocabulary};

    fn in_english() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::words::declare_into(&mut vocabulary).unwrap();
        alo_capability::words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **Every reason is a sentence a person can read**, and *not signed in*
    /// is the command road's own, with how to sign in.
    #[test]
    fn every_reason_is_said_and_not_signed_in_says_how_to_sign_in() {
        let strings = in_english();
        let said = NotPublished::NotSignedIn.said(&strings);
        assert!(
            said.text()
                .starts_with("Nothing was published: you are not signed in to GitHub.")
                && said.text().contains("gh auth login --web"),
            "{said:?}"
        );
        let another = NotPublished::SignedInAsAnother {
            signed_in: "bob".to_owned(),
            account: "anna".to_owned(),
        }
        .said(&strings);
        assert!(
            another
                .text()
                .contains("signed in to GitHub as bob, not anna"),
            "{another:?}"
        );
        for each in [
            NotPublished::NotAFolder("/home/anna/site.html".into()),
            NotPublished::NotAnAccount("anna smith".to_owned()),
            NotPublished::TheNameCannotBeCarried,
            NotPublished::NoGit,
            NotPublished::NoGh,
            NotPublished::GitHubDidNotAnswer {
                said: "error connecting to api.github.com".to_owned(),
            },
            NotPublished::NoRepository {
                account: "anna".to_owned(),
                said: "HTTP 422".to_owned(),
            },
            NotPublished::NoRoom,
            NotPublished::NotPrepared {
                said: "fatal".to_owned(),
            },
            NotPublished::Empty,
            NotPublished::Refused {
                said: "! [rejected]".to_owned(),
            },
            NotPublished::DidNotStart(alo_capability::ItDidNotStart::because("no sh")),
            NotPublished::EndedWith {
                status: 9,
                said: "odd".to_owned(),
            },
            NotPublished::NotHere("a read".to_owned()),
        ] {
            let said = each.said(&strings);
            assert!(!said.is_a_bug(), "{each:?}: {said:?}");
            assert!(
                said.text().starts_with("Nothing was published: "),
                "{said:?}"
            );
            assert!(
                !said.text().contains('{'),
                "a gap was left unfilled: {said:?}"
            );
        }
        // **A run stopped before it could tell is never told as nothing
        // published**: the site may have changed before the stop.
        for stopped in [NotPublished::TookTooLong, NotPublished::WasStopped] {
            let said = stopped.said(&strings);
            assert!(
                said.text()
                    .starts_with("It is not known whether it was published: ")
                    && said.text().ends_with("Look at your site to see."),
                "{said:?}"
            );
        }
    }
}
