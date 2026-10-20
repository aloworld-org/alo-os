//! **The table says what the engine said, and these are the facts that prove
//! it rather than restate it.**
//!
//! Every assertion here is a property of a real keyboard that somebody could
//! check by pressing a key, not an arithmetic identity of the file's own shape.
//! That matters because the file is generated: a test that compared the table to
//! itself would pass however wrong the generation was.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::THE_TABLE;
use super::{
    EVERY_LAYOUT, Held, THE_X11_OFFSET, a_code_point, a_layout_is_offered,
    every_code_point_asked_for, what_a_key_gives,
};

/// Every row of the table, as the tests and the regeneration check read it.
///
/// Test-only: nothing on the reading path walks the whole table.
///
/// Returns the layout, the evdev code, the modifier column exactly as written,
/// and the character. A malformed row is **not** skipped here, unlike in
/// `what_a_key_gives`: a caller counting rows needs to know the file is whole,
/// and a silently dropped row is how a table loses a key nobody notices.
fn every_row() -> impl Iterator<Item = (&'static str, u16, &'static str, char)> {
    THE_TABLE.lines().filter(|row| !row.is_empty()).map(|row| {
        let mut column = row.split('\t');
        let (Some(layout), Some(code), Some(mods), Some(point)) =
            (column.next(), column.next(), column.next(), column.next())
        else {
            panic!("a row of the keyboard table has fewer than four columns: {row:?}");
        };
        let code: u16 = code
            .parse()
            .unwrap_or_else(|_| panic!("a row's evdev column is not a number: {row:?}"));
        let character =
            a_code_point(point).unwrap_or_else(|| panic!("a row's code point is not one: {row:?}"));
        (layout, code, mods, character)
    })
}

/// `KEY_Q`, the physical key second from the left on the top letter row.
const PHYSICAL_Q: u16 = 16;
/// `KEY_W`, the one beside it.
const PHYSICAL_W: u16 = 17;
/// `KEY_A`, the leftmost of the home row.
const PHYSICAL_A: u16 = 30;
/// `KEY_1`, the leftmost of the number row.
const PHYSICAL_1: u16 = 2;
/// `KEY_ESC`, which gives no character on any layout.
const ESCAPE: u16 = 1;

/// **AZERTY swaps A and Q, and the table knows it.**
///
/// The single most consequential difference between the layouts a European
/// person is likely to have, and the one that makes a typed password fail
/// silently: on a French or Belgian board the key where QWERTY has `q` gives
/// `a`, and the key where QWERTY has `a` gives `q`.
#[test]
fn azerty_swaps_the_a_and_the_q() {
    assert_eq!(
        what_a_key_gives("us", PHYSICAL_Q, Held::nothing()),
        Some('q')
    );
    assert_eq!(
        what_a_key_gives("gb", PHYSICAL_Q, Held::nothing()),
        Some('q')
    );
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_Q, Held::nothing()),
        Some('a')
    );
    assert_eq!(
        what_a_key_gives("be", PHYSICAL_Q, Held::nothing()),
        Some('a')
    );

    assert_eq!(
        what_a_key_gives("us", PHYSICAL_A, Held::nothing()),
        Some('a')
    );
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_A, Held::nothing()),
        Some('q')
    );
    assert_eq!(
        what_a_key_gives("be", PHYSICAL_A, Held::nothing()),
        Some('q')
    );
}

/// **The table holds evdev codes, not the X11 codes the tool reports.**
///
/// `xkbcli` answers in X11 keycodes and `/dev/input/event*` reports evdev
/// codes, and they differ by exactly 8. This is the assertion that catches the
/// offset being applied twice, or not at all: `KEY_A` is evdev 30, so 30 gives
/// a letter and **38 must give nothing**, because 38 is `KEY_B`'s neighbour
/// region and not where a table built from X11 codes would have put `a`.
///
/// Off by eight, every key on every layout would give the wrong character and
/// nothing about it would look broken.
#[test]
fn the_codes_are_evdevs_and_not_the_tools() {
    assert_eq!(
        what_a_key_gives("us", PHYSICAL_A, Held::nothing()),
        Some('a')
    );
    assert_ne!(
        what_a_key_gives("us", PHYSICAL_A + 8, Held::nothing()),
        Some('a')
    );
    assert_eq!(
        what_a_key_gives("us", PHYSICAL_Q, Held::nothing()),
        Some('q')
    );
    assert_ne!(
        what_a_key_gives("us", PHYSICAL_Q + 8, Held::nothing()),
        Some('q')
    );
}

