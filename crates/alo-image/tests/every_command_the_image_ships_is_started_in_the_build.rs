//! **A binary the image copies into `/usr/bin` is run during the build**, and
//! this is the check that each one is.
//!
//! It reads `image/Containerfile` and nothing else, so it runs anywhere and
//! needs no network — the same division `the_image_has_what_a_developer_needs`
//! draws: the build proves the binaries, this proves the build still asks.
//!
//! # The fault this exists for
//!
//! `CLAUDE.md` names it twice over: *`test -x` on a converter binary stood in
//! for the converter starting. Three releases shipped one that died at launch
//! on eleven missing libraries, and no document of any format opened on a real
//! machine.* A binary that is copied in and never started is that fault with
//! the stat removed — there is not even a stat to find.
//!
//! The recipe already does the right thing for `alo-try` and says why in its
//! own words. **Nothing held it to doing it for the next one**, and on
//! 2026-10-19 the next one was `alo-ask`, which was not in the recipe at all.
//!
//! # What this proves, and the bigger gap it does not close
//!
//! It proves that **every `/usr/bin` binary the recipe ships is started by it.**
//!
//! It cannot prove that every command a person is *promised* is shipped. That
//! is absence, and walking what is present never finds it: `alo-ask` was named
//! in 151 files across plans, contracts and decisions (ADR 0124), in none of
//! them here, and this test would have been green throughout. Closing that
//! wants an authoritative list of what a person may type, and this repository
//! has no such list today — `docs/features.md` names commands in prose, not in
//! a form anything can read.
//!
//! **So it is written down here rather than left implied**, which is the only
//! honest thing available: *the gap this does not close is the one that cost an
//! image.*
//!
//! # What this found on its first run, recorded rather than papered over
//!
//! Written to walk **every** binary the recipe ships, it failed at once on
//! `alo-compositor`: copied into `/usr/bin`, made executable, and **never
//! started by the build.**
//!
//! That is the same shape as `alo-ask`'s gap and it is not the same fault. A
//! compositor cannot start in a build container — it wants a seat and a DRM
//! device, and a container has neither — so demanding it would be a test that
//! cannot be satisfied honestly, which is a wall rather than a guard.
//!
//! So the walk was narrowed to the half that holds for everything (**nothing is
//! stat-ed instead of started**) and the start is demanded only of the commands
//! a person types, which refuse with a sentence and exit 1 when given nothing.
//! **`alo-compositor` remains shipped and unstarted.** It is the desktop lane's
//! to close with whatever that binary can honestly answer in a container, and it
//! is named here so that it is a known gap rather than a quiet one.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a recipe that cannot be read is the failure being reported"
)]

/// The commands a person types, every one of which is a promise.
///
/// Written down here because the fault of 2026-10-19 was a command in no list
/// anything read. Derivation finds what is present; only a name finds what is
/// absent.
const WHAT_A_PERSON_TYPES: [&str; 2] = ["alo-ask", "alo-try"];

/// The recipe, as one string.
fn the_recipe() -> String {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("image/Containerfile");
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("the recipe is not readable at {}: {why}", at.display()))
}

