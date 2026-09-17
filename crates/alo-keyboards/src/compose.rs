//! The rented compose table, read: which sequence of keys writes which letter.
//!
//! `ü`, `è`, `ß`, `ł`, `ő`, `č`, `ġ`, `ħ`, `á` and `ż` are not typed by
//! anything in this crate. They are typed by the table `libX11` and
//! `libxkbcommon` ship — the same file every other desktop composes through —
//! and **alo OS writes no table of its own**. A letter this crate could produce
//! from a table somebody here typed would be a letter that appeared on one
//! machine and not another, and a European product that got `ġ` wrong for
//! Malta while getting `ü` right for Germany would have chosen which Europeans
//! it was for.
//!
//! # The shape of the file
//!
//! One sequence to a line: the keys, then `:`, then what they write.
//!
//! ```text
//! <dead_diaeresis> <u>         : "ü"   udiaeresis # LATIN SMALL LETTER U WITH DIAERESIS
//! <Multi_key> <s> <s>          : "ß"   ssharp     # LATIN SMALL LETTER SHARP S
//! ```
//!
//! `<Multi_key>` is the compose key ([`crate::ComposeKey`]); a `<dead_…>` key
//! is a dead key, which is a key on the layout rather than anything held down.
//! Both are sequences in the same table, which is why there is one reader here
//! and not two.
//!
//! # A line that cannot be read refuses the whole table
//!
//! The rule [`alo_kept`] states for a person's own file holds for this one:
//! half a table is the machine choosing the other half, and a person who typed
//! `ő` and got nothing would have no way of knowing that alo OS had quietly
//! dropped the line. [`NotComposed`] names the line instead.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::keysym::{Keysym, KeysymError};

/// Where `libX11` keeps the table on a machine alo OS ships.
///
/// The UTF-8 table, which is the one every other locale's file includes.
pub const THE_TABLE: &str = "/usr/share/X11/locale/en_US.UTF-8/Compose";

/// The rented table, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compose {
    /// Every sequence, and what it writes.
    entries: BTreeMap<Vec<Keysym>, String>,
}

impl Compose {
    /// The table, read from where the rented package keeps it.
    ///
    /// # Errors
    /// [`NotComposed`], naming the file.
    pub fn read() -> Result<Self, NotComposed> {
        Self::read_at(Path::new(THE_TABLE))
    }

    /// The table, read from a file named here.
    ///
    /// # Errors
    /// [`NotComposed`], naming the file and, where there is one, the line.
    pub fn read_at(at: &Path) -> Result<Self, NotComposed> {
        match std::fs::read_to_string(at) {
            Ok(text) => Self::read_text(&text, at),
            Err(why) if why.kind() == io::ErrorKind::NotFound => Err(NotComposed {
                at: at.to_owned(),
                why: NotATable::NotThere,
            }),
            Err(why) => Err(NotComposed {
                at: at.to_owned(),
                why: NotATable::Disk(why.kind()),
            }),
        }
    }

    /// The table, read from text that came from `at`.
    ///
    /// # Errors
    /// [`NotComposed`], naming the line that is not a sequence, or saying that
    /// the text held none at all.
    pub fn read_text(text: &str, at: &Path) -> Result<Self, NotComposed> {
        let mut entries = BTreeMap::new();
        for (counted, line) in text.lines().enumerate() {
            let at_line = counted.saturating_add(1);
            match one_line(line) {
                Ok(None) => {}
                Ok(Some((keys, writes))) => {
                    entries.entry(keys).or_insert(writes);
                }
                Err(_) => {
                    return Err(NotComposed {
                        at: at.to_owned(),
                        why: NotATable::NotASequence { line: at_line },
                    });
                }
            }
        }
        if entries.is_empty() {
            return Err(NotComposed {
                at: at.to_owned(),
                why: NotATable::NoSequencesInIt,
            });
        }
        Ok(Self { entries })
    }

    /// What this sequence writes, when the table has it whole.
    #[must_use]
    pub fn writes(&self, keys: &[Keysym]) -> Option<&str> {
        self.entries.get(keys).map(String::as_str)
    }

    /// Whether any sequence in the table begins with these keys and is longer,
    /// which is what makes a dead key wait rather than give up.
    #[must_use]
    pub fn begins_a_sequence(&self, keys: &[Keysym]) -> bool {
        // Every sequence beginning with these keys is one run of the map, since
        // a sequence sorts by its keys: so the walk stops at the first that
        // does not, rather than reading the rest of the table.
        self.entries
            .range(keys.to_vec()..)
            .take_while(|(sequence, _)| sequence.starts_with(keys))
            .any(|(sequence, _)| sequence.len() > keys.len())
    }

    /// How many sequences the table has.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.entries.len()
    }
}

/// One line of the table: nothing, or a sequence and what it writes.
fn one_line(line: &str) -> Result<Option<(Vec<Keysym>, String)>, Unreadable> {
    let mut rest = line.trim_start();
    if rest.is_empty() || rest.starts_with('#') {
        return Ok(None);
    }
    let mut keys = Vec::new();
    while let Some(after) = rest.strip_prefix('<') {
        let (named, beyond) = after.split_once('>').ok_or(Unreadable)?;
        keys.push(Keysym::named(named)?);
        rest = beyond.trim_start();
    }
    if keys.is_empty() {
        return Err(Unreadable);
    }
    rest = rest.strip_prefix(':').ok_or(Unreadable)?.trim_start();
    let quoted = rest.strip_prefix('"').ok_or(Unreadable)?;
    Ok(Some((keys, unescaped(quoted)?)))
}

