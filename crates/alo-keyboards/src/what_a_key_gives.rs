//! **What a key gives, on the layout a person chose.**
//!
//! This is the link that was missing between a key arriving and a character
//! being typed. `alo-installing` reads `/dev/input/event*` and knows which
//! **evdev** code was pressed; `Keysym` and `Compose` in this crate handle
//! everything after a keysym. Between them sat nothing, and without it the boot
//! environment cannot take a Wi-Fi password, a disk's name or a stick's name —
//! the only three things it ever asks anybody to type.
//!
//! # The engine answers, and we hold the answer
//!
//! `CLAUDE.md` is flat that engines are configured and never written in, and
//! the xkb symbol files make that bite: `be` defines 74 keys and carries 12
//! `include` directives — Belgian is mostly `include "fr(basic)"` plus
//! overrides — so reading them means implementing xkb's includes, variants,
//! levels and groups. **Whenever that code ran it would be the engine.**
//!
//! So `xkbcli how-to-type` is asked instead, once per layout and character, and
//! the committed table is what it said. xkb parses xkb; this crate holds the answer.
//! Neither CI nor an ordinary build needs the tool, which is the same bargain
//! `the-vocabulary.txt` makes.
//!
//! # The evdev offset, which would have been a silent total failure
//!
//! `xkbcli` reports **X11** keycodes and `/dev/input/event*` reports **evdev**
//! codes, and they differ by exactly 8. Measured two ways rather than taken
//! from documentation: `KEY_Q` is evdev 16 and `xkbcli` calls that key 24;
//! `KEY_A` is evdev 30 and `xkbcli` calls it 38. **The table stores evdev
//! codes**, because that is what arrives. Off by eight, every key would give
//! the wrong character and nothing would look broken.
//!
//! # What it cannot do, said plainly
//!
//! A character outside the generated set cannot be typed, and the set is
//! printable ASCII, the Latin-1 letters, and `Œ œ ſ €`. A dead key — the road
//! by which `^` then `e` makes `ê` — is **not** here: `how-to-type` answers
//! single keys, and composing is [`crate::Compose`]'s job, which already
//! exists. So this turns a key into a character and does not turn two into one.
//!
//! **And one known divergence, measured rather than suspected.** A key whose
//! level is *dead* has no row, so on a key where some levels are dead and one
//! is not, dropping an unused modifier can land on the live level: Belgian's
//! `AD11` is `[dead_circumflex, dead_diaeresis, bracketleft, dead_abovering]`
//! and AltGr-with-shift answers `[` where it should answer nothing. It cannot
//! be fixed from this table, because *this level is dead* and *this key ignores
//! this modifier* are the same absence in it. The fix is to generate each key's
//! consulted modifiers from `compile-keymap`'s own `type` declarations, which
//! would have made the caps-lock holes impossible rather than repaired —
//! `a_dead_level_resolves_to_its_neighbour_which_is_known_and_wrong` pins the
//! wrong answer so the fix is noticed.

use std::char;

/// What `xkbcli how-to-type` said, for every layout the environment offers.
///
/// Five tab-separated columns: the layout, the **evdev** code, which modifiers
/// were held, the code point, and the character itself. The last column is for
/// a person reading the file; [`what_a_key_gives`] reads the fourth, because a
/// code point cannot be mangled by an editor the way a bare character can.
const THE_TABLE: &str = include_str!("../the-keyboards.txt");

/// The layouts the environment's keyboard screen offers.
///
/// Eight, chosen by the desktop lane to cover the EU languages
/// `docs/features.md` promises with the fewest tables, plus the two that differ
/// most from the rest. A ninth card — *Something else* — is a decision nobody
/// has taken, and until somebody does, a layout absent from this list is
/// absent from the table too.
pub const EVERY_LAYOUT: [&str; 8] = ["be", "fr", "de", "es", "it", "nl", "gb", "us"];

/// What `xkbcli` adds to an evdev code, measured rather than taken on trust.
///
/// `KEY_Q` is evdev 16 and the tool calls that key 24; `KEY_A` is evdev 30 and
/// the tool calls it 38. Both differences are 8.
///
/// Only the regeneration test needs this: the committed table already holds
/// evdev codes, so nothing on the reading path converts anything.
#[cfg(test)]
pub(crate) const THE_X11_OFFSET: u16 = 8;

/// Every character the table was built for.
///
/// Printable ASCII, the Latin-1 letters without the multiplication and division
/// signs, and four a European password or machine name may hold. **A character
/// outside this set cannot be typed**, which is a real limit and is why it is a
/// named list rather than a range somebody has to infer.
///
/// Built from ranges rather than written out, because a hand-written list of
/// 192 code points is a list with a typo in it.
#[cfg(test)]
pub(crate) fn every_code_point_asked_for() -> Vec<u32> {
    (0x20..0x7F)
        .chain((0xA1..0x100).filter(|point| *point != 0xD7 && *point != 0xF7))
        .chain([0x0152, 0x0153, 0x017F, 0x20AC])
        .collect()
}

/// Which modifiers were held when the key came down.
///
/// `caps` is separate from `shift` rather than folded into it, because the two
/// are not the same question: the table holds rows for `Lock` alone, for
/// `Shift` alone and for both, and on several of these layouts they differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Held {
    /// Either shift key.
    pub shift: bool,
    /// The right-hand alt key, which xkb calls `Mod5` and a person calls AltGr.
    pub altgr: bool,
    /// Caps lock, which xkb calls `Lock`.
    pub caps: bool,
}

