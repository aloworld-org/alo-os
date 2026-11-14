//! **Which keys a character is**, in qemu's own names for them (QMP `qcode`s).
//!
//! A walk types text, and qemu takes keys, not characters. So every character
//! a walk may type is mapped here to one key and whether Shift is held for it,
//! on a US layout — the layout a fresh alo machine starts with. A character
//! with no mapping is **refused**, never dropped: a field typed with one
//! character missing is a field that fails with no clue which one, and the
//! old shell helper's rule (`type-text.sh`, *a character with no mapping
//! STOPS the run*) is kept.

/// One key, held with the modifiers listed, as a person presses a chord:
/// modifiers down, the key down, the key up, the modifiers up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    /// The modifiers held, in the order they go down, as `qcode`s.
    held: Vec<&'static str>,
    /// The key itself, as a `qcode`.
    key: &'static str,
}

impl Chord {
    /// One key with nothing held, by its `qcode` — `tab`, `ret`, `esc`, `end`.
    #[must_use]
    pub fn key(key: &'static str) -> Self {
        Self {
            held: Vec::new(),
            key,
        }
    }

    /// One key with one modifier held — `Chord::with("meta_l", "i")` is
    /// Super+I.
    #[must_use]
    pub fn with(modifier: &'static str, key: &'static str) -> Self {
        Self {
            held: vec![modifier],
            key,
        }
    }

    /// The modifiers, in the order they go down.
    #[must_use]
    pub fn held(&self) -> &[&'static str] {
        &self.held
    }

    /// The key.
    #[must_use]
    pub fn the_key(&self) -> &'static str {
        self.key
    }

    /// How the chord reads in a report: `meta_l+i`.
    #[must_use]
    pub fn said(&self) -> String {
        let mut parts: Vec<&str> = self.held.clone();
        parts.push(self.key);
        parts.join("+")
    }
}

/// A character this walk cannot type, by where it is in the text and never by
/// what it is — the text may be a secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Untypable {
    /// Where in the text, counting characters from zero.
    pub at: usize,
}

/// The chord that types one character, or `None` where there is none.
#[must_use]
pub fn of(character: char) -> Option<Chord> {
    if character.is_ascii_lowercase() || character.is_ascii_digit() {
        return plain(character);
    }
    if character.is_ascii_uppercase() {
        return plain(character.to_ascii_lowercase()).map(|it| Chord::with("shift", it.key));
    }
    let (key, shifted) = match character {
        ' ' => ("spc", false),
        '-' => ("minus", false),
        '_' => ("minus", true),
        '=' => ("equal", false),
        '+' => ("equal", true),
        '.' => ("dot", false),
        '>' => ("dot", true),
        ',' => ("comma", false),
        '<' => ("comma", true),
        '/' => ("slash", false),
        '?' => ("slash", true),
        ';' => ("semicolon", false),
        ':' => ("semicolon", true),
        '\'' => ("apostrophe", false),
        '"' => ("apostrophe", true),
        '[' => ("bracket_left", false),
        '{' => ("bracket_left", true),
        ']' => ("bracket_right", false),
        '}' => ("bracket_right", true),
        '\\' => ("backslash", false),
        '|' => ("backslash", true),
        '`' => ("grave_accent", false),
        '~' => ("grave_accent", true),
        '!' => ("1", true),
        '@' => ("2", true),
        '#' => ("3", true),
        '$' => ("4", true),
        '%' => ("5", true),
        '^' => ("6", true),
        '&' => ("7", true),
        '*' => ("8", true),
        '(' => ("9", true),
        ')' => ("0", true),
        _ => return None,
    };
    Some(if shifted {
        Chord::with("shift", key)
    } else {
        Chord::key(key)
    })
}

/// The chords that type a whole text, or where the first untypable character
/// is.
///
/// # Errors
///
/// [`Untypable`] at the first character with no key, so nothing is typed at
/// all rather than most of it.
pub fn typing(text: &str) -> Result<Vec<Chord>, Untypable> {
    text.chars()
        .enumerate()
        .map(|(at, character)| of(character).ok_or(Untypable { at }))
        .collect()
}

/// A lowercase letter or a digit, whose `qcode` is itself.
fn plain(character: char) -> Option<Chord> {
    const LETTERS_AND_DIGITS: [&str; 36] = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    ];
    LETTERS_AND_DIGITS
        .iter()
        .find(|it| it.chars().next() == Some(character))
        .map(|it| Chord::key(*it))
}

#[cfg(test)]
#[path = "keys_tests.rs"]
mod tests;
