//! **What the model is offered, declared**: one line for what every turn is
//! told, one for each verb it is offered, and one for each request of the set
//! with the verbs its roads ask for — each line a fingerprint of the exact
//! text and schema the model is given.
//!
//! The declaration is committed as `what-the-model-is-offered.txt`, and a
//! test holds that file to what the code offers ([`AS_COMMITTED`]). So a pull
//! request that changes what the model chooses **changes a line of it**, in
//! its own diff, where a reviewer sees which; and the every-try check reads
//! the base's file beside the code's to know which requests are the change's
//! own (`crate::its_own`). Nothing is guessed from which files a change
//! touched.
//!
//! # What a line answers to
//!
//! - **Every turn**: the instructions and the words after the verbs
//!   (`alo_instructing::the_products_words` without the verbs), what the set's
//!   machine tells a turn it may reach, the envelope a provider holds an
//!   answer to before any verb is in it, and the places the machine installs
//!   from.
//! - **A verb**: the text the model is told about it
//!   (`alo_instructing::verb_as_told::as_told`), what a call of it is judged
//!   by (what each argument takes), the sentence a done change is told in, and
//!   the branch of the envelope that holds a call of it — not why it needs no
//!   grant, which the model is never shown.
//! - **A request**: what the model is shown and how its road is judged
//!   (`ARequest::as_shown_and_judged`) — never the raw text of its
//!   declaration, so a field every request leaves at its default changes no
//!   line (the owner, 2026-11-20).
//!
//! # What no line answers to
//!
//! The code that writes the machine's scripted answers
//! (`crate::the_machine_answers`), the agent's loop and the protocol's reader
//! — a change to those can change what the model chooses next and owns no
//! request here, and only a reader of the change can see it.
//!
//! A fingerprint is the first sixteen hexadecimal digits of the text's
//! SHA-256: short enough to read in a diff, and a name for the text rather
//! than a copy of it, so the file is not a second verb list (`alo_capability::
//! Verb` says why there must not be one).

use std::collections::{BTreeMap, BTreeSet};

use alo_capability::{Verb, Verbs};
use alo_instructing::verb_as_told::as_told;
use alo_instructing::{Instructions, WhatItMayReach, working_in_an_application};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::a_road::{ARequest, Step};
use crate::the_set::{NotTheSet, TheSet};

/// The committed declaration's file name, in this crate's directory.
pub const THE_FILE: &str = "what-the-model-is-offered.txt";

/// The declaration as committed, which a test holds to what the code offers.
pub const AS_COMMITTED: &str = include_str!("../what-the-model-is-offered.txt");

/// The head of the file, saying what it is and how it is written again.
const THE_HEAD: &str = "\
# What alo's model is offered, one fingerprint a line (crates/alo-every-try,
# `the_declaration`). A change to what the model chooses changes a line here,
# and the every-try check tries the requests that line belongs to: the
# change's own, which must each take the road at least 9 of 10 tries.
#
# Written by `cargo run -p alo-every-try -- declare`. A test fails while this
# file is not what the code offers.
";

/// **Which way a declaration's lines were fingerprinted.** Two declarations
/// are compared line by line only in one scheme; a base in another is
/// compared with the head fingerprinted the base's way
/// (`crate::its_own::its_own_against`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    /// Until 2026-11-20: a request by the raw text of its definition
    /// (`crate::the_first_fingerprints`). Its files name no scheme.
    First,
    /// Since 2026-11-21: as [`Scheme::WhatIsShown`], with each answer
    /// fingerprinted as the model is sent it rather than as Rust prints it.
    AsSent,
    /// A request by what the model is shown and how its road is judged, a verb
    /// by what it is told and what a call of it is judged by.
    WhatIsShown,
}

impl Scheme {
    /// The number the file names it by.
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::First => 1,
            Self::WhatIsShown => 2,
            Self::AsSent => 3,
        }
    }
}

/// A declaration that could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the declaration does not read, at line {line}: {why}")]
pub struct NotDeclared {
    /// The line, counted from one.
    line: usize,
    /// What is wrong with it.
    why: String,
}

