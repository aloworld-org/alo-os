//! No English is written in the five crates that keep a person's settings,
//! outside the vocabulary `alo-strings` answers from.
//!
//! `alo-appearance`, `alo-dock`, `alo-shortcuts`, `alo-choosing` and
//! `alo-changing` are what a person's own folder is read and written by, and
//! what they are told when a file there did not read. `CLAUDE.md` calls
//! hardcoded English a bug; every crate's own tests hold its declared list, and
//! none of them can see a sentence written somewhere that is **not** on the
//! list — a `format!` in a refusal, an `#[error]` somebody reached for, a
//! string a surface will one day print. So this reads the shipped source of all
//! five for one.
//!
//! # What the search reads, and what it does not
//!
//! **Shipped means `src/`**, without `#[cfg(test)]` items or the modules they
//! declare: a test's English is read by whoever runs the test. Comments and doc
//! comments are skipped, because this repository argues in prose. A string
//! literal is **English** when it holds two ordinary words side by side — which
//! a key (`appearance.kept.not-read`), a serde name (`Written`), a gap
//! (`path`) and a mark printed on a key (`F12`) never do. A single word is not
//! caught, and cannot be told from an identifier by reading; the labels that are
//! one word are held by each crate's own list tests instead.
//!
//! Three places English is **meant** to be, and the search accepts by rule:
//!
//! | Where | Why it is not English on a screen |
//! |---|---|
//! | `saying`, `noting` and `counting` in a crate's `src/words.rs` | It is the vocabulary: the source text and the translator's note, declared into `alo-strings` |
//! | `#[expect]` and `#[allow]` | A lint's reason, read by a reviewer |
//! | `#[doc]` | Rustdoc |
//!
//! Anything else is refused unless it is on [`NOT_READ_BY_A_PERSON`] with the
//! argument for it — English that reaches a log or a developer and never a
//! screen. That list is checked in both directions: an entry the source no
//! longer holds is refused as well, because a list with a dead line in it is a
//! list nobody reads as a check.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The five crates whose source is read: the ones the plan for where a
/// person's settings are kept owns (`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`).
const THE_FIVE: [&str; 5] = [
    "alo-appearance",
    "alo-dock",
    "alo-shortcuts",
    "alo-choosing",
    "alo-changing",
];