/// **Digits need shift on AZERTY**, which is the second way a typed password
/// fails silently.
///
/// On a French board the number row gives `& é " ' ( -` unshifted and the
/// digits only with shift. Somebody typing a password with a `1` in it, on a
/// machine that assumed US, produces `&` and is told the password was refused.
#[test]
fn the_number_row_is_shifted_on_azerty_and_not_on_qwerty() {
    assert_eq!(
        what_a_key_gives("us", PHYSICAL_1, Held::nothing()),
        Some('1')
    );
    assert_eq!(
        what_a_key_gives("us", PHYSICAL_1, Held::nothing().and_shift()),
        Some('!')
    );

    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_1, Held::nothing()),
        Some('&')
    );
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_1, Held::nothing().and_shift()),
        Some('1')
    );
}

/// **AltGr reaches the third level, and Belgian and French differ there** —
/// which is the difference nobody would hand-write correctly.
///
/// Both boards are AZERTY and both give `z` at `KEY_W`. Hold AltGr and Belgian
/// gives `ſ` while French gives `«`; hold AltGr and shift and Belgian gives `§`
/// while French gives `<`. Two layouts a person might not be able to tell apart
/// from a list, distinguished by what the key actually produces.
#[test]
fn belgian_and_french_differ_at_the_third_level_of_one_key() {
    assert_eq!(
        what_a_key_gives("be", PHYSICAL_W, Held::nothing()),
        Some('z')
    );
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_W, Held::nothing()),
        Some('z')
    );

    assert_eq!(
        what_a_key_gives("be", PHYSICAL_W, Held::nothing().and_altgr()),
        Some('\u{17f}')
    );
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_W, Held::nothing().and_altgr()),
        Some('«')
    );
    assert_eq!(
        what_a_key_gives("be", PHYSICAL_W, Held::nothing().and_altgr().and_shift()),
        Some('§')
    );
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_W, Held::nothing().and_altgr().and_shift()),
        Some('<')
    );
}

/// **Caps lock is asked as its own question and the table answers it
/// separately.**
///
/// Folding caps into shift would be wrong on the row this asserts: with caps
/// lock on and nothing else, `KEY_Q` on French gives the capital, and with caps
/// lock **and** shift it gives the small letter back. A keyboard that treated
/// the two as one modifier would get the second of those wrong.
#[test]
fn caps_lock_is_not_shift() {
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_Q, Held::nothing().and_caps()),
        Some('A')
    );
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_Q, Held::nothing().and_caps().and_shift()),
        Some('a')
    );
}

/// **A key that gives no character gives [`None`]**, rather than a guess.
#[test]
fn a_key_that_is_not_a_character_gives_nothing() {
    assert_eq!(what_a_key_gives("us", ESCAPE, Held::nothing()), None);
    assert_eq!(what_a_key_gives("fr", ESCAPE, Held::nothing()), None);
}

/// **A layout nobody offers gives [`None`]**, and is not quietly served as US.
///
/// The silent-US fallback is the exact failure this whole road exists to
/// remove, so it is asserted rather than assumed: an unknown name answers
/// nothing at all, including for a key every layout has.
#[test]
fn a_layout_that_is_not_offered_is_not_quietly_served_as_us() {
    assert!(!a_layout_is_offered("xx"));
    assert_eq!(what_a_key_gives("xx", PHYSICAL_Q, Held::nothing()), None);
    assert_eq!(what_a_key_gives("", PHYSICAL_Q, Held::nothing()), None);
    // And the control: the same key on a layout that IS offered answers.
    assert!(what_a_key_gives("us", PHYSICAL_Q, Held::nothing()).is_some());
}

/// **Every offered layout is actually in the table**, with enough rows to be a
/// layout rather than a stub.
///
/// The control that matters: a generation that quietly produced nothing for one
/// layout would leave that screen's card selectable and every key dead.
#[test]
fn every_offered_layout_is_in_the_table() {
    for layout in EVERY_LAYOUT {
        let rows = every_row().filter(|(it, ..)| *it == layout).count();
        assert!(
            rows > 100,
            "{layout} has {rows} rows in the keyboard table, which is not a layout"
        );
        assert!(
            what_a_key_gives(layout, PHYSICAL_Q, Held::nothing()).is_some(),
            "{layout} gives nothing for the key every one of these boards has"
        );
    }
}

