//! The makers whose own signed releases this machine installs from, and the one
//! key each of them signs with (ADR 0158).
//!
//! **A maker's key is pinned here, never fetched.** Every Craft release carries
//! `release-signing-key.asc` beside the files it signs, and a file checked
//! against a key from the same page proves only that whoever uploaded one
//! uploaded the other. So the key is a file in this crate
//! (`makers/<name>.asc`), read at build time, and its fingerprint is written
//! beside it: `crate::signature` refuses any verdict that names another.
//!
//! A maker is a place by name, like a place set up in the rented tool, so an
//! organisation's rule about which places may be used (ADR 0016) reaches it in
//! the same words.

use crate::source::SourceName;

/// One maker, its release host and the key it signs with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Maker {
    /// What the place is called, here and in `catalogue.toml`.
    name: &'static str,
    /// The only addresses its releases are fetched from begin with this.
    releases_under: &'static str,
    /// The primary key's fingerprint, forty capital hexadecimal digits.
    fingerprint: &'static str,
    /// The public key, as the maker published it.
    key: &'static str,
}

/// Every maker this machine installs signed releases from.
///
/// **storytold** publishes the Craft applications. Its key was read on
/// 2026-11-17 from the latest release of seven Craft repositories, which all
/// carried this one.
pub const EVERY_MAKER: [Maker; 1] = [Maker {
    name: "storytold",
    releases_under: "https://github.com/storytold/",
    fingerprint: "2B76741B414AAD6CD33E631213A005C0B9D7C3ED",
    key: include_str!("../makers/storytold.asc"),
}];

impl Maker {
    /// The maker with this name, or [`None`] for one this machine does not
    /// know.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        EVERY_MAKER.into_iter().find(|one| one.name == name)
    }

    /// What the place is called.
    ///
    /// [`None`] only for a maker whose name the rented tool could not be handed,
    /// which a test holds every entry of [`EVERY_MAKER`] against.
    #[must_use]
    pub fn source(&self) -> Option<SourceName> {
        SourceName::checked(self.name)
    }

    /// The name, as written.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The fingerprint a verdict must name.
    #[must_use]
    pub const fn fingerprint(&self) -> &'static str {
        self.fingerprint
    }

    /// Whether this address is one of this maker's release files.
    ///
    /// `https://github.com/<maker>/<repository>/releases/download/<tag>/<file>`
    /// and nothing else: no query, no fragment, no `..`, no other host.
    #[must_use]
    pub fn publishes(&self, address: &str) -> bool {
        let Some(rest) = address.strip_prefix(self.releases_under) else {
            return false;
        };
        let parts: Vec<&str> = rest.split('/').collect();
        let fits = |part: &str| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
        };
        matches!(parts.as_slice(), [repository, "releases", "download", tag, file]
            if fits(repository) && fits(tag) && fits(file))
    }

    /// The host its releases are fetched from.
    #[must_use]
    pub fn host(&self) -> &'static str {
        self.releases_under
            .strip_prefix("https://")
            .and_then(|rest| rest.split('/').next())
            .unwrap_or("")
    }

    /// The key, as the bytes a keyring holds — the armour taken off.
    ///
    /// [`None`] for armour that does not decode, which a test holds every
    /// entry of [`EVERY_MAKER`] against.
    #[must_use]
    pub fn keyring(&self) -> Option<Vec<u8>> {
        dearmoured(self.key)
    }
}

/// The bytes inside an ASCII-armoured block: the lines between the blank line
/// after its header and its checksum line, decoded.
fn dearmoured(armoured: &str) -> Option<Vec<u8>> {
    use base64::Engine as _;
    let mut lines = armoured.lines().map(str::trim);
    lines.find(|line| line.starts_with("-----BEGIN PGP PUBLIC KEY BLOCK-----"))?;
    // Header lines, if any, end at the first blank line.
    lines.find(|line| line.is_empty())?;
    let body: String = lines
        .take_while(|line| !line.starts_with("-----END"))
        .filter(|line| !line.starts_with('='))
        .collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body)
        .ok()?;
    (!bytes.is_empty()).then_some(bytes)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every maker is a place the rented tool could be handed, has a forty-digit
    /// fingerprint, and a key whose armour decodes to an OpenPGP public key —
    /// whose first byte is the public-key packet's tag.
    #[test]
    fn every_maker_is_a_place_with_a_key_that_decodes() {
        for maker in EVERY_MAKER {
            assert!(maker.source().is_some(), "{}", maker.name());
            assert_eq!(maker.fingerprint().len(), 40, "{}", maker.name());
            assert!(
                maker
                    .fingerprint()
                    .chars()
                    .all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c)),
                "{}",
                maker.name()
            );
            let keyring = maker.keyring().unwrap();
            // An old-format public-key packet (0x99) or a new-format one (0xC6).
            assert!(
                matches!(keyring.first(), Some(0x98 | 0x99 | 0xC6)),
                "{} decodes to {:x?}",
                maker.name(),
                keyring.first()
            );
            assert_eq!(maker.host(), "github.com");
        }
    }

    #[test]
    fn a_maker_is_found_by_its_name_and_no_other() {
        assert_eq!(Maker::named("storytold").unwrap().name(), "storytold");
        for no in ["", "Storytold", "flathub", "storytold "] {
            assert!(Maker::named(no).is_none(), "{no:?}");
        }
    }

    /// **Only the maker's own release files**, and every way an address could
    /// be pointed somewhere else.
    #[test]
    fn a_maker_publishes_only_its_own_release_files() {
        let maker = Maker::named("storytold").unwrap();
        assert!(maker.publishes(
            "https://github.com/storytold/photocraft/releases/download/v0.23.1/\
             photocraft-0.23.1-linux-x86_64.flatpak"
        ));
        for no in [
            "http://github.com/storytold/photocraft/releases/download/v0.23.1/a.flatpak",
            "https://github.com/someone-else/photocraft/releases/download/v0.23.1/a.flatpak",
            "https://github.com.example.org/storytold/photocraft/releases/download/v1/a.flatpak",
            "https://github.com/storytold/photocraft/archive/v0.23.1.tar.gz",
            "https://github.com/storytold/photocraft/releases/download/v0.23.1/a.flatpak?x=1",
            "https://github.com/storytold/photocraft/releases/download/v0.23.1/a.flatpak#x",
            "https://github.com/storytold/photocraft/releases/download/../../x/a.flatpak",
            "https://github.com/storytold/photocraft/releases/download/v0.23.1/",
            "https://github.com/storytold/photocraft/releases/download/v0.23.1/a/b.flatpak",
            "https://github.com/storytold/",
            "",
        ] {
            assert!(!maker.publishes(no), "{no:?} was taken for a release file");
        }
    }

    #[test]
    fn armour_that_does_not_decode_is_no_keyring() {
        for no in [
            "",
            "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\n!!!!\n-----END PGP PUBLIC KEY BLOCK-----",
            "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\n-----END PGP PUBLIC KEY BLOCK-----",
            "mDMEaulK1xYJKwYBBAHaRw8BAQdA",
        ] {
            assert!(dearmoured(no).is_none(), "{no:?}");
        }
    }
}
