//! **The file a catalogue entry pins, against the file the machine actually
//! got** — asked of the real runtime, and asked of the failing case first.
//!
//! Task 16 of `docs/autonomy/v0-01-lane-b-plan.md`. Task 15 wrote two `sha256`
//! figures into `data/catalogue.toml` and nothing compared either of them with
//! anything, so the sentence rule 6 sells — *the file we graded and the file a
//! machine fetches are the same file or the fetch fails* — was a promise about
//! a field nobody read. The case it exists for is **a tag re-pointed at a
//! different upload**, which until now passed in silence.
//!
//! # Why the failing case is the one that matters
//!
//! A digest check is easy to write and easy to write wrongly, and every wrong
//! version passes the happy case. A check that compares nothing, a check that
//! compares a value with itself, a check whose mismatch branch warns and
//! carries on — all three are green against a file that matches. **So the
//! refusal is measured against the real runtime**, with a real artefact whose
//! real digest is deliberately not the one pinned, and the digest the refusal
//! reports is asserted to be the one the machine really holds.
//!
//! # The two halves
//!
//! The reader that pulls a digest out of what the runtime answers is held to
//! refusing every shape that is not one, in `src/ollama.rs`'s own tests — it is
//! private to that file, and a test is not a reason to widen a crate's public
//! surface. That half runs everywhere.
//!
//! These are `#[ignore]`d for the reason
//! `the_pinned_runtime_accepts_what_alo_os_sends`'s measurement is: they need
//! the runtime serving on this machine, and asserting against nothing would be
//! green worth nothing. Run them with:
//!
//! ```text
//! cargo test -p alo-models --test the_pin_an_entry_states -- --ignored --nocapture
//! ```
//!
//! They refuse a runtime of any other release rather than measuring it, and
//! they put the store back as they found it apart from the blob, which the
//! runtime keeps as it keeps any.
//!
//! # The artefact they use, and why it is not a catalogue entry
//!
//! `hf.co/bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M`, **105 MB**. The two
//! entries that state a pin are five and a half gigabytes each, and a check
//! that downloads five gigabytes to run is not a check anybody runs. What is
//! being measured here is the *mechanism* — that what the runtime reports for a
//! model pulled from `hf.co` is the `sha256` of the GGUF, and that a mismatch
//! refuses — and the mechanism is the same artefact shape either way.
//!
//! That the mechanism carries to the real entries was measured separately and
//! costs nothing: both `eurollm-9b-instruct` and `teuken-7b-instruct` were
//! checked against their registry manifests on 2026-09-20 and the
//! `application/vnd.ollama.image.model` layer digest equals the catalogue's
//! `sha256` in both. `docs/quirks.md` carries it.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::{Catalogue, ModelRuntime, Ollama, Progress, RuntimeError, THE_PINNED_RUNTIME};

/// Where the runtime listens on this machine.
const ON_THIS_MACHINE: &str = "http://127.0.0.1:11434";

/// A small real artefact on the registry the two pinned entries come from.
const A_SMALL_ARTEFACT: &str = "hf.co/bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M";

/// Its real digest, measured 2026-09-20 three ways that agreed: the registry
/// manifest's model layer, the manifest the runtime wrote on this disk, and
/// `sha256sum` of the blob itself.
const ITS_REAL_DIGEST: &str = "2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d";

/// A digest of the right shape that is not that file's.
const NOT_ITS_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// The id each test gives it, one apiece.
///
/// **Not one id shared between them.** These tests run in parallel against one
/// runtime store, and a model is named in that store by its catalogue id — so a
/// shared id means one test removing what another is asserting is installed,
/// which is what happened the first time they were run together. The id is the
/// shared state, so the id is what stops being shared.
const WHEN_IT_MATCHES: &str = "a-small-model-that-matches";

/// The id the mismatching test uses.
const WHEN_IT_DOES_NOT: &str = "a-small-model-that-does-not";

/// The id the unpinned test uses.
const WHEN_THERE_IS_NO_PIN: &str = "a-small-model-with-no-pin";

/// A catalogue of one entry, naming that artefact and pinning `sha256`.
fn pinning(id: &str, sha256: &str) -> Catalogue {
    let text = format!(
        "[[model]]\n\
         id = \"{id}\"\n\
         name = \"A Small Model\"\n\
         publisher = \"HuggingFaceTB\"\n\
         parameters_b = 0.135\n\
         quantisation = \"Q4_K_M\"\n\
         artefact = \"{A_SMALL_ARTEFACT}\"\n\
         download_bytes = 105454432\n\
         min_vram_gb = 1.0\n\
         min_ram_gb = 1.0\n\
         on_cpu = \"workable\"\n\
         drives_verbs = \"not-measured\"\n\
         upstream = \"https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct\"\n\
         licence = {{ name = \"Apache-2.0\", spdx = \"Apache-2.0\", \
         commercial_use = \"permitted\" }}\n\
         [model.requantised]\n\
         by = \"bartowski\"\n\
         sha256 = \"{sha256}\"\n\
         note = \"\"\"A small artefact used by this crate's own measurement of the pin, \
         and not an entry alo OS offers.\"\"\"\n"
    );
    Catalogue::parse(&text).expect("a catalogue of one entry")
}

