//! What an address points at, and whether that is this machine.
//!
//! One question, asked in one place, because three promises rest on it and all
//! three fail the same way if it is answered loosely.
//!
//! - [`crate::Provider::checked`] permits `http://` **only** to this machine.
//!   An address that is not this machine and is believed to be would put a key
//!   and somebody's question on the wire in clear.
//! - [`crate::Provider::source`] answers `InferenceSource::ThisMachine` for this
//!   machine, and everything downstream follows: no departure, nothing on the
//!   indicator, an answer that says *on this machine*, and
//!   `SourcePolicy::ThisMachineOnly` permitting it. An address that is not this
//!   machine and is believed to be is **law 1 failing silently** — a question
//!   leaves and the person is shown a quiet day.
//! - `alo_asking::Served` will not carry a question to an address this file
//!   does not vouch for, which is that same guarantee met one crate later.
//!
//! # The host is parsed, never matched by prefix
//!
//! This used to ask whether the address *started with* `127.0.0.1`, `localhost`
//! or `::1` after the scheme, and that is wrong in a way somebody could use on
//! purpose. `http://localhost.attacker.example` starts with `localhost`;
//! `http://127.0.0.1.attacker.example` starts with `127.0.0.1`; and
//! `http://127.0.0.1@attacker.example/` starts with `127.0.0.1` while the
//! connection goes to `attacker.example`, because everything before the last
//! `@` is a credential rather than a destination. All three were this machine as
//! far as this repository was concerned, so all three were reachable over
//! unencrypted http with a key attached and none of them would have appeared on
//! the indicator.
//!
//! So the authority is taken apart the way a URL is written — scheme, then
//! everything up to the first `/`, `?` or `#`, then whatever follows the last
//! `@`, then the host with its port removed — and the host is compared as a
//! whole. `localhost` is a name and is matched exactly; everything else has to
//! parse as an IP address, and `std::net::IpAddr::is_loopback` decides, so the
//! whole of `127.0.0.0/8` is this machine rather than only its first address.
//!
//! # Where it is deliberately not clever
//!
//! **An address that cannot be parsed is somewhere else**, which is the safe
//! direction in all three uses: an unparseable address is refused over http,
//! shown on the indicator if it is asked at all, and never carried by the local
//! door. `http://127.1`, which curl reaches, is one of these — it is not an IP
//! address by any parser in the standard library, so alo OS treats it as a name
//! and refuses it over http. A person writes the address in full.
//!
//! **An IPv4-mapped IPv6 address is somewhere else too.** `[::ffff:127.0.0.1]`
//! genuinely reaches loopback and `Ipv6Addr::is_loopback` says it does not; the
//! same safe direction applies, and nobody types it.
//!
//! **And loopback itself is still taken at face value.** A proxy listening on
//! `127.0.0.1` that forwards off the machine is this machine to every type in
//! this repository. That is `docs/quirks.md`'s entry and is caught at the
//! network boundary rather than here — what this file fixes is an address that
//! was never loopback at all.

use std::net::IpAddr;

/// Whether a connection to this endpoint reaches nothing but this machine.
///
/// `false` for anything that is not an `http://` or `https://` address, for an
/// address whose host cannot be read, and for every host that is not loopback.
pub(crate) fn is_on_this_machine(endpoint: &str) -> bool {
    authority_of(endpoint)
        .and_then(host_of)
        .is_some_and(is_this_machine)
}

/// The part of the address a connection is made out of: host, and port if there
/// is one.
///
/// [`None`] when this is not an address of a kind alo OS opens at all.
fn authority_of(endpoint: &str) -> Option<&str> {
    let after_scheme = endpoint
        .strip_prefix("https://")
        .or_else(|| endpoint.strip_prefix("http://"))?;
    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(after_scheme);
    // Everything before the last `@` is who is signing in, not where the
    // connection goes — and it is the half an address is made to look
    // trustworthy with.
    Some(match authority.rsplit_once('@') {
        Some((_credentials, destination)) => destination,
        None => authority,
    })
}

/// The host out of that, with any port taken off.
fn host_of(authority: &str) -> Option<&str> {
    if let Some(after_bracket) = authority.strip_prefix('[') {
        // An IPv6 literal is written in brackets, and a port comes after the
        // closing one.
        return after_bracket
            .split_once(']')
            .map(|(literal, _port)| literal);
    }
    if authority.matches(':').count() > 1 {
        // More than one colon and no brackets is an IPv6 literal somebody wrote
        // without them, which has nowhere in it for a port.
        return Some(authority);
    }
    Some(match authority.split_once(':') {
        Some((host, _port)) => host,
        None => authority,
    })
}

