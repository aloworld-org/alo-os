//! What an address points at, and whether that is this machine.
//!
//! One question, asked in one place, because three promises rest on it and all
//! three fail the same way if it is answered loosely.
//!
//! - [`crate::Provider::checked`] permits `http://` to this machine, and — since
//!   the owner's decision of 2026-10-30 — to an address on the person's own
//!   network ([`is_on_a_local_network`]), which Settings cautions about first.
//!   An address on the internet that is believed to be either would put a key
//!   and somebody's question on the internet in clear.
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
///
/// **Public so that a form asks the daemon's own question** rather than a second
/// reading of an address that could disagree with this one. Settings warns a
/// person adding a provider at this machine's address that alo cannot see where
/// that service sends their questions, and the warning has to appear for exactly
/// the addresses whose answers will say *a service at this machine's address*.
#[must_use]
pub fn is_on_this_machine(endpoint: &str) -> bool {
    authority_of(endpoint)
        .and_then(host_of)
        .is_some_and(is_this_machine)
}

/// Whether this endpoint's host is an address on the person's own network.
///
/// **The owner's decision of 2026-10-30** (ADR 0021): a person may add a
/// provider at another computer on their own network, such as an AI service
/// at `http://192.168.1.20:11434`, over plain `http://`, after a caution that
/// the question and the key cross that network unencrypted.
/// [`crate::Provider::checked`] accepts such an address and this is the whole
/// of what it means by one.
///
/// **Only an address written as numbers, and only these ranges**, which are
/// the ones no router carries onto the internet:
///
/// - IPv4 `10.0.0.0/8`, `172.16.0.0/12` and `192.168.0.0/16`, the private
///   ranges a home or office router hands out;
/// - IPv4 `169.254.0.0/16`, what two computers joined by one cable give
///   themselves;
/// - IPv6 `fc00::/7` (unique local) and `fe80::/10` (link-local).
///
/// **A name is never one**, `nas.local` included: nothing here resolves a
/// name, and a name that resolves to a home address today can resolve to the
/// internet tomorrow. Neither is the shared range `100.64.0.0/10`, which a
/// provider's network gives many customers at once, nor an IPv4 address
/// written inside IPv6 — the safe direction, as for loopback above.
///
/// This is **not** this machine: a question to such an address is shown on
/// the indicator and kept in the record as leaving, like any other provider's.
#[must_use]
pub fn is_on_a_local_network(endpoint: &str) -> bool {
    authority_of(endpoint)
        .and_then(host_of)
        .and_then(|host| host.parse::<IpAddr>().ok())
        .is_some_and(|address| match address {
            IpAddr::V4(four) => four.is_private() || four.is_link_local(),
            IpAddr::V6(six) => six.is_unique_local() || six.is_unicast_link_local(),
        })
}

/// Whether a question to this endpoint would cross the person's own network
/// unencrypted: plain `http://` to [an address on it](is_on_a_local_network)
/// that is not this machine's loopback.
///
/// **What Settings cautions about before a person saves** such a provider
/// (owner's decision, 2026-10-30; ADR 0021). Asked here, beside the rule
/// [`crate::Provider::checked`] applies, so the caution appears for exactly the
/// plain `http://` addresses that rule accepts and for no address it refuses.
#[must_use]
pub fn crosses_a_local_network_unencrypted(endpoint: &str) -> bool {
    endpoint.starts_with("http://")
        && !is_on_this_machine(endpoint)
        && is_on_a_local_network(endpoint)
}

/// Whether this endpoint's host is written as one of `addresses`.
///
/// **For a caution, never for a promise.** The owner decided on 2026-10-30
/// that Settings warns a person adding a provider at one of this machine's own
/// network addresses — `192.168.1.20`, say — as it does for loopback (ADR
/// 0021). Such an address is not *this machine* to [`is_on_this_machine`],
/// and stays so: the daemon still treats it as somewhere else and shows it on
/// the indicator, because an address a machine had when the provider was saved
/// is not one it is sure to have when the question is asked. (Plain `http://`
/// to it is accepted since the owner's later decision the same day, as
/// [`is_on_a_local_network`] says, with the caution that goes with that.) What this answers is only whether the
/// person typed an address the caller has just read off this machine.
///
/// Taken apart by the same rules as every other question here, so
/// `http://192.168.1.20@attacker.example` is `attacker.example` and is not
/// one of them. Only an address written as an IP literal can match: a name is
/// not resolved here, which is this file's rule.
#[must_use]
pub fn its_host_is_one_of(endpoint: &str, addresses: &[IpAddr]) -> bool {
    authority_of(endpoint)
        .and_then(host_of)
        .and_then(|host| host.parse::<IpAddr>().ok())
        .is_some_and(|host| addresses.contains(&host))
}

