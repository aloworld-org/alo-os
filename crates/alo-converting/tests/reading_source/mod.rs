//! Reading a crate's shipped Rust source for the string literals in it, and
//! telling English on a screen from everything else a literal can be.
//!
//! The reading is the one `alo-choosing`'s
//! `tests/no_english_outside_the_vocabulary.rs` does for the five crates that
//! keep a person's settings, and its rules are the same ones, so English means
//! the same thing in both searches. It is a module here rather than a
//! dependency on that test because a test file cannot be depended on, and it is
//! a module of its own rather than part of the test that uses it because what a
//! literal *is* and which literals are *allowed* change for different reasons.
//!
//! **Shipped means `src/`**, without `#[cfg(test)]` items or the modules they
//! declare. Comments and doc comments are skipped. A literal is **English** when
//! it holds two ordinary words side by side, which a key, a serde name, a gap's
//! name or a keyword a protocol defines never does. English is accepted by rule
//! in three places: `saying`, `noting` and `counting` in a crate's
//! `src/words.rs`, which is the vocabulary; `#[expect]` and `#[allow]`, a lint's
//! reason; and `#[doc]`.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// One piece of a source file: a character of code, or a whole string literal.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    /// A character that is code.
    Code(char),
    /// A string literal's text, and the line it starts on.
    Text(String, usize),
}

/// A string literal, with where it stood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// What it says.
    pub text: String,
    /// The line it starts on, counted from one.
    pub line: usize,
    /// The attribute it is inside, when it is inside one.
    pub attribute: Option<String>,
    /// The function or macro whose parentheses it is inside, nearest first.
    pub callee: Option<String>,
}

/// Whether this character can be part of an identifier.
fn identifier_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// A source file cut into code and string literals, with comments gone.
fn pieces_of(source: &str) -> Vec<Piece> {
    let chars: Vec<char> = source.chars().collect();
    let mut pieces = Vec::new();
    let mut line = 1;
    let mut at = 0;
    let at_char = |index: usize| chars.get(index).copied();
    while let Some(here) = at_char(at) {
        let next = at_char(at + 1);
        let before = at.checked_sub(1).and_then(at_char);
        let after_an_identifier = before.is_some_and(identifier_char);
        if here == '/' && next == Some('/') {
            while at_char(at).is_some_and(|c| c != '\n') {
                at += 1;
            }
        } else if here == '/' && next == Some('*') {
            let mut depth = 0_usize;
            while let Some(c) = at_char(at) {
                if c == '/' && at_char(at + 1) == Some('*') {
                    depth += 1;
                    at += 2;
                } else if c == '*' && at_char(at + 1) == Some('/') {
                    depth -= 1;
                    at += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if c == '\n' {
                        line += 1;
                    }
                    at += 1;
                }
            }
        } else if here == '"' {
            let starts = line;
            let mut text = String::new();
            at += 1;
            while let Some(c) = at_char(at) {
                match c {
                    '\\' => {
                        let escaped = at_char(at + 1);
                        if escaped == Some('\n') {
                            line += 1;
                            at += 2;
                            while at_char(at).is_some_and(char::is_whitespace) {
                                at += 1;
                            }
                            continue;
                        }
                        text.push(match escaped {
                            Some('n') => '\n',
                            Some('t') => '\t',
                            Some('r') => '\r',
                            Some('0') => '\0',
                            Some(other) => other,
                            None => '\\',
                        });
                        at += 2;
                    }
                    '"' => {
                        at += 1;
                        break;
                    }
                    _ => {
                        if c == '\n' {
                            line += 1;
                        }
                        text.push(c);
                        at += 1;
                    }
                }
            }
            pieces.push(Piece::Text(text, starts));
            continue;
        } else if !after_an_identifier
            && (here == 'r' || (here == 'b' && next == Some('r')))
            && raw_string_at(&chars, at).is_some()
        {
            let (hashes, opens) = raw_string_at(&chars, at).expect("checked above");
            let starts = line;
            let mut text = String::new();
            let mut index = opens;
            while let Some(c) = at_char(index) {
                let closes = c == '"' && (1..=hashes).all(|h| at_char(index + h) == Some('#'));
                if closes {
                    index += 1 + hashes;
                    break;
                }
                if c == '\n' {
                    line += 1;
                }
                text.push(c);
                index += 1;
            }
            pieces.push(Piece::Text(text, starts));
            at = index;
            continue;
        } else if here == '\'' {
            // A character literal is skipped whole, so `'"'` opens nothing; a
            // lifetime is code.
            if next == Some('\\') {
                let mut index = at + 3;
                while at_char(index).is_some_and(|c| c != '\'') {
                    index += 1;
                }
                at = index + 1;
                continue;
            }
            if at_char(at + 2) == Some('\'') {
                at += 3;
                continue;
            }
            pieces.push(Piece::Code(here));
            at += 1;
        } else {
            if here == '\n' {
                line += 1;
            }
            pieces.push(Piece::Code(here));
            at += 1;
        }
    }
    pieces
}

