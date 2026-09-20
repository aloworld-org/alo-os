//! What the verb and the converting service say to each other.
//!
//! ADR 0039 §A: *a second hand-written protocol, descriptors in and a report
//! out, kept to one file*. This is that file for the words; `passing.rs` is the
//! one that carries the two descriptors beside them.
//!
//! # The whole of it
//!
//! A request is one line, and a convert request arrives with exactly two open
//! descriptors — the original, read-only, and the copy, created empty:
//!
//! ```text
//! alo-converting 1 ready
//! alo-converting 1 convert word-document
//! ```
//!
//! An answer is lines ending in `end`:
//!
//! ```text
//! ready                     converted everything      converted not-everything
//! end                       end                       lost font Garamond
//!                                                     lost field date
//! refused too-large                                   lost linked picture
//! end                                                 lost comments
//!                                                     end
//! ```
//!
//! **No path crosses it.** The service is handed descriptors and cannot open a
//! name, and nothing in a request is a name. The one thing that crosses from a
//! document into an answer is a font's name, which [`FontName`] has already
//! made one printable line.
//!
//! **A reply that does not agree with itself is no reply**: `everything` with a
//! loss after it, or `not-everything` with none, reads as [`None`] and the verb
//! says the service did not answer rather than choosing which half to believe.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::carried::{Carried, Field, FontName, Linked, NotCarried};
use crate::conversion::Conversion;

/// What every request begins with: this protocol, and its version.
pub const SPOKEN: &str = "alo-converting 1";

/// The longest request line, in bytes.
pub const LONGEST_REQUEST: usize = 128;

/// The longest answer, in bytes.
pub const LONGEST_ANSWER: usize = 64 * 1024;

/// What the verb asks the service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// Whether it is there.
    Ready,
    /// Convert the document on the first descriptor into the copy on the
    /// second.
    Convert(Conversion),
}

/// What the service answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// It is there.
    Ready,
    /// The copy is written, and this is what it carried.
    Converted(Carried),
    /// Nothing was written to the copy, and this is why.
    Refused(Refusal),
}

/// Why the service wrote nothing to the copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Refusal {
    /// The request was not one it understands.
    NotUnderstood,
    /// The original is not the kind of document the request named.
    NotTheKindAsked,
    /// The original, or its copy, is larger than the service converts.
    TooLarge,
    /// The original could not be inventoried, so no copy was made.
    OriginalNotChecked,
    /// The engine did not produce a copy.
    CouldNotConvert,
    /// The engine took too long and was stopped.
    TooSlow,
    /// The copy could not be inventoried, so it was not written.
    CopyNotChecked,
    /// The copy could not be written.
    CopyNotWritten,
}

impl Refusal {
    /// Every refusal, in one order.
    pub const EVERY: [Self; 8] = [
        Self::NotUnderstood,
        Self::NotTheKindAsked,
        Self::TooLarge,
        Self::OriginalNotChecked,
        Self::CouldNotConvert,
        Self::TooSlow,
        Self::CopyNotChecked,
        Self::CopyNotWritten,
    ];

    /// How it is written on the wire.
    const fn written(self) -> &'static str {
        match self {
            Self::NotUnderstood => "not-understood",
            Self::NotTheKindAsked => "not-the-kind-asked",
            Self::TooLarge => "too-large",
            Self::OriginalNotChecked => "original-not-checked",
            Self::CouldNotConvert => "could-not-convert",
            Self::TooSlow => "too-slow",
            Self::CopyNotChecked => "copy-not-checked",
            Self::CopyNotWritten => "copy-not-written",
        }
    }
}

impl Request {
    /// The request as one line.
    #[must_use]
    pub fn written(self) -> String {
        match self {
            Self::Ready => format!("{SPOKEN} ready\n"),
            Self::Convert(conversion) => format!("{SPOKEN} convert {}\n", conversion.asked_as()),
        }
    }

    /// A request read from its line; [`None`] for anything else.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let asked = line
            .strip_suffix('\n')?
            .strip_prefix(SPOKEN)?
            .strip_prefix(' ')?;
        if asked == "ready" {
            return Some(Self::Ready);
        }
        Conversion::asked(asked.strip_prefix("convert ")?).map(Self::Convert)
    }
}

impl Answer {
    /// The answer as it is sent.
    #[must_use]
    pub fn written(&self) -> String {
        let mut written = String::new();
        match self {
            Self::Ready => written.push_str("ready\n"),
            Self::Refused(refusal) => {
                let _infallible = writeln!(written, "refused {}", refusal.written());
            }
            Self::Converted(Carried::Everything) => written.push_str("converted everything\n"),
            Self::Converted(carried @ Carried::NotEverything(_)) => {
                written.push_str("converted not-everything\n");
                for not in carried.not_carried() {
                    let _infallible = writeln!(written, "lost {}", lost(not));
                }
            }
        }
        written.push_str("end\n");
        written
    }

    /// An answer read from what was sent; [`None`] for anything that is not
    /// one, or that does not agree with itself.
    #[must_use]
    pub fn read(sent: &str) -> Option<Self> {
        let mut lines = sent.strip_suffix("end\n")?.lines();
        let first = lines.next()?;
        let rest: Vec<&str> = lines.collect();
        match first {
            "ready" if rest.is_empty() => Some(Self::Ready),
            "converted everything" if rest.is_empty() => Some(Self::Converted(Carried::Everything)),
            "converted not-everything" if !rest.is_empty() => {
                let mut not = BTreeSet::new();
                for line in rest {
                    not.insert(read_lost(line.strip_prefix("lost ")?)?);
                }
                Some(Self::Converted(Carried::of(not)))
            }
            _ => {
                let refused = first.strip_prefix("refused ")?;
                if !rest.is_empty() {
                    return None;
                }
                Refusal::EVERY
                    .into_iter()
                    .find(|refusal| refusal.written() == refused)
                    .map(Self::Refused)
            }
        }
    }
}