/// What is between the quotes, with the two escapes the table uses undone.
fn unescaped(quoted: &str) -> Result<String, Unreadable> {
    let mut written = String::new();
    let mut letters = quoted.chars();
    loop {
        match letters.next().ok_or(Unreadable)? {
            '"' => return Ok(written),
            '\\' => written.push(match letters.next().ok_or(Unreadable)? {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return Err(Unreadable),
            }),
            letter => written.push(letter),
        }
    }
}

/// A line that is not a sequence. Which line it was is the reader's to say.
struct Unreadable;

impl From<KeysymError> for Unreadable {
    fn from(_: KeysymError) -> Self {
        Self
    }
}

/// The rented compose table could not be read, so nothing composes and the
/// person is told rather than left typing letters that never arrive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotComposed {
    /// The file.
    at: PathBuf,
    /// What was wrong with it.
    why: NotATable,
}

impl NotComposed {
    /// The file that could not be read.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// What was wrong with it, for whoever is fixing the machine.
    #[must_use]
    pub fn why(&self) -> &NotATable {
        &self.why
    }

    /// The line the table stopped being one at, when it stopped at a line.
    #[must_use]
    pub fn line(&self) -> Option<usize> {
        match self.why {
            NotATable::NotASequence { line } => Some(line),
            _ => None,
        }
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> crate::words::Word {
        crate::words::COMPOSING_NOT_THERE
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &alo_strings::Strings) -> alo_strings::Said {
        strings.say(
            &self.word().key(),
            &alo_strings::Filling::of("path", self.at.display().to_string()),
        )
    }
}

/// Why the rented compose table was not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotATable {
    /// The file is not on this machine.
    NotThere,
    /// The disk would not give it up.
    Disk(io::ErrorKind),
    /// A line that is not a sequence, counted from one the way an editor counts.
    NotASequence {
        /// Which line.
        line: usize,
    },
    /// It was read and holds no sequences at all.
    NoSequencesInIt,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, keys};

    const A_TABLE: &str = "\
# a comment
<dead_diaeresis> <u>\t\t: \"ü\"\tudiaeresis # LATIN SMALL LETTER U WITH DIAERESIS
<Multi_key> <s> <s>\t\t: \"ß\"\tssharp
<Multi_key> <slash> <slash>\t: \"\\\\\"\tbackslash
<dead_diaeresis> <space>\t: \"\\\"\"\tquotedbl
";

    fn somewhere() -> PathBuf {
        PathBuf::from("/usr/share/X11/locale/en_US.UTF-8/Compose")
    }

    /// A sequence, its letter, and the two escapes the rented table uses.
    #[test]
    fn a_sequence_and_what_it_writes_are_read() {
        let table = Compose::read_text(A_TABLE, &somewhere()).unwrap();
        assert_eq!(table.how_many(), 4);
        assert_eq!(table.writes(&keys(&["dead_diaeresis", "u"])), Some("ü"));
        assert_eq!(table.writes(&keys(&["Multi_key", "s", "s"])), Some("ß"));
        assert_eq!(
            table.writes(&keys(&["Multi_key", "slash", "slash"])),
            Some("\\")
        );
        assert_eq!(
            table.writes(&keys(&["dead_diaeresis", "space"])),
            Some("\"")
        );
        assert_eq!(table.writes(&keys(&["dead_diaeresis"])), None);
    }

    /// A key that begins a longer sequence is told from one that ends it.
    #[test]
    fn a_key_that_begins_a_longer_sequence_is_known() {
        let table = Compose::read_text(A_TABLE, &somewhere()).unwrap();
        assert!(table.begins_a_sequence(&keys(&["dead_diaeresis"])));
        assert!(table.begins_a_sequence(&keys(&["Multi_key", "s"])));
        assert!(!table.begins_a_sequence(&keys(&["Multi_key", "s", "s"])));
        assert!(!table.begins_a_sequence(&keys(&["q"])));
    }

    /// **A line that is not a sequence refuses the whole table and says which
    /// line**, rather than being dropped so that one letter silently stops
    /// working.
    #[test]
    fn a_line_that_is_not_a_sequence_refuses_the_table_and_names_the_line() {
        for (broken, line) in [
            ("<dead_acute> <a>\t: \"á\"\n~Ctrl <a> <b> : \"x\"\n", 2),
            ("<dead_acute> <a>\t: \"á\"\n<a> <b> \"x\"\n", 2),
            ("<dead_acute <a>\t: \"á\"\n", 1),
            ("<dead_acute> <a>\t: \"\\q\"\n", 1),
            ("<dead_acute> <a>\t: á\n", 1),
            ("<dead acute> <a>\t: \"á\"\n", 1),
        ] {
            let refused = Compose::read_text(broken, &somewhere()).unwrap_err();
            assert_eq!(
                refused.why(),
                &NotATable::NotASequence { line },
                "{broken:?}"
            );
            assert_eq!(refused.line(), Some(line));
        }
    }

    /// A table with nothing in it is refused rather than read as a machine on
    /// which nothing composes.
    #[test]
    fn a_table_with_no_sequences_is_refused_and_says_where() {
        let refused = Compose::read_text("# only a comment\n", &somewhere()).unwrap_err();
        assert_eq!(refused.why(), &NotATable::NoSequencesInIt);
        assert_eq!(refused.line(), None);
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug());
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(said.text().contains("Compose"), "{said}");
    }

    /// A table that is not on this machine is refused, naming the file.
    #[test]
    fn a_table_that_is_not_on_this_machine_is_refused() {
        let nowhere = somewhere().join("not-a-directory").join("Compose");
        let refused = Compose::read_at(&nowhere).unwrap_err();
        assert!(matches!(
            refused.why(),
            NotATable::NotThere | NotATable::Disk(_)
        ));
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains(&nowhere.display().to_string())
        );
    }
}
