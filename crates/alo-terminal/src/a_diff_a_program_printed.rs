//! A file diff, recognised in what a program printed — or nothing, which is
//! the ordinary case and not the failure case.
//!
//! `docs/features.md`, starred: *Output that is not stuck in a character grid.*
//! The third of the design's three rich views, after the test result and the
//! benchmark.
//!
//! # The design specifies this one, since 2026-10-18
//!
//! `479:28986` — *Terminal / View / File diff*, 990x320 — measured off the live
//! file by the desktop lane and recorded in
//! `docs/design/the-terminal-window.md`:
//!
//! | what the design draws | what it is |
//! |---|---|
//! | `src/auth.ts · 1 removal, 1 addition` | the summary, from the counts |
//! | `Saved diff · Source: git diff · No changes applied by this view` | **the provenance, and it is not optional** |
//! | five rows — `12`, `13`, `−14`, `+14`, `15` | the body, with the signs in a gutter left of the numbers |
//! | three `Ask alo / Action` | not built — they are the agent's road |
//!
//! **The provenance says something the other two views do not.** A test result
//! says nobody checked it; this says **no changes are applied by looking at
//! it**. A diff is the one rich view whose subject a person might reasonably
//! fear it could act on, so the sentence that says it does not is the one the
//! design chose.
//!
//! *`Saved diff` is not drawn.* alo recognises a diff in what a program printed
//! rather than opening a saved one, so the words would describe something that
//! did not happen. **A deliberate departure, recorded here and in the design
//! document** rather than left in a diff of its own.
//!
//! # One format, and it is named
//!
//! **Unified diff, as `git diff` prints it**, which is the design's own
//! `Source: git diff`. `CLAUDE.md`: *one printer that works, not half of all
//! printers.*
//!
//! # Everything here is a claim the program made
//!
//! There is no `removals()`. There is [`ADiff::it_said_removals`], for the
//! reason [`crate::a_test_result_a_program_printed`] gives at length: a count
//! scraped out of a stranger's output is evidence of what was printed and of
//! nothing else.

use crate::words;

/// At most this many lines of a diff are read.
///
/// A bound rather than a budget. The design draws five; a screenful is the most
/// that could ever be shown, and this stops a screen of adversarial text
/// allocating without limit.
const AT_MOST_THIS_MANY_LINES: usize = 64;

/// What happened to one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowItChanged {
    /// It is there before and after, and is shown for context.
    Unchanged,
    /// It was taken away.
    Removed,
    /// It was put there.
    Added,
}

impl HowItChanged {
    /// The mark the design puts in the gutter left of the number.
    ///
    /// **The removal is `U+2212 MINUS SIGN` and not `U+002D HYPHEN-MINUS`.**
    /// What was measured off the design is *a long centred dash aligned with
    /// the plus's crossbar, visibly not a hyphen* — the exact codepoint was not
    /// byte-verified, and `docs/design/the-terminal-window.md` says so. What is
    /// certain is that it is not the ASCII hyphen, and `U+2212` is the glyph
    /// that description names.
    ///
    /// **Not a vocabulary string.** These are marks, not words: the same two in
    /// every language, like the `·` between the summary's parts.
    #[must_use]
    pub const fn the_mark(self) -> &'static str {
        match self {
            Self::Unchanged => " ",
            Self::Removed => "\u{2212}",
            Self::Added => "+",
        }
    }
}

/// One line of a diff, as the program printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ADiffLine {
    /// Which line it is, on whichever side it belongs to.
    number: u32,
    /// What happened to it.
    how: HowItChanged,
    /// The line itself, without its marker.
    text: String,
}

impl ADiffLine {
    /// Which line the program said this is.
    ///
    /// **The old side's number for a removal and the new side's for an
    /// addition**, which is why the design draws `−14` and `+14` together: one
    /// line replaced another at the same place.
    #[must_use]
    pub const fn it_said_number(&self) -> u32 {
        self.number
    }

    /// What the program said happened to it.
    #[must_use]
    pub const fn it_said_how(&self) -> HowItChanged {
        self.how
    }

    /// The line itself, as the program printed it, without its marker.
    #[must_use]
    pub fn it_said_text(&self) -> &str {
        &self.text
    }
}

/// A file diff, as a program printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ADiff {
    /// What the program called the file.
    file: String,
    /// How many lines it said were taken away.
    removals: usize,
    /// How many it said were put there.
    additions: usize,
    /// The lines, in the order the program printed them.
    lines: Vec<ADiffLine>,
}

impl ADiff {
    /// What the program called the file.
    ///
    /// **Not this repository's words**, and never a path this machine resolved
    /// — a diff of a file that does not exist here is still a diff.
    #[must_use]
    pub fn it_said_file(&self) -> &str {
        &self.file
    }

