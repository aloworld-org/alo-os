//! The two sizes rule 4 left without an artefact.
//!
//! Rule 4 of `data/catalogue.toml` made `quantisation` a claim an entry has to
//! be able to point at, and two entries could point at nothing:
//! `eurollm-9b-instruct` and `teuken-7b-instruct` name publishers who ship no
//! GGUF, so both state no quantisation. Their sizes were left where they were —
//! 5.6 GB and 4.6 GB, four-bit figures for artefacts neither entry any longer
//! claims — and `min_vram_gb` and `min_ram_gb` came from the same assumption.
//! Teuken stated ten gigabytes of system memory beside weights that are
//! fifteen, which is a figure somebody would size a machine against and be
//! wrong.
//!
//! **The road this catalogue took is the publisher's own release.** Where no
//! first-party quantised artefact exists, the entry states the weights its
//! publisher actually publishes, read off that repository's own file list. The
//! alternative — naming a stranger's requantisation — was refused because it
//! would put this catalogue's authority behind a file this catalogue never
//! chose, and because choosing whose requantisation is a decision with nobody's
//! name on it. `docs/quirks.md` has the reasoning under *Two entries kept a
//! four-bit size after they stopped claiming a four-bit file*.
//!
//! **Task 15 then paid rule 6's price for both entries**, so neither falls back
//! to a publisher's release any longer: each names a third party's Q4_K_M with a
//! digest behind it, and the figures a reader checks are that artefact's.
//! `the_file_the_two_european_entries_mean.rs` holds them to the uploads they
//! now name, which is where the *state your publisher's release* assertions went
//! — the fallback this task chose is still the shape an entry takes on its way
//! in, and every refusal below is still the rule that makes it one.
//!
//! What this file holds, one test per acceptance criterion in
//! `docs/autonomy/v0-01-lane-b-plan.md`'s task 13:
//!
//! - **The carry-or-fetch table agrees**, because correcting a size moves the
//!   measurement ADR 0025 owes, and a table left behind would say the weights
//!   question is smaller than it is.
//! - **The road is in the catalogue's own rules**, beside rule 4, because the
//!   next curator reads those and not the plan.
//! - **And the rule refuses what it exists to refuse**, which is the half a
//!   green run cannot tell anybody: every way a size can stop belonging to the
//!   artefact its entry names is put in front of the loader here.
//!
//! It needs no model, no runtime and no socket: it reads this repository's own
//! files and the catalogue compiled into this crate, for the reason
//! `the_carry_or_fetch_measurement.rs` gives.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{fs, path::Path};

use alo_models::{Catalogue, Driving, Model};

/// The two entries this task's correction was about, by the ids the catalogue
/// uses.
const THE_TWO: [&str; 2] = ["eurollm-9b-instruct", "teuken-7b-instruct"];

/// Where the carry-or-fetch measurement lives.
const THE_MEASUREMENT: &str = "docs/quirks.md";

/// The catalogue's own rules, which are its header comment.
const THE_RULES: &str = "crates/alo-models/data/catalogue.toml";