/// One request, declared: its name, the fingerprint of its definition, and
/// every verb its roads ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ARequestDeclared {
    /// What the request is called.
    named: String,
    /// The fingerprint of what the model is shown and how its road is judged.
    line: String,
    /// Every verb its roads and the reads aside of them ask for, once each.
    asks_for: Vec<String>,
}

impl ARequestDeclared {
    /// What the request is called.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// The fingerprint of what the model is shown and how its road is judged.
    #[must_use]
    pub fn line(&self) -> &str {
        &self.line
    }

    /// Every verb its roads ask for.
    #[must_use]
    pub fn asks_for(&self) -> &[String] {
        &self.asks_for
    }
}

/// What the model is offered, one fingerprint for each thing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    /// Which way its lines were fingerprinted.
    scheme: Scheme,
    /// What every turn is told.
    every_turn: String,
    /// Each verb, by name.
    verbs: BTreeMap<String, String>,
    /// Each request, in the set's order.
    requests: Vec<ARequestDeclared>,
}

/// How one scheme fingerprints a verb.
type AVerbsLine = fn(&Verb, &Value) -> String;

/// How one scheme fingerprints a request.
type ARequestsLine = fn(&ARequest) -> String;

/// The fingerprint of `text`.
pub(crate) fn fingerprint(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// **The line for what every turn is told**: `words` — what the model is told
/// apart from the verbs — the machine it is told it may reach, the envelope
/// before any verb is in it, and `places`.
#[must_use]
pub fn the_every_turn_line(words: &str, may_reach: &WhatItMayReach, places: &[String]) -> String {
    let envelope = alo_models::in_the_envelope::the_envelope_a_provider_accepts();
    fingerprint(&format!(
        "{words}\n--- may reach\n{}\n--- the envelope\n{envelope}\n--- the places\n{}",
        may_reach.told(),
        places.join("\n")
    ))
}

/// The branch of `envelope` that holds a call of the verb called `name`:
/// the object whose `verb` property is that one name.
pub(crate) fn the_branch_for<'a>(name: &str, envelope: &'a Value) -> Option<&'a Value> {
    match envelope {
        Value::Object(map) => {
            let named = envelope
                .pointer("/properties/verb/enum")
                .and_then(Value::as_array)
                .is_some_and(|names| {
                    names.len() == 1 && names.first().and_then(Value::as_str) == Some(name)
                });
            if named {
                return Some(envelope);
            }
            map.values().find_map(|inside| the_branch_for(name, inside))
        }
        Value::Array(each) => each.iter().find_map(|inside| the_branch_for(name, inside)),
        _ => None,
    }
}

/// **The line for one verb**: what the model is told about it, its whole
/// declaration, and the branch of `envelope` that holds a call of it.
#[must_use]
pub fn a_verbs_line(verb: &Verb, envelope: &Value) -> String {
    let branch = the_branch_for(verb.name(), envelope)
        .map(Value::to_string)
        .unwrap_or_default();
    let mut judged = String::new();
    for arg in verb.args() {
        judged.push_str(&format!("{} takes {:?}\n", arg.name(), arg.takes()));
    }
    judged.push_str(&format!("said as {:?}\n", verb.sentence()));
    if let Some(changes) = verb.what_it_changes() {
        judged.push_str(&format!("changes {changes:?}\n"));
    }
    if let Some(leaves) = verb.what_it_leaves() {
        judged.push_str(&format!("leaves {leaves:?}\n"));
    }
    fingerprint(&format!(
        "{}\n--- judged\n{judged}--- held to\n{branch}",
        as_told(verb)
    ))
}

/// One request's line in the second scheme: each answer as Rust prints it.
fn a_requests_line_as_printed(request: &ARequest) -> String {
    fingerprint(&request.as_shown_and_judged_with(crate::a_road::TheAnswerWritten::AsPrinted))
}

/// **The line for one request**: what the model is shown and how its road is
/// judged (`ARequest::as_shown_and_judged`), so a field every request leaves
/// at its default changes no line.
#[must_use]
pub fn a_requests_line(request: &ARequest) -> String {
    fingerprint(&request.as_shown_and_judged())
}

