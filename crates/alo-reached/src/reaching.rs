//! Which `pub fn` of the workspace nothing reaches.
//!
//! One tokenising pass per file, a count of identifiers built once, and then a
//! **fixed point**: drop the mentions made inside functions that nothing
//! reaches, recount, and repeat until the answer stops moving.
//!
//! Hole one on `docs/autonomy/what-nothing-calls.md` is why the fixed point is
//! not optional: `alo-dock`'s `where_it_sits` looked reached because a one-line
//! wrapper named it, and **nothing called the wrapper**. Reachability is not one
//! hop, and the clue was a result that looked wrong — two functions side by side
//! in one file doing the same job, one with a caller and one without.
//!
//! # The unit that dies is a function body, and the first version used the file
//!
//! The first version of this dropped a whole **file** once every `pub fn` in it
//! was unreached, and that falsely accused. `after_pub` deliberately ignores
//! `pub(crate)` and private functions, so a file holding one dead `pub fn` and
//! twenty live private ones was dropped entire, and every call those twenty made
//! stopped counting — over twelve rounds, cascading. Sampling five findings
//! caught three false ones, among them `Bounds::reaching_nothing_but`, which is
//! called in product code at `crates/alo-agentd/src/bounding.rs:214`.
//!
//! **What makes the function-sized unit sound is a guarantee `rustc` already
//! gives**: `dead_code` is warn-by-default and the gate runs `clippy` with
//! `-D warnings`, so a private or `pub(crate)` function nothing reaches **fails
//! CI today**. Only a `pub fn` can be dead and unnoticed, and those are exactly
//! the ones named here. So every line outside a `pub fn` body always counts, and
//! a `pub fn` body stops counting only once the function is known unreached.
//!
//! Where two `pub fn`s share a name — `new`, in forty impls — a body stops
//! counting only when **every** declaration of that name is unreached. That errs
//! towards counting a mention, which errs towards accusing nobody.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::the_code::a_crates_files;

/// One `pub fn`, and where it is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct APubFn {
    /// The crate's directory, as the workspace manifest names it.
    pub in_crate: String,
    /// The file it is declared in, relative to the repository.
    pub at: PathBuf,
    /// Its line, so a reader can open it.
    pub line: usize,
    /// Its name.
    pub named: String,
}

/// Every `pub fn` the workspace declares that nothing reaches: no call outside
/// its own declaration, and no mention in any test.
///
/// `root` is the repository. The answer is sorted, so two runs of the same tree
/// give the same list in the same order — a snapshot that reordered itself
/// would be a diff nobody could read.
///
/// # Panics
///
/// Never: every fallible read is treated as an absent file, because a crate
/// whose source cannot be read is not evidence about a function.
#[must_use]
pub fn what_nothing_reaches(root: &Path) -> Vec<APubFn> {
    let manifest =
        std::fs::read_to_string(root.join(crate::the_code::THE_WORKSPACE)).unwrap_or_default();
    let members: Vec<String> = alo_by_hand::declaring::members_of(&manifest)
        .into_iter()
        .filter(|named| named.starts_with("crates/"))
        .collect();

    // Every declaration, every countable segment of product text, and the
    // test text. A segment is either a `pub fn`'s body, which can stop
    // counting, or anything else, which never does — see this module's header.
    let mut declared: Vec<APubFn> = Vec::new();
    let mut segments: Vec<(Option<String>, String)> = Vec::new();
    let mut tests = String::new();
    for member in &members {
        let files = a_crates_files(&root.join(member));
        tests.push('\n');
        tests.push_str(&files.tests);
        for (at, code) in files.product {
            for (line, named) in pub_fns_in(&code) {
                declared.push(APubFn {
                    in_crate: member.clone(),
                    at: at.strip_prefix(root).unwrap_or(&at).to_path_buf(),
                    line,
                    named,
                });
            }
            segments.extend(segments_of(&code));
        }
    }
    declared.sort();

    let in_tests: HashSet<&str> = identifiers(&tests).collect();
    let mut declarations: HashMap<&str, usize> = HashMap::new();
    for it in &declared {
        *declarations.entry(it.named.as_str()).or_default() += 1;
    }

    let mut unreached: Vec<APubFn> = Vec::new();
    let mut dead: HashSet<String> = HashSet::new();

    // The fixed point. Twelve rounds is far more than this workspace needs —
    // the hand sweep settled in three — and a bound rather than `loop` so a
    // mistake here cannot hang a gate.
    for _ in 0..12 {
        let mut counted: HashMap<&str, usize> = HashMap::new();
        for (owner, text) in &segments {
            if owner.as_deref().is_some_and(|named| dead.contains(named)) {
                continue;
            }
            for word in identifiers(text) {
                *counted.entry(word).or_default() += 1;
            }
        }

        // **The guard the page requires**: if the most-mentioned declaration is
        // mentioned nowhere, the run matched nothing and must not report.
        let most = declared
            .iter()
            .map(|it| counted.get(it.named.as_str()).copied().unwrap_or(0))
            .max()
            .unwrap_or(0);
        if most == 0 {
            return Vec::new();
        }

        let found: Vec<APubFn> = declared
            .iter()
            .filter(|it| {
                if in_tests.contains(it.named.as_str()) {
                    return false;
                }
                let mentions = counted.get(it.named.as_str()).copied().unwrap_or(0);
                // Its own declarations are mentions of it that are not calls.
                let declares = declarations.get(it.named.as_str()).copied().unwrap_or(0);
                mentions <= declares
            })
            .cloned()
            .collect();

        // A name whose every declaration is unreached stops counting as a
        // caller, which is hole one.
        let mut sightings: HashMap<&str, usize> = HashMap::new();
        for it in &found {
            *sightings.entry(it.named.as_str()).or_default() += 1;
        }
        let next: HashSet<String> = sightings
            .iter()
            .filter(|(named, seen)| declarations.get(**named).copied().unwrap_or(0) == **seen)
            .map(|(named, _)| (*named).to_owned())
            .collect();

        unreached = found;
        if next == dead {
            break;
        }
        dead = next;
    }
    unreached
}

