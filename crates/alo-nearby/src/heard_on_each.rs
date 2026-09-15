//! What one look heard on each network this machine is on, as one answer.
//!
//! A machine in an office is often on more than one network at once — a docked
//! laptop on the wired LAN and on Wi-Fi, a GPU box with two ports. Whoever looks
//! asks on each of them ([`crate::Looking`] on a socket per network), and each
//! network's window is an [`Around`] of its own. [`Around::heard_on_each`] is
//! how those become one answer, and the rule it keeps is the task's sentence:
//! **a machine heard on two networks is one machine, with the address it
//! answered from on each.**
//!
//! # The rule, for machines
//!
//! One [`Found`](crate::Found) per identity. The first network it was heard on gives its
//! [`Found::address`](crate::Found::address) and its port; each further network that heard the same
//! identity at the same port adds the address it answered from there to
//! [`Found::also_at`](crate::Found::also_at). A further network that heard the identity at a
//! *different* port is stepped over, which is the rule one window already
//! keeps — within one window the first answer for an identity is the one kept —
//! and nothing on this machine dials a machine because it was found, so which
//! claim is written down first decides nothing about reaching it.
//!
//! # And a network heard in two families
//!
//! A network with IPv4 and IPv6 on it is asked twice — once at `224.0.0.251`,
//! once at `ff02::fb` on the interface — and each is a window of its own, so the
//! same rule makes **a machine heard over IPv4 and over IPv6 on one network one
//! machine with an address in each family**. Addresses are compared with the
//! interface a link-local one was heard on ([`crate::HeardFrom`]): `fe80::1` on
//! the wired network and `fe80::1` on the wireless one are two addresses. Which
//! one [`Found::address`](crate::Found::address) is — and so which a pairing dials — is whichever
//! window the caller hands in first; `alo-agentd` hands in every IPv4 network
//! before any IPv6 one, and says why.
//!
//! # And for workspaces, where two claims are a refusal
//!
//! A workspace heard from two addresses **on one network** is two claims
//! nobody can tell apart, and `alo-agentd` refuses to open it rather than prefer
//! one (task 18). That must still be true after this merge, so a network's
//! claims for one host are merged only where that network heard **one** of them:
//! a host heard alone on each of two networks is one workspace with an address on
//! each, and a host heard twice on either network stays two entries — which is
//! the refusal it was before there was a second network to look on.
//!
//! Advertisements are not proven, here or anywhere: a stranger on one network
//! can claim the identity of a host on the other. What makes that safe is what
//! made it safe on one network — finding confers nothing, a name is shown only
//! where the paired machine answered from the same addresses, and a workspace is
//! reached through its own sign-in.

use crate::looking::Around;

