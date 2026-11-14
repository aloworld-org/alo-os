//! **The short name of an application identifier**: its last part, in lower
//! case, less a version number it begins with.
//!
//! `org.gnome.TextEditor` is `texteditor`, `com.vscodium.codium` is `codium`,
//! and `libreoffice26.2-writer` is `writer`. It is the name a person types after
//! `alo-app` (`docs/contracts/terminal-commands.md`, *Naming the application*)
//! and one of the names an agent's call may give for an installed application
//! ([`crate::answering_to`]).
//!
//! **One definition, here.** It lived in `alo-app` until 2026-11-14, when the
//! agent's road needed the same answer; two definitions of one name would be
//! two answers to *which application is `writer`* on one machine. `alo-app`
//! re-exports this, and its tests of the rule stay where the rule was found.

/// The short name of an identifier: its last part, in lower case — **less a
/// version number and the hyphen after it** where that part begins with one.
///
/// The second half is for identifiers that are not reverse-DNS.
/// `libreoffice26.2-writer` has its last dot inside a version number, so its
/// last part is `2-writer`; its short name is `writer`. It never changes a
/// reverse-DNS identifier, because an element of one may not begin with a
/// digit (the D-Bus naming rule application identifiers follow).
#[must_use]
pub fn short_name(identifier: &str) -> String {
    let last = the_last_part(identifier);
    match without_a_version(&last) {
        Some(rest) => rest.to_owned(),
        None => last,
    }
}

/// The last part of an identifier, in lower case: the short name as `alo-app`
/// read it until 2026-11-08, which still reaches the application it reached.
#[must_use]
pub fn the_last_part(identifier: &str) -> String {
    identifier
        .rsplit('.')
        .next()
        .unwrap_or(identifier)
        .to_lowercase()
}

/// `part` less the digits it begins with and the hyphen after them, when it
/// begins that way and something is left.
fn without_a_version(part: &str) -> Option<&str> {
    let digits = part.bytes().take_while(u8::is_ascii_digit).count();
    let rest = part.get(digits..)?.strip_prefix('-')?;
    (digits > 0 && !rest.is_empty()).then_some(rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped code editor, whose short name is not the name it shows.
    #[test]
    fn vscodiums_short_name_is_codium() {
        assert_eq!(short_name("com.vscodium.codium"), "codium");
        assert_eq!(the_last_part("com.vscodium.codium"), "codium");
    }

    /// A version the last part begins with is left off; the last part keeps it.
    #[test]
    fn a_version_is_left_off_the_short_name_and_kept_in_the_last_part() {
        assert_eq!(short_name("libreoffice26.2-writer"), "writer");
        assert_eq!(the_last_part("libreoffice26.2-writer"), "2-writer");
    }
}