/// Every verb `request`'s roads ask for, the reads aside included, once each
/// in the order they first appear.
fn what_it_asks_for(request: &ARequest) -> Vec<String> {
    let mut seen = BTreeSet::new();
    std::iter::once(request.road())
        .chain(request.other_roads().iter().map(Vec::as_slice))
        .flatten()
        .chain(request.aside())
        .map(Step::verb)
        .filter(|verb| seen.insert(*verb))
        .map(str::to_owned)
        .collect()
}

impl Declared {
    /// What `set` and `verbs` offer the model.
    ///
    /// # Errors
    /// [`NotTheSet`], when two requests share a name and could not be told
    /// apart.
    pub fn of(set: &TheSet, verbs: &Verbs) -> Result<Self, NotTheSet> {
        Self::by_the_scheme(set, verbs, Scheme::AsSent)
    }

    /// What `set` and `verbs` offer the model, fingerprinted in `scheme`.
    ///
    /// # Errors
    /// As [`Declared::of`].
    pub fn by_the_scheme(set: &TheSet, verbs: &Verbs, scheme: Scheme) -> Result<Self, NotTheSet> {
        let (verb_line, request_line): (AVerbsLine, ARequestsLine) = match scheme {
            Scheme::First => (
                crate::the_first_fingerprints::a_verbs_line,
                crate::the_first_fingerprints::a_requests_line,
            ),
            Scheme::WhatIsShown => (a_verbs_line, a_requests_line_as_printed),
            Scheme::AsSent => (a_verbs_line, a_requests_line),
        };
        let envelope =
            alo_turn::the_shape_a_provider_is_held_to::the_envelope_naming_the_verbs_and_places(
                verbs,
                set.places(),
            );
        let words = format!(
            "{}{}",
            Instructions::SHOWN_TO_A_TURN.text(),
            working_in_an_application(verbs)
        );
        let offered: BTreeMap<String, String> = verbs
            .all()
            .map(|verb| (verb.name().to_owned(), verb_line(verb, &envelope)))
            .collect();
        let mut named = BTreeSet::new();
        let mut requests = Vec::with_capacity(set.requests().len());
        for request in set.requests() {
            if !named.insert(request.named()) {
                return Err(NotTheSet::because(format!(
                    "two requests are called `{}`",
                    request.named()
                )));
            }
            let asks_for = what_it_asks_for(request);
            if let Some(nowhere) = asks_for.iter().find(|verb| !offered.contains_key(*verb)) {
                return Err(NotTheSet::because(format!(
                    "`{}` asks for `{nowhere}`, which the model is not offered",
                    request.named()
                )));
            }
            requests.push(ARequestDeclared {
                named: request.named().to_owned(),
                line: request_line(request),
                asks_for,
            });
        }
        Ok(Self {
            scheme,
            every_turn: the_every_turn_line(&words, set.may_reach(), set.places()),
            verbs: offered,
            requests,
        })
    }

    /// A declaration written out by hand: what every turn is told, each verb
    /// and its line, and each request, its line and the verbs it asks for.
    #[must_use]
    pub fn by_hand(
        every_turn: &str,
        verbs: &[(&str, &str)],
        requests: &[(&str, &str, &[&str])],
    ) -> Self {
        Self {
            scheme: Scheme::AsSent,
            every_turn: every_turn.to_owned(),
            verbs: verbs
                .iter()
                .map(|(name, line)| ((*name).to_owned(), (*line).to_owned()))
                .collect(),
            requests: requests
                .iter()
                .map(|(named, line, asks_for)| ARequestDeclared {
                    named: (*named).to_owned(),
                    line: (*line).to_owned(),
                    asks_for: asks_for.iter().map(|verb| (*verb).to_owned()).collect(),
                })
                .collect(),
        }
    }

