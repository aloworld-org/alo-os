//! What the walk did not reach, kept beside the index so that a search can
//! say what it did not look at.
//!
//! An empty answer has to be *nothing matched* and never *nothing was looked
//! at*, and the difference is this: the folders the machine would not read,
//! the folders on another filesystem the walk did not enter, the folders it
//! had not finished when it reached its bound, and the things whose names
//! cannot be shown. Each is written into the index's first line, and the
//! search that answers *what was not searched* — the plan's next task — reads
//! it from there rather than walking to find out.

use alo_strings::{Counting, Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words;

/// What the walk under the folder could not reach.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Covered {
    /// Whether the walk finished, rather than stopping at its bound.
    pub whole: bool,
    /// The bound: how many things one walk looks at.
    pub most: usize,
    /// Folders the machine would not let the walk read, each with what it
    /// said.
    pub unread: Vec<Unread>,
    /// Folders on another filesystem, not entered.
    pub elsewhere: Vec<String>,
    /// Folders the walk had not finished listing when it stopped. Empty
    /// unless the walk was not [`Self::whole`].
    pub not_entered: Vec<String>,
    /// How many things were left out because their names cannot be shown.
    pub unnamed: usize,
}

/// A folder the walk found and could not read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unread {
    /// Where it is below the folder, with `/` between the parts.
    pub below: String,
    /// What the machine said when asked to list it, as a sentence.
    pub why: String,
}

impl Covered {
    /// Whether everything under the folder that could be reached was.
    #[must_use]
    pub fn is_everything(&self) -> bool {
        self.whole && self.unread.is_empty() && self.elsewhere.is_empty() && self.unnamed == 0
    }

    /// The sentence said once above an index that stopped at its bound — or
    /// nothing, for one that finished.
    #[must_use]
    pub fn not_the_whole(&self, strings: &Strings) -> Option<Said> {
        if self.whole {
            return None;
        }
        Some(strings.count(
            &words::NOT_WHOLE.key(),
            &Counting::of(self.most as u64),
            &Filling::of("most", self.most.to_string()),
        ))
    }

    /// The sentence said once above an index that left names out — or
    /// nothing, for one that did not.
    #[must_use]
    pub fn left_unnamed(&self, strings: &Strings) -> Option<Said> {
        if self.unnamed == 0 {
            return None;
        }
        Some(strings.count(
            &words::UNNAMED.key(),
            &Counting::of(self.unnamed as u64),
            &Filling::of("unnamed", self.unnamed.to_string()),
        ))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A walk that reached everything says nothing above the index; one that
    /// did not says exactly what.
    #[test]
    fn what_was_not_reached_is_said_and_what_was_is_not() {
        let strings = Strings::of(crate::finding_words().unwrap());
        let everything = Covered {
            whole: true,
            most: 20_000,
            unread: Vec::new(),
            elsewhere: Vec::new(),
            not_entered: Vec::new(),
            unnamed: 0,
        };
        assert!(everything.is_everything());
        assert!(everything.not_the_whole(&strings).is_none());
        assert!(everything.left_unnamed(&strings).is_none());

        let not = Covered {
            whole: false,
            unnamed: 3,
            not_entered: vec![String::new()],
            ..everything
        };
        assert!(!not.is_everything());
        let cut = not.not_the_whole(&strings).unwrap();
        assert!(cut.text().contains("20000"), "{cut}");
        assert!(cut.unfilled().is_empty(), "{cut}");
        let unnamed = not.left_unnamed(&strings).unwrap();
        assert!(unnamed.text().starts_with("3 things"), "{unnamed}");
    }
}
