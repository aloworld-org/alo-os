//! The one question this crate exists to answer: a person pressed this key
//! with these modifiers on this layout — what did they type?
//!
//! # Why this is a lookup and not a calculation
//!
//! Turning a keycode into a letter is, done properly, an engine: xkb's include
//! directives, variant selection, levels, groups, and the `types` and `compat`
//! directories beside `symbols`. The Belgian layout is mostly
//! `include "fr(basic)"` plus overrides, so even reading one layout correctly
//! means implementing all of that.
//!
//! This repository configures engines and does not write them, so the engine
//! that owns the question answers it — `xkbcommon`, at generation time — and
//! what is left here is reading a row out of
//! [`crate::the_table_xkbcommon_gave`] and choosing one of its four characters
//! by [`crate::which_level_a_key_is_at`]. That is why the install environment
//! needs no C library and no parser of anybody else's format for this.
//!
//! # What a `None` means, and what it does not
//!
//! `None` is *this key types nothing on this layout with these modifiers*. It
//! is the right answer for Enter, for an arrow, for Shift itself, and for the
//! function row — none of which puts a character anywhere. A caller that wants
//! to know whether the key was a modifier asks
//! [`crate::what_is_held_down::Held::is_one_of_the_three`], because *it
//! modified something* and *it typed nothing* are different facts and only one
//! of them is this module's.
//!
//! **It is never a fallback to another layout.** A layout this crate does not
//! carry does not quietly become `us`:
//! [`crate::the_eight_layouts::TheLayout`] has eight variants and no ninth, so
//! there is nothing to fall back to. That matters because the one thing worse
//! than refusing to type a Wi-Fi password is typing a different one and
//! reporting that the router refused it.
//!
//! # What is not covered, said plainly
//!
//! **The keypad.** Its keys carry actions rather than characters in a compiled
//! keymap — `KP_1` and the rest — and NumLock is a fourth modifier this crate
//! does not read. A person typing a password on the number pad types nothing.
//! The main block is whole, which is what the environment's screens ask for,
//! and this is named here rather than discovered.
//!
//! **Dead keys and compose sequences**, which are `alo-keyboards`' chain.

use crate::the_eight_layouts::TheLayout;
use crate::the_table_xkbcommon_gave::THE_TABLE;
use crate::what_is_held_down::Held;
use crate::which_level_a_key_is_at::{Levels, which_level};

/// The four characters this key carries on this layout, in level order.
///
/// `None` for a key the layout puts nothing on, which is most of a keyboard.
#[must_use]
pub fn what_a_key_carries(layout: TheLayout, keycode: u16) -> Option<&'static Levels> {
    let wanted = (layout.as_the_table_numbers_it(), keycode);
    THE_TABLE
        .binary_search_by_key(&wanted, |&(which, code, _)| (which, code))
        .ok()
        .and_then(|at| THE_TABLE.get(at))
        .map(|(_, _, levels)| levels)
}

/// What a person typed by pressing this key with these modifiers.
///
/// `keycode` is the kernel's own code for the key — `KEY_A` is 30 whatever is
/// printed on it — exactly as `/dev/input/event*` reports it, with no offset
/// applied. (xkb numbers the same key eight higher; the generator subtracts
/// that so no caller has to know.)
///
/// Which of the key's four characters comes out is
/// [`crate::which_level_a_key_is_at`]'s decision, and the rule it applies — *a
/// modifier the key does not use is ignored* — is why AltGr and the Q key type
/// `q` on the plain US layout, and why the space bar types a space with
/// everything held down.
///
/// # Examples
///
/// The physical key marked Q on a US keyboard, on the Belgian layout, types
/// `a` — and `@` on the third level, which is how a Belgian types an email
/// address:
///
/// ```
/// use alo_keytables::{Held, TheLayout, what_a_key_types};
///
/// const THE_Q_POSITION: u16 = 16;
///
/// assert_eq!(
///     what_a_key_types(TheLayout::Be, THE_Q_POSITION, Held::nothing()),
///     Some('a')
/// );
/// assert_eq!(
///     what_a_key_types(TheLayout::Us, THE_Q_POSITION, Held::nothing()),
///     Some('q')
/// );
/// ```
///
/// Enter types nothing, on every layout:
///
/// ```
/// use alo_keytables::{Held, TheLayout, what_a_key_types};
///
/// const ENTER: u16 = 28;
///
/// for layout in TheLayout::ALL {
///     assert_eq!(what_a_key_types(layout, ENTER, Held::nothing()), None);
/// }
/// ```
#[must_use]
pub fn what_a_key_types(layout: TheLayout, keycode: u16, held: Held) -> Option<char> {
    let levels = what_a_key_carries(layout, keycode)?;
    levels[which_level(levels, held)]
}

