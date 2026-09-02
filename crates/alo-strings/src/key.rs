//! What names one string.
//!
//! A key is what the code asks for and what a translator's file is sorted by.
//! It is written `area.what-it-says` — at least two parts — because the first
//! part says which part of the system a string came from. Without that, two
//! crates would both name a string `gone`, the vocabulary would refuse the
//! second one, and the crate that lost would be whichever was added later.
//!
//! **A key is not a sentence and is not shown to anybody**, with one exception
//! that is deliberate: when the code asks for a key that is not in the
//! vocabulary at all there is no honest text to show, so [`crate::Said`] shows
//! the key itself, marked. A blank space would look like a sentence that had
//! nothing to say.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::form::Form;

/// What names one string.
///
/// Checked when it is made, so everything downstream — the vocabulary, a
/// translation read off a disk, the lookup — is working with a key that is
/// already the right shape.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Key {
    /// The whole key, `area.what-it-says`, already checked.
    named: String,
}

impl Key {
    /// A key, if it is written the way keys are written.
    ///
    /// # Errors
    ///
    /// [`KeyError`], which says what to write instead.
    pub fn named(named: &str) -> Result<Self, KeyError> {
        if named.is_empty() {
            return Err(KeyError::Empty);
        }
        let mut parts = 0_usize;
        for part in named.split('.') {
            parts = parts.saturating_add(1);
            let mut characters = part.chars();
            let Some(first) = characters.next() else {
                return Err(KeyError::EmptyPart {
                    named: named.to_owned(),
                });
            };
            if !first.is_ascii_lowercase() {
                return Err(KeyError::BadStart {
                    part: part.to_owned(),
                });
            }
            for character in characters {
                if !(character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || character == '-')
                {
                    return Err(KeyError::BadCharacter {
                        part: part.to_owned(),
                        character,
                    });
                }
            }
        }
        if parts < 2 {
            return Err(KeyError::OnePart {
                named: named.to_owned(),
            });
        }
        Ok(Self {
            named: named.to_owned(),
        })
    }

    /// A key this repository wrote itself, checked by a test rather than at
    /// run time.
    ///
    /// A crate that declares what it can say writes its own keys as literals —
    /// they are not typed by anybody and cannot arrive from a file — and the
    /// alternative is every one of them being a `Result` that the calling code
    /// has to invent a fallback for. There is no honest fallback: a sentence
    /// that could not be looked up is a sentence nobody can read.
    ///
    /// So this is the same shape as `alo-shortcuts`' shipped bindings and
    /// `alo-appearance`'s shipped wallpaper — built by the compiler, with a
    /// test putting every one of them back through [`Key::named`], which is how
    /// what we ship stays held to the rule everybody else is held to.
    ///
    /// It takes a `&'static str` so that the only thing that can reach it is a
    /// literal. A key read from a file, or built from anything somebody typed,
    /// has to go through [`Key::named`] and be refused if it is wrong.
    ///
    /// A key that got past a missing test says nothing about safety: nothing
    /// declares it, so [`crate::Strings`] answers with the key itself and
    /// [`crate::Said::is_a_bug`] says so — the same answer as a key nobody
    /// declared, which is what this would be.
    #[must_use]
    pub fn unchecked(named: &'static str) -> Self {
        Self {
            named: named.to_owned(),
        }
    }

    /// Which part of the system this string comes from: everything before the
    /// first dot.
    #[must_use]
    pub fn area(&self) -> &str {
        self.named.split('.').next().unwrap_or(&self.named)
    }

    /// The key as it is written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.named
    }

    /// This key with a plural form on the end: `files.too-big` becomes
    /// `files.too-big.one`.
    ///
    /// This is how a countable string reaches a translator's file — one line
    /// per form, sorted next to each other because they share everything up to
    /// the last part. It cannot fail: every form is named in lowercase letters,
    /// which is what a part of a key is made of.
    #[must_use]
    pub fn for_form(&self, form: Form) -> Self {
        Self {
            named: format!("{}.{}", self.named, form.tag()),
        }
    }

    /// The key this one is a form of, and which form — or `None` when the last
    /// part is not a form's name.
    ///
    /// A key whose last part happens to be a form's name is not a form of
    /// anything unless something declared it countable, which is
    /// [`crate::Vocabulary`]'s question rather than this one's.
    pub(crate) fn without_form(&self) -> Option<(Self, Form)> {
        let (before, last) = self.named.rsplit_once('.')?;
        let form = Form::of_tag(last)?;
        if !before.contains('.') {
            return None;
        }
        Some((
            Self {
                named: before.to_owned(),
            },
            form,
        ))
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.named)
    }
}

impl TryFrom<String> for Key {
    type Error = KeyError;

    fn try_from(named: String) -> Result<Self, Self::Error> {
        Self::named(&named)
    }
}

impl From<Key> for String {
    fn from(key: Key) -> Self {
        key.named
    }
}

