//! A crate's files, split into the code that can call something and the tests
//! that only exercise it.
//!
//! **The split is the whole difficulty.** Four of the six holes
//! `docs/autonomy/what-nothing-calls.md` records are about text that looks like
//! a caller and is not — a comment, a `use` line, a test — and the fifth, this
//! crate's own, is about test text being dropped instead of moved.

use std::path::{Path, PathBuf};

/// The workspace manifest, whose member list says which crates there are.
pub const THE_WORKSPACE: &str = alo_by_hand::THE_WORKSPACE;

/// One crate's text, already split.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ACratesFiles {
    /// Each product file: where it is, and its code with comments and `use`
    /// lines gone and `#[cfg(test)]` blocks lifted out.
    pub product: Vec<(PathBuf, String)>,
    /// Every line of test text in the crate, from `*_tests.rs` files, from
    /// `tests/`, from `examples/`, and from `#[cfg(test)]` blocks lifted out of
    /// product files.
    ///
    /// **One bag rather than per-file**, because nothing asks *which* test
    /// reaches a function — only whether any does.
    pub tests: String,
}

/// Read a crate's files and split them.
///
/// `at` is the crate's directory, as the workspace manifest names it.
#[must_use]
pub fn a_crates_files(at: &Path) -> ACratesFiles {
    let mut files = ACratesFiles::default();
    for found in rust_files_under(&at.join("src")) {
        let Ok(raw) = std::fs::read_to_string(&found) else {
            continue;
        };
        if is_a_test_file(&found) {
            files.tests.push('\n');
            files.tests.push_str(&without_comments(&raw));
            continue;
        }
        let (code, in_test_blocks) = split_off_test_blocks(&raw);
        // **The lifted text goes to the tests and is not dropped.** This is
        // hole five: dropping it counts a function reached only by its own
        // module's tests in neither place, and reports it as an orphan.
        files.tests.push('\n');
        files.tests.push_str(&without_comments(&in_test_blocks));
        files
            .product
            .push((found, without_use_lines(&without_comments(&code))));
    }
    // A crate's `tests/` directory is tests by definition, and so — for this
    // gate's purpose — is `examples/`.
    //
    // **An example is compiled**: `clippy --all-targets` builds all 49 of them,
    // so a call from one is real code and not prose. It is counted as
    // *exercising* rather than as a caller, the same standing as a test,
    // because an example is a developer's check and not a road a person walks.
    // Reading it as a caller was the second of the three false findings the
    // first sample caught: `Nested::submit_with_approval` and
    // `Nested::pump_sign_in` are reached only from `crates/alo-shell/examples`,
    // and reporting them as reached by nothing was untrue.
    //
    // **So this gate passes a function only an example reaches**, which is a
    // hole and is named here rather than discovered: such a function has no
    // road a person can walk, and `docs/autonomy/what-nothing-calls.md` asks a
    // narrower question than that.
    for directory in ["tests", "examples"] {
        for found in rust_files_under(&at.join(directory)) {
            if let Ok(raw) = std::fs::read_to_string(&found) {
                files.tests.push('\n');
                files.tests.push_str(&without_comments(&raw));
            }
        }
    }
    files
}

/// Every `.rs` file under a directory, depth first, in a stable order.
fn rust_files_under(at: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(at) else {
        return found;
    };
    let mut here: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    here.sort();
    for path in here {
        if path.is_dir() {
            found.extend(rust_files_under(&path));
        } else if path.extension().is_some_and(|it| it == "rs") {
            found.push(path);
        }
    }
    found
}

/// Whether a file under `src` is a test file rather than a product one.
///
/// **`*_tests.rs` by name**, which is this workspace's convention for a test
/// module kept beside the file it tests — `docs/autonomy/what-nothing-calls.md`
/// hole two is that these were counted as callers, which flattered every crate.
fn is_a_test_file(at: &Path) -> bool {
    at.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with("_tests.rs") || name.ends_with("_testing.rs"))
}

