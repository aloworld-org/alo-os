//! One argument, as the record keeps it.
//!
//! A mirror of [`Value`], and deliberately a separate type rather than a reuse
//! of it. `Value` exists because an argument was validated against the verb
//! that declared it, and it has no `Deserialize` for exactly that reason: one
//! read back off a disk would have skipped the only step that makes it
//! trustworthy. The record has to read its own entries back, so it needs a type
//! whose meaning is *this is what was written down* rather than *this may be
//! acted on*.
//!
//! The two are kept in step by [`From<&Value>`], whose match is exhaustive on
//! purpose: a new kind of argument does not compile until somebody has decided
//! how the record keeps it. That is cheaper than discovering later that the
//! record has been quietly losing an argument nobody thought about.
//!
//! There is no way back. Nothing here turns into a `Value`, a call or anything
//! that runs — **a record is evidence, not an instruction.** A record that
//! could be replayed would be a second way to cause an execution, answerable to
//! no approval and no grant.

use std::path::{Path, PathBuf};

use alo_capability::Value;
use serde::{Deserialize, Serialize};

use crate::line::Line;

/// One validated argument, written down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Written {
    /// A full path on this machine.
    Path(PathBuf),
    /// An application identifier.
    Application(Line),
    /// One name.
    Name(Line),
    /// A whole number.
    Count(i64),
    /// One of the options the verb declared.
    Choice(Line),
}

impl Written {
    /// What this was, in words that are safe to show.
    ///
    /// A path goes through [`Line`] here rather than when it is kept, because
    /// the record keeps the path as it was — evidence of what was touched —
    /// while what is *shown* has to be one readable line.
    #[must_use]
    pub fn describe(&self) -> Line {
        match self {
            Self::Path(path) => Line::of(&path.display().to_string()),
            Self::Application(id) | Self::Name(id) | Self::Choice(id) => id.clone(),
            Self::Count(number) => Line::of(&number.to_string()),
        }
    }

    /// The path this argument was, when it was one.
    ///
    /// What a security review asks first is which files were touched, and
    /// answering it by parsing a description would be answering it by guessing.
    #[must_use]
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(path),
            Self::Application(_) | Self::Name(_) | Self::Count(_) | Self::Choice(_) => None,
        }
    }
}

impl From<&Value> for Written {
    /// Exhaustive on purpose: a new kind of argument will not compile until
    /// somebody has decided how the record keeps it.
    fn from(value: &Value) -> Self {
        match value {
            Value::Path(path) => Self::Path(path.clone()),
            Value::Application(id) => Self::Application(Line::of(id)),
            Value::Name(name) => Self::Name(Line::of(name)),
            Value::Count(number) => Self::Count(*number),
            Value::Choice(option) => Self::Choice(Line::of(option)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_capability::{Arg, Given, Takes};
    use alo_strings::Word;

    /// What an argument is for, which none of these tests is about — they are
    /// about the value that came out the other side of validation.
    const A_PURPOSE: Word = Word::saying("testing.argument.purpose", "whatever it is for");

    fn validated(arg: &Arg, given: &Given) -> Option<Value> {
        arg.validate(given).ok()
    }

    /// Every kind of argument there is survives being written down, so no call
    /// is recorded with a hole where one of its arguments was.
    #[test]
    fn every_kind_of_argument_can_be_written_down() {
        let cases = [
            (
                Arg::taking("file", A_PURPOSE, Takes::Path),
                Given::text("/home/anna/Invoices/march.pdf"),
                "/home/anna/Invoices/march.pdf",
            ),
            (
                Arg::taking("application", A_PURPOSE, Takes::Application),
                Given::text("org.blender.Blender"),
                "org.blender.Blender",
            ),
            (
                Arg::taking("name", A_PURPOSE, Takes::name(255)),
                Given::text("april.pdf"),
                "april.pdf",
            ),
            (
                Arg::taking("lines", A_PURPOSE, Takes::count(1, 500)),
                Given::number(7),
                "7",
            ),
            (
                Arg::taking("into", A_PURPOSE, Takes::choice(["archive", "trash"])),
                Given::text("archive"),
                "archive",
            ),
        ];
        for (arg, given, shown) in &cases {
            let value = validated(arg, given);
            let written = value.as_ref().map(Written::from);
            assert_eq!(
                written.as_ref().map(Written::describe),
                Some(Line::of(shown)),
                "{arg:?}"
            );
        }
    }

    /// Which files were touched is the first question a security review asks,
    /// so a path is kept as a path rather than as a description of one.
    #[test]
    fn a_path_is_kept_as_a_path() {
        let file = Arg::taking("file", A_PURPOSE, Takes::Path);
        let written = validated(&file, &Given::text("/home/anna/Invoices/march.pdf"))
            .as_ref()
            .map(Written::from);
        assert_eq!(
            written.as_ref().and_then(Written::as_path),
            Some(Path::new("/home/anna/Invoices/march.pdf"))
        );
        assert_eq!(Written::Count(7).as_path(), None);
    }

    /// The record is read back, so what was written has to survive the journey
    /// unchanged.
    #[test]
    fn an_argument_survives_being_written_down_and_read_back() {
        let written = Written::Path(PathBuf::from("/home/anna/Invoices/march.pdf"));
        let text = serde_json::to_string(&written).unwrap_or_default();
        assert_eq!(serde_json::from_str::<Written>(&text).ok(), Some(written));
    }
}
