//! Which of this machine's networks discovery is joined on, decided from what
//! the kernel reports and nothing else.
//!
//! *Machines find each other with zero configuration.* A machine in an office is
//! often on more than one network at once — a docked laptop on the wired LAN and
//! on Wi-Fi, a GPU box with two ports — and a discovery group joined with the
//! unspecified address is joined on **one** interface the kernel chooses. So a
//! machine on two networks was found on one and silently absent from the other.
//! This file is the decision that replaces that choice: discovery is joined on
//! **every** interface that is up, can carry multicast, and has an IPv4 address,
//! and never on loopback.
//!
//! # A pure function of the interfaces the kernel reports
//!
//! [`discovery_networks`] takes a list of [`Interface`]s and answers with the
//! [`Network`]s to join, so each rule is a test with no kernel in it. What the
//! kernel actually reports is `crate::route_messages`'s, and joining is
//! `crate::joining`'s; neither decides anything this file decides.
//!
//! - **Up** is the interface administratively up *and* running — `IFF_UP` and
//!   `IFF_RUNNING`. An interface a person switched on with no cable in it, or a
//!   Wi-Fi card associated with nothing, is not a network anybody can hear this
//!   machine on.
//! - **Multicast** is `IFF_MULTICAST`: a point-to-point tunnel without it would
//!   take the join and carry nothing.
//! - **An IPv4 address**, because discovery here is IPv4 multicast; the first one
//!   the kernel lists is the one the interface is joined and asked on.
//! - **Not loopback**, by the flag and by the address: a question on loopback
//!   reaches this machine only, which is not a network.
//!
//! # And over IPv6, where there is no IPv4 address at all
//!
//! A network nobody configured often has no IPv4 address on it — two machines
//! on one cable with no DHCP server, an office whose router is down, a network
//! run IPv6-only — and every interface on one still gives itself an IPv6
//! link-local address. So [`link_local_networks`] is the same rule with the
//! address rule changed: up and running, multicast, not loopback, and **an IPv6
//! link-local address** (`fe80::/10`) the kernel has finished checking, the
//! first of which is the one questions leave from. Discovery is joined there at
//! `ff02::fb` on that interface ([`alo_nearby::THE_IPV6_ADDRESS`]). An interface
//! with both is two networks — one per family — and what is said on each is the
//! same bytes. [`every_discovery_network`] is both lists, **every IPv4 network
//! first**: a machine heard in both families is written down at the address
//! heard first, which is the one a pairing dials, and an IPv4 address keeps
//! dialling what it dialled before a second family was asked.
//!
//! # And there is no setting
//!
//! Nothing here takes a list of networks from a person, an agent or a file (ADR
//! 0003): a list of networks to advertise on is the trusted-network switch by
//! another name. What the machine is plugged into is the whole of the input.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// `IFF_UP`: the interface is administratively up.
pub const IFF_UP: u32 = 0x1;
/// `IFF_LOOPBACK`: the interface is loopback.
pub const IFF_LOOPBACK: u32 = 0x8;
/// `IFF_RUNNING`: the interface is operationally up.
pub const IFF_RUNNING: u32 = 0x40;
/// `IFF_MULTICAST`: the interface carries multicast.
pub const IFF_MULTICAST: u32 = 0x1000;

/// One interface, as the kernel reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    /// The kernel's index for it, which is how an address names its interface.
    pub index: u32,
    /// Its name, for a line in the service log.
    pub name: String,
    /// The kernel's `IFF_*` flags for it.
    pub flags: u32,
    /// Its IPv4 addresses, in the order the kernel listed them.
    pub addresses: Vec<Ipv4Addr>,
    /// Its IPv6 addresses that can be used now — not ones the kernel is still
    /// checking nobody else holds, nor ones that check failed — in the order
    /// the kernel listed them.
    pub ipv6: Vec<Ipv6Addr>,
}

/// One network discovery is joined and asked on: an interface, and the address
/// it is joined and asked from — an IPv4 address, or an IPv6 link-local one, so
/// an interface with both is two networks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Network {
    /// The kernel's index for the interface.
    index: u32,
    /// The interface's name.
    name: String,
    /// The address the group is joined at, and questions leave from.
    address: IpAddr,
}

impl Network {
    /// The kernel's index for the interface.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// The interface's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The address the group is joined at, and questions leave from.
    #[must_use]
    pub const fn address(&self) -> IpAddr {
        self.address
    }
}

