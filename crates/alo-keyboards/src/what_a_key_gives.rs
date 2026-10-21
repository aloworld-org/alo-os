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
//! # A dead level is recorded, not absent
//!
//! **This was a divergence until 2026-10-21, and the fix is the interesting
//! part.** A level whose keysym produces no character — `dead_circumflex` and
//! sixteen others — used to have no row at all, so *this level is dead* and
//! *this key ignores this modifier* were the same absence. On Belgian's `AD11`,
//! `[dead_circumflex, dead_diaeresis, bracketleft, dead_abovering]`, dropping
//! the unused shift landed AltGr-with-shift on level 3 and typed `[` where it
//! should have typed nothing.
//!
//! **The first plan for fixing it was wrong and is worth recording as such:**
//! read each key's type from `compile-keymap`'s `type` declarations. A compiled
//! keymap states **no key's type** — measured, 0 of 419 key lines on Belgian
//! mention one, because xkbcommon resolves types internally and its text output
//! omits them. Inferring a type from the keysyms is what xkbcommon itself does,
//! and doing it here would be becoming the engine.
//!
//! What works instead: **`how-to-type` accepts a keysym by name**, so the engine
//! is asked about the dead levels directly and the table records them as rows
//! that give nothing. The dead names are read out of each compiled keymap rather
//! than listed by hand — 17 across the eight layouts, 13 to 16 on each, and
//! **none at all on `us`**, which is exactly why this was invisible to anybody
//! testing on a US board.

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

/// Every character the table is built for.
///
/// **Eight ranges, chosen by a coverage check rather than by judgement.** The
/// first version of this was printable ASCII, the Latin-1 letters and four more,
/// picked by hand — and it deliberately left out `×` and `÷` on the grounds that
/// nobody types them. A check that asked each compiled keymap which keysyms its
/// layouts actually put on keys then named what was unreachable, and `×` and `÷`
/// were on real keys on four layouts: `Mod5 Shift` on key 51 or 52. So a person
/// pressing that key got nothing, because of a judgement somebody made about
/// what they would want.
///
/// | range | what the check named in it |
/// |---|---|
/// | `0x0020..0x007F` | printable ASCII |
/// | `0x00A0..0x0100` | Latin-1 — **including `×` and `÷`**, which were excluded by hand |
/// | `0x0100..0x0250` | Latin Extended-A and B — `ſ` on German's S key |
/// | `0x0370..0x0400` | Greek — the omega Belgian puts on AltGr with shift |
/// | `0x1E00..0x1F00` | Latin Extended Additional — `ẞ`, the capital sharp s, on six layouts |
/// | `0x2000..0x2070` | general punctuation — `′ ″ ‹ › •` |
/// | `0x20A0..0x20C0` | currency — the Euro |
/// | `0x2100..0x2200` | letterlike, number forms and arrows |
///
/// *The ranges are the desktop lane's, and they are adopted here because this
/// lane's own coverage check named the same gaps rather than because they said
/// so.* A character outside them still cannot be typed, and the check is what
/// says whether that matters.
#[cfg(test)]
pub(crate) fn every_code_point_asked_for() -> Vec<u32> {
    (0x0020..0x007F)
        .chain(0x00A0..0x0100)
        .chain(0x0100..0x0250)
        .chain(0x0370..0x0400)
        .chain(0x1E00..0x1F00)
        .chain(0x2000..0x2070)
        .chain(0x20A0..0x20C0)
        .chain(0x2100..0x2200)
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
        // A `dead` fourth column is a level that produces no character —
        // `dead_circumflex` and sixteen others. It answers `None`, and it does
        // so **explicitly**: `a_code_point` would also fail on it, and relying
        // on that would leave the next reader to work out that a row exists and
        // is deliberately empty.
        return match column.next()? {
            "dead" => None,
            point => a_code_point(point),
        };
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
