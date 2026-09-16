//! What one road out is given, once its way has been decided.
//!
//! `deciding.rs` answers *which way*; this is that answer in the shape the road
//! actually needs. Two shapes, because there are two kinds of road out of this
//! machine and no third:
//!
//! | | |
//! |---|---|
//! | a road a **program** takes | [`Carried::variables`] — what the rented tool and the base are started with |
//! | a road **this workspace's own client** takes | [`Carried::as_an_address`] — what a request is configured with |
//!
//! # One function turns a password into text, and it is named here
//!
//! [`Carried::as_an_address`] is that function. Everything else is built from
//! it, nothing in this crate has a `Display` or a `Serialize` that could reach
//! a credential, and [`Carried`]'s own [`Debug`](std::fmt::Debug) is written by
//! hand to say nothing. So *where could a proxy password appear* has one
//! answer, and `grep` finds every caller of it.
//!
//! `alo_models::Secret` keeps its `bearer` private and hands the request to the
//! key instead. That is the better shape and it is not available here: the
//! clients are in other crates and the programs are processes, so the value has
//! to leave. What is kept instead is that it leaves through **one door**, with
//! the sentence above beside it.
//!
//! # A road that goes straight out says so, rather than saying nothing
//!
//! [`Carried::variables`] names every variable either way, and a road going
//! straight out gives each of them **empty**, which is how *no proxy* is said to
//! a program. Saying nothing would be leaving whatever the process that started
//! this one happened to be carrying, and that is not a setting anybody chose:
//! a machine whose proxy a person turned off would go on using one because a
//! service file still had a line in it. `alo-secrets` refuses to read an
//! environment for the same reason.

use std::fmt;

use crate::address::{NotAnAddress, ProxyAddress};
use crate::password::Password;
use crate::road::Way;

/// The places a child process never proxies, whatever it was given.
///
/// Belt and braces beside `crate::Reaching::is_this_machine`: this crate's own
/// decision already sends loopback straight out, and a program given a proxy
/// makes connections this crate never decided about — to a socket of its own,
/// to a service on this machine. A model answering here (ADR 0007) is on this
/// list twice rather than not at all.
pub const NEVER_THROUGH_A_PROXY: &str = "localhost,127.0.0.1,::1";

/// One road out, with its way decided and whatever it needs to take it.
///
/// Deliberately not `Clone`, not `Serialize` and not `PartialEq`: it can hold a
/// credential, and a credential that can be copied, written down or compared is
/// a credential in somewhere nobody meant it to be.
pub struct Carried {
    /// Which way this road goes.
    way: Way,
    /// The password the proxy wants, where it wants one.
    password: Option<Password>,
}

impl Carried {
    /// This road, going the way that was decided, with no credential.
    #[must_use]
    pub const fn of(way: Way) -> Self {
        Self {
            way,
            password: None,
        }
    }

    /// This road, going straight out.
    #[must_use]
    pub const fn straight() -> Self {
        Self::of(Way::Straight)
    }

    /// This road, through this proxy.
    #[must_use]
    pub const fn through(proxy: ProxyAddress) -> Self {
        Self::of(Way::Through(proxy))
    }

    /// The same road, with the password the keyring answered with.
    ///
    /// The password is **moved in** and cannot be taken back out: the only
    /// thing that reads it is [`Carried::as_an_address`], in this file.
    #[must_use]
    pub fn with_the_password(mut self, password: Password) -> Self {
        self.password = Some(password);
        self
    }

    /// Which way this road goes.
    #[must_use]
    pub const fn way(&self) -> &Way {
        &self.way
    }

    /// Whether this road goes straight out.
    #[must_use]
    pub const fn is_straight(&self) -> bool {
        self.way.is_straight()
    }

    /// The proxy this road goes through, **with the credential when there is
    /// one** — and [`None`] for a road going straight out.
    ///
    /// **This is the one function in this crate that turns a password into
    /// text.** What it hands back goes straight onto a client or a child
    /// process configured for this one road; it is never written down, never
    /// shown to a person, and never put in a record. What a person reads is
    /// [`Carried::shown`], which cannot carry one.
    #[must_use]
    pub fn as_an_address(&self) -> Option<String> {
        let proxy = self.way.through()?;
        Some(match (proxy.name(), self.password.as_ref()) {
            (Some(name), Some(password)) => format!(
                "{}://{}:{}@{}:{}",
                proxy.spoken_to().written(),
                encoded(name),
                encoded(password.as_str()),
                proxy.host(),
                proxy.port(),
            ),
            _ => proxy.written(),
        })
    }

