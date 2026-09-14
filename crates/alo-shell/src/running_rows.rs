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

use alo_measuring::{Number, Running};
use alo_strings::{Said, Strings};

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
pub(crate) fn remarks(running: &Running, strings: &Strings) -> Vec<String> {
    vec![
        number(running.memory_total(), strings),
        number(running.memory_available(), strings),
    ]
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
