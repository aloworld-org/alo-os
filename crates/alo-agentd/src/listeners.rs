//! The port presence advertises, listened on once per network this machine is
//! on — each listener held to that network's interface.
//!
//! *Machines find each other with zero configuration, and trust none of them for
//! it.* Until this file `crate::wire` bound **one** listener, held to nothing,
//! and an unheld TCP listener answers every handshake by the route: for a
//! listening socket the kernel leaves `ireq->ir_iif` zero, so
//! `inet_csk_route_req` looks the SYN-ACK's route up unconstrained
//! (`docs/quirks.md`, measured 2026-09-16). On a machine on two networks whose
//! routers hand out the same private range — the commonest office and the
//! commonest home — that means **only the machine on the network the route
//! points at can reach this one's port at all**: the studio on the cable
//! proposes, its SYN arrives, the reply leaves by the Wi-Fi, and the handshake
//! never completes. Its person is left typing an address, which is the step the
//! promise removes.
//!
//! So the port is listened on with **one IPv4 listener per IPv4 network, each
//! held to that network's interface** (`SO_BINDTOIFINDEX`, as
//! `crate::looking::held_to` already holds a datagram socket), beside **one
//! IPv6-only listener held to nothing**. The kernel then answers each handshake
//! out of the interface its listener is held to, and a machine on either network
//! is reached.
//!
//! # Held listeners replace the one, rather than joining it
//!
//! Measured on this kernel: two listeners on `0.0.0.0` at one port, held to
//! different interfaces, both bind — the kernel's bind-conflict check treats a
//! different `sk_bound_dev_if` as a different binding — and a listener at the
//! same port held to **nothing** beside them is refused `EADDRINUSE`. So the
//! wire cannot keep its listener in both families and add held ones beside it:
//! that listener binds IPv4 too. What it binds instead is the held ones and an
//! IPv6-only listener, which takes no IPv4 address and conflicts with none of
//! them.
//!
//! # Loopback is one of the networks
//!
//! Discovery is never joined on loopback, because a question there reaches this
//! machine only. A **listener** there is another matter: a connection to
//! `127.0.0.1` is answered today, by the person's own machine and by every test
//! on one host, and it goes on being answered because loopback is one of
//! [`crate::networks::listening_networks`]. What a connection accepted there is
//! measured on is `crate::arrived_on`'s answer, and it is *its own network*.
//!
//! # The listeners follow the kernel, as discovery's joins do
//!
//! A laptop docked an hour after it started would otherwise be unreachable on
//! the wired network all day. So the same notification `crate::joining` follows
//! moves these too ([`Listeners::changed`], called from
//! `crate::wire::Wire::networks_changed`): every network not yet listened on is
//! bound, and a listener whose interface has gone is let go of. **A network that
//! will not bind is a line in the service log and the others are still bound** —
//! a machine reachable on two of its three networks is better than a service
//! that would not start.
//!
//! What refuses a network's bind is somebody else holding the port there
//! (`EADDRINUSE`), and **that line is said once**, when the network is first
//! refused — not again on every notification that tries it and is refused
//! again. When it does bind, the log says so, once.
//!
//! # And the port is tried again when another program lets go of it
//!
//! A program closing a socket is not a network change, so a network refused
//! because its port was taken would otherwise wait for a cable somewhere to
//! change. The listeners hold a second socket the kernel writes to when a TCP
//! socket **at this port** is destroyed (`crate::told_of_a_port_let_go`, which
//! has the readings weighed and why that one), opened before the first bind so a
//! let-go cannot fall between a refusal and the subscription. On hearing it,
//! every network refused is tried again ([`Listeners::let_go_of`]); a network
//! nothing refused costs nothing. A kernel that will not open that socket is a
//! line in the service log, and the refusal line then says the port is tried
//! again only when this machine's networks next change.
//! `crate::a_port_another_program_let_go_of` is the measurement, on a real
//! kernel and with no capabilities.
//!
//! An interface that went between the kernel's report and the
//! bind does **not** refuse — `SO_BINDTOIFINDEX` takes an index no interface has
//! (`docs/quirks.md`) — and leaves a listener nothing can reach, which the next
//! notification lets go of because the index is no longer reported.
//!
//! A listener is matched to its network by the interface's **index** and not by
//! the whole network, so an address changing on an interface — a lease renewed,
//! a second address added — leaves the listener and the connections waiting in
//! its backlog where they are.
//!
//! **And an interface deleted and laid again at the same number needs nothing
//! either**, which is where the listeners differ from discovery's responders
//! (`crate::responding`). `SO_BINDTOIFINDEX` is a number the kernel compares with
//! the interface a handshake arrived on, so a listener held to it is reached on
//! whatever interface bears that number now, and it joins no group that could
//! have gone with the old one. Measured by
//! `crate::a_cable_re_laid_between_two_readings`: the port is reached on a cable
//! re-laid at its number between two readings with the same listener in place.
//!
//! # A machine that cannot read its own networks
//!
//! If the kernel will not say which interfaces this machine has, there is
//! nothing to hold a listener to. Rather than not listening at all, one listener
//! is bound held to nothing — which is what every alo machine did before this
//! file — and the service log says so. That machine is reachable where the route
//! points, and reads a connection's network the old way
//! (`crate::arrived_on::the_network_it_arrived_on`). It does not begin following
//! the kernel later, because the one unheld listener holds the port against
//! every held listener that would replace it.
//!
//! # Nothing here is a setting
//!
//! ADR 0003. Which interfaces are listened on is what the machine is plugged
//! into, read from the kernel; no person and no agent names one, and the port is
//! the constant task 1 made it.

