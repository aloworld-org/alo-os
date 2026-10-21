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
fn every_row() -> impl Iterator<Item = (&'static str, u16, &'static str, Option<char>)> {
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
        // `dead` in the fourth column is a level that produces no character.
        // Anything else must parse, so a mangled row still panics.
        let character = if point == "dead" {
            None
        } else {
            Some(
                a_code_point(point)
                    .unwrap_or_else(|| panic!("a row's code point is not one: {row:?}")),
            )
        };
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
/// `KEY_1` on a Belgian board — the ampersand key, whose type is `TWO_LEVEL`
/// and therefore does not consult caps lock.
const BELGIAN_AMPERSAND: u16 = 2;
/// `KEY_SPACE`, whose type is `ONE_LEVEL` and consults nothing at all.
const SPACE_BAR: u16 = 57;
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

/// **Caps lock does nothing to a key whose type does not list it**, rather than
/// stopping that key answering at all.
///
/// *Found by the desktop lane on 2026-10-21, measured against this table.*
/// Belgian's ampersand key has rows for nothing, `Shift`, `Mod5` and
/// `Mod5 Shift` and **no `Lock` row**, because `xkbcli how-to-type` only names
/// modifiers that *reach* a character and caps lock reaches nothing on a digit
/// key. Matched strictly, a person with caps lock on typed **nothing** from
/// that key — a Wi-Fi password stopping dead with no report but *the password
/// was refused*.
///
/// xkb's own rule is that a modifier a key's type does not list is ignored by
/// that key: `TWO_LEVEL` lists `Shift` and not `Lock`.
#[test]
fn caps_lock_does_nothing_to_a_key_that_does_not_consult_it() {
    assert_eq!(
        what_a_key_gives("be", BELGIAN_AMPERSAND, Held::nothing()),
        Some('&')
    );
    assert_eq!(
        what_a_key_gives("be", BELGIAN_AMPERSAND, Held::nothing().and_caps()),
        Some('&'),
        "caps lock stopped a key that does not consult it"
    );
    assert_eq!(
        what_a_key_gives("be", BELGIAN_AMPERSAND, Held::nothing().and_shift()),
        Some('1')
    );
    assert_eq!(
        what_a_key_gives(
            "be",
            BELGIAN_AMPERSAND,
            Held::nothing().and_shift().and_caps()
        ),
        Some('1'),
        "caps lock changed a key that does not consult it"
    );
}

/// **The space bar gives a space with anything held**, which is `ONE_LEVEL`.
///
/// The second hole of the same kind, and the more embarrassing one: one row in
/// the table, so a shifted space typed nothing. Every one of the eight
/// combinations is asserted, because a rule that holds for three of them and
/// not the fourth is the shape this fix exists to remove.
#[test]
fn the_space_bar_gives_a_space_whatever_is_held() {
    for layout in EVERY_LAYOUT {
        for held in [
            Held::nothing(),
            Held::nothing().and_shift(),
            Held::nothing().and_caps(),
            Held::nothing().and_altgr(),
            Held::nothing().and_shift().and_caps(),
            Held::nothing().and_shift().and_altgr(),
            Held::nothing().and_caps().and_altgr(),
            Held::nothing().and_shift().and_caps().and_altgr(),
        ] {
            assert_eq!(
                what_a_key_gives(layout, SPACE_BAR, held),
                Some(' '),
                "{layout} gave no space for {held:?}"
            );
        }
    }
}

/// **And the dropping does not go too far**: a key that *does* consult caps
/// lock still answers to it.
///
/// The control on the fix above. Masking every modifier away would make both
/// tests above pass and break every letter on every layout, so the letter keys
/// are asserted in the same breath.
#[test]
fn a_key_that_does_consult_caps_lock_still_answers_to_it() {
    assert_eq!(
        what_a_key_gives("be", PHYSICAL_Q, Held::nothing().and_caps()),
        Some('A')
    );
    assert_eq!(
        what_a_key_gives("be", PHYSICAL_Q, Held::nothing()),
        Some('a')
    );
    assert_eq!(
        what_a_key_gives("fr", PHYSICAL_Q, Held::nothing().and_caps().and_shift()),
        Some('a')
    );
    assert_eq!(
        what_a_key_gives("be", PHYSICAL_Q, Held::nothing().and_altgr()),
        Some('@'),
        "masking dropped AltGr from a key that consults it"
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

    let probe = std::process::Command::new("xkbcli")
        .args(["how-to-type", "--layout", "us", "0x0061"])
        .output();
    let Ok(probe) = probe else {
        panic!(
            "xkbcli is not on this machine, and this test does not pass without it. \
             Install libxkbcommon-tools, or leave this test ignored."
        )
    };
    assert!(
        probe.status.success() && String::from_utf8_lossy(&probe.stdout).contains("AC01"),
        "xkbcli is here and did not answer the question this test asks of it"
    );

    // Keyed by (layout, evdev, modifiers) and ordered by it, so the file's order
    // is a property of the data rather than of the order the queries ran in.
    let mut rows: std::collections::BTreeMap<(&str, u16, String), String> =
        std::collections::BTreeMap::new();

    for layout in EVERY_LAYOUT {
        // **The characters.** What the engine says produces each one.
        for point in every_code_point_asked_for() {
            let character = char::from_u32(point).expect("a code point this test named");
            for (evdev, held) in
                what_the_engine_said(&how_to_type(layout, &format!("0x{point:04X}"), false))
            {
                rows.entry((layout, evdev, held))
                    .or_insert_with(|| format!("U+{point:04X}\t{character}"));
            }
        }
        // **The dead levels.** A level whose keysym produces no character has no
        // row above, and an absent row and an ignored modifier are the same
        // absence — which is how AltGr-with-shift on Belgian's circumflex key
        // answered `[`, its own level 3, instead of nothing. Asked by keysym
        // NAME, which `how-to-type` accepts, so the engine still answers and
        // nothing here interprets its format.
        for name in dead_keysyms_of(layout) {
            for (evdev, held) in what_the_engine_said(&how_to_type(layout, &name, true)) {
                rows.entry((layout, evdev, held))
                    .or_insert_with(|| format!("dead\t{name}"));
            }
        }
    }

    let mut rebuilt = String::new();
    for ((layout, evdev, held), what) in &rows {
        writeln!(rebuilt, "{layout}\t{evdev}\t{held}\t{what}").expect("writing to a String");
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

/// Ask the engine about one character or one keysym on one layout.
fn how_to_type(layout: &str, what: &str, by_keysym: bool) -> String {
    let mut args = vec!["how-to-type"];
    if by_keysym {
        args.push("--keysym");
    }
    args.extend(["--layout", layout, what]);
    let said = std::process::Command::new("xkbcli")
        .args(&args)
        .output()
        .expect("xkbcli answered a moment ago");
    String::from_utf8_lossy(&said.stdout).into_owned()
}

/// Every (evdev code, modifiers) the engine named for the first layout group.
///
/// The modifier names are sorted so a row is one string rather than a set, and
/// the keycode has [`THE_X11_OFFSET`] taken off it because this table is read
/// against `/dev/input/event*`.
fn what_the_engine_said(out: &str) -> Vec<(u16, String)> {
    let mut found = Vec::new();
    for line in out.lines() {
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
        if before.split_whitespace().nth(1) != Some("1") {
            continue;
        }
        let mut held: Vec<&str> = mods.split_whitespace().collect();
        held.sort_unstable();
        found.push((x11 - THE_X11_OFFSET, held.join(" ")));
    }
    found
}

/// Which dead keysyms this layout actually puts on a key.
///
/// **Chosen from the data rather than from a list somebody wrote.** A compiled
/// keymap names every keysym on every key, so the dead ones are read out of it —
/// 17 across the eight layouts, 13 to 16 on each, and **none at all on `us`**,
/// which is exactly why a divergence here is invisible to anyone testing on a
/// US board.
///
/// This reads names out of a flat generated list. It does not resolve an
/// `include`, choose a variant or infer a type, which is the line between
/// configuring the engine and becoming it.
fn dead_keysyms_of(layout: &str) -> Vec<String> {
    let said = std::process::Command::new("xkbcli")
        .args(["compile-keymap", "--layout", layout])
        .output()
        .expect("xkbcli compiles a keymap");
    let text = String::from_utf8_lossy(&said.stdout);
    let Some((_, symbols)) = text.split_once("xkb_symbols") else {
        panic!("a compiled keymap for {layout} has no symbols section");
    };
    let mut names = std::collections::BTreeSet::new();
    // Key declarations run over several lines on nineteen to twenty-one keys per
    // layout, so this accumulates to the `;` rather than reading line by line —
    // the desktop lane's parse read one-line keys only and German could not type
    // a question mark.
    let mut inside: Option<String> = None;
    for line in symbols.lines() {
        if inside.is_none() && line.contains("key <") {
            inside = Some(String::new());
        }
        if let Some(held) = inside.as_mut() {
            held.push_str(line);
            if line.contains(';') {
                let whole = std::mem::take(held);
                inside = None;
                let Some(at) = whole.find('[') else { continue };
                for word in whole[at..].split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if word.starts_with("dead_") {
                        names.insert(word.to_owned());
                    }
                }
            }
        }
    }
    names.into_iter().collect()
}

/// **Every answer the engine gives, reproduced by the lookup** — the desktop
/// lane's oracle, which is the part worth keeping from the branch they and I
/// built in parallel.
///
/// It asks the question the other way round: for every character on every
/// offered layout, `xkbcli how-to-type` says which key and which modifiers
/// produce it, and the lookup must produce **that character** for that key and
/// those modifiers. Several thousand independent facts rather than the dozen
/// hand-picked ones above.
///
/// **Why this is worth having beside them, and what it does not do.** The tests
/// above are properties somebody chose; this is every fact the engine will
/// state — 1,768 of them, and a per-layout control refusing a run where any
/// layout states fewer than a hundred.
///
/// **It does not catch the two caps-lock holes, and this was measured rather
/// than assumed.** With the masking removed from `what_a_key_gives` this test
/// still passes, and `caps_lock_does_nothing_to_a_key_that_does_not_consult_it`
/// fails. The reason is structural: `how-to-type` names only the modifiers that
/// **reach** a character, so an oracle built from its answers can never ask
/// *what does this key do with caps lock on when caps lock reaches nothing*.
/// Asking in this direction proves the lookup repeats the engine; it cannot
/// prove the lookup answers everything a keyboard can be asked.
///
/// So the two kinds of check cover different things and both are kept. An
/// oracle that **would** catch the holes asks the other way round — every key
/// against all eight modifier combinations — which the desktop lane's own did,
/// and which is worth building if anyone wants that coverage. This one is not
/// it, and saying so is cheaper than somebody discovering it the way I did.
///
/// *Credited rather than reinvented: the two lanes built this table the same
/// night, theirs gated before pushing and so looked like nothing at all for two
/// hours. They did not open a competing pull request and sent the oracle
/// instead.*
///
/// Ignored for the same reason as the regeneration test: it needs `xkbcli`, and
/// `cargo test` printing `ignored` is honest where a silent skip would not be.
#[test]
#[ignore = "needs xkbcli from libxkbcommon-tools; run with --ignored"]
fn the_lookup_reproduces_every_answer_the_engine_gives() {
    let mut facts = 0usize;
    let mut by_layout: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut wrong: Vec<String> = Vec::new();

    for layout in EVERY_LAYOUT {
        for point in every_code_point_asked_for() {
            let said = std::process::Command::new("xkbcli")
                .args(["how-to-type", "--layout", layout, &format!("0x{point:04X}")])
                .output()
                .expect("xkbcli answers");
            let wanted = char::from_u32(point).expect("a code point this test named");
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
                if before.split_whitespace().nth(1) != Some("1") {
                    continue;
                }
                let held = Held {
                    shift: mods.split_whitespace().any(|it| it == "Shift"),
                    altgr: mods.split_whitespace().any(|it| it == "Mod5"),
                    caps: mods.split_whitespace().any(|it| it == "Lock"),
                };
                let evdev = x11 - THE_X11_OFFSET;
                facts += 1;
                *by_layout.entry(layout).or_default() += 1;
                let got = what_a_key_gives(layout, evdev, held);
                if got != Some(wanted) {
                    wrong.push(format!(
                        "{layout} key {evdev} with [{}] should give {wanted:?} and gives {got:?}",
                        mods.trim()
                    ));
                }
            }
        }
    }

    // **The control, and its first threshold was borrowed and wrong.**
    //
    // It asserted `facts > 2_000`, because the desktop lane measured 2,127 — and
    // this run states **1,768**, because their oracle asks for a wider set of
    // code points than `every_code_point_asked_for`. A number measured by
    // another method is not a control on this one, and the check fired on the
    // first run and said so.
    //
    // Counted per layout instead, which is justified by this method rather than
    // by anybody's total: a layout that silently answered nothing is the failure
    // worth refusing, and no single number has to be defended.
    for layout in EVERY_LAYOUT {
        let its_own = by_layout.get(layout).copied().unwrap_or_default();
        assert!(
            its_own > 100,
            "the engine stated only {its_own} facts for {layout}, which is too few to \
             be a layout — the tool or this parse is broken, not the lookup"
        );
    }
    assert_eq!(
        facts,
        by_layout.values().sum::<usize>(),
        "the per-layout counts do not add up to the total, so this control is broken"
    );
    assert!(
        wrong.is_empty(),
        "{} of {facts} answers the engine gives are not what the lookup gives:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// **A dead level gives nothing, rather than the character on the level beside
/// it.**
///
/// *Predicted by the desktop lane on 2026-10-21 and measured here.* Belgian's
/// `AD11` is `[dead_circumflex, dead_diaeresis, bracketleft, dead_abovering]` —
/// **three of its four levels are dead**, so only level 3 produces a character
/// and the table has exactly one row for that key: `Mod5` gives `[`.
///
/// The masking in `what_a_key_gives` reads the modifiers a key consults out of
/// its own rows, so for this key it sees `Mod5` alone. That is right for three
/// of the four combinations and **wrong for the fourth**: AltGr with shift is
/// level 4, which is dead, and dropping the shift lands it on level 3's `[`.
///
/// # Why it is recorded rather than fixed here
///
/// It cannot be fixed from this table: nothing in it records that a level is
/// dead, so *this level produces no character* and *this key ignores this
/// modifier* are the same absence. The fix is to generate each key's consulted
/// modifiers from `compile-keymap`'s own `type` declarations instead of
/// inferring them from the rows — which would also have made the caps-lock
/// holes impossible rather than repaired. That is a generator change and a
/// regenerated table, and it is this lane's next piece.
///
/// **This test pins the wrong answer on purpose.** When the generator learns
/// about dead levels this test fails, which is how it should be found.
#[test]
fn a_dead_level_gives_nothing_rather_than_its_neighbour() {
    const BELGIAN_CIRCUMFLEX: u16 = 26;

    // The three that are right.
    assert_eq!(
        what_a_key_gives("be", BELGIAN_CIRCUMFLEX, Held::nothing()),
        None,
        "level 1 is dead_circumflex and gives no character"
    );
    assert_eq!(
        what_a_key_gives("be", BELGIAN_CIRCUMFLEX, Held::nothing().and_shift()),
        None,
        "level 2 is dead_diaeresis and gives no character"
    );
    assert_eq!(
        what_a_key_gives("be", BELGIAN_CIRCUMFLEX, Held::nothing().and_altgr()),
        Some('['),
        "level 3 is bracketleft"
    );

    // And the one that is wrong, pinned.
    assert_eq!(
        what_a_key_gives(
            "be",
            BELGIAN_CIRCUMFLEX,
            Held::nothing().and_altgr().and_shift()
        ),
        None,
        "level 4 is dead_abovering and gives no character"
    );

    // The control on the whole fix: this key now has four rows, one per
    // level, three of them dead. Before the generator learnt about dead
    // levels it had exactly one.
    let its_rows: Vec<_> = every_row()
        .filter(|(layout, code, ..)| *layout == "be" && *code == BELGIAN_CIRCUMFLEX)
        .collect();
    assert_eq!(
        its_rows.len(),
        4,
        "every level of this key should have a row"
    );
    assert_eq!(
        its_rows.iter().filter(|(.., what)| what.is_none()).count(),
        3,
        "three of this key's four levels are dead"
    );
}
