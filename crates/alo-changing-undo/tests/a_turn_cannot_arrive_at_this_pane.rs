//! **A turn cannot arrive at this pane** — the same walk
//! `crates/alo-letting-go/tests/a_turn_cannot_arrive_at_this_road.rs` makes,
//! with this crate held to the same answer.
//!
//! Task 15 of `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` asked for
//! this by name, so that *a Settings pane cannot become a way in from a turn by
//! being linked somewhere convenient*. The worry is exact: a pane is an ordinary
//! crate with no capability of its own, and nothing about it looks dangerous in a
//! `Cargo.toml`. What makes it worth a test is that it sits one name away from
//! the privileged remover, and a pane that grew a dependency on
//! `alo-letting-go` for a type — months from now, for a perfectly good local
//! reason — would carry that road into the process an agent's turn runs in.
//!
//! # What is held, and why it is not "this crate is not linked"
//!
//! This crate **is** reachable from a turn, and the honest test says so.
//! `alo-agentd` ships `alo-saying`, which is the machine's one vocabulary and
//! therefore ships every crate that declares a word — this one included, for its
//! three sentences. A test claiming otherwise would be wrong, and the day
//! somebody noticed, the promise it stood for would go with it.
//!
//! So what is held is the sentence that is true and is the point: **the only road
//! into this crate from where a turn runs is its vocabulary**, and **this crate
//! itself cannot reach the act or the file** — it ships no dependency on
//! `alo-letting-go`, names neither of the two functions that are the act, and
//! names no file of a person's. A pane that cannot name a road cannot take it by
//! accident, and one that began naming it would be doing so in a change somebody
//! had to write.

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
const THIS_CRATE: &str = "alo-changing-undo";

/// The crate holding the one privileged remover, which this one must not ship.
const THE_PRIVILEGED_ONE: &str = "alo-letting-go";

/// The one crate a turn runs inside that may ship this one, and why: it is the
/// machine's one vocabulary and ships every crate that declares a word.
const FOR_ITS_WORDS: &str = "alo-saying";

/// The only names anything outside this crate may spell of it — its vocabulary,
/// and nothing that decides anything.
const THE_VOCABULARY: [&str; 2] = ["declare_into", "changing_undo_words"];

/// Where every crate in this workspace is.
fn the_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("this crate is under crates/")
        .to_owned()
}

/// What one manifest **ships** — its dependencies, and the ones a target adds,
/// and never a dev-dependency, which is a test's and is in nobody's process.
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

/// **This pane ships no road to the act.** Not `alo-letting-go`, at any depth of
/// what it ships — so the one privileged remover on this machine cannot arrive
/// here by way of a pane, and `alo-letting-go`'s own test keeps holding.
#[test]
fn this_pane_ships_no_road_to_the_act() {
    let reached = everything_reachable_from(&[THIS_CRATE]);
    assert!(
        reached.contains("alo-keeping-up"),
        "the walk did not reach alo-keeping-up, so it is not reading the manifests"
    );
    assert!(
        !reached.contains(THE_PRIVILEGED_ONE),
        "{THIS_CRATE} reaches {THE_PRIVILEGED_ONE}, which puts the one privileged remover \
         inside every process this pane is linked into — including a turn's, by way of \
         {FOR_ITS_WORDS}: {reached:?}"
    );
}

/// **The only road into this pane from where a turn runs is its vocabulary.**
/// One crate in the whole set ships it, and it is the machine's one list of
/// words — so a second crate reaching for a type, a constant or a function of
/// this one fails here, in the change that did it.
#[test]
fn the_only_road_into_this_pane_from_a_turn_is_its_vocabulary() {
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

/// **And this pane names no road of its own.** Not the folder a person leaves an
/// asking in, not the unit that carries it out, not either of the two functions
/// that are the act, and not the person's file. Held over this crate's own
/// sources, because the dependency test above is about manifests and a name can
/// arrive in a file before it arrives in one.
#[test]
fn this_pane_names_no_road_a_person_alone_may_take() {
    let mut read = 0_usize;
    for file in every_source_under(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path()) {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|why| panic!("{} could not be read: {why}", file.display()));
        read += 1;
        for never in [
            "/run/alo/asked-to-forget",
            "alo-forgetting",
            "alo_letting_go::ask",
            "alo_letting_go::forget",
            "alo_letting_go::asking",
            "alo_letting_go::forgetting",
            "undo.toml",
        ] {
            assert!(
                !text.contains(never),
                "{} names {never}, which is a road only a person may take, from a crate \
                 that is linked into a turn's process",
                file.display()
            );
        }
    }
    assert!(
        read >= 4,
        "only {read} files were read, which is not this crate"
    );
}

/// **Nothing outside this crate spells anything of it but its vocabulary.**
/// The mirror of the test above, over every crate a turn reaches: a surface may
/// declare this pane's words, and may not name its decisions.
#[test]
fn nothing_a_turn_runs_inside_names_more_than_this_panes_words() {
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
            for (which, _) in text.match_indices("alo_changing_undo::") {
                let named = text
                    .get(which + "alo_changing_undo::".len()..)
                    .unwrap_or_default()
                    .split(|what: char| !what.is_alphanumeric() && what != '_')
                    .next()
                    .unwrap_or_default();
                assert!(
                    THE_VOCABULARY.contains(&named),
                    "{} names alo_changing_undo::{named}, and the only road in is its vocabulary",
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