/// Every key this layout types something on, lowest keycode first.
///
/// What this is for: a layout screen that shows a person their own keyboard so
/// they can see it is the right one before typing a password into it, and the
/// test below that no layout came out empty. A layout with no rows at all would
/// make every screen that asks a question silently unusable while every test
/// about *drawing* it still passed, which is the shape this repository keeps
/// finding.
#[must_use]
pub fn every_key_that_types_something(layout: TheLayout) -> Vec<u16> {
    let which = layout.as_the_table_numbers_it();
    THE_TABLE
        .iter()
        .filter(|&&(this, _, _)| this == which)
        .map(|&(_, code, _)| code)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{every_key_that_types_something, what_a_key_carries, what_a_key_types};
    use crate::the_eight_layouts::TheLayout;
    use crate::the_table_xkbcommon_gave::THE_TABLE;
    use crate::what_is_held_down::{ALT_GR, CAPS_LOCK, Held, LEFT_SHIFT};

    /// The physical key a US keyboard marks Q.
    const THE_Q_POSITION: u16 = 16;
    /// The physical key a US keyboard marks 1.
    const THE_ONE_POSITION: u16 = 2;

    #[test]
    fn the_table_is_sorted_or_the_lookup_is_a_lie() {
        // `binary_search_by_key` on unsorted data does not fail; it returns the
        // wrong row or no row, and every test below would pass or fail for
        // reasons that have nothing to do with any layout. So this is checked
        // first and separately.
        let keys: Vec<(u8, u16)> = THE_TABLE
            .iter()
            .map(|&(which, code, _)| (which, code))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(keys, sorted, "the generated table is not in sorted order");
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len(), "one key has two rows");
    }

    #[test]
    fn no_row_in_the_table_is_empty() {
        // A row with four `None`s would be a key the generator thought was
        // worth writing down and that types nothing — which would mean the
        // parse of the keymap had gone wrong somewhere silently.
        for &(which, keycode, levels) in THE_TABLE {
            assert!(
                levels.iter().any(Option::is_some),
                "layout {which}, key {keycode} carries nothing at all"
            );
        }
    }

    #[test]
    fn the_same_key_types_different_letters_on_different_layouts() {
        // The control for every claim in this file: if these agreed, the
        // layout argument would not be doing anything and nothing else here
        // would be worth reading.
        assert_eq!(
            what_a_key_types(TheLayout::Be, THE_Q_POSITION, Held::nothing()),
            Some('a')
        );
        assert_eq!(
            what_a_key_types(TheLayout::Us, THE_Q_POSITION, Held::nothing()),
            Some('q')
        );
        assert_eq!(
            what_a_key_types(TheLayout::De, THE_Q_POSITION, Held::nothing()),
            Some('q')
        );
    }

    #[test]
    fn caps_lock_shifts_a_letter_and_leaves_a_digit_alone() {
        let lock = Held::nothing().and_this_went_down(CAPS_LOCK);
        let shift = Held::nothing().and_this_went_down(LEFT_SHIFT);
        let both = lock.and_this_went_down(LEFT_SHIFT);

        // The letter: CapsLock shifts it, and Shift puts it back.
        assert_eq!(
            what_a_key_types(TheLayout::Be, THE_Q_POSITION, lock),
            Some('A')
        );
        assert_eq!(
            what_a_key_types(TheLayout::Be, THE_Q_POSITION, both),
            Some('a')
        );

        // The digit: Shift reaches 1, and CapsLock reaches nothing new. This is
        // the pair a table built by asking *how do I type this character* gets
        // wrong — it has no row for CapsLock on this key at all, so the key
        // types nothing, in a Wi-Fi password where nobody can see it.
        assert_eq!(
            what_a_key_types(TheLayout::Be, THE_ONE_POSITION, Held::nothing()),
            Some('&')
        );
        assert_eq!(
            what_a_key_types(TheLayout::Be, THE_ONE_POSITION, shift),
            Some('1')
        );
        assert_eq!(
            what_a_key_types(TheLayout::Be, THE_ONE_POSITION, lock),
            Some('&'),
            "CapsLock changed a digit key, or left it typing nothing"
        );
        assert_eq!(
            what_a_key_types(TheLayout::Be, THE_ONE_POSITION, both),
            Some('1'),
            "CapsLock cancelled Shift on a key whose type ignores CapsLock"
        );
    }

    #[test]
    fn the_third_level_is_reached_by_altgr_where_the_key_has_one() {
        let altgr = Held::nothing().and_this_went_down(ALT_GR);
        // A Belgian types an email address with AltGr.
        assert_eq!(
            what_a_key_types(TheLayout::Be, THE_Q_POSITION, altgr),
            Some('@')
        );
        // And the fourth level with Shift as well.
        assert_eq!(
            what_a_key_types(
                TheLayout::Be,
                THE_Q_POSITION,
                altgr.and_this_went_down(LEFT_SHIFT)
            ),
            Some('Ω')
        );
        // The plain US layout has no third level on this key, so AltGr is a
        // modifier the key does not use, and it is ignored rather than
        // swallowing the keystroke.
        assert_eq!(
            what_a_key_types(TheLayout::Us, THE_Q_POSITION, altgr),
            Some('q'),
            "AltGr on a key with no third level lost the keystroke"
        );
    }

    #[test]
    fn a_key_that_types_nothing_says_so_on_every_layout() {
        // Enter, Escape, Backspace, Tab, the left Alt, and F1.
        for keycode in [28, 1, 14, 15, 56, 59] {
            for layout in TheLayout::ALL {
                assert_eq!(
                    what_a_key_types(layout, keycode, Held::nothing()),
                    None,
                    "{layout:?} claims keycode {keycode} types a character"
                );
            }
        }
    }

    #[test]
    fn space_types_a_space_everywhere_whatever_is_held() {
        const SPACE: u16 = 57;
        let everything = Held::nothing()
            .and_this_went_down(LEFT_SHIFT)
            .and_this_went_down(CAPS_LOCK)
            .and_this_went_down(ALT_GR);
        for layout in TheLayout::ALL {
            assert_eq!(
                what_a_key_types(layout, SPACE, Held::nothing()),
                Some(' '),
                "{layout:?} does not type a space"
            );
            assert_eq!(
                what_a_key_types(layout, SPACE, everything),
                Some(' '),
                "{layout:?} stopped typing a space when a modifier was held"
            );
        }
    }

    #[test]
    fn every_layout_can_type_a_password() {
        // Every letter, every digit, and the punctuation a router's own
        // default password is made of. A table short of one of these lets a
        // person get most of a password in and no further, which they can
        // only read as the password being refused.
        const AND_THESE: [char; 10] = ['-', '_', '.', ',', '@', '!', '?', '+', '=', ':'];

        for layout in TheLayout::ALL {
            let keys = every_key_that_types_something(layout);
            assert!(
                keys.len() > 40,
                "{layout:?} types on only {} keys, which is not a keyboard",
                keys.len()
            );

            let wanted = ('a'..='z')
                .chain('A'..='Z')
                .chain('0'..='9')
                .chain(AND_THESE);
            for character in wanted {
                let found = keys.iter().any(|&code| {
                    [false, true].into_iter().any(|shift| {
                        [false, true].into_iter().any(|altgr| {
                            let mut held = Held::nothing();
                            if shift {
                                held = held.and_this_went_down(LEFT_SHIFT);
                            }
                            if altgr {
                                held = held.and_this_went_down(ALT_GR);
                            }
                            what_a_key_types(layout, code, held) == Some(character)
                        })
                    })
                });
                assert!(found, "{layout:?} cannot type {character:?}");
            }
        }
    }

    #[test]
    fn the_keys_a_layout_types_on_are_listed_once_and_in_order() {
        for layout in TheLayout::ALL {
            let keys = every_key_that_types_something(layout);
            let mut sorted = keys.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(
                keys, sorted,
                "{layout:?} listed a key twice or out of order"
            );
        }
    }

    #[test]
    fn a_key_the_layout_puts_nothing_on_carries_nothing() {
        assert!(what_a_key_carries(TheLayout::Be, 28).is_none(), "Enter");
        assert!(
            what_a_key_carries(TheLayout::Be, THE_Q_POSITION).is_some(),
            "the Q position"
        );
    }
}
