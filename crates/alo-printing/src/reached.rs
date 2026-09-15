//! Where a printer is, and therefore whether printing on it is a document
//! leaving this machine.
//!
//! The printing service names every device it finds by an address — a URI whose
//! scheme is how it would be reached. That scheme is machinery and a person
//! never reads it; what they read, and what law 1 needs, is the one fact it
//! decides: **is this printer on this machine, or across the network?**
//!
//! # A printer across the network is a destination
//!
//! A document sent to a printer on the office network has left this machine,
//! as surely as one sent to a service on another continent. So a printer
//! [`Reached::OnTheNetwork`] has an [`alo_egress::Destination`], and printing
//! on it for an agent is an [`alo_egress::Leaving`] with
//! [`alo_egress::Why::Sending`] — the indicator lights for it the way it lights
//! for anything else an agent sends. Nobody paired a printer and nobody
//! declared where it runs, so it is the honest kind of destination, an address,
//! and an organisation that permits nothing to leave permits no agent to print
//! across the network: `docs/autonomy/updates/` has the report that says so.
//!
//! # And a printer on this machine is not
//!
//! A printer on a USB cable, or one reached over IPP-over-USB at this machine's
//! own loopback address, is a document going down a wire to a device on the
//! desk. Nothing leaves, and [`Reached::leaving`] answers [`None`] — which is
//! the same shape `alo_egress::Leaving::asking` gives a question answered here.
//!
//! # Deciding conservatively
//!
//! A printer found through DNS-SD is named by its service name, not by where it
//! is. It is counted as **across the network** whatever it later resolves to:
//! the indicator lighting for a print that did not leave is a line too many,
//! and the other mistake is a document leaving silently.

use alo_capability::Grantee;
use alo_egress::{Destination, DestinationError, Leaving, Why};

/// Where a printer is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reached {
    /// On this machine: on a cable, or at this machine's own address.
    OnThisMachine,
    /// Across the network, at this host.
    OnTheNetwork {
        /// The host, or the service name it announced itself under.
        host: String,
    },
}

/// How a printer at an address is spoken to, which decides whether it can be
/// set up without a driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaks {
    /// IPP, which every printer sold as driverless speaks, and the printing
    /// service sets up from what the printer says about itself.
    Driverless,
    /// Something that needs a program from the printer's maker to describe
    /// the printer, which this machine does not install.
    OnlyThroughItsMakersProgram,
}

impl Reached {
    /// Where a device at this address is, and how it is spoken to — or
    /// [`None`] for an address that is not a printer this crate sets up at
    /// all: a file, a printer made of software, a serial port.
    #[must_use]
    pub fn of_device(address: &str) -> Option<(Self, Speaks)> {
        let (scheme, rest) = address.split_once("://")?;
        let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
        match scheme.to_ascii_lowercase().as_str() {
            "ipp" | "ipps" => Some((at_host(authority)?, Speaks::Driverless)),
            "dnssd" => {
                let service = percent_decoded(authority)?;
                let speaks = if service.contains("._ipp._tcp") || service.contains("._ipps._tcp") {
                    Speaks::Driverless
                } else {
                    Speaks::OnlyThroughItsMakersProgram
                };
                Some((Self::OnTheNetwork { host: service }, speaks))
            }
            "ippusb" => Some((Self::OnThisMachine, Speaks::Driverless)),
            "usb" => Some((Self::OnThisMachine, Speaks::OnlyThroughItsMakersProgram)),
            "socket" | "lpd" => Some((at_host(authority)?, Speaks::OnlyThroughItsMakersProgram)),
            _ => None,
        }
    }

    /// Whether printing on it sends a document across the network.
    #[must_use]
    pub fn crosses_the_network(&self) -> bool {
        matches!(self, Self::OnTheNetwork { .. })
    }

    /// Where a document printed on it goes, if it leaves this machine.
    ///
    /// # Errors
    /// [`DestinationError`] for a host that could not be shown on one line —
    /// and then nothing may be sent to it, because nothing could say where it
    /// went.
    pub fn destination(&self) -> Result<Option<Destination>, DestinationError> {
        match self {
            Self::OnThisMachine => Ok(None),
            Self::OnTheNetwork { host } => Destination::at(host).map(Some),
        }
    }

