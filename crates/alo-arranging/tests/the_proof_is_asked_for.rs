//! `keep` asks whether the disk is holding the layout, read from the source.
//!
//! **Why this is a source test and not a behavioural one.** `keep` is stronger
//! than a plain write for one reason: it hands `kept_text` a proof, so the
//! rename happens only if the bytes the disk gave back read as the layout that
//! was meant. The only way to observe that at runtime is a filesystem which
//! answers a sync without keeping what it was given, and no unit test can
//! arrange one.
//!
//! So the proof was removed twice on 2026-10-03 to find out what held it.
//! Gutting the comparison failed its own test, as it should. **Making `keep`
//! stop asking for it left all twenty-two tests passing** — a tested function
//! with an unverified caller, which is the shape this repository keeps finding.
//! This is what holds the caller.
//!
//! Comments are taken out before the source is read, because `keeping.rs`
//! documents the very function being guarded: a guard that a doc comment could
//! satisfy would be satisfied by deleting the code and keeping the prose.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

/// The keeping module's source, with every comment line taken out.
fn the_code() -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("keeping.rs");
    let text = std::fs::read_to_string(&path).unwrap();
    text.lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// **`keep` hands the proof to the write**, rather than writing and hoping.
#[test]
fn keep_asks_whether_the_disk_is_holding_the_layout() {
    let code = the_code();

    assert!(
        code.contains("these_bytes_are(arrangement, back)"),
        "keep no longer hands the proof to kept_text, so a write the disk \
         mangled would be reported as kept"
    );
    assert!(
        code.contains("alo_kept::kept_text("),
        "keep no longer goes through the disk discipline it was written to rent"
    );
}

/// **And the proof is still a comparison, not a formality.**
///
/// Narrower than *the function exists*: a body that answered `Ok(())` would
/// keep the name, keep the call site, and hold nothing. Its own test catches
/// that, and this says so here rather than leaving the two guards to be read
/// apart.
#[test]
fn the_proof_compares_what_came_back_with_what_was_meant() {
    let code = the_code();

    assert!(
        code.contains("Ok(read_back) if &read_back == arrangement"),
        "the proof no longer compares the bytes that came back with the layout \
         that was meant; `only_the_layout_that_was_meant_passes_the_proof` is \
         the test that fails when this does"
    );
}
