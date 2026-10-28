//! That alo's Markdown is read into marks with its characters taken out, that
//! what was never Markdown is kept, and that a person's words are never read.

use super::{HowItIsMarked, MarkedWords};

/// Each stretch of the words with how it is marked, for comparing.
fn stretches(words: &MarkedWords) -> Vec<(&str, HowItIsMarked)> {
    let mut from = 0;
    words
        .marks()
        .iter()
        .map(|mark| {
            let one = words.text().get(from..mark.ends).unwrap_or_default();
            from = mark.ends;
            (one, mark.how)
        })
        .collect()
}

/// **The answer a booted machine showed on 2026-10-28**, read for its marks:
/// the words a person reads lose every `**`, and the two headers are bold.
#[test]
fn the_answer_the_walk_saw_reads_with_its_bold() {
    let said = "Here are the key components:\n\n1. **Header**:\n   - The word \"Invoice\" \
                clearly indicated.\n\n2. **Unique Invoice Number**:\n   - Each invoice has one.";
    let words = MarkedWords::read_from(said);
    assert_eq!(
        words.text(),
        "Here are the key components:\n\n1. Header:\n   \u{2022} The word \"Invoice\" \
         clearly indicated.\n\n2. Unique Invoice Number:\n   \u{2022} Each invoice has one."
    );
    let strong: Vec<&str> = stretches(&words)
        .into_iter()
        .filter(|(_, how)| *how == HowItIsMarked::Strong)
        .map(|(one, _)| one)
        .collect();
    assert_eq!(strong, ["Header", "Unique Invoice Number"]);
}

/// **One to three hashes make a heading; four to six are bold.**
#[test]
fn a_heading_is_a_heading_up_to_three_hashes() {
    for said in ["# What it holds", "## What it holds", "### What it holds"] {
        let words = MarkedWords::read_from(said);
        assert_eq!(words.text(), "What it holds");
        assert_eq!(
            stretches(&words),
            [("What it holds", HowItIsMarked::Heading)],
            "{said:?}"
        );
    }
    let deeper = MarkedWords::read_from("#### What it holds");
    assert_eq!(
        stretches(&deeper),
        [("What it holds", HowItIsMarked::Strong)]
    );
}

/// **Code is marked and its backticks go; a fence's lines go and what is
/// inside it is code.**
#[test]
fn code_is_marked_and_its_backticks_go() {
    let inline = MarkedWords::read_from("Run `alo-back` to undo.");
    assert_eq!(inline.text(), "Run alo-back to undo.");
    assert_eq!(
        stretches(&inline),
        [
            ("Run ", HowItIsMarked::Plain),
            ("alo-back", HowItIsMarked::Code),
            (" to undo.", HowItIsMarked::Plain),
        ]
    );
    let fenced = MarkedWords::read_from("Then:\n```sh\nalo-back\n```");
    assert_eq!(fenced.text(), "Then:\nalo-back");
    assert_eq!(
        stretches(&fenced).last(),
        Some(&("alo-back", HowItIsMarked::Code))
    );
}

/// **A link is said plainly, and underscores in pairs are bold.**
#[test]
fn links_are_said_and_underscores_are_bold() {
    let link = MarkedWords::read_from("See [the guide](https://example.org/guide).");
    assert_eq!(link.text(), "See the guide (https://example.org/guide).");
    assert_eq!(
        stretches(&link),
        [(
            "See the guide (https://example.org/guide).",
            HowItIsMarked::Plain
        )]
    );
    let note = MarkedWords::read_from("__Note__: kept");
    assert_eq!(
        stretches(&note),
        [
            ("Note", HowItIsMarked::Strong),
            (": kept", HowItIsMarked::Plain)
        ]
    );
}

/// **What was never Markdown is kept, and is not marked**: a lone asterisk, a
/// name with underscores, a hash with no space, a bracket that is not a link.
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
        let words = MarkedWords::read_from(plain);
        assert_eq!(words.text(), plain, "{plain:?} was changed");
        assert!(
            words
                .marks()
                .iter()
                .all(|mark| mark.how == HowItIsMarked::Plain),
            "{plain:?} was marked"
        );
    }
}

/// **The marks cover the whole text, in order**, so a drawing that walks them
/// draws every character exactly once.
#[test]
fn the_marks_cover_the_whole_text() {
    let words = MarkedWords::read_from("# Plan\n\n- **one** and `two`\nthree");
    let mut from = 0;
    for mark in words.marks() {
        assert!(mark.ends > from, "a stretch is empty or out of order");
        from = mark.ends;
    }
    assert_eq!(from, words.text().len());
}

/// **A person's words are never read**: the same marks, kept as typed, with
/// nothing marked. The refusal path of this file.
#[test]
fn words_as_said_are_not_read() {
    let typed = "rename **draft**.txt and `# not a heading`";
    let words = MarkedWords::as_said(typed);
    assert_eq!(words.text(), typed);
    assert!(words.marks().is_empty());
}
