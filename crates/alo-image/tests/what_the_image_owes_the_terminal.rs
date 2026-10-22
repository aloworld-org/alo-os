//! **What alo's own terminal needs from the machine underneath it.**
//!
//! `alo_terminal`'s `WHAT_THIS_TERMINAL_IS_CALLED` tells every shell it starts
//! that it is on an `xterm-256color`. That is a claim about the machine as much
//! as about the terminal: a program reads the name, looks it up in a terminfo
//! database, and refuses to run when the lookup fails. `clear` answers *TERM
//! environment variable not set.* when there is no name, and something equally
//! unhelpful when there is a name nothing can resolve.
//!
//! So the name is only as good as the image carrying the database, and nothing
//! in `alo-terminal` can check that — a terminfo database is a property of an
//! installed system. This is where it can be checked.
//!
//! # What was measured, and what this test actually holds
//!
//! **Measured once, by hand, on 2026-10-17**: the pinned base was pulled and
//! asked, and it carries `/usr/share/terminfo/x/xterm-256color` and
//! `/usr/sbin/clear`.
//!
//! ```text
//! podman run --rm quay.io/fedora/fedora-bootc:42@sha256:077182b6… \
//!     ls /usr/share/terminfo/x/xterm-256color
//! ```
//!
//! **That measurement does not run again, and this test is not pretending it
//! does.** Pulling a base image needs a network and a container runtime, which
//! a gate has neither of reliably. What this holds instead is the one thing
//! that would make the measurement stale: **the base is pinned by digest**, so
//! it cannot change under the measurement without this line changing. When
//! somebody bumps the digest, this fails and says to measure again.
//!
//! *A measurement performed once and a check that runs for ever are different
//! things, and `CLAUDE.md` asks which one a report is. This is the first, with
//! the second built around it to say when the first expired.*

#![expect(
    clippy::panic,
    clippy::expect_used,
    reason = "in a test, a panic naming the file and the line is the failure being reported"
)]

use std::path::Path;

/// The base the terminfo database was measured on.
///
/// Changing the base means the measurement in this file's header is about
/// another image. Re-run the command there against the new one and put its
/// digest here.
const THE_BASE_THAT_WAS_MEASURED: &str = "quay.io/fedora/fedora-bootc:42@sha256:077182b6ba853b3348d0bede602ac30b9e6568c6422bcf0654de5af96f19b9c3";

/// What the recipe calls the base.
const THE_ARGUMENT: &str = "ARG THE_BASE=";

/// Where the shell integration a person pastes into their own shell lives.
const THE_INTEGRATION: &str = "crates/alo-terminal/shell-integration";

/// Every `alo-…` word in those files that is **not** a command.
///
/// Two, and each is named rather than matched by a pattern, because a pattern
/// loose enough to exclude them would be loose enough to exclude a real command
/// somebody forgot to ship — which is the whole fault this is about.
const NOT_COMMANDS: [&str; 2] = [
    // The OSC 133 mark `alo.bash` prints to say what was typed. A sequence on a
    // stream, named in `docs/contracts/shell-integration-marks.md`, never run.
    "alo-typed",
    // The file itself, named in its own comments.
    "alo-before.bash",
];

/// The recipe this repository builds its image from.
fn the_recipe() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("image")
        .join("Containerfile");
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} did not read: {why}", at.display()))
}

/// **The base is still the one the terminfo database was measured on.**
///
/// Not a check that terminfo is there — see this file's header for why it
/// cannot be, and for the measurement that found it. This is the expiry date on
/// that measurement.
#[test]
fn the_base_is_the_one_the_terminal_was_measured_against() {
    let recipe = the_recipe();
    let named = recipe
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(THE_ARGUMENT))
        .expect("the recipe names a base for the image");

    assert_eq!(
        named, THE_BASE_THAT_WAS_MEASURED,
        "the image's base has changed since anybody asked it whether it carries a terminfo \
         database. alo's own terminal tells every shell it starts that it is on an \
         `xterm-256color`, and a name nothing can resolve leaves `clear`, `less`, `vim` and \
         `top` refusing to run. Pull the new base, run `ls \
         /usr/share/terminfo/x/xterm-256color` in it, and when it is there put the new digest \
         in this test beside the measurement in its header."
    );
}

/// **The base is pinned by digest, which is what makes the measurement hold.**
///
/// A floating tag would let the image change under the measurement without a
/// line of this repository changing, and the test above would go on passing
/// while saying nothing. The pin is the mechanism; this holds the mechanism.
#[test]
fn the_base_is_pinned_by_digest_and_not_by_a_tag() {
    let recipe = the_recipe();
    let named = recipe
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(THE_ARGUMENT))
        .expect("the recipe names a base for the image");

    assert!(
        named.contains("@sha256:"),
        "the image's base is {named:?}, which is a tag rather than a digest. A tag moves, so \
         anything measured about the base — the terminfo database alo's terminal needs, among \
         the rest — stops being about the image this repository builds."
    );
}

