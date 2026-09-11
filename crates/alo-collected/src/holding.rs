//! The check: every crate of this workspace that declares words, against the one
//! vocabulary that collects them.
//!
//! Three directions, and only the first is what anybody would write by hand.
//! **Every crate that declares words is collected, or is named apart with a
//! reason** — that is the rule. **Every crate collected still declares words** —
//! that is what stops the list becoming a record of a workspace that has moved
//! on. **Every crate named apart still declares words, and still has an
//! argument** — that is what stops an exception outliving the fact that made it
//! one.
//!
//! Findings are collected rather than returned one at a time. A check that
//! stopped at the first problem would be read as *one problem*, and a change that
//! splits a crate in two usually moves more than one list.

use crate::{
    apart::is_a_reason,
    declaring::{THE_WORKSPACE, whoever_declares_words},
    finding::Finding,
};

/// What the check calls the list of crates whose words are collected.
///
/// Named rather than spelled out at each use, so a finding about the list and a
/// reader looking for the list are looking for the same words.
pub const THE_VOCABULARY: &str = "crates/alo-saying's list of what it collects";

/// What the check calls the list of crates deliberately outside it.
pub const THE_EXCEPTIONS: &str = "crates/alo-saying's list of what stands apart";

/// What the crates of this workspace add up to, once every one of them has been
/// accounted for.
///
/// The counts are the shape of the claim rather than a score: **declaring** is
/// every crate with words in it, and it is exactly **collected** plus **apart**
/// or this is `Err`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    /// How many crates of this workspace declare words.
    declaring: usize,

    /// How many of them are collected into the one vocabulary.
    collected: usize,

    /// How many stand deliberately outside it, with a reason.
    apart: usize,
}

impl Held {
    /// How many crates of this workspace declare words.
    #[must_use]
    pub fn declaring(&self) -> usize {
        self.declaring
    }

    /// How many of them are collected into the one vocabulary.
    #[must_use]
    pub fn collected(&self) -> usize {
        self.collected
    }

    /// How many stand deliberately outside it, with a reason.
    #[must_use]
    pub fn apart(&self) -> usize {
        self.apart
    }
}

/// Every crate of this workspace that declares words, against the vocabulary
/// that collects them and the exceptions written beside it.
///
/// `collected` is the vocabulary's own list of the crates it declares — the same
/// one it declares from, never copied — and `apart` is each crate deliberately
/// outside it with the reason it is. `reading` answers with the text of a file
/// named by its repository-relative path, or [`None`] where there is no such
/// file; nothing here opens anything.
///
/// # Errors
///
/// Every [`Finding`], in the order the workspace, then the two lists, then the
/// crates raise them.
pub fn held(
    collected: &[&str],
    apart: &[(&str, &str)],
    manifest: &str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Result<Held, Vec<Finding>> {
    let declaring = whoever_declares_words(manifest, reading);
    if declaring.is_empty() {
        return Err(vec![Finding::NoWorkspaceToWalk {
            manifest: THE_WORKSPACE,
        }]);
    }

    let mut findings = Vec::new();
    whether_each_list_names_a_crate_once(collected, apart, &mut findings);

    for crate_name in &declaring {
        let is_collected = collected.contains(&crate_name.as_str());
        let is_apart = apart.iter().any(|(named, _)| named == crate_name);
        if is_collected && is_apart {
            findings.push(Finding::ACrateBothCollectedAndApart {
                crate_name: crate_name.clone(),
            });
        }
        if !is_collected && !is_apart {
            findings.push(Finding::ACrateNothingCollects {
                crate_name: crate_name.clone(),
            });
        }
    }

    for named in collected {
        if !declaring.iter().any(|crate_name| crate_name == named) {
            findings.push(Finding::ACollectedCrateThatSaysNothing {
                crate_name: (*named).to_owned(),
            });
        }
    }

    for (named, reason) in apart {
        if !declaring.iter().any(|crate_name| crate_name == named) {
            findings.push(Finding::AnExceptionNobodyNeeds {
                crate_name: (*named).to_owned(),
            });
        }
        if !is_a_reason(reason) {
            findings.push(Finding::AnExceptionWithNoReason {
                crate_name: (*named).to_owned(),
                said: (*reason).to_owned(),
            });
        }
    }

    if findings.is_empty() {
        Ok(Held {
            declaring: declaring.len(),
            collected: collected.len(),
            apart: apart.len(),
        })
    } else {
        Err(findings)
    }
}

/// Whether either list names a crate more than once.
///
/// The counts above are the only thing that says nothing was quietly dropped,
/// and a list that names a crate twice makes both of them add up while one crate
/// goes uncollected.
fn whether_each_list_names_a_crate_once(
    collected: &[&str],
    apart: &[(&str, &str)],
    findings: &mut Vec<Finding>,
) {
    let apart_names: Vec<&str> = apart.iter().map(|(named, _)| *named).collect();
    for (list, names) in [
        (THE_VOCABULARY, collected),
        (THE_EXCEPTIONS, &apart_names[..]),
    ] {
        let mut said: Vec<&str> = Vec::new();
        for name in names {
            let times = names.iter().filter(|other| *other == name).count();
            if times > 1 && !said.contains(name) {
                said.push(name);
                findings.push(Finding::ACrateNamedTwice {
                    crate_name: (*name).to_owned(),
                    list,
                    times,
                });
            }
        }
    }
}
