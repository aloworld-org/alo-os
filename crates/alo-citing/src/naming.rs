//! Where this repository points at a decision by the name of its file.
//!
//! The other half of a pointer. `ADR 0016` is the form a sentence uses; a link
//! in a document and a path in a comment use the filename, and a filename is the
//! pointer a reader is least likely to check, because it looks like it was
//! copied rather than typed. It is also the one that rots on its own: a decision
//! renamed leaves every link to it reading exactly as it did before.
//!
//! # What is read as one
//!
//! **A token ending in `.md` whose last segment begins with four digits and a
//! hyphen.** That is the shape every decision's filename has, in both the forms
//! this repository writes — `docs/decisions/0024-what-a-person-signs-in-at.md`
//! from anywhere, and the bare `0016-the-organisation-bounds-and-the-person-chooses.md`
//! a link inside another decision uses.
//!
//! Nothing else in this repository is named that way, which is what makes the
//! shape usable as the rule; a document elsewhere that starts its name with four
//! digits and a hyphen would be read as a decision and would have to be renamed.
//! That is written here rather than discovered, and it is a small price for
//! catching a link nobody would otherwise open until it mattered.

use crate::citation::Named;

/// What a decision's filename ends in.
const ENDS_IN: &str = ".md";

/// How many digits a decision's number has.
const DIGITS: usize = 4;

/// What may be part of a filename or the path in front of it.
fn is_part_of_a_path(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/')
}

/// Every reference to a decision by the name of its file in one file, in the
/// order they are written.
#[must_use]
pub fn named_in(file: &str, text: &str) -> Vec<Named> {
    let mut named = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for (ends, _) in line.match_indices(ENDS_IN) {
            let Some(token) = line
                .get(..ends.saturating_add(ENDS_IN.len()))
                .map(|up_to_here| {
                    up_to_here
                        .rsplit(|character| !is_part_of_a_path(character))
                        .next()
                        .unwrap_or_default()
                })
            else {
                continue;
            };
            let filename = token.rsplit('/').next().unwrap_or(token);
            if looks_like_a_decision(filename) {
                named.push(Named::new(
                    filename.to_owned(),
                    file.to_owned(),
                    index.saturating_add(1),
                ));
            }
        }
    }
    named
}

/// Whether a filename is written the way a decision's filename is written.
fn looks_like_a_decision(filename: &str) -> bool {
    let digits: String = filename
        .chars()
        .take(DIGITS)
        .take_while(char::is_ascii_digit)
        .collect();
    digits.len() == DIGITS
        && filename.chars().nth(DIGITS) == Some('-')
        && filename.ends_with(ENDS_IN)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names of every decision cited in a text.
    fn names(text: &str) -> Vec<String> {
        named_in("a-file.md", text)
            .iter()
            .map(|named| named.named().to_owned())
            .collect()
    }

    /// Both forms this repository writes: the path from anywhere, and the bare
    /// filename a link inside another decision uses.
    #[test]
    fn both_forms_this_repository_writes_are_read() {
        assert_eq!(
            names("see `docs/decisions/0024-what-a-person-signs-in-at.md` for the options"),
            ["0024-what-a-person-signs-in-at.md"]
        );
        assert_eq!(
            names("[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md) bounds"),
            ["0016-the-organisation-bounds-and-the-person-chooses.md"]
        );
    }

    /// A document that is not named the way a decision is named is not read as
    /// one, which is most of this repository.
    #[test]
    fn a_document_named_any_other_way_is_not_a_decision() {
        assert_eq!(
            names("docs/autonomy/updates/every-verbs-by-hand-answer.md and CHANGELOG.md"),
            [] as [&str; 0]
        );
        assert_eq!(
            names("0024.md is not, and 024-short.md is not"),
            [] as [&str; 0]
        );
    }

    /// More than one on a line is more than one citation, because a context line
    /// in a decision names half a dozen.
    #[test]
    fn a_line_may_name_more_than_one() {
        assert_eq!(
            names("(0008-where-inference-happens.md), (0012-gpl-and-the-machine-is-the-owners.md)"),
            [
                "0008-where-inference-happens.md",
                "0012-gpl-and-the-machine-is-the-owners.md"
            ]
        );
    }

    /// And where it was read is kept, because a finding about a link is acted on
    /// by opening that line.
    #[test]
    fn where_it_was_read_is_kept() {
        let named = named_in(
            "docs/features.md",
            "nothing\nnothing\n(0011-the-base-is-rented-and-the-image-is-a-container.md)",
        );
        assert_eq!(named.first().map(Named::line), Some(3));
        assert_eq!(named.first().map(Named::file), Some("docs/features.md"));
    }
}