/// Every network discovery joins and asks on, out of what the kernel reports:
/// each interface that is up and running, carries multicast and has an IPv4
/// address, and is not loopback — in the order the kernel reported them.
#[must_use]
pub fn discovery_networks(reported: &[Interface]) -> Vec<Network> {
    reported
        .iter()
        .filter(|interface| carries_discovery(interface))
        .filter_map(|interface| {
            let address = interface
                .addresses
                .iter()
                .copied()
                .find(|address| !address.is_loopback() && !address.is_unspecified())?;
            Some(Network {
                index: interface.index,
                name: interface.name.clone(),
                address: address.into(),
            })
        })
        .collect()
}

/// Every network discovery joins and asks on over IPv6, out of what the kernel
/// reports: each interface that is up and running, carries multicast, has an
/// IPv6 link-local address, and is not loopback — in the order the kernel
/// reported them.
///
/// Link-local and nothing else: `ff02::fb` is a link-local group, a question to
/// it is asked on one interface from that interface's own link-local address,
/// and that address is the one every interface has with nobody configuring it.
#[must_use]
pub fn link_local_networks(reported: &[Interface]) -> Vec<Network> {
    reported
        .iter()
        .filter(|interface| carries_discovery(interface))
        .filter_map(|interface| {
            let address = interface.ipv6.iter().copied().find(is_link_local)?;
            Some(Network {
                index: interface.index,
                name: interface.name.clone(),
                address: address.into(),
            })
        })
        .collect()
}

/// Every network discovery joins and asks on, in both families: every IPv4
/// network, then every IPv6 link-local one — see this module's documentation
/// for why in that order.
#[must_use]
pub fn every_discovery_network(reported: &[Interface]) -> Vec<Network> {
    let mut networks = discovery_networks(reported);
    networks.extend(link_local_networks(reported));
    networks
}

/// Up and running, carrying multicast, and not loopback.
fn carries_discovery(interface: &Interface) -> bool {
    let has = |flag: u32| interface.flags & flag == flag;
    has(IFF_UP) && has(IFF_RUNNING) && has(IFF_MULTICAST) && !has(IFF_LOOPBACK)
}

/// Whether `address` is in `fe80::/10`.
const fn is_link_local(address: &Ipv6Addr) -> bool {
    address.segments()[0] & 0xffc0 == 0xfe80
}