/// A file of this repository, read.
fn reading(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// The catalogue we ship, which loads before anything here is asked of it.
fn the_catalogue() -> Catalogue {
    Catalogue::built_in().expect("the built-in catalogue loads; its own test says so first")
}

/// One entry of the catalogue we ship.
fn the_entry(id: &str) -> Model {
    the_catalogue()
        .get(id)
        .unwrap_or_else(|| panic!("the catalogue no longer offers `{id}`"))
        .clone()
}

/// An entry written out, so a refusal can be shown happening.
///
/// Everything but the three figures under test is sound, so each refusal below
/// is about the fault it names and not about a fixture that was never loadable.
fn an_entry(parameters_b: f32, bytes: u64, vram: f32, ram: f32, quantised: bool) -> String {
    let pair = if quantised {
        "quantisation = \"Q4_K_M\"\nartefact = \"runtime:sized-q4_K_M\""
    } else {
        ""
    };
    format!(
        "[[model]]\nid = \"sized\"\nname = \"Sized\"\npublisher = \"p\"\n\
         parameters_b = {parameters_b}\n{pair}\n\
         download_bytes = {bytes}\nmin_vram_gb = {vram}\nmin_ram_gb = {ram}\n\
         on_cpu = \"slow\"\ndrives_verbs = \"not-measured\"\n\
         upstream = \"https://example.test/sized\"\n\
         licence = {{ name = \"Apache-2.0\", spdx = \"Apache-2.0\", \
         commercial_use = \"permitted\" }}\n"
    )
}

/// **No grade moved.** A size is not a measurement of driving, and neither
/// entry has been measured — the constraint the task is bounded by, as a test
/// rather than as an intention.
///
/// Still true after task 15 named an artefact for each of them, and for a
/// stronger reason than it was here: naming a file is not running one either.
#[test]
fn correcting_a_size_moved_no_grade() {
    for id in THE_TWO {
        assert_eq!(
            the_entry(id).drives_verbs,
            Driving::NotMeasured,
            "`{id}` gained a grade in a change that ran no measurement"
        );
    }
}

/// **The carry-or-fetch table is brought back into agreement**, with the two
/// corrected sizes in it.
///
/// `the_carry_or_fetch_measurement.rs` already refuses a table that disagrees
/// with the catalogue in general. This is the same requirement aimed at this
/// change in particular: the numbers that moved are the numbers that had to
/// move there, so a passing suite cannot be a table that agrees because both
/// halves were left behind together.
#[test]
fn the_carry_or_fetch_table_carries_the_two_corrected_sizes() {
    let measurement = reading(THE_MEASUREMENT);
    for id in THE_TWO {
        let model = the_entry(id);
        let row = format!(
            "| `{id}` | {} | `not-measured` |",
            with_underscores(model.download_bytes)
        );
        assert!(
            measurement.contains(&row),
            "{THE_MEASUREMENT} does not carry `{row}`. Correcting a size moves the measurement \
             ADR 0025 owes, and a table left behind says the weights question is smaller than it \
             is"
        );
    }
    // The two four-bit leftovers this task replaced, and the two publishers'
    // releases task 15 replaced in turn: a table row carrying any of the four
    // is a row about a file no entry names.
    for stale in [
        "5_600_000_000",
        "4_600_000_000",
        "18_304_683_360",
        "14_905_484_192",
    ] {
        assert!(
            !measurement.contains(&format!("| {stale} |")),
            "{THE_MEASUREMENT} still carries {stale} as a table row: that is a size one of \
             these two entries has stopped stating, so the table is answering for a file the \
             catalogue does not name"
        );
    }
}

/// A byte count the way `data/catalogue.toml` and the measurement's table write
/// one — groups of three from the right, underscore-separated.
fn with_underscores(bytes: u64) -> String {
    let digits = bytes.to_string();
    let mut out = String::new();
    for (seen, digit) in digits.chars().rev().enumerate() {
        if seen > 0 && seen % 3 == 0 {
            out.push('_');
        }
        out.push(digit);
    }
    out.chars().rev().collect()
}

/// **The road is written into the catalogue's own rules, beside rule 4**,
/// because the next curator reads those and not this lane's plan.
///
/// Held to three things a reader needs: that there is a fifth rule at all, that
/// it says which road was taken, and that it says what was refused — a rule
/// that recorded only the choice would leave the next curator free to make the
/// other one without knowing it had been considered.
#[test]
fn the_road_taken_is_written_into_the_catalogues_own_rules() {
    let rules = reading(THE_RULES);
    let header: String = rules
        .lines()
        .take_while(|line| line.starts_with('#') || line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    // The count moved to six when ADR 0026 answered what an entry may do
    // *instead* of falling back to a publisher's release; rule 5 is still the
    // fallback, and this test is still about it.
    assert!(
        header.contains("Six rules") && header.contains("# 5. "),
        "the catalogue's header does not carry a fifth rule, so the road this change took lives \
         only in a plan the next curator will not read"
    );
    // Rule 5 alone: its own line and the indented ones under it, so a phrase
    // borrowed from the narrative further down the header cannot stand in for
    // the rule saying it.
    let (_, after) = header
        .split_once("# 5. ")
        .expect("the header has a rule 5; the assertion above said so");
    let rule_five = after
        .lines()
        .take_while(|line| !line.starts_with("# ") || line.starts_with("#    "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !rule_five.contains("Seven entries"),
        "the extraction ran past rule 5 into the header's narrative, so every assertion below \
         could be satisfied by a sentence the rule does not make"
    );
    assert!(
        rule_five.contains("publisher's own release"),
        "rule 5 does not say which road was taken for an entry that names no artefact"
    );
    assert!(
        rule_five.contains("requantisation"),
        "rule 5 does not say what was refused, so the next curator cannot tell a decision from an \
         omission"
    );
    assert!(
        rule_five.contains("min_vram_gb") && rule_five.contains("min_ram_gb"),
        "rule 5 covers the size and leaves the two memory figures, which came from the same \
         four-bit assumption"
    );
}

/// **And the rule refuses each way a size can stop belonging to the artefact
/// its entry names**, starting from a sound entry so the refusals are about the
/// fault and not about a fixture that never loaded.
#[test]
fn a_size_that_belongs_to_no_artefact_the_entry_names_is_refused() {
    // Sound: 1.7 billion parameters at four bits, and again at `bfloat16` with
    // no quantisation claimed. Both shapes the catalogue allows.
    for (bytes, vram, ram, quantised) in [
        (1_060_000_000, 2.0, 3.0, true),
        (3_400_000_000, 4.0, 6.0, false),
    ] {
        assert!(
            Catalogue::parse(&an_entry(1.7, bytes, vram, ram, quantised)).is_ok(),
            "a sound entry was refused, so the refusals below say nothing"
        );
    }

    // The one this test exists for: the four-bit size left behind on an entry
    // that has given up its quantisation. This is `teuken-7b-instruct` and
    // `eurollm-9b-instruct` exactly as they stood.
    refused(
        an_entry(1.7, 1_060_000_000, 2.0, 3.0, false),
        "never chose",
        "a four-bit size on an entry that names no artefact was accepted",
    );

    // Its mirror: a full-precision size on an entry that says it fetches a
    // quantised file. The same mistake pointing the other way, and the one a
    // curator makes when they add an artefact and forget the size.
    refused(
        an_entry(1.7, 3_400_000_000, 4.0, 6.0, true),
        "not what the release it came from does",
        "a release-sized figure on a quantised entry was accepted",
    );

    // A size that is no artefact of this many parameters at either precision.
    refused(
        an_entry(1.7, 1, 2.0, 3.0, true),
        "not about the same model",
        "a token size was accepted; `download_bytes = 1` is what six fixtures used to say",
    );
    refused(
        an_entry(1.7, 20_000_000_000, 24.0, 28.0, false),
        "not the weights alone",
        "a size larger than these parameters can be at full precision was accepted",
    );

    // The two memory figures, which came from the same assumption as the size
    // and are the ones a person sizes a machine against.
    refused(
        an_entry(7.5, 14_905_484_192, 6.0, 21.0, false),
        "card too small",
        "a card that cannot hold the weights was accepted",
    );
    refused(
        an_entry(7.5, 14_905_484_192, 17.0, 10.0, false),
        "system memory too small",
        "ten gigabytes beside fifteen gigabytes of weights was accepted, which is the figure \
         `teuken-7b-instruct` actually carried",
    );

    // And the parameter count the ratio is taken against, without which none of
    // the above is checking anything.
    refused(
        an_entry(0.0, 1_060_000_000, 2.0, 3.0, true),
        "checked against nothing",
        "an entry with no parameter count was accepted",
    );
    refused(
        an_entry(1.7, 1_060_000_000, 2.0, 0.0, true),
        "system memory must be stated",
        "an entry stating no system memory was accepted",
    );
}

/// One refusal, shown happening: the entry fails to load and says why in words
/// a curator can act on.
fn refused(text: String, saying: &str, complaint: &str) {
    match Catalogue::parse(&text) {
        Ok(_) => panic!("{complaint}"),
        Err(why) => assert!(
            why.to_string().contains(saying),
            "{complaint}: it was refused, and for `{why}` rather than for `{saying}`"
        ),
    }
}
