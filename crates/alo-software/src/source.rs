//! A place applications come from: what this machine calls it, where it is,
//! and whether what it sends is checked.
//!
//! # The rented tool's configuration, read and never written
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md` is plain about it: *no
//! package manager of ours. Sources are the rented tool's configuration.* So a
//! [`Source`] is made from what that tool says a place is ([`Configured`]) and
//! from nothing else. Nothing in this crate adds a place, edits one, or turns a
//! check on or off; a place a person or an organisation set up is what there
//! is to install from.
//!
//! # Three facts, and each is a refusal waiting to happen
//!
//! - **Its name**, which is how a person and a verb choose it. Checked here,
//!   because it is handed to the rented tool as an argument and an argument that
//!   began with `-` would be read as an instruction to that tool instead.
//! - **Where it is on the network**, as an `alo_egress::Destination`, because
//!   installing from it leaves the machine and the indicator names the place.
//!   A place with no address this machine can read has no destination, and
//!   nothing is installed from it rather than something leaving unnamed.
//! - **Whether what it sends is checked against its signature.** The rented tool
//!   does the checking; what this crate does is refuse a place set up with the
//!   checking turned off, before anything is fetched from it.

use alo_egress::Destination;

/// What this machine calls a place applications come from.
///
/// Letters, digits, `.`, `_` and `-`, not beginning with `-`, at most
/// [`SourceName::LONGEST`] characters — the shape the rented tool gives a name,
/// and one that can never be read as an option by it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceName(String);

impl SourceName {
    /// The most characters a name may be.
    pub const LONGEST: usize = 64;

    /// The name, if it is one.
    ///
    /// [`None`] for anything that is not: empty, too long, holding a space, a
    /// separator or a control character, or beginning with `-`. There is no
    /// error type because there is nothing to tell the person beyond *that is
    /// not a place this machine installs from*, which is what every caller says
    /// ([`crate::NotDone::NotEnabled`]).
    #[must_use]
    pub fn checked(named: &str) -> Option<Self> {
        let named = named.trim();
        let shaped = !named.is_empty()
            && named.chars().count() <= Self::LONGEST
            && !named.starts_with('-')
            && named
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
        shaped.then(|| Self(named.to_owned()))
    }

    /// The name, as this machine knows it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A place, exactly as the rented tool describes it.
///
/// Plain data, filled by whatever reads that tool's configuration — the rented
/// door in [`crate::rented`] on a machine, or a test. Nothing in it is trusted
/// until [`Source::of`] has read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configured {
    /// The name the tool gives it.
    pub name: String,
    /// The address it was set up with.
    pub address: String,
    /// Whether what it sends is checked against its signature.
    pub checks_signatures: bool,
    /// Whether it was set up and then switched off.
    pub switched_off: bool,
}

/// A place applications come from, once this crate has read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// What it is called.
    name: SourceName,
    /// Where installing from it reaches, when it has an address this machine
    /// can read.
    destination: Option<Destination>,
    /// The host in that address, kept beside the destination because a road out
    /// is decided about a host and `alo_egress::Destination` does not give one
    /// back. Two readings of one address would be two answers to *where does
    /// this reach*, so it is read once, in [`Source::of`], and both are made
    /// from it.
    host: Option<String>,
    /// Whether what it sends is checked.
    checks_signatures: bool,
}

impl Source {
    /// A place as the rented tool described it, or [`None`] when it is not
    /// one anything could be installed from by name — switched off, or with a
    /// name the tool could never be handed safely.
    #[must_use]
    pub fn of(configured: &Configured) -> Option<Self> {
        if configured.switched_off {
            return None;
        }
        Some(Self {
            name: SourceName::checked(&configured.name)?,
            destination: host_of(&configured.address).and_then(|host| Destination::at(host).ok()),
            host: host_of(&configured.address).map(str::to_owned),
            checks_signatures: configured.checks_signatures,
        })
    }

    /// What it is called.
    #[must_use]
    pub fn name(&self) -> &SourceName {
        &self.name
    }

    /// Where installing from it reaches, or [`None`] when its address is not
    /// one this machine reads.
    #[must_use]
    pub fn destination(&self) -> Option<&Destination> {
        self.destination.as_ref()
    }

    /// The host installing from it reaches, which is what a road out is
    /// decided about (`crate::road`).
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// Whether what it sends is checked against its signature.
    #[must_use]
    pub const fn checks_signatures(&self) -> bool {
        self.checks_signatures
    }
}

/// The host in a network address, or [`None`] for anything else.
///
/// Only addresses that reach a host over the network are read: `https`,
/// `http`, and the rented tool's `oci+https` and `oci+http`. A folder on this
/// machine, or a scheme this crate does not know, is not a place anything
/// leaves for, and a source that cannot name where it leaves for is one
/// nothing is installed from (see the top of this file).
fn host_of(address: &str) -> Option<&str> {
    let (scheme, rest) = address.trim().split_once("://")?;
    if !matches!(
        scheme.to_ascii_lowercase().as_str(),
        "https" | "http" | "oci+https" | "oci+http"
    ) {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    let without_user = authority.rsplit('@').next()?;
    let host = if let Some(bracketed) = without_user.strip_prefix('[') {
        bracketed.split(']').next()?
    } else {
        without_user.split(':').next()?
    };
    (!host.is_empty()).then_some(host)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn configured(name: &str, address: &str) -> Configured {
        Configured {
            name: name.to_owned(),
            address: address.to_owned(),
            checks_signatures: true,
            switched_off: false,
        }
    }

    #[test]
    fn a_name_is_the_shape_the_rented_tool_gives_one() {
        for good in ["flathub", "acme-apps", "apps.example_2"] {
            assert_eq!(SourceName::checked(good).unwrap().as_str(), good);
        }
        assert_eq!(
            SourceName::checked("  flathub ").unwrap().as_str(),
            "flathub"
        );
    }

    /// **A name that could be read as an instruction to the rented tool is not
    /// a name.** `--no-deps` handed to it as a place would change what it does.
    #[test]
    fn a_name_that_could_be_read_as_an_option_or_a_path_is_refused() {
        for bad in [
            "",
            "   ",
            "--no-deps",
            "-v",
            "acme apps",
            "../elsewhere",
            "acme/apps",
            "acme\napps",
            "flathub;reboot",
            &"a".repeat(SourceName::LONGEST + 1),
        ] {
            assert!(SourceName::checked(bad).is_none(), "{bad:?}");
        }
    }

    /// **A place gives back the host a road out is decided about**, and it is
    /// the same host its destination was made from.
    ///
    /// The two are read from one address exactly once, so a place cannot come
    /// to be shown on the indicator as one host and reached through a proxy
    /// decided about another.
    #[test]
    fn a_place_gives_back_the_host_its_destination_was_made_from() {
        let source = Source::of(&Configured {
            name: "flathub".to_owned(),
            address: "https://dl.flathub.org/repo/".to_owned(),
            checks_signatures: true,
            switched_off: false,
        })
        .unwrap();
        assert_eq!(source.host(), Some("dl.flathub.org"));
        assert_eq!(
            source.destination(),
            Some(&Destination::at("dl.flathub.org").unwrap())
        );

        let nowhere = Source::of(&Configured {
            name: "local".to_owned(),
            address: "file:///srv/apps".to_owned(),
            checks_signatures: true,
            switched_off: false,
        })
        .unwrap();
        assert_eq!(nowhere.host(), None);
        assert_eq!(nowhere.destination(), None);
    }

    #[test]
    fn a_network_address_names_its_host() {
        for (address, host) in [
            ("https://dl.flathub.org/repo/", "dl.flathub.org"),
            ("http://apps.example.org:8080/repo", "apps.example.org"),
            ("oci+https://registry.example.org", "registry.example.org"),
            ("https://user@mirror.example.org/", "mirror.example.org"),
            ("https://[fd00::1]:443/repo", "fd00::1"),
        ] {
            assert_eq!(host_of(address), Some(host), "{address}");
        }
    }

    /// **A place nothing leaves for has no destination**, and so nothing is
    /// installed from it: a folder on this machine, a scheme nobody knows, or
    /// an address with no host.
    #[test]
    fn an_address_that_names_no_host_on_the_network_has_no_destination() {
        for address in [
            "file:///var/lib/apps",
            "ftp://apps.example.org/",
            "apps.example.org/repo",
            "https:///repo",
            "",
        ] {
            assert_eq!(host_of(address), None, "{address}");
            let source = Source::of(&configured("acme", address)).unwrap();
            assert!(source.destination().is_none(), "{address}");
        }
    }

    #[test]
    fn a_source_is_read_from_what_the_tool_says() {
        let source = Source::of(&configured("flathub", "https://dl.flathub.org/repo/")).unwrap();
        assert_eq!(source.name().as_str(), "flathub");
        assert_eq!(
            source.destination(),
            Some(&Destination::at("dl.flathub.org").unwrap())
        );
        assert!(source.checks_signatures());
    }

    /// **A place switched off is not a place**, and neither is one whose name
    /// could not be handed to the rented tool.
    #[test]
    fn a_switched_off_or_badly_named_place_is_not_a_source() {
        let mut off = configured("flathub", "https://dl.flathub.org/repo/");
        off.switched_off = true;
        assert!(Source::of(&off).is_none());
        assert!(Source::of(&configured("-x", "https://dl.flathub.org/repo/")).is_none());
    }
}