/// Join each of `networks` with `join`, and answer with the ones that joined.
///
/// **A network that cannot be joined is a line, and the others are still
/// joined**: `said` is handed one sentence naming the interface and what the
/// machine said, for the service log, and nothing stops. A machine on three
/// networks that cannot join one of them is found on two, which is better than a
/// service that is found on none because it would not start.
pub fn joined_on(
    networks: Vec<Network>,
    mut join: impl FnMut(&Network) -> Result<(), std::io::Error>,
    mut said: impl FnMut(&str),
) -> Vec<Network> {
    networks
        .into_iter()
        .filter(|network| match join(network) {
            Ok(()) => true,
            Err(why) => {
                said(&format!(
                    "discovery could not be joined on {} ({}): {why}; this machine is not found on that network until it can be",
                    network.name, network.address
                ));
                false
            }
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use super::*;

    /// Everything an interface discovery joins on has.
    const JOINABLE: u32 = IFF_UP | IFF_RUNNING | IFF_MULTICAST;

    /// An interface with these flags and one address.
    fn an_interface(index: u32, name: &str, flags: u32, address: Option<Ipv4Addr>) -> Interface {
        Interface {
            index,
            name: name.to_owned(),
            flags,
            addresses: address.into_iter().collect(),
            ipv6: Vec::new(),
        }
    }

    /// An interface with these flags and these IPv6 addresses, and no IPv4.
    fn an_ipv6_interface(index: u32, name: &str, flags: u32, ipv6: &[Ipv6Addr]) -> Interface {
        Interface {
            index,
            name: name.to_owned(),
            flags,
            addresses: Vec::new(),
            ipv6: ipv6.to_vec(),
        }
    }

    /// The link-local address the wired interface gave itself.
    const WIRED_LINK_LOCAL: Ipv6Addr =
        Ipv6Addr::new(0xfe80, 0, 0, 0, 0xa406, 0xe5ff, 0xfe4b, 0xac9e);
    /// A global IPv6 address, which is not link-local.
    const GLOBAL: Ipv6Addr = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 7);

    /// The wired network's address.
    const WIRED: Ipv4Addr = Ipv4Addr::new(10, 61, 1, 1);
    /// The wireless network's.
    const WIRELESS: Ipv4Addr = Ipv4Addr::new(10, 61, 2, 1);

    /// **Every interface that is up, carries multicast and has an address is
    /// joined** — both of a docked laptop's, in the kernel's order.
    #[test]
    fn every_network_the_machine_is_on_is_joined() {
        let networks = discovery_networks(&[
            an_interface(2, "eth0", JOINABLE, Some(WIRED)),
            an_interface(3, "wlan0", JOINABLE | 0x2, Some(WIRELESS)),
        ]);
        assert_eq!(
            networks
                .iter()
                .map(|network| (network.index(), network.name(), network.address()))
                .collect::<Vec<_>>(),
            vec![
                (2, "eth0", IpAddr::from(WIRED)),
                (3, "wlan0", IpAddr::from(WIRELESS))
            ]
        );
    }

    /// **An interface that is down is not joined** — neither one switched off
    /// nor one switched on with nothing at the other end of it.
    #[test]
    fn an_interface_that_is_down_is_not_joined() {
        let networks = discovery_networks(&[
            an_interface(2, "eth0", IFF_MULTICAST, Some(WIRED)),
            an_interface(3, "eth1", IFF_UP | IFF_MULTICAST, Some(WIRELESS)),
        ]);
        assert!(networks.is_empty(), "{networks:?}");
    }

    /// **An interface without multicast is not joined.**
    #[test]
    fn an_interface_without_multicast_is_not_joined() {
        let networks =
            discovery_networks(&[an_interface(4, "tun0", IFF_UP | IFF_RUNNING, Some(WIRED))]);
        assert!(networks.is_empty(), "{networks:?}");
    }

    /// **An interface without an IPv4 address is not joined**, and an address
    /// that is no address at all is not one.
    #[test]
    fn an_interface_without_an_address_is_not_joined() {
        let networks = discovery_networks(&[
            an_interface(2, "eth0", JOINABLE, None),
            an_interface(3, "eth1", JOINABLE, Some(Ipv4Addr::UNSPECIFIED)),
        ]);
        assert!(networks.is_empty(), "{networks:?}");
    }

    /// **Loopback is not joined**, by its flag or by its address.
    #[test]
    fn loopback_is_not_joined() {
        let networks = discovery_networks(&[
            an_interface(1, "lo", JOINABLE | IFF_LOOPBACK, Some(Ipv4Addr::LOCALHOST)),
            an_interface(5, "odd", JOINABLE, Some(Ipv4Addr::new(127, 0, 0, 2))),
        ]);
        assert!(networks.is_empty(), "{networks:?}");
    }

    /// **The first usable address is the one joined at**, when an interface
    /// has several.
    #[test]
    fn the_first_usable_address_is_the_one_joined_at() {
        let mut interface = an_interface(2, "eth0", JOINABLE, Some(Ipv4Addr::UNSPECIFIED));
        interface.addresses.extend([WIRED, WIRELESS]);
        let networks = discovery_networks(&[interface]);
        assert_eq!(networks.first().unwrap().address(), IpAddr::from(WIRED));
    }

    /// **Over IPv6, every interface that is up, carries multicast and has a
    /// link-local address is joined** — one with no IPv4 address at all, the
    /// cable between two machines with no DHCP server — at the first link-local
    /// address, past a global one listed before it.
    #[test]
    fn every_interface_with_a_link_local_address_is_joined_over_ipv6() {
        let networks = link_local_networks(&[
            an_ipv6_interface(2, "eth0", JOINABLE, &[GLOBAL, WIRED_LINK_LOCAL]),
            an_ipv6_interface(
                3,
                "wlan0",
                JOINABLE,
                &[Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 9)],
            ),
        ]);
        assert_eq!(
            networks
                .iter()
                .map(|network| (network.index(), network.name(), network.address()))
                .collect::<Vec<_>>(),
            vec![
                (2, "eth0", IpAddr::from(WIRED_LINK_LOCAL)),
                (
                    3,
                    "wlan0",
                    IpAddr::from(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 9))
                )
            ]
        );
        assert!(
            discovery_networks(&[an_ipv6_interface(2, "eth0", JOINABLE, &[WIRED_LINK_LOCAL])])
                .is_empty()
        );
    }

    /// **Over IPv6, an interface that is down is not joined**, switched off or
    /// with nothing at the other end.
    #[test]
    fn an_interface_that_is_down_is_not_joined_over_ipv6() {
        let networks = link_local_networks(&[
            an_ipv6_interface(2, "eth0", IFF_MULTICAST, &[WIRED_LINK_LOCAL]),
            an_ipv6_interface(3, "eth1", IFF_UP | IFF_MULTICAST, &[WIRED_LINK_LOCAL]),
        ]);
        assert!(networks.is_empty(), "{networks:?}");
    }

    /// **Over IPv6, an interface without multicast is not joined.**
    #[test]
    fn an_interface_without_multicast_is_not_joined_over_ipv6() {
        let networks = link_local_networks(&[an_ipv6_interface(
            4,
            "tun0",
            IFF_UP | IFF_RUNNING,
            &[WIRED_LINK_LOCAL],
        )]);
        assert!(networks.is_empty(), "{networks:?}");
    }

    /// **Over IPv6, an interface without a link-local address is not joined** —
    /// not one with no IPv6 address, and not one whose only address is global,
    /// because the group is link-local and a question to it leaves from the
    /// interface's own link-local address.
    #[test]
    fn an_interface_without_a_link_local_address_is_not_joined_over_ipv6() {
        let networks = link_local_networks(&[
            an_ipv6_interface(2, "eth0", JOINABLE, &[]),
            an_ipv6_interface(3, "eth1", JOINABLE, &[GLOBAL, Ipv6Addr::UNSPECIFIED]),
            an_interface(5, "eth2", JOINABLE, Some(WIRED)),
        ]);
        assert!(networks.is_empty(), "{networks:?}");
    }

    /// **Over IPv6, loopback is not joined**, by its flag — whatever address
    /// it carries.
    #[test]
    fn loopback_is_not_joined_over_ipv6() {
        let networks = link_local_networks(&[an_ipv6_interface(
            1,
            "lo",
            JOINABLE | IFF_LOOPBACK,
            &[Ipv6Addr::LOCALHOST, WIRED_LINK_LOCAL],
        )]);
        assert!(networks.is_empty(), "{networks:?}");
    }

    /// **An interface with both families is two networks, every IPv4 one
    /// first**, so a machine heard in both is dialled where it was before IPv6
    /// was asked.
    #[test]
    fn an_interface_with_both_families_is_two_networks_ipv4_first() {
        let mut both = an_interface(2, "eth0", JOINABLE, Some(WIRED));
        both.ipv6.push(WIRED_LINK_LOCAL);
        let networks = every_discovery_network(&[
            an_ipv6_interface(
                3,
                "cable0",
                JOINABLE,
                &[Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 3)],
            ),
            both,
        ]);
        assert_eq!(
            networks
                .iter()
                .map(|network| (network.name(), network.address().is_ipv4()))
                .collect::<Vec<_>>(),
            vec![("eth0", true), ("cable0", false), ("eth0", false)]
        );
    }

    /// **A network over IPv6 that cannot be joined is a line in the service
    /// log too**, naming the interface and its address, and the others are
    /// joined.
    #[test]
    fn a_network_over_ipv6_that_cannot_be_joined_is_a_line_and_the_others_are_joined() {
        let mut both = an_interface(2, "eth0", JOINABLE, Some(WIRED));
        both.ipv6.push(WIRED_LINK_LOCAL);
        let mut lines = Vec::new();
        let joined = joined_on(
            every_discovery_network(&[both]),
            |network| {
                if network.address().is_ipv6() {
                    Err(std::io::Error::from(std::io::ErrorKind::AddrNotAvailable))
                } else {
                    Ok(())
                }
            },
            |line| lines.push(line.to_owned()),
        );
        assert_eq!(joined.len(), 1, "{joined:?}");
        assert!(joined.first().unwrap().address().is_ipv4());
        assert_eq!(lines.len(), 1, "{lines:?}");
        let line = lines.first().unwrap();
        assert!(
            line.contains("eth0") && line.contains("fe80::a406"),
            "{line}"
        );
    }

    /// **A network that cannot be joined is a line in the service log, and the
    /// others are still joined.**
    #[test]
    fn a_network_that_cannot_be_joined_is_a_line_and_the_others_are_joined() {
        let networks = discovery_networks(&[
            an_interface(2, "eth0", JOINABLE, Some(WIRED)),
            an_interface(3, "wlan0", JOINABLE, Some(WIRELESS)),
            an_interface(6, "eth2", JOINABLE, Some(Ipv4Addr::new(10, 61, 3, 1))),
        ]);
        let mut tried = Vec::new();
        let mut lines = Vec::new();
        let joined = joined_on(
            networks,
            |network| {
                tried.push(network.name().to_owned());
                if network.name() == "wlan0" {
                    Err(std::io::Error::from(std::io::ErrorKind::AddrNotAvailable))
                } else {
                    Ok(())
                }
            },
            |line| lines.push(line.to_owned()),
        );
        assert_eq!(tried, ["eth0", "wlan0", "eth2"]);
        assert_eq!(
            joined.iter().map(Network::name).collect::<Vec<_>>(),
            ["eth0", "eth2"]
        );
        assert_eq!(lines.len(), 1, "{lines:?}");
        let line = lines.first().unwrap();
        assert!(
            line.contains("wlan0") && line.contains("10.61.2.1"),
            "{line}"
        );
    }
}