/// Every `alo-…` command the shell integration names, and where it was named.
fn what_the_integration_runs() -> Vec<(String, String)> {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(THE_INTEGRATION);
    let mut named = Vec::new();
    let read =
        std::fs::read_dir(&at).unwrap_or_else(|why| panic!("{} was not read: {why}", at.display()));
    let mut files: Vec<std::path::PathBuf> = read
        .map(|entry| entry.expect("an entry of this repository").path())
        .filter(|at| at.extension().is_some_and(|had| had == "bash"))
        .collect();
    // `read_dir` has no order and the order differs by filesystem, so a test
    // that reported in the order it read would say something different on
    // another disk.
    files.sort();
    for file in files {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|why| panic!("{} was not read: {why}", file.display()));
        let file = file
            .file_name()
            .expect("a file this test just read")
            .to_string_lossy()
            .into_owned();
        for word in text.split(|at: char| !at.is_alphanumeric() && at != '-' && at != '.') {
            let word = word.trim_matches('.');
            if !word.starts_with("alo-") || NOT_COMMANDS.contains(&word) {
                continue;
            }
            let found = (word.to_owned(), file.clone());
            if !named.contains(&found) {
                named.push(found);
            }
        }
    }
    named
}

/// **Every command the shell integration names is a command the image ships.**
///
/// # The gap this closes, and it had been open for a day
///
/// `shell-integration/alo-before.bash` sets `PS0='$(alo-before)'`, which is what
/// the Settings row under Terminal writes into a person's own `.bashrc`. On
/// 2026-10-22 that landed, and **`alo-before` was in no image**: not built, not
/// copied, not named anywhere under `image/`. The row would have turned on a
/// line that reports a command not found before every command the person typed,
/// for ever, because `PS0` is expanded each time.
///
/// Nothing was red. The binary built, its tests passed, the row drew, the
/// snippet was correct, and the one thing nobody checked was whether a person at
/// a machine could run the thing it names. **It is the third time this exact
/// shape has been found here** — `alo-shipping` named in the wrong package
/// (`the_recipe_builds_what_it_asks_for`), `alo-ask` mentioned in 151 files and
/// shipped by none of them, and now this.
///
/// # Why it is read off the integration rather than from a list
///
/// A list in this file would be a second place to keep up to date, and the first
/// place to go stale. What a person's shell runs is in the shell integration;
/// this reads that, and the only hand-kept thing is the two words that are not
/// commands.
#[test]
fn every_command_the_shell_integration_names_is_one_the_image_ships() {
    let recipe = the_recipe();
    let named = what_the_integration_runs();

    // The control. An empty reading would make the loop below pass against any
    // recipe at all, which is this repository's own commonest false green.
    assert!(
        named.len() > 1,
        "read {} command(s) out of {THE_INTEGRATION}, so this test is broken rather \
         than the recipe: {named:?}",
        named.len()
    );

    let mut missing = Vec::new();
    for (command, file) in &named {
        let shipped = recipe
            .lines()
            .any(|line| line.trim_end().ends_with(&format!("/usr/bin/{command}")));
        if !shipped {
            missing.push(format!("{command}, named in {file}"));
        }
    }
    assert!(
        missing.is_empty(),
        "the shell integration a person pastes into their own shell runs {} command(s) \
         the image does not put on their path:\n\n- {}\n\nEach one is a line in their \
         `.bashrc` that reports a command not found. Add a `--package` to the build \
         stage and a `COPY … /usr/bin/<name>` beside `alo-try` and `alo-ask`.",
        missing.len(),
        missing.join("\n- ")
    );
}

/// **Every such command is also built**, which is the other half of being
/// shipped.
///
/// A `COPY` of a path the build stage never produced fails the image build two
/// hundred lines after the mistake, with *no such file or directory* — which is
/// the failure `the_recipe_builds_what_it_asks_for` was written about. This
/// catches it in a gate instead.
#[test]
fn every_command_the_shell_integration_names_is_built_by_the_recipe() {
    let recipe = the_recipe();
    let named = what_the_integration_runs();
    assert!(named.len() > 1, "read nothing, so this test is broken");

    let mut missing = Vec::new();
    for (command, file) in &named {
        if !recipe.contains(&format!("--package {command}")) {
            missing.push(format!("{command}, named in {file}"));
        }
    }
    assert!(
        missing.is_empty(),
        "the recipe copies {} command(s) it never asks `cargo build` for:\n\n- {}\n\n\
         `cargo build --package X` is what puts the binary where the `COPY` looks, and a \
         `COPY` of a path that was never built fails the image build rather than the gates.",
        missing.len(),
        missing.join("\n- ")
    );
}