    /// The same, with the line of `verb` changed, as a change to its words
    /// would change it.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_the_verb_changed(mut self, verb: &str) -> Self {
        if let Some(line) = self.verbs.get_mut(verb) {
            line.push_str("-changed");
        }
        self
    }

    /// Which way its lines were fingerprinted.
    #[must_use]
    pub const fn scheme(&self) -> Scheme {
        self.scheme
    }

    /// The line for what every turn is told.
    #[must_use]
    pub fn every_turn(&self) -> &str {
        &self.every_turn
    }

    /// The name of every verb offered, in order.
    pub fn verb_names(&self) -> impl Iterator<Item = &str> {
        self.verbs.keys().map(String::as_str)
    }

    /// The line for the verb called `name`, if it is offered.
    #[must_use]
    pub fn verb(&self, name: &str) -> Option<&str> {
        self.verbs.get(name).map(String::as_str)
    }

    /// Every request, in the set's order.
    #[must_use]
    pub fn requests(&self) -> &[ARequestDeclared] {
        &self.requests
    }

    /// The declaration as the file holds it.
    #[must_use]
    pub fn written(&self) -> String {
        let mut text = THE_HEAD.to_owned();
        if self.scheme != Scheme::First {
            text.push_str(&format!("\nfingerprints\t{}", self.scheme.number()));
        }
        text.push_str(&format!("\nevery-turn\t{}\n", self.every_turn));
        for (name, line) in &self.verbs {
            text.push_str(&format!("verb\t{name}\t{line}\n"));
        }
        for request in &self.requests {
            text.push_str(&format!(
                "request\t{}\t{}\t{}\n",
                request.named,
                request.line,
                request.asks_for.join(",")
            ));
        }
        text
    }

    /// The declaration `text` holds.
    ///
    /// # Errors
    /// [`NotDeclared`], for a line that is not one of the three kinds, a verb
    /// or request declared twice, a request asking for a verb not declared,
    /// or no line for what every turn is told.
    pub fn read(text: &str) -> Result<Self, NotDeclared> {
        let refused = |line: usize, why: &str| NotDeclared {
            line,
            why: why.to_owned(),
        };
        let mut scheme = None;
        let mut every_turn = None;
        let mut verbs = BTreeMap::new();
        let mut requests: Vec<ARequestDeclared> = Vec::new();
        for (at, line) in text.lines().enumerate() {
            let number = at + 1;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            match line.split('\t').collect::<Vec<_>>().as_slice() {
                ["fingerprints", which] => {
                    let named = match *which {
                        "2" => Scheme::WhatIsShown,
                        "3" => Scheme::AsSent,
                        _ => {
                            return Err(refused(
                                number,
                                "a scheme of fingerprints this check does not know",
                            ));
                        }
                    };
                    if scheme.replace(named).is_some() {
                        return Err(refused(number, "the scheme is declared twice"));
                    }
                }
                ["every-turn", fingerprint] => {
                    if every_turn.replace((*fingerprint).to_owned()).is_some() {
                        return Err(refused(number, "what every turn is told is declared twice"));
                    }
                }
                ["verb", name, fingerprint] => {
                    if verbs
                        .insert((*name).to_owned(), (*fingerprint).to_owned())
                        .is_some()
                    {
                        return Err(refused(number, "a verb is declared twice"));
                    }
                }
                ["request", named, fingerprint, asks_for] => {
                    if requests.iter().any(|one| one.named == *named) {
                        return Err(refused(number, "a request is declared twice"));
                    }
                    requests.push(ARequestDeclared {
                        named: (*named).to_owned(),
                        line: (*fingerprint).to_owned(),
                        asks_for: asks_for
                            .split(',')
                            .filter(|verb| !verb.is_empty())
                            .map(str::to_owned)
                            .collect(),
                    });
                }
                _ => {
                    return Err(refused(
                        number,
                        "a line is `every-turn`, `verb` or `request` and its fields, split by tabs",
                    ));
                }
            }
        }
        let every_turn =
            every_turn.ok_or_else(|| refused(0, "what every turn is told is not declared"))?;
        if let Some(request) = requests
            .iter()
            .find(|one| one.asks_for.iter().any(|verb| !verbs.contains_key(verb)))
        {
            return Err(refused(
                0,
                &format!("`{}` asks for a verb that is not declared", request.named),
            ));
        }
        Ok(Self {
            scheme: scheme.unwrap_or(Scheme::First),
            every_turn,
            verbs,
            requests,
        })
    }
}

#[cfg(test)]
#[path = "the_declaration_tests.rs"]
mod tests;