    /// The proxy this road goes through, as a person reads it: **no
    /// credential**, ever.
    ///
    /// What a settings panel shows and what a support answer names. There is no
    /// argument that could put a password in.
    #[must_use]
    pub fn shown(&self) -> Option<String> {
        self.way.through().map(ProxyAddress::written)
    }

    /// This road, as a request this workspace makes is configured with it.
    ///
    /// [`None`] for a road going straight out — and a caller **passes that
    /// `None` on**, rather than leaving the client to decide. A client left to
    /// decide reads the environment of whatever process it is in, which is not
    /// a setting a person chose and cannot be shown to them; `alo-secrets`
    /// refuses to read an environment for the same reason.
    ///
    /// # Errors
    /// [`NotAnAddress::NotAHost`] if the client cannot use the address. It
    /// cannot happen to an address [`ProxyAddress::checked`] made — that is
    /// what the host check there is for — and it is a refusal rather than a
    /// silent straight-out road, because a client with no proxy where there
    /// should be one sends a company's traffic around its own rule.
    pub fn for_a_request(&self) -> Result<Option<ureq::Proxy>, NotAnAddress> {
        match self.as_an_address() {
            None => Ok(None),
            Some(address) => ureq::Proxy::new(&address)
                .map(Some)
                .map_err(|_| NotAnAddress::NotAHost),
        }
    }

    /// What a program taking this road is started with.
    ///
    /// The names every program on a machine like this already reads, in both
    /// cases, because the programs disagree about which they read and a
    /// machine-wide proxy that reached half of them would be worse than none.
    ///
    /// A road going straight out gives each of them **empty**, which is how
    /// *no proxy* is said — see this file's last section.
    #[must_use]
    pub fn variables(&self) -> Vec<(&'static str, String)> {
        let address = self.as_an_address().unwrap_or_default();
        let mut given = vec![
            ("http_proxy", address.clone()),
            ("HTTP_PROXY", address.clone()),
            ("https_proxy", address.clone()),
            ("HTTPS_PROXY", address),
        ];
        given.push(("no_proxy", NEVER_THROUGH_A_PROXY.to_owned()));
        given.push(("NO_PROXY", NEVER_THROUGH_A_PROXY.to_owned()));
        given
    }
}

/// Says nothing, on purpose — the reason `alo_models::Secret` writes its own.
impl fmt::Debug for Carried {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Carried")
            .field("way", &self.shown())
            .field("password", &self.password.is_some())
            .finish()
    }
}

