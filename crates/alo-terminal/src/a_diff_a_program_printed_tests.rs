//! **The refusals outnumber the happy paths**, and the sharpest of them is the
//! one that keeps a `+++ b/file` header from being read as a line somebody
//! added.

use super::*;

/// `git diff`'s output for the change the design draws.
fn the_designs_diff() -> Vec<String> {
    [
        "diff --git a/src/auth.ts b/src/auth.ts",
        "index 50680969..104b82b6 100644",
        "--- a/src/auth.ts",
        "+++ b/src/auth.ts",
        "@@ -12,4 +12,4 @@",
        " export async function signIn() {",
        "   const response = await login();",
        "-  return response.user.name;",
        "+  return response.user?.name ?? \"\";",
        " }",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect()
}

/// **The design's own diff comes back as the design draws it.**
///
/// `src/auth.ts`, one removal, one addition, and five rows numbered 12, 13,
/// 14, 14, 15 — the removal and the addition sharing a number because one line
/// replaced another at the same place.
#[test]
fn the_designs_diff_is_read_as_the_design_draws_it() {
    let found = a_diff_in(&the_designs_diff()).expect("git diff's output is a diff");

    assert_eq!(found.it_said_file(), "src/auth.ts");
    assert_eq!(found.it_said_removals(), 1);
    assert_eq!(found.it_said_additions(), 1);

    let numbered: Vec<_> = found
        .it_said_lines()
        .iter()
        .map(|line| (line.it_said_number(), line.it_said_how()))
        .collect();
    assert_eq!(
        numbered,
        vec![
            (12, HowItChanged::Unchanged),
            (13, HowItChanged::Unchanged),
            (14, HowItChanged::Removed),
            (14, HowItChanged::Added),
            (15, HowItChanged::Unchanged),
        ],
        "the design draws 12, 13, \u{2212}14, +14, 15 — the removal and the addition sharing a \
         number because one line replaced another at the same place"
    );
}

/// **`+++ b/file` is a header, not a line somebody added.**
///
/// It begins with a `+`, so a reader that checked markers before headers would
/// count it as an addition, number it, and draw the file's own name as a line
/// of its contents. The count would be one too many on every diff ever shown.
#[test]
fn the_file_header_is_not_read_as_an_added_line() {
    let found = a_diff_in(&the_designs_diff()).expect("a diff");
    assert_eq!(
        found.it_said_additions(),
        1,
        "only the real addition counts, not the `+++` header"
    );
    assert!(
        found
            .it_said_lines()
            .iter()
            .all(|line| !line.it_said_text().contains("src/auth.ts")),
        "the file's name was drawn as one of its own lines"
    );
}

/// **`--- a/file` is a header, not a line somebody removed**, for the same
/// reason.
#[test]
fn the_other_file_header_is_not_read_as_a_removed_line() {
    let found = a_diff_in(&the_designs_diff()).expect("a diff");
    assert_eq!(found.it_said_removals(), 1);
}

/// **A removal carries the old side's number and an addition the new side's.**
///
/// The two advance separately, which is what makes a replaced line show the
/// same number twice and a long insertion keep counting.
#[test]
fn the_two_sides_are_numbered_separately() {
    let lines: Vec<String> = [
        "+++ b/x",
        "@@ -10,2 +20,4 @@",
        " context",
        "+one",
        "+two",
        " after",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();

    let found = a_diff_in(&lines).expect("a diff");
    let numbered: Vec<_> = found
        .it_said_lines()
        .iter()
        .map(ADiffLine::it_said_number)
        .collect();
    assert_eq!(
        numbered,
        vec![20, 21, 22, 23],
        "the new side runs 20, 21, 22, 23 while the old side only reaches 11"
    );
}

/// **The marks are the design's, and the removal is not an ASCII hyphen.**
///
/// What was measured is *a long centred dash, visibly not a hyphen*. The exact
/// codepoint was an inference and the design document says so, which is why
/// this asserts the distinction that was measured rather than the one that was
/// guessed.
#[test]
fn the_removal_mark_is_not_the_ascii_hyphen() {
    assert_ne!(HowItChanged::Removed.the_mark(), "-");
    assert_eq!(HowItChanged::Added.the_mark(), "+");
    assert_ne!(
        HowItChanged::Removed.the_mark(),
        HowItChanged::Added.the_mark()
    );
}

/// **A diff with no file header is not a diff.**
#[test]
fn a_hunk_with_no_file_is_refused() {
    let lines: Vec<String> = ["@@ -1,2 +1,2 @@", "-gone", "+there"]
        .iter()
        .map(|it| (*it).to_owned())
        .collect();
    assert_eq!(a_diff_in(&lines), None);
}

/// **Lines before any hunk header are not numbered from a guess.**
#[test]
fn lines_before_a_hunk_header_are_not_read() {
    let lines: Vec<String> = ["+++ b/x", "-this is not in a hunk", "+nor is this"]
        .iter()
        .map(|it| (*it).to_owned())
        .collect();
    assert_eq!(a_diff_in(&lines), None, "no hunk, so nothing is numbered");
}

/// **A hunk header this cannot read starts no hunk.**
#[test]
fn an_unreadable_hunk_header_starts_nothing() {
    let lines: Vec<String> = ["+++ b/x", "@@ what even is this @@", "-gone", "+there"]
        .iter()
        .map(|it| (*it).to_owned())
        .collect();
    assert_eq!(a_diff_in(&lines), None);
}

/// **Output after the diff ends does not join it.**
///
/// A prompt, or the next command's output, begins with none of the three
/// markers and ends the hunk rather than being numbered into it.
#[test]
fn output_after_the_hunk_does_not_join_it() {
    let mut lines = the_designs_diff();
    lines.push("someone@machine:~$ echo done".to_owned());
    lines.push("done".to_owned());

    let found = a_diff_in(&lines).expect("a diff");
    assert_eq!(
        found.it_said_lines().len(),
        5,
        "the five hunk lines, and nothing the shell printed afterwards"
    );
}

/// **A diff of a deleted file names the file it was**, not `/dev/null`.
#[test]
fn a_deleted_files_diff_does_not_take_dev_null_as_its_name() {
    let lines: Vec<String> = [
        "--- a/gone.txt",
        "+++ /dev/null",
        "@@ -1,1 +0,0 @@",
        "-it was here",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();
    assert_eq!(
        a_diff_in(&lines),
        None,
        "with no name but /dev/null there is nothing to title the view with"
    );
}

/// **Ordinary output is not a diff**, which is nearly everything.
#[test]
fn ordinary_output_is_not_a_diff() {
    let lines: Vec<String> = [
        "total 48",
        "- a bullet point",
        "+ another one",
        "a line mentioning git diff",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();
    assert_eq!(a_diff_in(&lines), None);
}

/// **A screen of markers cannot make this allocate without limit.**
#[test]
fn a_screen_of_changes_stops_at_the_bound() {
    let mut lines = vec!["+++ b/x".to_owned(), "@@ -1,1 +1,400 @@".to_owned()];
    for which in 0..400 {
        lines.push(format!("+line {which}"));
    }
    let found = a_diff_in(&lines).expect("a diff");
    assert!(
        found.it_said_lines().len() <= AT_MOST_THIS_MANY_LINES,
        "read {} lines, which is past the bound",
        found.it_said_lines().len()
    );
}

/// **Every diff carries its provenance**, and no shape of the type omits it.
#[test]
fn a_diff_always_says_where_it_came_from_and_that_nothing_is_applied() {
    let found = a_diff_in(&the_designs_diff()).expect("a diff");
    assert_eq!(
        found.what_printed_it().named(),
        "terminal.diff.from-git-diff"
    );
    assert_eq!(
        found.and_nothing_is_applied().named(),
        "terminal.diff.nothing-applied"
    );
}
