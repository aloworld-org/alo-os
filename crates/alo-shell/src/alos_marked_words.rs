//! alo's words read for the emphasis a model marks in Markdown, so a person
//! sees a heading as a heading and bold as bold rather than reading past `**`
//! and `#`.
//!
//! # What the owner decided, and so what this does
//!
//! Until 2026-10-28 every mark was removed, because the design draws message
//! text in one plain style and nothing in it says what a bold word or a heading
//! inside a message looks like. **On 2026-10-28 the owner decided they are
//! drawn**, with the styles taken from the design's existing type scale;
//! `docs/design/emphasis-in-alos-answers.md` records the decision and the
//! mapping. So this reads each mark into `HowItIsMarked` and leaves *how* it
//! looks to `crate::task_text`, which holds the type styles:
//!
//! - `**strong**` and `__strong__`, in pairs, are `HowItIsMarked::Strong`;
//! - `#`, `##` and `###` before a heading make the line a
//!   `HowItIsMarked::Heading`;
//! - `` `code` `` in pairs, and the lines inside a ```` ``` ```` fence, are
//!   `HowItIsMarked::Code`; the fence lines themselves go.
//!
//! # What is still only taken out
//!
//! - `####` to `######` lose their hashes and the line is drawn
//!   `HowItIsMarked::Strong` — the type scale has one heading style for a
//!   message, and three sizes of heading inside a reply is a document, not a
//!   conversation;
//! - `-`, `*` and `+` starting a list item become `•`, as before;
//! - `[words](address)` becomes `words (address)` — **links are not drawn as
//!   links**: nothing in a task window can be followed, and where a link goes
//!   is still said rather than hidden. Images and tables are not read at all.
//!
//! **Single `*` and `_` are left alone**, because they are too often not
//! Markdown — a multiplication, a file called `my_file_name`. Removing them
//! would change what alo said; leaving them costs a stray character.
//!
//! # Only alo's own words
//!
//! A person's words are theirs and are drawn exactly as typed. A proposed
//! change is the sentence a person approves, and `alo_tasks` holds that nothing
//! may rewrite it. So `MarkedWords::read_from` is applied to alo's replies
//! and questions only, by `crate::the_tasks_words`, and everything else is
//! `MarkedWords::as_said`.

/// How one stretch of alo's words is marked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowItIsMarked {
    /// Not marked: drawn in the block's own style.
    Plain,
    /// `**bold**` or `__bold__`.
    Strong,
    /// A line that began with `#`, `##` or `###`.
    Heading,
    /// `` `code` ``, or a line inside a fenced block.
    Code,
}

/// Where one marked stretch ends, and how it is marked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AMark {
    /// The byte in the text where this stretch ends; it begins where the one
    /// before it ended.
    pub ends: usize,
    /// How it is marked.
    pub how: HowItIsMarked,
}

/// Words, and how each stretch of them is marked.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkedWords {
    /// The words a person reads, with the marks taken out.
    text: String,
    /// Each stretch, in order, covering the whole text. Empty means plain.
    marks: Vec<AMark>,
}

impl MarkedWords {
    /// Words exactly as said, with nothing read into them.
    #[must_use]
    pub fn as_said(said: &str) -> Self {
        Self {
            text: said.to_owned(),
            marks: Vec::new(),
        }
    }

    /// alo's words, with its Markdown read into marks.
    #[must_use]
    pub fn read_from(said: &str) -> Self {
        let mut words = Self::default();
        let mut in_a_fence = false;
        let mut first = true;
        for line in said.lines() {
            if is_a_fence(line) {
                in_a_fence = !in_a_fence;
                continue;
            }
            if !first {
                words.push("\n", HowItIsMarked::Plain);
            }
            first = false;
            if in_a_fence {
                words.push(line, HowItIsMarked::Code);
            } else {
                words.one_line(line);
            }
        }
        words
    }

    /// The words a person reads.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// How each stretch is marked; empty when nothing is.
    #[must_use]
    pub fn marks(&self) -> &[AMark] {
        &self.marks
    }

    /// Add words marked this way, joining them to the stretch before when it
    /// is marked the same.
    fn push(&mut self, words: &str, how: HowItIsMarked) {
        if words.is_empty() {
            return;
        }
        self.text.push_str(words);
        let ends = self.text.len();
        match self.marks.last_mut() {
            Some(last) if last.how == how => last.ends = ends,
            _ => self.marks.push(AMark { ends, how }),
        }
    }

    /// One line, read for its marks.
    fn one_line(&mut self, line: &str) {
        let indent_is = line.len() - line.trim_start().len();
        let (indent, rest) = line.split_at(indent_is);
        self.push(indent, HowItIsMarked::Plain);
        let (rest, whole_line) = match a_heading(rest) {
            Some((hashes, heading)) if hashes <= 3 => (heading, HowItIsMarked::Heading),
            Some((_, heading)) => (heading, HowItIsMarked::Strong),
            None => (rest, HowItIsMarked::Plain),
        };
        let rest = as_a_bullet(rest);
        let rest = links_said_plainly(&rest);
        self.inline(&rest, whole_line);
    }

    /// Bold and code inside one line, which is otherwise marked `whole_line`.
    ///
    /// A mark counts only when the line holds an even number of it, so a lone
    /// `**` that was never Markdown is kept as written. Inside a heading,
    /// bold and code take the heading's style: it is already the heaviest.
    fn inline(&mut self, line: &str, whole_line: HowItIsMarked) {
        let pairs = |mark: &str| {
            let how_many = line.matches(mark).count();
            how_many >= 2 && how_many.is_multiple_of(2)
        };
        let (stars, unders, ticks) = (pairs("**"), pairs("__"), pairs("`"));
        let (mut strong, mut code) = (false, false);
        let mut rest = line;
        while let Some(letter) = rest.chars().next() {
            let mark = if ticks && rest.starts_with('`') {
                code = !code;
                1
            } else if !code
                && ((stars && rest.starts_with("**")) || (unders && rest.starts_with("__")))
            {
                strong = !strong;
                2
            } else {
                0
            };
            if mark > 0 {
                rest = rest.get(mark..).unwrap_or_default();
                continue;
            }
            let how = match whole_line {
                HowItIsMarked::Heading => HowItIsMarked::Heading,
                _ if code => HowItIsMarked::Code,
                _ if strong => HowItIsMarked::Strong,
                other => other,
            };
            let (one, after) = rest.split_at(letter.len_utf8());
            self.push(one, how);
            rest = after;
        }
    }
}

/// Whether this line only opens or closes a block of code.
fn is_a_fence(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// `# Heading` to how many hashes and `Heading`, for one to six hashes and a
/// space; [`None`] for a line that is not a heading.
fn a_heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|letter| *letter == '#').count();
    if (1..=6).contains(&hashes)
        && let Some(after) = line.get(hashes..)
        && let Some(heading) = after.strip_prefix(' ')
    {
        return Some((hashes, heading));
    }
    None
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
#[path = "alos_marked_words_tests.rs"]
mod tests;