impl Around {
    /// One answer out of what each network's window heard, in the order the
    /// networks were asked — see this module's documentation for the rule.
    ///
    /// A look on one network is unchanged by passing through here: the answer
    /// is that network's window as it was heard.
    #[must_use]
    pub fn heard_on_each(each: impl IntoIterator<Item = Self>) -> Self {
        let mut heard = Self::default();
        for network in each {
            for found in network.machines {
                match heard
                    .machines
                    .iter_mut()
                    .find(|already| already.machine == found.machine)
                {
                    Some(already) => {
                        if already.port == found.port
                            && already.addresses().all(|at| at != found.address)
                        {
                            already.also_at.push(found.address);
                        }
                    }
                    None => heard.machines.push(found),
                }
            }
            for found in &network.workspaces {
                let alone_on_this_network = network
                    .workspaces
                    .iter()
                    .filter(|other| other.host() == found.host())
                    .count()
                    == 1;
                let already = heard.workspaces.iter_mut().find(|already| {
                    already.host() == found.host() && already.port() == found.port()
                });
                match (alone_on_this_network, already) {
                    (true, Some(already)) => already.also_heard_at(found.address()),
                    _ => heard.workspaces.push(found.clone()),
                }
            }
        }
        heard
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::looking::Around;
    use crate::machine::MachineId;
    use crate::presence::Found;
    use crate::workspace::FoundWorkspace;

    /// The machine heard.
    fn the_studio() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// Another machine.
    fn reception() -> MachineId {
        MachineId::read("99998888777766665555444433332222").unwrap()
    }

    /// An address on the wired network.
    fn wired(last: u8) -> IpAddr {
        Ipv4Addr::new(10, 61, 1, last).into()
    }

    /// An address on the wireless one.
    fn wireless(last: u8) -> IpAddr {
        Ipv4Addr::new(10, 61, 2, last).into()
    }

    /// One network's window.
    fn a_window(machines: Vec<Found>, workspaces: Vec<FoundWorkspace>) -> Around {
        Around {
            machines,
            workspaces,
        }
    }

    /// **A machine heard on two networks is one machine, with the address it
    /// answered from on each.**
    #[test]
    fn a_machine_heard_on_two_networks_is_one_machine_with_an_address_on_each() {
        let heard = Around::heard_on_each([
            a_window(vec![Found::seen(the_studio(), 7_610, wired(2))], vec![]),
            a_window(vec![Found::seen(the_studio(), 7_610, wireless(2))], vec![]),
        ]);
        assert_eq!(heard.machines.len(), 1, "{heard:?}");
        let one = heard.machines.first().unwrap();
        assert_eq!(one.machine, the_studio());
        assert_eq!(one.address, wired(2));
        assert_eq!(
            one.addresses().collect::<Vec<_>>(),
            vec![wired(2), wireless(2)]
        );
        assert_eq!(
            one.where_it_answers(),
            std::net::SocketAddr::new(wired(2), 7_610)
        );
    }

    /// **Two machines on two networks stay two**, each where it was heard, and
    /// a look on one network passes through unchanged.
    #[test]
    fn different_machines_stay_apart_and_one_network_is_unchanged() {
        let one_network = a_window(
            vec![
                Found::seen(the_studio(), 7_610, wired(2)),
                Found::seen(reception(), 7_610, wired(3)),
            ],
            vec![FoundWorkspace::heard(the_studio(), 8_443, wired(2).into())],
        );
        assert_eq!(Around::heard_on_each([one_network.clone()]), one_network);

        let heard = Around::heard_on_each([
            a_window(vec![Found::seen(the_studio(), 7_610, wired(2))], vec![]),
            a_window(vec![Found::seen(reception(), 7_610, wireless(3))], vec![]),
        ]);
        assert_eq!(heard.machines.len(), 2, "{heard:?}");
        assert!(heard.machines.iter().all(|found| found.also_at.is_empty()));
    }

    /// A link-local address on the interface numbered `scope`.
    fn link_local(last: u16, scope: u32) -> crate::HeardFrom {
        crate::HeardFrom::of(std::net::SocketAddr::V6(std::net::SocketAddrV6::new(
            std::net::Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, last),
            5_353,
            0,
            scope,
        )))
    }

    /// **A machine heard over IPv4 and over IPv6 on one network is one machine
    /// with an address in each family**, the link-local one with its interface;
    /// the same address heard again in the same family adds nothing, and the
    /// same link-local address on another interface is another address.
    #[test]
    fn a_machine_heard_in_both_families_is_one_machine_with_an_address_in_each() {
        let heard = Around::heard_on_each([
            a_window(vec![Found::seen(the_studio(), 7_610, wired(2))], vec![]),
            a_window(
                vec![Found::heard(the_studio(), 7_610, link_local(2, 3))],
                vec![],
            ),
            a_window(
                vec![Found::heard(the_studio(), 7_610, link_local(2, 3))],
                vec![],
            ),
            a_window(
                vec![Found::heard(the_studio(), 7_610, link_local(2, 4))],
                vec![],
            ),
        ]);
        assert_eq!(heard.machines.len(), 1, "{heard:?}");
        let one = heard.machines.first().unwrap();
        assert_eq!(one.address, wired(2));
        assert_eq!(
            one.addresses().collect::<Vec<_>>(),
            vec![wired(2).into(), link_local(2, 3), link_local(2, 4)]
        );
        assert_eq!(
            one.where_it_answers(),
            std::net::SocketAddr::new(wired(2), 7_610),
            "the first window heard is what is dialled"
        );

        let workspaces = Around::heard_on_each([
            a_window(
                vec![],
                vec![FoundWorkspace::heard(the_studio(), 8_443, wired(2).into())],
            ),
            a_window(
                vec![],
                vec![FoundWorkspace::heard(the_studio(), 8_443, link_local(2, 3))],
            ),
        ]);
        assert_eq!(workspaces.workspaces.len(), 1, "{workspaces:?}");
        assert_eq!(
            workspaces.workspaces.first().unwrap().addresses().count(),
            2
        );
    }