    /// The egress an agent causes by printing on it, or [`None`] when nothing
    /// leaves.
    ///
    /// # Errors
    /// [`DestinationError`], as [`Reached::destination`].
    pub fn leaving(&self, agent: &Grantee) -> Result<Option<Leaving>, DestinationError> {
        Ok(self
            .destination()?
            .map(|destination| Leaving::because(agent, Why::Sending, destination)))
    }
}

/// A printer at this `host[:port]`, which is on this machine when the host is
/// this machine's own.
fn at_host(authority: &str) -> Option<Reached> {
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(bracketed) = authority.strip_prefix('[') {
        bracketed.split(']').next()?
    } else {
        authority.split(':').next()?
    };
    let host = percent_decoded(host)?;
    if host.is_empty() {
        return None;
    }
    let local = host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    Some(if local {
        Reached::OnThisMachine
    } else {
        Reached::OnTheNetwork { host }
    })
}

/// Text with its `%xx` escapes decoded, or [`None`] if they are not valid
/// UTF-8 or not escapes at all.
fn percent_decoded(text: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut rest = text.as_bytes();
    while let Some((&first, after)) = rest.split_first() {
        if first == b'%' {
            let hex = after.get(..2)?;
            let hex = std::str::from_utf8(hex).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = after.get(2..)?;
        } else {
            bytes.push(first);
            rest = after;
        }
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every scheme the printing service's backends name a printer by lands on
    /// one side of the network or the other, and one of the two ways of being
    /// spoken to.
    #[test]
    fn every_address_a_printer_is_found_at_says_where_it_is() {
        let cases = [
            (
                "ipp://192.168.1.20:631/ipp/print",
                false,
                Speaks::Driverless,
            ),
            (
                "ipps://printer.office.example/ipp/print",
                false,
                Speaks::Driverless,
            ),
            (
                "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000",
                false,
                Speaks::Driverless,
            ),
            (
                "dnssd://Old%20Laser._pdl-datastream._tcp.local/",
                false,
                Speaks::OnlyThroughItsMakersProgram,
            ),
            (
                "socket://192.168.1.30:9100",
                false,
                Speaks::OnlyThroughItsMakersProgram,
            ),
            ("ipp://localhost:60000/ipp/print", true, Speaks::Driverless),
            ("ipp://127.0.0.1:60000/ipp/print", true, Speaks::Driverless),
            ("ipp://[::1]:60000/ipp/print", true, Speaks::Driverless),
            ("ippusb://HP%20LaserJet/?serial=X", true, Speaks::Driverless),
            (
                "usb://HP/LaserJet?serial=X",
                true,
                Speaks::OnlyThroughItsMakersProgram,
            ),
        ];
        for (address, on_this_machine, speaks) in cases {
            let (reached, how) = Reached::of_device(address).unwrap();
            assert_eq!(!reached.crosses_the_network(), on_this_machine, "{address}");
            assert_eq!(how, speaks, "{address}");
        }
        assert_eq!(
            Reached::of_device("dnssd://Brother%20HL-L2350DW._ipp._tcp.local/")
                .unwrap()
                .0,
            Reached::OnTheNetwork {
                host: "Brother HL-L2350DW._ipp._tcp.local".to_owned()
            }
        );
    }

    /// **What is not a printer is not found as one.** A file, a printer made
    /// of software and a port nothing announces are not places this crate sets
    /// a document going.
    #[test]
    fn an_address_that_is_not_a_printer_is_not_one() {
        for address in [
            "file:///dev/null",
            "cups-pdf:/",
            "serial:/dev/ttyS0",
            "ipp://",
            "ipp://%zz/",
            "no scheme at all",
        ] {
            assert_eq!(Reached::of_device(address), None, "{address}");
        }
    }

    /// **A document to a printer across the network is an agent sending
    /// something**, and a document down a cable is not a departure at all.
    #[test]
    fn across_the_network_is_a_departure_and_down_a_cable_is_not() {
        let agent = Grantee::named("@files");
        let (network, _) = Reached::of_device("ipp://192.168.1.20/ipp/print").unwrap();
        let leaving = network.leaving(&agent).unwrap().unwrap();
        assert_eq!(leaving.why(), Why::Sending);
        assert_eq!(
            leaving.destination(),
            &Destination::at("192.168.1.20").unwrap()
        );
        assert_eq!(leaving.agent(), &agent);

        let (cable, _) = Reached::of_device("usb://HP/LaserJet").unwrap();
        assert_eq!(cable.leaving(&agent).unwrap(), None);
    }
}
