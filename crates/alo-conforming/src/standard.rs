//! Which standard this is about, and what this list of it is **not**.
//!
//! # The version is named, so a revision is a visible change
//!
//! A conformance file that does not say which version of a standard it was
//! written against is a file nobody can check. [`THE_VERSION`] is in the type
//! system and in every report this crate could feed, and the day somebody moves
//! to a later one, every clause here is a line in a diff rather than a sentence
//! that quietly changed meaning.
//!
//! # What this list is
//!
//! **Evidence for a person to write a report from.** Each clause carries what
//! this repository can show for it: a test that exists and runs in the gate, a
//! task in a plan that has not been done yet, or a reason it does not apply.
//!
//! # What this list is not
//!
//! - **It is not a conformance claim.** `docs/features.md` puts the report at
//!   v1; this is v0.5 building what a report will rest on. Nothing here says
//!   alo OS conforms to anything.
//! - **It is not the standard's text.** EN 301 549 is published by ETSI, CEN and
//!   CENELEC and is somebody else's copyright. Every requirement here is **one
//!   sentence in this repository's own words** about what the clause asks for,
//!   written so that a reader can tell which clause is meant. A person writing
//!   the report reads the standard.
//! - **It has not been checked against the published text.** Every clause here
//!   carries [`crate::Checked`], and today **every one of them says it has not
//!   been**: the numbers and the one-sentence readings were written from this
//!   lane's knowledge of the standard, with the text not in front of it. That is
//!   worth having — it is a list to check rather than a list to trust — and it
//!   is worth saying in the loudest available voice, which is a field on every
//!   row and a count in [`crate::Held`].
//!
//! A clause number that turns out to be wrong is therefore **a thing this file
//! expects**, and the shape it fails in is a person checking it and correcting
//! it, rather than a report going out with it inside.

/// The standard these clauses are from.
pub const THE_STANDARD: &str = "EN 301 549";

/// The version they were written against.
pub const THE_VERSION: &str = "V3.2.1 (2021-03)";

/// Where a reader of this repository finds the shell's accessibility work.
pub const THE_PLAN: &str = "docs/autonomy/v0-5-access-and-language-plan.md";

#[cfg(test)]
mod tests {
    use super::*;

    /// **The version is named and is a version**, because a conformance file
    /// that does not say which edition it read is one nobody can check.
    #[test]
    fn the_standard_and_its_version_are_named() {
        assert!(THE_STANDARD.starts_with("EN "));
        assert!(THE_VERSION.starts_with('V'), "{THE_VERSION}");
        assert!(
            THE_VERSION.contains("20"),
            "the version says nothing about when it was published: {THE_VERSION}"
        );
    }
}