/// Whether a raw string begins here, as `r"`, `r#"`, `br"` and so on: how
/// many hashes it has and where its text begins.
fn raw_string_at(chars: &[char], at: usize) -> Option<(usize, usize)> {
    let mut index = at;
    if chars.get(index) == Some(&'b') {
        index += 1;
    }
    if chars.get(index) != Some(&'r') {
        return None;
    }
    index += 1;
    let mut hashes = 0;
    while chars.get(index) == Some(&'#') {
        hashes += 1;
        index += 1;
    }
    (chars.get(index) == Some(&'"')).then_some((hashes, index + 1))
}

/// Where the code at `from` spells `pattern`, ignoring whitespace in the code:
/// the index just past it.
fn spells(pieces: &[Piece], from: usize, pattern: &str) -> Option<usize> {
    let mut index = from;
    for wanted in pattern.chars() {
        loop {
            match pieces.get(index) {
                Some(Piece::Code(c)) if c.is_whitespace() => index += 1,
                Some(Piece::Code(c)) if *c == wanted => {
                    index += 1;
                    break;
                }
                _ => return None,
            }
        }
    }
    Some(index)
}

/// The index just past the item that begins at `from`: its attributes, and
/// then everything to its `;` or its matching `}`.
fn past_the_item(pieces: &[Piece], from: usize) -> usize {
    let mut index = from;
    let mut brackets = 0_usize;
    let mut braces = 0_usize;
    while let Some(piece) = pieces.get(index) {
        index += 1;
        let Piece::Code(c) = piece else { continue };
        match c {
            '[' => brackets += 1,
            ']' => brackets = brackets.saturating_sub(1),
            '{' => braces += 1,
            '}' => {
                braces = braces.saturating_sub(1);
                if braces == 0 && brackets == 0 {
                    return index;
                }
            }
            ';' if braces == 0 && brackets == 0 => return index,
            _ => {}
        }
    }
    index
}

/// What a source file shipped: its string literals outside `#[cfg(test)]`
/// items, and the modules those items declare, whose files are not shipped.
pub fn shipped_literals(source: &str) -> (Vec<Literal>, Vec<String>) {
    let pieces = pieces_of(source);
    let mut literals = Vec::new();
    let mut test_modules = Vec::new();
    // What each open parenthesis or attribute bracket was opened by.
    let mut open: Vec<(char, Option<String>)> = Vec::new();
    let mut identifier = String::new();
    let mut last_identifier: Option<String> = None;
    let mut index = 0;
    while let Some(piece) = pieces.get(index) {
        if let Some(past) = spells(&pieces, index, "#[cfg(test)]") {
            let item_ends = past_the_item(&pieces, past);
            let declared: String = pieces
                .get(past..item_ends)
                .unwrap_or_default()
                .iter()
                .filter_map(|p| match p {
                    Piece::Code(c) => Some(*c),
                    Piece::Text(..) => None,
                })
                .collect();
            let words: Vec<&str> = declared
                .split(|c: char| !identifier_char(c))
                .filter(|w| !w.is_empty())
                .collect();
            if let [.., "mod", name] = words.as_slice()
                && declared.trim_end().ends_with(';')
            {
                test_modules.push((*name).to_owned());
            }
            index = item_ends;
            identifier.clear();
            continue;
        }
        if let Some(past) = spells(&pieces, index, "#[").or_else(|| spells(&pieces, index, "#![")) {
            let name: String = pieces
                .get(past..)
                .unwrap_or_default()
                .iter()
                .skip_while(|p| matches!(p, Piece::Code(c) if c.is_whitespace()))
                .map_while(|p| match p {
                    Piece::Code(c) if identifier_char(*c) => Some(*c),
                    _ => None,
                })
                .collect();
            open.push(('[', Some(name)));
            index = past;
            identifier.clear();
            continue;
        }
        match piece {
            Piece::Code(c) if identifier_char(*c) => identifier.push(*c),
            Piece::Code(c) => {
                if !identifier.is_empty() {
                    last_identifier = Some(std::mem::take(&mut identifier));
                }
                match c {
                    '(' => open.push(('(', last_identifier.take())),
                    '[' => open.push(('[', None)),
                    ')' | ']' => {
                        open.pop();
                    }
                    '!' => {}
                    c if c.is_whitespace() => {}
                    _ => last_identifier = None,
                }
            }
            Piece::Text(text, line) => {
                identifier.clear();
                last_identifier = None;
                literals.push(Literal {
                    text: text.clone(),
                    line: *line,
                    attribute: open
                        .iter()
                        .find(|(bracket, name)| *bracket == '[' && name.is_some())
                        .and_then(|(_, name)| name.clone()),
                    callee: open
                        .iter()
                        .rev()
                        .find(|(bracket, _)| *bracket == '(')
                        .and_then(|(_, name)| name.clone()),
                });
            }
        }
        index += 1;
    }
    (literals, test_modules)
}

