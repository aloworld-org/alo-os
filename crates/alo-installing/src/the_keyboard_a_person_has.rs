//! **Which `/dev/input/event*` is the keyboard a person is typing on.**
//!
//! `is_a_keyboard` has existed since a key first reached this environment and
//! has never had a caller: it answers *is this device a keyboard* about a
//! capabilities bitmask somebody else had to find. This is the somebody else.
//!
//! # Why a search rather than a fixed path
//!
//! `/dev/input/event0` is whatever the kernel enumerated first, and on the
//! machine this was measured on that is the **power button**. A program that
//! opened `event0` and waited would wait for ever while a person typed into
//! nothing. The number is not a property of the hardware and changes between
//! boots, so it is searched for every time rather than remembered.
//!
//! # Measured on a machine with three of them
//!
//! | `/sys/class/input` | name | what it is |
//! |---|---|---|
//! | `event0` | Power Button | one key, and not a keyboard |
//! | `event1` | Apple Inc. Virtual USB Keyboard | the keyboard |
//! | `event2` | Apple Inc. Virtual USB Digitizer | a tablet, which reports keys |
//!
//! Two of the three report keys. That is why [`crate::is_a_keyboard`] asks for
//! `A`, `Z` **and** `Enter` together rather than *has any key at all*, and it is
//! why this walks the list instead of trusting a position in it.
//!
//! # Everything is relative to a root
//!
//! The directory is an argument, so a test points this at a tree it built and
//! the production caller passes `/`. Nothing here needs a device node to exist:
//! the search reads sysfs only, and opening the event file is the caller's next
//! act.

use std::path::{Path, PathBuf};

use crate::is_a_keyboard;

/// Where the kernel lists what it has, under the root it is asked about.
pub const WHERE_THE_DEVICES_ARE: &str = "sys/class/input";

/// Where the event files are, under the same root.
pub const WHERE_THE_EVENTS_ARE: &str = "dev/input";

/// What a device keeps its key bitmask in, under its own directory.
const WHERE_ITS_KEYS_ARE: &str = "device/capabilities/key";

/// Why no keyboard was found.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NoKeyboard {
    /// The kernel lists no input devices at all.
    #[error("this machine lists no input devices: {0} is not there or holds nothing")]
    NothingIsListed(String),
    /// Devices are listed and none of them is a keyboard.
    #[error("none of the {0} input devices on this machine is a keyboard")]
    NoneIsAKeyboard(usize),
}

/// The keyboard a person is typing on, as a path to open.
///
/// Returns the **lowest-numbered** device that is a keyboard, so a machine with
/// two of them — a laptop's own and one plugged in — is answered
/// deterministically rather than by directory order. Both work; the point is
/// that two runs on one machine agree.
///
/// # Errors
///
/// [`NoKeyboard::NothingIsListed`] where the kernel lists nothing, which is a
/// machine this environment cannot ask anything of; and
/// [`NoKeyboard::NoneIsAKeyboard`] where it lists devices and none has `A`, `Z`
/// and `Enter`. The second says how many it looked at, because *none of three*
/// and *none of nothing* are different faults and the first is far worse.
pub fn the_keyboard_a_person_has(root: &Path) -> Result<PathBuf, NoKeyboard> {
    let listed = root.join(WHERE_THE_DEVICES_ARE);
    let Ok(entries) = std::fs::read_dir(&listed) else {
        return Err(NoKeyboard::NothingIsListed(listed.display().to_string()));
    };
    let events = the_event_numbers_in_order(
        entries
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned()),
    );
    if events.is_empty() {
        return Err(NoKeyboard::NothingIsListed(listed.display().to_string()));
    }

    for number in &events {
        let at = listed.join(format!("event{number}"));
        let Ok(capabilities) = std::fs::read_to_string(at.join(WHERE_ITS_KEYS_ARE)) else {
            continue;
        };
        if is_a_keyboard(&capabilities) {
            return Ok(root
                .join(WHERE_THE_EVENTS_ARE)
                .join(format!("event{number}")));
        }
    }
    Err(NoKeyboard::NoneIsAKeyboard(events.len()))
}

/// The `event*` devices among these names, by number, lowest first.
///
/// # Why this is not inside the walk above
///
/// *Lowest-numbered wins whatever order the names arrive in* is the property
/// that makes two runs on one machine agree, and it is the one a test cannot
/// reach through a directory. **Directory order belongs to the filesystem and
/// differs between them**, so a test that wants an order has to either accept
/// whatever it is given — and then cannot tell a sorted search from an unsorted
/// one — or assert a premise about it, which is a test that passes on one
/// filesystem and fails on another.
///
/// Measured on 2026-10-21: the first version of that test asserted the premise.
/// `read_dir` returns `event3` first on APFS, where it was written, so it was
/// adversarial and passed; it returns `event1` first on the hosted runner's
/// filesystem, so the premise assertion fired and **the merge queue could not
/// land the change**. Nothing was wrong with the code under test.
///
/// Taking the names as an argument moves the property somewhere a test can
/// supply the adversarial order itself, on any filesystem, with no premise to
/// assert.
fn the_event_numbers_in_order(names: impl Iterator<Item = String>) -> Vec<u32> {
    let mut events: Vec<u32> = names
        .filter_map(|name| {
            // `event12` and not `input12`, `mice` or `js0`: only an event device
            // has a file to read key presses from.
            name.strip_prefix("event")
                .and_then(|it| it.parse::<u32>().ok())
        })
        .collect();
    events.sort_unstable();
    events
}

#[cfg(test)]
#[path = "the_keyboard_a_person_has_tests.rs"]
mod tests;