/// A file's text, split into each `pub fn`'s body and everything else.
///
/// The `None` segments always count as callers; a `Some(name)` segment counts
/// only while that name is still reachable. This module's header says why that
/// is the right unit and what the file-sized one got wrong.
fn segments_of(code: &str) -> Vec<(Option<String>, String)> {
    let mut found = Vec::new();
    let mut outside = String::new();
    let mut from = 0_usize;
    while let Some((named, past)) = next_pub_fn(code, from) {
        // Where the body opens — if the item **has** one. A `pub fn` inside an
        // `extern` block ends at a semicolon and owns no text.
        let rest = code.get(past..).unwrap_or_default();
        let ends = rest.find(';');
        let opens = rest.find('{');
        let open = match (ends, opens) {
            (Some(ends), Some(opens)) if ends < opens => None,
            (_, Some(opens)) => Some(past.saturating_add(opens)),
            (_, None) => None,
        };
        let Some(open) = open else {
            outside.push_str(code.get(from..past).unwrap_or_default());
            from = past;
            continue;
        };
        let end = crate::the_code::closing_brace(code, open);
        outside.push_str(code.get(from..open).unwrap_or_default());
        found.push((
            Some(named),
            code.get(open..end).unwrap_or_default().to_owned(),
        ));
        from = end;
    }
    outside.push_str(code.get(from..).unwrap_or_default());
    found.push((None, outside));
    found
}

/// The next `pub fn` declaration at or after `from`: its name, and the offset
/// just past that name.
fn next_pub_fn(code: &str, from: usize) -> Option<(String, usize)> {
    let mut at = from;
    for line in code.get(from..)?.split_inclusive('\n') {
        let began = at;
        at = at.saturating_add(line.len());
        if let Some((named, past)) = the_name_declared_on(line) {
            return Some((named, began.saturating_add(past)));
        }
    }
    None
}

/// The `pub fn` name a line declares and the offset just past it, if it
/// declares one.
///
/// The offset comes from what the line has **left** at each step rather than
/// from searching for the name, because a name can occur earlier in the line —
/// `pub fn fn_of(fn_of: u8)` is legal — and a search would find the wrong one.
fn the_name_declared_on(line: &str) -> Option<(String, usize)> {
    let rest = after_pub(line.trim_start())?;
    let rest = rest.trim_start();
    let rest = ["const ", "async ", "unsafe ", "extern "]
        .iter()
        .fold(rest, |rest, word| {
            rest.strip_prefix(word).unwrap_or(rest).trim_start()
        });
    let rest = rest.strip_prefix("fn ")?;
    let named: String = rest
        .chars()
        .take_while(|it| it.is_alphanumeric() || *it == '_')
        .collect();
    if named.is_empty() {
        return None;
    }
    // Every step above produced a suffix of `line`, so where `rest` begins is
    // the line's length less what remains of it.
    let begins = line.len().saturating_sub(rest.len());
    let past = begins.saturating_add(named.len());
    Some((named, past))
}

/// Every `pub fn` a file declares, with its line.
///
/// Matched on the line rather than parsed: this workspace writes one `pub fn`
/// per line and `cargo fmt` keeps it that way, and a parser here would be a
/// second Rust front end to maintain for one question.
fn pub_fns_in(code: &str) -> Vec<(usize, String)> {
    code.lines()
        .enumerate()
        .filter_map(|(at, line)| {
            the_name_declared_on(line).map(|(named, _)| (at.saturating_add(1), named))
        })
        .collect()
}

/// What follows `pub` or `pub(…)` on a line, or nothing where the line is not
/// public.
///
/// **`pub(crate)` and `pub(super)` are not public surface** and are left out: a
/// function visible only inside its own crate is reached or not by the ordinary
/// rules of that crate, and `rustc`'s own dead-code warning already catches it.
/// What this gate is for is the surface `rustc` cannot warn about.
fn after_pub(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("pub")?;
    if rest.starts_with('(') {
        return None;
    }
    Some(rest)
}

/// Every identifier in some text.
fn identifiers(text: &str) -> impl Iterator<Item = &str> {
    text.split(|it: char| !(it.is_alphanumeric() || it == '_'))
        .filter(|word| !word.is_empty() && !word.starts_with(|it: char| it.is_numeric()))
}

#[cfg(test)]
#[path = "reaching_tests.rs"]
mod tests;