/// No door chooses a network: ADR 0003's *no trusted-network setting*, as the
/// refusal a request shaped like one gets.
#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod no_door_chooses_a_network {
    use std::net::{Ipv4Addr, TcpListener, UdpSocket};

    use alo_capability::Grants;
    use alo_record::Record;

    use crate::answering::what_a_person_said;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::holding::Holding;
    use crate::network::TheNetwork;
    use crate::pairing::Nearby;
    use crate::rereading::WhatIsGranted;
    use crate::testing::{
        NothingIsRemembered, a_message, hour, noon, nothing_has_been_chosen,
        on_a_machine_that_answers, on_a_machine_with_no_turn, reception,
    };
    use crate::wire::Wire;

    /// **No request on either door names, chooses or turns off a network — or a
    /// family, IPv4 or IPv6 — to discover, advertise or pair on.** Requests
    /// shaped as though one could — on the
    /// person's door and on the agent's — are refused, nothing is written
    /// down, and what the machine advertises is what it was.
    #[test]
    fn no_request_on_either_door_chooses_a_network() {
        let lines = [
            r#"{"discover-on":{"interface":"eth0"}}"#,
            r#"{"advertise-on":{"networks":["eth0","wlan0"]}}"#,
            r#"{"discovery":{"off":true}}"#,
            r#"{"networks":{}}"#,
            r#"{"advertised":{"interface":"wlan0"}}"#,
            r#"{"workspaces":{"interface":"eth0"}}"#,
            r#"{"ipv6":{"off":true}}"#,
            r#"{"discover-on":{"family":"ipv4"}}"#,
            r#"{"advertised":{"family":"ipv6"}}"#,
            r#"{"pair":{"machine":"aaaabbbbccccddddeeeeffff00001111","may":["models"],"seconds":60,"family":"ipv4"}}"#,
        ];
        let quiet = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let wire = Wire::on(
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            reception(),
            quiet.local_addr().unwrap().port(),
        )
        .unwrap();
        let before = format!("{:?}", wire.advertising());
        let network = TheNetwork::on(reception());

        for line in lines {
            let mut record = Record::default();
            let said = on_a_machine_with_no_turn(
                "no-door-chooses-a-network",
                &mut record,
                |machine, _, strings, _, _| {
                    let mut grants = Grants::default();
                    what_a_person_said(
                        &a_message(line),
                        &mut Holding::Nobody(machine),
                        &mut WhatIsGranted::of(&mut grants, &NothingIsRemembered),
                        &Nearby {
                            network: &network,
                            looking: &wire,
                            advertising: &wire.advertising(),
                        },
                        strings,
                        noon(),
                    )
                    .unwrap()
                },
            );
            assert!(said.refusal().is_some(), "the person's door: {line}");
            assert!(record.is_empty(), "{line}");
        }

        let corridor = Corridor {
            network: &network,
            looking: &wire,
            naming: network.names(),
        };
        let mut questions = nothing_has_been_chosen();
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, grants, strings| {
            for line in lines {
                let said = what_an_agent_said(
                    &a_message(line),
                    turning,
                    &mut questions,
                    Some(&corridor),
                    grants,
                    strings,
                    hour(),
                    noon(),
                );
                assert!(said.refusal().is_some(), "the agent's door: {line}");
            }
        });

        assert_eq!(format!("{:?}", wire.advertising()), before);
        assert!(wire.joined().is_empty());
    }
}
