//! **Whether the rented speech engine has a voice for each of the 24, named.**
//!
//! Task 2 of `docs/autonomy/v0-5-access-and-language-plan.md`, whose constraint
//! is the point of this file: *where the rented speech engine has no voice for a
//! language, that is stated per language, not hidden.* A screen reader that
//! cannot speak somebody's language is not an accessible machine for them, and a
//! table that quietly listed 24 languages would be the kind of claim
//! `docs/features.md` says we do not make.
//!
//! # What this is measured against, and what is not decided yet
//!
//! **alo OS pins no speech engine today.** ADR 0011 says engines are rented and
//! configured — Orca, AT-SPI and a speech engine among them — but no version of
//! any of them is pinned in `alo-image`, so there is nothing here to call *the*
//! engine's voices. What this table holds is what **eSpeak NG 1.51**
//! (`1.51+dfsg-12build1`) offers, the engine Orca speaks through by default on
//! the distribution alo OS is built from, read from `espeak-ng --voices` on
//! 2026-09-16 in the Linux VM this lane gates in.
//!
//! When the image pins an engine, this table is re-read against that one and
//! whatever it does not have becomes a language stated as unspoken — which is
//! why the check below is a test that can be run against a real engine rather
//! than a list somebody transcribed once.

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
    /// On 2026-09-16 eSpeak NG 1.51 has a voice for all 24, which is stated
    /// here as the fact it is rather than assumed of whatever is pinned later:
    /// a language that loses its voice when the image pins an engine fails this
    /// and is written down as unspoken.
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
