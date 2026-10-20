//! A key pressed on a real keyboard, reaching this program.
//!
//! Item 3 of `docs/autonomy/the-environment-in-full-plan.md`, and **only half
//! of it** — see *What this cannot do* below, which is the half that turned out
//! to rest on something that does not exist.
//!
//! # Why the environment needs this at all
//!
//! [ADR 0128](../../../docs/decisions/0128-whether-a-wi-fi-password-may-be-written-to-the-staging-partition.md)
//! gave the environment screens of its own: a network to choose, a disk to
//! choose, and a disk's name to type before anything is erased. Choosing is
//! moving through a list and pressing a key; typing is characters. **This file
//! does the first and not the second**, and the reason is worth more than the
//! code.
//!
//! # No ioctl to find the keyboard, and one small struct to read it
//!
//! `EVIOCGBIT` would say which keys a device has. The kernel also publishes it:
//! `/sys/class/input/event<N>/device/capabilities/key` is the same bitmask as
//! text, most significant word first. So finding a keyboard is parsing, which a
//! test can hold.
//!
//! Reading one is `struct input_event`, and that is a struct — but the
//! smallest and most stable one in this part of the kernel: two 64-bit times, a
//! `u16` type, a `u16` code, an `i32` value, twenty-four bytes altogether, and
//! unchanged for the life of the interface. It is parsed from bytes here rather
//! than transmuted, so a test drives it and no `unsafe` appears.
//!
//! # What this cannot do, and why it is the interesting half
//!
//! **A keycode is not a character.** `KEY_A` is 30 whatever is printed on the
//! key, and turning 30 into `a` — or into `q` on a French keyboard — needs the
//! layout the person uses. **There is no layout in this environment.** Measured
//! on 2026-10-20: `image/installing/alo-installing.conf` sets no keymap, the
//! environment's recipe sets none, the loader's `grub.cfg` passes none, and the
//! image has no `/etc/vconsole.conf` at all.
//!
//! *The plan said this file would use "the keymap the person chose at the
//! loader". There is no such thing, and that sentence was written without
//! checking — the same fault this repository keeps finding, in a plan two hours
//! old.*
//!
//! So the keys below are the ones whose **position** carries their meaning
//! rather than their printing: arrows, `Enter`, `Escape`, `Backspace`, `Tab`.
//! Those are identical on every layout, which is exactly why every installer
//! in the world asks for a keyboard layout **with a list a person moves through
//! using them**. A list is navigable now; a password is not, and will not be
//! until somebody chooses a layout, which is a screen nobody has designed.

/// Where the kernel publishes which keys a device has.
///
/// `<N>` is an event number; the file is a bitmask as text.
pub const WHERE_THE_KEYS_ARE: &str = "/sys/class/input";

/// How many bytes one `input_event` is: two 64-bit times, two `u16`s and an
/// `i32`.
pub const ONE_EVENT: usize = 24;

/// The event type for a key, `EV_KEY`.
const A_KEY: u16 = 1;

/// Keycodes whose meaning is their place on the keyboard rather than what is
/// printed on them.
mod code {
    /// `KEY_ENTER`.
    pub const ENTER: u16 = 28;
    /// `KEY_ESC`.
    pub const ESCAPE: u16 = 1;
    /// `KEY_BACKSPACE`.
    pub const BACKSPACE: u16 = 14;
    /// `KEY_TAB`.
    pub const TAB: u16 = 15;
    /// `KEY_UP`.
    pub const UP: u16 = 103;
    /// `KEY_DOWN`.
    pub const DOWN: u16 = 108;
    /// `KEY_A`, and `KEY_Z` and `KEY_ENTER` beside it: the three a device must
    /// have for this to call it a keyboard.
    pub const A: u16 = 30;
    /// `KEY_Z`.
    pub const Z: u16 = 44;
}

/// A press this program understands without knowing the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressed {
    /// Up the list.
    Up,
    /// Down the list.
    Down,
    /// Take this one.
    Enter,
    /// Go back.
    Escape,
    /// Rub one out.
    Backspace,
    /// On to the next thing.
    Tab,
}

