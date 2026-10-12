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
    entry::Entry, evidence::Evidence, finding::Finding, owed::Owed, promise::Promise,
    waiting::Waiting,
};

/// What the release's promises add up to, once everything has been reconciled.
///
/// The counts are the honest shape of v0.01 rather than a score: **wholly
/// shown** and **wholly owed** are the two ends, and *partly* is where nearly
/// every promise in this repository actually sits — code finished, machine never
/// asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reconciled {
    /// How many promises the definition makes.
    promises: usize,

    /// How many entries the ledger holds about them.
    ///
    /// **Counted separately from the promises since the version tiers went.**
    /// They were the same number when a release's ledger answered a release's
    /// promises one for one; now the document makes every promise alo OS will
    /// ever keep and the ledger holds entries for the ones somebody has looked
    /// at. The three categories below sum to **this**, because they count
    /// entries — and asserting they summed to the promises was the thing that
    /// stopped holding.
    entries: usize,

    /// How many are shown by evidence with nothing owed on them.
    wholly_shown: usize,

    /// How many are shown in part, with the rest named as owed.
    partly_owed: usize,

    /// How many have no evidence at all and are named as owed.
    wholly_owed: usize,
}

impl Reconciled {
    /// How many entries the ledger holds.
    #[must_use]
    pub const fn entries(self) -> usize {
        self.entries
    }

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
    reconcile_these(
        &crate::promise::promises_in(features),
        ledger,
        THE_LEDGERS_HEADING,
        reading,
    )
}

/// The heading the ledger gathers its entries under.
///
/// **One heading, now that there are no tiers.** It still says *v0.01* because
/// that is the text in the file and this constant must match it; the file's own
/// heading changes when somebody renames it, and this is where they will find
/// out that they must.
pub const THE_LEDGERS_HEADING: &str = "## Every v0.01 promise, one at a time";

/// The audit, for a set of promises a caller chose.
///
/// **The caller says what the subject is**, which is the whole of what changed
/// when the version tiers went ([ADR
/// 0105](../../../docs/decisions/0105-there-are-no-versions-until-there-is-a-launch.md)).
/// It used to be *the promises at tier T*, and a tier was a thing the audit
/// looked up for itself — so when the markers were removed the audit silently
/// reconciled the empty set, which passes.
///
/// Now a caller hands over the promises it means. `ROADMAP.md`'s launch list is
/// the set that answers *are we there*; `docs/features.md`'s bullets are the set
/// that answers *does every entry quote something real*. They are different
/// questions and they were one function.
///
/// # Errors
/// As [`reconcile`].
pub fn reconcile_these(
    promises: &[crate::promise::Promise],
    ledger: &str,
    heading: &'static str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Result<Reconciled, Vec<Finding>> {
    the_audit(
        promises,
        promises,
        &crate::ledger::entries_under(ledger, heading),
        heading,
        reading,
    )
}

/// The audit with its two questions asked of two sets.
///
/// `in_the_document` is every promise the features document makes — an entry
/// must quote one of these or it is about nothing. `on_the_launch_list` is what
/// must each **have** an entry. See [`the_audit`] for why they stopped being
/// one set.
///
/// # Errors
/// As [`reconcile`].
pub fn reconcile_the_launch(
    in_the_document: &[crate::promise::Promise],
    on_the_launch_list: &[crate::promise::Promise],
    ledger: &str,
    heading: &'static str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Result<Reconciled, Vec<Finding>> {
    the_audit(
        in_the_document,
        on_the_launch_list,
        &crate::ledger::entries_under(ledger, heading),
        heading,
        reading,
    )
}

/// The same audit, for one release's promises against that release's ledger.
///
/// The tier is a [`crate::tier::Tier`] rather than two strings, because the
/// marker and the heading have to agree: a caller that passed v0.5's marker and
/// v0.01's heading would reconcile ninety-two promises against forty-six entries
/// about other promises and report every one of them unanswered.
///
/// # Errors
///
/// As [`reconcile`], and [`Finding::NoLedgerForThisTier`] where the tier has no
/// ledger to read.
pub fn reconcile_at(
    features: &str,
    ledger: &str,
    tier: crate::tier::Tier,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Result<Reconciled, Vec<Finding>> {
    let Some((_, heading)) = tier.ledger() else {
        return Err(vec![Finding::NoLedgerForThisTier { tier: tier.named() }]);
    };
    let promises = crate::promise::promises_at(features, tier);
    the_audit(
        &promises,
        &promises,
        &crate::ledger::entries_under(ledger, heading),
        heading,
        reading,
    )
}

/// The audit itself, over promises and entries somebody else chose.
fn the_audit(
    promises: &[crate::promise::Promise],
    covering: &[crate::promise::Promise],
    entries: &[crate::Entry],
    heading: &'static str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Result<Reconciled, Vec<Finding>> {
    {
        let promises = promises.to_vec();
        let entries = entries.to_vec();

        if entries.is_empty() && !promises.is_empty() {
            return Err(vec![Finding::NothingToReconcile {
                promises: promises.len(),
                heading,
            }]);
        }

        let mut findings = Vec::new();
        let mut answered = vec![false; promises.len()];
        let mut reconciled = Reconciled {
            promises: promises.len(),
            entries: entries.len(),
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

        // **Coverage is asked of `covering`, not of every promise.** They are the
        // same set for a caller auditing one document against its own ledger. They
        // differ for the one that matters since the tiers went: an entry must quote
        // a promise that exists *anywhere* in the features document, and the
        // promises that must each **have** an entry are the launch list's.
        //
        // Merged, as they were, the audit demanded a ledger entry for all three
        // hundred things alo OS will ever do — two hundred and five findings, none
        // of them a fault.
        for (promise, answered) in promises.iter().zip(&answered) {
            if !answered && covering.iter().any(|want| want.words() == promise.words()) {
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

    if let Some(owed) = entry.owed() {
        if !owed.is_an_answer() {
            findings.push(Finding::AShrugRatherThanAnAnswer {
                promise: entry.promise().to_owned(),
                said: owed.sentence().to_owned(),
            });
        }
        where_the_work_is(entry, owed, reading, findings);
    }
}

/// Whether what a promise waits on is somewhere a reader can go.
///
/// Two rules, and they are deliberately not the same rule. **Every** pointer in
/// what is owed is followed, wherever the entry sits, because a dead one reads
/// exactly like an answer. And a promise with **no evidence at all** must carry
/// at least one: a promise shown in part leaves the reader the code it already
/// has, and a promise shown by nothing leaves them the sentence and nothing else.
fn where_the_work_is(
    entry: &Entry,
    owed: &Owed,
    reading: &dyn Fn(&str) -> Option<String>,
    findings: &mut Vec<Finding>,
) {
    let waits = Waiting::named_in(owed.sentence());
    if waits.is_empty() && entry.names().is_empty() {
        findings.push(Finding::APromiseOwedWithNowhereToGo {
            promise: entry.promise().to_owned(),
        });
    }
    for waiting in waits {
        if let Err(why) = waiting.whether_it_is_there(reading) {
            findings.push(Finding::AWaitNobodyCanFollow {
                promise: entry.promise().to_owned(),
                waiting: waiting.said(),
                why,
            });
        }
    }
}
