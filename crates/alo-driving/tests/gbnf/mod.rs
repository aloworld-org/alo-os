//! A reader for the subset of GBNF `alo_driving::grammar_for` writes.
//!
//! Only what that function emits: a rule per line, alternation, sequence,
//! groups, quoted literals with `\"` and `\\x` escapes, character classes with
//! ranges and negation, and the postfixes `*` and `?`. Anything else is a panic
//! rather than a quiet acceptance, because a matcher that skipped what it did
//! not understand would pass every test by not reading the grammar.
//!
//! `llama.cpp` is the authority on GBNF; this says what the grammar means in the
//! subset it is written in. `tests/the_grammar_holds_every_call_this_machine_accepts.rs`
//! says why that is the right division.

use std::collections::HashMap;

/// One term of a sequence.
#[derive(Debug, Clone)]
enum Term {
    /// Exactly this text.
    Text(String),
    /// Whatever that rule matches.
    Rule(String),
    /// Any of these.
    Either(Vec<Vec<Term>>),
    /// One character in, or not in, this set.
    Class {
        /// The ranges, each a pair of ends, both included.
        ranges: Vec<(char, char)>,
        /// Whether the set is what it may **not** be.
        negated: bool,
    },
    /// The term, as many times as it goes, including none.
    Any(Box<Term>),
    /// The term, or nothing.
    Maybe(Box<Term>),
    /// The term, at least once.
    Some(Box<Term>),
}

/// A grammar, read.
pub struct Grammar {
    /// Every rule by name, each a list of alternatives.
    rules: HashMap<String, Vec<Vec<Term>>>,
}

impl Grammar {
    /// Read a grammar. Panics on anything outside the subset above.
    pub fn of(text: &str) -> Self {
        let mut rules = HashMap::new();
        let mut lines: Vec<String> = Vec::new();
        // A rule may be written over more than one line; a new one starts where
        // `::=` appears.
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            if line.contains("::=") {
                lines.push(line.to_owned());
            } else {
                let last = lines.last_mut().expect("a continuation before any rule");
                last.push(' ');
                last.push_str(line.trim());
            }
        }
        for line in lines {
            let (name, body) = line.split_once("::=").expect("a rule with no ::=");
            let alternatives = Reading::of(body).alternatives();
            rules.insert(name.trim().to_owned(), alternatives);
        }
        assert!(rules.contains_key("root"), "a grammar with no root");
        Self { rules }
    }

    /// Whether the grammar matches the whole of this text.
    pub fn accepts(&self, text: &str) -> bool {
        let start: Vec<char> = text.chars().collect();
        self.alternatives("root")
            .iter()
            .any(|sequence| self.sequence(sequence, &start).contains(&start.len()))
    }

    /// The alternatives of a rule, or a panic naming the rule nothing declares.
    fn alternatives(&self, name: &str) -> &Vec<Vec<Term>> {
        self.rules
            .get(name)
            .unwrap_or_else(|| panic!("the grammar names a rule it does not declare: {name}"))
    }

    /// Every length of `text` one sequence can consume from the start.
    fn sequence(&self, sequence: &[Term], text: &[char]) -> Vec<usize> {
        let mut reached = vec![0_usize];
        for term in sequence {
            let mut next: Vec<usize> = Vec::new();
            for at in reached {
                let Some(rest) = text.get(at..) else {
                    continue;
                };
                for grew in self.term(term, rest) {
                    let to = at + grew;
                    if !next.contains(&to) {
                        next.push(to);
                    }
                }
            }
            if next.is_empty() {
                return Vec::new();
            }
            reached = next;
        }
        reached
    }

    /// Every length one term can consume from the start.
    fn term(&self, term: &Term, text: &[char]) -> Vec<usize> {
        match term {
            Term::Text(want) => {
                let wanted: Vec<char> = want.chars().collect();
                match text.get(..wanted.len()) {
                    Some(start) if start == wanted => vec![wanted.len()],
                    _ => Vec::new(),
                }
            }
            Term::Rule(name) => self
                .alternatives(name)
                .iter()
                .flat_map(|sequence| self.sequence(sequence, text))
                .collect(),
            Term::Either(alternatives) => alternatives
                .iter()
                .flat_map(|sequence| self.sequence(sequence, text))
                .collect(),
            Term::Class { ranges, negated } => match text.first() {
                Some(letter) => {
                    let inside = ranges
                        .iter()
                        .any(|(from, to)| letter >= from && letter <= to);
                    if inside != *negated {
                        vec![1]
                    } else {
                        Vec::new()
                    }
                }
                None => Vec::new(),
            },
            Term::Maybe(inner) => {
                let mut lengths = vec![0];
                lengths.extend(self.term(inner, text));
                lengths
            }
            Term::Any(inner) | Term::Some(inner) => {
                let mut lengths: Vec<usize> = if matches!(term, Term::Any(_)) {
                    vec![0]
                } else {
                    Vec::new()
                };
                let mut edge: Vec<usize> = vec![0];
                loop {
                    let mut grown: Vec<usize> = Vec::new();
                    for at in edge {
                        let Some(rest) = text.get(at..) else {
                            continue;
                        };
                        for more in self.term(inner, rest) {
                            let to = at + more;
                            if more > 0 && !lengths.contains(&to) && !grown.contains(&to) {
                                grown.push(to);
                            }
                        }
                    }
                    if grown.is_empty() {
                        return lengths;
                    }
                    lengths.extend(grown.iter().copied());
                    edge = grown;
                }
            }
        }
    }
}

