//! **Every letter the plan names, typed through the rented compose tables.**
//!
//! `ü`, `è`, `ß`, `ł`, `ő`, `č`, `ġ`, `ħ`, the Irish letters and the Maltese
//! ones — each one pressed key by key here, against
//! `/usr/share/X11/locale/en_US.UTF-8/Compose`, the table `libX11` ships and
//! every other desktop composes through. **No table in this repository is
//! consulted**: if the assertion below holds, it holds because the rented data
//! says so.
//!
//! Two ways to each letter, and both are in the same table: a **dead key**,
//! which is a key on the layout that makes no letter of its own, and the
//! **compose key**, which is any key a person chose for it.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nobody has pressed
//! a key on a certified machine here: what is held is that the table on this
//! machine writes these letters for these sequences, and that alo OS neither
//! invents one nor loses one.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_keyboards::{Compose, Composing, NotATable, THE_TABLE, Typed, keyboard_words};
use alo_strings::Strings;

/// The rented table on this machine.
fn rented_table() -> Compose {
    let read = Compose::read();
    assert!(
        read.is_ok(),
        "the rented compose table ({THE_TABLE}) could not be read: {read:?} — alo OS ships \
         libX11's locale data, and a machine running these tests needs it installed"
    );
    let table = read.unwrap();
    assert!(
        table.how_many() > 1000,
        "the rented table has only {} sequences in it, which is not the table this test is about",
        table.how_many()
    );
    table
}

/// What these keys write on this machine, pressed one after another.
fn typing(table: &Compose, keys: &[&str]) -> Option<String> {
    Composing::on(table)
        .typing(keys)
        .unwrap()
        .written()
        .map(ToOwned::to_owned)
}

/// **Every letter the plan names, through a dead key.**
///
/// `Müller` and `Liège` first, because they are what the plan calls them: test
/// cases in a European product, not edge cases.
#[test]
fn every_letter_the_plan_names_is_typed_with_a_dead_key() {
    let table = rented_table();
    for (letter, keys) in [
        ("ü", ["dead_diaeresis", "u"]),
        ("è", ["dead_grave", "e"]),
        ("ł", ["dead_stroke", "l"]),
        ("ő", ["dead_doubleacute", "o"]),
        ("č", ["dead_caron", "c"]),
        ("ġ", ["dead_abovedot", "g"]),
        ("ħ", ["dead_stroke", "h"]),
        ("á", ["dead_acute", "a"]),
    ] {
        assert_eq!(
            typing(&table, &keys).as_deref(),
            Some(letter),
            "the rented table does not write {letter} for {keys:?}"
        );
    }
}

/// **And every one of them through a compose key**, which is the road for the
/// letters a person's own layout has no dead key for — `ß` among them, which is
/// why it is only in this half of the pair.
#[test]
fn every_letter_the_plan_names_is_typed_with_the_compose_key() {
    let table = rented_table();
    for (letter, keys) in [
        ("ü", vec!["Multi_key", "quotedbl", "u"]),
        ("è", vec!["Multi_key", "grave", "e"]),
        ("ß", vec!["Multi_key", "s", "s"]),
        ("ł", vec!["Multi_key", "slash", "l"]),
        ("ő", vec!["Multi_key", "equal", "o"]),
        ("č", vec!["Multi_key", "c", "c"]),
        ("ġ", vec!["Multi_key", "period", "g"]),
        ("ħ", vec!["Multi_key", "slash", "h"]),
    ] {
        assert_eq!(
            typing(&table, &keys).as_deref(),
            Some(letter),
            "the rented table does not write {letter} for {keys:?}"
        );
    }
}

/// **The Irish letters**: the five fadas, which is the whole of what Irish adds
/// to the Latin alphabet.
#[test]
fn the_irish_letters_are_typed() {
    let table = rented_table();
    for (letter, key) in [("á", "a"), ("é", "e"), ("í", "i"), ("ó", "o"), ("ú", "u")] {
        assert_eq!(
            typing(&table, &["dead_acute", key]).as_deref(),
            Some(letter),
            "no fada on {key}"
        );
        assert_eq!(
            typing(&table, &["Multi_key", "acute", key]).as_deref(),
            Some(letter)
        );
    }
    let upper = typing(&table, &["dead_acute", "A"]);
    assert_eq!(upper.as_deref(), Some("Á"));
}