use std::collections::BTreeSet;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::num::NonZeroU32;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::sync::{Arc, Mutex, PoisonError};

use alo_corridor::AT_MOST_A_VERB;
use alo_nearby::http::{self, WHILE_THE_WIRE_ANSWERS};
use alo_nearby::{HeardFrom, NotNearby};

use crate::arrived_on::{ArrivedOn, the_network_it_arrived_on, what_a_listener_held_to};
use crate::networks::{Network, listening_networks};
use crate::refusing::NotBound;
use crate::route_messages::reported_by_the_kernel;
use crate::told_of_a_port_let_go::told_when_let_go_of;
use crate::wire::Knocked;

/// What this machine says when it is listening nowhere at all.
const NOTHING_IS_LISTENING: &str = "the port presence advertises";

/// One listener on the port, and what it says about the connections it accepts.
#[derive(Debug)]
struct Listener {
    /// The network it is held to, and nothing for one held to none.
    on: Option<Network>,
    /// What every connection it accepts arrived on, where the listener itself
    /// says — and nothing for a listener held to nothing, whose connections are
    /// read one at a time (`crate::arrived_on::the_network_it_arrived_on`).
    arrived: Option<ArrivedOn>,
    /// The listener.
    listener: TcpListener,
}

impl Listener {
    /// Accept one connection and read what it carries, with the network it
    /// arrived on beside it.
    fn accept_one(&self) -> Result<Knocked, NotNearby> {
        let (stream, who) = self
            .listener
            .accept()
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        stream
            .set_read_timeout(Some(WHILE_THE_WIRE_ANSWERS))
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        stream
            .set_write_timeout(Some(WHILE_THE_WIRE_ANSWERS))
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        let arrived = self
            .arrived
            .unwrap_or_else(|| the_network_it_arrived_on(&stream));
        // Every way the read fails is carried rather than answered here: a
        // stranger's bytes and a stranger who sent nothing for ten seconds are
        // both one connection, and neither is a reason to stop serving.
        let message = http::read_message_of_at_most(&stream, AT_MOST_A_VERB);
        Ok(Knocked {
            stream,
            from: HeardFrom::of(who),
            arrived,
            message,
        })
    }
}