/// The runtime on this machine, refused unless it is the pinned release.
fn the_real_runtime() -> String {
    let version: serde_json::Value = ureq::get(format!("{ON_THIS_MACHINE}/api/version"))
        .call()
        .expect("the model runtime is serving on this machine")
        .into_body()
        .read_json()
        .expect("it answers /api/version");
    let running = version
        .get("version")
        .and_then(serde_json::Value::as_str)
        .expect("it says which release it is")
        .to_owned();
    assert_eq!(
        running, THE_PINNED_RUNTIME,
        "this measurement is about the pinned release and this machine runs another; \
         a later runtime is a deliberate change, walked against the real program"
    );
    running
}

/// Nothing to report while it downloads.
fn quietly() -> impl alo_models::ProgressSink {
    struct Quiet;
    impl alo_models::ProgressSink for Quiet {
        fn advanced(&mut self, _: Progress) {}
    }
    Quiet
}

/// **A fetch whose pin does not match the file ends in a refusal that names
/// both digests** — measured against the real runtime and a real artefact.
///
/// This is the test the task asks for first. The entry pins a digest of the
/// right shape that is not this file's; the fetch must fail, and the digest it
/// reports as *arrived* must be the one the machine really holds, because a
/// refusal that reported the expected value back would pass while comparing
/// nothing.
#[test]
#[ignore = "needs the pinned model runtime serving on this machine"]
fn a_file_that_is_not_the_one_pinned_is_refused_and_both_digests_are_named() {
    let running = the_real_runtime();
    let ollama = Ollama::at(ON_THIS_MACHINE, pinning(WHEN_IT_DOES_NOT, NOT_ITS_DIGEST));

    let refused = ollama
        .fetch(WHEN_IT_DOES_NOT, &mut quietly())
        .expect_err("a file that is not the one pinned is not installed");

    match refused {
        RuntimeError::NotThePinnedFile {
            model,
            expected,
            arrived,
        } => {
            assert_eq!(model, WHEN_IT_DOES_NOT);
            assert_eq!(expected, NOT_ITS_DIGEST, "it reported what it was promised");
            assert_eq!(
                arrived, ITS_REAL_DIGEST,
                "it reported the digest of the file this machine actually holds, \
                 which is the half a check that compares nothing cannot produce"
            );
            assert_ne!(expected, arrived);
        }
        other => panic!("a mismatched pin was refused for another reason: {other:?}"),
    }

    // And it never named it for the catalogue: a refusal that installed it
    // anyway would be a warning wearing a refusal's type.
    let installed = ollama.installed().expect("the runtime lists what it has");
    assert!(
        !installed.iter().any(|one| one.id == WHEN_IT_DOES_NOT),
        "the model was installed under its catalogue id despite the refusal"
    );

    println!("refused on {running}: expected {NOT_ITS_DIGEST}, arrived {ITS_REAL_DIGEST}");
}

/// **A fetch whose pin matches ends with the machine holding that file.**
///
/// The other half, and the weaker one: it is here so that the refusal above is
/// known to be a refusal of the mismatch rather than of everything.
#[test]
#[ignore = "needs the pinned model runtime serving on this machine"]
fn a_file_that_is_the_one_pinned_is_installed() {
    let _running = the_real_runtime();
    let ollama = Ollama::at(ON_THIS_MACHINE, pinning(WHEN_IT_MATCHES, ITS_REAL_DIGEST));

    ollama
        .fetch(WHEN_IT_MATCHES, &mut quietly())
        .expect("the file the entry pins is the file this machine has");

    let installed = ollama.installed().expect("the runtime lists what it has");
    assert!(
        installed.iter().any(|one| one.id == WHEN_IT_MATCHES),
        "the fetch reported success and the runtime does not have it"
    );

    // Put the store back: the blob stays, as the runtime keeps any.
    ollama
        .remove(WHEN_IT_MATCHES)
        .expect("the test's own model");
}

/// **An entry that states no pin is not newly able to fail**, and asks the
/// runtime nothing extra.
///
/// The acceptance asks for this in as many words. It is measured by fetching an
/// entry with no `[model.requantised]` block and finding it installed — on a
/// runtime where, if the check ran, the pin would be absent and the fetch would
/// have to refuse.
#[test]
#[ignore = "needs the pinned model runtime serving on this machine"]
fn an_entry_with_no_pin_is_unaffected() {
    let _running = the_real_runtime();
    let text = format!(
        "[[model]]\n\
         id = \"{WHEN_THERE_IS_NO_PIN}\"\n\
         name = \"A Small Model\"\n\
         publisher = \"HuggingFaceTB\"\n\
         parameters_b = 0.135\n\
         quantisation = \"Q4_K_M\"\n\
         artefact = \"{A_SMALL_ARTEFACT}\"\n\
         download_bytes = 105454432\n\
         min_vram_gb = 1.0\n\
         min_ram_gb = 1.0\n\
         on_cpu = \"workable\"\n\
         drives_verbs = \"not-measured\"\n\
         upstream = \"https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct\"\n\
         licence = {{ name = \"Apache-2.0\", spdx = \"Apache-2.0\", \
         commercial_use = \"permitted\" }}\n"
    );
    let ollama = Ollama::at(
        ON_THIS_MACHINE,
        Catalogue::parse(&text).expect("a catalogue of one unpinned entry"),
    );

    ollama
        .fetch(WHEN_THERE_IS_NO_PIN, &mut quietly())
        .expect("an entry that states no pin is fetched as it always was");
    ollama
        .remove(WHEN_THERE_IS_NO_PIN)
        .expect("the test's own model");
}
