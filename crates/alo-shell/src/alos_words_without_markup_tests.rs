//! That the Markdown a model writes is taken out, and that what was never
//! Markdown is kept.

use super::without_markup;

/// **The answer a booted machine showed on 2026-10-28**, read plainly.
#[test]
fn the_answer_the_walk_saw_reads_plainly() {
    let said = "Here are the key components:\n\n1. **Header**:\n   - The word \"Invoice\" \
                clearly indicated.\n\n2. **Unique Invoice Number**:\n   - Each invoice has one.";
    assert_eq!(
        without_markup(said),
        "Here are the key components:\n\n1. Header:\n   \u{2022} The word \"Invoice\" \
         clearly indicated.\n\n2. Unique Invoice Number:\n   \u{2022} Each invoice has one."
    );
}

/// **Headings, code and links lose their marks and keep their words.**
#[test]
fn headings_code_and_links_keep_their_words() {
    assert_eq!(without_markup("## What it holds"), "What it holds");
    assert_eq!(
        without_markup("Run `alo-back` to undo."),
        "Run alo-back to undo."
    );
    assert_eq!(without_markup("```sh\nalo-back\n```"), "alo-back");
    assert_eq!(
        without_markup("See [the guide](https://example.org/guide)."),
        "See the guide (https://example.org/guide)."
    );
    assert_eq!(without_markup("__Note__: kept"), "Note: kept");
}

/// **What was never Markdown is kept**: a lone asterisk, a name with
/// underscores, a hash with no space, a bracket that is not a link.
#[test]
fn what_was_never_markdown_is_kept() {
    for plain in [
        "2 * 3 is 6",
        "open my_file_name.txt",
        "#hashtag",
        "an odd **one",
        "a list [1, 2] of two",
        "1. a numbered line",
        "",
    ] {
        assert_eq!(without_markup(plain), plain, "{plain:?} was changed");
    }
}
