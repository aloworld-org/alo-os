//! The 24 official languages of the European Union, each written as it is
//! written by the people who speak it.
//!
//! This is `docs/features.md`'s list, as code rather than as a sentence in a
//! document, and the test at the bottom of this file is what stops the two from
//! drifting apart. The list is here rather than in a configuration file because
//! it is not configuration: which languages the Union has is a fact about the
//! Union, and a machine that shipped with a shorter list would be making the
//! decision `docs/features.md` refuses to make — *not "English plus the big
//! five": a sovereignty product that cannot speak Maltese or Irish is selling
//! sovereignty to some Europeans and not others.*
//!
//! **The list is a starting point and not a boundary.** Any language somebody
//! contributes is a language alo OS speaks; nothing anywhere checks a
//! translation against this list before accepting it. What being on it buys is
//! one thing: a name in its own language, so the language picker can offer it
//! before anybody has translated a word.
//!
//! There are no translations in this repository yet. This list is what a first
//! release ships knowing, and what a release note counts against.

/// One of the Union's languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Official {
    /// The language tag, which is what a [`crate::Language`] is made from.
    pub tag: &'static str,
    /// What the language is called in itself, which is how it is offered to
    /// somebody who does not read ours.
    pub calls_itself: &'static str,
    /// What it is called in English, which is how `docs/features.md` lists it
    /// and is used for nothing else.
    pub in_english: &'static str,
}

/// All 24, in the order `docs/features.md` lists them, which is alphabetical by
/// the English name because that is the order the Union itself publishes.
pub const OFFICIAL: [Official; 24] = [
    Official {
        tag: "bg",
        calls_itself: "Български",
        in_english: "Bulgarian",
    },
    Official {
        tag: "hr",
        calls_itself: "Hrvatski",
        in_english: "Croatian",
    },
    Official {
        tag: "cs",
        calls_itself: "Čeština",
        in_english: "Czech",
    },
    Official {
        tag: "da",
        calls_itself: "Dansk",
        in_english: "Danish",
    },
    Official {
        tag: "nl",
        calls_itself: "Nederlands",
        in_english: "Dutch",
    },
    Official {
        tag: "en",
        calls_itself: "English",
        in_english: "English",
    },
    Official {
        tag: "et",
        calls_itself: "Eesti",
        in_english: "Estonian",
    },
    Official {
        tag: "fi",
        calls_itself: "Suomi",
        in_english: "Finnish",
    },
    Official {
        tag: "fr",
        calls_itself: "Français",
        in_english: "French",
    },
    Official {
        tag: "de",
        calls_itself: "Deutsch",
        in_english: "German",
    },
    Official {
        tag: "el",
        calls_itself: "Ελληνικά",
        in_english: "Greek",
    },
    Official {
        tag: "hu",
        calls_itself: "Magyar",
        in_english: "Hungarian",
    },
    Official {
        tag: "ga",
        calls_itself: "Gaeilge",
        in_english: "Irish",
    },
    Official {
        tag: "it",
        calls_itself: "Italiano",
        in_english: "Italian",
    },
    Official {
        tag: "lv",
        calls_itself: "Latviešu",
        in_english: "Latvian",
    },
    Official {
        tag: "lt",
        calls_itself: "Lietuvių",
        in_english: "Lithuanian",
    },
    Official {
        tag: "mt",
        calls_itself: "Malti",
        in_english: "Maltese",
    },
    Official {
        tag: "pl",
        calls_itself: "Polski",
        in_english: "Polish",
    },
    Official {
        tag: "pt",
        calls_itself: "Português",
        in_english: "Portuguese",
    },
    Official {
        tag: "ro",
        calls_itself: "Română",
        in_english: "Romanian",
    },
    Official {
        tag: "sk",
        calls_itself: "Slovenčina",
        in_english: "Slovak",
    },
    Official {
        tag: "sl",
        calls_itself: "Slovenščina",
        in_english: "Slovenian",
    },
    Official {
        tag: "es",
        calls_itself: "Español",
        in_english: "Spanish",
    },
    Official {
        tag: "sv",
        calls_itself: "Svenska",
        in_english: "Swedish",
    },
];

/// The language the code itself is written in, and the one every string in this
/// repository is written in before anybody translates it.
///
/// English is not the important language here and is not a default anybody
/// chose: it is the language the source happens to be in, which is why
/// [`crate::Said`] always says whether an answer came from a translation or
/// from the source. A person shown English because Latvian was missing has been
/// shown a gap, and the type says so.
pub const THE_SOURCE: &str = "en";

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::language::Language;

    /// **The list is the one in `docs/features.md`**, and this test is what
    /// stops the two from drifting apart: a language dropped here without the
    /// document changing is caught by whoever wrote the document.
    #[test]
    fn the_languages_are_the_ones_the_features_document_names() {
        let named = [
            "Bulgarian",
            "Croatian",
            "Czech",
            "Danish",
            "Dutch",
            "Estonian",
            "English",
            "Finnish",
            "French",
            "German",
            "Greek",
            "Hungarian",
            "Irish",
            "Italian",
            "Latvian",
            "Lithuanian",
            "Maltese",
            "Polish",
            "Portuguese",
            "Romanian",
            "Slovak",
            "Slovenian",
            "Spanish",
            "Swedish",
        ];
        assert_eq!(named.len(), 24);
        assert_eq!(OFFICIAL.len(), 24);
        for name in named {
            assert!(
                OFFICIAL.iter().any(|official| official.in_english == name),
                "{name} is in docs/features.md and not here"
            );
        }
    }

    /// Every tag is a language this crate can make, and no two entries are the
    /// same language — a picker with one language in it twice is a picker
    /// nobody can use.
    #[test]
    fn every_tag_is_a_language_and_no_two_are_one_language() {
        for (at, official) in OFFICIAL.iter().enumerate() {
            let language = Language::written(official.tag).unwrap();
            assert_eq!(language.tag(), official.tag);
            assert!(!official.calls_itself.is_empty());
            for other in OFFICIAL.iter().skip(at.saturating_add(1)) {
                assert_ne!(official.tag, other.tag);
                assert_ne!(official.calls_itself, other.calls_itself);
                assert_ne!(official.in_english, other.in_english);
            }
        }
    }

    /// **A language is named in its own language**, and half of these names are
    /// not writable in ASCII at all. A list that had quietly been transliterated
    /// would be a list a person scanning for their own language would miss.
    #[test]
    fn the_names_are_in_their_own_languages() {
        let in_its_own_script = OFFICIAL
            .iter()
            .filter(|official| !official.calls_itself.is_ascii())
            .count();
        assert!(
            in_its_own_script >= 10,
            "only {in_its_own_script} names carry their own letters — has the list been transliterated?"
        );
        for official in OFFICIAL {
            if official.in_english != "English" {
                assert_ne!(
                    official.calls_itself, official.in_english,
                    "{} is listed under its English name",
                    official.in_english
                );
            }
        }
    }

    /// The source language is one of the 24 rather than something outside the
    /// list, so a person who prefers English is answered by the same machinery
    /// as everybody else.
    #[test]
    fn the_source_language_is_in_the_list() {
        assert!(OFFICIAL.iter().any(|official| official.tag == THE_SOURCE));
    }
}