/// One thing not carried, as written after `lost `.
fn lost(not: &NotCarried) -> String {
    match not {
        NotCarried::FontSubstituted(font) => format!("font {}", font.as_str()),
        NotCarried::FieldFixed(field) => format!("field {}", field_written(*field)),
        NotCarried::Macros => "macros".to_owned(),
        NotCarried::LinkedNotFetched(linked) => format!("linked {}", linked_written(*linked)),
        NotCarried::Comments => "comments".to_owned(),
        NotCarried::TrackedChanges => "tracked-changes".to_owned(),
    }
}

/// One thing not carried, read from after `lost `.
fn read_lost(written: &str) -> Option<NotCarried> {
    if let Some(font) = written.strip_prefix("font ") {
        let name = FontName::from_document(font)?;
        return (name.as_str() == font).then_some(NotCarried::FontSubstituted(name));
    }
    if let Some(field) = written.strip_prefix("field ") {
        return Field::EVERY
            .into_iter()
            .find(|known| field_written(*known) == field)
            .map(NotCarried::FieldFixed);
    }
    if let Some(linked) = written.strip_prefix("linked ") {
        return Linked::EVERY
            .into_iter()
            .find(|known| linked_written(*known) == linked)
            .map(NotCarried::LinkedNotFetched);
    }
    match written {
        "macros" => Some(NotCarried::Macros),
        "comments" => Some(NotCarried::Comments),
        "tracked-changes" => Some(NotCarried::TrackedChanges),
        _ => None,
    }
}

/// A field kind on the wire.
const fn field_written(field: Field) -> &'static str {
    match field {
        Field::Date => "date",
        Field::Time => "time",
        Field::FileName => "file-name",
        Field::Author => "author",
        Field::MergeField => "merge-field",
        Field::TheCurrentMoment => "current-moment",
        Field::ARandomNumber => "random-number",
    }
}

/// A kind of linked content on the wire.
const fn linked_written(linked: Linked) -> &'static str {
    match linked {
        Linked::Picture => "picture",
        Linked::Data => "data",
        Linked::Template => "template",
        Linked::SomethingElse => "something-else",
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Every request and every answer reads back as itself.**
    #[test]
    fn everything_said_reads_back_as_itself() {
        let mut requests = vec![Request::Ready];
        requests.extend(Conversion::EVERY.map(Request::Convert));
        for request in requests {
            let written = request.written();
            assert!(written.len() <= LONGEST_REQUEST);
            assert_eq!(Request::read(&written), Some(request));
        }

        let mut not = BTreeSet::from([
            NotCarried::FontSubstituted(FontName::from_document("Aptos Display").unwrap()),
            NotCarried::Macros,
            NotCarried::Comments,
            NotCarried::TrackedChanges,
        ]);
        not.extend(Field::EVERY.map(NotCarried::FieldFixed));
        not.extend(Linked::EVERY.map(NotCarried::LinkedNotFetched));
        let mut answers = vec![
            Answer::Ready,
            Answer::Converted(Carried::Everything),
            Answer::Converted(Carried::of(not)),
        ];
        answers.extend(Refusal::EVERY.map(Answer::Refused));
        for answer in answers {
            assert_eq!(Answer::read(&answer.written()), Some(answer));
        }
    }

    /// **The word of a held-back conversion is not a request.**
    ///
    /// It can be written — the line is decided, and the day the conversion is
    /// offered nothing on the wire changes — and the service does not read it
    /// back, so no request can ask for a conversion whose original has never
    /// been inventoried. The asymmetry is the whole of what holding one back
    /// means, and this is where it is visible.
    #[test]
    fn a_held_back_conversion_can_be_written_and_is_not_read_back() {
        for held in Conversion::HELD_BACK {
            let written = Request::Convert(held).written();
            assert!(written.contains(held.asked_as()), "{written}");
            assert!(written.len() <= LONGEST_REQUEST);
            assert_eq!(
                Request::read(&written),
                None,
                "the service answered to {held:?}, whose original it cannot inventory"
            );
        }
    }

    /// **No path crosses the wire**: a request is one of four lines.
    #[test]
    fn no_request_carries_a_path_or_anything_else() {
        for sent in [
            "alo-converting 1 convert /home/anna/report.docx\n",
            "alo-converting 1 convert word-document /etc/shadow\n",
            "alo-converting 2 ready\n",
            "alo-converting 1 ready",
            "alo-converting 1 run the-engine\n",
        ] {
            assert_eq!(Request::read(sent), None, "{sent:?}");
        }
    }

    /// **An answer that does not agree with itself is no answer**, and a lost
    /// thing nobody knows is not guessed at.
    #[test]
    fn an_answer_that_contradicts_itself_is_no_answer() {
        for sent in [
            "converted everything\nlost comments\nend\n",
            "converted not-everything\nend\n",
            "converted not-everything\nlost the-weather\nend\n",
            "converted not-everything\nlost field tomorrow\nend\n",
            "converted not-everything\nlost font  Padded \nend\n",
            "refused too-large\nlost comments\nend\n",
            "refused by-choice\nend\n",
            "ready\n",
            "",
        ] {
            assert_eq!(Answer::read(sent), None, "{sent:?}");
        }
    }
}
