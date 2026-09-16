//! Which screens the person might be watching, and which lines are not screens.
//!
//! A laptop shows its console on its display; a virtual machine may show it on
//! a serial line; a server may do both. The kernel says which consoles it was
//! started with in `/sys/class/tty/console/active`, and a sentence meant for a
//! person who has no other window is written to **every one of them** — not to
//! whichever the service manager happened to connect, which is the last one
//! named and may be the one nobody is looking at.
//!
//! **The machinery's own words go only to the serial lines.** When a program
//! this environment runs fails, what it complained of names the machinery, and
//! no sentence a person reads may (`docs/features.md`: *a person never learns
//! the name of anything we rented*). A serial line is where a technician, or the
//! virtual-machine test, reads a machine that has no other log they can reach —
//! so the complaint is written there, and never on a virtual terminal, which is
//! the screen.

use std::path::PathBuf;

/// Where the kernel lists its active consoles.
pub const ACTIVE: &str = "/sys/class/tty/console/active";

/// The device for every console the kernel lists, in the order it lists them.
///
/// A name that is not a plain terminal name is left out rather than opened: the
/// file is the kernel's, but a path built from it is still a path.
#[must_use]
pub fn every_console(active: &str) -> Vec<PathBuf> {
    plain_names(active)
        .map(|name| PathBuf::from("/dev").join(name))
        .collect()
}

/// The device for every console the kernel lists that is **not a screen**:
/// every one but the machine's own terminals.
///
/// `ttyS0`, `hvc0` and `ttyAMA0` are lines to somewhere else; `tty0` and `tty1`
/// are the display in front of the person.
#[must_use]
pub fn every_serial_line(active: &str) -> Vec<PathBuf> {
    plain_names(active)
        .filter(|name| !is_a_screen(name))
        .map(|name| PathBuf::from("/dev").join(name))
        .collect()
}

/// The names in the kernel's list that are plain terminal names.
fn plain_names(active: &str) -> impl Iterator<Item = &str> {
    active
        .split_ascii_whitespace()
        .filter(|name| !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// Whether a console is the machine's own terminal rather than a line to
/// somewhere else: a virtual terminal, `tty` and a number, or `tty` itself.
fn is_a_screen(name: &str) -> bool {
    name.strip_prefix("tty")
        .is_some_and(|number| number.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The display and the serial line are both written to.
    #[test]
    fn every_active_console_is_written_to() {
        assert_eq!(
            every_console("tty0 ttyS0\n"),
            [PathBuf::from("/dev/tty0"), PathBuf::from("/dev/ttyS0")]
        );
        assert_eq!(every_console(""), Vec::<PathBuf>::new());
    }

    /// A name that is not a terminal's is never opened.
    #[test]
    fn a_name_that_is_not_a_terminals_is_left_out() {
        assert_eq!(
            every_console("../sda tty0 hvc/0 ttyS0"),
            [PathBuf::from("/dev/tty0"), PathBuf::from("/dev/ttyS0")]
        );
    }

    /// **The machinery's words never reach a screen**: of the loader's two
    /// consoles only the serial line is one, and a machine with only a display
    /// has none.
    #[test]
    fn a_screen_is_never_a_serial_line() {
        assert_eq!(
            every_serial_line("tty0 ttyS0\n"),
            [PathBuf::from("/dev/ttyS0")]
        );
        assert_eq!(
            every_serial_line("hvc0 tty1 ttyAMA0 tty63"),
            [PathBuf::from("/dev/hvc0"), PathBuf::from("/dev/ttyAMA0")]
        );
        assert_eq!(every_serial_line("tty0"), Vec::<PathBuf>::new());
        assert_eq!(every_serial_line(""), Vec::<PathBuf>::new());
        // `tty` alone is a terminal of this machine's, not a line, and a name
        // that only begins like a virtual terminal's is not one — nor is a path
        // that ends in one.
        assert_eq!(
            every_serial_line("tty ttyX1 ../tty0"),
            [PathBuf::from("/dev/ttyX1")]
        );
    }
}