/// Every listener on the port presence advertises.
#[derive(Debug)]
pub struct Listeners {
    /// The IPv4 listeners, one per network this machine is on, each held to
    /// that network's interface — and empty on a machine listening through
    /// [`fixed`](Self::fixed) alone.
    ///
    /// **Locked by the service's own thread and no other.** What is taken out
    /// of it is a handle onto each listener ([`Listening::of`]), so a round
    /// waits and accepts holding no lock at all, and a listener whose interface
    /// went while a round held a handle onto it closes when that round ends.
    held: Mutex<Vec<Arc<Listener>>>,
    /// The listeners that do not change: the IPv6-only one on a machine that
    /// bound its own, the one held to nothing on a machine that could not read
    /// its networks, and the one a test handed in.
    fixed: Vec<Arc<Listener>>,
    /// The interfaces of the networks the port was refused on, by number —
    /// each said in the service log once, when it was first refused.
    ///
    /// Locked only while [`held`](Self::held) is, and after it.
    refused: Mutex<BTreeSet<u32>>,
    /// The socket the kernel says a TCP socket at the port was destroyed on,
    /// when it opened (`crate::told_of_a_port_let_go`).
    told: Option<OwnedFd>,
    /// Whether [`held`](Self::held) follows the kernel's networks.
    follows: bool,
    /// The port every one of them is bound at, which is the port presence
    /// advertises.
    port: u16,
}

impl Listeners {
    /// Listen at `port` on every network this machine is on, held to each, and
    /// over IPv6 beside them.
    ///
    /// Every failure on the way — the kernel's interfaces unreadable, one
    /// network refusing the bind, IPv6 left out of the kernel — is a line handed
    /// to `said` and never a refusal to start. Listening **nowhere at all** is
    /// the one refusal: a machine that advertises a port nothing answers on is a
    /// machine that lies about itself.
    ///
    /// # Errors
    ///
    /// [`NotBound::NoWire`] when nothing could be listened on, in either family.
    pub fn bound(port: u16, said: &mut dyn FnMut(&str)) -> Result<Self, NotBound> {
        let mut fixed = Vec::new();
        let mut refused = None;
        match crate::unix::an_ipv6_only_listener_on(port) {
            Ok(listener) => fixed.push(Arc::new(Listener {
                on: None,
                arrived: None,
                listener,
            })),
            Err(why) => {
                said(&format!(
                    "the port presence advertises could not be bound over IPv6 ({why}); it is bound over IPv4 alone, and a machine on a network with no IPv4 address cannot reach this one"
                ));
                refused = Some(why);
            }
        }
        let reported = reported_by_the_kernel();
        if let Err(why) = &reported {
            said(&format!(
                "this machine's networks could not be read ({why}); the port presence advertises is bound once and held to no network, so it answers a handshake by the route and a machine on a network the route does not point at cannot reach this one"
            ));
            match held_to(port, None) {
                Ok(listener) => fixed.push(Arc::new(Listener {
                    on: None,
                    arrived: None,
                    listener,
                })),
                Err(why) => refused = Some(why),
            }
        }
        // Before the first bind, so a program letting go between a refusal and
        // the subscription is still heard.
        let told = match &reported {
            Ok(_) => match told_when_let_go_of(port) {
                Ok(told) => Some(told),
                Err(why) => {
                    said(&format!(
                        "the kernel will not say when a program lets go of the port presence advertises ({why}); a network where another program holds it is tried again only when this machine's networks next change"
                    ));
                    None
                }
            },
            Err(_) => None,
        };
        let listeners = Self {
            held: Mutex::new(Vec::new()),
            refused: Mutex::new(BTreeSet::new()),
            told,
            fixed,
            follows: reported.is_ok(),
            port,
        };
        if let Ok(reported) = reported {
            listeners.listen_on(&listening_networks(&reported), said);
        }
        if listeners.listening().is_empty() {
            return Err(NotBound::NoWire {
                what: NOTHING_IS_LISTENING,
                why: refused
                    .unwrap_or_else(|| std::io::Error::from(std::io::ErrorKind::AddrNotAvailable)),
            });
        }
        Ok(listeners)
    }

