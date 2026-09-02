//! A sentence with gaps in it, and the text that comes out when the gaps are
//! filled.
//!
//! Almost every string a person reads names something: *{path} is not a folder*,
//! *{path} holds {bytes} bytes and a verb reads at most {most}*. The gaps are
//! written `{name}` — the same spelling Rust's own formatting and `thiserror`
//! use, so a message being externalised moves across without being rewritten,
//! and a plain brace is written `{{` or `}}`.
//!
//! **The gaps are named and never numbered.** A positional gap gives a
//! translator nothing to move: German puts the size before the name, Irish puts
//! the verb first, and a language that reorders a sentence has to be able to
//! reorder what is in it. `{}` is refused for that reason and the refusal says
//! so.
//!
//! **A gap the caller did not fill stays visible.** It would be easy to drop
//! it, and the result would be a sentence with a hole where a file name should
//! be, in front of somebody who cannot tell that anything is wrong. So an
//! unfilled gap comes out as `{name}` and [`Filled::unfilled`] names it, which
//! is what the test for it reads.

use std::fmt;

use serde::Serialize;

use crate::filling::Filling;

/// One piece of a template: text as written, or a gap to be filled.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    /// Text, with `{{` and `}}` already turned into plain braces.
    Text(String),
    /// A gap, by name.
    Gap(String),
}

/// A sentence with named gaps in it.
///
/// It is written back out exactly as it arrived, so a translation read off a
/// disk, checked, and written out again is the same file — a translator's work
/// is not reformatted by a machine that happened to parse it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(into = "String")]
pub struct Template {
    /// The sentence, split into text and gaps.
    parts: Vec<Part>,
    /// Every gap name, in the order it first appears, without repeats.
    gaps: Vec<String>,
    /// The sentence exactly as it was written, which is what is written back
    /// out to a file.
    written: String,
}

impl Template {
    /// A template, if the sentence is one.
    ///
    /// # Errors
    ///
    /// [`TemplateError`], which says what to write instead.
    pub fn written(written: &str) -> Result<Self, TemplateError> {
        if written.trim().is_empty() {
            return Err(TemplateError::Empty);
        }
        let mut parts = Vec::new();
        let mut gaps: Vec<String> = Vec::new();
        let mut text = String::new();
        let mut characters = written.chars().peekable();
        while let Some(character) = characters.next() {
            match character {
                '{' if characters.peek() == Some(&'{') => {
                    characters.next();
                    text.push('{');
                }
                '}' if characters.peek() == Some(&'}') => {
                    characters.next();
                    text.push('}');
                }
                '}' => {
                    return Err(TemplateError::StrayBrace {
                        written: written.to_owned(),
                    });
                }
                '{' => {
                    if !text.is_empty() {
                        parts.push(Part::Text(std::mem::take(&mut text)));
                    }
                    let mut name = String::new();
                    let mut closed = false;
                    for character in characters.by_ref() {
                        if character == '}' {
                            closed = true;
                            break;
                        }
                        name.push(character);
                    }
                    if !closed {
                        return Err(TemplateError::Unclosed {
                            written: written.to_owned(),
                        });
                    }
                    check(&name)?;
                    if !gaps.iter().any(|already| already == &name) {
                        gaps.push(name.clone());
                    }
                    parts.push(Part::Gap(name));
                }
                character => text.push(character),
            }
        }
        if !text.is_empty() {
            parts.push(Part::Text(text));
        }
        Ok(Self {
            parts,
            gaps,
            written: written.to_owned(),
        })
    }

    /// Every gap in it, by name, in the order they first appear.
    #[must_use]
    pub fn gaps(&self) -> &[String] {
        &self.gaps
    }

    /// Whether this template has a gap by that name.
    #[must_use]
    pub fn has(&self, gap: &str) -> bool {
        self.gaps.iter().any(|name| name == gap)
    }

    /// The sentence exactly as it was written, gaps and all.
    #[must_use]
    pub fn as_written(&self) -> &str {
        &self.written
    }

    /// Fill the gaps and answer with the text.
    ///
    /// A gap with no value stays as `{name}` and is named in
    /// [`Filled::unfilled`]; see this module's documentation for why it is not
    /// simply dropped.
    #[must_use]
    pub fn fill(&self, filling: &Filling) -> Filled {
        let mut text = String::new();
        let mut unfilled: Vec<String> = Vec::new();
        for part in &self.parts {
            match part {
                Part::Text(written) => text.push_str(written),
                Part::Gap(name) => match filling.value(name) {
                    Some(value) => text.push_str(value),
                    None => {
                        text.push('{');
                        text.push_str(name);
                        text.push('}');
                        if !unfilled.iter().any(|already| already == name) {
                            unfilled.push(name.clone());
                        }
                    }
                },
            }
        }
        Filled { text, unfilled }
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.written)
    }
}

impl From<Template> for String {
    fn from(template: Template) -> Self {
        template.written
    }
}

/// Whether a gap is named the way gaps are named.
fn check(name: &str) -> Result<(), TemplateError> {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return Err(TemplateError::Unnamed);
    };
    if !first.is_ascii_lowercase() {
        return Err(TemplateError::BadName {
            name: name.to_owned(),
        });
    }
    for character in characters {
        if !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_') {
            return Err(TemplateError::BadName {
                name: name.to_owned(),
            });
        }
    }
    Ok(())
}

