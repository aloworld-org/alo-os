//! **This crate never makes a `WindowId`. It only ever holds one it was handed.**
//!
//! # The rule, and where it came from
//!
//! `alo_dock::WindowId` and the compositor's own `window_number` are two numberings that had
//! **never met** — measured 2026-09-30 across the tree: `Window::of` had no production caller
//! anywhere, so nothing had ever built one of `alo-dock`'s windows from a real surface. The
//! bridge did not exist, which made that the cheapest moment it would ever be to decide how it
//! gets built.
//!
//! The rule agreed between the three lanes: **a `WindowId` is minted from `window_number` at
//! one named conversion and nowhere else.** That conversion lives in `alo-shell`, because
//! `window_number` is a private module of that crate and cannot be named from here at all —
//! which is the right home for a second reason: **this crate should no more know what a window
//! number is than it knows what a camera is.** It receives an id from whoever performed the
//! gesture and asks where it came from no more than it asks where a zoom came from.
//!
//! # Why the rule needs a check on this side too
//!
//! `WindowId::numbered` is **public**. Anybody holding a `u64` can mint one — including from a
//! number read off something else entirely, a length, an index, a count of previews. A rule
//! that says *from `window_number` and nowhere else* is a convention until something fails.
//!
//! The failure it prevents is specific and would land on the wrong lane: **a `WindowId` minted
//! here from the wrong number restores the wrong window exactly once, and it looks like a bug
//! in the panel.** The panel would be doing precisely what it was told.
//!
//! # The check is complete rather than decorative, and that was measured
//!
//! `WindowId` is `pub struct WindowId(u64)` with a **private** field, `numbered` is its only
//! constructor, and it derives no `Default`. So from outside `alo-dock` there is exactly one
//! way to make one, and forbidding that one forbids all of them. A check that closed one door
//! while another stood open would be the shape this crate has already been caught by twice —
//! a guard against a name nobody could write, and a `Default` the compiler already refused.
//!
//! # Its other half is in `alo-shell` and is not this lane's
//!
//! That `WindowId::numbered` appears exactly once outside tests, in the named conversion. The
//! rule has two halves because it can be broken in two crates, and each half lives where the
//! breaking would happen. Said to that lane rather than assumed of them.
#![expect(
    clippy::expect_used,
    reason = "this test reads this crate's own source and manifest, so a missing file is the \
              test's own mistake and a panic naming it is the failure being reported — the \
              same exemption `alo-dock`'s own test modules take, with its words"
)]

use std::path::{Path, PathBuf};

/// The only way to make a `WindowId`, and therefore the only thing to forbid.
const MINTING_A_WINDOW_ID: &str = "WindowId::numbered";

#[test]
fn no_source_file_in_this_crate_mints_a_window_id() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut minting: Vec<String> = Vec::new();
    for file in every_file(&src) {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for line in text.lines() {
            if only_the_code(line).contains(MINTING_A_WINDOW_ID) {
                minting.push(format!("{}: {}", file.display(), line.trim()));
            }
        }
    }
    assert!(
        minting.is_empty(),
        "this crate mints a WindowId, and it must only ever hold one it was handed.\n\n\
         WindowId and the compositor's window_number are two numberings, bridged at one named \
         conversion in alo-shell and nowhere else. A WindowId made here comes from a number \
         this crate chose — a length, an index, a count — and the window it names is whichever \
         window happens to have that number.\n\n\
         The consequence is not a compile error. It restores the wrong window exactly once, \
         and it looks like a bug in the panel, which will be doing exactly what it was told.\n\n\
         If an id is genuinely needed, it is handed in by whoever performed the gesture — the \
         same road the zoom and the Place take, and for the same reason.\n\n\
         Found:\n{minting:#?}"
    );
}

/// **And the crate takes nothing that could tell it a window number.**
///
/// The rule survives a check on calls only while the number itself is out of reach. `alo-shell`
/// keeps `window_number` private, so this cannot be reached today — and this asserts that the
/// manifest has not grown a dependency that would offer another road to one.
#[test]
fn the_manifest_takes_nothing_that_could_hand_it_a_window_number() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("this crate's own manifest");
    // **Comment lines are dropped, and that is not tidiness.** This check reads the manifest
    // as text rather than as parsed TOML, so a comment inside the dependencies block naming a
    // forbidden crate would fail it — and this crate's manifest explains at length *why* it
    // does not depend on things. Without this, the check would fire on its own reasoning, and
    // the fix somebody reached for would be to delete the reasoning. The same trap the source
    // checks avoid with `only_the_code`, arriving through a different file format.
    let dependencies: String = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap_or_default()
        .split("\n[")
        .next()
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    for reaching in ["alo-shell", "smithay", "wayland-server", "alo-compositing"] {
        assert!(
            !dependencies.contains(reaching),
            "alo-put-aside depends on {reaching}, which could hand it a compositor's own \
             window numbering.\n\n\
             This crate holds ids and never makes them. A dependency that knows what a surface \
             is offers a second road to a WindowId, and the rule that there is one conversion \
             stops being checkable from here."
        );
    }
}

/// A line with its documentation and string literals removed, so that prose naming the
/// constructor in order to forbid it is not read as calling it.
///
/// This file names `WindowId::numbered` several times on purpose, in the sentences explaining
/// why no code here may call it. Without this the check would fail on its own explanation — and
/// the fix somebody reached for would be to delete the explanation.
fn only_the_code(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return String::new();
    }
    let mut code = String::new();
    let mut inside_a_string = false;
    let mut letters = line.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '\\' if inside_a_string => {
                letters.next();
            }
            '"' => inside_a_string = !inside_a_string,
            _ if !inside_a_string => code.push(letter),
            _ => {}
        }
    }
    code
}

/// Every `.rs` under a folder.
fn every_file(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(every_file(&path));
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            found.push(path);
        }
    }
    found
}
