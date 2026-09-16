//! A proxy, as an address a road out is sent through.
//!
//! One host, one port, how it is spoken to, and — when a company network asks
//! for one — a name and **where the password is kept**, never the password.
//! Everything else about a proxy is somebody else's software: alo OS runs no
//! proxy server, and this is the whole of what it knows about one.
//!
//! # A password written into the address is refused, not carried
//!
//! `http://anna:hunter2@proxy.example.com:8080` is how a proxy credential is
//! usually pasted, and taking it would put a password into the machine's
//! settings file in the same act. It is refused
//! ([`NotAnAddress::CarriesAPassword`]) and the person is told to type the name
//! and the password separately, which is what puts the password in the keyring
//! —  `alo_software::WebAddress` refuses the same shape for the same reason.
//!
//! # It is written out without the credential, and that matters twice
//!
//! [`ProxyAddress::written`] is the address alone. It is what a settings panel
//! shows, what the proxy portal answers an application with, and what a
//! refusal names — three places a credential must not reach, kept out by there
//! being no way to put one in. The one place a credential does go is
//! [`crate::Carried`], which takes the password as an argument rather than
//! finding it here.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::password::WhereThePasswordIs;
use crate::reaching::LONGEST;
use crate::words;

/// How a proxy is spoken to.
///
/// The two an ordinary company proxy offers. **No SOCKS**, deliberately: a
/// SOCKS proxy is a different thing to honour — it carries every protocol
/// rather than this machine's two — and adding a member here later is additive,
/// whereas shipping one this crate cannot actually hand to every road out would
/// be a setting that silently does nothing on some of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpokenTo {
    /// Reached in clear, which is what nearly every company proxy is.
    Http,
    /// Reached encrypted.
    Https,
}

impl SpokenTo {
    /// Both of them, in the order this file declares them.
    pub const EVERY: [Self; 2] = [Self::Http, Self::Https];

    /// How it is written at the front of an address.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

/// Why some text is not a proxy address.
///
/// **No `Display`**, so that the only road to words is
/// [`NotAnAddress::said`]: every one of these is somebody having just typed
/// something into a settings panel, and an English sentence one `to_string()`
/// away from that panel is a sentence whose author had no reason to think about
/// language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAnAddress {
    /// Nothing, or only blank space, where the proxy goes.
    Nowhere,
    /// Blank space or a control character inside it.
    NotOneLine,
    /// Longer than a host can be.
    TooLong,
    /// A port of zero, which names no service.
    NoPort,
    /// A name and a password written into the address.
    CarriesAPassword,
    /// A name given with nowhere for its password to be kept.
    NoPlaceForThePassword,
    /// Not the name of a host, or a number a host is written as.
    ///
    /// The checks above hold an address to being one line; this holds it to
    /// being a **host**, because the value goes on to configure a client that
    /// parses it, and a refusal that happened there would be a refusal in the
    /// middle of somebody's work rather than in the settings panel they typed
    /// it into.
    NotAHost,
}

impl NotAnAddress {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> words::Word {
        match self {
            Self::Nowhere => words::ADDRESS_NOWHERE,
            Self::NotOneLine => words::ADDRESS_NOT_ONE_LINE,
            Self::TooLong => words::ADDRESS_TOO_LONG,
            Self::NoPort => words::ADDRESS_NO_PORT,
            Self::CarriesAPassword => words::ADDRESS_CARRIES_A_PASSWORD,
            Self::NoPlaceForThePassword => words::ADDRESS_NO_PLACE_FOR_THE_PASSWORD,
            Self::NotAHost => words::ADDRESS_NOT_A_HOST,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// A proxy a road out is sent through.
///
/// Built only by [`ProxyAddress::checked`], so holding one is the proof that the
/// host is one line, the port names something, and no credential was written
/// into it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyAddress {
    /// How it is spoken to.
    spoken_to: SpokenTo,
    /// The proxy's own host.
    host: String,
    /// The port it answers on.
    port: u16,
    /// The name it wants, when it wants one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Where that name's password is kept. **Never the password.**
    #[serde(default, skip_serializing_if = "Option::is_none")]
    password: Option<WhereThePasswordIs>,
}

impl ProxyAddress {
    /// A proxy at this host and port, spoken to this way, with no credential.
    ///
    /// # Errors
    /// [`NotAnAddress`], naming the first thing that does not hold.
    pub fn checked(spoken_to: SpokenTo, host: &str, port: u16) -> Result<Self, NotAnAddress> {
        let host = host.trim();
        if host.is_empty() {
            return Err(NotAnAddress::Nowhere);
        }
        if host
            .chars()
            .any(|letter| letter.is_control() || letter.is_whitespace())
        {
            return Err(NotAnAddress::NotOneLine);
        }
        // A credential pasted along with the address. Refused here rather than
        // stripped, because stripping it would save a password somebody typed
        // into a file — the one outcome this whole file exists to prevent.
        if host.contains('@') {
            return Err(NotAnAddress::CarriesAPassword);
        }
        if host.chars().count() > LONGEST {
            return Err(NotAnAddress::TooLong);
        }
        if port == 0 {
            return Err(NotAnAddress::NoPort);
        }
        if !a_host(host) {
            return Err(NotAnAddress::NotAHost);
        }
        Ok(Self {
            spoken_to,
            host: host.to_owned(),
            port,
            name: None,
            password: None,
        })
    }

