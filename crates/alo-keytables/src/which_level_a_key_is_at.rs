//! Which of a key's four characters the modifiers select.
//!
//! # The rule, and where it comes from
//!
//! It is not invented here. It is read off the key *types* in a compiled xkb
//! keymap, measured on 2026-10-20:
//!
//! ```text
//! type "ONE_LEVEL"  { modifiers= none; }
//! type "TWO_LEVEL"  { modifiers= Shift;       map[Shift]= 2; }
//! type "ALPHABETIC" { modifiers= Shift+Lock;  map[Shift]= 2; map[Lock]= 2; }
//! ```
//!
//! and, for the keys that carry four characters:
//!
//! ```text
//! type "FOUR_LEVEL_ALPHABETIC" {
//!     modifiers= Shift+Lock+LevelThree;
//!     map[Shift]= 2;  map[Lock]= 2;
//!     map[LevelThree]= 3;  map[Shift+LevelThree]= 4;
//!     map[Lock+LevelThree]= 4;  map[Shift+Lock+LevelThree]= 3;
//! }
//! ```
//!
//! Three things follow, and together they are the whole of this module:
//!
//! 1. **A modifier a key's type does not list is ignored by that key.** Lock is
//!    absent from `TWO_LEVEL`, so CapsLock does nothing at all to a digit key —
//!    it does not shift it, and it does not stop it typing. `ONE_LEVEL` lists
//!    nothing, so the space bar types a space with anything held down.
//! 2. **Shift and Lock both map to the upper half of a pair, so together they
//!    cancel.** That is why CapsLock plus Shift on a letter gives the small
//!    letter back, which is what every keyboard does and what nobody writes
//!    down.
//! 3. **CapsLock shifts a pair only where that pair is a case pair.** A key's
//!    four characters are two pairs — the lower one AltGr-off, the upper one
//!    AltGr-on — and the same question is asked of whichever pair is in play.
//!
//! Rule 3 is why the table above is `FOUR_LEVEL_ALPHABETIC` and the keys are
//! not. `ALPHABETIC` sends Lock+AltGr to level 4, and **the engine's own
//! answers say level 3** for the Belgian Q key, whose upper pair is `@` and `Ω`
//! — not a case pair. That key is `FOUR_LEVEL_SEMIALPHABETIC`, where the first
//! pair counts Lock and the second does not. Rather than carry two type names,
//! rule 3 asks the question the names are shorthand for, which also answers a
//! key whose upper pair *is* a case pair without anybody having to notice one
//! exists.
//!
//! # How a key's type is known without reading it
//!
//! A compiled keymap names a key's type only where it is not the canonical one
//! for its shape, so for most keys the type is not in the text at all. What
//! *is* in the text is the characters, and the three rules above are entirely
//! questions about those:
//!
//! | the pair in play carries | it uses |
//! |---|---|
//! | an upper half | Shift |
//! | an upper half that is the capital of its lower half | Shift and Lock |
//! | nothing at all, as the AltGr pair of most keys | neither, so AltGr is ignored |
//!
//! So [`which_level`] takes the key's characters rather than a flag, and
//! applies rule 1 by **dropping every modifier the key does not use** before it
//! works anything out.
//!
//! # Why this is computed rather than tabulated
//!
//! The first version of this crate tabulated it: the generator asked
//! `xkbcommon` *how do I type this character* for a thousand characters on each
//! layout, and filed each answer under the modifiers that reached it. **That
//! produced a table with holes exactly where rule 1 applies** — pressing the
//! Belgian ampersand key with CapsLock on had no row, because no character is
//! reached *by* CapsLock on that key, so no answer ever mentioned it. A person
//! with CapsLock on would have typed nothing at all into a Wi-Fi password, and
//! every test about drawing the screen would still have passed.
//!
//! Both holes were found by tests asserting what a keyboard does — a letter
//! *and* a digit under four combinations each, and the space bar with
//! everything held — because a pair is what separates a right rule from one
//! that happens to agree.
//!
//! # What is deliberately not here
//!
//! NumLock, which is the keypad's own business and which
//! [`crate::what_a_key_types`] says outright is not covered.

use crate::what_is_held_down::Held;

/// How many characters a key can carry.
///
/// Four, because that is what xkb's four-level types hold and what every one of
/// the eight layouts uses at most: plain, Shift, AltGr, and AltGr with Shift.
pub const HOW_MANY_LEVELS: usize = 4;

/// What a key carries, in level order, with `None` where it carries nothing.
pub type Levels = [Option<char>; HOW_MANY_LEVELS];

