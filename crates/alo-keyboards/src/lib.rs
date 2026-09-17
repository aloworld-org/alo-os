//! Which keyboard a person types on, and how they type a letter that is not on
//! it.
//!
//! `ROADMAP.md` v0.5: *keyboard layouts with dead keys and a compose key, input
//! methods*. **"Müller" and "Liège" are test cases in a European product, not
//! edge cases** — and neither is *Ġgantija*, which is why every one of the 24
//! languages `alo-strings` carries has a keyboard here and a test that names it.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`layout`] | A keyboard as the rented data names it |
//! | [`offering`] | Which keyboard is offered with which language, for all 24 |
//! | [`rented`] | The machine's own list of keyboards, read and never written |
//! | [`keysym`] | One key, named the way the rented tables name it |
//! | [`compose`] | The rented compose table: which keys write which letter |
//! | [`typing`] | A dead key waiting, and the letter it is put on |
//! | [`compose_key`] | Which key composes, out of the rented options |
//! | [`switching`] | The one shortcut that switches keyboards |
//! | [`methods`] | Input methods, for the scripts a keyboard cannot type |
//! | [`these`] | A person's keyboards, as a list that is never empty |
//! | [`keyboards`] | The whole of it: what a person has and what they are on |
//! | [`changes`] | Only what the person changed |
//! | [`keeping`] | `keyboards.toml`, in their own folder (ADR 0038) |
//! | [`unkept`] | What they are told when that file did not read |
//! | [`refusing`] | Why nothing changed |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! ```
//! use alo_keyboards::{Keyboards, Layout, Rented};
//! use alo_strings::Language;
//! use std::path::Path;
//!
//! // A person who chose Greek is handed the Greek keyboard, not a list.
//! let greek = Language::written("el")?;
//! let mut keyboards = Keyboards::offered_with(&greek);
//! assert_eq!(keyboards.in_use(), &Layout::named("gr")?);
//!
//! // And adds a second one by naming the language, never a layout's code.
//! let rented = Rented::read_text(
//!     "! layout\n  gr Greek\n  de German\n",
//!     Path::new("/usr/share/X11/xkb/rules/evdev.lst"),
//! ).map_err(|refused| format!("{refused:?}"))?;
//! let added = keyboards.add_for(&Language::written("de")?, &rented)
//!     .map_err(|refused| format!("{refused:?}"))?;
//! assert_eq!(added, Layout::named("de")?);
//! assert_eq!(keyboards.in_the_status_area().as_deref(), Some("GR"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Nothing here is keyboard data
//!
//! No layout, no compose sequence, no input-method engine is written in this
//! crate. `xkeyboard-config`, `libX11`'s compose tables and a rented
//! input-method framework are configured and never patched (ADR 0011). **What
//! is ours is which keyboard is offered with which language, and that the offer
//! is complete** — [`offering`], one table, held to the rented data by
//! `tests/a_keyboard_for_every_language.rs`.
//!
//! # Nothing here draws, and nothing here presses a key
//!
//! The compositor reads what this crate decides. A [`typing::Composing`] is
//! handed key names and answers what they wrote; it opens no device, watches no
//! input and holds nothing between one person's windows.
//!
//! # Nothing here is the agent's
//!
//! A keyboard is changed by a person's hands, and what somebody types is not
//! context the agent is offered (ADR 0001 §4). This crate depends on neither
//! `alo-capability` nor `alo-context`, which is what makes that structural
//! rather than remembered.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence is declared in [`words`] and answered through `alo-strings`;
//! [`Refused`], [`ComposeKey`], [`NotRented`], [`NotComposed`], [`FileNotRead`]
//! and [`FileNotWritten`] have `said`, not `Display`. The exceptions are
//! [`LayoutError`] and [`KeysymError`], which are said to whoever is reading a
//! log about a name that came from a rented file.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod changes;
pub mod compose;
pub mod compose_key;
pub mod keeping;
pub mod keyboards;
pub mod keysym;
pub mod layout;
pub mod methods;
pub mod offering;
pub mod refusing;
pub mod rented;
pub mod switching;
pub mod these;
pub mod typing;
pub mod unkept;
pub mod words;

#[cfg(test)]
mod testing;

pub use changes::Changes;
pub use compose::{Compose, NotATable, NotComposed, THE_TABLE};
pub use compose_key::ComposeKey;
pub use keyboards::Keyboards;
pub use keysym::{Keysym, KeysymError};
pub use layout::{Layout, LayoutError};
pub use methods::{IT_IS_STARTED_BY, THE_FRAMEWORK, Writing};
pub use offering::{EVERY_OFFER, Offer, offered_with};
pub use refusing::Refused;
pub use rented::{NotAList, NotRented, Rented, THE_RULES};
pub use these::These;
pub use typing::{Composing, Typed};
pub use unkept::{FileNotRead, FileNotWritten};
pub use words::{Word, WordsError, declare_into, keyboard_words};
