//! **ADR 0068's enforcement**: the English under a published key does not move
//! without somebody answering whether the meaning moved with it.
//!
//! [ADR 0068](../../../docs/decisions/0068-a-published-sentence-changes-by-getting-a-new-key.md)
//! decides that a sentence whose **meaning** changes gets a new key and the old
//! key retires, while a rewording that leaves the claim alone keeps the key it
//! has. The test to apply is one question:
//!
//! > Would a correct translation of the old English still be a correct
//! > translation of the new one?
//!
//! Yes — same key, and translators are told it was reworded. No — new key, and
//! the old one retires. That question is the whole rule, and this file exists
//! because three commits had already changed a `Word`'s text with nothing to
//! consult, so the next person in a hurry will do it again unless something asks.
//!
//! # How it asks
//!
//! [`SNAPSHOT`] is every key this machine can say and the English under it,
//! generated and checked in. A change that moves any of that text fails
//! [`the_snapshot_is_what_this_machine_says`], which prints the question above
//! and what to do about either answer.
//!
//! Regenerating the snapshot is the moment the answer is required:
//!
//! ```text
//! UPDATE_VOCABULARY_SNAPSHOT=1 cargo test -p alo-saying --test a_published_sentence_keeps_its_key
//! ```
//!
//! That **refuses to write** for any key whose text moved and which
//! [`REWORDED`] does not cover, naming the key. So the declaration is not a
//! formality alongside the change — it is the thing that lets the change be made.
//!
//! # What this is, and what it is not
//!
//! It makes the question unavoidable **in review**: a moved sentence appears as a
//! changed line in a generated file beside a required declaration saying why the
//! meaning held. It does not make a false declaration impossible, and nothing
//! mechanical could — whether a translation of the old English still fits the new
//! one is a judgement in every language the sentence exists in. What it removes is
//! the case that actually happened: the text moving with **nobody asked at all**.
//!
//! A new key and a retired key need no declaration. Adding a sentence is
//! additive, and a retired key's translations are simply never consulted again
//! (ADR 0068 §4) — the harm this guards against is a translation that still
//! renders and now lies.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// Every key and the English under it, generated and checked in.
const SNAPSHOT: &str = "the-vocabulary.txt";

/// Where a rewording that kept its key says why the meaning held.
const REWORDED: &str = "reworded.txt";

/// Setting this regenerates [`SNAPSHOT`], refusing any undeclared rewording.
const UPDATE: &str = "UPDATE_VOCABULARY_SNAPSHOT";

/// The question ADR 0068 turns on, printed wherever this fails.
const THE_QUESTION: &str =
    "Would a correct translation of the OLD English still be a correct translation of the NEW one?";

/// This crate's directory.
fn beside_this_crate(named: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(named)
}

/// Every key this machine can say, and the English under it.
fn what_this_machine_says() -> BTreeMap<String, String> {
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");
    let mut said = BTreeMap::new();
    for phrase in vocabulary.phrases() {
        let key = phrase.key().as_str().to_owned();
        let english = phrase.source().as_written().to_owned();
        assert!(
            !english.contains('\t') && !english.contains('\n'),
            "{key} holds a tab or a newline, which this snapshot's one-line-per-key \
             shape cannot record — and a sentence a person reads has no business with either"
        );
        assert!(
            said.insert(key.clone(), english).is_none(),
            "{key} is said twice by one vocabulary"
        );
    }
    assert!(
        said.len() > 400,
        "only {} keys were read, which is not this machine's vocabulary",
        said.len()
    );
    said
}

/// [`SNAPSHOT`] as it is checked in: every key and the English under it.
fn as_last_declared() -> BTreeMap<String, String> {
    let at = beside_this_crate(SNAPSHOT);
    let text = fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
    let mut said = BTreeMap::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, english) = line
            .split_once('\t')
            .unwrap_or_else(|| panic!("{SNAPSHOT} has a line with no tab in it: {line}"));
        said.insert(key.to_owned(), english.to_owned());
    }
    said
}

/// [`REWORDED`], read: each key it covers, and the English it says that key now
/// reads.
fn rewordings_declared() -> BTreeMap<String, String> {
    let at = beside_this_crate(REWORDED);
    let text = fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
    let mut declared = BTreeMap::new();
    let mut key: Option<String> = None;
    let mut now: Option<String> = None;
    let mut why: Option<String> = None;
    for line in text.lines() {
        let line = line.trim_end();
        if line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("key: ") {
            assert!(
                key.is_none(),
                "{REWORDED}: a record began before the last ended"
            );
            key = Some(rest.trim().to_owned());
        } else if let Some(rest) = line.strip_prefix("now: ") {
            now = Some(rest.to_owned());
        } else if let Some(rest) = line.strip_prefix("why: ") {
            why = Some(rest.trim().to_owned());
        } else if line.is_empty() {
            if let Some(named) = key.take() {
                let reads = now
                    .take()
                    .unwrap_or_else(|| panic!("{REWORDED}: {named} has no `now:` line"));
                let reason = why
                    .take()
                    .unwrap_or_else(|| panic!("{REWORDED}: {named} has no `why:` line"));
                assert!(
                    reason.len() > 40,
                    "{REWORDED}: {named}'s `why:` does not answer the question — {THE_QUESTION}"
                );
                declared.insert(named, reads);
            }
            now = None;
            why = None;
        }
    }
    assert!(
        key.is_none(),
        "{REWORDED} ends mid-record; a record ends with a blank line"
    );
    declared
}