/// Which of this key's four characters these modifiers select.
///
/// The answer is always nought to three. A level that turns out to be empty is
/// not this function's to solve: whether the key carries anything there is a
/// fact about the key, and the caller has the key.
///
/// # Examples
///
/// The space bar carries one character and so uses no modifier at all:
///
/// ```
/// use alo_keytables::which_level_a_key_is_at::which_level;
/// use alo_keytables::Held;
///
/// const LEFT_SHIFT: u16 = 42;
/// const ALT_GR: u16 = 100;
///
/// let space = [Some(' '), None, None, None];
/// let held = Held::nothing()
///     .and_this_went_down(LEFT_SHIFT)
///     .and_this_went_down(ALT_GR);
///
/// assert_eq!(which_level(&space, held), 0);
/// ```
#[must_use]
pub fn which_level(levels: &Levels, held: Held) -> usize {
    // Rule 1, for AltGr: a key with nothing on either of its upper two levels
    // does not use AltGr at all, and ignores it.
    let altgr = held.altgr() && (levels[2].is_some() || levels[3].is_some());
    let pair = if altgr { 2 } else { 0 };

    // Rule 1, for Shift: this pair uses Shift only if it has an upper half.
    let shift = held.shift() && levels[pair + 1].is_some();
    // Rule 3: CapsLock shifts a pair only where that pair is a case pair.
    let lock = held.lock() && is_a_case_pair(levels[pair], levels[pair + 1]);

    // Rule 2: Shift and Lock both reach the upper half, so together they
    // return to the lower one.
    pair + usize::from(shift != lock)
}

/// Whether these two characters are the same letter in both cases.
///
/// This is the whole of rule 3, and it is what xkb's type names say in words:
/// `ALPHABETIC` means the upper half of the pair is the capital of the lower,
/// and `SEMIALPHABETIC` means the first pair is and the second is not.
///
/// # Why it asks about case and not about the letter
///
/// The obvious test is *is the upper half the lower half capitalised*, and it
/// disagrees with the engine twice over:
///
/// | the pair | that test | what `xkbcommon` does |
/// |---|---|---|
/// | `ß` and `ẞ` | **no** — Rust's `char::to_uppercase` is Unicode's *full* mapping, where `ß` becomes `SS` | counts Lock |
/// | `ſ` and `ẞ` — the German S key's AltGr pair | **no**, and they are genuinely different letters | counts Lock |
///
/// Both were found by comparing against the engine's own answers rather than
/// by reading about it, and together they say what its rule actually is: a
/// **lower-case character under an upper-case one**, whether or not they are
/// the same letter. That is what is asked here, so this crate agrees with the
/// machine a person will be using rather than with a tidier rule.
///
/// It is still false of `&` and `1`, of `é` and `2`, and of `ß` and `?`, which
/// are the pairs that matter for CapsLock being ignored.
#[must_use]
pub fn is_a_case_pair(lower: Option<char>, upper: Option<char>) -> bool {
    let (Some(lower), Some(upper)) = (lower, upper) else {
        return false;
    };
    lower.is_lowercase() && upper.is_uppercase()
}

/// Whether CapsLock counts on this key's first pair.
///
/// The Belgian `é` key is not alphabetic by this test — its second level is
/// `2` — and xkb agrees, which is the sort of case the derivation is checked
/// against rather than assumed over.
#[must_use]
pub fn is_alphabetic(levels: &Levels) -> bool {
    is_a_case_pair(levels[0], levels[1])
}

#[cfg(test)]
mod tests {
    use super::{Levels, is_alphabetic, which_level};
    use crate::what_is_held_down::{ALT_GR, CAPS_LOCK, Held, LEFT_SHIFT};

    /// The Belgian Q position: a letter with a third and fourth level.
    const A_LETTER: Levels = [Some('a'), Some('A'), Some('@'), Some('Ω')];
    /// The Belgian 1 position: not a letter, and with a third and fourth level.
    const A_DIGIT: Levels = [Some('&'), Some('1'), Some('|'), Some('¡')];
    /// The US Q position: a letter with no third level at all.
    const TWO_LEVELS_ONLY: Levels = [Some('q'), Some('Q'), None, None];
    /// The space bar.
    const ONE_LEVEL_ONLY: Levels = [Some(' '), None, None, None];

    fn with(shift: bool, lock: bool, altgr: bool) -> Held {
        let mut held = Held::nothing();
        if shift {
            held = held.and_this_went_down(LEFT_SHIFT);
        }
        if lock {
            held = held.and_this_went_down(CAPS_LOCK);
        }
        if altgr {
            held = held.and_this_went_down(ALT_GR);
        }
        held
    }

    #[test]
    fn nothing_held_is_the_first_level() {
        for levels in [A_LETTER, A_DIGIT, TWO_LEVELS_ONLY, ONE_LEVEL_ONLY] {
            assert_eq!(which_level(&levels, Held::nothing()), 0);
        }
    }

    #[test]
    fn shift_reaches_the_second_level_on_any_key_that_has_one() {
        assert_eq!(which_level(&A_LETTER, with(true, false, false)), 1);
        assert_eq!(which_level(&A_DIGIT, with(true, false, false)), 1);
        assert_eq!(which_level(&TWO_LEVELS_ONLY, with(true, false, false)), 1);
    }

