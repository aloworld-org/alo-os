//! The fixtures this crate's own tests are written against.
//!
//! Every file here that says something has the same question to answer — *what
//! does this say on a machine with no translations* — and answering it from one
//! fixture is what stops eight files inventing eight vocabularies that resemble
//! the real one. The real one is [`crate::keyboard_words`].
//!
//! The rented data here is **a few lines shaped like the real files**, so that a
//! unit test is about this crate's arithmetic rather than about whichever
//! version of `xkeyboard-config` a developer happens to have. The real files on
//! the real machine are read by the tests in `tests/`, which is where the
//! promise *every language has a keyboard that exists* is actually held.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_strings::{Language, Strings};

use crate::compose::Compose;
use crate::keysym::Keysym;
use crate::rented::Rented;
use crate::words::keyboard_words;

/// This crate's own words, with nothing translated.
///
/// `alo-shortcuts`' words are in it too, and have to be: the one sentence this
/// crate says about a chord fills two of its gaps from that crate — the keys
/// held and what the shortcut does — and a vocabulary holding only this
/// crate's would answer both with a key nobody declared. On the machine there
/// is one vocabulary (`alo-saying`); this is the smallest fixture that is
/// honest about that.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = keyboard_words().unwrap();
    alo_shortcuts::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A language, written.
pub(crate) fn a_language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// These keys, named.
pub(crate) fn keys(named: &[&str]) -> Vec<Keysym> {
    named
        .iter()
        .map(|name| Keysym::named(name).unwrap())
        .collect()
}

/// A handful of lines shaped like the machine's list of keyboards.
pub(crate) fn rented() -> Rented {
    Rented::read_text(
        "\
! layout
  de              German
  fr              French
  gr              Greek
  us              English (US)
! variant
  intl            us: English (US, intl., with dead keys)
! option
  grp:alt_space_toggle Alt+Space
  compose:menu         Menu
",
        Path::new("/usr/share/X11/xkb/rules/evdev.lst"),
    )
    .unwrap()
}

/// A handful of lines shaped like the rented compose table.
pub(crate) fn a_small_table() -> Compose {
    Compose::read_text(
        "\
<dead_diaeresis> <u>\t: \"ü\"\tudiaeresis
<Multi_key> <s> <s>\t: \"ß\"\tssharp
",
        Path::new("/usr/share/X11/locale/en_US.UTF-8/Compose"),
    )
    .unwrap()
}
