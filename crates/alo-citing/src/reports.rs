//! Every task report this repository points at, held to being one somebody
//! wrote — and to being written so that something other than a reader can find
//! it.
//!
//! This is the same fault as a citation of a decision nobody made, in the other
//! family of documents this repository points into. `docs/autonomy/updates/`
//! holds the account of every change, `ROADMAP.md` offers those accounts as
//! evidence for its boxes, and the reports cite each other. A reader believes a
//! filename without opening it for exactly the reason they believe an ADR number:
//! it looks like a fact.
//!
//! # What was found, and why a convention was not enough
//!
//! On 2026-09-28 the repository held **284** report citations written short, as
//! `updates/x.md`. Resolved relative to the file doing the citing, 238 land —
//! every one of them in `docs/autonomy/COMPOSITOR.md`, which sits in
//! `docs/autonomy/` where that form is correct. The other **46** resolve under no
//! rule at all: 33 in `ROADMAP.md`, which is at the repository root, and 13 in
//! reports citing their siblings from inside `docs/autonomy/updates/`, where the
//! short form would mean `docs/autonomy/updates/updates/x.md`.
//!
//! **Every one of those 46 documents exists.** Nothing was missing and no claim
//! was false; the same file was cited three ways and one of the three could not be
//! resolved by anything but a person who already knew the convention. That is
//! precisely as useful as no citation, and worse than none in one respect: it
//! reads as though it has been checked.
//!
//! A convention is what those 46 already violated, which is why this is a module
//! and not a paragraph in a style guide. The rule is held here, mechanically, in
//! the crate that already holds every ADR pointer to landing.
//!
//! # The rule, stated so a machine can apply it
//!
//! A citation resolves when it is written **from the repository root** —
//! `docs/autonomy/updates/x.md`. The short form `updates/x.md` is legal in a file
//! **directly inside `docs/autonomy/`**, and nowhere else, because that is the one
//! place it resolves relative to itself.
//!
//! The short form is kept legal there rather than banned outright for a reason
//! worth stating: `COMPOSITOR.md` uses it 238 times correctly, and a rule that
//! made 238 true sentences into findings to gain uniformity would be this check
//! demanding that correct writing be changed. What it may not do is resolve
//! *somewhere*: `../updates/x.md` from a report is refused for the same reason
//! `../decisions/` was refused in a ledger entry — a path a checker cannot follow
//! is a path nobody checks.
//!
//! # Two faults, because a pointer fails in two ways
//!
//! | | |
//! |---|---|
//! | The report named does not exist | [`Amiss::AReportNobodyWrote`] |
//! | The report exists and the citation cannot be resolved from where it is written | [`Amiss::ShortenedWhereItDoesNotResolve`] |
//!
//! The second is not the smaller of the two. A citation of a report that was never
//! written is caught the first time somebody looks; a citation that resolves only
//! for the person who wrote it survives every reading, because each reader who
//! knows the convention silently supplies the prefix and each tool silently
//! reports nothing.
//!
//! # It reads no disk, like the rest of this crate
//!
//! [`cited_in`] is handed a file's name and its text; [`held`] is handed those
//! citations and the names of the reports that exist. The caller decides what
//! repository is being checked, which is what lets both faults be shown happening
//! in a test rather than described.

use core::fmt;

/// Where the reports live, from the repository root.
const WHERE_THE_REPORTS_ARE: &str = "docs/autonomy/updates/";

/// The one directory the short form may be written in, because it resolves there.
const WHERE_THE_SHORT_FORM_RESOLVES: &str = "docs/autonomy/";

/// What the short form looks like.
const SHORTENED: &str = "updates/";

/// What a report is written in.
const A_DOCUMENT: &str = ".md";

/// How a citation was written, which decides whether anything can follow it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// `docs/autonomy/updates/x.md` — followable from the root, from anywhere.
    FromTheRoot,

    /// `updates/x.md` or `../updates/x.md` — followable only from
    /// `docs/autonomy/`.
    Shortened,
}

/// One report citation, as the text actually has it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cited {
    /// The report's own filename, with no directory in front of it.
    named: String,

    /// The file the citation was written in, from the repository root.
    file: String,

    /// Which line of that file, counting from one.
    line: usize,

    /// Which of the spellings was used, which is what decides whether anything
    /// can follow it from here.
    written: Written,
}

impl Cited {
    /// The report's own filename, without any directory.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// The file the citation is written in, from the repository root.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Which line of it, counting from one.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// How it was written.
    #[must_use]
    pub fn written(&self) -> Written {
        self.written
    }

