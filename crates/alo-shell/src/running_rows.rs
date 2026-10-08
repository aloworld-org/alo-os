//! What is running, as the rows of a panel: every number the kernel gave, as
//! its own digits, and every sentence `alo-measuring` says in a number's place.
//!
//! One row per process, in the order `alo_measuring::Running` holds them —
//! its name, its process id, its resident memory in bytes, its share of the
//! processor in thousandths, the bytes it read, wrote, received and sent each
//! second, and whose network those last two are — then one row per process
//! that ended between the readings, with `alo-measuring`'s sentence for that.
//! Above the rows, the machine's memory and the memory still available, in
//! bytes.
//!
//! **Nothing here makes a number.** A number the kernel gave is drawn as the
//! digits of that number: never rounded, scaled into a unit, added up, averaged
//! or sorted by. A number the kernel did not give is `alo-measuring`'s
//! sentence, never a zero. How a large number is grouped for a reader's region
//! is not decided in any crate yet, so none is grouped here.

use alo_measuring::{Number, Running, words};
use alo_strings::{Filling, Said, Strings, Word};

use crate::desktop_list::ListRow;

/// A number as drawn: its digits, or the sentence in its place.
pub(crate) fn number(number: &Number, strings: &Strings) -> String {
    match number.value() {
        Some(value) => value.to_string(),
        None => number
            .instead(strings)
            .map(Said::into_text)
            .unwrap_or_default(),
    }
}

/// The machine's memory, then the memory available, drawn above the rows.
///
/// **Each carries the word for what it is.** Until 2026-10-08 these were two
/// bare numbers — a person saw `33657806848` above `32728788992` with nothing
/// to tell them which was which, or that either was memory at all. The file
/// knew: the line above this one has said so all along, to whoever read the
/// source.
///
/// The sentence is `alo-measuring`'s rather than one of this crate's, because
/// *what this number is* is a fact about the reading and not about the drawing,
/// and that crate already owns every other sentence these two windows say.
///
/// **The digits are untouched.** The rule at the top of this file stands — the
/// number is drawn as the number, never rounded and never scaled into a unit.
/// A label is not a unit.
pub(crate) fn remarks(running: &Running, strings: &Strings) -> Vec<String> {
    vec![
        beside(&words::MEMORY_ALL, running.memory_total(), strings),
        beside(
            &words::MEMORY_AVAILABLE,
            running.memory_available(),
            strings,
        ),
    ]
}

/// One reading with the word for what it is beside it.
///
/// The reading still goes through [`number`], so one the kernel did not give is
/// `alo-measuring`'s sentence in its place rather than a zero — and it reads
/// correctly under a label, because that sentence says why there is no number.
fn beside(word: &Word, reading: &Number, strings: &Strings) -> String {
    strings
        .say(&word.key(), &Filling::of("bytes", number(reading, strings)))
        .into_text()
}

/// Every process still running, then every process that ended, as rows.
pub(crate) fn rows(running: &Running, strings: &Strings) -> Vec<ListRow> {
    let mut rows: Vec<ListRow> = running
        .processes()
        .iter()
        .map(|process| ListRow {
            depth: 0,
            opened: None,
            cells: vec![
                process.name.clone(),
                process.pid.to_string(),
                number(&process.memory, strings),
                number(&process.processor, strings),
                number(&process.read, strings),
                number(&process.written, strings),
                number(&process.received, strings),
                number(&process.sent, strings),
                process.network.said(strings).into_text(),
            ],
        })
        .collect();
    rows.extend(running.gone().iter().map(|gone| ListRow {
        depth: 0,
        opened: None,
        cells: vec![
            gone.name.clone(),
            gone.pid.to_string(),
            gone.said(strings).into_text(),
        ],
    }));
    rows
}
