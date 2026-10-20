//! A key a person pressed, and the letter it put on the screen.
//!
//! This is the one link that stood between the install environment and
//! everything it asks a person to type. `alo-installing` could already find a
//! keyboard and read a press off it; `alo-keyboards` could already take a
//! keysym and compose a letter from it. **In between was evdev keycode +
//! modifiers + layout → keysym, and nothing in this repository did it** except
//! the compositor, through `xkbcommon`, which reaches it only by way of
//! `smithay` and `winit`.
//!
//! So this crate is that link, and it is a table rather than an engine.
//!
//! # Why it matters more than its size suggests
//!
//! Everything the install environment ever asks a person to type runs through
//! here: the Wi-Fi password, the disk's own name typed back before anything is
//! erased, and the name of a stick before it is written. There is nothing else
//! it asks anybody to type. Until this existed, **a layout screen would have
//! been a screen that asks a question and cannot act on the answer** — a person
//! could choose Belgian and it would change nothing they could observe, because
//! nothing could be typed either way.
//!
//! And the failure it removes is the silent one. A person typing their own
//! Wi-Fi password on an AZERTY keyboard against a US table gets it wrong with
//! nothing on screen to show it, and the environment's only possible report is
//! *the password was refused* — the truth about the wrong thing.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`the_eight_layouts`] | The eight layouts the environment offers, and the names `xkbcommon` knows them by |
//! | [`what_is_held_down`] | Shift, CapsLock and AltGr, which are the only three that change what a key types |
//! | [`which_level_a_key_is_at`] | Which of a key's four characters those three select, and why CapsLock is not Shift |
//! | [`mod@what_a_key_types`] | The lookup itself |
//! | [`the_table_xkbcommon_gave`] | The table. Generated, checked in, and not edited by hand |
//!
//! # What is not here, and where it is
//!
//! **Dead keys and compose sequences.** A dead key types nothing by itself and
//! then puts an accent on the next letter, and `alo-keyboards` already carries
//! that whole chain in pure Rust — `Keysym`, `Compose`, `Typed`. This crate
//! stops at the character a single key produces, which is what the eight
//! layouts put a person's own accented letters on directly.
//!
//! **Chords.** `Ctrl+A` types nothing; it means something, and meaning belongs
//! to whoever reads chords. The three modifiers here are only the three that
//! select between the characters printed on one key.
//!
//! # Examples
//!
//! The same physical key, on two keyboards:
//!
//! ```
//! use alo_keytables::{Held, TheLayout, what_a_key_types};
//!
//! // The key a US keyboard marks Q. `KEY_Q` is 16 whatever is printed on it.
//! let pressed = 16;
//!
//! assert_eq!(what_a_key_types(TheLayout::Us, pressed, Held::nothing()), Some('q'));
//! assert_eq!(what_a_key_types(TheLayout::Be, pressed, Held::nothing()), Some('a'));
//! ```
//!
//! And a password being typed, modifiers and all:
//!
//! ```
//! use alo_keytables::{Held, TheLayout, what_a_key_types};
//!
//! const LEFT_SHIFT: u16 = 42;
//! const THE_Q_POSITION: u16 = 16;
//!
//! let mut held = Held::nothing();
//! let mut typed = String::new();
//!
//! for (keycode, down) in [(LEFT_SHIFT, true), (THE_Q_POSITION, true), (LEFT_SHIFT, false), (THE_Q_POSITION, true)] {
//!     held = if down { held.and_this_went_down(keycode) } else { held.and_this_came_up(keycode) };
//!     if !Held::is_one_of_the_three(keycode) && down {
//!         if let Some(letter) = what_a_key_types(TheLayout::Be, keycode, held) {
//!             typed.push(letter);
//!         }
//!     }
//! }
//!
//! assert_eq!(typed, "Aa");
//! ```

pub mod the_eight_layouts;
pub mod the_table_xkbcommon_gave;
pub mod what_a_key_types;
pub mod what_is_held_down;
pub mod which_level_a_key_is_at;

pub use the_eight_layouts::TheLayout;
pub use what_a_key_types::{every_key_that_types_something, what_a_key_carries, what_a_key_types};
pub use what_is_held_down::Held;
