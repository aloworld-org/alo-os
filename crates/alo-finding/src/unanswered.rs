//! Why a search that was permitted did not answer.
//!
//! Every one of these is about a call that reached [`crate::Searched`] with
//! its authority intact — the grants said yes, and the disk was never a
//! question — and could still not be answered from the index it was handed.
//! It is not a refusal by the capability model: that is
//! `alo_capability::Refused`, made before anything here is reached, and it
//! travels to the record by its own road. This is the other kind of not
//! happening, and a record that flattened the two would tell a security
//! review that the grants stopped a query that was merely too long.

use alo_strings::{Filling, Said, Strings};

use crate::asking::NotAsked;
use crate::refusing::NotIndexed;
use crate::words;

/// Why a permitted search did not answer.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NotAnswered {
    /// The query was not one: nothing asked, or more than a person asks. The
    /// index's own refusal, made before anything was searched.
    #[error(transparent)]
    Asked(#[from] NotAsked),

    /// The index at hand is not this folder's, so it cannot answer for it.
    #[error(transparent)]
    Indexed(#[from] NotIndexed),

    /// A verb that is not this crate's to answer.
    #[error("nothing in the index does {verb}")]
    NotThisCrates {
        /// The verb as it was called.
        verb: String,
    },

    /// A verb performed without an argument it declares: a verb to write
    /// again rather than a call to make again.
    #[error("{verb} was performed without {argument}")]
    Missing {
        /// The verb as it was called.
        verb: String,
        /// The argument that did not arrive.
        argument: String,
    },
}

impl NotAnswered {
    /// This, in the language the person reads.
    ///
    /// The two refusals the index makes itself are said in the index's own
    /// words; the two about a verb are this crate's, declared in
    /// [`crate::words`].
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::Asked(why) => why.said(strings),
            Self::Indexed(why) => why.said(strings),
            Self::NotThisCrates { verb } => strings.say(
                &words::NOT_THIS_CRATES.key(),
                &Filling::of("verb", verb.clone()),
            ),
            Self::Missing { verb, argument } => strings.say(
                &words::MISSING.key(),
                &Filling::of("verb", verb.clone()).and("argument", argument.clone()),
            ),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Every way of not answering has a sentence, none is a key on somebody's
    /// screen, and the ones that name a verb name it.
    #[test]
    fn every_way_of_not_answering_is_said_in_a_sentence_a_person_reads() {
        let strings = Strings::of(crate::finding_words().unwrap());
        let every = [
            NotAnswered::Asked(NotAsked::Nothing),
            NotAnswered::Indexed(NotIndexed::NotTheSame {
                asked: PathBuf::from("/home/anna/Pictures"),
                indexed: PathBuf::from("/home/anna/Documents"),
            }),
            NotAnswered::NotThisCrates {
                verb: "list_folder".to_owned(),
            },
            NotAnswered::Missing {
                verb: "search_files".to_owned(),
                argument: "named".to_owned(),
            },
        ];
        for why in &every {
            let said = why.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            assert!(!said.text().starts_with("finding."), "{said}");
        }
        assert!(every[2].said(&strings).text().contains("list_folder"));
        let missing = every[3].said(&strings).into_text();
        assert!(missing.contains("search_files") && missing.contains("named"));
        assert!(every[1].said(&strings).text().contains("Pictures"));
    }
}