/// A template with its gaps filled in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filled {
    /// The text, ready to be shown.
    text: String,
    /// The gaps nobody gave a value for, in the order they appear.
    unfilled: Vec<String>,
}

impl Filled {
    /// The text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The gaps nobody gave a value for. Empty is the ordinary case.
    #[must_use]
    pub fn unfilled(&self) -> &[String] {
        &self.unfilled
    }

    /// Whether every gap was filled.
    #[must_use]
    pub fn is_whole(&self) -> bool {
        self.unfilled.is_empty()
    }

    /// The text, given away.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }
}

impl fmt::Display for Filled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Why something is not a template.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum TemplateError {
    /// Nothing, or only spaces.
    #[error(
        "write the sentence — an empty string reaches a person as a blank space where a sentence should have been"
    )]
    Empty,

    /// A gap that never closes.
    #[error(
        "close the gap in {written} with a }} — an opening brace that never closes would print the rest of the sentence as if it were a name"
    )]
    Unclosed {
        /// The sentence.
        written: String,
    },

    /// A closing brace with nothing opened.
    #[error(
        "write }}}} for a plain closing brace in {written} — a lone }} is read as the end of a gap"
    )]
    StrayBrace {
        /// The sentence.
        written: String,
    },

    /// `{}`.
    #[error(
        "name the gap — an unnamed one gives a translator nothing to move, and a language that puts the size before the file name has to be able to move it"
    )]
    Unnamed,

    /// A gap named something a Rust field could not be called.
    #[error(
        "call the gap {name} something in lowercase letters, digits and underscores — it is the name of the thing the sentence is about, and it is what a translator matches on"
    )]
    BadName {
        /// The name as written.
        name: String,
    },
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_sentence_with_no_gaps_is_a_template() {
        let template = Template::written("Nothing was moved.").unwrap();
        assert!(template.gaps().is_empty());
        assert_eq!(
            template.fill(&Filling::nothing()).text(),
            "Nothing was moved."
        );
    }

    #[test]
    fn the_gaps_are_named_and_come_out_in_order() {
        let template =
            Template::written("{path} holds {bytes} bytes and a verb reads at most {most}")
                .unwrap();
        assert_eq!(template.gaps(), ["path", "bytes", "most"]);
        assert!(template.has("bytes"));
        assert!(!template.has("folder"));
    }

    #[test]
    fn a_gap_used_twice_is_listed_once_and_filled_twice() {
        let template = Template::written("{path} is not a folder — read {path} as a file").unwrap();
        assert_eq!(template.gaps(), ["path"]);
        let filled = template.fill(&Filling::of("path", "/home/ada/notes"));
        assert_eq!(
            filled.text(),
            "/home/ada/notes is not a folder — read /home/ada/notes as a file"
        );
    }

    /// **An unfilled gap stays visible and is named.** A sentence that quietly
    /// dropped it would reach a person with a hole in it where the file name
    /// should be, and nothing anywhere would say so.
    #[test]
    fn an_unfilled_gap_stays_visible_and_is_named() {
        let template = Template::written("{path} holds {bytes} bytes").unwrap();
        let filled = template.fill(&Filling::of("path", "/tmp/x"));
        assert_eq!(filled.text(), "/tmp/x holds {bytes} bytes");
        assert_eq!(filled.unfilled(), ["bytes"]);
        assert!(!filled.is_whole());
    }

    #[test]
    fn a_plain_brace_is_written_twice() {
        let template = Template::written("{{{path}}} is what it is called").unwrap();
        assert_eq!(template.gaps(), ["path"]);
        assert_eq!(
            template.fill(&Filling::of("path", "notes")).text(),
            "{notes} is what it is called"
        );
    }

    /// A positional gap is refused, and the refusal is the whole reason the
    /// crate insists on names: a translator reordering a sentence has to have
    /// something to reorder by.
    #[test]
    fn a_gap_with_no_name_is_refused() {
        assert_eq!(
            Template::written("{} is not a folder"),
            Err(TemplateError::Unnamed)
        );
        assert_eq!(
            Template::written("{0} is not a folder"),
            Err(TemplateError::BadName {
                name: "0".to_owned()
            })
        );
    }

    #[test]
    fn an_unclosed_gap_and_a_stray_brace_are_refused() {
        assert!(matches!(
            Template::written("{path is not a folder"),
            Err(TemplateError::Unclosed { .. })
        ));
        assert!(matches!(
            Template::written("the archive is written to } there"),
            Err(TemplateError::StrayBrace { .. })
        ));
    }

    #[test]
    fn nothing_and_only_spaces_are_refused() {
        assert_eq!(Template::written(""), Err(TemplateError::Empty));
        assert_eq!(Template::written("   "), Err(TemplateError::Empty));
    }

    #[test]
    fn a_gap_named_like_a_sentence_is_refused() {
        for written in ["{Path}", "{the path}", "{path-name}"] {
            assert!(
                matches!(
                    Template::written(written),
                    Err(TemplateError::BadName { .. })
                ),
                "{written}"
            );
        }
    }

    /// A template is written back out exactly as it arrived, so a translation
    /// read off a disk, checked, and written out again is the same file.
    #[test]
    fn a_template_is_written_back_as_it_arrived() {
        let written = "{path} holds {bytes} bytes — {{that}} is a lot";
        assert_eq!(Template::written(written).unwrap().to_string(), written);
    }
}
