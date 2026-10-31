//! What the unit an application runs in is called.
//!
//! systemd's *Desktop Environment Integration* convention, which GNOME and KDE
//! both follow: an application a desktop starts runs in
//!
//! ```text
//! app-<launcher>-<ApplicationID>@<RANDOM>.service
//! ```
//!
//! in the person's own manager, under `app.slice`. The launcher here is `alo`,
//! the application ID is the entry's desktop file ID, and `<RANDOM>` tells two
//! windows of one application apart. **`-` separates the parts, so a `-` in
//! the application ID is written `\x2d`**, as is every byte a unit name may
//! not hold — the convention's own rule, and `systemd-escape`'s spelling.
//!
//! The name is what makes this checkable on a machine: `systemctl --user
//! list-units 'app-alo-*'` lists every application a person's presses
//! started, and which one is which.

use std::fmt::Write as _;

/// The launcher's part of the name.
const LAUNCHER: &str = "alo";

/// The unit for one start of the application with this identifier.
///
/// `distinct` is anything that differs between two starts — the desktop uses
/// the clock and a counter. It is written in hexadecimal, so it can never hold
/// a character the name could not.
///
/// [`None`] when the name would be longer than the 255 bytes systemd allows,
/// which an escaped identifier can be: the desktop then says so rather than
/// asking the manager for a unit it will refuse.
#[must_use]
pub fn the_unit_for(identifier: &str, distinct: u64) -> Option<String> {
    let name = format!("app-{LAUNCHER}-{}@{distinct:x}.service", escaped(identifier));
    (name.len() <= 255).then_some(name)
}

/// An application ID written so that it is one part of a unit name.
fn escaped(identifier: &str) -> String {
    let mut out = String::with_capacity(identifier.len());
    for (at, byte) in identifier.bytes().enumerate() {
        let kept = byte.is_ascii_alphanumeric() || byte == b'_' || byte == b':' || byte == b'.';
        // A leading `.` would make the part look hidden; `systemd-escape`
        // writes it escaped, and so does this.
        if kept && !(at == 0 && byte == b'.') {
            out.push(char::from(byte));
        } else {
            // Writing to a `String` cannot fail.
            let _ = write!(out, "\\x{byte:02x}");
        }
    }
    out
}

#[cfg(test)]
#[path = "the_unit_tests.rs"]
mod tests;
