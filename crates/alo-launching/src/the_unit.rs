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
    let name = format!(
        "app-{LAUNCHER}-{}@{distinct:x}.service",
        escaped(identifier)
    );
    (name.len() <= 255).then_some(name)
}

/// Whether a unit with this name runs the application with this identifier.
///
/// The question ADR 0148's counter asks, so it is answered beside the rule that
/// names the units rather than by a second copy of it. Two shapes, both in the
/// person's own manager under `app.slice`:
///
/// - `app-alo-<escaped identifier>@<n>.service` — what this desktop starts when a
///   person presses an application ([`the_unit_for`]);
/// - `app-flatpak-<identifier>-<n>.scope` — what Flatpak moves an application
///   into when it starts it, with the identifier as Flatpak writes it, unescaped,
///   and `<n>` a number.
///
/// **Exact, never a prefix of another application's name**: `org.gnome.Text`
/// is not `org.gnome.TextEditor`, because what follows the identifier must be
/// the `@` or the `-<n>.scope` the shape requires.
#[must_use]
pub fn is_a_unit_of(unit: &str, identifier: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }
    if let Some(rest) = unit
        .strip_prefix("app-")
        .and_then(|rest| rest.strip_prefix(LAUNCHER))
        .and_then(|rest| rest.strip_prefix('-'))
        .and_then(|rest| rest.strip_prefix(escaped(identifier).as_str()))
        .and_then(|rest| rest.strip_prefix('@'))
    {
        return rest.strip_suffix(".service").is_some_and(|distinct| {
            !distinct.is_empty() && distinct.bytes().all(|b| b.is_ascii_hexdigit())
        });
    }
    unit.strip_prefix("app-flatpak-")
        .and_then(|rest| rest.strip_prefix(identifier))
        .and_then(|rest| rest.strip_prefix('-'))
        .and_then(|rest| rest.strip_suffix(".scope"))
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
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