    /// Whether anything reading this repository could follow it from where it is.
    ///
    /// The short form resolves in a file directly inside `docs/autonomy/` and
    /// nowhere else — not in a subdirectory of it, which is where 13 of the 46
    /// were.
    #[must_use]
    pub fn resolves(&self) -> bool {
        match self.written {
            Written::FromTheRoot => true,
            Written::Shortened => self
                .file
                .strip_prefix(WHERE_THE_SHORT_FORM_RESOLVES)
                .is_some_and(|rest| !rest.contains('/')),
        }
    }
}

/// What is wrong with a report citation that was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Amiss {
    /// It names a report that is not in this repository.
    AReportNobodyWrote {
        /// The filename cited.
        named: String,
        /// Where the citation is.
        file: String,
        /// Which line.
        line: usize,
    },

    /// The report is there, and the citation cannot be followed from where it
    /// is written.
    ShortenedWhereItDoesNotResolve {
        /// The filename cited.
        named: String,
        /// Where the citation is.
        file: String,
        /// Which line.
        line: usize,
    },
}

impl fmt::Display for Amiss {
    fn fmt(&self, into: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AReportNobodyWrote { named, file, line } => write!(
                into,
                "{file}:{line} cites the report `{named}`, and there is no \
                 {WHERE_THE_REPORTS_ARE}{named} in this repository. Either the report was never \
                 written or it was renamed and this sentence was left behind — and both read, to \
                 somebody who does not open it, exactly like evidence."
            ),
            Self::ShortenedWhereItDoesNotResolve { named, file, line } => write!(
                into,
                "{file}:{line} cites `{SHORTENED}{named}`, which exists at \
                 {WHERE_THE_REPORTS_ARE}{named} and cannot be followed from here. Write it \
                 `{WHERE_THE_REPORTS_ARE}{named}`. The short form is legal only in a file \
                 directly inside {WHERE_THE_SHORT_FORM_RESOLVES}, where it resolves relative to \
                 itself; from anywhere else it is a path only a reader who already knows the \
                 convention can follow, which is the same as one nobody checks."
            ),
        }
    }
}

/// Every report citation in one file's text.
///
/// Citations are read out of the sentences rather than out of a list kept here,
/// so a citation added anywhere is one this check sees, with nothing to update —
/// the same rule the rest of this crate follows.
///
/// A citation is taken as one only inside backticks, because that is how this
/// repository writes a path and because prose about *the updates directory*
/// should not become a finding.
#[must_use]
pub fn cited_in(file: &str, text: &str) -> Vec<Cited> {
    let mut found = Vec::new();
    for (counted, line) in text.lines().enumerate() {
        for piece in line.split('`').skip(1).step_by(2) {
            let Some(named) = the_report_named(piece) else {
                continue;
            };
            found.push(Cited {
                named: named.to_owned(),
                file: file.to_owned(),
                line: counted + 1,
                written: if piece.starts_with(WHERE_THE_REPORTS_ARE) {
                    Written::FromTheRoot
                } else {
                    Written::Shortened
                },
            });
        }
    }
    found
}

/// The report a backticked piece names, if it names one at all.
///
/// Three spellings are read, and only three: from the root, shortened, and
/// shortened with a `../` in front of it. Anything else — a path into another
/// directory, a sentence with a space in it — is not a citation of a report and
/// is left alone rather than guessed at.
fn the_report_named(piece: &str) -> Option<&str> {
    let named = piece.strip_prefix(WHERE_THE_REPORTS_ARE).or_else(|| {
        piece
            .strip_prefix("../")
            .unwrap_or(piece)
            .strip_prefix(SHORTENED)
    })?;
    if named.ends_with(A_DOCUMENT) && !named.contains('/') && !named.contains(' ') {
        Some(named)
    } else {
        None
    }
}