/// One body, being read left to right.
struct Reading {
    /// What is left of it.
    letters: Vec<char>,
    /// Where the reader is.
    at: usize,
}

impl Reading {
    /// Read this body.
    fn of(body: &str) -> Self {
        Self {
            letters: body.chars().collect(),
            at: 0,
        }
    }

    /// Every alternative in it.
    fn alternatives(&mut self) -> Vec<Vec<Term>> {
        let mut all = vec![Vec::new()];
        loop {
            self.spaces();
            match self.letters.get(self.at) {
                None | Some(')') => return all,
                Some('|') => {
                    self.at += 1;
                    all.push(Vec::new());
                }
                Some(_) => {
                    let term = self.term();
                    all.last_mut().expect("an alternative to add to").push(term);
                }
            }
        }
    }

    /// One term, with whatever postfix follows it.
    fn term(&mut self) -> Term {
        let term = match *self.letters.get(self.at).expect("a term to read") {
            '"' => Term::Text(self.text()),
            '(' => {
                self.at += 1;
                let inside = self.alternatives();
                assert_eq!(self.letters.get(self.at), Some(&')'), "a group with no end");
                self.at += 1;
                Term::Either(inside)
            }
            '[' => self.class(),
            letter if letter.is_alphanumeric() || letter == '-' || letter == '_' => {
                Term::Rule(self.name())
            }
            other => panic!("this reader does not know `{other}` in a grammar"),
        };
        match self.letters.get(self.at) {
            Some('*') => {
                self.at += 1;
                Term::Any(Box::new(term))
            }
            Some('?') => {
                self.at += 1;
                Term::Maybe(Box::new(term))
            }
            Some('+') => {
                self.at += 1;
                Term::Some(Box::new(term))
            }
            _ => term,
        }
    }

    /// A quoted literal, with its escapes read.
    fn text(&mut self) -> String {
        self.at += 1;
        let mut said = String::new();
        loop {
            let letter = *self
                .letters
                .get(self.at)
                .expect("a quoted literal with no end");
            self.at += 1;
            match letter {
                '"' => return said,
                '\\' => said.push(self.escape()),
                other => said.push(other),
            }
        }
    }

    /// One escape, after the backslash.
    fn escape(&mut self) -> char {
        let letter = *self
            .letters
            .get(self.at)
            .expect("an escape with nothing after it");
        self.at += 1;
        match letter {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'x' => {
                let hex: String = self
                    .letters
                    .get(self.at..self.at + 2)
                    .expect("two hexadecimal digits")
                    .iter()
                    .collect();
                self.at += 2;
                char::from_u32(u32::from_str_radix(&hex, 16).expect("two hexadecimal digits"))
                    .expect("a character")
            }
            other => other,
        }
    }

    /// A character class, `[...]`, with ranges and an optional `^`.
    fn class(&mut self) -> Term {
        self.at += 1;
        let negated = self.letters.get(self.at) == Some(&'^');
        if negated {
            self.at += 1;
        }
        let mut ranges: Vec<(char, char)> = Vec::new();
        loop {
            let letter = *self.letters.get(self.at).expect("a class with no end");
            self.at += 1;
            match letter {
                ']' => return Term::Class { ranges, negated },
                first => {
                    let first = if first == '\\' { self.escape() } else { first };
                    if self.letters.get(self.at) == Some(&'-')
                        && self.letters.get(self.at + 1) != Some(&']')
                    {
                        self.at += 1;
                        let last = *self.letters.get(self.at).expect("an end to the range");
                        self.at += 1;
                        let last = if last == '\\' { self.escape() } else { last };
                        ranges.push((first, last));
                    } else {
                        ranges.push((first, first));
                    }
                }
            }
        }
    }

    /// A rule's name.
    fn name(&mut self) -> String {
        let from = self.at;
        while self
            .letters
            .get(self.at)
            .is_some_and(|letter| letter.is_alphanumeric() || *letter == '-' || *letter == '_')
        {
            self.at += 1;
        }
        self.letters
            .get(from..self.at)
            .expect("a name that was just read")
            .iter()
            .collect()
    }

    /// Whatever spaces are next.
    fn spaces(&mut self) {
        while self
            .letters
            .get(self.at)
            .is_some_and(|letter| letter.is_whitespace())
        {
            self.at += 1;
        }
    }
}
