//! **The camera is stored in exactly one place, read from the source.**
//!
//! Until 2026-10-04 there were two: a field on `Server` and a copy on
//! `Surfaces::popups`, kept in step by **seven** assignments — six mutators across
//! `canvas_camera`, `canvas_show_all` and `canvas_the_world`, and one in
//! `Server::render_frame` which re-assigned the copy once a frame. That last one is
//! why nothing was ever stale, so the pattern was a working design rather than a
//! bug, and a test asserting *the two agree* would have passed every day.
//!
//! **What a runtime test cannot show is the shape.** Canvas task 9 makes a camera
//! **per viewport**, and *the* camera syncing into *the* popups has no meaning with
//! two displays at their own zoom. The thing that had to go is the second home, and
//! the only way to hold that is to read the source and count.
//!
//! # Why this is a source-level guard and not a behaviour test
//!
//! The same reason `the_recheck_has_a_caller.rs` is one. A second copy introduced
//! tomorrow, assigned correctly at every site, **behaves identically** — that is the
//! whole trouble with it. It costs nothing until somebody adds the mutator that
//! forgets, or until two viewports make *which copy* a real question. No assertion
//! about a running compositor can see it coming; counting the declarations can.
#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "an unreadable source file or an unexpected None here is the failure this test \
              reports, and a formatted panic names the file it was reading"
)]

use std::path::Path;

/// This crate's source directory.
fn src() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every `.rs` file of this crate, as (name, text).
///
/// Test modules are included rather than skipped: a fixture that keeps its own
/// camera beside the real one is the same fault wearing a test's name, and the
/// `_tests.rs` files here are compiled into the crate.
fn every_file() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let reading = std::fs::read_dir(src()).expect("this crate has a src directory");
    for entry in reading {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().is_some_and(|it| it == "rs") {
            let name = path
                .file_name()
                .expect("a file with an extension has a name")
                .to_string_lossy()
                .into_owned();
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|why| panic!("{name} must be readable: {why}"));
            found.push((name, text));
        }
    }
    assert!(
        found.len() > 50,
        "read {} source files, which is too few to be this crate — the directory walk is wrong \
         and every count below would be vacuously small",
        found.len()
    );
    found
}

/// Which types in this file declare a `camera` **field**, by struct name.
///
/// **A field, never a parameter**, and that distinction is the whole of why this
/// function exists. The first version of this test matched the text
/// `camera: alo_canvas::Camera,` anywhere and reported ten files — because that is
/// also exactly how an **argument** is written, and this change had just made the
/// camera an argument in seven places. A guard whose first run names the thing the
/// change deliberately introduced is measuring the wrong noun.
///
/// So: a struct body is `…struct Name {` to the next `}` at column zero, and a
/// field is a line inside it at field indentation. The crate is `cargo fmt` clean,
/// which is what makes a column rule sound here rather than hopeful.
fn structs_with_a_camera_field(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut inside: Option<String> = None;
    for line in text.lines() {
        if let Some(name) = &inside {
            if line == "}" {
                inside = None;
                continue;
            }
            // A field, at one level of indentation, not a nested item or a comment.
            let field = line.strip_prefix("    ").is_some_and(|it| {
                !it.starts_with(' ') && !it.starts_with("//") && !it.starts_with('#')
            });
            if field && line.contains("camera: ") {
                found.push(name.clone());
            }
            continue;
        }
        if let Some((before, _)) = line.split_once(" {") {
            if let Some(name) = before.rsplit_once("struct ").map(|(_, name)| name) {
                // `Target<R, D: ScanoutDevice>` and `OnThePlane<'a>` are `Target` and
                // `OnThePlane`. Generics are part of how a type is written, never part
                // of which type it is, and a list keyed on the spelling would need
                // editing the day somebody added a parameter.
                let bare = name.split_once('<').map_or(name, |(bare, _)| bare);
                inside = Some(bare.trim().to_owned());
            }
        }
    }
    found
}