    /// The same proxy, with the name it wants and the keyring name its password
    /// is kept under.
    ///
    /// # Errors
    /// [`NotAnAddress::NotOneLine`] for a name that is not one,
    /// [`NotAnAddress::CarriesAPassword`] for a name with a password written
    /// into it, and [`NotAnAddress::NoPlaceForThePassword`] for a blank keyring
    /// name — a proxy that wants a name and has nowhere to find its password is
    /// a setting that would fail on the first road out, so it fails here.
    pub fn signing_in(
        mut self,
        name: &str,
        password: WhereThePasswordIs,
    ) -> Result<Self, NotAnAddress> {
        let name = name.trim();
        if name.is_empty() {
            return Err(NotAnAddress::NoPlaceForThePassword);
        }
        if name.chars().any(char::is_control) {
            return Err(NotAnAddress::NotOneLine);
        }
        if name.contains(':') || name.contains('@') {
            return Err(NotAnAddress::CarriesAPassword);
        }
        self.name = Some(name.to_owned());
        self.password = Some(password);
        Ok(self)
    }

    /// How it is spoken to.
    #[must_use]
    pub const fn spoken_to(&self) -> SpokenTo {
        self.spoken_to
    }

    /// The proxy's own host.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The port it answers on.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// The name it wants, where it wants one.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Where the password is kept, where there is one.
    #[must_use]
    pub const fn password(&self) -> Option<&WhereThePasswordIs> {
        self.password.as_ref()
    }

