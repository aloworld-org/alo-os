//! Which of the three modifiers is on, which is the whole of what a layout
//! needs to know to turn a keycode into a letter.
//!
//! # Why three and not the usual eight
//!
//! A keyboard has Control, Alt, Super and Meta too, and **none of them changes
//! what a key types.** `Ctrl+A` types nothing; it means something. The three
//! here are the three that select between the characters printed on one key:
//! Shift, CapsLock and AltGr. Every other modifier belongs to whoever reads the
//! chord, and this crate does not read chords.
//!
//! # Why CapsLock is kept apart from Shift rather than folded into it
//!
//! Because they are not the same, and the difference is per key rather than per
//! keyboard. Measured on the Belgian layout, 2026-10-20:
//!
//! | key | plain | Shift | CapsLock | Shift + CapsLock |
//! |---|---|---|---|---|
//! | the Q position | a | A | A | a |
//! | the 1 position | &amp; | 1 | &amp; | 1 |
//!
//! On the letter, CapsLock shifts and Shift un-shifts it again. On the digit,
//! CapsLock does nothing at all. That is xkb's key *types* — `ALPHABETIC`
//! against `TWO_LEVEL` — and the reason this crate needs none of that machinery
//! is that [`crate::the_table_xkbcommon_gave`] holds the answer for all eight
//! combinations of the three, as `xkbcommon` itself gave them. **So a key type
//! is never interpreted here; it was already applied by the engine that owns
//! it.**
//!
//! Folding CapsLock into Shift would have typed `1` where a person pressed the
//! ampersand key with CapsLock on, silently, in a Wi-Fi password — which the
//! environment could only report as *the password was refused*.

/// The evdev code of the left Shift key.
///
/// From `linux/input-event-codes.h`, read on 2026-10-20 rather than
/// remembered. These are kernel interface numbers and do not move.
pub const LEFT_SHIFT: u16 = 42;

/// The evdev code of the right Shift key.
pub const RIGHT_SHIFT: u16 = 54;

/// The evdev code of CapsLock.
pub const CAPS_LOCK: u16 = 58;

/// The evdev code of the right Alt key, which is AltGr where a layout has one.
///
/// **The left Alt is not this key**, on any of the eight layouts, which is why
/// it is absent here: on a layout with a third level, right Alt reaches it and
/// left Alt does not.
pub const ALT_GR: u16 = 100;

/// Which of the three modifiers is on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Held {
    /// Either Shift key is down.
    shift: bool,
    /// CapsLock is on — a latch, not a key being held.
    lock: bool,
    /// AltGr is down.
    altgr: bool,
}

impl Held {
    /// Nothing held and CapsLock off, which is where a keyboard starts.
    #[must_use]
    pub const fn nothing() -> Self {
        Self {
            shift: false,
            lock: false,
            altgr: false,
        }
    }

    /// Either Shift key is down.
    #[must_use]
    pub const fn shift(self) -> bool {
        self.shift
    }

    /// CapsLock is on.
    #[must_use]
    pub const fn lock(self) -> bool {
        self.lock
    }

    /// AltGr is down.
    #[must_use]
    pub const fn altgr(self) -> bool {
        self.altgr
    }

    /// What this becomes when a key goes down.
    ///
    /// Returns the state unchanged for every key that is not one of the three,
    /// so a caller may pass every key it reads and need no list of its own.
    ///
    /// **CapsLock latches on the press and not on the release**, which is what
    /// a keyboard does: one press turns it on, the next turns it off. Shift and
    /// AltGr are held, so they follow the key.
    #[must_use]
    pub const fn and_this_went_down(mut self, keycode: u16) -> Self {
        match keycode {
            LEFT_SHIFT | RIGHT_SHIFT => self.shift = true,
            ALT_GR => self.altgr = true,
            CAPS_LOCK => self.lock = !self.lock,
            _ => {}
        }
        self
    }

    /// What this becomes when a key comes up.
    ///
    /// CapsLock is untouched here on purpose: it was already latched by the
    /// press, and latching again on the release would turn it straight back
    /// off, so a person would never be able to switch it on at all.
    #[must_use]
    pub const fn and_this_came_up(mut self, keycode: u16) -> Self {
        match keycode {
            LEFT_SHIFT | RIGHT_SHIFT => self.shift = false,
            ALT_GR => self.altgr = false,
            _ => {}
        }
        self
    }