/// **Four types in this crate hold a camera, and exactly one of them is state.**
///
/// `Surfaces` is the one home. The other three are not session state and could not
/// drift from it:
///
/// - `Target` and `Nested` are the backends. `FrameTarget::look_at` hands each the
///   camera to draw **this frame** with — the seam that exists so a backend cannot
///   draw a zoom nobody assigned it.
/// - `OnThePlane` is a borrow bundle built per paint, `Copy`, holding the frames and
///   the camera together for the reason its own note gives: *separating them invites
///   a caller to pass windows with somebody else's camera.*
///
/// They are **named** rather than matched loosely, so a fifth holder is a failing
/// test and a decision rather than a silent second home. `OnThePlane` was found by
/// this test rather than known in advance, which is the case for naming them.
#[test]
fn only_one_type_in_this_crate_stores_the_camera_as_state() {
    /// Which types may hold a camera field, and what for.
    const ALLOWED: [(&str, &str); 4] = [
        ("Surfaces", "the one home"),
        (
            "Target",
            "a backend's per-frame input, via FrameTarget::look_at",
        ),
        ("Nested", "the same, for the nested backend"),
        (
            "OnThePlane",
            "a borrow bundle built per paint, never stored",
        ),
    ];

    let mut declared: Vec<(String, String)> = Vec::new();
    for (name, text) in every_file() {
        for holder in structs_with_a_camera_field(&text) {
            declared.push((holder, name.clone()));
        }
    }

    for (holder, file) in &declared {
        let allowed = ALLOWED.iter().find(|(named, _)| named == holder);
        assert!(
            allowed.is_some(),
            "`{holder}` in {file} holds a camera field and is not one of the types allowed to. \
             A second home is what canvas task 9 cannot be built over: with two displays at \
             their own zoom there is no *the* camera to copy into the other. If this is a new \
             backend's per-frame input, add it to ALLOWED with what it is for; if it is session \
             state, it belongs on `Surfaces` beside the Place and the level. Declared by: \
             {declared:?}"
        );
    }

    assert_eq!(
        declared.len(),
        ALLOWED.len(),
        "{} type(s) hold a camera field and {} are expected. **Fewer is the dangerous \
         direction**: it means the scan stopped finding them, and this test would then pass \
         however many copies came back. Declared by: {declared:?}",
        declared.len(),
        ALLOWED.len()
    );
}

/// **Nothing assigns a camera into the popups any more.**
///
/// The exact text of the seven assignments that used to exist. Held as its own
/// check because the field could come back under a different name and still be the
/// same mistake — and because this is the one of the two that names what went
/// wrong rather than what is allowed.
#[test]
fn nothing_keeps_a_second_camera_in_step_by_hand() {
    /// The two files whose `self.camera = camera;` is a backend being told what to
    /// draw this frame with, not state being kept in step.
    ///
    /// Named by file because that is what distinguishes them: both are the body of
    /// `FrameTarget::look_at`, which exists so the backend cannot draw a zoom
    /// nobody assigned. The previous version of this test flagged them, and they
    /// are the one case where assigning a camera outside its home is correct.
    const THE_BACKENDS: [&str; 2] = ["direct_target.rs", "nested.rs"];

    let mut found: Vec<String> = Vec::new();
    for (name, text) in every_file() {
        if THE_BACKENDS.contains(&name.as_str()) {
            continue;
        }
        for line in text.lines() {
            let said = line.trim();
            // An assignment, not a read: `.camera = ` with a path on the left.
            let assigns = said.starts_with("self.") && said.contains(".camera = ");
            // The one home is allowed to be assigned. Anything else is a copy.
            if assigns && !said.starts_with("self.surfaces.camera = ") {
                found.push(format!("{name}: {said}"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "{} assignment(s) keep a camera somewhere other than its one home. Seven of these \
         existed until 2026-10-04 and none of them was wrong — `Server::render_frame` \
         re-assigned the copy once a frame, so a forgotten mutator was harmless. That is \
         exactly why a behaviour test never caught it, and why this one reads the source: \
         {found:?}",
        found.len()
    );
}