/// Why something is not a key.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum KeyError {
    /// Nothing at all.
    #[error(
        "give the string a name — something like files.not-a-folder, which is what a translator's file is sorted by"
    )]
    Empty,

    /// One part, so nothing says where the string came from.
    #[error(
        "call it something.{named} — the part before the dot says which part of the system the string comes from, so that two of them cannot land on one name"
    )]
    OnePart {
        /// What was offered.
        named: String,
    },

    /// Two dots together, or a dot at an end.
    #[error("{named} has an empty part in it — every part between the dots needs a name")]
    EmptyPart {
        /// What was offered.
        named: String,
    },

    /// A part that starts with something other than a letter.
    #[error(
        "start {part} with a lowercase letter — a key is read by people sorting a file, not by a machine"
    )]
    BadStart {
        /// The part that starts wrongly.
        part: String,
    },

    /// A character that is not allowed in a key.
    #[error(
        "take {character} out of {part} — keys are lowercase letters, digits and hyphens, so that a key is the same on every machine that writes one down"
    )]
    BadCharacter {
        /// The part it is in.
        part: String,
        /// The character.
        character: char,
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
    fn a_key_says_where_it_came_from() {
        let key = Key::named("files.not-a-folder").unwrap();
        assert_eq!(key.area(), "files");
        assert_eq!(key.to_string(), "files.not-a-folder");
    }

    #[test]
    fn more_than_two_parts_is_fine() {
        let key = Key::named("files.failed.too-big").unwrap();
        assert_eq!(key.area(), "files");
    }

    /// A key the code wrote itself is the same key the check would have made,
    /// which is the whole of what [`Key::unchecked`] promises — and the test
    /// that every crate's own keys are written that way is that crate's, beside
    /// the keys.
    #[test]
    fn a_key_the_code_wrote_itself_is_the_key_the_check_would_have_made() {
        assert_eq!(
            Key::unchecked("files.failed.too-big"),
            Key::named("files.failed.too-big").unwrap()
        );
    }

    /// **A key with no area is refused**, because the area is the only thing
    /// stopping two crates from naming one string and one of them losing
    /// quietly to whichever was registered second.
    #[test]
    fn a_key_without_an_area_is_refused() {
        assert_eq!(
            Key::named("gone"),
            Err(KeyError::OnePart {
                named: "gone".to_owned()
            })
        );
    }

    #[test]
    fn nothing_at_all_is_refused() {
        assert_eq!(Key::named(""), Err(KeyError::Empty));
    }

    #[test]
    fn a_dot_at_either_end_or_two_together_is_refused() {
        for named in [".files", "files.", "files..gone"] {
            assert_eq!(
                Key::named(named),
                Err(KeyError::EmptyPart {
                    named: named.to_owned()
                }),
                "{named}"
            );
        }
    }

    /// A key is lowercase, and the reason is not tidiness: a file sorted by key
    /// sorts differently depending on whether the reader's tools fold case, and
    /// a translator working through two files in a different order from ours is
    /// a translator who misses a line.
    #[test]
    fn capitals_spaces_and_underscores_are_refused() {
        for named in ["Files.gone", "files.Not-A-Folder"] {
            assert!(
                matches!(Key::named(named), Err(KeyError::BadStart { .. })),
                "{named}"
            );
        }
        for named in [
            "files.notAFolder",
            "files.not a folder",
            "files.not_a_folder",
        ] {
            assert!(
                matches!(Key::named(named), Err(KeyError::BadCharacter { .. })),
                "{named}"
            );
        }
    }

    #[test]
    fn a_part_starting_with_a_digit_or_a_hyphen_is_refused() {
        for named in ["files.1st", "files.-gone", "2files.gone"] {
            assert!(
                matches!(Key::named(named), Err(KeyError::BadStart { .. })),
                "{named}"
            );
        }
    }

    /// Every refusal says what to do rather than what went wrong, which is the
    /// rule `alo-models` set and every crate since has kept.
    #[test]
    fn every_refusal_says_what_to_do() {
        let refusals = [
            Key::named("").unwrap_err(),
            Key::named("gone").unwrap_err(),
            Key::named("files.").unwrap_err(),
            Key::named("Files.gone").unwrap_err(),
            Key::named("files.a b").unwrap_err(),
        ];
        for refusal in refusals {
            let said = refusal.to_string();
            assert!(said.len() > 30, "{said}");
            assert!(!said.starts_with("invalid"), "{said}");
        }
    }

    /// A countable string reaches a translator as one key per form, sorted next
    /// to each other because they share everything up to the last part.
    #[test]
    fn a_key_takes_a_form_on_the_end_and_gives_it_back() {
        let key = Key::named("files.too-big").unwrap();
        assert_eq!(key.for_form(Form::One).as_str(), "files.too-big.one");
        assert_eq!(key.for_form(Form::Other).as_str(), "files.too-big.other");
        for form in crate::form::EVERY_FORM {
            assert_eq!(
                key.for_form(form).without_form(),
                Some((key.clone(), form)),
                "{form}"
            );
        }
        // And it is still a key, checked like any other.
        assert!(Key::named(key.for_form(Form::Few).as_str()).is_ok());
    }

    /// A key whose last part is not a form's name is not a form of anything,
    /// and neither is one that would leave no area behind.
    #[test]
    fn a_key_that_is_not_a_form_of_something_says_so() {
        assert_eq!(Key::named("files.too-big").unwrap().without_form(), None);
        assert_eq!(
            Key::named("files.gone.someday").unwrap().without_form(),
            None
        );
        assert_eq!(
            Key::named("files.one").unwrap().without_form(),
            None,
            "there would be no area left"
        );
    }

    /// A key read back off a disk is checked the same way as one written in
    /// code, because a translation file is a file somebody edited by hand.
    #[test]
    fn a_key_is_checked_when_it_is_read_back() {
        let written = serde_json::to_string(&Key::named("files.gone").unwrap()).unwrap();
        assert_eq!(written, "\"files.gone\"");
        assert_eq!(
            serde_json::from_str::<Key>(&written).unwrap(),
            Key::named("files.gone").unwrap()
        );
        assert!(serde_json::from_str::<Key>("\"gone\"").is_err());
    }
}
