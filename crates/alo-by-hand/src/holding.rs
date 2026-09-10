//! The check: every verb this machine ships, against what `docs/by-hand.md` says
//! a person does instead.
//!
//! Three directions, and only the first is what anybody would write by hand.
//! **Every verb must be answered** — that is the rule. **Every answer must be
//! about a verb the machine still declares** — that is what stops the document
//! becoming a record of a verb list that has moved on. **Every crate that
//! declares verbs must have been handed in** — that is what stops the whole
//! check being about a subset nobody chose.
//!
//! Findings are collected rather than returned one at a time. A check that
//! stopped at the first problem would be read as *one problem*, and a change that
//! adds a verb list usually adds several verbs at once.

use alo_capability::Verbs;

use crate::{
    declaring::{THE_WORKSPACE, whoever_declares_verbs},
    document::{THE_ANSWERS, entries_in},
    entry::Entry,
    finding::Finding,
    promised::{Promised, promises_in, releases_among},
};

/// What the verbs add up to, once every one of them has been answered.
///
/// The counts are the honest shape of the promise rather than a score. **By
/// hand** is a verb whose plain way is promised and nothing is owed on it;
/// **partly** is one whose plain way is promised and which still owes something;
/// **owed** is one the definition does not promise a plain way for at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    /// How many verbs alo OS ships.
    verbs: usize,

    /// How many name a plain way with nothing owed on it.
    by_hand: usize,

    /// How many name a plain way and still owe something.
    partly: usize,

    /// How many have no plain way promised anywhere, named as owed.
    owed: usize,
}

impl Held {
    /// How many verbs alo OS ships.
    #[must_use]
    pub fn verbs(&self) -> usize {
        self.verbs
    }

    /// How many name a plain way with nothing owed on it.
    #[must_use]
    pub fn by_hand(&self) -> usize {
        self.by_hand
    }

    /// How many name a plain way and still owe something.
    #[must_use]
    pub fn partly(&self) -> usize {
        self.partly
    }

    /// How many have no plain way promised anywhere, named as owed.
    #[must_use]
    pub fn owed(&self) -> usize {
        self.owed
    }
}

/// Every verb on a list, against a by-hand document and the definition.
///
/// `verbs` is the registry a daemon would be handed — the verbs themselves,
/// never their names copied out — and `from` names the crates they were declared
/// by, which is what lets a crate declaring verbs behind this check's back be
/// found. `reading` answers with the text of a file named by its
/// repository-relative path, or [`None`] where there is no such file; nothing
/// here opens anything.
///
/// # Errors
///
/// Every [`Finding`], in the order the workspace, then the document, then the
/// verb list raise them.
pub fn held(
    verbs: &Verbs,
    from: &[&str],
    document: &str,
    features: &str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Result<Held, Vec<Finding>> {
    let mut findings = Vec::new();
    whether_every_list_was_handed_in(from, reading, &mut findings);

    let entries = entries_in(document);
    if entries.is_empty() && !verbs.is_empty() {
        findings.push(Finding::NothingToCheck {
            verbs: verbs.len(),
            heading: THE_ANSWERS,
        });
        return Err(findings);
    }

    let promises = promises_in(features);
    let releases: Vec<String> = releases_among(&promises)
        .iter()
        .map(|release| release.as_written().to_owned())
        .collect();

    let shipped: Vec<&str> = verbs.all().map(alo_capability::Verb::name).collect();
    let mut answered = vec![false; shipped.len()];
    let mut held = Held {
        verbs: shipped.len(),
        by_hand: 0,
        partly: 0,
        owed: 0,
    };

    for entry in &entries {
        match shipped.iter().position(|name| *name == entry.verb()) {
            Some(at) => match answered.get_mut(at) {
                Some(already) if *already => findings.push(Finding::AVerbAnsweredTwice {
                    verb: entry.verb().to_owned(),
                }),
                Some(already) => *already = true,
                None => {}
            },
            None => {
                findings.push(Finding::AnEntryAboutNoVerb {
                    verb: entry.verb().to_owned(),
                });
                continue;
            }
        }
        whether_it_answers(entry, &promises, &releases, &mut findings, &mut held);
    }

    for (verb, answered) in shipped.iter().zip(&answered) {
        if !answered {
            findings.push(Finding::AVerbNobodyAnswered {
                verb: (*verb).to_owned(),
            });
        }
    }

    if findings.is_empty() {
        Ok(held)
    } else {
        Err(findings)
    }
}

/// Whether every crate of this workspace that declares verbs was handed to the
/// check.
fn whether_every_list_was_handed_in(
    from: &[&str],
    reading: &dyn Fn(&str) -> Option<String>,
    findings: &mut Vec<Finding>,
) {
    let Some(manifest) = reading(THE_WORKSPACE) else {
        findings.push(Finding::NoWorkspaceToWalk {
            manifest: THE_WORKSPACE,
        });
        return;
    };
    let declaring = whoever_declares_verbs(&manifest, reading);
    if declaring.is_empty() {
        findings.push(Finding::NoWorkspaceToWalk {
            manifest: THE_WORKSPACE,
        });
        return;
    }
    for crate_name in declaring {
        if !from.contains(&crate_name.as_str()) {
            findings.push(Finding::AVerbListNobodyHandedIn { crate_name });
        }
    }
}

/// Whether an entry answers about its verb, and what its answer costs the count.
fn whether_it_answers(
    entry: &Entry,
    promises: &[Promised],
    releases: &[String],
    findings: &mut Vec<Finding>,
    held: &mut Held,
) {
    if !entry.says_something() {
        findings.push(Finding::AVerbWithNeither {
            verb: entry.verb().to_owned(),
        });
        return;
    }

    if let Some(plain) = entry.by_hand() {
        if plain.naming().is_empty() {
            findings.push(Finding::AnAnswerNamingNoSurface {
                verb: entry.verb().to_owned(),
            });
        }
        for quoted in plain.naming() {
            match promises
                .iter()
                .filter(|promise| promise.carries(quoted))
                .count()
            {
                1 => {}
                0 => findings.push(Finding::AWayNothingPromises {
                    verb: entry.verb().to_owned(),
                    quoted: quoted.clone(),
                }),
                several => findings.push(Finding::AWayPromisedMoreThanOnce {
                    verb: entry.verb().to_owned(),
                    quoted: quoted.clone(),
                    promises: several,
                }),
            }
        }
        if !plain.is_an_answer() {
            findings.push(Finding::AShrugRatherThanAnAnswer {
                verb: entry.verb().to_owned(),
                said: plain.sentence().to_owned(),
            });
        }
    }

    if let Some(owed) = entry.owed() {
        match owed.release() {
            None => findings.push(Finding::AnOwedAnswerWithNoRelease {
                verb: entry.verb().to_owned(),
            }),
            Some(named) if !releases.iter().any(|release| release == named) => {
                findings.push(Finding::AReleaseNobodyShips {
                    verb: entry.verb().to_owned(),
                    named: named.to_owned(),
                });
            }
            Some(_) => {}
        }
        if !owed.is_an_answer() {
            findings.push(Finding::AShrugRatherThanAnAnswer {
                verb: entry.verb().to_owned(),
                said: owed.sentence().to_owned(),
            });
        }
    }

    match (entry.by_hand().is_some(), entry.owed().is_some()) {
        (true, false) => held.by_hand += 1,
        (true, true) => held.partly += 1,
        (false, true) => held.owed += 1,
        (false, false) => {}
    }
}
