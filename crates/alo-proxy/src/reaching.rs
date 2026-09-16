//! Where a road out is going, as a proxy decision needs it.
//!
//! A proxy decision needs two things about a destination and no more: **which
//! scheme** the road is spoken over, because a manual setting names an address
//! per scheme, and **which host** it is going to, because that is what an
//! exception list is written about. Nothing else about a destination — a path,
//! a query, a body — reaches a decision here, and there is nowhere for it to
//! arrive.
//!
//! **It is checked before it can become one.** A host reaches this from a
//! provider a person typed, from a place applications come from, and from the
//! address updates are fetched at; it then becomes an argument to a program
//! (`evaluator.rs`) and a line a person reads. Text carrying a newline or an
//! escape code would be two arguments wearing one host, which is
//! `alo_egress::Destination`'s refusal and the same one.
//!
//! **This is not `alo_egress::Destination`.** That type answers *where did that
//! go* for a person and a record, in the three kinds a person is told apart. A
//! decision about a road out is a narrower question asked earlier, and this
//! crate holds no dependency on that one — which is what keeps the indicator's
//! answer and the proxy's answer from being one value read two ways. The rule
//! between them is `road.rs`'s: the indicator names **this** host, never the
//! proxy's.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

/// The most characters a host may be.
///
/// A hostname cannot be longer than this and still be a hostname, so the bound
/// costs nothing real. The same number `alo_egress::Destination` holds itself
/// to, for the same reason.
pub const LONGEST: usize = 253;

/// How a road out is spoken.
///
/// Two, and they are the two `alo_software::WebAddress` already decided are the
/// open web. A proxy setting names an address per scheme because a company
/// network routinely sends the two different ways, and because that is the
/// shape every application on the machine already reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scheme {
    /// Spoken in clear.
    Http,
    /// Spoken encrypted.
    Https,
}

impl Scheme {
    /// Both of them, in the order this file declares them.
    pub const EVERY: [Self; 2] = [Self::Http, Self::Https];

    /// How the scheme is written in an address.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

/// Why some text is not a host a road can be going to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotReachable {
    /// Nothing, or only blank space.
    #[error("no host was given")]
    Nowhere,
    /// Blank space or a control character inside it: two lines wearing one host.
    #[error("that host is not one line")]
    NotOneLine,
    /// Longer than a host can be.
    #[error("that host is longer than {LONGEST} characters")]
    TooLong,
    /// An address that is not spoken over `http` or `https`.
    ///
    /// The two schemes this machine has a proxy answer for. An application
    /// asking about anything else is told so rather than being handed the
    /// answer for a scheme it did not ask about.
    #[error("a road out of this machine is spoken over http or https, and that address is not")]
    NotAWayOut,
}

/// Where a road out is going.
///
/// Built only by [`Reaching::over`], so holding one is the proof that the host
/// in it is one line and can be put on a program's argument list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaching {
    /// How the road is spoken.
    over: Scheme,
    /// The host it is going to.
    host: String,
}

impl Reaching {
    /// A road out, over this scheme, to this host.
    ///
    /// # Errors
    /// [`NotReachable`], naming the first thing that does not hold.
    pub fn over(over: Scheme, host: &str) -> Result<Self, NotReachable> {
        let host = host.trim();
        if host.is_empty() {
            return Err(NotReachable::Nowhere);
        }
        if host
            .chars()
            .any(|letter| letter.is_control() || letter.is_whitespace())
        {
            return Err(NotReachable::NotOneLine);
        }
        if host.chars().count() > LONGEST {
            return Err(NotReachable::TooLong);
        }
        Ok(Self {
            over,
            host: host.to_owned(),
        })
    }

    /// A road out, read out of an address.
    ///
    /// What an application hands the proxy portal is an address rather than a
    /// scheme and a host, so this is the portal's own boundary check. It keeps
    /// the scheme and the host and **throws the rest away**: the path and the
    /// query are the application's business, and a proxy decision that held on
    /// to them would be this machine keeping a list of what its applications
    /// are opening.
    ///
    /// # Errors
    /// [`NotReachable`], naming the first thing that does not hold.
    pub fn of(address: &str) -> Result<Self, NotReachable> {
        let address = address.trim();
        if address.is_empty() {
            return Err(NotReachable::Nowhere);
        }
        if address
            .chars()
            .any(|letter| letter.is_control() || letter.is_whitespace())
        {
            return Err(NotReachable::NotOneLine);
        }
        let Some((scheme, rest)) = address.split_once("://") else {
            return Err(NotReachable::NotAWayOut);
        };
        let over = if scheme.eq_ignore_ascii_case("https") {
            Scheme::Https
        } else if scheme.eq_ignore_ascii_case("http") {
            Scheme::Http
        } else {
            return Err(NotReachable::NotAWayOut);
        };
        let authority = match rest.find(['/', '?', '#']) {
            Some(at) => rest.get(..at).unwrap_or_default(),
            None => rest,
        };
        if authority.contains('@') {
            return Err(NotReachable::NotAWayOut);
        }
        let host = match authority.strip_prefix('[') {
            Some(inside) => inside.split_once(']').map_or(authority, |(host, _)| host),
            None => authority.split(':').next().unwrap_or(authority),
        };
        Self::over(over, host)
    }

    /// How the road is spoken.
    #[must_use]
    pub const fn scheme(&self) -> Scheme {
        self.over
    }

