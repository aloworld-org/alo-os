//! **A turn cannot arrive at this road** — not merely that no verb is on the
//! broker's list, but that nothing an agent's turn runs inside can take it.
//!
//! `tests/nothing_here_is_a_verb.rs` holds the first sentence: no name on
//! `alo_broker::SystemVerb`'s list begins `undo.`, and the list has not grown.
//! That is the right test for a closed enum and it is not the whole promise. A
//! verb is one way to arrive somewhere; linking the code and calling it is
//! another, and a crate that quietly became a dependency of the daemon or of the
//! turn would put the one privileged remover on this machine inside the process
//! an agent is running in.
//!
//! Task 14 of `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` asked for
//! this by name, when the owner narrowed the road to *a person's act in
//! Settings, and never a broker verb*. It is stronger than the verb list and
//! weaker than a kernel boundary, and what it actually rules out is the change
//! nobody would notice: an `alo-letting-go` added to somebody's `Cargo.toml` for
//! a type, months from now, in a crate a turn links.
//!
//! # How it is held, and the one true thing that is not
//!
//! The workspace's own manifests, read as text, walked from the two crates an
//! agent's turn really is: `alo-agentd`, the daemon it speaks to, and
//! `alo-turn`, the turn itself. Everything either of them **ships** — never a
//! dev-dependency, which is a test's and is not in anybody's process — is the
//! set a turn can reach.
//!
//! **This crate is in that set, and the honest test says so.** `alo-agentd`
//! ships `alo-saying`, which is the machine's one vocabulary and therefore
//! ships every crate that declares a word — this one included, for the nine
//! sentences about `undo.toml` and an asking that could not be left. A test
//! claiming the crate is not linked would be a test that is simply wrong, and
//! the day somebody noticed, the promise it was standing for would go with it.
//!
//! So what is held is the sentence that is actually true and is actually the
//! point: **the only road into this crate from where a turn runs is its
//! vocabulary.** `alo-saying` is the one crate in the set that ships it; the
//! only names anything there spells are `declare_into` and `letting_go_words`;
//! and nothing in the set names the folder a person leaves an asking in, the
//! unit that carries it out, or either of the two functions that are the road.
//! A crate that cannot name a road cannot take it by accident, and one that
//! began naming it would be doing so in a change somebody had to write.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what could not be read is the failure being reported"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The crates an agent's turn really is: the daemon it speaks to, and the turn.
const WHERE_A_TURN_RUNS: [&str; 2] = ["alo-agentd", "alo-turn"];

/// This crate.
const THIS_CRATE: &str = "alo-letting-go";

/// The one crate a turn runs inside that may ship this one, and why: it is the
/// machine's one vocabulary and ships every crate that declares a word.
const FOR_ITS_WORDS: &str = "alo-saying";

/// The only two names anything outside this crate may spell of it — both of
/// them a vocabulary, neither of them an act.
const THE_VOCABULARY: [&str; 2] = ["declare_into", "letting_go_words"];

/// Where every crate in this workspace is.
fn the_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("this crate is under crates/")
        .to_owned()
}

/// What one manifest **ships** — its dependencies, and the ones a target adds,
/// and never a dev-dependency.
///
/// Read as text rather than through a parser, for
/// `tests/nothing_here_is_a_verb.rs`'s reason: what is promised is about a file
/// somebody audits.
fn what_it_ships(manifest: &str) -> BTreeSet<String> {
    let mut shipped = BTreeSet::new();
    let mut inside = false;
    for line in manifest.lines().map(str::trim) {
        if let Some(heading) = line.strip_prefix('[') {
            inside = heading.contains("dependencies") && !heading.contains("dev-dependencies");
            continue;
        }
        if !inside || line.starts_with('#') {
            continue;
        }
        let Some((named, rest)) = line.split_once('=') else {
            continue;
        };
        let named = named.trim();
        if named.is_empty() || !rest.contains("path") {
            continue;
        }
        shipped.insert(named.trim_matches('"').to_owned());
    }
    shipped
}

/// Every crate in the workspace, and what each ships.
fn the_workspace() -> BTreeMap<String, BTreeSet<String>> {
    let mut workspace = BTreeMap::new();
    let crates = the_crates();
    for entry in std::fs::read_dir(&crates).expect("crates/ is readable") {
        let at = entry.expect("a crate directory").path();
        let manifest = at.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let named = at
            .file_name()
            .and_then(|named| named.to_str())
            .expect("a crate is named")
            .to_owned();
        let text = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|why| panic!("{} could not be read: {why}", manifest.display()));
        workspace.insert(named, what_it_ships(&text));
    }
    workspace
}

