//! The window of what is running, opened on a written kernel and on this
//! machine's own: what it shows, what each number is, and what it refuses.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::path::Path;
use std::time::Duration;

use super::*;
use crate::WindowControlLabels;
use crate::desktop_list::{ListShows, picture};
use crate::desktop_look::DesktopLook;
use crate::desktop_raster::running_shows;
use crate::desktop_testing::{Written, a_second, an_afternoon, an_appearance, noon_look, words};
use alo_measuring::{Number, Process};
use alo_strings::Direction;
use smithay::utils::Rectangle;

/// A window opened on the written afternoon.
fn opened_on_an_afternoon() -> RunningWindow {
    let (earlier, later) = an_afternoon();
    let mut window = RunningWindow::closed();
    window.opened(Ok(earlier), Ok(later), a_second());
    window
}

/// What the window is showing, which must be what is running.
fn running_of(window: &RunningWindow) -> &Running {
    match window.shows() {
        RunningShows::Running { running, .. } => running,
        other => panic!("nothing running is shown: {other:?}"),
    }
}

/// The window drawn in the whole of a tall display, in `look`.
fn drawn(window: &RunningWindow, look: DesktopLook) -> crate::desktop_list::ListPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    let size = (1920, 2160);
    picture(
        &running_shows(window, &words()),
        &mut labels.fonts,
        size,
        Rectangle::from_size(size.into()),
        crate::desktop_list::ListLook {
            measure: look.measure(),
            palette: look.palette().unwrap(),
            right_to_left: look.reading() == Direction::RightToLeft,
        },
    )
    .unwrap()
}

/// A number as the test expects to see it drawn: the digits of the value the
/// kernel gave, or `alo-measuring`'s sentence in its place.
fn expected(number: &Number) -> String {
    match number.value() {
        Some(value) => format!("{value}"),
        None => number.instead(&words()).unwrap().into_text(),
    }
}

/// The process called `name`.
fn process<'a>(running: &'a Running, name: &str) -> &'a Process {
    running
        .processes()
        .iter()
        .find(|process| process.name == name)
        .unwrap()
}

/// **Every number in the window is the one the kernel gave**, as its own
/// digits: the process id, resident memory in bytes, the share of the
/// processor in thousandths, and bytes read, written, received and sent each
/// second — each drawn cell exactly the `alo_measuring::Number` behind it, in
/// the order `alo-measuring` holds the processes, with nothing rounded, scaled,
/// summed or sorted. A number the kernel would not show is `alo-measuring`'s
/// sentence and never a zero; a process that began since the last reading has
/// no rate yet and says so; and a process that ended is drawn as ended.
#[test]
fn every_number_in_the_running_window_is_the_one_the_kernel_gave() {
    let window = opened_on_an_afternoon();
    let running = running_of(&window);
    let strings = words();
    let picture = drawn(&window, noon_look(&an_appearance(), Direction::LeftToRight));

    // The machine's memory above the rows, as the kernel counted it.
    let remarks: Vec<&str> = picture.remarks.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        remarks,
        [
            expected(running.memory_total()).as_str(),
            expected(running.memory_available()).as_str()
        ]
    );
    assert_eq!(running.memory_total().value(), Some(16_000_000 * 1024));

    // Every process, then every process that ended, in alo-measuring's order.
    assert_eq!(
        picture.rows.len(),
        running.processes().len() + running.gone().len()
    );
    for (row, process) in picture.rows.iter().zip(running.processes()) {
        let cells: Vec<&str> = row.cells.iter().map(|cell| cell.text.as_str()).collect();
        assert_eq!(
            cells,
            [
                process.name.as_str(),
                format!("{}", process.pid).as_str(),
                expected(&process.memory).as_str(),
                expected(&process.processor).as_str(),
                expected(&process.read).as_str(),
                expected(&process.written).as_str(),
                expected(&process.received).as_str(),
                expected(&process.sent).as_str(),
                process.network.said(&strings).text(),
            ],
            "{}",
            process.name
        );
    }

    // What the written kernel said, checked by hand: 52 000 kB resident, 250 of
    // 1000 ticks, 900 000 bytes read and 70 000 written in the second.
    let editor = process(running, "editor");
    assert_eq!(editor.memory.value(), Some(52_000 * 1024));
    assert_eq!(editor.processor.value(), Some(250));
    assert_eq!(editor.read.value(), Some(900_000));
    assert_eq!(editor.written.value(), Some(70_000));
    let editor_row = &picture.rows[0];
    assert_eq!(editor_row.cells[2].text, "53248000");
    assert_eq!(editor_row.cells[3].text, "250");
    assert_eq!(editor_row.cells[4].text, "900000");

    // Withheld is a sentence, not a zero.
    let theirs = process(running, "theirs");
    assert!(matches!(theirs.read, Number::Withheld { .. }));
    let theirs_row = picture
        .rows
        .iter()
        .find(|row| row.cells[0].text == "theirs")
        .unwrap();
    assert_eq!(theirs_row.cells[4].text, "The kernel did not show this.");
    assert_ne!(theirs_row.cells[4].text, "0");

    // A newcomer has no rate yet, and says so.
    let newcomer_row = picture
        .rows
        .iter()
        .find(|row| row.cells[0].text == "newcomer")
        .unwrap();
    assert_eq!(
        newcomer_row.cells[3].text,
        "Started since the last reading."
    );

    // The sandboxed application's network is its own, and the window says so.
    let sandboxed_row = picture
        .rows
        .iter()
        .find(|row| row.cells[0].text == "sandboxed")
        .unwrap();
    assert_eq!(sandboxed_row.cells[6].text, "12000");
    assert_eq!(
        sandboxed_row.cells[8].text,
        "Network traffic counted for this process alone."
    );

    // The process that ended is the last row, drawn as ended.
    let ended = picture.rows.last().unwrap();
    let cells: Vec<&str> = ended.cells.iter().map(|cell| cell.text.as_str()).collect();
    assert_eq!(cells, ["finished", "404", "Ended since the last reading."]);

    // And no cell holds a sentence nobody collected.
    for row in &picture.rows {
        for cell in &row.cells {
            assert!(!cell.text.contains("measuring."), "{}", cell.text);
        }
    }
}

