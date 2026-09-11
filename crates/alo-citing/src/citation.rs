//! What a citation is: a number, the words it was written in, and where.
//!
//! Everything this crate refuses names all three, because the reader of a
//! finding is whoever wrote the line — and *an ADR reference does not resolve*
//! would send them to read a directory rather than to look at one sentence they
//! have open.
//!
//! A citation deliberately does **not** carry what the citing text claims the
//! decision says. Whether `ADR 0001 §3` really is about granting a folder is a
//! reader's job and nothing mechanical reaches it; this crate judges the
//! pointer, never the argument.

/// One reference to a decision, as it was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    /// The number as written, four digits, so `0016` and not `16`.
    number: String,

    /// The citation's own words, quoted back to whoever wrote them.
    said: String,

    /// The repository-relative path of the file it is in.
    file: String,

    /// Which line of that file, counted from one.
    line: usize,

    /// The repository whose decisions it points into, where that is not this
    /// one.
    ///
    /// `Some` is a citation of a neighbouring repository's decisions —
    /// `alo-workplace`'s, in the four places this repository carries one — and
    /// nothing under `docs/decisions/` here answers for it.
    elsewhere: Option<String>,
}

impl Citation {
    /// One reference to a decision, as it was written.
    #[must_use]
    pub fn new(
        number: String,
        said: String,
        file: String,
        line: usize,
        elsewhere: Option<String>,
    ) -> Self {
        Self {
            number,
            said,
            file,
            line,
            elsewhere,
        }
    }

    /// The number as written, four digits.
    #[must_use]
    pub fn number(&self) -> &str {
        &self.number
    }

    /// The citation's own words.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// The repository-relative path of the file it is in.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Which line of that file, counted from one.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// The repository whose decisions it points into, where that is not this
    /// one.
    #[must_use]
    pub fn elsewhere(&self) -> Option<&str> {
        self.elsewhere.as_deref()
    }
}

/// One reference to a decision by the name of its file.
///
/// The other half of how this repository points at a decision: a link in a
/// document, or a path in a comment. A filename is the pointer a reader is most
/// likely to follow and least likely to check, because it looks like it was
/// copied rather than typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    /// The filename as written, without any directory in front of it.
    named: String,

    /// The repository-relative path of the file it is in.
    file: String,

    /// Which line of that file, counted from one.
    line: usize,
}

impl Named {
    /// One reference to a decision by the name of its file.
    #[must_use]
    pub fn new(named: String, file: String, line: usize) -> Self {
        Self { named, file, line }
    }

    /// The filename as written.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// The repository-relative path of the file it is in.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Which line of that file, counted from one.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A citation keeps the words it was written in rather than a tidied form,
    /// so whoever has the line open recognises it.
    #[test]
    fn a_citation_quotes_what_was_written() {
        let citation = Citation::new(
            "0015".to_owned(),
            "ADR-0015".to_owned(),
            "docs/autonomy/updates/a-report.md".to_owned(),
            144,
            None,
        );
        assert_eq!(citation.number(), "0015");
        assert_eq!(citation.said(), "ADR-0015");
        assert_eq!(citation.line(), 144);
        assert_eq!(citation.elsewhere(), None);
    }

    /// And one into a neighbouring repository says so, because nothing under
    /// this repository's `docs/decisions/` answers for it.
    ///
    /// The words are assembled rather than written out, for the reason
    /// `tests/every_decision_this_repository_points_at.rs` gives in full: a
    /// number this repository has no decision for, spelled out beside the word
    /// that makes it a citation, would be a citation this file really carries.
    #[test]
    fn a_citation_elsewhere_names_the_repository() {
        let elsewheres = "0047";
        let citation = Citation::new(
            elsewheres.to_owned(),
            format!("ADR {elsewheres}"),
            "docs/contracts/agent-verbs.md".to_owned(),
            131,
            Some("alo-workplace".to_owned()),
        );
        assert_eq!(citation.elsewhere(), Some("alo-workplace"));
        assert_eq!(citation.file(), "docs/contracts/agent-verbs.md");
    }

    /// A filename citation keeps the name and where it was read.
    #[test]
    fn a_named_file_keeps_the_name_and_where() {
        let named = Named::new(
            "0016-the-organisation-bounds-and-the-person-chooses.md".to_owned(),
            "docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md".to_owned(),
            9,
        );
        assert!(named.named().starts_with("0016-"));
        assert_eq!(named.line(), 9);
        assert!(named.file().ends_with(".md"));
    }
}
