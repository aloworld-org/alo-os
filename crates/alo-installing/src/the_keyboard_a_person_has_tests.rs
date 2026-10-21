//! **The three devices a real machine listed, and which of them is the
//! keyboard.**
//!
//! The bitmasks here are **not invented**. They were read off
//! `/sys/class/input/event*/device/capabilities/key` on a machine with a power
//! button, a keyboard and a digitizer on it — which is the case that matters,
//! because two of the three report keys and only one is a keyboard.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::{
    NoKeyboard, WHERE_THE_DEVICES_ARE, the_event_numbers_in_order, the_keyboard_a_person_has,
};

/// The names as an iterator of owned strings, which is what the walk hands over.
fn named(names: &[&str]) -> Vec<u32> {
    the_event_numbers_in_order(names.iter().map(|it| (*it).to_owned()))
}

/// A power button, measured. One key, high in the bitmask, and no letters.
const A_POWER_BUTTON: &str = "8000 10000000000000 0";

/// A keyboard, measured. Its last word has the letters and `Enter` in it.
const A_KEYBOARD: &str = "1000000000007 ff800000000007ff febeffdff3cfffff fffffffffffffffe";

/// A digitizer, measured. It reports keys — three of them — and is not a
/// keyboard.
const A_DIGITIZER: &str = "70000 0 0 0 0";

/// A tree with these devices listed in it, in this order.
fn a_machine_listing(devices: &[(&str, &str)]) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-keyboard-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    for (name, keys) in devices {
        let its = at.join(WHERE_THE_DEVICES_ARE).join(name);
        std::fs::create_dir_all(its.join("device/capabilities")).expect("a directory of our own");
        std::fs::write(its.join("device/capabilities/key"), keys).expect("writing a bitmask");
    }
    at
}

/// **The keyboard is found past the power button**, which is `event0` on the
/// machine these were measured from.
///
/// The failure this refuses: opening `event0` and waiting for ever while a
/// person types into nothing. The number is not a property of the hardware.
#[test]
fn the_keyboard_is_found_past_the_power_button() {
    let root = a_machine_listing(&[
        ("event0", A_POWER_BUTTON),
        ("event1", A_KEYBOARD),
        ("event2", A_DIGITIZER),
    ]);
    assert_eq!(
        the_keyboard_a_person_has(&root).expect("a keyboard is listed"),
        root.join("dev/input/event1")
    );
}

/// **A digitizer is not a keyboard**, even though it reports keys.
///
/// The second device on that machine that would pass *has any key at all*.
#[test]
fn a_digitizer_that_reports_keys_is_not_chosen() {
    let root = a_machine_listing(&[("event0", A_POWER_BUTTON), ("event1", A_DIGITIZER)]);
    assert_eq!(
        the_keyboard_a_person_has(&root),
        Err(NoKeyboard::NoneIsAKeyboard(2))
    );
}

/// **The lowest number comes first whatever order the names arrive in** — and
/// the order is this test's to choose, not the filesystem's.
///
/// A laptop with its own keyboard and one plugged in has two, and both would
/// work. What matters is that two runs on one machine agree rather than
/// depending on `read_dir`.
///
/// Every case below starts with a name that is **not** the lowest, so a search
/// that answered the first name it was given would fail here. That is what the
/// previous version of this test tried to arrange through a directory and could
/// not: `read_dir` returns `event3` first on APFS and `event1` first on the
/// hosted runner, so it asserted the premise, held on one filesystem and
/// **blocked the merge queue on the other**. The order is an argument now.
#[test]
fn the_lowest_number_is_first_whatever_order_the_names_arrive_in() {
    assert_eq!(named(&["event3", "event1", "event2"]), vec![1, 2, 3]);
    assert_eq!(named(&["event2", "event3", "event1"]), vec![1, 2, 3]);
    assert_eq!(named(&["event3", "event2", "event1"]), vec![1, 2, 3]);
    // Two digits, where sorting the text rather than the number answers 11.
    assert_eq!(named(&["event11", "event2", "event9"]), vec![2, 9, 11]);
}

/// **Only the event devices are numbered**, not the `input*`, `mice` and `js0`
/// beside them, and not an `event` with nothing after it.
#[test]
fn the_names_that_are_not_event_devices_are_dropped() {
    assert_eq!(
        named(&["input0", "mice", "js0", "event5", "event", "eventx"]),
        vec![5]
    );
    assert!(named(&["input0", "mice", "js0"]).is_empty());
}

/// **The lowest-numbered keyboard is the one answered**, through the real walk.
///
/// This is the same property as above seen end to end. It does not prove the
/// sort by itself — on a filesystem that happens to list `event1` first it
/// would pass unsorted — which is exactly why the sort is proved separately,
/// where the order is given rather than discovered.
#[test]
fn the_lowest_numbered_keyboard_is_the_one_answered() {
    let root = a_machine_listing(&[
        ("event1", A_KEYBOARD),
        ("event2", A_KEYBOARD),
        ("event3", A_KEYBOARD),
    ]);
    assert_eq!(
        the_keyboard_a_person_has(&root).expect("a keyboard is listed"),
        root.join("dev/input/event1")
    );
}