/// What a key event said, where it said anything this understands.
///
/// A release is [`None`] rather than a variant: a list moves on the press, and
/// a program that acted on both would move twice for one keystroke. An
/// autorepeat (`value` 2) **is** a press, because holding an arrow down to
/// travel a list is how a person uses one.
///
/// **Short reads are held by the `?` on each `get` and not by a length check.**
/// There was a length check here and it was removed: a mutation deleting it
/// changed nothing, because every byte is reached through `get`, which answers
/// [`None`] past the end. A guard that cannot fail is worse than no guard — it
/// reads as the thing holding the bounds while something else does, and the
/// next person to shorten this function would trust the wrong line.
#[must_use]
pub fn what_was_pressed(event: &[u8]) -> Option<Pressed> {
    let said = what_an_event_said(event)?;
    if said.value == 0 {
        return None;
    }
    match said.code {
        code::UP => Some(Pressed::Up),
        code::DOWN => Some(Pressed::Down),
        code::ENTER => Some(Pressed::Enter),
        code::ESCAPE => Some(Pressed::Escape),
        code::BACKSPACE => Some(Pressed::Backspace),
        code::TAB => Some(Pressed::Tab),
        _ => None,
    }
}

/// Which key an event was about, and what happened to it.
///
/// `value` is the kernel's: **0 a release, 1 a press, 2 an autorepeat.** It is
/// carried rather than reduced to a boolean because the two readers of this
/// want different things from it — [`what_was_pressed`] discards releases,
/// since a list that moved on both would move twice for one keystroke, and
/// `crate::WhatAPersonTyped` **needs** them, because a shift whose release is
/// never seen stays held for the rest of the install.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Said {
    /// The evdev code, as `input-event-codes.h` numbers it.
    pub code: u16,
    /// 0 a release, 1 a press, 2 an autorepeat.
    pub value: i32,
}

/// The key event in these bytes, where they are one.
///
/// [`None`] for anything that is not a key event — a synchronisation marker, a
/// mouse movement — and for a short read, which every `get` here holds.
#[must_use]
pub fn what_an_event_said(event: &[u8]) -> Option<Said> {
    let kind = u16::from_ne_bytes([*event.get(16)?, *event.get(17)?]);
    if kind != A_KEY {
        return None;
    }
    Some(Said {
        code: u16::from_ne_bytes([*event.get(18)?, *event.get(19)?]),
        value: i32::from_ne_bytes([
            *event.get(20)?,
            *event.get(21)?,
            *event.get(22)?,
            *event.get(23)?,
        ]),
    })
}

/// Whether a device with these capabilities is a keyboard.
///
/// **Three keys, not one.** A power button reports `KEY_POWER` and a lid switch
/// reports a key too, so *has any key at all* would offer a person a list to
/// move through with a lid. A device that has `A`, `Z` and `Enter` is a
/// keyboard; nothing else in a computer has all three.
#[must_use]
pub fn is_a_keyboard(capabilities: &str) -> bool {
    [code::A, code::Z, code::ENTER]
        .iter()
        .all(|key| has_key(capabilities, *key))
}

