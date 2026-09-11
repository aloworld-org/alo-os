//! The file the two European entries mean.
//!
//! ADR 0026 decided what naming a stranger's requantisation costs — whose it
//! is, which file exactly, and what a reader needs to know — and left the
//! choosing to a curator, because picking a file is an act with a name on it
//! rather than a side effect of writing a rule. Nobody had paid that price:
//! `eurollm-9b-instruct` and `teuken-7b-instruct` still claimed no quantisation
//! and still stated their publishers' `bfloat16` releases, 18.30 GB and
//! 14.91 GB, which is the honest fallback and is not a model anybody runs.
//!
//! This file is the choice, as a check rather than as a claim. One test per
//! acceptance criterion in `docs/autonomy/v0-01-lane-b-plan.md`'s task 15:
//!
//! - **Each entry names an upload and states all three things about it**, and
//!   the artefact and its digest are compared with what those repositories
//!   published on 2026-09-11, written down here. A test that only asked the
//!   catalogue whether it was self-consistent would pass on an invented digest.
//! - **Every figure that moves with it has moved**: rule 5 binds
//!   `download_bytes`, `min_vram_gb`, `min_ram_gb` and `on_cpu` to the artefact
//!   now named, so the size is that file's own and the memory figures are above
//!   it.
//! - **The licence was read against the uploader's repository too**, which is
//!   the criterion that nearly went wrong: openGPT-X publishes Teuken twice
//!   under two licences and the two best-known requantisers took the research
//!   one. So the artefact an entry names has to be a requantisation of the
//!   release its own `upstream` names, and that is checked rather than trusted.
//! - **No grade was earned**, because no run was made: this lane's box cannot
//!   hold a 7B model at four bits (`the_grade_the_weights_wait_on.rs` has the
//!   numbers), and an artefact named is not an artefact measured.
//! - **And the consequence is stated rather than discovered**: a four-bit Teuken
//!   is `workable` again, so the shorter list a machine with 16 GB and no card
//!   is offered goes back to seven.
//!
//! It needs no model, no runtime and no socket: it reads the catalogue compiled
//! into this crate and this repository's own files, which is deliberate — the
//! box that curated these entries cannot load either of them.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{fs, path::Path};

use alo_models::{Catalogue, Driving, Model, OnCpu};

/// Where the carry-or-fetch measurement lives.
const THE_MEASUREMENT: &str = "docs/quirks.md";

/// One upload, as the repository it lives in published it on 2026-09-11.
///
/// These are the numbers this test compares the catalogue against, and they
/// come from outside it: the artefact as the pinned runtime would spell it, the
/// `sha256` the repository's own file list gives for that file, and the bytes
/// that file actually is. A catalogue entry cannot satisfy them by being
/// internally tidy.
struct Upload {
    /// The catalogue id.
    id: &'static str,
    /// Who made the file, as they publish under.
    by: &'static str,
    /// What the entry fetches, spelled as the runtime spells it.
    artefact: &'static str,
    /// The artefact's own digest, off the repository's file list.
    sha256: &'static str,
    /// What that file is, in bytes.
    bytes: u64,
    /// How the entry behaves on a processor once it is this size.
    on_cpu: OnCpu,
}

/// The two uploads chosen, and why each rather than its neighbours — the
/// grounds live in the entries and in `docs/quirks.md`; what is here is the
/// evidence a reader can check.
const THE_TWO: [Upload; 2] = [
    Upload {
        id: "eurollm-9b-instruct",
        by: "bartowski",
        artefact: "hf.co/bartowski/EuroLLM-9B-Instruct-GGUF:Q4_K_M",
        sha256: "785a3b2883532381704ef74f866f822f179a931801d1ed1cf12e6deeb838806b",
        bytes: 5_582_838_496,
        // Nine billion parameters is `gemma-2-9b-instruct`'s class, and that
        // entry is slow for the same reason: the weights fit an ordinary
        // laptop's memory and the arithmetic does not fit its processor.
        on_cpu: OnCpu::Slow,
    },
    Upload {
        id: "teuken-7b-instruct",
        by: "mradermacher",
        artefact: "hf.co/mradermacher/Teuken-7B-instruct-commercial-v0.4-GGUF:Q4_K_M",
        sha256: "03fd13daafb6f20c1c5f4b908d163841cdd96803a6f5a7c0c39dd236a2b1630b",
        bytes: 5_018_868_512,
        on_cpu: OnCpu::Workable,
    },
];