/// **This machine's own kernel.** Two readings of the real `/proc` a moment
/// apart open the window, and the process running this test is in it under its
/// own process id, with its memory drawn as the digits the kernel gave.
#[test]
fn the_running_window_reads_this_machines_kernel() {
    let earlier = Reading::now();
    std::thread::sleep(Duration::from_millis(250));
    let later = Reading::now();
    let mut window = RunningWindow::closed();
    window.opened(earlier, later, Duration::from_millis(250));
    let running = running_of(&window);
    let ours = running.process(std::process::id()).unwrap();
    assert!(ours.memory.value().is_some_and(|bytes| bytes > 0));
    assert_eq!(
        ours.memory.from().file(),
        Path::new(&format!("/proc/{}/status", ours.pid))
    );

    let picture = drawn(&window, noon_look(&an_appearance(), Direction::LeftToRight));
    assert!(!picture.rows.is_empty());
    for row in &picture.rows {
        let pid: u32 = row.cells[1].text.parse().unwrap();
        if let Some(process) = running.process(pid) {
            assert_eq!(row.cells[2].text, expected(&process.memory));
        }
    }
}

/// **A refusal is `alo-measuring`'s sentence in the window, never an empty
/// list**: the same reading twice is the same moment, no time between two
/// readings is no interval, and a kernel that cannot be read says where.
#[test]
fn a_reading_that_cannot_be_a_rate_is_refused_in_the_window() {
    let strings = words();
    let (earlier, later) = an_afternoon();

    let mut same = RunningWindow::closed();
    same.opened(Ok(earlier.clone()), Ok(earlier.clone()), a_second());
    assert!(matches!(
        same.shows(),
        RunningShows::Refusal(NotMeasured::SameMoment)
    ));

    let mut instant = RunningWindow::closed();
    instant.opened(Ok(earlier.clone()), Ok(later.clone()), Duration::ZERO);
    assert!(matches!(
        instant.shows(),
        RunningShows::Refusal(NotMeasured::NoInterval)
    ));

    let nothing = Written::default();
    let unreadable = Reading::of_kernel(&nothing, Path::new("/proc"));
    let mut blind = RunningWindow::closed();
    blind.opened(unreadable, Ok(later), a_second());
    let RunningShows::Refusal(why) = blind.shows() else {
        panic!("an unreadable kernel is not refused: {:?}", blind.shows());
    };
    assert!(matches!(why, NotMeasured::Unreadable { .. }));
    match running_shows(&blind, &strings) {
        ListShows::Refusal(said) => {
            assert_eq!(said, why.said(&strings).into_text());
            assert!(said.contains("/proc"), "{said}");
        }
        other => panic!("not drawn as a refusal: {other:?}"),
    }

    // Drawn, the refusal is a sentence in a panel with no rows.
    let picture = drawn(&blind, noon_look(&an_appearance(), Direction::LeftToRight));
    assert!(picture.rows.is_empty());
    assert_eq!(
        picture.refusal.unwrap().text,
        why.said(&strings).into_text()
    );
}