/// Whether this text holds two ordinary words side by side.
pub fn is_english(text: &str) -> bool {
    let ordinary = |word: &str| {
        let mut chars = word.chars();
        chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && word.chars().count() >= 2
            && chars.all(|c| c.is_ascii_lowercase() || c == '\'')
    };
    let words: Vec<&str> = text
        .split_whitespace()
        .map(|word| word.trim_end_matches([',', '.', ';', ':']))
        .collect();
    words
        .windows(2)
        .any(|pair| pair.iter().all(|word| ordinary(word)))
}

/// Why a literal is English that is meant to be there, or `None` when it is
/// not accepted by rule.
fn accepted_by_rule(file_name: &str, literal: &Literal) -> Option<&'static str> {
    match literal.attribute.as_deref() {
        Some("expect" | "allow") => return Some("a lint's reason"),
        Some("doc") => return Some("rustdoc"),
        _ => {}
    }
    let declared = matches!(
        literal.callee.as_deref(),
        Some("saying" | "noting" | "counting")
    );
    (file_name == "words.rs" && declared).then_some("the vocabulary")
}

/// English in a source file that no rule accepts: `(line, text)`.
pub fn english_in(file_name: &str, source: &str) -> Vec<(usize, String)> {
    shipped_literals(source)
        .0
        .into_iter()
        .filter(|literal| is_english(&literal.text))
        .filter(|literal| accepted_by_rule(file_name, literal).is_none())
        .map(|literal| (literal.line, literal.text))
        .collect()
}

/// This repository, from the crate this test is in.
pub fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// Every `.rs` file under this folder.
fn rust_files_under(folder: &Path, into: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(folder)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", folder.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files_under(&path, into);
        } else if path.extension().is_some_and(|it| it == "rs") {
            into.push(path);
        }
    }
}

/// Every shipped source file of this crate, as `(path under src/, text)`,
/// without the files `#[cfg(test)]` declares.
pub fn shipped_files_of(member: &str) -> Vec<(String, String)> {
    let src = the_repository().join("crates").join(member).join("src");
    let mut files = Vec::new();
    rust_files_under(&src, &mut files);
    let mut read: BTreeMap<String, String> = files
        .into_iter()
        .map(|file| {
            let under = file
                .strip_prefix(&src)
                .expect("found under src")
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&file)
                .unwrap_or_else(|why| panic!("{} could not be read: {why}", file.display()));
            (under, text)
        })
        .collect();
    let test_only: Vec<String> = read
        .iter()
        .flat_map(|(under, text)| {
            let folder = Path::new(under).parent().map(Path::to_path_buf);
            shipped_literals(text).1.into_iter().map(move |module| {
                folder
                    .clone()
                    .unwrap_or_default()
                    .join(format!("{module}.rs"))
                    .to_string_lossy()
                    .replace('\\', "/")
            })
        })
        .collect();
    for module in test_only {
        read.remove(&module);
    }
    read.into_iter().collect()
}