/// Bytes per parameter at which `Catalogue::parse` separates a quantised
/// artefact's size from a full-precision release's.
///
/// The loader's own constant is private, which is right. This is the figure
/// rule 5 states in words, checked against the behaviour rather than shared
/// with it.
const THE_PRECISION_LINE: f64 = 1.5;

/// A file of this repository, read.
fn reading(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// One entry of the catalogue we ship.
fn the_entry(id: &str) -> Model {
    Catalogue::built_in()
        .expect("the built-in catalogue loads; its own test says so first")
        .get(id)
        .unwrap_or_else(|| panic!("the catalogue no longer offers `{id}`"))
        .clone()
}

/// The last segment of a Hugging Face repository path — the release's own name.
fn the_release_in(upstream: &str) -> &str {
    upstream
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(upstream)
}

/// **Whether this entry's artefact is a requantisation of the release its
/// `upstream` names**, by that release's own name.
///
/// The check the Teuken split needs, and the reason it is a function rather
/// than four lines inside a test: it has a refusal path worth showing. A
/// publisher who ships one model under two licences ships two releases with
/// different names, and an uploader's repository is named after the one they
/// converted. So an entry whose licence is true of the commercial release and
/// whose artefact says `research` is a licence stated wrongly, which is rule 1's
/// harm arriving through rule 6's door — and nothing in the loader can see it,
/// because a research requantisation has a perfectly good digest.
///
/// **Asked only of entries that borrow a file**, which is where the question
/// exists. A first-party artefact is spelled the way the pinned runtime's own
/// library spells it — `mistral:7b-instruct-v0.3-q4_K_M` — and that convention
/// carries no release name to compare, so the same check over those entries
/// would be a rule about a runtime's tagging habits rather than about a
/// licence.
fn the_artefact_is_of_the_release_upstream_names(model: &Model) -> bool {
    if model.requantised.is_none() {
        return true;
    }
    let Some((_, artefact)) = model.quantised_at() else {
        // Nothing to disagree with: rule 5's fallback names no file at all.
        return true;
    };
    artefact
        .to_lowercase()
        .contains(&the_release_in(&model.upstream).to_lowercase())
}

/// **Each of the two entries names a third party's upload, and says all three
/// things about it.**
///
/// The digest and the artefact are compared with what those repositories
/// published, which is the whole point of a pin: the file this catalogue means
/// is the file a machine would fetch, or the fetch fails rather than quietly
/// substituting another.
#[test]
fn each_european_entry_names_the_upload_it_means_with_a_pin_behind_it() {
    for upload in &THE_TWO {
        let model = the_entry(upload.id);
        let (quantisation, artefact) = model.quantised_at().unwrap_or_else(|| {
            panic!(
                "`{}` still names no artefact. Rule 6 is the road out of the publisher's own \
                 release, and this task is somebody walking it",
                upload.id
            )
        });
        assert_eq!(quantisation, "Q4_K_M", "`{}`", upload.id);
        assert_eq!(
            artefact, upload.artefact,
            "`{}` names an artefact this test has not checked. The digest below was read off one \
             repository's file list; if the entry means a different upload, the evidence moves \
             with it",
            upload.id
        );

        let borrowed = model.requantised.as_ref().unwrap_or_else(|| {
            panic!(
                "`{}` names a file neither of its publishers published and does not say whose it \
                 is, which is the entry looking first-party — exactly what ADR 0026 exists to \
                 prevent",
                upload.id
            )
        });
        assert_eq!(borrowed.by, upload.by, "`{}`", upload.id);
        assert_eq!(
            borrowed.sha256, upload.sha256,
            "`{}` pins a digest that is not the one its repository publishes for this file",
            upload.id
        );
        // The note is the part a curator who has not opened the file cannot
        // write. Held to two things a reader needs and neither of which comes
        // free: what the upload's own repository says its licence is, and when
        // somebody looked.
        let note = borrowed.note.to_lowercase();
        assert!(
            note.contains("license") || note.contains("licence"),
            "`{}`'s note says nothing about the terms the upload itself carries, which may differ \
             from the model's",
            upload.id
        );
        assert!(
            note.contains("2026-09-11"),
            "`{}`'s note does not say when the upload was read. A pin ages, and the date is how \
             the next curator knows how far",
            upload.id
        );
    }
}

/// **Every figure that moves with the artefact has moved**, because rule 5
/// binds them to the file an entry names.
#[test]
fn the_figures_that_travel_with_the_artefact_are_that_artefacts_own() {
    for upload in &THE_TWO {
        let model = the_entry(upload.id);
        assert_eq!(
            model.download_bytes, upload.bytes,
            "`{}` states a size that is not the size of the file it names",
            upload.id
        );
        assert!(
            model.bytes_per_parameter() < THE_PRECISION_LINE,
            "`{}` states {:.2} bytes per parameter beside a four-bit artefact, which is a \
             full-precision release's figure",
            upload.id,
            model.bytes_per_parameter()
        );
        let weights_gb = model.weights_gb();
        assert!(
            f64::from(model.min_ram_gb) >= weights_gb && f64::from(model.min_vram_gb) >= weights_gb,
            "`{}` asks for {} GB of memory and {} GB of video memory beside {weights_gb:.2} GB of \
             weights",
            upload.id,
            model.min_ram_gb,
            model.min_vram_gb
        );
        assert_eq!(
            model.on_cpu, upload.on_cpu,
            "`{}` describes a processor's experience of weights it no longer carries",
            upload.id
        );
    }
}

/// **The licence was read against the uploader's repository as well as the
/// publisher's**, which for Teuken is the difference between an Apache-2.0
/// entry and one quietly backed by a research-licensed file.
///
/// Asked of every borrowing entry in the catalogue rather than of the two,
/// because the mistake is available to whoever adds the next one.
#[test]
fn no_entry_that_borrows_a_file_names_a_requantisation_of_another_release() {
    let catalogue = Catalogue::built_in().expect("the built-in catalogue loads");
    for model in &catalogue.models {
        assert!(
            the_artefact_is_of_the_release_upstream_names(model),
            "`{}` fetches `{}` and states the licence of `{}`. A publisher who ships one model \
             under two licences ships two releases, and an uploader's repository is named after \
             the one they converted — so this entry may be stating terms that are not the file's",
            model.id,
            model.artefact.as_deref().unwrap_or_default(),
            model.upstream
        );
    }
}

/// **And that check refuses what it exists to refuse**, which is the half a
/// green catalogue cannot show anybody: the Teuken mistake, written out as the
/// entry it would have been.
#[test]
fn a_requantisation_of_the_wrong_release_is_caught() {
    let entry = |artefact: &str, upstream: &str| {
        let catalogue = Catalogue::parse(&format!(
            "[[model]]\nid = \"split\"\nname = \"Split\"\npublisher = \"A Publisher\"\n\
             parameters_b = 7.5\nquantisation = \"Q4_K_M\"\nartefact = \"{artefact}\"\n\
             download_bytes = 5_018_868_512\nmin_vram_gb = 6.5\nmin_ram_gb = 10.0\n\
             on_cpu = \"workable\"\ndrives_verbs = \"not-measured\"\n\
             upstream = \"{upstream}\"\n\
             licence = {{ name = \"Apache-2.0\", spdx = \"Apache-2.0\", \
             commercial_use = \"permitted\" }}\n\n\
             [model.requantised]\nby = \"somebody-else\"\n\
             sha256 = \"{}\"\nnote = \"read 2026-09-11; license: apache-2.0\"\n",
            "0".repeat(64)
        ))
        .expect("the fixture loads; only the agreement between two of its fields is under test");
        catalogue
            .models
            .first()
            .expect("the fixture has one entry")
            .clone()
    };

    let commercial = "https://example.test/publisher/Model-7B-instruct-commercial-v0.4";
    assert!(
        the_artefact_is_of_the_release_upstream_names(&entry(
            "hf.co/somebody-else/Model-7B-instruct-commercial-v0.4-GGUF:Q4_K_M",
            commercial,
        )),
        "a requantisation of the release the entry names was refused, so the refusal below says \
         nothing"
    );
    assert!(
        !the_artefact_is_of_the_release_upstream_names(&entry(
            "hf.co/somebody-else/Model-7B-instruct-research-v0.4-GGUF:Q4_K_M",
            commercial,
        )),
        "an entry stating a commercial licence and fetching a requantisation of the research \
         release was accepted. That is the mistake the popular Teuken uploads would have caused, \
         and no digest check can see it"
    );

    // And a first-party artefact is outside the question: the runtime's library
    // spells one `publisher-model:tag-q4_K_M`, which names no release to
    // compare, and an entry borrowing nothing has nobody else's terms to carry.
    let first_party = Catalogue::parse(
        "[[model]]\nid = \"first-party\"\nname = \"First Party\"\npublisher = \"A Publisher\"\n\
         parameters_b = 7.5\nquantisation = \"Q4_K_M\"\nartefact = \"runtime:tag-q4_K_M\"\n\
         download_bytes = 5_018_868_512\nmin_vram_gb = 6.5\nmin_ram_gb = 10.0\n\
         on_cpu = \"workable\"\ndrives_verbs = \"not-measured\"\n\
         upstream = \"https://example.test/publisher/Model-7B-instruct-commercial-v0.4\"\n\
         licence = { name = \"Apache-2.0\", spdx = \"Apache-2.0\", \
         commercial_use = \"permitted\" }\n",
    )
    .expect("an entry naming its publisher's own artefact loads");
    assert!(the_artefact_is_of_the_release_upstream_names(
        first_party.models.first().expect("one entry")
    ));

    // And the fallback shape is not caught by it: an entry that names no file
    // disagrees with nothing.
    let unnamed = Catalogue::parse(
        "[[model]]\nid = \"unnamed\"\nname = \"Unnamed\"\npublisher = \"A Publisher\"\n\
         parameters_b = 7.5\ndownload_bytes = 14_905_484_192\nmin_vram_gb = 17.0\n\
         min_ram_gb = 21.0\non_cpu = \"slow\"\ndrives_verbs = \"not-measured\"\n\
         upstream = \"https://example.test/publisher/Model-7B-instruct-research-v0.4\"\n\
         licence = { name = \"Apache-2.0\", spdx = \"Apache-2.0\", \
         commercial_use = \"permitted\" }\n",
    )
    .expect("an entry that names no artefact loads; rule 5 is the fallback");
    assert!(the_artefact_is_of_the_release_upstream_names(
        unnamed.models.first().expect("one entry")
    ));
}

/// **No grade was earned here.** An artefact named is not an artefact measured,
/// and nothing on this lane's box can run either of these.
#[test]
fn naming_a_file_earned_no_grade() {
    for upload in &THE_TWO {
        let model = the_entry(upload.id);
        assert_eq!(
            model.drives_verbs,
            Driving::NotMeasured,
            "`{}` claims a grade. `alo-driving` measured that a 7B model at four bits does not \
             run usefully on this box, so a grade appearing beside a newly named file is a grade \
             from somewhere other than a run",
            upload.id
        );
        assert_eq!(model.graded_against(), None, "`{}`", upload.id);
    }
}

/// **The carry-or-fetch table carries both artefacts' sizes**, because naming a
/// file moves the measurement ADR 0025 owes just as correcting one did.
#[test]
fn the_carry_or_fetch_table_carries_both_artefacts() {
    let measurement = reading(THE_MEASUREMENT);
    for upload in &THE_TWO {
        let row = format!(
            "| `{}` | {} | `not-measured` |",
            upload.id,
            with_underscores(upload.bytes)
        );
        assert!(
            measurement.contains(&row),
            "{THE_MEASUREMENT} does not carry `{row}`"
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

/// **The consequence, stated rather than discovered:** a four-bit Teuken is
/// back among the models a machine with 16 GB and no graphics card may choose
/// between, and EuroLLM is not.
///
/// Both halves matter. Teuken returns because it now names a file that really
/// is five gigabytes; EuroLLM stays out because nine billion parameters on a
/// processor is `slow` whatever they are quantised to, and an entry that came
/// back for the wrong reason would be the four-bit-leftover bug wearing a
/// digest.
#[test]
fn the_shorter_list_a_laptop_is_offered_has_teuken_back_in_it_and_not_eurollm() {
    let catalogue = Catalogue::built_in().expect("the built-in catalogue loads");
    let offered: Vec<&str> = catalogue
        .to_choose_from_on_cpu(16.0)
        .iter()
        .map(|model| model.id.as_str())
        .collect();
    assert!(
        offered.contains(&"teuken-7b-instruct"),
        "the entry names a five-gigabyte artefact and is still not offered to a laptop: {offered:?}"
    );
    assert!(
        !offered.contains(&"eurollm-9b-instruct"),
        "nine billion parameters were offered as something a processor handles: {offered:?}"
    );
    assert_eq!(
        offered.len(),
        7,
        "the shorter list is seven; `crates/alo-driving/tests/\
         from_a_prompt_to_what_a_machine_offers.rs` says the same number to a person, and the two \
         are the same count or one of them is stale: {offered:?}"
    );
}
