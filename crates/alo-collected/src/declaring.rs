//! Which crates of this workspace declare words, read out of the workspace.
//!
//! This is the half of the check no list can do. Comparing two lists of crate
//! names would only ever say that the list beside `alo-saying` agrees with
//! another list beside it; what has to be answered is *which crates in this
//! repository actually have words in them*, and the only place that is written
//! down is the crates.
//!
//! # The convention this reads, which is therefore a rule
//!
//! **A crate declares its words in `src/words.rs`, through a `pub fn
//! declare_into` that puts them on somebody else's vocabulary.** Twenty-three
//! crates do exactly that today, `docs/contracts/translations.md` now says so
//! where whoever adds one reads it, and this file is what makes it load-bearing
//! rather than tidy.
//!
//! What it does not catch is a crate that declares words somewhere else on
//! purpose. Nothing mechanical reaches that — a file can always be called
//! something else — which is why the convention is written into the contract as
//! well as read here: the check catches the author who did not know, and the
//! contract answers the author who is deciding.
//!
//! # The manifest is parsed by `alo-by-hand`
//!
//! Which crates a workspace has is one question with one answer, and
//! `alo_by_hand::declaring::members_of` already answers it for the crates that
//! declare verbs. A second parser here would be a second thing to fix the day
//! `Cargo.toml` changes shape, and the two checks would then disagree about what
//! this repository contains — which is the same failure as two lists of crate
//! names, one floor up.

use alo_by_hand::declaring::members_of;

/// Where the workspace says what is in it.
pub const THE_WORKSPACE: &str = alo_by_hand::THE_WORKSPACE;

/// Where a crate declares its words.
const WHERE_THE_WORDS_ARE: &str = "src/words.rs";

/// How a crate hands its words to somebody else's vocabulary.
const HANDS_THEM_OVER: &str = "pub fn declare_into";

/// Every crate of this workspace that declares words, by name.
///
/// `reading` answers with the text of a file named by its repository-relative
/// path, or [`None`] where there is no such file. Nothing here opens anything —
/// which is what lets every finding this crate exists for be shown happening
/// against a fixture, rather than only after somebody has written the crate that
/// would cause it.
#[must_use]
pub fn whoever_declares_words(
    manifest: &str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Vec<String> {
    members_of(manifest)
        .iter()
        .filter(|member| {
            reading(&format!("{member}/{WHERE_THE_WORDS_ARE}"))
                .is_some_and(|source| source.contains(HANDS_THEM_OVER))
        })
        .filter_map(|member| member.rsplit('/').next().map(str::to_owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A workspace written the way this one is written, with a commented-out
    /// member and an `exclude` list after the members.
    const A_WORKSPACE: &str = "\
[workspace]
resolver = \"3\"
members = [
  \"crates/alo-telling\",
  \"crates/alo-strings\",
  # \"crates/alo-nothing\", not a member yet
  \"crates/alo-collected\",
]
exclude = [
  \"crates/alo-bounding-kernel\",
]
";

    /// **A crate that declares words is one whose `words.rs` hands them over.**
    /// `alo-strings` is the machinery every other crate declares *into* and has
    /// no words of its own; a crate with no `words.rs` at all is not one either.
    #[test]
    fn the_crates_that_declare_words_are_the_ones_that_hand_them_over() {
        let workspace = |path: &str| match path {
            "crates/alo-telling/src/words.rs" => {
                Some("pub fn declare_into(vocabulary: &mut Vocabulary) {}".to_owned())
            }
            "crates/alo-strings/src/words.rs" => {
                Some("pub fn phrase(&self, key: &Key) -> Option<&Phrase> {}".to_owned())
            }
            _ => None,
        };
        assert_eq!(
            whoever_declares_words(A_WORKSPACE, &workspace),
            ["alo-telling".to_owned()],
            "the machinery the words are declared into was read as a crate that \
             declares words, or the crate that declares them was missed"
        );
    }

    /// A manifest this parser does not understand yields nothing, which is what
    /// [`crate::finding::Finding::NoWorkspaceToWalk`] is for: an empty list is
    /// never quietly taken for a workspace where nothing says anything.
    #[test]
    fn a_manifest_that_says_nothing_yields_nothing() {
        let anything = |_: &str| Some("pub fn declare_into".to_owned());
        for unreadable in [
            "",
            "[workspace]\nresolver = \"3\"\n",
            "members = [\n  \"crates/alo-telling\",\n",
        ] {
            assert!(
                whoever_declares_words(unreadable, &anything).is_empty(),
                "{unreadable:?}"
            );
        }
    }

    /// A crate whose `verbs.rs` hands verbs over is not thereby a crate with
    /// words: the two checks read two files, and `alo-files` happens to do both.
    #[test]
    fn a_verb_list_is_not_a_word_list() {
        let only_verbs = |path: &str| match path {
            "crates/alo-telling/src/verbs.rs" => {
                Some("pub fn declare_into(verbs: &mut Verbs) {}".to_owned())
            }
            _ => None,
        };
        assert!(
            whoever_declares_words(A_WORKSPACE, &only_verbs).is_empty(),
            "a crate that declares verbs was read as a crate that declares words"
        );
    }
}