/// Everything these crates ship, and everything those ship, all the way down.
fn everything_reachable_from(roots: &[&str]) -> BTreeSet<String> {
    let workspace = the_workspace();
    let mut reached = BTreeSet::new();
    let mut walking: Vec<String> = roots.iter().map(|named| (*named).to_owned()).collect();
    while let Some(named) = walking.pop() {
        if !reached.insert(named.clone()) {
            continue;
        }
        if let Some(ships) = workspace.get(&named) {
            walking.extend(ships.iter().cloned());
        }
    }
    reached
}

/// **The only road into this crate from where a turn runs is its vocabulary.**
/// One crate in the whole set ships it, and it is the machine's one list of
/// words — so a second crate reaching for a type, a constant or a function of
/// this one fails here, in the change that did it.
#[test]
fn the_only_road_into_this_crate_from_a_turn_is_its_vocabulary() {
    let workspace = the_workspace();
    let reached = everything_reachable_from(&WHERE_A_TURN_RUNS);

    // The walk found something, so that a parsing mistake cannot make this pass
    // by reaching nothing at all.
    assert!(
        reached.len() > 10,
        "the walk reached {} crates, which is not a turn: {reached:?}",
        reached.len()
    );
    for expected in ["alo-files", "alo-capability", "alo-record", "alo-strings"] {
        assert!(
            reached.contains(expected),
            "the walk did not reach {expected}, so it is not reading the manifests"
        );
    }

    let ships_it: BTreeSet<&String> = reached
        .iter()
        .filter(|named| {
            workspace
                .get(*named)
                .is_some_and(|ships| ships.contains(THIS_CRATE))
        })
        .collect();
    assert_eq!(
        ships_it,
        BTreeSet::from([&FOR_ITS_WORDS.to_owned()]),
        "something a turn runs inside ships {THIS_CRATE} for something other than its words"
    );
}

/// **And nothing a turn runs inside can name this road.** Not the folder a
/// person leaves an asking in, not the unit that carries it out, and not either
/// of the two functions that are the act — while the one crate that does name
/// this one spells nothing of it but its vocabulary. A crate that cannot name a
/// road cannot take it by accident, and one that began naming it would be doing
/// so in a change somebody had to write.
#[test]
fn nothing_an_agents_turn_runs_inside_can_name_this_road() {
    let reached = everything_reachable_from(&WHERE_A_TURN_RUNS);
    let crates = the_crates();
    let mut read = 0_usize;

    for named in &reached {
        let at = crates.join(named);
        if !at.is_dir() || named == THIS_CRATE {
            continue;
        }
        for file in every_source_under(&at.join("src")) {
            let text = std::fs::read_to_string(&file)
                .unwrap_or_else(|why| panic!("{} could not be read: {why}", file.display()));
            read += 1;

            // The road itself: the folder a person leaves an asking in, the
            // unit that carries it out, and the two functions that are the act.
            for never in [
                alo_letting_go::THE_ASKING,
                "alo-forgetting",
                "alo_letting_go::ask",
                "alo_letting_go::forget",
                "alo_letting_go::asking",
                "alo_letting_go::forgetting",
            ] {
                assert!(
                    !text.contains(never),
                    "{} names {never}, which is a road only a person may take",
                    file.display()
                );
            }

            // And of this crate at all, only its vocabulary is ever spelt.
            for (which, _) in text.match_indices("alo_letting_go::") {
                let named = text
                    .get(which + "alo_letting_go::".len()..)
                    .unwrap_or_default()
                    .split(|what: char| !what.is_alphanumeric() && what != '_')
                    .next()
                    .unwrap_or_default();
                assert!(
                    THE_VOCABULARY.contains(&named),
                    "{} names alo_letting_go::{named}, and the only road in is its vocabulary",
                    file.display()
                );
            }
        }
    }

    assert!(
        read > 50,
        "only {read} files were read, which is not a turn"
    );
}

/// Every `.rs` file under a directory, however deep.
fn every_source_under(at: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(at) else {
        return found;
    };
    for entry in entries.flatten() {
        let at = entry.path();
        if at.is_dir() {
            found.extend(every_source_under(&at));
        } else if at.extension().is_some_and(|what| what == "rs") {
            found.push(at);
        }
    }
    found
}