    /// This machine listening on a listener somebody else bound, for a test that
    /// puts two machines on one host.
    ///
    /// One listener, held to nothing and following nothing: what a connection on
    /// it arrived on is read off the connection, as every connection was read
    /// before there were held listeners.
    ///
    /// # Errors
    ///
    /// [`NotBound::NoWire`] when the listener will not say where it is bound.
    pub fn on(listener: TcpListener) -> Result<Self, NotBound> {
        let port = listener
            .local_addr()
            .map_err(|why| NotBound::NoWire {
                what: NOTHING_IS_LISTENING,
                why,
            })?
            .port();
        Ok(Self {
            held: Mutex::new(Vec::new()),
            refused: Mutex::new(BTreeSet::new()),
            told: None,
            fixed: vec![Arc::new(Listener {
                on: None,
                arrived: None,
                listener,
            })],
            follows: false,
            port,
        })
    }

    /// The port every listener is bound at.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// The networks the port is listened on now, in the kernel's order — empty
    /// on a machine listening through a listener held to no network.
    #[must_use]
    pub fn listened_on(&self) -> Vec<Network> {
        self.listening()
            .iter()
            .filter_map(|listener| listener.on.clone())
            .collect()
    }

    /// The kernel said a network changed: listen on every network this machine
    /// is on now that is not listened on yet, and let go of the listeners whose
    /// interfaces have gone.
    ///
    /// Nothing this machine says moves — the same identity, port and workspace
    /// answer on every network — so this changes only where it can be reached. A
    /// failure is a line in the service log.
    pub fn changed(&self, said: &mut dyn FnMut(&str)) {
        if !self.follows {
            return;
        }
        match reported_by_the_kernel() {
            Ok(reported) => self.listen_on(&listening_networks(&reported), said),
            Err(why) => said(&format!(
                "this machine's networks could not be read ({why}); the port stays listened on where it was"
            )),
        }
    }

    /// What to wait on for the kernel saying a TCP socket at the port was
    /// destroyed, when it can.
    #[must_use]
    pub fn let_go_waiting_on(&self) -> Option<BorrowedFd<'_>> {
        self.told.as_ref().map(AsFd::as_fd)
    }

    /// The kernel said a TCP socket at the port was destroyed: empty what it
    /// said, and try again every network the port was refused on.
    ///
    /// What was destroyed may be a connection the wire itself closed, or one
    /// of the other program's while it goes on holding the port, so a network
    /// still refused is still refused — and not said again. With nothing
    /// refused, nothing is asked of the kernel at all.
    pub fn let_go_of(&self, said: &mut dyn FnMut(&str)) {
        let Some(told) = &self.told else {
            return;
        };
        if let Err(why) = crate::unix::emptied(told.as_fd(), &mut |_| {}) {
            said(&format!(
                "what the kernel said about the port presence advertises being let go of could not be read ({why}); every network it was refused on is tried again anyway"
            ));
        }
        if self
            .refused
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_empty()
        {
            return;
        }
        self.changed(said);
    }

    /// A handle onto each listener, in the order they are waited on and
    /// accepted from.
    ///
    /// The lock is taken for exactly this call, so nothing a round then does
    /// while it waits — which may be for ever — is done holding it.
    fn listening(&self) -> Vec<Arc<Listener>> {
        let held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        held.iter().chain(&self.fixed).map(Arc::clone).collect()
    }

    /// Listen on every one of `networks` not listened on already, and let go of
    /// the listeners whose networks are not among them.
    ///
    /// A network refused is said once, when it is first refused; one that binds
    /// after being refused is said once, when it binds; one that goes is
    /// forgotten, so a network that comes back and is refused is said again.
    fn listen_on(&self, networks: &[Network], said: &mut dyn FnMut(&str)) {
        let on = |listener: &Arc<Listener>| listener.on.as_ref().map(Network::index);
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let mut refused = self.refused.lock().unwrap_or_else(PoisonError::into_inner);
        held.retain(|listener| {
            on(listener)
                .is_some_and(|index| networks.iter().any(|network| network.index() == index))
        });
        refused.retain(|index| networks.iter().any(|network| network.index() == *index));
        for network in networks {
            if held
                .iter()
                .any(|listener| on(listener) == Some(network.index()))
            {
                continue;
            }
            match held_to(self.port, NonZeroU32::new(network.index())) {
                Ok(listener) => {
                    held.push(Arc::new(Listener {
                        on: Some(network.clone()),
                        arrived: Some(what_a_listener_held_to(network)),
                        listener,
                    }));
                    if refused.remove(&network.index()) {
                        said(&format!(
                            "the port presence advertises is bound on {} ({}) now that nothing else holds it there; a machine on that network can reach this one",
                            network.name(),
                            network.address()
                        ));
                    }
                }
                // A network that will not be listened on is a line, and the
                // others are still listened on: a machine reachable on two of
                // its three networks beats a service that would not start.
                Err(why) => {
                    if refused.insert(network.index()) {
                        said(&format!(
                            "the port presence advertises could not be bound on {} ({}): {why}; {}",
                            network.name(),
                            network.address(),
                            self.until()
                        ));
                    }
                }
            }
        }
    }

    /// What the service log says about when a refused network is tried again —
    /// which depends on whether the kernel will say the port was let go of.
    const fn until(&self) -> &'static str {
        if self.told.is_some() {
            "a machine on that network cannot reach this one until it can be, and it is tried again as soon as the kernel says a program let go of the port"
        } else {
            "a machine on that network cannot reach this one until it can be, and it is tried again only when this machine's networks next change"
        }
    }
}

