//! Which screens the person might be watching.
//!
//! A laptop shows its console on its display; a virtual machine may show it on
//! a serial line; a server may do both. The kernel says which consoles it was
//! started with in `/sys/class/tty/console/active`, and a sentence meant for a
//! person who has no other window is written to **every one of them** — not to
//! whichever the service manager happened to connect, which is the last one
//! named and may be the one nobody is looking at.

use std::path::PathBuf;

/// Where the kernel lists its active consoles.
pub const ACTIVE: &str = "/sys/class/tty/console/active";

/// The device for every console the kernel lists, in the order it lists them.
///
/// A name that is not a plain terminal name is left out rather than opened: the
/// file is the kernel's, but a path built from it is still a path.
#[must_use]
pub fn every_console(active: &str) -> Vec<PathBuf> {
    active
        .split_ascii_whitespace()
        .filter(|name| !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric()))
        .map(|name| PathBuf::from("/dev").join(name))
        .collect()
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
}