/// **`event11` is not read as `event1`**, which a prefix match would do.
#[test]
fn a_two_digit_device_is_its_own_number() {
    let root = a_machine_listing(&[("event1", A_POWER_BUTTON), ("event11", A_KEYBOARD)]);
    assert_eq!(
        the_keyboard_a_person_has(&root).expect("a keyboard is listed"),
        root.join("dev/input/event11")
    );
}

/// **Only `event*` is considered**, not the `input*`, `mice` and `js0` beside
/// them.
///
/// Measured: that directory holds `event0 event1 event2 input0 input1 input2
/// js0 mice`. Only an event device has a file to read presses from, and an
/// `input*` directory holds the same capabilities — so a search that took
/// anything with a bitmask would answer a path that cannot be opened.
#[test]
fn the_things_beside_the_event_devices_are_not_offered() {
    let root = a_machine_listing(&[
        ("input0", A_KEYBOARD),
        ("mice", A_KEYBOARD),
        ("js0", A_KEYBOARD),
    ]);
    assert_eq!(
        the_keyboard_a_person_has(&root),
        Err(NoKeyboard::NothingIsListed(
            root.join(WHERE_THE_DEVICES_ARE).display().to_string()
        )),
        "something that is not an event device was offered as one"
    );
}

/// **Nothing listed and nothing suitable are different faults**, and the second
/// says how many it looked at.
///
/// *None of three* means this machine has input devices and none is a keyboard,
/// which is strange hardware. *None of nothing* means the kernel listed no input
/// at all, which is a machine this environment cannot ask anything of. Reporting
/// them the same way would lose the difference exactly where somebody needs it.
#[test]
fn nothing_listed_is_a_different_fault_from_nothing_suitable() {
    let empty = a_machine_listing(&[]);
    std::fs::create_dir_all(empty.join(WHERE_THE_DEVICES_ARE)).expect("an empty listing");
    assert_eq!(
        the_keyboard_a_person_has(&empty),
        Err(NoKeyboard::NothingIsListed(
            empty.join(WHERE_THE_DEVICES_ARE).display().to_string()
        ))
    );

    let absent = Path::new("/this/does/not/exist/either");
    assert!(matches!(
        the_keyboard_a_person_has(absent),
        Err(NoKeyboard::NothingIsListed(_))
    ));

    let three = a_machine_listing(&[
        ("event0", A_POWER_BUTTON),
        ("event1", A_DIGITIZER),
        ("event2", A_POWER_BUTTON),
    ]);
    assert_eq!(
        the_keyboard_a_person_has(&three),
        Err(NoKeyboard::NoneIsAKeyboard(3))
    );
}

/// **A device with no bitmask file is stepped over rather than ending the
/// search.**
///
/// A device can be listed while its capabilities are unreadable — a permission,
/// a race with the kernel enumerating. Treating that as *no keyboard on this
/// machine* would refuse a person their own keyboard because of a device that
/// has nothing to do with it.
#[test]
fn a_device_that_cannot_be_read_does_not_end_the_search() {
    let root = a_machine_listing(&[("event1", A_KEYBOARD)]);
    std::fs::create_dir_all(root.join(WHERE_THE_DEVICES_ARE).join("event0"))
        .expect("a device with nothing in it");
    assert_eq!(
        the_keyboard_a_person_has(&root).expect("the keyboard past the unreadable one"),
        root.join("dev/input/event1")
    );
}

/// **And on this machine**, whatever it is, the answer is a path that exists.
///
/// Ignored, because a build machine need not have a keyboard at all and a test
/// that passed when there was none would be worthless. Run it where there is
/// one: it is the only check here that reads real hardware rather than a tree
/// this file wrote.
#[test]
#[ignore = "reads this machine's own /sys/class/input; run with --ignored"]
fn this_machine_has_a_keyboard_and_its_path_is_there() {
    let found = the_keyboard_a_person_has(Path::new("/"))
        .expect("this machine lists a keyboard — run this only where one is attached");
    assert!(
        found.exists(),
        "the search answered {found:?}, which is not there"
    );
    // `Path::starts_with` compares whole components, so a path literally
    // beginning `/dev/input/event1` does NOT "start with" `/dev/input/event`.
    // The first version of this asserted that and failed on a correct answer,
    // reporting *which is not an event device* about a path that plainly is.
    assert_eq!(
        found.parent(),
        Some(Path::new("/dev/input")),
        "the search answered {found:?}, which is not under /dev/input"
    );
    assert!(
        found
            .file_name()
            .and_then(|it| it.to_str())
            .is_some_and(|it| it.starts_with("event")),
        "the search answered {found:?}, whose name is not an event device"
    );
}