    #[test]
    fn caps_lock_shifts_a_letter_and_is_ignored_by_everything_else() {
        // The pair that separates this rule from one that happens to agree
        // with it: the same modifier, two kinds of key, two answers.
        assert_eq!(which_level(&A_LETTER, with(false, true, false)), 1);
        assert_eq!(
            which_level(&A_DIGIT, with(false, true, false)),
            0,
            "CapsLock moved a key whose type does not list it"
        );
    }

    #[test]
    fn shift_and_caps_lock_cancel_on_a_letter_and_do_not_on_anything_else() {
        assert_eq!(
            which_level(&A_LETTER, with(true, true, false)),
            0,
            "a letter with CapsLock on and Shift held did not come back small"
        );
        assert_eq!(
            which_level(&A_DIGIT, with(true, true, false)),
            1,
            "CapsLock cancelled Shift on a key that ignores CapsLock"
        );
    }

    #[test]
    fn altgr_reaches_the_third_level_and_with_shift_the_fourth() {
        assert_eq!(which_level(&A_LETTER, with(false, false, true)), 2);
        assert_eq!(which_level(&A_LETTER, with(true, false, true)), 3);
    }

    #[test]
    fn caps_lock_is_ignored_on_an_upper_pair_that_is_not_a_case_pair() {
        // Rule 3, and the assertion here was the wrong way round until the
        // engine's own answers were compared against it. The Belgian Q key's
        // upper pair is `@` and omega: AltGr with CapsLock gives `@`, because
        // CapsLock has nothing to shift between those two.
        assert_eq!(which_level(&A_LETTER, with(false, true, true)), 2);
        assert_eq!(which_level(&A_LETTER, with(true, true, true)), 3);
        assert_eq!(which_level(&A_DIGIT, with(false, true, true)), 2);
    }

    #[test]
    fn caps_lock_shifts_an_upper_pair_that_is_a_case_pair() {
        // The other side of rule 3, which asks the question xkb's two type
        // names are shorthand for rather than carrying the names. Nothing
        // across the eight layouts had to have such a key for this to be the
        // right rule, and asking is what makes it right either way.
        let both_pairs: Levels = [Some('a'), Some('A'), Some('ä'), Some('Ä')];
        assert_eq!(which_level(&both_pairs, with(false, true, true)), 3);
        assert_eq!(which_level(&both_pairs, with(true, true, true)), 2);
    }

    #[test]
    fn a_modifier_a_key_does_not_use_is_ignored_rather_than_swallowing_the_key() {
        // This is rule 1, and it is the one the first version of this crate got
        // wrong in three separate places.
        assert_eq!(
            which_level(&TWO_LEVELS_ONLY, with(false, false, true)),
            0,
            "AltGr on a key with no third level did not fall back to the first"
        );
        assert_eq!(
            which_level(&TWO_LEVELS_ONLY, with(true, false, true)),
            1,
            "AltGr with Shift on a two-level key did not land on its Shift level"
        );
        for shift in [false, true] {
            for lock in [false, true] {
                for altgr in [false, true] {
                    assert_eq!(
                        which_level(&ONE_LEVEL_ONLY, with(shift, lock, altgr)),
                        0,
                        "the space bar stopped typing a space with {shift} {lock} {altgr} held"
                    );
                }
            }
        }
    }

    #[test]
    fn every_combination_lands_on_one_of_the_four() {
        for levels in [A_LETTER, A_DIGIT, TWO_LEVELS_ONLY, ONE_LEVEL_ONLY] {
            for shift in [false, true] {
                for lock in [false, true] {
                    for altgr in [false, true] {
                        let level = which_level(&levels, with(shift, lock, altgr));
                        assert!(level < 4, "{shift} {lock} {altgr} gave level {level}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_letter_key_is_alphabetic_and_a_digit_key_is_not() {
        assert!(is_alphabetic(&A_LETTER));
        assert!(is_alphabetic(&TWO_LEVELS_ONLY));
        assert!(!is_alphabetic(&A_DIGIT));
        // The Belgian 2 position: the second level is not the capital of the
        // first, so CapsLock does not count on it.
        assert!(!is_alphabetic(&[Some('é'), Some('2'), Some('@'), None]));
        // The German sharp s: its second level is a question mark.
        assert!(!is_alphabetic(&[Some('ß'), Some('?'), Some('\\'), None]));
        assert!(!is_alphabetic(&ONE_LEVEL_ONLY));
        assert!(!is_alphabetic(&[None, None, None, None]));
    }

    #[test]
    fn an_accented_letter_is_alphabetic_when_its_capital_is_on_the_key() {
        // Nothing about this rule is about the English alphabet: a key
        // carrying e-acute and its capital counts CapsLock like any other
        // letter, which is what a person in Liege or Malmo needs.
        assert!(is_alphabetic(&[Some('é'), Some('É'), None, None]));
        assert!(is_alphabetic(&[Some('ø'), Some('Ø'), None, None]));
        assert!(is_alphabetic(&[Some('ż'), Some('Ż'), None, None]));
    }
}