    /// How many lines the program said were taken away.
    #[must_use]
    pub const fn it_said_removals(&self) -> usize {
        self.removals
    }

    /// How many lines the program said were put there.
    #[must_use]
    pub const fn it_said_additions(&self) -> usize {
        self.additions
    }

    /// The lines, in the order the program printed them.
    #[must_use]
    pub fn it_said_lines(&self) -> &[ADiffLine] {
        &self.lines
    }

    /// What printed it, for the provenance line.
    #[must_use]
    pub const fn what_printed_it(&self) -> alo_strings::Word {
        words::FROM_A_GIT_DIFF
    }

    /// That looking at it changes nothing, which no shape of this type omits.
    ///
    /// **The sentence the design chose for this view and not for the other
    /// two.** A diff is the one rich result whose subject a person might fear
    /// it could act on.
    #[must_use]
    pub const fn and_nothing_is_applied(&self) -> alo_strings::Word {
        words::NO_CHANGES_APPLIED_BY_THIS_VIEW
    }
}

/// Whether this line is safe to read a diff out of.
fn it_is_plain_text(line: &str) -> bool {
    !line.chars().any(char::is_control)
}

/// The two starting line numbers on a `@@ -12,4 +12,4 @@` header.
///
/// Answers `(old, new)`. A header this cannot read starts no hunk, so the lines
/// after it are not read either — numbering them from a guess would put a
/// stranger's code beside numbers nobody printed.
fn where_the_hunk_starts(line: &str) -> Option<(u32, u32)> {
    let inside = line.trim().strip_prefix("@@ ")?;
    let (inside, _) = inside.split_once(" @@")?;
    let mut sides = inside.split_whitespace();
    let old = sides.next()?.strip_prefix('-')?;
    let new = sides.next()?.strip_prefix('+')?;
    let first = |side: &str| side.split_once(',').map_or(side, |(at, _)| at).parse().ok();
    Some((first(old)?, first(new)?))
}

/// A file diff recognised in these lines, or [`None`].
///
/// [`None`] for almost all output a terminal ever shows, and that is the
/// ordinary case rather than a failure.
///
/// # What it reads
///
/// A `+++ b/<path>` header for the file, a `@@` header for where the lines are
/// numbered from, and the lines after it.
///
/// # What makes it answer [`None`]
///
/// No file header, no hunk header, or a hunk with nothing in it. **A diff with
/// no changed line is not shown**: `git diff` prints nothing at all in that
/// case, so anything reaching here with only context lines is not a diff of
/// anything.
#[must_use]
pub fn a_diff_in(lines: &[String]) -> Option<ADiff> {
    let mut file: Option<String> = None;
    let mut found: Vec<ADiffLine> = Vec::new();
    let mut removals = 0_usize;
    let mut additions = 0_usize;
    let (mut old, mut new) = (0_u32, 0_u32);
    let mut inside_a_hunk = false;

    for line in lines {
        if found.len() >= AT_MOST_THIS_MANY_LINES {
            break;
        }
        if !it_is_plain_text(line) {
            continue;
        }

        // **The file header before the marker arms**, because `+++ b/x` starts
        // with a `+` and would otherwise be read as a line somebody added.
        if let Some(named) = line.strip_prefix("+++ ") {
            let named = named.trim();
            let named = named.strip_prefix("b/").unwrap_or(named);
            if named != "/dev/null" && !named.is_empty() {
                file = Some(named.to_owned());
            }
            continue;
        }
        if line.starts_with("--- ") {
            continue;
        }
        if let Some((from_old, from_new)) = where_the_hunk_starts(line) {
            old = from_old;
            new = from_new;
            inside_a_hunk = true;
            continue;
        }
        if !inside_a_hunk {
            continue;
        }

        let mut marks = line.chars();
        let (how, number) = match marks.next() {
            Some('-') => {
                let at = old;
                old = old.checked_add(1)?;
                removals = removals.checked_add(1)?;
                (HowItChanged::Removed, at)
            }
            Some('+') => {
                let at = new;
                new = new.checked_add(1)?;
                additions = additions.checked_add(1)?;
                (HowItChanged::Added, at)
            }
            Some(' ') => {
                let at = new;
                old = old.checked_add(1)?;
                new = new.checked_add(1)?;
                (HowItChanged::Unchanged, at)
            }
            // **Anything else ends the hunk**, which is how `git diff` is read
            // back: the next `diff --git`, a `\ No newline` note, or ordinary
            // output after the command finished.
            _ => {
                inside_a_hunk = false;
                continue;
            }
        };
        found.push(ADiffLine {
            number,
            how,
            text: marks.as_str().to_owned(),
        });
    }

    let file = file?;
    (removals > 0 || additions > 0).then_some(ADiff {
        file,
        removals,
        additions,
        lines: found,
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "a_diff_a_program_printed_tests.rs"]
mod tests;