/// English in shipped source that no person using the machine reads, each
/// with the reason — `(crate, file under src/, a fragment of the literal,
/// why)`.
const NOT_READ_BY_A_PERSON: &[(&str, &str, &str, &str)] = &[
    (
        "alo-changing",
        "refusing.rs",
        "the machine refused the grant",
        "the Display of NotChanged, for a log; a person is told NotChanged::said",
    ),
    (
        "alo-changing",
        "refusing.rs",
        "the grants were not kept",
        "the Display of NotChanged, for a log; a person is told NotChanged::said",
    ),
    (
        "alo-changing",
        "refusing.rs",
        "refused to revoke the pairing",
        "the Display of NotChanged, for a log; a person is told NotChanged::said",
    ),
    (
        "alo-changing",
        "refusing.rs",
        "no agent service was listening",
        "the Display of NotChanged, for a log; a person is told NotChanged::said",
    ),
    (
        "alo-changing",
        "refusing.rs",
        "did not say whether the pairing was revoked",
        "the Display of NotChanged, for a log; a person is told NotChanged::said",
    ),
    (
        "alo-shortcuts",
        "defaults.rs",
        "is in the defaults twice",
        "DefaultsError is the release's own list contradicting itself, a defect in this repository",
    ),
    (
        "alo-shortcuts",
        "defaults.rs",
        "more than one default is on",
        "DefaultsError is the release's own list contradicting itself, a defect in this repository",
    ),
    (
        "alo-choosing",
        "writing.rs",
        "were different settings",
        "NotWritten::NotExpressible's detail for whoever fixes alo OS; a person reads choosing.change.not-expressible",
    ),
    (
        "alo-choosing",
        "writing.rs",
        "was not settings this alo OS reads",
        "NotWritten::NotExpressible's detail for whoever fixes alo OS; a person reads choosing.change.not-expressible",
    ),
];

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
struct Literal {
    /// What it says.
    text: String,
    /// The line it starts on, counted from one.
    line: usize,
    /// The attribute it is inside, when it is inside one.
    attribute: Option<String>,
    /// The function or macro whose parentheses it is inside, nearest first.
    callee: Option<String>,
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
fn shipped_literals(source: &str) -> (Vec<Literal>, Vec<String>) {
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
fn is_english(text: &str) -> bool {
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
fn english_in(file_name: &str, source: &str) -> Vec<(usize, String)> {
    shipped_literals(source)
        .0
        .into_iter()
        .filter(|literal| is_english(&literal.text))
        .filter(|literal| accepted_by_rule(file_name, literal).is_none())
        .map(|literal| (literal.line, literal.text))
        .collect()
}

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
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

/// Every shipped source file of one of the five, as `(path under src/, text)`,
/// without the files `#[cfg(test)]` declares.
fn shipped_files_of(member: &str) -> Vec<(String, String)> {
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

/// **No shipped source of the five crates writes English outside the
/// vocabulary**, except the English on [`NOT_READ_BY_A_PERSON`] — and every
/// entry on that list is still English somewhere, so the list cannot rot into
/// permission nobody needs.
#[test]
fn the_five_crates_write_no_english_outside_the_vocabulary() {
    let mut unexplained = Vec::new();
    let mut explained = vec![false; NOT_READ_BY_A_PERSON.len()];
    for member in THE_FIVE {
        for (under, source) in shipped_files_of(member) {
            let file_name = under.rsplit('/').next().unwrap_or(&under).to_owned();
            for (line, text) in english_in(&file_name, &source) {
                let explaining =
                    NOT_READ_BY_A_PERSON
                        .iter()
                        .position(|(crate_, file, fragment, _)| {
                            *crate_ == member && *file == under && text.contains(fragment)
                        });
                match explaining {
                    Some(entry) => {
                        if let Some(seen) = explained.get_mut(entry) {
                            *seen = true;
                        }
                    }
                    None => unexplained.push(format!("{member}/src/{under}:{line}: {text:?}")),
                }
            }
        }
    }
    assert!(
        unexplained.is_empty(),
        "English a person could read, written outside alo-strings — declare it in the crate's \
         words.rs, or argue it onto NOT_READ_BY_A_PERSON: {unexplained:#?}"
    );
    let stale: Vec<_> = NOT_READ_BY_A_PERSON
        .iter()
        .zip(&explained)
        .filter(|(_, seen)| !**seen)
        .map(|(entry, _)| entry)
        .collect();
    assert!(
        stale.is_empty(),
        "an exception the source no longer needs: {stale:#?}"
    );
}

/// **Every exception carries its argument.** A fragment with a shrug beside it
/// is English a person might read, with permission.
#[test]
fn every_exception_says_why_nobody_reads_it() {
    for (member, file, fragment, why) in NOT_READ_BY_A_PERSON {
        assert!(THE_FIVE.contains(member), "{member} is not one of the five");
        assert!(
            is_english(fragment),
            "{fragment:?} is not what the search finds"
        );
        assert!(
            why.split_whitespace().count() >= 6,
            "{member}/src/{file}: {fragment:?} has no argument beside it"
        );
    }
}

/// **The search reads what it says it reads.** Every sentence each of the
/// five crates declares is found in its `words.rs` as the vocabulary — one
/// `saying` per phrase in the machine's vocabulary under that crate's area — so
/// the test above finding nothing is a measurement rather than a search that
/// looked nowhere.
#[test]
fn the_search_finds_every_crates_declared_vocabulary() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for member in THE_FIVE {
        let area = member.trim_start_matches("alo-");
        let declared = vocabulary
            .phrases()
            .filter(|phrase| phrase.key().area() == area)
            .count();
        assert!(
            declared > 0,
            "{member} declares nothing the machine collects"
        );
        let (literals, _) = shipped_files_of(member)
            .into_iter()
            .find(|(under, _)| under == "words.rs")
            .map(|(_, source)| shipped_literals(&source))
            .unwrap_or_else(|| panic!("{member} has no src/words.rs"));
        let sayings = literals
            .iter()
            .filter(|literal| literal.callee.as_deref() == Some("saying"))
            .count();
        assert_eq!(
            sayings,
            declared * 2,
            "{member}: a key and a sentence for each of its {declared} phrases"
        );
    }
}

/// **A sentence written anywhere else is found**, however it is spelt — held
/// against text, so the rule is shown refusing without planting English in the
/// repository.
#[test]
fn a_sentence_outside_the_vocabulary_is_found_however_it_is_spelt() {
    let plain = "fn f() -> String { format!(\"your settings could not be read\") }";
    assert_eq!(english_in("keeping.rs", plain).len(), 1);

    let raw = "const SAID: &str = r#\"the \"file\" did not read\"#;";
    assert_eq!(
        english_in("keeping.rs", raw),
        [(1, "the \"file\" did not read".to_owned())]
    );

    let continued = "fn f() {\n    let _ = \"nothing in the \\\n        file was used\";\n}";
    assert_eq!(
        english_in("keeping.rs", continued),
        [(2, "nothing in the file was used".to_owned())]
    );

    let displayed = "#[derive(thiserror::Error)]\nenum E {\n    #[error(\"the file was wrong\")]\n    Wrong,\n}";
    assert_eq!(
        english_in("unkept.rs", displayed).len(),
        1,
        "an #[error] is English"
    );
}

/// **The vocabulary is accepted only where it is declared.** The same call in
/// another file is English written outside the list, and a stray sentence in
/// `words.rs` is not the vocabulary because of the file it is in.
#[test]
fn the_vocabulary_is_accepted_in_words_rs_and_nowhere_else() {
    let declared = "pub const W: Word = Word::saying(\"dock.kept.not-read\", \"your dock settings could not be read\")\n    .noting(\"{path} is a file on this machine\");";
    assert!(english_in("words.rs", declared).is_empty());
    assert_eq!(english_in("unkept.rs", declared).len(), 2);

    let stray = "pub fn f() -> String { format!(\"the list did not declare\") }";
    assert_eq!(english_in("words.rs", stray).len(), 1);
}

/// **What is not a sentence is not found**: comments, doc comments, a lint's
/// reason, rustdoc, keys, serde names, a mark on a key, a gap's name, and a
/// character literal that is a quotation mark.
#[test]
fn what_is_not_a_sentence_on_a_screen_is_not_found() {
    let source = r#"
//! What this file says in prose, which is not "a sentence on a screen".
/// A doc comment "with a quotation in it".
#[expect(clippy::unwrap_used, reason = "in a test, a panic is the failure")]
#[doc = concat!("The ", "A", " key.")]
#[serde(try_from = "Written", into = "Written")]
struct S;
/* a block comment "with words in it" /* nested "here" */ still comment */
fn f(c: char) -> bool {
    let _ = ("appearance.kept.not-read", "F12", "path", "format = 1\n");
    c == '"' || c == '\'' && lifetime::<'static>()
}
"#;
    assert_eq!(
        english_in("keeping.rs", source),
        Vec::<(usize, String)>::new()
    );
}

/// **A `#[cfg(test)]` item is not shipped, and the code after it is.** A test
/// module's English is read by whoever runs the tests; a module it declares by
/// name is a file that is not shipped either.
#[test]
fn a_test_module_is_skipped_and_what_follows_it_is_not() {
    let source = "#[cfg(test)]\nmod tests {\n    fn t() { let _ = \"a sentence in a test\"; }\n}\n\nfn shipped() { let _ = \"a sentence that ships\"; }\n";
    assert_eq!(
        english_in("keeping.rs", source),
        [(6, "a sentence that ships".to_owned())]
    );

    let declared = "#[cfg(test)]\nmod testing;\n\npub mod keeping;\n";
    assert_eq!(shipped_literals(declared).1, ["testing"]);
}