    /// **The same identity at a different port on the other network is not
    /// added to the machine**: the first claim is kept, as one window keeps it.
    #[test]
    fn the_same_identity_at_another_port_is_not_the_same_machine() {
        let heard = Around::heard_on_each([
            a_window(vec![Found::seen(the_studio(), 7_610, wired(2))], vec![]),
            a_window(vec![Found::seen(the_studio(), 9_999, wireless(9))], vec![]),
        ]);
        assert_eq!(heard.machines.len(), 1, "{heard:?}");
        let one = heard.machines.first().unwrap();
        assert_eq!(one.port, 7_610);
        assert!(one.also_at.is_empty(), "{one:?}");
    }

    /// **A workspace heard alone on each of two networks is one workspace**
    /// with an address on each.
    #[test]
    fn a_workspace_heard_alone_on_each_network_is_one_workspace() {
        let heard = Around::heard_on_each([
            a_window(
                vec![],
                vec![FoundWorkspace::heard(the_studio(), 8_443, wired(2).into())],
            ),
            a_window(
                vec![],
                vec![FoundWorkspace::heard(
                    the_studio(),
                    8_443,
                    wireless(2).into(),
                )],
            ),
        ]);
        assert_eq!(heard.workspaces.len(), 1, "{heard:?}");
        let one = heard.workspaces.first().unwrap();
        assert_eq!(one.address(), wired(2));
        assert_eq!(
            one.addresses().collect::<Vec<_>>(),
            vec![wired(2), wireless(2)]
        );
    }

    /// **Two claims on one network stay two**, whatever the other network
    /// heard — so the refusal to open a workspace nobody can tell apart is
    /// what it was on one network.
    #[test]
    fn two_claims_for_one_workspace_on_one_network_are_not_merged_away() {
        let heard = Around::heard_on_each([
            a_window(
                vec![],
                vec![
                    FoundWorkspace::heard(the_studio(), 8_443, wired(2).into()),
                    FoundWorkspace::heard(the_studio(), 8_443, wired(66).into()),
                ],
            ),
            a_window(
                vec![],
                vec![FoundWorkspace::heard(
                    the_studio(),
                    8_443,
                    wireless(2).into(),
                )],
            ),
        ]);
        let places: Vec<_> = heard
            .workspaces
            .iter()
            .map(FoundWorkspace::where_it_answers)
            .collect();
        assert_eq!(places.len(), 2, "{heard:?}");
        let first = places.first().unwrap();
        assert!(
            places.iter().any(|at| at != first),
            "two claims were merged into one: {heard:?}"
        );

        let heard = Around::heard_on_each([
            a_window(
                vec![],
                vec![FoundWorkspace::heard(the_studio(), 8_443, wired(2).into())],
            ),
            a_window(
                vec![],
                vec![
                    FoundWorkspace::heard(the_studio(), 8_443, wireless(2).into()),
                    FoundWorkspace::heard(the_studio(), 8_443, wireless(66).into()),
                ],
            ),
        ]);
        assert_eq!(heard.workspaces.len(), 3, "{heard:?}");
    }
}