    /// The host it is going to — **the real destination**, which is what an
    /// indicator line names.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Whether this host is this machine.
    ///
    /// Loopback never goes through a proxy, whatever the setting says, and that
    /// is decided here rather than left to an exception somebody remembers to
    /// write: a model answering on this machine (ADR 0007) is reached on
    /// loopback, and a machine whose proxy setting silently broke its own
    /// runtime would be a machine whose whole product stopped working the day
    /// somebody typed a company address into a settings panel.
    ///
    /// The name, and an address the kernel itself calls loopback — the whole of
    /// `127.0.0.0/8` and `::1`. Nothing clever beyond that: a host that merely
    /// *resolves* to loopback is not this, because what is being decided is
    /// what to write on a road out, not what the kernel will do with it.
    #[must_use]
    pub fn is_this_machine(&self) -> bool {
        let host = self.host.trim_start_matches('[').trim_end_matches(']');
        host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    }

    /// This road written as an address, which is what an automatic
    /// configuration is asked about.
    ///
    /// The scheme and the host, and nothing after them. What a script is given
    /// is deliberately not the whole address a road is taking: a path and a
    /// query are somebody's document, somebody's search or somebody's
    /// identifier, and handing them to a company's script would make the proxy
    /// setting the most complete record of a person's day on the machine.
    #[must_use]
    pub fn as_an_address(&self) -> String {
        format!("{}://{}/", self.over.written(), self.host)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A host is one line, or it is not a host — the refusal an argument list
    /// and an indicator line both depend on.
    #[test]
    fn a_host_that_is_not_one_line_never_becomes_a_road() {
        for attempt in [
            "example.com\nand nothing is leaving",
            "\u{1b}[2Kexample.com",
            "example .com",
            "example.com\u{7}",
        ] {
            assert_eq!(
                Reaching::over(Scheme::Https, attempt).unwrap_err(),
                NotReachable::NotOneLine,
                "{attempt:?}"
            );
        }
        assert_eq!(
            Reaching::over(Scheme::Https, "   ").unwrap_err(),
            NotReachable::Nowhere
        );
        assert_eq!(
            Reaching::over(Scheme::Https, &"x".repeat(LONGEST + 1)).unwrap_err(),
            NotReachable::TooLong
        );
        assert!(Reaching::over(Scheme::Https, &"x".repeat(LONGEST)).is_ok());
    }

    /// **This machine is never reached through a proxy**, whichever way it is
    /// written — and a host that merely looks like it is not this machine.
    #[test]
    fn this_machine_is_recognised_however_it_is_written_and_nothing_else_is() {
        for here in [
            "localhost",
            "LOCALHOST",
            "127.0.0.1",
            "127.13.9.2",
            "::1",
            "[::1]",
        ] {
            assert!(
                Reaching::over(Scheme::Http, here)
                    .unwrap()
                    .is_this_machine(),
                "{here}"
            );
        }
        for elsewhere in [
            "localhost.example.com",
            "127.example.com",
            "12.7.0.1",
            "example.com",
        ] {
            assert!(
                !Reaching::over(Scheme::Http, elsewhere)
                    .unwrap()
                    .is_this_machine(),
                "{elsewhere}"
            );
        }
    }

    /// **A script is told where a road is going and not what is on it.** The
    /// address carries the scheme and the host, and there is nowhere in the
    /// type for a path to have arrived from.
    #[test]
    fn the_address_a_script_is_asked_about_carries_no_path() {
        let road = Reaching::over(Scheme::Https, "files.example.com").unwrap();
        assert_eq!(road.as_an_address(), "https://files.example.com/");
        assert_eq!(road.host(), "files.example.com");
        assert_eq!(
            Reaching::over(Scheme::Http, "files.example.com")
                .unwrap()
                .as_an_address(),
            "http://files.example.com/"
        );
    }

    /// **An address an application handed over becomes a road, or is refused.**
    /// What is kept is the scheme and the host; the path and the query are
    /// thrown away rather than held.
    #[test]
    fn an_address_becomes_a_road_out_and_keeps_nothing_of_what_was_on_it() {
        let road =
            Reaching::of("https://files.example.com:443/what/anna/opened?q=her+name").unwrap();
        assert_eq!(road.scheme(), Scheme::Https);
        assert_eq!(road.host(), "files.example.com");
        assert_eq!(road.as_an_address(), "https://files.example.com/");

        assert_eq!(Reaching::of("http://[::1]:8080/x").unwrap().host(), "::1");
        assert_eq!(
            Reaching::of("HTTP://Example.com").unwrap().host(),
            "Example.com"
        );
    }

    /// An address that is not a road out of this machine is refused rather than
    /// answered about, including one carrying a credential.
    #[test]
    fn an_address_that_is_not_a_road_out_is_refused() {
        for attempt in [
            "file:///etc/passwd",
            "ftp://files.example.com/",
            "files.example.com",
            "https://anna:hunter2@files.example.com/",
        ] {
            assert_eq!(
                Reaching::of(attempt).unwrap_err(),
                NotReachable::NotAWayOut,
                "{attempt:?}"
            );
        }
        assert_eq!(Reaching::of("   ").unwrap_err(), NotReachable::Nowhere);
        assert_eq!(
            Reaching::of("https://files.example.com/\u{1b}[2K").unwrap_err(),
            NotReachable::NotOneLine
        );
        assert_eq!(
            Reaching::of("https:///path").unwrap_err(),
            NotReachable::Nowhere
        );
    }

    /// Both schemes are on the list, which is what a settings panel walks and
    /// what the road tests walk.
    #[test]
    fn both_schemes_are_on_the_list() {
        assert_eq!(Scheme::EVERY.len(), 2);
        assert_eq!(Scheme::EVERY.map(Scheme::written), ["http", "https"]);
    }
}