/// A recipe's lines with its comments taken out.
///
/// A Containerfile comment is a line whose first non-blank character is `#`;
/// there is no inline comment syntax. Without this, a binary discussed in a
/// comment reads as one that is shipped — and a binary *named* in a comment
/// reads as one that is started.
fn what_a_shell_would_run(recipe: &str) -> String {
    recipe
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Every binary the recipe copies to `/usr/bin`, by the name it lands under.
///
/// Derived from the recipe rather than listed here, which is the whole point:
/// a list in this file would need somebody to remember it, and remembering is
/// what failed.
fn what_the_image_ships(recipe: &str) -> Vec<String> {
    let mut shipped: Vec<String> = recipe
        .lines()
        .filter(|line| line.trim_start().starts_with("COPY"))
        .filter_map(|line| line.split_whitespace().last())
        .filter_map(|landing| landing.strip_prefix("/usr/bin/"))
        .filter(|name| name.starts_with("alo-"))
        .map(str::to_owned)
        .collect();
    shipped.sort_unstable();
    shipped.dedup();
    shipped
}

/// **No shipped binary is stat-ed instead of started.**
///
/// Derived over everything the recipe ships, because this half holds for all of
/// them: whatever cannot be *started* in a container must not be given a
/// `test -x` that reads as though it had been. A binary with no check is
/// honestly unchecked; a binary with a stat is a binary somebody thought they
/// had checked, and that is the substitution `CLAUDE.md` names.
#[test]
fn no_shipped_binary_is_stat_ed_instead_of_started() {
    let recipe = the_recipe();
    let runnable = what_a_shell_would_run(&recipe);
    let shipped = what_the_image_ships(&recipe);

    assert!(
        shipped.len() >= 2,
        "the recipe ships {shipped:?} into /usr/bin, which is fewer than the two commands a \
         person types — so this test has stopped reading the recipe"
    );

    for binary in &shipped {
        assert!(
            !runnable.contains(&format!("test -x /usr/bin/{binary}")),
            "{binary} is stat-ed with `test -x` instead of being run, which is the exact \
             substitution CLAUDE.md names: a check that stands in for the thing is not the thing"
        );
    }
}

/// **The commands a person types are started by the build**, which is the half
/// that only applies to them.
///
/// It is the named two and not every shipped binary, and the difference is the
/// finding this test was written by. Walking everything demanded that
/// `alo-compositor` start in a build container, and it cannot: a compositor
/// wants a seat and a DRM device and a container has neither. **So the recipe
/// ships `alo-compositor` and never starts it, which is recorded in this file's
/// header rather than papered over** — it is a real gap and it is the desktop
/// lane's to close with whatever that binary can honestly answer.
///
/// A command a person types has no such excuse. Both of these refuse with a
/// sentence and exit 1 when given nothing, which makes a failed run the pass
/// and a missing library a different failure.
#[test]
fn the_commands_a_person_types_are_started_by_the_build() {
    let used = where_a_binary_is_used(&the_recipe());
    for command in WHAT_A_PERSON_TYPES {
        assert!(
            used.contains(&format!("/usr/bin/{command}")),
            "{command} is copied and made executable and the recipe does nothing else with it, \
             so the build never starts it. A binary the build never starts can die at launch on \
             a missing library and nothing notices — CLAUDE.md's converter, three releases \
             running"
        );
    }
}

/// The recipe with its comments, its copies and its `chmod` taken out — so what
/// is left is the recipe **doing** something with a binary.
///
/// **This counted mentions until it was mutated.** The first version asked for a
/// binary to be named twice outside a comment, on the reasoning that a copy and
/// a run are two. Removing the run-proof entirely left the copy and the `chmod`,
/// which are also two, and the test stayed green: *a count of mentions stood in
/// for an invocation*, which is the substitution this whole file is about,
/// written into the guard on the same day as three paragraphs warning of it.
///
/// Commands are joined across their `\` continuations first, because a `chmod`
/// spans eight lines and a line-at-a-time filter would drop one line of it and
/// keep the other seven.
fn where_a_binary_is_used(recipe: &str) -> String {
    what_a_shell_would_run(recipe)
        .replace("\\\n", " ")
        .lines()
        .map(str::trim_start)
        .filter(|command| !command.starts_with("COPY"))
        .filter(|command| !command.starts_with("RUN chmod") && !command.starts_with("chmod "))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// **The two commands a person types are among them**, named rather than
/// derived.
///
/// Derivation finds what is present; this is the one place a name is written
/// down, because the fault of 2026-10-19 was a command that was in no list
/// anything read. Two is not *every* command a person may ever type — see this
/// file's header for why no such list exists yet — but a named two that must be
/// there is stronger than a derived list that cannot miss anything.
#[test]
fn the_commands_a_person_types_are_shipped() {
    let shipped = what_the_image_ships(&the_recipe());
    for command in WHAT_A_PERSON_TYPES {
        assert!(
            shipped.iter().any(|name| name == command),
            "{command} is a command a person types and the image does not ship it: {shipped:?}. \
             ADR 0124 measured that there is no `alo` binary and that `alo-ask` is the only one a \
             person could type"
        );
    }
}

/// **And the comment-stripping does not let a real reference through.**
///
/// The tests above would both pass on a recipe that had stopped running
/// anything, if stripping comments also stripped commands. The only thing that
/// tells the two apart is putting one back, in a fixture rather than in the
/// recipe.
#[test]
fn stripping_comments_leaves_the_commands_alone() {
    let pretending = "# /usr/bin/alo-ask is what this comment is about\n\
                      RUN set -e; \\\n    /usr/bin/alo-ask 2>&1 | grep -q 'nothing'\n";
    let runnable = what_a_shell_would_run(pretending);
    assert!(
        runnable.contains("/usr/bin/alo-ask"),
        "stripping comments also stripped a command, so the guards above check nothing"
    );
    assert_eq!(
        runnable.matches("/usr/bin/alo-ask").count(),
        1,
        "the comment's mention survived, so a binary discussed and never run would read as run"
    );
}