/// The listeners of one round of the service: what to wait on, and what to
/// accept from once the waiting says somebody is there.
#[derive(Debug)]
pub struct Listening {
    /// A handle onto each listener, in the order they are waited on.
    listeners: Vec<Arc<Listener>>,
}

impl Listening {
    /// The listeners as they stand, for one round.
    #[must_use]
    pub fn of(listeners: &Listeners) -> Self {
        Self {
            listeners: listeners.listening(),
        }
    }

    /// What to wait on for a message on the port: one for each listener.
    #[must_use]
    pub fn waiting_on(&self) -> Vec<BorrowedFd<'_>> {
        self.listeners
            .iter()
            .map(|listener| listener.listener.as_fd())
            .collect()
    }

    /// Accept one connection from the first listener `ready` says has one, and
    /// read what it carries.
    ///
    /// `ready` is what [`waiting_on`](Self::waiting_on) answered with, in the
    /// same order. One connection and not one per listener that spoke: a round
    /// that read every listener would hold every other door while it did, and
    /// the connections not accepted wait in the kernel's backlog and are ready
    /// again at once.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] when the listener will not accept, the
    /// connection will not take its timeouts, or nothing was ready at all — each
    /// of them the machine's, and a reason to stop rather than spin.
    pub fn accept_one(&self, ready: &[bool]) -> Result<Knocked, NotNearby> {
        ready
            .iter()
            .position(|said| *said)
            .and_then(|which| self.listeners.get(which))
            .ok_or_else(|| {
                NotNearby::TheNetwork("nothing was listening where a message arrived".to_owned())
            })?
            .accept_one()
    }

    /// Accept one connection from the listener bound first, for a wire on one
    /// listener somebody handed in.
    ///
    /// # Errors
    ///
    /// As [`accept_one`](Self::accept_one).
    pub fn accept_the_first(&self) -> Result<Knocked, NotNearby> {
        self.accept_one(&[true])
    }
}

