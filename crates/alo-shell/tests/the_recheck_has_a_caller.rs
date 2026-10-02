//! **The recovery recheck is called by the draw**, read from the source this
//! crate ships because nothing can run that draw.
//!
//! This repository's recurring fault is a rule that is written, tested and never
//! asked. `crate::canvas_never_lost` held *a frame keeps a usable part of its name
//! outside every fixed control* for weeks with unit tests, integration tests and
//! **no caller on any path a person could take**. Its detector and its mover —
//! `frames_the_controls_now_hide` and `bring_back_frames_the_controls_hide` — then
//! repeated it one layer up: both tested, both called by tests alone, so on a
//! running machine the Dock could grow over a frame and nothing noticed.
//!
//! # Why this is read rather than run
//!
//! The caller is `direct_desktop`'s `desktop`, and it **opens a graphics card**:
//! `with_active_device`, `discover_atomic_output`, an atomic commit. There is no
//! fixture for it and there is not going to be one — `direct_desktop_tests` builds
//! its pictures by calling `frame_pictures` directly for exactly this reason, and
//! the integration fixtures reach `Server::render`, which is a different path and
//! never touches a desktop frame. So every test that can run drives the recheck by
//! handing the bounds over itself, **as a draw would**, and not one of them would
//! fail if the draw stopped calling it.
//!
//! That is the hole this file closes, and it is the only tool that fits: the
//! question *is this symbol called from production* is answerable by reading, and
//! answerable by nothing else here.
//!
//! # The test proves its own reading
//!
//! An assertion that a name appears in a file passes just as well when the name
//! appears **in a comment**, and this file's subject is a comment-heavy one — the
//! call site carries a paragraph that names the function it calls. An empty or
//! careless search would therefore report success about prose. So the stripping is
//! measured rather than trusted: the name is counted before and after comments are
//! removed, and **the count must fall**. If it does not, the stripper did nothing
//! and every other assertion here is about raw text.
#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use std::path::{Path, PathBuf};

/// The draw that lays the fixed controls out and must act on them moving.
const THE_DRAW: &str = "direct_desktop.rs";

/// Where the draw records the controls it laid out.
const RECORDS_THEM: &str = "the_fixed_controls_were_drawn";

/// What it must call once they have moved.
const ACTS_ON_THEM: &str = "bring_back_frames_the_moved_controls_hide";

/// This crate's source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// One file with its comment lines and trailing comments taken out.
///
/// Line comments only, which is all this file needs and all it should claim:
/// `direct_desktop.rs` has no block comments and no string literal containing
/// either symbol. A general Rust stripper here would be a second, untested
/// implementation of something the assertions below check directly.
fn the_code_of(written: &str) -> String {
    written
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| match line.split_once("//") {
            Some((before, _)) => before,
            None => line,
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// How many times `name` appears in `text`.
fn how_often(text: &str, name: &str) -> usize {
    text.matches(name).count()
}

/// **The draw acts on the controls having moved, in code rather than in a
/// comment.**
#[test]
fn the_draw_brings_back_frames_the_moved_controls_hide() {
    let at = src().join(THE_DRAW);
    // `expect` rather than a formatted `panic!`: the workspace denies
    // `clippy::panic` everywhere, tests included, and the path is a constant this
    // file already names.
    let written = std::fs::read_to_string(&at)
        .expect("direct_desktop.rs is this crate's desktop draw and must be readable");
    let code = the_code_of(&written);

    // The anchor first: a search that cannot find what is certainly there is
    // measuring its own mistake rather than the file. If the draw has stopped
    // recording the controls at all then this file is asking the wrong question
    // and must say so, rather than reporting the absence of the other symbol.
    assert!(
        how_often(&code, RECORDS_THEM) > 0,
        "{THE_DRAW} does not call {RECORDS_THEM}, so either the draw no longer \
         records where the fixed controls are — which breaks the rule outright — \
         or this test is reading the wrong file"
    );

    // The stripper is measured, not trusted. The call site names the function it
    // calls in its own comment, so a stripper that did nothing would make the
    // assertion below pass on prose.
    let in_prose_too = how_often(&written, ACTS_ON_THEM);
    let in_code = how_often(&code, ACTS_ON_THEM);
    assert!(
        in_code < in_prose_too,
        "comments were not removed: {ACTS_ON_THEM} appears {in_prose_too} times in \
         {THE_DRAW} and {in_code} times after stripping, so this test cannot tell \
         a call from a sentence about one"
    );

    assert!(
        in_code > 0,
        "{THE_DRAW} never calls {ACTS_ON_THEM}, so the controls are recorded on \
         every frame and a frame they have moved over is never brought back. The \
         detector and the mover are both written and both tested; without this \
         call they are reachable from tests alone, which is the fault this whole \
         module exists because of"
    );
}