/// The host and port a request to this endpoint would connect to.
///
/// **Where a name is turned into something to resolve**, and it is here because
/// this is the file that already knows how an authority is written and has the
/// tests for the ways one can be made to look like something it is not. ADR
/// 0020 separates resolving from connecting; this is the first half of that,
/// and it is deliberately not a resolver — it answers *what to look up*, and
/// nothing here touches a name server.
///
/// The port is the one written after the host, or the scheme's own: 443 for
/// `https://` and 80 for `http://`.
///
/// [`None`] for anything that is not one of those two schemes, or that has no
/// host, which is the same answer this file's own loopback question gives such
/// an address.
#[must_use]
pub fn where_it_connects(endpoint: &str) -> Option<(String, u16)> {
    let authority = authority_of(endpoint)?;
    let host = host_of(authority)?;
    let port = match authority.rsplit_once(':') {
        // A colon inside brackets is part of an IPv6 address rather than a
        // port, and `host_of` is what knows the difference.
        Some((_, port)) if !port.contains(']') => port.parse().ok()?,
        _ if endpoint.starts_with("https://") => 443,
        _ => 80,
    };
    Some((host.to_owned(), port))
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

    /// **An address written as one of the machine's own is one of them**, in
    /// either family, with or without a port or a path.
    #[test]
    fn an_address_written_as_one_of_these_is_one_of_them() {
        let addresses: [IpAddr; 2] = [
            IpAddr::from([192, 168, 1, 20]),
            IpAddr::from([0xfd00, 0, 0, 0, 0, 0, 0, 0x20]),
        ];
        for endpoint in [
            "http://192.168.1.20:8000/v1",
            "https://192.168.1.20",
            "http://[fd00::20]:8080",
        ] {
            assert!(its_host_is_one_of(endpoint, &addresses), "{endpoint}");
        }
    }

    /// **And everything else is not**: another address, a name, a lookalike,
    /// and a credential put in front of somewhere else.
    #[test]
    fn an_address_not_written_as_one_of_these_is_not() {
        let addresses = [IpAddr::from([192, 168, 1, 20])];
        for endpoint in [
            "http://192.168.1.21:8000",
            "http://192.168.1.20.attacker.example",
            "http://192.168.1.20@attacker.example/",
            "http://my-machine.local:8000",
            "ftp://192.168.1.20",
            "",
        ] {
            assert!(!its_host_is_one_of(endpoint, &addresses), "{endpoint}");
        }
        assert!(!its_host_is_one_of("http://192.168.1.20", &[]));
    }

    /// **An address on the person's own network**, in every range the owner's
    /// decision of 2026-10-30 covers, with and without a port or a path.
    #[test]
    fn an_address_on_a_local_network_is_one() {
        for endpoint in [
            "http://192.168.1.20:11434",
            "http://192.168.0.1",
            "http://10.0.0.5/v1",
            "http://10.255.255.255",
            "http://172.16.0.1:8080/v1",
            "http://172.31.255.254",
            "http://169.254.10.20:8000",
            "http://[fd00::1]:8080",
            "http://[fc00::1]",
            "http://[fe80::1]:8080",
            "https://192.168.1.20:8443",
        ] {
            assert!(is_on_a_local_network(endpoint), "{endpoint}");
        }
    }

    /// **And everything else is not**: the internet, the shared range, this
    /// machine's loopback, every name, the lookalikes this file exists for, and
    /// an address that cannot be read.
    #[test]
    fn an_address_that_is_not_on_a_local_network_is_not_one() {
        for endpoint in [
            "http://198.51.100.7:8000",
            "http://8.8.8.8",
            "http://172.32.0.1",
            "http://172.15.255.255",
            "http://100.64.0.1",
            "http://[2001:db8::1]",
            "http://127.0.0.1:11434",
            "http://[::1]:8080",
            "http://localhost:8000",
            "http://nas.local:11434",
            "http://my-nas",
            "http://192.168.1.20.attacker.example",
            "http://192.168.1.20@attacker.example/",
            "http://attacker.example/192.168.1.20",
            "http://[::ffff:192.168.1.20]",
            "http://192.168.1",
            "ftp://192.168.1.20",
            "192.168.1.20:11434",
            "",
        ] {
            assert!(!is_on_a_local_network(endpoint), "{endpoint}");
        }
    }

    /// **Only plain `http://` to such an address crosses it unencrypted**:
    /// https does not, loopback does not, and an address on the internet is
    /// not on the person's network at all.
    #[test]
    fn only_plain_http_to_a_local_address_crosses_it_unencrypted() {
        assert!(crosses_a_local_network_unencrypted(
            "http://192.168.1.20:11434"
        ));
        assert!(crosses_a_local_network_unencrypted("http://[fd00::1]:8080"));
        for endpoint in [
            "https://192.168.1.20:8443",
            "http://127.0.0.1:11434",
            "http://localhost:11434",
            "http://198.51.100.7:8000",
            "http://nas.local:11434",
            "http://192.168.1.20@attacker.example/",
        ] {
            assert!(!crosses_a_local_network_unencrypted(endpoint), "{endpoint}");
        }
    }

    /// **What a request would connect to**, which is the first half of ADR
    /// 0020's separation: a name to look up and a port to reach it on, decided
    /// here where the lookalike cases already have tests.
    #[test]
    fn where_a_request_would_connect_is_the_host_and_the_scheme_s_port() {
        assert_eq!(
            where_it_connects("https://api.mistral.ai/v1"),
            Some(("api.mistral.ai".to_owned(), 443))
        );
        assert_eq!(
            where_it_connects("http://127.0.0.1:41234"),
            Some(("127.0.0.1".to_owned(), 41234))
        );
        assert_eq!(
            where_it_connects("http://127.0.0.1:41234/v1"),
            Some(("127.0.0.1".to_owned(), 41234))
        );
        assert_eq!(
            where_it_connects("http://localhost"),
            Some(("localhost".to_owned(), 80))
        );
        assert_eq!(
            where_it_connects("https://[::1]:8443/v1"),
            Some(("::1".to_owned(), 8443))
        );
        // Not an address this can reach, and not one it guesses at either.
        assert_eq!(where_it_connects("ftp://example.test"), None);
    }
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
