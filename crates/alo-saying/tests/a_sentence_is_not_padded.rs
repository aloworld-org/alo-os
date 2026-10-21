//! No sentence this machine says has whitespace in it that nobody meant.
//!
//! # How this was found, which is the argument for it existing
//!
//! By **running the program**, once, for the first time. The Windows installer
//! had never been built as a binary on the machine it installs from — no
//! toolchain for it was present — so nobody had ever seen it print anything.
//! The first time it ran, it said:
//!
//! ```text
//! This computer keeps no choice of which system starts, so there is nothing
//! to change here.      Nothing was changed
//! ```
//!
//! Six spaces, in the middle of a published sentence, declared that way in
//! `words.rs` and visible to every translator who would ever open it. Nine
//! more were found the same way across six crates.
//!
//! **Nothing could have caught it.** `the-vocabulary.txt` holds every key and
//! its English and would have shown the spaces to anybody reading that line —
//! it catches a sentence *changing*, not a sentence being wrong when it was
//! written. Every other check asks whether a key exists, whether a gap is
//! filled, whether a translation is present. None of them reads the text as
//! text.
//!
//! # Why three spaces and not two
//!
//! Because two are sometimes meant. `alo_installing::words::A_DISK_AND_ITS_SIZE`
//! joins a disk and its size with a middle dot and **two spaces each side**,
//! and says so in its own note. Three in a row has no use in any sentence in
//! any of the 24 languages this product promises, so three is where the line
//! goes.
//!
//! A tab, a newline and a carriage return are refused outright: a sentence is
//! one line, and `crates/alo-saying/tests/a_published_sentence_keeps_its_key.rs`
//! already depends on that to keep its snapshot a table.
//!
//! # What this does not cover, and it is one crate
//!
//! Whatever `alo-saying` does not collect — and there is exactly one such
//! crate. `alo-agentd` is excluded deliberately, for a reason written down in
//! both its own `Cargo.toml` entry and `collecting.rs`: it is Linux-only. **So
//! that crate's sentences are not checked here.** Three of them were carrying
//! this same fault and were fixed in the change that added this file, and they
//! were found by reading the source rather than by anything in it.
//!
//! That is worth stating rather than leaving to be discovered. A reader who
//! sees this test pass should not conclude that no sentence anywhere is
//! padded; it means no sentence **this machine collects** is. Covering the
//! last crate needs a source-level check, which is a different test from this
//! one.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

/// Three in a row is where the line goes. See this file's header for why not
/// two.
const TOO_MANY: &str = "   ";

#[test]
fn no_sentence_has_a_run_of_spaces_nobody_meant() {
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");

    let mut padded: Vec<String> = Vec::new();
    for phrase in vocabulary.phrases() {
        let english = phrase.source().as_written();
        if english.contains(TOO_MANY) {
            padded.push(format!("{}: {english:?}", phrase.key().as_str()));
        }
    }
    padded.sort_unstable();

    assert!(
        padded.is_empty(),
        "{} published sentence(s) carry a run of three or more spaces that \
         nobody meant. They were written that way, so every translation will \
         copy them, and a person reads the gap on their screen:\n  {}",
        padded.len(),
        padded.join("\n  ")
    );
}

#[test]
fn no_sentence_is_more_than_one_line_or_has_a_tab_in_it() {
    // A sentence is one line. The vocabulary snapshot is a table of
    // key-and-tab-and-English, so a tab or a newline inside a sentence would
    // make a row that cannot be read back — which is why
    // `a_published_sentence_keeps_its_key` asserts it too, on its way past.
    // Asserted here as well because that one asserts it in order to build its
    // snapshot, and this one asserts it as a property of the sentence.
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");

    let mut wrong: Vec<String> = Vec::new();
    for phrase in vocabulary.phrases() {
        let english = phrase.source().as_written();
        let named = |what: &str| format!("{} has {what}", phrase.key().as_str());
        if english.contains('\t') {
            wrong.push(named("a tab in it"));
        }
        if english.contains('\n') {
            wrong.push(named("a newline in it"));
        }
        if english.contains('\r') {
            wrong.push(named("a carriage return in it"));
        }
    }
    wrong.sort_unstable();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn no_sentence_begins_or_ends_with_a_space() {
    // A leading space is drawn, so it moves the first word; a trailing one
    // pushes whatever follows. Both are the same kind of fault as the run
    // above and neither is visible in a diff.
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");

    let mut edged: Vec<String> = Vec::new();
    for phrase in vocabulary.phrases() {
        let english = phrase.source().as_written();
        if english != english.trim() {
            edged.push(format!("{}: {english:?}", phrase.key().as_str()));
        }
    }
    edged.sort_unstable();
    assert!(
        edged.is_empty(),
        "{} sentence(s) begin or end with whitespace:\n  {}",
        edged.len(),
        edged.join("\n  ")
    );
}

#[test]
fn the_machine_says_enough_for_these_to_mean_anything() {
    // **The control.** Every assertion above passes trivially against a
    // vocabulary that is empty or nearly so — and this crate has already had
    // a search that under-reported. If the collection ever stops finding most
    // of the sentences, these three checks stop being about anything and say
    // nothing about it.
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");
    let how_many = vocabulary.phrases().count();
    assert!(
        how_many > 1_000,
        "only {how_many} sentences were collected, so the checks in this file \
         are not reading the machine's vocabulary"
    );
}