    /// Whether this key only changes what other keys type.
    ///
    /// A caller drawing what was typed uses this to know that nothing is owed
    /// for the press — rather than looking up a character, finding none, and
    /// having to guess whether the key was a modifier or one the table is
    /// missing. Those two are not the same and a person feels the difference.
    #[must_use]
    pub const fn is_one_of_the_three(keycode: u16) -> bool {
        matches!(keycode, LEFT_SHIFT | RIGHT_SHIFT | CAPS_LOCK | ALT_GR)
    }

    /// How the table names this combination.
    ///
    /// Three bits, Shift the highest, in the order the generator wrote them.
    #[must_use]
    pub(crate) const fn as_the_table_numbers_it(self) -> u8 {
        ((self.shift as u8) << 2) | ((self.lock as u8) << 1) | (self.altgr as u8)
    }

    /// The combination the table numbers this way.
    ///
    /// `cfg(test)` on purpose. Nothing in the product reads the table backwards
    /// — a caller has a keyboard in front of it and builds a [`Held`] from
    /// presses — so this exists to check that
    /// [`Self::as_the_table_numbers_it`] loses nothing, which the table does
    /// depend on. Shipping it unused would be a surface nobody needs in the one
    /// crate the install environment links.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn the_one_numbered(which: u8) -> Self {
        Self {
            shift: which & 0b100 != 0,
            lock: which & 0b010 != 0,
            altgr: which & 0b001 != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ALT_GR, CAPS_LOCK, Held, LEFT_SHIFT, RIGHT_SHIFT};

    #[test]
    fn a_keyboard_starts_with_nothing_on() {
        let held = Held::nothing();
        assert!(!held.shift() && !held.lock() && !held.altgr());
        assert_eq!(held, Held::default());
    }

    #[test]
    fn shift_follows_the_key_and_either_one_does_it() {
        for key in [LEFT_SHIFT, RIGHT_SHIFT] {
            let down = Held::nothing().and_this_went_down(key);
            assert!(down.shift(), "{key} did not hold shift down");
            assert!(!down.and_this_came_up(key).shift(), "{key} stayed held");
        }
    }

    #[test]
    fn caps_lock_latches_on_the_press_and_the_release_does_not_undo_it() {
        let on = Held::nothing().and_this_went_down(CAPS_LOCK);
        assert!(on.lock(), "one press did not turn it on");
        assert!(
            on.and_this_came_up(CAPS_LOCK).lock(),
            "the release turned it back off, so nobody could ever switch it on"
        );
        assert!(
            !on.and_this_went_down(CAPS_LOCK).lock(),
            "a second press did not turn it off"
        );
    }

    #[test]
    fn altgr_is_the_right_alt_and_the_left_one_is_not_it() {
        assert!(Held::nothing().and_this_went_down(ALT_GR).altgr());
        // 56 is KEY_LEFTALT. A layout's third level is reached by the right one
        // and not by this.
        assert!(!Held::nothing().and_this_went_down(56).altgr());
    }

    #[test]
    fn a_key_that_is_not_a_modifier_changes_nothing() {
        let held = Held::nothing()
            .and_this_went_down(LEFT_SHIFT)
            .and_this_went_down(16)
            .and_this_came_up(16)
            .and_this_went_down(28);
        assert!(held.shift(), "an ordinary key disturbed shift");
        assert!(!held.lock() && !held.altgr());
    }

    #[test]
    fn the_three_are_known_apart_from_every_other_key() {
        for key in [LEFT_SHIFT, RIGHT_SHIFT, CAPS_LOCK, ALT_GR] {
            assert!(Held::is_one_of_the_three(key), "{key} was not recognised");
        }
        // The letter row, Enter, space and the left Alt are not modifiers.
        for key in [16, 28, 57, 56, 2] {
            assert!(
                !Held::is_one_of_the_three(key),
                "{key} was called a modifier"
            );
        }
    }

    #[test]
    fn every_combination_has_its_own_number_and_comes_back_the_same() {
        let mut seen = [false; 8];
        for shift in [false, true] {
            for lock in [false, true] {
                for altgr in [false, true] {
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
                    let number = held.as_the_table_numbers_it();
                    let at = usize::from(number);
                    assert!(!seen[at], "two combinations are numbered {number}");
                    seen[at] = true;
                    assert_eq!(
                        Held::the_one_numbered(number),
                        held,
                        "number {number} did not come back as what it was made from"
                    );
                }
            }
        }
        assert!(seen.into_iter().all(|one| one), "a number is unused");
    }
}