/// A name or a password, written so that it is one piece of an address.
///
/// The characters that would otherwise end the credential early and make the
/// rest of it look like a host: `:`, `@`, `/`, `?`, `#`, `%`, and anything
/// outside the unreserved set. A credential is refused before it gets here if
/// it is not one line (`password.rs`), so this is about structure rather than
/// about safety — but an address that silently means something other than what
/// somebody typed is how a machine reaches a proxy nobody chose.
fn encoded(text: &str) -> String {
    let mut written = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            written.push(char::from(byte));
        } else {
            written.push_str(&format!("%{byte:02X}"));
        }
    }
    written
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::address::SpokenTo;
    use crate::password::WhereThePasswordIs;

    fn an_address() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
    }

    fn signing_in() -> ProxyAddress {
        an_address()
            .signing_in(
                "anna",
                WhereThePasswordIs::named("the company proxy").unwrap(),
            )
            .unwrap()
    }

    /// A road through a proxy that wants nothing is the address, and a program
    /// taking it is started with it under every name programs read.
    #[test]
    fn a_road_through_a_proxy_is_given_to_a_program_under_every_name_it_reads() {
        let carried = Carried::through(an_address());
        assert_eq!(
            carried.as_an_address(),
            Some("http://proxy.example.com:8080".to_owned())
        );
        let variables = carried.variables();
        for name in [
            "http_proxy",
            "HTTP_PROXY",
            "https_proxy",
            "HTTPS_PROXY",
            "no_proxy",
            "NO_PROXY",
        ] {
            assert!(
                variables.iter().any(|(given, _)| *given == name),
                "{name} was not given"
            );
        }
    }

    /// **A road going straight out says so**, rather than leaving whatever the
    /// process that started this one was carrying.
    #[test]
    fn a_road_going_straight_out_says_no_proxy_rather_than_saying_nothing() {
        let carried = Carried::straight();
        assert!(carried.is_straight());
        assert_eq!(carried.as_an_address(), None);
        assert_eq!(carried.shown(), None);
        assert_eq!(
            carried.variables(),
            vec![
                ("http_proxy", String::new()),
                ("HTTP_PROXY", String::new()),
                ("https_proxy", String::new()),
                ("HTTPS_PROXY", String::new()),
                ("no_proxy", NEVER_THROUGH_A_PROXY.to_owned()),
                ("NO_PROXY", NEVER_THROUGH_A_PROXY.to_owned()),
            ]
        );
    }

    /// **This machine is never proxied by a program either**, whatever it was
    /// started with.
    #[test]
    fn a_program_is_told_never_to_proxy_this_machine() {
        for carried in [Carried::straight(), Carried::through(an_address())] {
            let never = carried
                .variables()
                .into_iter()
                .find(|(name, _)| *name == "no_proxy")
                .map(|(_, value)| value);
            assert_eq!(never, Some(NEVER_THROUGH_A_PROXY.to_owned()));
            assert!(NEVER_THROUGH_A_PROXY.contains("127.0.0.1"));
            assert!(NEVER_THROUGH_A_PROXY.contains("localhost"));
            assert!(NEVER_THROUGH_A_PROXY.contains("::1"));
        }
    }

    /// The credential travels on the road that needs it, and is written so that
    /// nothing in it can be read as part of the address.
    #[test]
    fn the_credential_travels_on_the_road_and_is_written_as_one_piece() {
        let carried =
            Carried::through(signing_in()).with_the_password(Password::typed("hun:ter@2").unwrap());
        assert_eq!(
            carried.as_an_address(),
            Some("http://anna:hun%3Ater%402@proxy.example.com:8080".to_owned())
        );
    }

    /// **What a person reads never carries the credential**, and neither does
    /// anything formatted.
    #[test]
    fn nothing_a_person_reads_and_nothing_formatted_carries_the_credential() {
        let carried =
            Carried::through(signing_in()).with_the_password(Password::typed("hunter2").unwrap());
        assert_eq!(
            carried.shown(),
            Some("http://proxy.example.com:8080".to_owned())
        );
        assert!(!carried.shown().unwrap_or_default().contains("hunter2"));

        let formatted = format!("{carried:?}");
        assert!(!formatted.contains("hunter2"), "{formatted}");
        assert!(!formatted.contains("anna"), "{formatted}");
        assert!(formatted.contains("proxy.example.com"), "{formatted}");
    }

    /// A proxy that wants a name but was given no password is the address
    /// alone: a half-credential in an address would reach the proxy as somebody
    /// with no password, and the proxy would say so.
    #[test]
    fn a_name_with_no_password_is_not_written_into_the_address() {
        assert_eq!(
            Carried::through(signing_in()).as_an_address(),
            Some("http://proxy.example.com:8080".to_owned())
        );
    }

    /// **A request is configured with this road**, and a road going straight
    /// out configures a client with nothing — which is what stops the client
    /// reading a process environment nobody chose.
    #[test]
    fn a_request_is_configured_with_this_road_and_a_straight_one_with_nothing() {
        assert!(
            Carried::through(an_address())
                .for_a_request()
                .unwrap()
                .is_some()
        );
        assert!(Carried::straight().for_a_request().unwrap().is_none());
        assert!(
            Carried::through(signing_in())
                .with_the_password(Password::typed("hunter2").unwrap())
                .for_a_request()
                .unwrap()
                .is_some(),
            "a credential travels on the road that needs it"
        );
    }

    /// The way is readable back, because the caller that made it is not always
    /// the caller that takes it.
    #[test]
    fn the_way_this_road_goes_is_readable_back() {
        assert_eq!(Carried::straight().way(), &Way::Straight);
        assert_eq!(
            Carried::through(an_address()).way(),
            &Way::Through(an_address())
        );
    }
}