/// **The snapshot is what this machine says** — and a sentence whose English has
/// moved stops here until somebody answers ADR 0068's question.
#[test]
fn the_snapshot_is_what_this_machine_says() {
    let live = what_this_machine_says();

    if std::env::var_os(UPDATE).is_some() {
        write_the_snapshot(&live);
        return;
    }

    let declared = as_last_declared();
    let moved: Vec<(&String, &String, &String)> = live
        .iter()
        .filter_map(|(key, english)| {
            declared
                .get(key)
                .filter(|was| *was != english)
                .map(|was| (key, was, english))
        })
        .collect();

    let mut said = String::new();
    for (key, was, english) in &moved {
        writeln!(said, "\n  {key}\n    was: {was}\n    now: {english}").unwrap();
    }
    assert!(
        moved.is_empty(),
        "The English under {} published key(s) has moved.\n\n{THE_QUESTION}\n\n\
         YES, the meaning is unchanged — keep the key: add a record to {REWORDED} saying so, \
         then regenerate with `{UPDATE}=1 cargo test -p alo-saying \
         --test a_published_sentence_keeps_its_key`.\n\n\
         NO, the meaning changed — the key may not carry it (ADR 0068): add a new key, retire \
         this one in the same change, and regenerate.\n\
         {said}",
        moved.len()
    );

    let added: Vec<&String> = live
        .keys()
        .filter(|key| !declared.contains_key(*key))
        .collect();
    let retired: Vec<&String> = declared
        .keys()
        .filter(|key| !live.contains_key(*key))
        .collect();
    assert!(
        added.is_empty() && retired.is_empty(),
        "{SNAPSHOT} is out of date: {} key(s) added and {} retired. Neither needs a \
         declaration — a new sentence is additive and a retired key's translations are \
         never consulted again — so regenerate with `{UPDATE}=1 cargo test -p alo-saying \
         --test a_published_sentence_keeps_its_key`.\n  added:   {added:?}\n  retired: {retired:?}",
        added.len(),
        retired.len()
    );
}

/// **Every rewording declared still reads as it says it does.** A record that has
/// gone stale — the sentence moved again, or was retired — is a declaration
/// standing for something that is no longer there, which is the thing this file
/// is against.
#[test]
fn every_rewording_declared_still_reads_as_it_says() {
    let live = what_this_machine_says();
    for (key, reads) in rewordings_declared() {
        let now = live.get(&key).unwrap_or_else(|| {
            panic!(
                "{REWORDED} declares a rewording of {key}, which this machine no longer says. \
                 A retired key's record goes with it."
            )
        });
        assert_eq!(
            now, &reads,
            "{REWORDED} says {key} now reads one thing and it reads another. If it was reworded \
             again, the record says so; if the meaning moved, ADR 0068 says it is a new key."
        );
    }
}

/// Write [`SNAPSHOT`] from what this machine says — refusing any key whose
/// English has moved and which [`REWORDED`] does not cover.
///
/// The refusal is the enforcement. Regenerating is the one moment somebody
/// **must** answer ADR 0068's question, so it is the moment to ask.
fn write_the_snapshot(live: &BTreeMap<String, String>) {
    let declared = as_last_declared();
    let rewordings = rewordings_declared();

    let undeclared: Vec<&String> = live
        .iter()
        .filter(|(key, english)| {
            declared
                .get(*key)
                .is_some_and(|was| was != *english && rewordings.get(*key) != Some(english))
        })
        .map(|(key, _)| key)
        .collect();
    assert!(
        undeclared.is_empty(),
        "REFUSING to write {SNAPSHOT}. The English under {} key(s) has moved and {REWORDED} does \
         not cover it.\n\n{THE_QUESTION}\n\n\
         Answer it in {REWORDED} — `key:`, `now:` with the exact new English, and `why:` saying \
         why a translation of the old still fits — or give the sentence a new key and retire this \
         one (ADR 0068).\n  {undeclared:?}",
        undeclared.len()
    );

    let mut text = String::new();
    text.push_str(concat!(
        "# Every key this machine can say, and the English under it.\n",
        "#\n",
        "# GENERATED. Do not edit by hand:\n",
        "#   UPDATE_VOCABULARY_SNAPSHOT=1 cargo test -p alo-saying \\\n",
        "#     --test a_published_sentence_keeps_its_key\n",
        "#\n",
        "# It exists so that the English under a published key cannot move without somebody\n",
        "# answering whether the meaning moved with it. A translation is keyed, so editing the\n",
        "# English under a key leaves every translation of it rendering perfectly and saying\n",
        "# something this product no longer says. ADR 0068 decides what may change; reworded.txt\n",
        "# beside this file is where a rewording that kept its key says why the meaning held.\n",
        "#\n",
        "# One line per key: the key, a tab, then the English exactly as the crate declares it.\n",
        "\n",
    ));
    for (key, english) in live {
        writeln!(text, "{key}\t{english}").unwrap();
    }
    let at = beside_this_crate(SNAPSHOT);
    fs::write(&at, text)
        .unwrap_or_else(|why| panic!("{} could not be written: {why}", at.display()));
    println!("wrote {} with {} keys", at.display(), live.len());
}
