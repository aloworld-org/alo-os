//! The keys held down while another one is pressed.
//!
//! Four of them, and the list is closed for the same reason every other list in
//! this repository is: a person setting a shortcut picks from what is here, and
//! a settings panel that could offer a fifth would be offering something the
//! compositor cannot deliver.
//!
//! **Shift is a modifier that does not modify.** Holding it turns `2` into `@`
//! and `a` into `A`, which is a thing the keyboard already does on its own, so
//! Shift alone leaves a key still meaning what it was going to mean. That is why
//! [`Modifier::changes_the_meaning`] exists and why [`Modifiers::enough`] is
//! asked before a chord is allowed to exist — the reasoning, and the refusal it
//! causes, are in [`crate::chord`].
//!
//! **The order is fixed.** Held modifiers are shown Super, Ctrl, Alt, Shift
//! whatever order they were pressed in, because a person looking for the
//! shortcut they set should find the same words every time they look.
//!
//! **Three of the four are printed differently on the keyboards this will run
//! on** — a German keyboard says *Strg* and a French one *Maj* — so what a
//! modifier is called is a string like any other, and the only road to it is
//! [`Modifier::said`]. Neither type here has a `Display`; the reasoning is in
//! [`crate::action`], and what replaces it for a whole set is
//! [`Modifiers::shown`].

use std::fmt;

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// One key that changes what another key does.
///
/// The names are a stored format — they are what a settings file holds — so
/// they change additively or not at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Modifier {
    /// The key between Ctrl and Alt, printed with a Windows logo on most of the
    /// machines this will run on and called Super everywhere in software.
    Super,
    /// Ctrl.
    Ctrl,
    /// Alt.
    Alt,
    /// Shift.
    Shift,
}

impl Modifier {
    /// Every modifier there is, in the order they are shown in.
    pub const ALL: [Self; 4] = [Self::Super, Self::Ctrl, Self::Alt, Self::Shift];

    /// The string this crate declares for it.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::Super => words::SUPER,
            Self::Ctrl => words::CTRL,
            Self::Alt => words::ALT,
            Self::Shift => words::SHIFT,
        }
    }

    /// What this is called where a person can see it, in the language they
    /// read.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// Whether holding this makes a key mean something other than itself.
    ///
    /// True of Super, Ctrl and Alt. False of Shift, which produces a capital
    /// letter or the character above the digit — something a person types on
    /// purpose all day.
    #[must_use]
    pub fn changes_the_meaning(self) -> bool {
        !matches!(self, Self::Shift)
    }
}

/// The modifiers held down at one moment.
///
/// A set rather than a list: holding Ctrl twice is not a thing, and two chords
/// that differ only in the order somebody pressed Ctrl and Alt are the same
/// chord. Stored as a list of names so a settings file reads as
/// `["Super", "Shift"]` rather than as four booleans, and normalised on the way
/// in so a hand-written file with duplicates or a different order still means
/// what it says.
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "Vec<Modifier>", into = "Vec<Modifier>")]
pub struct Modifiers {
    /// Super held.
    super_held: bool,
    /// Ctrl held.
    ctrl: bool,
    /// Alt held.
    alt: bool,
    /// Shift held.
    shift: bool,
}

impl Modifiers {
    /// Nothing held.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            super_held: false,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    /// This one held, and nothing else.
    #[must_use]
    pub const fn just(modifier: Modifier) -> Self {
        Self::none().and(modifier)
    }

    /// These, and that one as well.
    #[must_use]
    pub const fn and(self, modifier: Modifier) -> Self {
        match modifier {
            Modifier::Super => Self {
                super_held: true,
                ..self
            },
            Modifier::Ctrl => Self { ctrl: true, ..self },
            Modifier::Alt => Self { alt: true, ..self },
            Modifier::Shift => Self {
                shift: true,
                ..self
            },
        }
    }

    /// Whether this one is held.
    #[must_use]
    pub const fn holds(self, modifier: Modifier) -> bool {
        match modifier {
            Modifier::Super => self.super_held,
            Modifier::Ctrl => self.ctrl,
            Modifier::Alt => self.alt,
            Modifier::Shift => self.shift,
        }
    }

    /// What is held, in the order it is shown in.
    pub fn held(self) -> impl Iterator<Item = Modifier> {
        Modifier::ALL
            .into_iter()
            .filter(move |modifier| self.holds(*modifier))
    }

    /// Whether nothing at all is held.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.held().next().is_none()
    }

    /// Whether what is held is enough to make a key mean something other than
    /// the character it prints.
    ///
    /// Shift on its own is not: `Shift+2` is `@`, and a shortcut bound to it
    /// would take `@` away from every mail address anybody types.
    #[must_use]
    pub fn enough(self) -> bool {
        self.held().any(Modifier::changes_the_meaning)
    }

    /// What is held, written the way a keyboard shortcut is written: the names
    /// in the reader's own language, joined with `+`, in the fixed order.
    /// Empty when nothing is held.
    ///
    /// A `String` rather than a `Said`, because it is composed rather than
    /// looked up — each name is a string of its own and whether any one of them
    /// was translated is [`Modifier::said`]'s to answer. **The `+` is not a
    /// string.** It is the notation every desktop writes a chord in, the same
    /// decision `alo-appearance` makes about a time of day being written
    /// `18:00` in a settings file whatever the region does.
    #[must_use]
    pub fn shown(self, strings: &Strings) -> String {
        let mut written = String::new();
        for modifier in self.held() {
            if !written.is_empty() {
                written.push('+');
            }
            written.push_str(modifier.said(strings).text());
        }
        written
    }
}

