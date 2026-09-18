//! The check: the one list of crates whose verbs alo OS ships, against the
//! workspace that has them.
//!
//! Two directions, and only the first is what anybody would write by hand.
//! **Every crate that declares verbs is on the list** — that is the rule, and
//! the one a new crate breaks. **Every crate on the list still declares verbs**
//! — that is what stops the list becoming a record of a workspace that has moved
//! on, and a list with a dead name in it is one nobody reads as a check.
//!
//! Findings are collected rather than returned one at a time. A check that
//! stopped at the first problem would be read as *one problem*, and a change
//! that splits a crate in two moves more than one line.

use alo_by_hand::{THE_WORKSPACE, whoever_declares_verbs};

use crate::finding::Finding;

/// What the crates of this workspace add up to, once every one of them has been
/// accounted for.
///
/// The counts are the shape of the claim rather than a score: **declaring** is
/// every crate of this workspace with verbs in it, and it is exactly **listed**
/// or this is `Err`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    /// How many crates of this workspace declare verbs.
    declaring: usize,

    /// How many the one list names.
    listed: usize,
}

impl Held {
    /// How many crates of this workspace declare verbs.
    #[must_use]
    pub fn declaring(&self) -> usize {
        self.declaring
    }

    /// How many the one list names.
    #[must_use]
    pub fn listed(&self) -> usize {
        self.listed
    }
}

/// The one list of crates whose verbs alo OS ships, against the workspace that
/// has them.
///
/// `listed` is that list — [`crate::WHO_DECLARES_THEM`], never a copy of it —
/// and `reading` answers with the text of a file named by its
/// repository-relative path, or [`None`] where there is no such file. Nothing
/// here opens anything, which is what lets the finding this crate exists for be
/// shown happening against a fixture rather than only after somebody has written
/// the crate that would cause it.
///
/// # Errors
///
/// Every [`Finding`], in the order the workspace, then the list, then the crates
/// raise them.
pub fn held(
    listed: &[&str],
    manifest: &str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Result<Held, Vec<Finding>> {
    let declaring = whoever_declares_verbs(manifest, reading);
    if declaring.is_empty() {
        return Err(vec![Finding::NoWorkspaceToWalk {
            manifest: THE_WORKSPACE,
        }]);
    }

    let mut findings = Vec::new();
    whether_the_list_names_a_crate_once(listed, &mut findings);

    for crate_name in &declaring {
        if !listed.contains(&crate_name.as_str()) {
            findings.push(Finding::ACrateNothingHandsIn {
                crate_name: crate_name.clone(),
            });
        }
    }

    for named in listed {
        if !declaring.iter().any(|crate_name| crate_name == named) {
            findings.push(Finding::AListedCrateWithNoVerbs {
                crate_name: (*named).to_owned(),
            });
        }
    }

    if findings.is_empty() {
        Ok(Held {
            declaring: declaring.len(),
            listed: listed.len(),
        })
    } else {
        Err(findings)
    }
}

/// Whether the list names a crate more than once.
///
/// The counts above are the only thing that says nothing was quietly dropped,
/// and a list that names a crate twice makes them add up while a crate goes
/// unchecked.
fn whether_the_list_names_a_crate_once(listed: &[&str], findings: &mut Vec<Finding>) {
    let mut said: Vec<&str> = Vec::new();
    for name in listed {
        let times = listed.iter().filter(|other| *other == name).count();
        if times > 1 && !said.contains(name) {
            said.push(name);
            findings.push(Finding::ACrateNamedTwice {
                crate_name: (*name).to_owned(),
                times,
            });
        }
    }
}
