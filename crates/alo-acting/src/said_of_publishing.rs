//! **What the journal says about publishing to GitHub Pages and signing in to
//! GitHub**, declared once, here, so the daemon and the shell that write each
//! sentence and the walk that reads it (`crates/alo-walking`) cannot drift
//! apart — `crate::said_on_the_log`'s reason, for Demo 2's second road.
//!
//! Until these existed a publication left nothing on the journal: published,
//! refused because nobody was signed in, or not done at all, the only trace a
//! walk on a booted machine could read was a picture.
//!
//! **Never the folder, the account or the code GitHub shows**: the folder may
//! name the person's work, the account is who they are, and the code is theirs
//! to type at GitHub. Only what became of it.

/// The words every sentence about a publication begins with.
pub const A_PUBLICATION: &str = "alo publishing a folder to GitHub Pages:";

/// What became of one publication, as the daemon says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Publishing {
    /// The folder went to the site: a commit was pushed.
    Published,
    /// The site already held exactly the folder, so nothing was sent.
    NothingNew,
    /// Nobody is signed in to GitHub, so nothing was published — and the
    /// sign-in sheet is told to rise.
    NotSignedIn,
    /// It was refused, or the machine could not carry it out, for any other
    /// reason.
    NotDone,
}

impl Publishing {
    /// Every outcome, so a test can hold each sentence apart from the others.
    pub const ALL: [Self; 4] = [
        Self::Published,
        Self::NothingNew,
        Self::NotSignedIn,
        Self::NotDone,
    ];

    /// The words a walk reads for.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::Published => "published",
            Self::NothingNew => "nothing new to send",
            Self::NotSignedIn => "nobody is signed in to GitHub",
            Self::NotDone => "not done",
        }
    }
}

/// The sentence for one publication, and what became of it.
#[must_use]
pub fn a_publication(became: Publishing) -> String {
    format!("{A_PUBLICATION} {}", became.written())
}

/// The sentence when the GitHub sign-in sheet goes up, by itself or from
/// Settings.
pub const THE_GITHUB_SHEET_IS_UP: &str = "the GitHub sign-in sheet is up";

/// The sentence when the GitHub sign-in sheet shows the code GitHub gave —
/// **never the code itself**.
pub const THE_GITHUB_SHEET_SHOWS_A_CODE: &str = "the GitHub sign-in sheet shows a code";

#[cfg(test)]
mod tests {
    use super::*;

    /// **Each outcome is its own sentence, and no outcome's words are found
    /// inside another's** — so a walk that waits for *published* is never
    /// answered by *not done*, and the other way round.
    ///
    /// What would have to be true for this to pass while broken: the daemon
    /// writing its line with its own `format!` rather than [`a_publication`],
    /// which a search for the phrase finds.
    #[test]
    fn each_outcome_is_a_sentence_no_other_outcome_answers() {
        for became in Publishing::ALL {
            let said = a_publication(became);
            assert!(said.starts_with(A_PUBLICATION), "{said}");
            assert!(said.ends_with(became.written()), "{said}");
            for other in Publishing::ALL {
                if other != became {
                    assert!(
                        !said.contains(other.written()),
                        "{said:?} also reads as {:?}",
                        other.written()
                    );
                }
            }
        }
        assert_eq!(
            a_publication(Publishing::Published),
            "alo publishing a folder to GitHub Pages: published"
        );
    }

    /// **The sheet's two sentences are apart**: the one for a code shown does
    /// not read as the sheet merely being up, and neither carries a code.
    #[test]
    fn the_sheets_two_sentences_are_apart() {
        assert!(!THE_GITHUB_SHEET_IS_UP.contains(THE_GITHUB_SHEET_SHOWS_A_CODE));
        assert!(!THE_GITHUB_SHEET_SHOWS_A_CODE.contains(THE_GITHUB_SHEET_IS_UP));
    }
}
