//! Which characters in a text a person could miss, and why.
//!
//! ADR 0001, amended 2026-11-13: a text argument may hold **any** character,
//! and the approval marks the ones a person reading it could miss rather than
//! refusing them. This file decides which those are. What the approval does
//! with the answer is [`crate::the_whole_text`].
//!
//! **What it finds, and what it does not.** Five things, each named in
//! [`WhyMarked`]:
//!
//! - control characters, other than a newline and a tab, which every editor
//!   shows as what they are;
//! - characters that change the direction the text is drawn in — the ones a
//!   reader sees in one order and a program reads in another;
//! - characters that draw nothing at all, such as a zero-width space, a soft
//!   hyphen, and the tag characters that can carry a whole hidden message;
//! - spaces that are not an ordinary space;
//! - a letter from another alphabet in the middle of a word — Latin, Greek and
//!   Cyrillic are told apart, which is where the familiar look-alikes are
//!   (`раypal` with a Cyrillic `р` and `а`).
//!
//! It does **not** find every confusion there is. Two words in one alphabet
//! that look alike (`rn` and `m`), a look-alike from an alphabet other than
//! those three, and a whole word written in another alphabet are not marked:
//! Unicode's own confusables table would be the way to find those, and this
//! file does not carry it. The approval still shows every one of those
//! characters, as it shows every character; it simply does not point at them.
//!
//! Variation selectors (`U+FE00`–`U+FE0F`) are left unmarked on purpose: they
//! are inside most emoji a person types, and a mark on every heart would teach
//! a person to stop reading the marks.

/// Why a character is marked in the approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhyMarked {
    /// A control character: a carriage return on its own, an escape, a bell, a
    /// null. Not a newline and not a tab.
    Control,
    /// A character that changes the direction the text is drawn in, so what a
    /// person reads and what is delivered are in different orders.
    Direction,
    /// A character that draws nothing: a zero-width space, a joiner, a soft
    /// hyphen, a byte-order mark, a tag character.
    Invisible,
    /// A space, or a break, that is not an ordinary space or newline.
    Space,
    /// A letter from another alphabet in the middle of a word written in one.
    OtherAlphabet,
}

/// Why this character would be marked wherever it stands, if it would.
///
/// A letter from another alphabet is not decided here, because it depends on
/// the word around it: see [`from_another_alphabet`].
#[must_use]
pub(crate) fn on_its_own(character: char) -> Option<WhyMarked> {
    match character {
        '\n' | '\t' => None,
        '\u{061C}'
        | '\u{200E}'
        | '\u{200F}'
        | '\u{202A}'..='\u{202E}'
        | '\u{2066}'..='\u{2069}' => Some(WhyMarked::Direction),
        '\u{00AD}'
        | '\u{034F}'
        | '\u{115F}'
        | '\u{1160}'
        | '\u{17B4}'
        | '\u{17B5}'
        | '\u{180B}'..='\u{180F}'
        | '\u{200B}'..='\u{200D}'
        | '\u{2060}'..='\u{2064}'
        | '\u{3164}'
        | '\u{FEFF}'
        | '\u{FFA0}'
        | '\u{E0000}'..='\u{E007F}' => Some(WhyMarked::Invisible),
        '\u{00A0}'
        | '\u{1680}'
        | '\u{2000}'..='\u{200A}'
        | '\u{2028}'
        | '\u{2029}'
        | '\u{202F}'
        | '\u{205F}'
        | '\u{3000}' => Some(WhyMarked::Space),
        other if other.is_control() => Some(WhyMarked::Control),
        _ => None,
    }
}

/// The three alphabets whose letters are told apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alphabet {
    /// The Latin alphabet, with its accented letters.
    Latin,
    /// The Greek alphabet, with its accented letters.
    Greek,
    /// The Cyrillic alphabet and its supplement.
    Cyrillic,
}

/// Which of the three alphabets a character is a letter of, if any.
fn alphabet(character: char) -> Option<Alphabet> {
    match character {
        'a'..='z' | 'A'..='Z' => Some(Alphabet::Latin),
        '\u{00D7}' | '\u{00F7}' => None,
        '\u{00C0}'..='\u{024F}' | '\u{1E00}'..='\u{1EFF}' => Some(Alphabet::Latin),
        '\u{0370}'..='\u{03FF}' | '\u{1F00}'..='\u{1FFF}' => Some(Alphabet::Greek),
        '\u{0400}'..='\u{052F}' => Some(Alphabet::Cyrillic),
        _ => None,
    }
}