/// Whether that host is this machine.
fn is_this_machine(host: &str) -> bool {
    // A name written with the DNS root on the end is the same name.
    let host = host.strip_suffix('.').unwrap_or(host);
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    host.parse::<IpAddr>()
        .is_ok_and(|address| address.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The refusals this file exists for**, and every one of them was this
    /// machine before it: a name that begins with a loopback name, an address
    /// that begins with a loopback address, and a credential put in front of
    /// somewhere else so that the address reads as though it were local.
    #[test]
    fn an_address_that_only_begins_like_this_machine_is_somewhere_else() {
        for endpoint in [
            "http://localhost.attacker.example",
            "http://localhost.attacker.example:8080/v1",
            "http://127.0.0.1.attacker.example",
            "https://127.0.0.1.attacker.example/v1",
            "http://127.0.0.1@attacker.example/",
            "http://localhost@attacker.example:8080",
            "http://[::1]@attacker.example",
            "https://attacker.example/?to=localhost",
            "https://attacker.example/#127.0.0.1",
            "https://attacker.example/127.0.0.1",
            "http://localhostile.example",
            "http://notlocalhost",
        ] {
            assert!(!is_on_this_machine(endpoint), "{endpoint}");
        }
    }

    /// The ordinary local addresses, however they are written — and the whole
    /// of `127.0.0.0/8`, because a runtime bound to `127.0.0.2` is on this
    /// machine as surely as one bound to `127.0.0.1`.
    #[test]
    fn the_addresses_that_really_are_this_machine_are_this_machine() {
        for endpoint in [
            "http://127.0.0.1",
            "http://127.0.0.1:11434",
            "http://127.0.0.1:11434/v1",
            "http://127.0.0.2:8000",
            "http://127.9.9.9",
            "http://localhost",
            "http://localhost:1234/v1",
            "http://LocalHost:1234",
            "http://localhost./",
            "http://[::1]",
            "http://[::1]:8080/v1",
            "http://::1",
            "https://localhost:8443",
        ] {
            assert!(is_on_this_machine(endpoint), "{endpoint}");
        }
    }

    /// A provider anybody actually pays for is somewhere else, which is the
    /// case this question is asked about all day.
    #[test]
    fn a_provider_somewhere_else_is_somewhere_else() {
        for endpoint in [
            "https://api.mistral.ai",
            "https://api.mistral.ai/v1",
            "https://api.example.fr:8443/v1",
        ] {
            assert!(!is_on_this_machine(endpoint), "{endpoint}");
        }
    }

    /// **An address that cannot be read is somewhere else**, which is the safe
    /// direction in all three of this file's uses: refused over http, shown on
    /// the indicator, and never carried by the local door.
    #[test]
    fn an_address_that_cannot_be_read_is_treated_as_somewhere_else() {
        for endpoint in [
            "",
            "localhost:11434",
            "ftp://localhost",
            "file:///etc/hosts",
            "http://",
            "https://",
            // Reachable by curl, not an IP address to any parser in the
            // standard library, and therefore a name that is not `localhost`.
            "http://127.1",
            // Genuinely loopback, and `Ipv6Addr::is_loopback` says otherwise.
            "http://[::ffff:127.0.0.1]",
        ] {
            assert!(!is_on_this_machine(endpoint), "{endpoint}");
        }
    }

    /// The parts an address is taken apart into, asserted directly, so that a
    /// failure above says which of the three steps went wrong.
    #[test]
    fn an_address_is_taken_apart_the_way_it_was_written() {
        assert_eq!(
            authority_of("http://127.0.0.1:11434/v1"),
            Some("127.0.0.1:11434")
        );
        assert_eq!(authority_of("https://a.example?x=1"), Some("a.example"));
        assert_eq!(authority_of("https://a.example#f"), Some("a.example"));
        assert_eq!(
            authority_of("https://user:pw@a.example/x"),
            Some("a.example")
        );
        assert_eq!(authority_of("gopher://a.example"), None);

        assert_eq!(host_of("127.0.0.1:11434"), Some("127.0.0.1"));
        assert_eq!(host_of("[::1]:8080"), Some("::1"));
        assert_eq!(host_of("::1"), Some("::1"));
        assert_eq!(host_of("localhost"), Some("localhost"));
        assert_eq!(host_of(""), Some(""));

        assert!(is_this_machine("127.0.0.1"));
        assert!(!is_this_machine(""));
    }
}
