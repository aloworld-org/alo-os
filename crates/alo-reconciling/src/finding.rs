//! What the audit found, in sentences whoever is reconciling the release acts
//! on.
//!
//! Every finding names the promise or the entry it is about and says what to do
//! about it. That is not politeness: a finding is read months after the change
//! that caused it, by somebody who was not there, and *the ledger is out of
//! date* would send them to read both documents from the top — which is the
//! seven-times-over reading this crate exists to end.

use crate::evidence::Amiss;

/// One thing wrong between the definition and the ledger.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Finding {
    /// A promise nothing in the ledger is about.
    ///
    /// The one this audit exists for, and the one that really happened six
    /// times: a promise arrives in `docs/features.md` and nobody reconciles it,
    /// so the release believes it is complete because nothing is asking.
    #[error(
        "docs/features.md promises `{promise}` and the ledger says nothing about it. Every v0.01 \
         promise is reconciled in docs/autonomy/v0-01-evidence.md — name the test or the report \
         that shows it, or say what is owed. A promise nothing asks about is how six of these were \
         missed one at a time"
    )]
    APromiseNobodyReconciled {
        /// The promise, as `docs/features.md` words it.
        promise: String,
    },

    /// An entry answering about a promise the definition no longer makes.
    ///
    /// Either the promise was reworded — in which case the evidence needs
    /// looking at again, by whoever reworded it — or it was withdrawn, in which
    /// case the entry goes with it.
    #[error(
        "the ledger has an entry for `{quoted}`, and no v0.01 promise in docs/features.md contains \
         those words. If the promise was reworded, quote it again and look at the evidence while \
         you are there; if it was withdrawn, the entry goes with it"
    )]
    AnEntryAboutNothing {
        /// The phrase the entry quotes.
        quoted: String,
    },

    /// An entry whose quotation fits more than one promise.
    ///
    /// Left alone, one of the two promises would be reconciled by an entry that
    /// was never about it — which is worse than not being reconciled, because it
    /// reads as done.
    #[error(
        "the ledger's entry `{quoted}` matches {promises} v0.01 promises in docs/features.md. \
         Quote enough of the promise to name one of them: an entry that fits two reconciles \
         whichever it reached first, and the other reads as done"
    )]
    AnEntryAboutTwoPromises {
        /// The phrase the entry quotes.
        quoted: String,
        /// How many promises carry it.
        promises: usize,
    },

    /// A promise two entries answer about.
    #[error(
        "two entries in the ledger are about the promise `{promise}`. One promise, one entry — two \
         answers about one promise is two people's knowledge of it, and the shorter answer is the \
         one that gets read"
    )]
    APromiseAnsweredTwice {
        /// The promise, as `docs/features.md` words it.
        promise: String,
    },

    /// A promise with no evidence and nothing owed.
    ///
    /// **This is the finding the task exists to produce**, and it is written
    /// down before anything else is.
    #[error(
        "the ledger's entry for `{promise}` names no test and no report, and says nothing is owed. \
         One or the other: a promise with neither is a promise nobody can show and nobody has \
         admitted to"
    )]
    APromiseWithNeither {
        /// The phrase the entry quotes.
        promise: String,
    },

    /// A named file that is not evidence at all.
    #[error(
        "the ledger's entry for `{promise}` offers `{named}` as showing it, and that is neither a \
         test in this repository nor a report under docs/autonomy/updates/. Evidence is something \
         somebody can run or read; a decision, a design note or a file in another repository is \
         not"
    )]
    SomethingThatIsNotEvidence {
        /// The phrase the entry quotes.
        promise: String,
        /// What it named.
        named: String,
    },

    /// A named file that is not there, or is there and tests nothing.
    #[error(
        "the ledger's entry for `{promise}` names `{named}`, and {why}. Evidence that moved is \
         evidence nobody can run, and it reads exactly like evidence that is still there"
    )]
    EvidenceThatIsNotThere {
        /// The phrase the entry quotes.
        promise: String,
        /// What it named.
        named: String,
        /// Which of the two ways it fails to be evidence.
        why: NotEvidence,
    },

    /// What is owed, said in a way nobody could act on.
    #[error(
        "the ledger's entry for `{promise}` says `{said}` is what is still owed. That is a shrug \
         rather than an answer: name the thing that is missing and why it is not done, the way the \
         entries around it do"
    )]
    AShrugRatherThanAnAnswer {
        /// The phrase the entry quotes.
        promise: String,
        /// What it said.
        said: String,
    },

    /// A promise with no evidence at all, and nothing saying where the work is.
    ///
    /// Being owed is not the finding — four v0.01 promises are, honestly. The
    /// finding is being owed with nowhere for the next reader to go: a decision
    /// to make, or a task in a plan. Without one of those the sentence is a
    /// verdict, and whoever inherits it starts the reading again.
    #[error(
        "the ledger's entry for `{promise}` names no test and no report, and does not say where \
         the work is. A promise with no evidence at all names the decision it waits on under \
         docs/decisions/, or the task that is the increment — as `task <n> of \
         `docs/autonomy/<plan>.md``, with the plan beside the number. A promise that is missing \
         and points nowhere is one the next reader derives again from scratch"
    )]
    APromiseOwedWithNowhereToGo {
        /// The phrase the entry quotes.
        promise: String,
    },

    /// A promise waiting on something nobody can find.
    ///
    /// The one kind of pointer a reader believes without opening, because the
    /// number looks like a fact — the argument `alo-citing` makes about ADR
    /// citations, about the place a promise with nothing behind it comes to rest.
    #[error(
        "the ledger's entry for `{promise}` says it waits on `{waiting}`, and {why}. A promise \
         sent somewhere nobody can follow reads exactly like a promise somebody answered"
    )]
    AWaitNobodyCanFollow {
        /// The phrase the entry quotes.
        promise: String,
        /// The pointer, as the entry wrote it.
        waiting: String,
        /// Which of the three ways it cannot be followed.
        why: crate::waiting::NoSuchWait,
    },

    /// A ledger with nothing in it, against a definition that promises things.
    ///
    /// Everything else here is about one promise. This is the failure that would
    /// otherwise be silent: a heading that moved, a file that was replaced, and
    /// an audit passing over a document it never found.
    #[error(
        "docs/features.md makes {promises} v0.01 promises and the ledger has no entries at all, so \
         this audit is checking nothing. The entries live under `{heading}` in \
         docs/autonomy/v0-01-evidence.md; if that heading moved, this crate moves with it"
    )]
    NothingToReconcile {
        /// How many promises went unreconciled.
        promises: usize,
        /// The heading the entries are read from.
        heading: &'static str,
    },
}