/// **Reading again is a rate over the reading the window holds**: the later
/// reading becomes the earlier one, a failed reading is refused in the window,
/// and the next good reading shows what is running again.
#[test]
fn reading_again_is_a_rate_over_the_reading_the_window_holds() {
    let (earlier, later) = an_afternoon();
    let mut window = RunningWindow::closed();
    window.opened(Ok(earlier.clone()), Ok(later.clone()), a_second());
    assert!(matches!(window.shows(), RunningShows::Running { .. }));

    // The same moment again: `later` is what the window already holds.
    window.read_again(Ok(later.clone()), a_second());
    assert!(matches!(
        window.shows(),
        RunningShows::Refusal(NotMeasured::SameMoment)
    ));

    // A reading that failed is refused, and the one after it recovers.
    window.read_again(Err(NotMeasured::NotOnThisHost), a_second());
    assert!(matches!(
        window.shows(),
        RunningShows::Refusal(NotMeasured::NotOnThisHost)
    ));
    let mut newer = Written::a_machine(3000);
    newer.process(
        101,
        "editor",
        50,
        crate::desktop_testing::Totals {
            ticks: 500,
            memory_kb: 52_000,
            read: 900_100,
            written: 70_200,
            received: 300,
            sent: 400,
        },
        4_026_531_840,
    );
    window.read_again(Ok(newer.reading()), Duration::from_secs(2));
    let running = running_of(&window);
    assert_eq!(running.interval(), Duration::from_secs(2));
    assert_eq!(running.process(101).unwrap().read.value(), Some(0));
}

/// **The view moves within the rows and no further**, a closed window does
/// nothing with any key or reading, and closing lets every reading go.
#[test]
fn the_view_moves_within_the_rows_and_a_closed_window_does_nothing() {
    let mut window = opened_on_an_afternoon();
    let rows = {
        let running = running_of(&window);
        running.processes().len() + running.gone().len()
    };
    assert_eq!(window.pressed(RunningKey::Up), RunningPressed::Nothing);
    assert_eq!(window.pressed(RunningKey::Down), RunningPressed::Moved);
    assert_eq!(window.pressed(RunningKey::Last), RunningPressed::Moved);
    assert!(matches!(
        window.shows(),
        RunningShows::Running { moved_past, .. } if moved_past == rows - 1
    ));
    assert_eq!(window.pressed(RunningKey::Down), RunningPressed::Nothing);
    assert_eq!(window.pressed(RunningKey::First), RunningPressed::Moved);
    assert_eq!(window.pressed(RunningKey::Nothing), RunningPressed::Nothing);
    assert_eq!(
        window.pressed(RunningKey::ReadAgain),
        RunningPressed::ReadAgain
    );
    assert_eq!(window.pressed(RunningKey::Close), RunningPressed::Closed);
    assert!(!window.is_open());
    assert_eq!(window.shows(), RunningShows::Nothing);

    for key in [
        RunningKey::Down,
        RunningKey::Last,
        RunningKey::ReadAgain,
        RunningKey::Close,
    ] {
        assert_eq!(window.pressed(key), RunningPressed::Nothing);
    }
    let (_, later) = an_afternoon();
    window.read_again(Ok(later), a_second());
    assert_eq!(window.shows(), RunningShows::Nothing);
    assert!(!window.is_open());
}