/// For one word, which of its characters are letters from another alphabet
/// than the one the word is mostly written in.
///
/// A word is a run of letters and digits. A word in one alphabet marks
/// nothing, however unusual the alphabet: Ελληνικά and кириллица are words, not
/// look-alikes. Where a word mixes them, the letters of the alphabet it holds
/// most of are left alone and the rest are marked; on a tie, Latin is taken to
/// be the word's own, because a look-alike is almost always a stranger in a
/// Latin word.
#[must_use]
pub(crate) fn from_another_alphabet(word: &[char]) -> Vec<bool> {
    let mut counted = [
        (Alphabet::Latin, 0_usize),
        (Alphabet::Greek, 0),
        (Alphabet::Cyrillic, 0),
    ];
    for character in word {
        if let Some(found) = alphabet(*character) {
            for (each, count) in &mut counted {
                if *each == found {
                    *count += 1;
                }
            }
        }
    }
    let present = counted.iter().filter(|(_, count)| *count > 0).count();
    if present < 2 {
        return vec![false; word.len()];
    }
    // `max_by_key` keeps the last of equals, so the list is walked from the
    // end to make Latin win a tie.
    let own = counted
        .iter()
        .rev()
        .max_by_key(|(_, count)| *count)
        .map_or(Alphabet::Latin, |(each, _)| *each);
    word.iter()
        .map(|character| alphabet(*character).is_some_and(|found| found != own))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A newline and a tab are what a page of code is made of; everything else
    /// a terminal would act on is marked.
    #[test]
    fn a_newline_and_a_tab_are_ordinary_and_other_controls_are_not() {
        assert_eq!(on_its_own('\n'), None);
        assert_eq!(on_its_own('\t'), None);
        for control in ['\r', '\u{1b}', '\u{7}', '\0', '\u{7f}', '\u{85}'] {
            assert_eq!(on_its_own(control), Some(WhyMarked::Control), "{control:?}");
        }
    }

    /// The characters behind the attacks a reader cannot see.
    #[test]
    fn the_characters_a_reader_cannot_see_are_found() {
        for (character, why) in [
            ('\u{202E}', WhyMarked::Direction),
            ('\u{2067}', WhyMarked::Direction),
            ('\u{200F}', WhyMarked::Direction),
            ('\u{200B}', WhyMarked::Invisible),
            ('\u{200D}', WhyMarked::Invisible),
            ('\u{FEFF}', WhyMarked::Invisible),
            ('\u{00AD}', WhyMarked::Invisible),
            ('\u{E0041}', WhyMarked::Invisible),
            ('\u{00A0}', WhyMarked::Space),
            ('\u{3000}', WhyMarked::Space),
            ('\u{2028}', WhyMarked::Space),
        ] {
            assert_eq!(on_its_own(character), Some(why), "{character:?}");
        }
    }

    /// Ordinary text in any language, and an emoji with its variation
    /// selector, is not marked.
    #[test]
    fn ordinary_text_in_any_language_is_not_marked() {
        for character in "Grüße, Ελλάδα, Україна, 日本語, عربي ❤\u{FE0F} {}();".chars()
        {
            assert_eq!(on_its_own(character), None, "{character:?}");
        }
    }

    /// A Cyrillic `а` in a Latin word is marked; a word wholly in one alphabet
    /// is not.
    #[test]
    fn a_letter_from_another_alphabet_is_marked_only_inside_a_mixed_word() {
        let mixed: Vec<char> = "p\u{0430}ypal".chars().collect();
        assert_eq!(
            from_another_alphabet(&mixed),
            vec![false, true, false, false, false, false]
        );
        for word in ["paypal", "кириллица", "Ελληνικά", "x86", "日本語"] {
            let word: Vec<char> = word.chars().collect();
            assert!(!from_another_alphabet(&word).contains(&true), "{word:?}");
        }
    }

    /// A word mostly in Cyrillic marks the Latin letter in it, and a tie is
    /// read as a Latin word.
    #[test]
    fn the_alphabet_a_word_holds_most_of_is_its_own() {
        let cyrillic: Vec<char> = "пpивет".chars().collect();
        assert_eq!(
            from_another_alphabet(&cyrillic),
            vec![false, true, false, false, false, false]
        );
        let tie: Vec<char> = "a\u{0430}".chars().collect();
        assert_eq!(from_another_alphabet(&tie), vec![false, true]);
    }
}