/// An IPv4 listener at `port` on every address, held to the interface the kernel
/// numbers `interface` — or to nothing, where there is no interface to hold it
/// to.
///
/// The one thing `std` cannot do here, and the same option `crate::looking`
/// sets on a datagram socket: `SO_BINDTOIFINDEX`, through `socket2`. The kernel
/// then gives this listener only the connections that arrived on that interface
/// and — which is what the port was changed for — answers their handshakes out
/// of it.
///
/// Held **before** it binds, which is what lets two listeners on `0.0.0.0` at
/// one port coexist: the kernel's bind-conflict check treats a different
/// `sk_bound_dev_if` as a different binding (`docs/quirks.md`).
///
/// # Errors
///
/// What the kernel answered: an interface that went away since the networks were
/// read, `EADDRINUSE` from a listener at the same port held to nothing, and
/// everything a bind can fail with.
fn held_to(port: u16, interface: Option<NonZeroU32>) -> Result<TcpListener, std::io::Error> {
    /// As many connections as wait to be accepted as `std` lets wait.
    const WAITING: i32 = 128;
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?;
    socket.bind_device_by_index_v4(interface)?;
    socket.set_reuse_address(true)?;
    let at: SocketAddr = (Ipv4Addr::UNSPECIFIED, port).into();
    socket.bind(&at.into())?;
    socket.listen(WAITING)?;
    Ok(socket.into())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::io::Write as _;
    use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
    use std::num::NonZeroU32;
    use std::os::fd::BorrowedFd;
    use std::time::Duration;

    use super::{Listeners, Listening, held_to};
    use crate::arrived_on::ArrivedOn;
    use crate::networks::listening_networks;
    use crate::route_messages::reported_by_the_kernel;
    use crate::unix::{ready, ready_and};

    /// A port nothing else on this machine is on.
    fn a_free_port() -> u16 {
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    /// Wait on every listener and accept the one connection that is there.
    fn a_knock(listening: &Listening) -> crate::wire::Knocked {
        let nothing: [Option<BorrowedFd<'_>>; 0] = [];
        let waiting_on = listening.waiting_on();
        let (_, ready) = ready_and(&nothing, &waiting_on, Some(Duration::from_secs(10))).unwrap();
        listening.accept_one(&ready).unwrap()
    }

    /// **The port is listened on once per network, held to each** — and
    /// loopback is one of them, so a connection to `127.0.0.1` is answered, and
    /// is measured as its own network.
    #[test]
    fn the_port_is_listened_on_once_per_network_and_loopback_is_one() {
        let mut said = Vec::new();
        let port = a_free_port();
        let listeners = Listeners::bound(port, &mut |line| said.push(line.to_owned())).unwrap();
        let listened = listeners.listened_on();
        assert!(
            listened
                .iter()
                .any(|network| network.address().is_loopback()),
            "loopback is not listened on: {listened:?} {said:?}"
        );
        assert_eq!(listeners.port(), port);

        let listening = Listening::of(&listeners);
        // One per network, and the IPv6-only one beside them.
        assert_eq!(listening.waiting_on().len(), listened.len() + 1);

        let at = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let mut client = TcpStream::connect(at).unwrap();
        client
            .write_all(alo_nearby::http::a_request("/somewhere", "h", "one line\n").as_bytes())
            .unwrap();
        let knocked = a_knock(&listening);
        assert_eq!(knocked.arrived, ArrivedOn::ItsOwnNetwork, "{knocked:?}");
        assert!(knocked.from.ip().is_loopback());
        assert_eq!(knocked.message.unwrap().body, "one line\n");
    }

    /// **A listener held to nothing beside the held ones is refused**, which is
    /// why the wire binds held listeners and an IPv6-only one rather than
    /// keeping one listener in both families beside them.
    #[test]
    fn an_unheld_listener_beside_the_held_ones_is_refused() {
        let port = a_free_port();
        let listeners = Listeners::bound(port, &mut |_| {}).unwrap();
        assert!(!listeners.listened_on().is_empty());
        assert_eq!(
            held_to(port, None).unwrap_err().kind(),
            std::io::ErrorKind::AddrInUse,
            "an unheld listener bound beside the held ones"
        );
    }

    /// **A wire on a listener somebody handed in listens on that one alone**,
    /// follows no kernel, and reads what a connection arrived on off the
    /// connection.
    #[test]
    fn a_listener_handed_in_is_the_only_one() {
        let handed = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = handed.local_addr().unwrap();
        let listeners = Listeners::on(handed).unwrap();
        assert_eq!(listeners.port(), at.port());
        assert!(listeners.listened_on().is_empty());
        listeners.changed(&mut |line| panic!("a handed-in listener followed the kernel: {line}"));

        let listening = Listening::of(&listeners);
        assert_eq!(listening.waiting_on().len(), 1);
        let mut client = TcpStream::connect(at).unwrap();
        client
            .write_all(alo_nearby::http::a_request("/somewhere", "h", "a line\n").as_bytes())
            .unwrap();
        let knocked = listening.accept_the_first().unwrap();
        assert_eq!(
            knocked.arrived,
            crate::arrived_on::the_network_it_arrived_on(&knocked.stream)
        );
        assert_eq!(knocked.arrived, ArrivedOn::ItsOwnNetwork);
        assert!(knocked.from.ip().is_loopback());
    }

    /// **Nothing ready is not a connection**: a round that accepted where
    /// nothing spoke would hold the whole service on an accept nobody answers,
    /// so it is refused instead.
    #[test]
    fn accepting_where_nothing_was_ready_is_refused() {
        let handed = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let listeners = Listeners::on(handed).unwrap();
        let listening = Listening::of(&listeners);
        assert!(listening.accept_one(&[false]).is_err());
        assert!(listening.accept_one(&[]).is_err());
    }

    /// **A network that goes is let go of and a network that comes back is
    /// listened on again**, which is what following the kernel is for: the
    /// listeners here are asked to follow a list rather than the kernel, so the
    /// rule is a test with no cable in it.
    #[test]
    fn a_network_that_goes_is_let_go_of_and_one_that_comes_is_listened_on() {
        let port = a_free_port();
        let listeners = Listeners::bound(port, &mut |_| {}).unwrap();
        let networks = listeners.listened_on();
        assert!(!networks.is_empty());

        listeners.listen_on(&[], &mut |line| panic!("{line}"));
        assert!(
            listeners.listened_on().is_empty(),
            "a network that went is still listened on"
        );
        // The port is free again for a listener held to nothing, which is the
        // proof the held ones really let go.
        drop(held_to(port, None).unwrap());

        listeners.listen_on(&networks, &mut |line| panic!("{line}"));
        assert_eq!(listeners.listened_on(), networks);
    }

    /// **A network that will not bind is a line in the service log, and the
    /// others are still bound** — never a stopped service. What refuses here is
    /// what really refuses on a machine: somebody else already holding the port
    /// on one network (`EADDRINUSE`), which is loopback because loopback is the
    /// one network every host a test runs on has.
    ///
    /// An interface that is simply not there does **not** refuse:
    /// `SO_BINDTOIFINDEX` takes an index no interface has (`docs/quirks.md`), so
    /// a network that went between the kernel's report and the bind leaves a
    /// listener nothing reaches, let go of at the next notification.
    #[test]
    fn a_network_that_will_not_bind_is_a_line_and_the_others_are_still_bound() {
        let port = a_free_port();
        let reported = reported_by_the_kernel().unwrap();
        let every = listening_networks(&reported);
        let loopback = every
            .iter()
            .find(|network| network.address().is_loopback())
            .unwrap()
            .clone();
        let somebody_else = held_to(port, NonZeroU32::new(loopback.index())).unwrap();

        let mut said = Vec::new();
        let listeners = Listeners::bound(port, &mut |line| said.push(line.to_owned()))
            .unwrap_or_else(|why| panic!("one network refusing stopped the service: {why:?}"));
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(
            said.first()
                .is_some_and(|line| line.contains(loopback.name()) && line.contains("in use")),
            "the line does not name the network and why: {said:?}"
        );
        let others: Vec<_> = every
            .into_iter()
            .filter(|network| network.index() != loopback.index())
            .collect();
        assert_eq!(
            listeners.listened_on(),
            others,
            "a network that would not bind cost the others their listeners"
        );

        // Tried again on a notification while somebody else still holds it, it
        // is still refused, and not said a second time.
        listeners.changed(&mut |line| panic!("a refusal was said twice: {line}"));
        assert!(
            !listeners
                .listened_on()
                .iter()
                .any(|network| network.index() == loopback.index())
        );

        // Once somebody else lets go, the kernel says so with no network
        // changing, the port is listened on there, and that is said once.
        drop(somebody_else);
        heard_let_go(&listeners);
        let mut bound = Vec::new();
        listeners.let_go_of(&mut |line| bound.push(line.to_owned()));
        assert_eq!(bound.len(), 1, "{bound:?}");
        assert!(
            bound.first().is_some_and(|line| line.starts_with(&format!(
                "the port presence advertises is bound on {} (",
                loopback.name()
            ))),
            "{bound:?}"
        );
        listeners.changed(&mut |line| panic!("a bind was said twice: {line}"));
        assert!(
            listeners
                .listened_on()
                .iter()
                .any(|network| network.index() == loopback.index()),
            "loopback was not listened on once it was free"
        );
        let listening = Listening::of(&listeners);
        let mut client = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        client
            .write_all(alo_nearby::http::a_request("/somewhere", "h", "still\n").as_bytes())
            .unwrap();
        let knocked = a_knock(&listening);
        assert_eq!(knocked.arrived, ArrivedOn::ItsOwnNetwork);
        assert_eq!(knocked.message.unwrap().body, "still\n");
    }

    /// Wait until the kernel says a TCP socket at the listeners' port was
    /// destroyed, failing if it says nothing.
    fn heard_let_go(listeners: &Listeners) {
        let told = [listeners.let_go_waiting_on()];
        assert!(told[0].is_some(), "the listeners hear nothing let go of");
        let said = ready(&told, Some(Duration::from_secs(5))).unwrap();
        assert_eq!(said, [true], "the kernel never said the port was let go of");
    }

    /// **Hearing is not proof**: a connection at the port closing while
    /// somebody else's listener goes on holding it is heard, tried, still
    /// refused — and nothing more is said.
    #[test]
    fn a_let_go_that_leaves_the_port_held_is_still_refused_and_not_said_again() {
        let port = a_free_port();
        let reported = reported_by_the_kernel().unwrap();
        let loopback = listening_networks(&reported)
            .into_iter()
            .find(|network| network.address().is_loopback())
            .unwrap();
        let somebody_else = held_to(port, NonZeroU32::new(loopback.index())).unwrap();
        let mut said = Vec::new();
        let listeners = Listeners::bound(port, &mut |line| said.push(line.to_owned())).unwrap();
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(
            said.first()
                .is_some_and(|line| line.contains("as soon as the kernel says a program let go")),
            "the refusal does not say when it is tried again: {said:?}"
        );

        let connection = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        let (accepted, _) = somebody_else.accept().unwrap();
        drop(accepted);
        drop(connection);
        heard_let_go(&listeners);
        listeners.let_go_of(&mut |line| panic!("a port still held was said: {line}"));
        assert!(
            !listeners
                .listened_on()
                .iter()
                .any(|network| network.index() == loopback.index()),
            "a port somebody else still holds was counted listened on"
        );
        drop(somebody_else);
    }

    /// **With nothing refused, a let-go is emptied and nothing else is done** —
    /// the wire's own connections closing are heard, and change nothing.
    #[test]
    fn a_let_go_with_nothing_refused_changes_nothing() {
        let port = a_free_port();
        let listeners = Listeners::bound(port, &mut |_| {}).unwrap();
        let before = listeners.listened_on();
        let listening = Listening::of(&listeners);
        let connection = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        let knocked = a_knock(&listening);
        drop(knocked);
        drop(connection);
        heard_let_go(&listeners);
        listeners.let_go_of(&mut |line| panic!("{line}"));
        assert_eq!(listeners.listened_on(), before);
        // And again, with what the kernel said already emptied: the socket does
        // not block, so a round woken for nothing returns rather than hangs.
        // Whether the kernel sends more after this is not asserted — it sends
        // these from a queue of its own, so a socket closed earlier (the one
        // `a_free_port` bound, say) may be said late.
        listeners.let_go_of(&mut |line| panic!("{line}"));
        assert_eq!(listeners.listened_on(), before);
    }

    /// **A wire on a listener handed in hears nothing let go of**: it follows
    /// no kernel, and has no network to try again.
    #[test]
    fn a_listener_handed_in_hears_nothing_let_go_of() {
        let listeners =
            Listeners::on(TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap()).unwrap();
        assert!(listeners.let_go_waiting_on().is_none());
        listeners.let_go_of(&mut |line| panic!("{line}"));
    }
}