    /// The address, with **no credential in it**.
    ///
    /// What a settings panel shows, what an application is answered with, and
    /// what a person reads in a refusal. There is no argument that could put a
    /// password in.
    #[must_use]
    pub fn written(&self) -> String {
        format!("{}://{}:{}", self.spoken_to.written(), self.host, self.port)
    }
}

/// Whether this text is a host: a name written in labels, a number in the
/// four-part form, or a number in brackets.
///
/// Held here rather than left to whatever parses the address later, so that a
/// person typing one is refused in the settings panel they typed it into.
fn a_host(host: &str) -> bool {
    if let Some(inside) = host.strip_prefix('[') {
        return inside
            .strip_suffix(']')
            .is_some_and(|number| number.parse::<std::net::Ipv6Addr>().is_ok());
    }
    !host.starts_with('.')
        && !host.ends_with('.')
        && !host.contains("..")
        && host.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|letter| letter.is_ascii_alphanumeric() || letter == '-')
        })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    fn kept() -> WhereThePasswordIs {
        WhereThePasswordIs::named("the company proxy").unwrap()
    }

    /// **A password pasted into the address is refused, never taken.** This is
    /// the refusal that keeps a credential out of the machine's settings file.
    #[test]
    fn a_password_written_into_the_address_is_refused() {
        assert_eq!(
            ProxyAddress::checked(SpokenTo::Http, "anna:hunter2@proxy.example.com", 8080)
                .unwrap_err(),
            NotAnAddress::CarriesAPassword
        );
        assert_eq!(
            ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080)
                .unwrap()
                .signing_in("anna:hunter2", kept())
                .unwrap_err(),
            NotAnAddress::CarriesAPassword
        );
    }

    /// Every other way an address can fail to be one, each named.
    #[test]
    fn an_address_that_is_not_one_is_refused_in_its_own_words() {
        assert_eq!(
            ProxyAddress::checked(SpokenTo::Http, "  ", 8080).unwrap_err(),
            NotAnAddress::Nowhere
        );
        assert_eq!(
            ProxyAddress::checked(SpokenTo::Http, "proxy\nexample.com", 8080).unwrap_err(),
            NotAnAddress::NotOneLine
        );
        assert_eq!(
            ProxyAddress::checked(SpokenTo::Http, &"x".repeat(LONGEST + 1), 8080).unwrap_err(),
            NotAnAddress::TooLong
        );
        assert_eq!(
            ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 0).unwrap_err(),
            NotAnAddress::NoPort
        );
        assert_eq!(
            ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080)
                .unwrap()
                .signing_in("  ", kept())
                .unwrap_err(),
            NotAnAddress::NoPlaceForThePassword
        );
    }

    /// **The address as anybody reads it carries no credential**, whether or
    /// not the proxy wants one.
    #[test]
    fn the_address_anybody_reads_carries_no_credential() {
        let plain = ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap();
        assert_eq!(plain.written(), "http://proxy.example.com:8080");

        let signing_in = plain.clone().signing_in("anna", kept()).unwrap();
        assert_eq!(signing_in.written(), plain.written());
        assert_eq!(signing_in.name(), Some("anna"));
        assert_eq!(signing_in.password(), Some(&kept()));
        assert!(!signing_in.written().contains("anna"));
    }

    /// **Written down, it holds a name to look up and never a password** —
    /// the shape ADR 0022 asks of a settings structure.
    #[test]
    fn written_down_it_holds_where_the_password_is_and_never_one() {
        let address = ProxyAddress::checked(SpokenTo::Https, "proxy.example.com", 3128)
            .unwrap()
            .signing_in("anna", kept())
            .unwrap();
        let written = serde_json::to_string(&address).unwrap();
        assert!(written.contains("the company proxy"), "{written}");
        assert!(!written.contains("hunter2"), "{written}");
        assert_eq!(
            serde_json::from_str::<ProxyAddress>(&written).ok(),
            Some(address)
        );
    }

    /// A proxy that wants nothing writes nothing where a credential would be,
    /// so a settings file for the ordinary case has no empty credential in it.
    #[test]
    fn a_proxy_that_wants_no_credential_writes_none() {
        let written = serde_json::to_string(
            &ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap(),
        )
        .unwrap();
        assert!(!written.contains("name"), "{written}");
        assert!(!written.contains("password"), "{written}");
    }

    /// Each refusal says what to do, in the language the person reads, and
    /// none of them has a gap for what was typed.
    #[test]
    fn each_refusal_says_what_to_do_and_quotes_nothing() {
        let strings = in_english();
        for refusal in [
            NotAnAddress::Nowhere,
            NotAnAddress::NotOneLine,
            NotAnAddress::TooLong,
            NotAnAddress::NoPort,
            NotAnAddress::CarriesAPassword,
            NotAnAddress::NoPlaceForThePassword,
            NotAnAddress::NotAHost,
        ] {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{refusal:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refusal:?}: {said}");
        }
        assert!(
            NotAnAddress::CarriesAPassword
                .said(&strings)
                .text()
                .contains("password"),
        );
    }

    /// **A host that is not one is refused where it was typed**, rather than
    /// in the middle of somebody's work by whatever parses it later — and the
    /// forms that are hosts are accepted.
    #[test]
    fn something_that_is_not_a_host_is_refused_where_it_was_typed() {
        for host in [
            "proxy.example.com",
            "proxy",
            "PROXY.Example.COM",
            "10.0.0.7",
            "[2001:db8::1]",
            "a-b.example.com",
        ] {
            assert!(
                ProxyAddress::checked(SpokenTo::Http, host, 8080).is_ok(),
                "{host}"
            );
        }
        for host in [
            "[::1",
            "[not-a-number]",
            "proxy_example.com",
            ".example.com",
            "example.com.",
            "exam..ple.com",
            "-example.com",
            "example-.com",
            "proxy/../etc",
        ] {
            assert_eq!(
                ProxyAddress::checked(SpokenTo::Http, host, 8080).unwrap_err(),
                NotAnAddress::NotAHost,
                "{host}"
            );
        }
    }

    /// Both ways of speaking to a proxy are on the list a settings panel walks.
    #[test]
    fn both_ways_of_speaking_to_a_proxy_are_on_the_list() {
        assert_eq!(SpokenTo::EVERY.len(), 2);
        assert_eq!(SpokenTo::EVERY.map(SpokenTo::written), ["http", "https"]);
    }
}