impl From<Vec<Modifier>> for Modifiers {
    fn from(held: Vec<Modifier>) -> Self {
        held.into_iter().fold(Self::none(), Self::and)
    }
}

impl From<Modifiers> for Vec<Modifier> {
    fn from(modifiers: Modifiers) -> Self {
        modifiers.held().collect()
    }
}

impl fmt::Debug for Modifiers {
    /// The names a settings file holds, joined with `+` — `Super+Shift` rather
    /// than four booleans.
    ///
    /// Hand-written because the derived one is four fields of which three are
    /// usually `false`, and because this is the rendering a programmer reads:
    /// stable names that are nobody's language, which is what
    /// [`crate::DefaultsError`] says a contradictory set of defaults with.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for modifier in self.held() {
            if !first {
                f.write_str("+")?;
            }
            write!(f, "{modifier:?}")?;
            first = false;
        }
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Holding something and then asking about it agrees, for all four.
    #[test]
    fn what_is_held_is_what_was_pressed() {
        for modifier in Modifier::ALL {
            let held = Modifiers::just(modifier);
            assert!(held.holds(modifier), "{modifier:?}");
            for other in Modifier::ALL {
                assert_eq!(
                    held.holds(other),
                    other == modifier,
                    "{other:?} in {held:?}"
                );
            }
        }
    }

    /// A set, not a list: pressing the same key twice changes nothing, and the
    /// order it was pressed in is not remembered.
    #[test]
    fn the_same_modifier_twice_is_once_and_the_order_is_not_kept() {
        let one = Modifiers::just(Modifier::Ctrl)
            .and(Modifier::Ctrl)
            .and(Modifier::Alt);
        let other = Modifiers::just(Modifier::Alt).and(Modifier::Ctrl);
        assert_eq!(one, other);
        assert_eq!(one.held().count(), 2);
    }

    /// The words are the same whichever order they were pressed in, because a
    /// person looking for the shortcut they set has to find it twice.
    #[test]
    fn the_order_shown_is_always_the_same() {
        let strings = in_english();
        let held = Modifiers::just(Modifier::Shift)
            .and(Modifier::Alt)
            .and(Modifier::Ctrl)
            .and(Modifier::Super);
        assert_eq!(held.shown(&strings), "Super+Ctrl+Alt+Shift");
        assert_eq!(Modifiers::none().shown(&strings), "");
        assert!(Modifiers::none().is_empty());
    }

    /// **What is held is read in the person's own language.** A German keyboard
    /// prints Strg where an English one prints Ctrl, so a shortcuts panel that
    /// said Ctrl would be naming a key that is not on the keyboard in front of
    /// them.
    #[test]
    fn what_is_held_is_read_in_the_persons_own_language() {
        let strings = translated(&[(words::CTRL, "Strg"), (words::SHIFT, "Umschalt")]);
        let held = Modifiers::just(Modifier::Ctrl).and(Modifier::Shift);
        assert_eq!(held.shown(&strings), "Strg+Umschalt");
        assert!(Modifier::Ctrl.said(&strings).is_translated());

        // Alt is Alt on a German keyboard, and nobody had to translate it for
        // the sentence to read — it just says it was not translated.
        let alt = Modifier::Alt.said(&strings);
        assert_eq!(alt.text(), "Alt");
        assert!(!alt.is_translated());
    }

    /// A programmer reading a set reads the names a settings file holds, which
    /// are nobody's language and do not move when a translation arrives.
    #[test]
    fn a_programmer_reads_the_names_the_file_holds() {
        let held = Modifiers::just(Modifier::Super).and(Modifier::Shift);
        assert_eq!(format!("{held:?}"), "Super+Shift");
        assert_eq!(format!("{:?}", Modifiers::none()), "");
    }

    /// **Shift does not change what a key means**, it changes which character
    /// the key prints — so it is never enough on its own, and any of the other
    /// three is.
    #[test]
    fn shift_alone_is_not_enough_and_the_others_are() {
        assert!(!Modifiers::none().enough());
        assert!(!Modifiers::just(Modifier::Shift).enough());
        assert!(!Modifier::Shift.changes_the_meaning());
        for modifier in [Modifier::Super, Modifier::Ctrl, Modifier::Alt] {
            assert!(modifier.changes_the_meaning(), "{modifier:?}");
            assert!(Modifiers::just(modifier).enough(), "{modifier:?}");
            assert!(
                Modifiers::just(modifier).and(Modifier::Shift).enough(),
                "{modifier:?}"
            );
        }
    }

    /// A settings file holds the names, and a hand-written one that repeats
    /// itself or lists them backwards still means what it says.
    #[test]
    fn a_settings_file_holds_names_and_an_untidy_one_still_reads() {
        let held = Modifiers::just(Modifier::Super).and(Modifier::Shift);
        let written = serde_json::to_string(&held).unwrap();
        assert_eq!(written, r#"["Super","Shift"]"#);
        assert_eq!(serde_json::from_str::<Modifiers>(&written).unwrap(), held);

        let untidy: Modifiers = serde_json::from_str(r#"["Shift","Super","Shift"]"#).unwrap();
        assert_eq!(untidy, held);
    }
}
