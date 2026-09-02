//! The key a shortcut is bound to, and the closed list of the ones that can be.
//!
//! **A key here is the one that prints this character on the keyboard in front
//! of the person**, not a position on an American keyboard. [`Key::Q`] is the Q
//! key: on a French keyboard it is where an English one has A, and a French
//! person setting `Super+Q` presses the key marked Q and gets `Super+Q`. Binding
//! to positions would mean a shortcut a person cannot read off their own
//! keyboard, and in a product whose first target is 24 languages that is not a
//! detail.
//!
//! It leaves one thing for whatever reads the keyboard, and it is written here
//! rather than discovered later: **a layout that prints no Latin letters** —
//! Greek, Bulgarian, Ukrainian — has no key marked Q at all. Every desktop
//! answers this the same way, by matching the shortcut against the Latin layout
//! the person also has, and the compositor is where that lookup belongs. This
//! type is what it produces, not how it gets there.
//!
//! **The list is closed** and it is short on purpose. Everything in it is a key
//! a person could sensibly ask a system shortcut to be built on; the volume and
//! brightness keys are not here because they are not chords and will arrive with
//! the status area that shows what they changed.
//!
//! One macro declares the list once. Three parallel lists — the variants, the
//! labels, and the array a settings panel walks — would drift, and the way that
//! drift shows up is a key nobody can pick from a list that still compiles.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Declares the closed list of keys, what each is called, and the array of all
/// of them, from one list of pairs.
macro_rules! keys {
    ($($variant:ident => $label:literal),* $(,)?) => {
        /// A key a shortcut can be built on.
        ///
        /// The names are a stored format — they are what a settings file holds
        /// — so they change additively or not at all.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub enum Key {
            $(
                #[doc = concat!("The ", $label, " key.")]
                $variant,
            )*
        }

        impl Key {
            /// Every key a shortcut can be built on, in the order a settings
            /// panel offers them.
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];

            /// What this key is called where a person can see it.
            #[must_use]
            pub fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),*
                }
            }
        }
    };
}

keys! {
    A => "A",
    B => "B",
    C => "C",
    D => "D",
    E => "E",
    F => "F",
    G => "G",
    H => "H",
    I => "I",
    J => "J",
    K => "K",
    L => "L",
    M => "M",
    N => "N",
    O => "O",
    P => "P",
    Q => "Q",
    R => "R",
    S => "S",
    T => "T",
    U => "U",
    V => "V",
    W => "W",
    X => "X",
    Y => "Y",
    Z => "Z",
    Digit0 => "0",
    Digit1 => "1",
    Digit2 => "2",
    Digit3 => "3",
    Digit4 => "4",
    Digit5 => "5",
    Digit6 => "6",
    Digit7 => "7",
    Digit8 => "8",
    Digit9 => "9",
    F1 => "F1",
    F2 => "F2",
    F3 => "F3",
    F4 => "F4",
    F5 => "F5",
    F6 => "F6",
    F7 => "F7",
    F8 => "F8",
    F9 => "F9",
    F10 => "F10",
    F11 => "F11",
    F12 => "F12",
    Space => "Space",
    Tab => "Tab",
    Enter => "Enter",
    Escape => "Escape",
    Backspace => "Backspace",
    Delete => "Delete",
    Insert => "Insert",
    Home => "Home",
    End => "End",
    PageUp => "Page Up",
    PageDown => "Page Down",
    Left => "Left",
    Right => "Right",
    Up => "Up",
    Down => "Down",
    Print => "Print",
    Comma => ",",
    Period => ".",
    Slash => "/",
    Minus => "-",
    Equals => "=",
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every key can be shown, and no two of them are shown the same way. A
    /// picker with two rows reading `F1` is a picker nobody can use.
    #[test]
    fn every_key_has_a_label_of_its_own() {
        let mut labels = BTreeSet::new();
        for key in Key::ALL {
            let label = key.label();
            assert!(!label.is_empty(), "{key:?}");
            assert!(labels.insert(label), "two keys are both called {label}");
        }
        assert_eq!(labels.len(), Key::ALL.len());
    }

    /// The list is one list: the array a panel walks and the variants that
    /// exist cannot come apart, because the macro writes both from the same
    /// line.
    #[test]
    fn the_array_holds_each_key_once() {
        let unique: BTreeSet<Key> = Key::ALL.iter().copied().collect();
        assert_eq!(unique.len(), Key::ALL.len());
        assert!(Key::ALL.contains(&Key::Q));
        assert!(Key::ALL.contains(&Key::PageDown));
    }

    /// A settings file holds the name, not the number, so a key means the same
    /// thing after a release that adds one.
    #[test]
    fn a_settings_file_holds_the_name() {
        assert_eq!(serde_json::to_string(&Key::PageUp).unwrap(), r#""PageUp""#);
        assert_eq!(
            serde_json::from_str::<Key>(r#""Digit4""#).unwrap(),
            Key::Digit4
        );
        assert!(serde_json::from_str::<Key>(r#""AnyOldKey""#).is_err());
    }

    /// What a person reads is the character on the key, and for the keys that
    /// print nothing, the word for it.
    #[test]
    fn a_key_is_shown_as_it_is_marked() {
        assert_eq!(Key::Q.to_string(), "Q");
        assert_eq!(Key::Digit7.to_string(), "7");
        assert_eq!(Key::Comma.to_string(), ",");
        assert_eq!(Key::PageDown.to_string(), "Page Down");
    }
}