/// Hold every citation to landing, and to being followable from where it is.
///
/// `reports` is the filename of every report that exists, without its directory.
/// Nothing here opens anything.
#[must_use]
pub fn held(cited: &[Cited], reports: &[&str]) -> Vec<Amiss> {
    let mut amiss = Vec::new();
    for citation in cited {
        if !reports.contains(&citation.named()) {
            amiss.push(Amiss::AReportNobodyWrote {
                named: citation.named().to_owned(),
                file: citation.file().to_owned(),
                line: citation.line(),
            });
        } else if !citation.resolves() {
            amiss.push(Amiss::ShortenedWhereItDoesNotResolve {
                named: citation.named().to_owned(),
                file: citation.file().to_owned(),
                line: citation.line(),
            });
        }
    }
    amiss
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three spellings are read, and prose about a directory is not.
    #[test]
    fn a_citation_is_read_however_it_is_spelled_and_prose_is_not() {
        let text = "\
See `docs/autonomy/updates/a.md` for it.
Also `updates/b.md` and `../updates/c.md`.
The updates directory holds them, and `docs/features.md` does not.
A sentence with `updates/not a report.md` in it is not one.
";
        // Every citation at once rather than one subscript at a time: the whole
        // list is what is being asserted, and a mismatch anywhere names all three.
        assert_eq!(
            cited_in("ROADMAP.md", text)
                .iter()
                .map(|one| (one.named().to_owned(), one.written(), one.line()))
                .collect::<Vec<_>>(),
            vec![
                ("a.md".to_owned(), Written::FromTheRoot, 1),
                ("b.md".to_owned(), Written::Shortened, 2),
                ("c.md".to_owned(), Written::Shortened, 2),
            ]
        );
    }

    /// **The short form resolves in `docs/autonomy/` and not below it**, which is
    /// the distinction 13 of the 46 fell into.
    #[test]
    fn the_short_form_resolves_in_one_directory_and_not_in_its_children() {
        let shortened = |file: &str| {
            cited_in(file, "see `updates/a.md`")
                .first()
                .is_some_and(Cited::resolves)
        };
        assert!(shortened("docs/autonomy/COMPOSITOR.md"));
        assert!(!shortened("docs/autonomy/updates/a-report.md"));
        assert!(!shortened("ROADMAP.md"));
        // A real path. This line first named an invented decision file instead,
        // and the crate's own check refused it — then refused the comment that
        // explained the refusal, because that comment still spelled the invented
        // name out. Twice in one file is the strongest argument available that the
        // rule has to be mechanical: its author could not write four lines about
        // it without breaking it.
        assert!(!shortened("docs/features.md"));

        // From the root it resolves wherever it is written.
        assert!(
            cited_in("ROADMAP.md", "see `docs/autonomy/updates/a.md`")
                .first()
                .is_some_and(Cited::resolves)
        );
    }

    /// A report nobody wrote is the first finding, and it is preferred to the
    /// second: telling somebody to rewrite a path to a file that is not there
    /// would be the less useful of the two sentences.
    #[test]
    fn a_report_nobody_wrote_is_the_finding_even_when_it_is_also_shortened() {
        let cited = cited_in("ROADMAP.md", "see `updates/never-written.md`");
        let amiss = held(&cited, &["a.md"]);
        assert_eq!(
            amiss,
            vec![Amiss::AReportNobodyWrote {
                named: "never-written.md".to_owned(),
                file: "ROADMAP.md".to_owned(),
                line: 1,
            }]
        );
        let said = amiss.first().map(Amiss::to_string).unwrap_or_default();
        assert!(
            said.contains("docs/autonomy/updates/never-written.md"),
            "{said}"
        );
    }

    /// A report that exists, cited where nothing can follow it, and the sentence
    /// says what to write instead.
    #[test]
    fn a_citation_only_its_writer_can_resolve_is_the_other_finding() {
        let cited = cited_in(
            "docs/autonomy/updates/one-report.md",
            "see `updates/another.md`",
        );
        let amiss = held(&cited, &["another.md", "one-report.md"]);
        assert_eq!(
            amiss,
            vec![Amiss::ShortenedWhereItDoesNotResolve {
                named: "another.md".to_owned(),
                file: "docs/autonomy/updates/one-report.md".to_owned(),
                line: 1,
            }]
        );
        let said = amiss.first().map(Amiss::to_string).unwrap_or_default();
        assert!(
            said.contains("Write it `docs/autonomy/updates/another.md`"),
            "{said}"
        );
    }

    /// The form that is right everywhere is held to nothing.
    #[test]
    fn a_citation_written_from_the_root_is_no_finding_anywhere() {
        for file in [
            "ROADMAP.md",
            "docs/autonomy/COMPOSITOR.md",
            "docs/autonomy/updates/one-report.md",
            "CHANGELOG.md",
        ] {
            let cited = cited_in(file, "see `docs/autonomy/updates/a.md`");
            assert_eq!(held(&cited, &["a.md"]), vec![], "{file}");
        }
    }
}
