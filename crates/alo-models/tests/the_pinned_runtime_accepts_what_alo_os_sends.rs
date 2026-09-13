//! **Whether the pinned runtime accepts what alo OS sends it**, asked of the
//! runtime rather than of a socket this repository wrote.
//!
//! Three reports ended on the same sentence — *what no test here shows is that
//! the pinned runtime accepts this exact request*. The three requests are the
//! ones `alo-models` makes of Ollama: a brought file handed to it, a question
//! put by the id that taught it, and the listing read back. On 2026-09-13 they
//! were put to Ollama 0.34.0 on an Apple M3 with 8 GB, and the first was
//! refused — `400 {"error":"neither 'from' or 'files' was specified"}` — which
//! is why `bring` now sends the two requests `src/handing_over.rs` describes.
//!
//! # Two halves, and only one needs a runtime
//!
//! [`the_image_pins_the_runtime_these_requests_were_measured_against`] reads
//! this repository and runs everywhere: the release [`THE_PINNED_RUNTIME`]
//! names is the release the image installs, so a newer runtime cannot arrive
//! without somebody walking this road again.
//!
//! [`the_three_requests_put_to_the_real_runtime`] is `#[ignore]`d, for the reason
//! `alo-driving`'s measurement is: it needs the runtime serving on this machine
//! and a weights file on this disk, and asserting against nothing would be
//! green worth nothing. Run it with:
//!
//! ```text
//! ALO_BROUGHT_FILE=/absolute/path/to/weights.gguf \
//!   cargo test -p alo-models --test the_pinned_runtime_accepts_what_alo_os_sends \
//!   -- --ignored --nocapture
//! ```
//!
//! It refuses a runtime of any other release rather than measuring it, and it
//! removes the model it created afterwards, leaving the runtime's store as it
//! found it apart from the blob — which the runtime keeps, as it keeps any.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_models::{Catalogue, ModelRuntime, Ollama, THE_PINNED_RUNTIME, Weights};

/// Where the runtime listens on this machine.
const ON_THIS_MACHINE: &str = "http://127.0.0.1:11434";

/// The weights file to bring. Without it there is nothing to walk.
const WHICH_FILE: &str = "ALO_BROUGHT_FILE";

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// **The release these requests were measured against is the one the image
/// installs.**
#[test]
fn the_image_pins_the_runtime_these_requests_were_measured_against() {
    let recipe = std::fs::read_to_string(the_repository().join("image").join("Containerfile"))
        .expect("the image's recipe is where the repository keeps it");
    let pinned: Vec<&str> = recipe
        .lines()
        .filter_map(|line| line.trim().strip_prefix("ARG THE_RUNTIME="))
        .collect();
    assert_eq!(
        pinned,
        vec![THE_PINNED_RUNTIME],
        "the image installs a runtime other than the one alo-models' requests were put to. Walk \
         `the_three_requests_put_to_the_real_runtime` against it, fix what it refuses, and only \
         then move THE_PINNED_RUNTIME"
    );
}

/// **The three requests, put to the runtime on this machine.**
#[test]
#[ignore = "needs the pinned runtime serving on this machine — run it with ALO_BROUGHT_FILE set"]
fn the_three_requests_put_to_the_real_runtime() {
    let file = std::env::var(WHICH_FILE)
        .map(PathBuf::from)
        .expect("set ALO_BROUGHT_FILE to an absolute path of a weights file on this disk");

    let version: serde_json::Value = serde_json::from_str(
        &ureq::get(format!("{ON_THIS_MACHINE}/api/version"))
            .call()
            .expect("the runtime answers on this machine")
            .into_body()
            .read_to_string()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        version.get("version").and_then(serde_json::Value::as_str),
        Some(THE_PINNED_RUNTIME),
        "this is not the runtime the image pins, so what it accepts says nothing about alo OS"
    );

    let runtime = Ollama::at(ON_THIS_MACHINE, Catalogue::built_in().unwrap());
    let weights = Weights::at(&file).unwrap();

    // 1. The brought file, handed over.
    runtime
        .bring(&weights)
        .expect("the pinned runtime accepts a brought file");
    println!("brought {} as {}", file.display(), weights.id);

    // 2. A question, by the id the door taught it.
    let said = runtime
        .answers("Answer with the single word: ready.", &weights.id)
        .expect("the pinned runtime answers by the id it was taught");
    println!("answered: {said:?}");
    assert!(!said.trim().is_empty());

    // 3. The listing, read back.
    let installed = runtime.installed().expect("the listing parses");
    let listed = installed
        .iter()
        .find(|entry| entry.id == weights.id)
        .expect("the brought file is in the listing under the id it was taught");
    println!("listed: {listed:?}");
    assert!(listed.bytes_on_disk > 0);

    runtime.unload(&weights.id).expect("the model is let go of");
    runtime
        .remove(&weights.id)
        .expect("the probe's model is not left behind");
    assert!(
        runtime
            .installed()
            .unwrap()
            .iter()
            .all(|entry| entry.id != weights.id),
        "the probe's model is still listed"
    );
}
