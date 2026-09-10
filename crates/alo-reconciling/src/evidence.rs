//! What may be offered as showing a promise, and what is refused as not being
//! evidence at all.
//!
//! Two kinds and no third. **A test** is something a machine runs and a reviewer
//! can run again; **a report** under `docs/autonomy/updates/` is the account of
//! a change, written while the knowledge was fresh, which is what the gate in
//! `CLAUDE.md` already requires of every change. Anything else — a source file
//! with no test in it, a design note, another repository — is a pointer at
//! something nobody can execute, and this audit exists because pointers like
//! that read exactly like evidence.

/// Where a test lives in this repository, as the prefix of its path.
const WHERE_THE_TESTS_ARE: &str = "crates/";

/// What a Rust source file is called.
const A_SOURCE_FILE: &str = ".rs";

/// Where the task reports live.
const WHERE_THE_REPORTS_ARE: &str = "docs/autonomy/updates/";

/// What a report is written in.
const A_DOCUMENT: &str = ".md";

/// What makes a source file a test rather than a file that happens to compile.
///
/// A unit test beside the code is as much evidence as an integration test, and
/// several v0.01 promises are held by one — so the question is whether the file
/// tests anything, not which directory it is in.
const A_TEST: &str = "#[test]";

/// One thing offered as showing a promise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evidence {
    /// A file in this repository that tests something.
    ATest(String),

    /// A task report under `docs/autonomy/updates/`.
    AReport(String),
}

/// What is wrong with a piece of evidence that was named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Amiss {
    /// There is no such file in this repository.
    ///
    /// A test that was renamed and a report that was never written look the
    /// same from the ledger, and both leave a promise with nothing behind it.
    NotThere,

    /// The file is there and tests nothing.
    NothingIsTested,
}

impl Evidence {
    /// What a path names, or [`None`] if it names neither a test nor a report.
    ///
    /// The path is repository-relative, which is how every other document here
    /// names a file, so a ledger entry can be pasted into an editor.
    #[must_use]
    pub fn named(path: &str) -> Option<Self> {
        if path.starts_with(WHERE_THE_TESTS_ARE) && path.ends_with(A_SOURCE_FILE) {
            return Some(Self::ATest(path.to_owned()));
        }
        if path.starts_with(WHERE_THE_REPORTS_ARE) && path.ends_with(A_DOCUMENT) {
            return Some(Self::AReport(path.to_owned()));
        }
        None
    }

    /// Where it is, relative to this repository.
    #[must_use]
    pub fn at(&self) -> &str {
        match self {
            Self::ATest(path) | Self::AReport(path) => path,
        }
    }

    /// Whether it is really there, and really the kind of thing it claims to be.
    ///
    /// `reading` answers with a file's text, or [`None`] where there is no such
    /// file. Nothing here opens anything: the caller decides what repository is
    /// being audited, which is what lets the refusals be shown happening.
    ///
    /// # Errors
    ///
    /// [`Amiss`], saying which of the two ways a named file fails to be
    /// evidence.
    pub fn whether_it_is_there(
        &self,
        reading: &dyn Fn(&str) -> Option<String>,
    ) -> Result<(), Amiss> {
        let Some(text) = reading(self.at()) else {
            return Err(Amiss::NotThere);
        };
        match self {
            Self::ATest(_) if !text.contains(A_TEST) => Err(Amiss::NothingIsTested),
            Self::ATest(_) | Self::AReport(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test, a report, and the things that are neither.
    #[test]
    fn evidence_is_a_test_or_a_report_and_nothing_else() {
        assert_eq!(
            Evidence::named("crates/alo-files/tests/doing.rs"),
            Some(Evidence::ATest(
                "crates/alo-files/tests/doing.rs".to_owned()
            ))
        );
        assert_eq!(
            Evidence::named("crates/alo-dock/src/layout.rs"),
            Some(Evidence::ATest("crates/alo-dock/src/layout.rs".to_owned())),
            "a unit test beside the code is evidence too, and several promises \
             are held by one"
        );
        assert_eq!(
            Evidence::named("docs/autonomy/updates/native-folder-selection.md"),
            Some(Evidence::AReport(
                "docs/autonomy/updates/native-folder-selection.md".to_owned()
            ))
        );
        assert_eq!(
            Evidence::named("docs/decisions/0001-the-agent-boundary.md"),
            None,
            "a decision is where an argument was settled, not evidence that \
             anything was built"
        );
        assert_eq!(
            Evidence::named("ROADMAP.md"),
            None,
            "the release-progress document was accepted as evidence for the \
             release, which is the audit reading its own conclusion back"
        );
        assert_eq!(
            Evidence::named("alo-workplace/web/src/ds/tokens.css"),
            None,
            "a file in another repository was accepted, and nothing here can \
             run it"
        );
    }

    /// A named file that is not there, and one that is there and tests nothing.
    #[test]
    fn a_test_that_is_not_there_is_not_evidence() {
        let repository = |path: &str| match path {
            "crates/alo-files/tests/doing.rs" => Some("#[test]\nfn it_does() {}".to_owned()),
            "crates/alo-files/src/lib.rs" => Some("//! No test in here.".to_owned()),
            "docs/autonomy/updates/native-folder-selection.md" => Some("# A report".to_owned()),
            _ => None,
        };
        let there = |path: &str| {
            Evidence::named(path).map(|evidence| evidence.whether_it_is_there(&repository))
        };

        assert_eq!(there("crates/alo-files/tests/doing.rs"), Some(Ok(())));
        assert_eq!(
            there("docs/autonomy/updates/native-folder-selection.md"),
            Some(Ok(()))
        );
        assert_eq!(
            there("crates/alo-files/tests/a_test_nobody_wrote.rs"),
            Some(Err(Amiss::NotThere)),
            "a promise pointed at a test that does not exist and was reconciled"
        );
        assert_eq!(
            there("crates/alo-files/src/lib.rs"),
            Some(Err(Amiss::NothingIsTested)),
            "a source file with nothing tested in it was accepted as a test, \
             which is how a promise ends up shown by a file that only reads as \
             though it were checked"
        );
    }
}