/// Which of the two ways a named file fails to be evidence, in words.
///
/// Its own type rather than a sentence built at the point of the finding: the
/// two cases are told apart by whoever fixes them — a path to correct, or a file
/// to write a test in — and a `Display` that spelt them out inline would put
/// that distinction in a format string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotEvidence {
    /// There is no such file.
    #[error("there is no such file in this repository")]
    NotThere,

    /// The file is there and tests nothing.
    #[error("that file tests nothing, so it shows a promise only by being pointed at")]
    NothingIsTested,
}

impl From<Amiss> for NotEvidence {
    fn from(amiss: Amiss) -> Self {
        match amiss {
            Amiss::NotThere => Self::NotThere,
            Amiss::NothingIsTested => Self::NothingIsTested,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every finding names the promise it is about and what to do next, because
    /// it is read by somebody who was not there when it was caused.
    #[test]
    fn a_finding_names_the_promise_and_what_to_do() {
        let said = Finding::APromiseNobodyReconciled {
            promise: "Copy, cut and paste".to_owned(),
        }
        .to_string();
        assert!(said.contains("Copy, cut and paste"), "{said}");
        assert!(said.contains("docs/autonomy/v0-01-evidence.md"), "{said}");
    }

    /// And the two ways a named file is not evidence stay two, so the sentence
    /// says which fix it is.
    #[test]
    fn evidence_that_is_not_there_says_which_way() {
        assert_eq!(
            NotEvidence::from(Amiss::NotThere).to_string(),
            "there is no such file in this repository"
        );
        assert!(
            NotEvidence::from(Amiss::NothingIsTested)
                .to_string()
                .contains("tests nothing")
        );
    }
}