/// **The Maltese letters**: `ċ`, `ġ`, `ħ` and `ż`, and their capitals — the
/// four that make Maltese Maltese, and the ones a product that shipped
/// "English plus the big five" would never have tried.
#[test]
fn the_maltese_letters_are_typed() {
    let table = rented_table();
    for (letter, keys) in [
        ("ċ", ["dead_abovedot", "c"]),
        ("ġ", ["dead_abovedot", "g"]),
        ("ħ", ["dead_stroke", "h"]),
        ("ż", ["dead_abovedot", "z"]),
        ("Ċ", ["dead_abovedot", "C"]),
        ("Ġ", ["dead_abovedot", "G"]),
        ("Ħ", ["dead_stroke", "H"]),
        ("Ż", ["dead_abovedot", "Z"]),
    ] {
        assert_eq!(
            typing(&table, &keys).as_deref(),
            Some(letter),
            "the rented table does not write {letter} for {keys:?}"
        );
    }
}

/// A dead key waits, shows that it is waiting, and writes on the second key —
/// which is what *Müller* is actually typed like.
#[test]
fn a_dead_key_waits_before_it_writes() {
    let table = rented_table();
    let mut composing = Composing::on(&table);
    assert_eq!(composing.typing(&["M"]).unwrap(), Typed::MeansItself);
    assert_eq!(
        composing.typing(&["dead_diaeresis"]).unwrap(),
        Typed::Waiting
    );
    assert_eq!(composing.waiting().len(), 1);
    assert_eq!(composing.typing(&["u"]).unwrap(), Typed::Wrote("ü"));
    assert!(composing.waiting().is_empty());
    for key in ["l", "l", "e", "r"] {
        assert_eq!(composing.typing(&[key]).unwrap(), Typed::MeansItself);
    }
}

/// **A sequence the rented table does not have writes nothing at all.**
///
/// The refusal that matters most here, because the tempting bug is the other
/// one: a table of our own that filled the gap, or a best guess at what the
/// person meant. alo OS types what the rented table says and nothing else.
#[test]
fn a_sequence_the_rented_table_does_not_have_writes_nothing() {
    let table = rented_table();
    for keys in [
        vec!["dead_diaeresis", "q"],
        vec!["dead_stroke", "q"],
        vec!["Multi_key", "q", "q"],
    ] {
        assert_eq!(
            typing(&table, &keys),
            None,
            "{keys:?} wrote something the rented table does not have"
        );
    }
    let mut composing = Composing::on(&table);
    assert_eq!(
        composing.typing(&["dead_diaeresis", "q"]).unwrap(),
        Typed::NothingInTheTable
    );
    assert!(composing.waiting().is_empty());
    assert_eq!(
        composing.typing(&["dead_diaeresis", "u"]).unwrap(),
        Typed::Wrote("ü")
    );
}

/// **A key that is not a key's name is refused**, rather than pressed as
/// something the rented table will never have.
#[test]
fn a_key_that_is_not_a_key_is_refused() {
    let table = rented_table();
    for key in ["dead acute", "<u>", "", "u; rm -rf /"] {
        assert!(
            Composing::on(&table).typing(&[key]).is_err(),
            "{key:?} was pressed"
        );
    }
}

/// **A table that is not a table is refused whole, and says so in the person's
/// own language** — with the file named, because the fault is in the machine
/// rather than in anything they did.
#[test]
fn a_table_that_is_not_one_is_refused_whole_and_said() {
    let at = Path::new(THE_TABLE);
    let refused =
        Compose::read_text("<dead_acute> <a> : \"á\"\nthis is not a sequence\n", at).unwrap_err();
    assert_eq!(refused.why(), &NotATable::NotASequence { line: 2 });

    let strings = Strings::of(keyboard_words().unwrap());
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
    assert!(said.text().contains(THE_TABLE), "{said}");
    for named in ["XKB", "keysym", "Multi_key", "dead_"] {
        assert!(!said.text().contains(named), "{said} names {named}");
    }
}
