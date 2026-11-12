//! **Whether the speech engine the image ships has a voice for each of the 24,
//! named.**
//!
//! Task 2 of `docs/autonomy/access-and-language-plan.md`, whose constraint
//! is the point of this file: *where the rented speech engine has no voice for a
//! language, that is stated per language, not hidden.* A screen reader that
//! cannot speak somebody's language is not an accessible machine for them, and a
//! table that quietly listed 24 languages would be the kind of claim
//! `docs/features.md` says we do not make.
//!
//! # What this is measured against
//!
//! **The engine the image ships**: eSpeak NG **1.51.1**, the pinned base's own
//! package, which Orca speaks through
//! ([ADR 0151](../../../docs/decisions/0151-alo-ships-a-screen-reader-and-its-voices-stay-on-the-machine.md)).
//! Re-read on 2026-11-12 in that base with `espeak-ng --voices`, and each of
//! the 24 made to speak a word to a file; every voice named below is the one
//! that list gives. *Until that date this table was eSpeak NG 1.51 as Ubuntu
//! packages it, read on 2026-09-16, because no engine was pinned; the names
//! did not change.*
//!
//! **Which half runs for ever**: the recipe speaks a word in every language
//! this table says has a voice, on every image build, and
//! `crates/alo-image/tests/the_screen_reader_speaks_on_this_machine.rs` holds
//! the recipe's list to this table on every gate. A language that loses its
//! voice in a later base fails the build and is written here as unspoken.

/// **A voice for every one of the 24 official languages**, as the engine names
/// it, or an empty name where that engine has none.
///
/// Where a language has several — English has eight — this is the first the
/// engine lists, because which regional voice a person is given is their
/// setting to make and not this table's claim.
pub const A_VOICE_FOR_EACH: [(&str, &str); 24] = [
    ("bg", "Bulgarian"),
    ("hr", "Croatian"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("nl", "Dutch"),
    ("en", "English_(Caribbean)"),
    ("et", "Estonian"),
    ("fi", "Finnish"),
    ("fr", "French_(Belgium)"),
    ("de", "German"),
    ("el", "Greek"),
    ("hu", "Hungarian"),
    ("ga", "Gaelic_(Irish)"),
    ("it", "Italian"),
    ("lv", "Latvian"),
    ("lt", "Lithuanian"),
    ("mt", "Maltese"),
    ("pl", "Polish"),
    ("pt", "Portuguese_(Portugal)"),
    ("ro", "Romanian"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("es", "Spanish_(Spain)"),
    ("sv", "Swedish"),
];

/// Whether the engine measured here has a voice for this language.
#[must_use]
pub fn has_a_voice(tag: &str) -> bool {
    A_VOICE_FOR_EACH
        .iter()
        .any(|(language, voice)| *language == tag && !voice.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every one of the 24 is named, spoken or not.**
    ///
    /// On 2026-11-12 the eSpeak NG 1.51.1 the image ships has a voice for all
    /// 24, which is stated here as the fact it is rather than assumed of a
    /// later base: a language that loses its voice fails this and is written
    /// down as unspoken.
    #[test]
    fn every_official_language_is_named_spoken_or_not() {
        assert_eq!(A_VOICE_FOR_EACH.len(), 24);
        let unspoken: Vec<&str> = A_VOICE_FOR_EACH
            .iter()
            .filter(|(_, voice)| voice.is_empty())
            .map(|(language, _)| *language)
            .collect();
        assert!(
            unspoken.is_empty(),
            "these languages have no voice and the report must say so: {unspoken:?}"
        );
        for (language, _) in A_VOICE_FOR_EACH {
            assert!(has_a_voice(language), "{language}");
        }
        assert!(!has_a_voice("is"), "a language this machine does not carry");
    }

    /// **The 24 here are the 24 `alo-strings` carries**, in its order, so a
    /// language added there cannot be missed here.
    #[test]
    fn the_languages_are_the_ones_this_machine_carries() {
        let carried: Vec<&str> = alo_strings::union::OFFICIAL
            .iter()
            .map(|official| official.tag)
            .collect();
        let named: Vec<&str> = A_VOICE_FOR_EACH
            .iter()
            .map(|(language, _)| *language)
            .collect();
        assert_eq!(named, carried);
    }
}