/// Comments gone, so prose naming a function is not a caller.
///
/// A doc comment above a function almost always names it — the prose and the
/// thing are adjacent in this repository by habit — so counting comments makes
/// every documented function look reached.
fn without_comments(code: &str) -> String {
    let no_blocks = strip_block_comments(code);
    no_blocks
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                return String::new();
            }
            line.find("//")
                .map_or_else(|| line.to_owned(), |at| line[..at].to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `/* … */` gone, including nested, which Rust allows.
fn strip_block_comments(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut depth = 0_usize;
    let mut rest = code.char_indices().peekable();
    while let Some((at, this)) = rest.next() {
        let two = code.get(at..at.saturating_add(2));
        if two == Some("/*") {
            depth = depth.saturating_add(1);
            rest.next();
            continue;
        }
        if two == Some("*/") && depth > 0 {
            depth = depth.saturating_sub(1);
            rest.next();
            continue;
        }
        if depth == 0 {
            out.push(this);
        }
    }
    out
}

/// `use` lines gone, because a re-export is a declaration and not a call.
///
/// Hole three: nearly every crate re-exports its own surface in `lib.rs`, so
/// counting `use` lines flattered every crate at once.
fn without_use_lines(code: &str) -> String {
    code.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("use ")
                || trimmed.starts_with("pub use ")
                || trimmed.starts_with("pub(crate) use ")
                || trimmed.starts_with("pub(super) use "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Split a product file into its code and the text of its `#[cfg(test)]`
/// blocks, by matching braces.
///
/// Returns `(code, in_test_blocks)`. Both are kept: see this module's header
/// and hole five.
fn split_off_test_blocks(code: &str) -> (String, String) {
    let mut kept = String::with_capacity(code.len());
    let mut tests = String::new();
    let mut from = 0_usize;
    while let Some(found) = code.get(from..).and_then(|rest| rest.find("#[cfg(test)]")) {
        let at = from.saturating_add(found);
        kept.push_str(code.get(from..at).unwrap_or_default());
        // The block's body starts at the next `{` after the attribute — but
        // only if the attributed item **has** one.
        //
        // **A `#[cfg(test)] use std::fmt;` has none**, and the next `{` in the
        // file belongs to whatever comes after it. The first version of this
        // took that brace and lifted an ordinary function's whole body into the
        // tests, so every call inside it stopped counting as a caller. Caught
        // by `a_cfg_test_with_no_block_keeps_the_rest_of_the_file`, which is the
        // only reason this comment exists.
        //
        // So: whichever comes first, a `;` or a `{`. A semicolon means the
        // attributed item ended without a body and there is nothing to lift.
        let after = at.saturating_add("#[cfg(test)]".len());
        let rest = code.get(after..).unwrap_or_default();
        let ends = rest.find(';');
        let opens = rest.find('{');
        let open = match (ends, opens) {
            (Some(ends), Some(opens)) if ends < opens => {
                kept.push_str(code.get(after..).unwrap_or_default());
                return (kept, tests);
            }
            (_, Some(opens)) => opens,
            (_, None) => {
                kept.push_str(code.get(after..).unwrap_or_default());
                return (kept, tests);
            }
        };
        let open = after.saturating_add(open);
        let end = closing_brace(code, open);
        tests.push('\n');
        tests.push_str(code.get(open..end).unwrap_or_default());
        from = end;
    }
    kept.push_str(code.get(from..).unwrap_or_default());
    (kept, tests)
}

/// Where the brace opened at `open` closes, or the end of the text.
pub(crate) fn closing_brace(code: &str, open: usize) -> usize {
    let mut depth = 0_usize;
    for (at, this) in code.char_indices().skip_while(|(at, _)| *at < open) {
        match this {
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return at.saturating_add(1);
                }
            }
            _ => {}
        }
    }
    code.len()
}

#[cfg(test)]
#[path = "the_code_tests.rs"]
mod tests;