/// Whether this capabilities bitmask has this key.
///
/// The kernel writes it as space-separated 64-bit hexadecimal words, **most
/// significant first**, so the last word holds keys 0 to 63. Reading it the
/// other way round is the mistake this function exists to make once.
#[must_use]
pub fn has_key(capabilities: &str, key: u16) -> bool {
    let words: Vec<&str> = capabilities.split_whitespace().collect();
    let from_the_end = (key / 64) as usize;
    if from_the_end >= words.len() {
        return false;
    }
    let Some(word) = words.get(words.len() - 1 - from_the_end) else {
        return false;
    };
    let Ok(bits) = u64::from_str_radix(word, 16) else {
        return false;
    };
    bits & (1u64 << (key % 64)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One `input_event`'s twenty-four bytes.
    fn an_event(kind: u16, code: u16, value: i32) -> Vec<u8> {
        let mut bytes = vec![0u8; 16];
        bytes.extend_from_slice(&kind.to_ne_bytes());
        bytes.extend_from_slice(&code.to_ne_bytes());
        bytes.extend_from_slice(&value.to_ne_bytes());
        bytes
    }

    /// A capabilities mask with these keys in it, written the way the kernel
    /// writes one: most significant word first.
    fn a_mask_with(keys: &[u16]) -> String {
        let mut words = [0u64; 12];
        for key in keys {
            let at = (key / 64) as usize;
            if let Some(word) = words.get_mut(at) {
                *word |= 1u64 << (key % 64);
            }
        }
        words
            .iter()
            .rev()
            .map(|word| format!("{word:x}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// **Every key whose meaning is its place is read.**
    #[test]
    fn the_keys_that_mean_their_place_are_read() {
        for (code, expected) in [
            (code::UP, Pressed::Up),
            (code::DOWN, Pressed::Down),
            (code::ENTER, Pressed::Enter),
            (code::ESCAPE, Pressed::Escape),
            (code::BACKSPACE, Pressed::Backspace),
            (code::TAB, Pressed::Tab),
        ] {
            assert_eq!(
                what_was_pressed(&an_event(A_KEY, code, 1)),
                Some(expected),
                "code {code}"
            );
        }
    }

    /// **A release is not a press**, or a list would move twice for one
    /// keystroke.
    #[test]
    fn a_release_is_not_a_press() {
        assert_eq!(what_was_pressed(&an_event(A_KEY, code::DOWN, 0)), None);
    }

    /// **An autorepeat is a press**, because holding an arrow down to travel a
    /// list is how a person uses one.
    #[test]
    fn an_autorepeat_is_a_press() {
        assert_eq!(
            what_was_pressed(&an_event(A_KEY, code::DOWN, 2)),
            Some(Pressed::Down)
        );
    }

    /// **Nothing that is not a key event is a press.**
    ///
    /// A keyboard emits synchronisation events (`EV_SYN`, type 0) between
    /// every group and miscellaneous ones (`EV_MSC`, type 4) beside each key,
    /// so a reader that did not check the type would act on those too.
    #[test]
    fn nothing_that_is_not_a_key_event_is_a_press() {
        for kind in [0u16, 2, 3, 4, 5] {
            assert_eq!(
                what_was_pressed(&an_event(kind, code::DOWN, 1)),
                None,
                "type {kind}"
            );
        }
    }

    /// **A key this does not understand is not a guess.**
    ///
    /// `KEY_A` is the case that matters: it arrives, and without a layout it
    /// cannot become a character, so it becomes nothing rather than something.
    #[test]
    fn a_key_with_no_meaning_without_a_layout_is_nothing() {
        assert_eq!(what_was_pressed(&an_event(A_KEY, code::A, 1)), None);
        assert_eq!(what_was_pressed(&an_event(A_KEY, code::Z, 1)), None);
    }

    /// **A short read is not an event.**
    #[test]
    fn a_short_read_is_not_an_event() {
        for len in 0..ONE_EVENT {
            let mut short = an_event(A_KEY, code::ENTER, 1);
            short.truncate(len);
            assert_eq!(what_was_pressed(&short), None, "{len} bytes");
        }
        assert!(
            what_was_pressed(&an_event(A_KEY, code::ENTER, 1)).is_some(),
            "the control: the whole thing reads"
        );
    }

    /// **A keyboard is a keyboard and a power button is not.**
    ///
    /// The second half is the point: *has any key at all* would offer a person
    /// a list to move through with a lid switch.
    #[test]
    fn a_keyboard_is_told_from_a_power_button() {
        assert!(is_a_keyboard(&a_mask_with(&[
            code::A,
            code::Z,
            code::ENTER
        ])));
        // KEY_POWER is 116, and a lid switch reports keys too.
        assert!(!is_a_keyboard(&a_mask_with(&[116])));
        assert!(!is_a_keyboard(&a_mask_with(&[code::ENTER])));
        assert!(!is_a_keyboard(&a_mask_with(&[code::A, code::Z])));
        assert!(!is_a_keyboard(""));
    }

    /// **The last word holds the lowest keys**, which is the one thing about
    /// this file's format that a reader gets backwards.
    ///
    /// `KEY_ESC` is 1, so it is in the final word. A reader taking the words
    /// in the order they appear would look for it in the first.
    #[test]
    fn the_last_word_holds_the_lowest_keys() {
        let mask = a_mask_with(&[code::ESCAPE]);
        assert!(mask.ends_with(" 2"), "{mask}");
        assert!(has_key(&mask, code::ESCAPE));
        // And a key in a higher word is found in an earlier one: KEY_UP is
        // 103, so it is in the second word from the end.
        let higher = a_mask_with(&[code::UP]);
        assert!(has_key(&higher, code::UP), "{higher}");
        assert!(!has_key(&higher, code::ESCAPE));
    }

    /// **A mask that is not a mask has no keys**, rather than panicking or
    /// answering yes.
    #[test]
    fn a_mask_that_is_not_one_has_no_keys() {
        for not_a_mask in ["", "   ", "zzz", "0x20", "-1"] {
            assert!(!has_key(not_a_mask, code::ENTER), "{not_a_mask:?}");
            assert!(!is_a_keyboard(not_a_mask), "{not_a_mask:?}");
        }
    }
}
