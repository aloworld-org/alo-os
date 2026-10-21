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

use super::{NoKeyboard, WHERE_THE_DEVICES_ARE, the_keyboard_a_person_has};

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

/// **The lowest-numbered keyboard wins, whatever order the directory is read
/// in** — and this test checks its own premise, because the first version of it
/// did not and was worthless.
///
/// A laptop with its own keyboard and one plugged in has two, and both would
/// work. What matters is that two runs on one machine agree rather than
/// depending on `read_dir`.
///
/// **Measured: removing the sort did not fail the first version of this test.**
/// Its fixture was `event9 event2 event11`, and `read_dir` on this filesystem
/// returns `event2` first — already the lowest — so the answer was the same
/// either way. `event1 event2 event3` comes back `event3` first, which is
/// adversarial.
///
/// Directory order is the filesystem's and cannot be forced from a test, so the
/// premise is **asserted**: if the first entry is already the lowest here, this
/// check cannot tell a sorted search from an unsorted one, and it says so
/// rather than passing. The remedy is other names, and the failure names it.
#[test]
fn the_lowest_numbered_keyboard_is_chosen_whatever_the_directory_order() {
    let root = a_machine_listing(&[
        ("event1", A_KEYBOARD),
        ("event2", A_KEYBOARD),
        ("event3", A_KEYBOARD),
    ]);

    // The premise, read off the filesystem this is running on.
    let first = std::fs::read_dir(root.join(WHERE_THE_DEVICES_ARE))
        .expect("the listing this test just wrote")
        .flatten()
        .map(|it| it.file_name().to_string_lossy().into_owned())
        .find(|name| name.starts_with("event"))
        .expect("a device in the listing");
    assert_ne!(
        first, "event1",
        "this filesystem returns the lowest-numbered device first, so this test \
         cannot tell a sorted search from an unsorted one. Pick other names: \
         `event8 event17 event4 event13` was adversarial where this was written."
    );

    assert_eq!(
        the_keyboard_a_person_has(&root).expect("a keyboard is listed"),
        root.join("dev/input/event1"),
        "the search answered the first device the directory gave rather than the \
         lowest-numbered one, so it did not sort"
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
