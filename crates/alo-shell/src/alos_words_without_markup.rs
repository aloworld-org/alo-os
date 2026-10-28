//! alo's words with the Markdown a model writes taken out of them, so a person
//! reads what it said rather than how it was marked up.
//!
//! Walking a booted machine on 2026-10-28, an answer arrived as
//! `1. **Header**:` and `  - The word "Invoice" clearly indicated.` — asterisks
//! a person has to read past, in a window the design calls *chat plus readable
//! activity, not a terminal log*.
//!
//! # What the design allows, and so what this does
//!
//! `docs/design/the-ask-alo-task-window.md` gives what was said **one style**:
//! Body/Large, 16 / 25, Regular. There is no bold body, no heading inside a
//! message and no code face in the measured frames, so rendering Markdown — a
//! heavier weight for `**`, a larger size for `#` — would be drawing styles the
//! design never drew. **What this does instead is remove the marks**, which
//! needs no style at all:
//!
//! - `**strong**` and `__strong__` lose their markers when they come in pairs;
//! - `` `code` `` loses its backticks in pairs, and a ```` ``` ```` fence line goes;
//! - `#` to `######` before a heading go;
//! - `-`, `*` and `+` starting a list item become `•`;
//! - `[words](address)` becomes `words (address)`, so where a link goes is
//!   still said rather than hidden.
//!
//! Numbered lists, line breaks and indentation are left as they are: they read
//! as plain text already.
//!
//! **Single `*` and `_` are left alone**, because they are too often not
//! Markdown — a multiplication, a file called `my_file_name`. Removing them
//! would change what alo said; leaving them costs a stray character.
//!
//! # Only alo's own words
//!
//! A person's words are theirs and are drawn exactly as typed. A proposed
//! change is the sentence a person approves, and `alo_tasks` holds that nothing
//! may rewrite it. So this is applied to alo's replies and questions only, by
//! `crate::the_tasks_words`.

/// What alo said, without its Markdown marks.
pub(crate) fn without_markup(said: &str) -> String {
    said.lines()
        .filter(|line| !is_a_fence(line))
        .map(one_line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether this line only opens or closes a block of code.
fn is_a_fence(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// One line, without its marks.
fn one_line(line: &str) -> String {
    let indent_is = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_is);
    let rest = without_a_heading(rest);
    let rest = as_a_bullet(rest);
    let rest = without_pairs(&rest, "**");
    let rest = without_pairs(&rest, "__");
    let rest = without_pairs(&rest, "`");
    let rest = links_said_plainly(&rest);
    format!("{indent}{rest}")
}

/// `# Heading` to `Heading`, for one to six hashes and a space.
fn without_a_heading(line: &str) -> &str {
    let hashes = line.chars().take_while(|letter| *letter == '#').count();
    if (1..=6).contains(&hashes)
        && let Some(after) = line.get(hashes..)
        && let Some(heading) = after.strip_prefix(' ')
    {
        return heading;
    }
    line
}

/// `- item`, `* item` and `+ item` to `• item`.
fn as_a_bullet(line: &str) -> String {
    for mark in ["- ", "* ", "+ "] {
        if let Some(item) = line.strip_prefix(mark) {
            return format!("\u{2022} {item}");
        }
    }
    line.to_owned()
}

/// Every `mark`, gone — but only when they come in pairs, so a lone one that
/// was never Markdown is kept.
fn without_pairs(line: &str, mark: &str) -> String {
    let how_many = line.matches(mark).count();
    if how_many >= 2 && how_many.is_multiple_of(2) {
        line.replace(mark, "")
    } else {
        line.to_owned()
    }
}

/// `[words](address)` to `words (address)`.
fn links_said_plainly(line: &str) -> String {
    let mut said = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(opens) = rest.find('[') {
        let (before, from) = rest.split_at(opens);
        said.push_str(before);
        let link = from.get(1..).and_then(|after| {
            let closes = after.find("](")?;
            let words = after.get(..closes)?;
            let after_words = after.get(closes + 2..)?;
            let ends = after_words.find(')')?;
            let address = after_words.get(..ends)?;
            let left = after_words.get(ends + 1..)?;
            Some((words, address, left))
        });
        match link {
            Some((words, address, left)) => {
                said.push_str(words);
                said.push_str(" (");
                said.push_str(address);
                said.push(')');
                rest = left;
            }
            None => {
                said.push('[');
                rest = from.get(1..).unwrap_or_default();
            }
        }
    }
    said.push_str(rest);
    said
}

#[cfg(test)]
#[path = "alos_words_without_markup_tests.rs"]
mod tests;