/// **The file is whole**: every row has its four columns and parses.
///
/// `every_row` panics rather than skipping, so this is the test that would fail
/// if the table were truncated, re-saved by something that ate the tabs, or
/// regenerated into a shape this crate cannot read.
#[test]
fn the_table_is_whole() {
    let rows = every_row().count();
    assert!(
        rows > 1_000,
        "the keyboard table has {rows} rows, which is fewer than one layout's worth"
    );
    for (layout, _, mods, _) in every_row() {
        assert!(
            a_layout_is_offered(layout),
            "the table holds {layout}, which is not an offered layout"
        );
        assert!(
            matches!(
                mods,
                "" | "Shift"
                    | "Lock"
                    | "Lock Shift"
                    | "Mod5"
                    | "Mod5 Shift"
                    | "Lock Mod5"
                    | "Lock Mod5 Shift"
            ),
            "the table holds a modifier column this crate cannot read: {mods:?}"
        );
    }
}

/// **The table is what `xkbcli` says today**, regenerated and compared.
///
/// Ignored by default because it needs `xkbcli`, which is in
/// `libxkbcommon-tools` and is on no build machine by default. **Ignored
/// rather than silently skipped**: `cargo test` prints it as `ignored`, so a
/// reader can see that it did not run, which a test that quietly passed when
/// the tool was absent would not.
///
/// Run it where the tool is:
///
/// ```text
/// cargo test -p alo-keyboards --lib -- --ignored the_table_is_what_the_engine_says
/// UPDATE_KEYBOARD_TABLE=1 cargo test -p alo-keyboards --lib -- --ignored the_table_is_what_the_engine_says
/// ```
///
/// The second writes the file. This is `the-vocabulary.txt`'s bargain: the
/// artefact is generated and committed, so an ordinary build needs no engine,
/// and the thing that produced it is in the repository rather than in somebody's
/// shell history.
#[test]
#[ignore = "needs xkbcli from libxkbcommon-tools; run with --ignored"]
fn the_table_is_what_the_engine_says() {
    use std::fmt::Write as _;

    let found = std::process::Command::new("xkbcli")
        .args(["how-to-type", "--layout", "us", "0x0061"])
        .output();
    let Ok(probe) = found else {
        panic!(
            "xkbcli is not on this machine, and this test does not pass without it. \
             Install libxkbcommon-tools, or leave this test ignored."
        )
    };
    assert!(
        probe.status.success() && String::from_utf8_lossy(&probe.stdout).contains("AC01"),
        "xkbcli is here and did not answer the question this test asks of it"
    );

    let mut rebuilt = String::new();
    let mut seen = std::collections::BTreeSet::new();
    for layout in EVERY_LAYOUT {
        for point in every_code_point_asked_for() {
            let said = std::process::Command::new("xkbcli")
                .args(["how-to-type", "--layout", layout, &format!("0x{point:04X}")])
                .output()
                .expect("xkbcli answered a moment ago");
            for line in String::from_utf8_lossy(&said.stdout).lines() {
                let Some((x11, rest)) = line.split_once(char::is_whitespace) else {
                    continue;
                };
                let Ok(x11) = x11.trim().parse::<u16>() else {
                    continue;
                };
                let Some((before, mods)) = rest.rsplit_once('[') else {
                    continue;
                };
                let Some(mods) = mods.split(']').next() else {
                    continue;
                };
                // only the first layout group, which is the one a person chose
                if before.split_whitespace().nth(1) != Some("1") {
                    continue;
                }
                let mut held: Vec<&str> = mods.split_whitespace().collect();
                held.sort_unstable();
                let held = held.join(" ");
                let evdev = x11 - THE_X11_OFFSET;
                if seen.insert((layout, evdev, held.clone())) {
                    let character = char::from_u32(point).expect("a code point this test named");
                    writeln!(
                        rebuilt,
                        "{layout}\t{evdev}\t{held}\tU+{point:04X}\t{character}"
                    )
                    .expect("writing to a String");
                }
            }
        }
    }

    let beside = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("the-keyboards.txt");
    if std::env::var_os("UPDATE_KEYBOARD_TABLE").is_some() {
        std::fs::write(&beside, &rebuilt).expect("writing the keyboard table");
        panic!("the keyboard table was rewritten — run again without UPDATE_KEYBOARD_TABLE");
    }

    let committed = std::fs::read_to_string(&beside).expect("this crate's own keyboard table");
    let committed: Vec<&str> = committed.lines().collect();
    let rebuilt: Vec<&str> = rebuilt.lines().collect();
    assert_eq!(
        rebuilt.len(),
        committed.len(),
        "the engine answers {} rows and the committed table holds {}",
        rebuilt.len(),
        committed.len()
    );
    for (n, (was, now)) in committed.iter().zip(&rebuilt).enumerate() {
        assert_eq!(
            was, now,
            "row {n} of the keyboard table is not what the engine says today"
        );
    }
}