impl Held {
    /// Nothing held.
    #[must_use]
    pub const fn nothing() -> Self {
        Self {
            shift: false,
            altgr: false,
            caps: false,
        }
    }

    /// The same, with shift down.
    #[must_use]
    pub const fn and_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// The same, with AltGr down.
    #[must_use]
    pub const fn and_altgr(mut self) -> Self {
        self.altgr = true;
        self
    }

    /// The same, with caps lock on.
    #[must_use]
    pub const fn and_caps(mut self) -> Self {
        self.caps = true;
        self
    }

    /// The same, with every modifier this key pays no attention to dropped.
    ///
    /// See `what_this_key_consults`: a modifier a key's type does not list is
    /// ignored by that key, so holding it must not stop the key answering.
    const fn but_only(self, consulted: Self) -> Self {
        Self {
            shift: self.shift && consulted.shift,
            altgr: self.altgr && consulted.altgr,
            caps: self.caps && consulted.caps,
        }
    }

    /// How this reads in the committed table's third column.
    ///
    /// The names are xkb's and the order is sorted, which is how the table was
    /// written — so a row is found by one string comparison rather than by
    /// parsing a set on every keypress.
    fn as_the_table_writes_it(self) -> &'static str {
        match (self.caps, self.altgr, self.shift) {
            (false, false, false) => "",
            (false, false, true) => "Shift",
            (false, true, false) => "Mod5",
            (false, true, true) => "Mod5 Shift",
            (true, false, false) => "Lock",
            (true, false, true) => "Lock Shift",
            (true, true, false) => "Lock Mod5",
            (true, true, true) => "Lock Mod5 Shift",
        }
    }
}

/// Whether this is a layout the environment offers.
#[must_use]
pub fn a_layout_is_offered(layout: &str) -> bool {
    EVERY_LAYOUT.contains(&layout)
}

/// What this key gives on this layout, with these modifiers held.
///
/// [`None`] where the layout is not one of [`EVERY_LAYOUT`], where that key
/// gives nothing at that level, or where the key is not a character key at all —
/// Escape, the function row, the arrows. A caller showing what somebody typed
/// adds nothing for [`None`] rather than guessing.
#[must_use]
pub fn what_a_key_gives(layout: &str, evdev: u16, held: Held) -> Option<char> {
    if !a_layout_is_offered(layout) {
        return None;
    }
    let wanted = held.but_only(what_this_key_consults(layout, evdev));
    let wanted = wanted.as_the_table_writes_it();
    for row in THE_TABLE.lines() {
        let mut column = row.split('\t');
        if column.next()? != layout {
            continue;
        }
        let Some(Ok(code)) = column.next().map(str::parse::<u16>) else {
            continue;
        };
        if code != evdev || column.next()? != wanted {
            continue;
        }
        return column.next().and_then(a_code_point);
    }
    None
}

/// Which modifiers this key pays any attention to at all.
///
/// **A modifier a key's type does not list is ignored by that key**, which is
/// xkb's own rule and the reason this function exists. The keymap says it
/// outright:
///
/// ```text
/// type "ONE_LEVEL"  { modifiers= none; }
/// type "TWO_LEVEL"  { modifiers= Shift; map[Shift]= 2; }
/// type "ALPHABETIC" { modifiers= Shift+Lock; map[Shift]= 2; map[Lock]= 2; }
/// ```
///
/// `Lock` is absent from `TWO_LEVEL`, so caps lock does nothing to a digit key —
/// it neither shifts it nor stops it. `ONE_LEVEL` lists nothing, so the space
/// bar gives a space with anything held.
///
/// **The table cannot say this directly and the hole it left was person-facing.**
/// `xkbcli how-to-type` answers *which modifiers reach this character*, so a
/// modifier that reaches nothing is never mentioned — Belgian's ampersand key
/// had rows for nothing, `Shift`, `Mod5` and `Mod5 Shift` and **no `Lock` row at
/// all**, and the space bar had one row. Matched strictly, a person with caps
/// lock on typed nothing from that key, and a shifted space typed nothing: a
/// Wi-Fi password that stops dead with no report but *the password was refused*.
///
/// So the modifiers a key consults are read back out of its own rows — the union
/// of every modifier any row for it mentions — and anything else is dropped
/// before matching. *Found by the desktop lane on 2026-10-21, measured against
/// this table rather than inferred, and the diagnosis is theirs.*
fn what_this_key_consults(layout: &str, evdev: u16) -> Held {
    let mut consulted = Held::nothing();
    for row in THE_TABLE.lines() {
        let mut column = row.split('\t');
        if column.next() != Some(layout) {
            continue;
        }
        let Some(Ok(code)) = column.next().map(str::parse::<u16>) else {
            continue;
        };
        if code != evdev {
            continue;
        }
        let Some(mods) = column.next() else {
            continue;
        };
        for one in mods.split_whitespace() {
            match one {
                "Shift" => consulted.shift = true,
                "Mod5" => consulted.altgr = true,
                "Lock" => consulted.caps = true,
                _ => {}
            }
        }
    }
    consulted
}

/// The character a `U+XXXX` column names.
fn a_code_point(written: &str) -> Option<char> {
    let digits = written.strip_prefix("U+")?;
    char::from_u32(u32::from_str_radix(digits, 16).ok()?)
}

#[cfg(test)]
#[path = "what_a_key_gives_tests.rs"]
mod tests;
