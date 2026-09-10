//! The reconciliation: the definition, the ledger, and everything between them
//! that does not add up.
//!
//! Both directions are checked, and the second is the one a reading never does.
//! **Every promise must be answered** — that is the audit. **Every answer must
//! be about a promise the definition still makes** — that is what stops the
//! ledger becoming a record of a release that has moved on, where a rewording
//! nobody noticed leaves a promise reconciled by an entry about the sentence it
//! used to be.
//!
//! Findings are collected rather than returned one at a time. An audit that
//! stopped at the first problem would be read as *one problem*, and the last
//! time these two documents were compared there were six.

use crate::{
    entry::Entry,
    evidence::Evidence,
    finding::Finding,
    ledger::{THE_ENTRIES, entries_in},
    promise::{Promise, promises_in},
};

/// What the release's promises add up to, once everything has been reconciled.
///
/// The counts are the honest shape of v0.01 rather than a score: **wholly
/// shown** and **wholly owed** are the two ends, and *partly* is where nearly
/// every promise in this repository actually sits — code finished, machine never
/// asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reconciled {
    /// How many promises the definition makes at v0.01.
    promises: usize,

    /// How many are shown by evidence with nothing owed on them.
    wholly_shown: usize,

    /// How many are shown in part, with the rest named as owed.
    partly_owed: usize,

    /// How many have no evidence at all and are named as owed.
    wholly_owed: usize,
}

impl Reconciled {
    /// How many promises the definition makes at v0.01.
    #[must_use]
    pub fn promises(&self) -> usize {
        self.promises
    }

    /// How many are shown by evidence with nothing owed on them.
    #[must_use]
    pub fn wholly_shown(&self) -> usize {
        self.wholly_shown
    }

    /// How many are shown in part, with the rest named as owed.
    #[must_use]
    pub fn partly_owed(&self) -> usize {
        self.partly_owed
    }

    /// How many have no evidence at all and are named as owed.
    #[must_use]
    pub fn wholly_owed(&self) -> usize {
        self.wholly_owed
    }
}

/// Every v0.01 promise in a definition, against a ledger's answers about them.
///
/// `reading` answers with the text of a file named by its repository-relative
/// path, or [`None`] where there is no such file. Nothing here opens anything.
///
/// # Errors
///
/// Every [`Finding`], in the order the ledger and then the definition raise
/// them — an entry that is wrong is reported where it is written, and a promise
/// nobody reconciled is reported in the order the definition promises it.
pub fn reconcile(
    features: &str,
    ledger: &str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Result<Reconciled, Vec<Finding>> {
    let promises = promises_in(features);
    let entries = entries_in(ledger);

    if entries.is_empty() && !promises.is_empty() {
        return Err(vec![Finding::NothingToReconcile {
            promises: promises.len(),
            heading: THE_ENTRIES,
        }]);
    }

    let mut findings = Vec::new();
    let mut answered = vec![false; promises.len()];
    let mut reconciled = Reconciled {
        promises: promises.len(),
        wholly_shown: 0,
        partly_owed: 0,
        wholly_owed: 0,
    };

    for entry in &entries {
        let Some(about) = whose_promise(entry, &promises, &mut findings) else {
            continue;
        };
        match answered.get_mut(about) {
            Some(already) if *already => {
                findings.push(Finding::APromiseAnsweredTwice {
                    promise: entry.promise().to_owned(),
                });
            }
            Some(already) => *already = true,
            None => {}
        }
        whether_it_answers(entry, reading, &mut findings, &mut reconciled);
    }

    for (promise, answered) in promises.iter().zip(&answered) {
        if !answered {
            findings.push(Finding::APromiseNobodyReconciled {
                promise: promise.words().to_owned(),
            });
        }
    }

    if findings.is_empty() {
        Ok(reconciled)
    } else {
        Err(findings)
    }
}

/// Which promise an entry is about, or the finding saying it is about none or
/// more than one.
fn whose_promise(
    entry: &Entry,
    promises: &[Promise],
    findings: &mut Vec<Finding>,
) -> Option<usize> {
    let carrying: Vec<usize> = promises
        .iter()
        .enumerate()
        .filter(|(_, promise)| promise.carries(entry.promise()))
        .map(|(at, _)| at)
        .collect();

    match carrying.as_slice() {
        [about] => Some(*about),
        [] => {
            findings.push(Finding::AnEntryAboutNothing {
                quoted: entry.promise().to_owned(),
            });
            None
        }
        several => {
            findings.push(Finding::AnEntryAboutTwoPromises {
                quoted: entry.promise().to_owned(),
                promises: several.len(),
            });
            None
        }
    }
}

/// Whether an entry answers about its promise, and what its answer costs the
/// count.
fn whether_it_answers(
    entry: &Entry,
    reading: &dyn Fn(&str) -> Option<String>,
    findings: &mut Vec<Finding>,
    reconciled: &mut Reconciled,
) {
    if !entry.says_something() {
        findings.push(Finding::APromiseWithNeither {
            promise: entry.promise().to_owned(),
        });
        return;
    }

    for named in entry.names() {
        let Some(evidence) = Evidence::named(named) else {
            findings.push(Finding::SomethingThatIsNotEvidence {
                promise: entry.promise().to_owned(),
                named: named.clone(),
            });
            continue;
        };
        if let Err(amiss) = evidence.whether_it_is_there(reading) {
            findings.push(Finding::EvidenceThatIsNotThere {
                promise: entry.promise().to_owned(),
                named: named.clone(),
                why: amiss.into(),
            });
        }
    }

    match (entry.names().is_empty(), entry.owed()) {
        (false, None) => reconciled.wholly_shown += 1,
        (false, Some(_)) => reconciled.partly_owed += 1,
        (true, Some(_)) => reconciled.wholly_owed += 1,
        (true, None) => {}
    }

    if let Some(owed) = entry.owed()
        && !owed.is_an_answer()
    {
        findings.push(Finding::AShrugRatherThanAnAnswer {
            promise: entry.promise().to_owned(),
            said: owed.sentence().to_owned(),
        });
    }
}
