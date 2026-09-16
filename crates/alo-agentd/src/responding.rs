//! Discovery answered once per network this machine is on — each socket held to
//! that network's interface, so an answer leaves on the network its question
//! arrived on.
//!
//! *Machines find each other with zero configuration, and trust none of them for
//! it.* `crate::listeners` made this machine **reachable** on a network the route
//! does not point at: one held listener per network, and the kernel answers each
//! handshake out of the interface it came in on. What still left by the route was
//! the other half of being found — **the answer to *who is here***. Until this
//! file `crate::wire` answered discovery on one socket per family held to no
//! network, and `alo_nearby::Answering::answer_one` replies to the asking
//! machine's unicast address: on a machine whose two routers hand out
//! `192.168.1.0/24`, the answer to the studio at `192.168.1.20` on the cable went
//! out whichever interface the route picked — to somebody else at that address,
//! or to nobody. The studio never found this machine and could not propose to it
//! at all, which is the step the promise removes.
//!
//! # How an answer is held, and why this reading
//!
//! Three readings the kernel really gives were available:
//!
//! - **One datagram socket per network, held to its interface** — this file.
//!   `SO_BINDTOIFINDEX` before the bind, as `crate::looking::held_to` holds a
//!   socket for asking and `crate::listeners::held_to` one for listening, and
//!   each socket joined to the group on its own network. The kernel then hands a
//!   question only to the socket held to the interface it arrived on, and the
//!   answer that socket sends leaves by that interface whatever the route says.
//!   Measured on this kernel (`docs/quirks.md`, 2026-09-16).
//! - **`IP_PKTINFO` read back with `recvmsg` and answered with `sendmsg`**, which
//!   is how a general-purpose responder does it: one socket per family, the
//!   arriving interface read off each datagram. It is the more exact reading —
//!   it needs no list of interfaces at all, so it would answer correctly even on
//!   a machine whose own interfaces cannot be read. **It is not available here.**
//!   Control messages are `cmsghdr` bytes: `rustix` has no `IP_PKTINFO`
//!   ancillary message, `socket2` hands out the control buffer and nothing that
//!   parses it, and building one by hand means laying out a kernel structure out
//!   of raw bytes — `CLAUDE.md` forbids `unsafe`, and a hand-laid `cmsghdr` is
//!   the shape of thing that law exists for. It is a change to how a datagram is
//!   read that a later task can make on its own merits, behind a crate that does
//!   it safely.
//! - **Letting the answer leave by the route**, which is what the code did
//!   before and is exactly what ADR 0044 refuses: the route is not the network
//!   the question came in on, and on two networks with one private range it is
//!   the wrong machine.
//!
//! # What the sockets cost, network by network
//!
//! - **Loopback.** A question to `127.0.0.1` is answered today — by the person's
//!   own machine, and by every test that puts two machines on one host — so
//!   loopback is one of the networks answered on, exactly as it is one of
//!   [`listening_networks`]. What it does **not** get is a join: the group is
//!   never joined there, because a multicast question on loopback reaches this
//!   machine only. So a loopback responder answers the unicast questions that
//!   arrive at `127.0.0.1`, and nothing else.
//! - **An interface that carries no multicast** — a point-to-point tunnel — is
//!   answered on for the same reason and joined for none. A machine that dials
//!   this one's discovery port across it is answered; nobody is found by asking a
//!   group that does not travel.
//! - **Over IPv6 nothing is held.** A question over IPv6 discovery is a
//!   link-local one (`ff02::fb`, `crate::networks::link_local_networks`), and a
//!   link-local address carries the interface it was heard on in its own scope
//!   (ADR 0041): the kernel sends the answer back out that interface without
//!   being told to. So the IPv6 responder is one socket held to nothing, joined
//!   per link-local network by `crate::joining`, and what it answers is held by
//!   the address it answers to. What that leaves open is a question from a
//!   **global** IPv6 address arriving on one of two interfaces, whose answer
//!   would go by the route; discovery never asks from one, and a machine that
//!   does is not a machine this one found.
//!
//! # A question whose network cannot be read is answered on no network
//!
//! An interface the kernel numbers **zero** is no interface, and a socket cannot
//! be held to it: that network takes no responder, the service log says so, and a
//! question arriving there is answered by nobody. A machine whose interfaces
//! cannot be read at all takes **no responders at all** and says so — it is not
//! found until it can read them.
//!
//! This is deliberately **not** what `crate::listeners` does, which binds one
//! listener held to nothing rather than listening nowhere, and the two differ
//! because the failures differ. An unheld listener that answers a handshake by
//! the route merely fails to complete it: the machine is unreachable, and nobody
//! is told anything untrue. An unheld **answer** is a datagram saying *this
//! machine is here* delivered to a machine that did not ask, on a network this
//! machine may not be answering for at all — while the machine that did ask waits
//! and finds nothing either way. Silence is the true thing to say when this
//! machine cannot tell which network it would be speaking on.
//!
//! # The responders follow the kernel, as the listeners and the joins do
//!
//! A laptop docked an hour after it started would otherwise be found on the Wi-Fi
//! all day and be silent on the wired network it was plugged into.
//! [`Responders::changed`] is called from `crate::wire::Wire::networks_changed`
//! on the same notification `crate::joining` follows: every network not answered
//! on yet takes a socket, and a network that has gone is let go of. **A network
//! that will not take a socket is a line in the service log and the others still
//! answer** — a machine found on two of its three networks beats a service that
//! would not start.
//!
//! **And the thread that answers is told.** The responders are waited on by
//! `crate::answering_discovery`'s thread, not by the service's, and that thread
//! cannot read the kernel's notification itself — the service already does, and
//! two readers of one socket take each other's messages. So whenever the set of
//! responders moves, it says so ([`crate::told_of_a_move`]), and the thread takes
//! the responders again. Without it, a thread asleep on the sockets it had before
//! a cable was plugged in never waits on the new one, and a question on that cable
//! is answered only once somebody on another network happens to ask — measured by
//! `crate::a_cable_pulled_and_plugged_in_again`, which failed until this was so.
//!
//! # A number is not an interface
//!
//! A responder is matched to what the kernel reports by its interface's
//! **number**, so an address changing on an interface — a lease renewed — leaves
//! the socket where it is. But the kernel gives a free number to the next
//! interface that asks, and a cable deleted and laid again at the same number
//! **between two readings of the interfaces** reads the same in both. The socket
//! held to that number is still good — `SO_BINDTOIFINDEX` is a number the kernel
//! compares with the interface a datagram arrived on — and its membership is not:
//! the kernel took the deleted interface out of the group and left the socket's
//! record of the membership behind (`docs/quirks.md`). A responder kept for its
//! number answers questions that never reach it, and the machine is not found on
//! that network until the service restarts.
//!
//! So a round also reads **what the kernel said went**
//! ([`crate::interfaces_that_went`]), and a responder whose interface it said was
//! deleted is let go of **whether or not its number is reported again**; where it
//! is, the network is answered on by a new socket joined afresh, exactly as a
//! network plugged in is. Two things make that safe rather than merely likely:
//!
//! - **A responder let go of leaves the group then and there**, and does not wait
//!   for its socket to close. The kernel counts memberships per interface, and a
//!   socket closing leaves the group on whatever interface has its number *by
//!   then* — which, a moment after a cable is re-laid, is the new interface the
//!   new socket has just joined. The thread answering discovery may hold the old
//!   socket open for the rest of its round, so waiting for the close would take
//!   the new membership with it.
//! - **A join refused `EADDRINUSE` is taken afresh** — left, and joined again —
//!   and never read as joined, as `crate::joining` has it over IPv6: the refusal
//!   says the socket holds a record at that number, and not that any interface
//!   is in the group behind it.
//!
//! What the kernel said and could not be read is every interface having gone:
//! every responder is let go of and answered again once, which costs one report
//! per network on a rare round and never leaves the machine counted joined where
//! it is not.
//!
//! The listeners need none of this (`crate::listeners`): a TCP listener held to a
//! number is reached on whatever interface bears it, and joins nothing.
//! `crate::a_cable_re_laid_between_two_readings` is the measurement.
//!
//! # Nothing here is a setting, and nothing it says moves
//!
//! ADR 0003. Which interfaces are answered on is what the machine is plugged
//! into, read from the kernel; no person and no agent names one. What is said is
//! one [`Presence`] and one workspace, made once and answered with on every
//! network and in both families — a machine that said something different on one
//! network would be two machines to whoever asked on both.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::num::{NonZeroU16, NonZeroU32};
use std::os::fd::{AsFd as _, BorrowedFd};
use std::sync::{Arc, Mutex, PoisonError};

use alo_nearby::{Answering, NotNearby, Presence, THE_ADDRESS};

use crate::interfaces_that_went::Went;
use crate::networks::{Interface, Network, discovery_networks, listening_networks};
use crate::route_messages::reported_by_the_kernel;
use crate::told_of_a_move::ToldOfAMove;

/// One socket discovery is answered on, and what it is held to.
struct Responder {
    /// The network it is held to, and nothing for one held to none — the IPv6
    /// socket, and a socket a test handed in.
    on: Option<Network>,
    /// Whether the discovery group is joined on it, which is how a question
    /// asked of everybody reaches this machine. False on loopback, on an
    /// interface that carries no multicast, and on a socket held to nothing.
    joined: bool,
    /// A second handle onto the socket [`Answering`] owns, so that a round can
    /// wait on it without taking the answering apart.
    socket: UdpSocket,
    /// This machine, answering on that socket.
    answering: Answering,
}

impl Responder {
    /// Leave the group this responder joined — now, rather than when the last
    /// handle onto its socket closes, by which time its interface's number may
    /// belong to an interface a new responder has joined.
    fn leave(&self) {
        if self.joined
            && let Some(IpAddr::V4(address)) = self.on.as_ref().map(Network::address)
        {
            left(&self.socket, address);
        }
    }
}

impl std::fmt::Debug for Responder {
    /// Written by hand because [`Answering`] prints nothing of itself.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Responder")
            .field("on", &self.on)
            .field("joined", &self.joined)
            .finish_non_exhaustive()
    }
}

/// Every socket discovery is answered on, and what this machine answers with.
#[derive(Debug)]
pub struct Responders {
    /// The IPv4 responders, one per network this machine is on, each held to
    /// that network's interface — and empty on a machine answering through
    /// [`fixed`](Self::fixed) alone.
    ///
    /// **Locked for exactly as long as it takes to read or change it.** What is
    /// taken out of it is a handle onto each responder ([`Responding::of`]), so a
    /// round waits and answers holding no lock at all, and a socket whose
    /// interface went while a round held a handle onto it closes when that round
    /// ends.
    held: Mutex<Vec<Arc<Responder>>>,
    /// The responders that do not change: the IPv6 one on a machine that bound
    /// its own, and the one a test handed in.
    fixed: Vec<Arc<Responder>>,
    /// What this machine says about itself, which does not change.
    presence: Presence,
    /// The workspace it says it hosts, if it hosts one — kept so that a network
    /// answered on later says the same thing as the ones answered on at start.
    workspace: Option<NonZeroU16>,
    /// Whether [`held`](Self::held) follows the kernel's networks.
    follows: bool,
    /// Said whenever [`held`](Self::held) moves, so the thread waiting on the
    /// responders takes them again — and nothing on a machine that follows no
    /// kernel, or could not make the pair.
    moved: Option<ToldOfAMove>,
    /// The port every one of them is bound at, which is where discovery asks.
    port: u16,
}

impl Responders {
    /// Answer discovery at `port` on every network this machine is on, held to
    /// each, with `presence`.
    ///
    /// Every failure on the way is a line handed to `said` and never a refusal to
    /// start: a machine that cannot read its networks, and a network that will
    /// not take a socket, are each answered on no network rather than by the
    /// route — see this module's documentation for why that is silence here where
    /// `crate::listeners` binds a listener held to nothing.
    #[must_use]
    pub fn bound(port: u16, presence: Presence, said: &mut dyn FnMut(&str)) -> Self {
        let reported = reported_by_the_kernel();
        let moved = match reported.as_ref().map(|_| ToldOfAMove::made()) {
            Ok(Ok(moved)) => Some(moved),
            Ok(Err(why)) => {
                said(&format!(
                    "the thread answering discovery cannot be told when a network changes ({why}); a network plugged in after start is answered on only once a question arrives on another"
                ));
                None
            }
            Err(_) => None,
        };
        let responders = Self {
            held: Mutex::new(Vec::new()),
            fixed: Vec::new(),
            presence,
            workspace: None,
            follows: reported.is_ok(),
            moved,
            port,
        };
        match reported {
            Ok(reported) => responders.answer_on_what_was(&reported, &Went::nothing(), said),
            Err(why) => said(&format!(
                "this machine's networks could not be read ({why}); discovery is answered on no network at all, so this machine is not found until they can be — an answer held to no network leaves by the route, which on a machine on two networks reaches whoever is at that address on the other one"
            )),
        }
        responders
    }

    /// This machine answering on a socket somebody else bound, for a test that
    /// puts two machines on one host.
    ///
    /// One responder, held to nothing and following nothing: what it answers goes
    /// wherever the socket it was handed sends it, which is that caller's
    /// decision and not this machine's.
    ///
    /// # Errors
    ///
    /// What the kernel said when the socket could not be waited on beside the
    /// answering.
    pub fn on(socket: UdpSocket, presence: Presence) -> Result<Self, std::io::Error> {
        let port = socket.local_addr().map(|at| at.port()).unwrap_or_default();
        let responders = Self {
            held: Mutex::new(Vec::new()),
            fixed: Vec::new(),
            presence,
            workspace: None,
            follows: false,
            moved: None,
            port,
        };
        let fixed = responders.a_responder_on(socket, None, false)?;
        Ok(Self {
            fixed: vec![Arc::new(fixed)],
            ..responders
        })
    }

    /// The same machine, also answering on `socket` — the IPv6 one, held to
    /// nothing because a link-local address carries its own network.
    ///
    /// # Errors
    ///
    /// As [`on`](Self::on).
    pub fn also_answering_on(self, socket: UdpSocket) -> Result<Self, std::io::Error> {
        let one = self.a_responder_on(socket, None, false)?;
        let mut fixed = self.fixed;
        fixed.push(Arc::new(one));
        Ok(Self { fixed, ..self })
    }

    /// What this machine says about itself.
    #[must_use]
    pub const fn presence(&self) -> &Presence {
        &self.presence
    }

    /// The workspace this machine answers for, if it answers for one.
    #[must_use]
    pub const fn workspace(&self) -> Option<NonZeroU16> {
        self.workspace
    }

    /// The same machine, also answering that it hosts a workspace at `port` — on
    /// every network it answers on now and on every one it answers on later.
    ///
    /// Taken by value, before the service is handed the wire: what is advertised
    /// is decided once, at the start, and nothing a running service does can move
    /// it. A responder whose socket cannot be taken a second time is let go of
    /// with a line rather than left answering something other than what this
    /// machine advertises.
    #[must_use]
    pub fn hosting(self, port: NonZeroU16, said: &mut dyn FnMut(&str)) -> Self {
        let told = Self {
            workspace: Some(port),
            held: Mutex::new(Vec::new()),
            fixed: Vec::new(),
            presence: self.presence.clone(),
            follows: self.follows,
            moved: None,
            port: self.port,
        };
        let held = {
            let held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
            told.told_of(&held, said)
        };
        let fixed = told.told_of(&self.fixed, said);
        Self {
            held: Mutex::new(held),
            fixed,
            moved: self.moved,
            ..told
        }
    }

    /// Each of `responders`, answering what this one says on the socket it
    /// already holds.
    fn told_of(
        &self,
        responders: &[Arc<Responder>],
        said: &mut dyn FnMut(&str),
    ) -> Vec<Arc<Responder>> {
        responders
            .iter()
            .filter_map(|responder| {
                let told = responder.socket.try_clone().and_then(|socket| {
                    self.a_responder_on(socket, responder.on.clone(), responder.joined)
                });
                match told {
                    Ok(told) => Some(Arc::new(told)),
                    Err(why) => {
                        said(&format!(
                            "discovery could not be answered again on {} ({why}); this machine is not found there",
                            responder
                                .on
                                .as_ref()
                                .map_or("the socket it answers on", Network::name)
                        ));
                        None
                    }
                }
            })
            .collect()
    }

    /// The networks discovery is answered on now, in the kernel's order — empty
    /// on a machine answering through a socket held to no network.
    #[must_use]
    pub fn answered_on(&self) -> Vec<Network> {
        self.answering()
            .iter()
            .filter_map(|responder| responder.on.clone())
            .collect()
    }

    /// The networks the discovery group is joined on now, which are those of
    /// [`answered_on`](Self::answered_on) that carry multicast and are not
    /// loopback.
    #[must_use]
    pub fn joined(&self) -> Vec<Network> {
        self.answering()
            .iter()
            .filter(|responder| responder.joined)
            .filter_map(|responder| responder.on.clone())
            .collect()
    }

    /// What to wait on, and empty, for the responders having moved — nothing on a
    /// machine whose responders never move.
    #[must_use]
    pub const fn moved(&self) -> Option<&ToldOfAMove> {
        self.moved.as_ref()
    }

    /// The kernel said a network changed: answer on every network this machine is
    /// on now that is not answered on yet, and let go of the sockets whose
    /// interfaces have gone — those not reported, and those whose interface
    /// `went` says was deleted, whose number may be reported again on an
    /// interface nobody joined.
    ///
    /// Nothing this machine says moves — the same identity, port and workspace
    /// answer on every network — so this changes only where it is heard. A
    /// failure is a line in the service log.
    pub fn changed(&self, went: &Went, said: &mut dyn FnMut(&str)) {
        if !self.follows {
            return;
        }
        match reported_by_the_kernel() {
            Ok(reported) => self.answer_on_what_was(&reported, went, said),
            Err(why) => said(&format!(
                "this machine's networks could not be read ({why}); discovery stays answered where it was"
            )),
        }
    }

    /// A handle onto each responder, in the order they are waited on and answered
    /// from.
    ///
    /// The lock is taken for exactly this call, so nothing a round then does
    /// while it waits — which may be for ever — is done holding it.
    fn answering(&self) -> Vec<Arc<Responder>> {
        let held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        held.iter().chain(&self.fixed).map(Arc::clone).collect()
    }

    /// Answer on every network `reported` says this machine is on, joined to the
    /// group on those that carry one.
    fn answer_on_what_was(&self, reported: &[Interface], went: &Went, said: &mut dyn FnMut(&str)) {
        self.answer_on(
            &listening_networks(reported),
            &discovery_networks(reported),
            went,
            said,
        );
    }

    /// Answer on every one of `networks` not answered on already, joining the
    /// group on those that are also in `groups`, and let go of the sockets whose
    /// networks are not among them or whose interfaces `went` — each leaving its
    /// group as it is let go of.
    fn answer_on(
        &self,
        networks: &[Network],
        groups: &[Network],
        went: &Went,
        said: &mut dyn FnMut(&str),
    ) {
        let on = |responder: &Arc<Responder>| responder.on.as_ref().map(Network::index);
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let before = held.len();
        held.retain(|responder| {
            let stays = on(responder).is_some_and(|index| {
                !went.includes(index) && networks.iter().any(|network| network.index() == index)
            });
            if !stays {
                responder.leave();
            }
            stays
        });
        let mut moved = held.len() != before;
        for network in networks {
            if held
                .iter()
                .any(|responder| on(responder) == Some(network.index()))
            {
                continue;
            }
            match self.a_responder(network, groups) {
                Ok(responder) => {
                    held.push(Arc::new(responder));
                    moved = true;
                }
                // A network that will not take a socket is a line, and the others
                // are still answered on: a machine found on two of its three
                // networks beats a service that would not start.
                Err(why) => said(&format!(
                    "discovery could not be answered on {} ({}): {why}; this machine is not found on that network until it can be, and nothing answers there by the route in its place",
                    network.name(),
                    network.address()
                )),
            }
        }
        drop(held);
        if moved && let Some(Err(why)) = self.moved.as_ref().map(ToldOfAMove::tell) {
            said(&format!(
                "the thread answering discovery could not be told the networks changed ({why}); a network plugged in is answered on once a question arrives on another"
            ));
        }
    }

    /// A responder on `network`: a socket held to its interface, joined to the
    /// group where `groups` says that network carries one.
    fn a_responder(
        &self,
        network: &Network,
        groups: &[Network],
    ) -> Result<Responder, std::io::Error> {
        let interface = NonZeroU32::new(network.index()).ok_or_else(|| {
            std::io::Error::other("the kernel numbers this interface zero, which is no interface")
        })?;
        let socket = held_to(self.port, interface)?;
        let group = groups
            .iter()
            .find(|group| group.index() == network.index())
            .map(Network::address);
        if let Some(IpAddr::V4(address)) = group {
            joined(&socket, address)?;
        }
        self.a_responder_on(socket, Some(network.clone()), group.is_some())
    }

    /// A responder on `socket`, answering what this machine says.
    fn a_responder_on(
        &self,
        socket: UdpSocket,
        on: Option<Network>,
        joined: bool,
    ) -> Result<Responder, std::io::Error> {
        let waiting_on = socket.try_clone()?;
        let answering = Answering::on(socket, self.presence.clone());
        let answering = match self.workspace {
            Some(port) => answering.hosting_a_workspace_at(port),
            None => answering,
        };
        Ok(Responder {
            on,
            joined,
            socket: waiting_on,
            answering,
        })
    }
}

/// The responders of one round of the service: what to wait on, and what to
/// answer from once the waiting says a question is there.
#[derive(Debug)]
pub struct Responding {
    /// A handle onto each responder, in the order they are waited on.
    responders: Vec<Arc<Responder>>,
}

impl Responding {
    /// The responders as they stand, for one round.
    #[must_use]
    pub fn of(responders: &Responders) -> Self {
        Self {
            responders: responders.answering(),
        }
    }

    /// What to wait on for a discovery question: one for each responder.
    #[must_use]
    pub fn waiting_on(&self) -> Vec<BorrowedFd<'_>> {
        self.responders
            .iter()
            .map(|responder| responder.socket.as_fd())
            .collect()
    }

    /// Answer one question on the first responder `ready` says has one.
    ///
    /// `ready` is what [`waiting_on`](Self::waiting_on) answered with, in the
    /// same order. One question and not one per socket that spoke: a round that
    /// read every socket would hold the others while it did, and a datagram not
    /// read stays where it is and is ready again at once.
    ///
    /// Who was answered, or nothing at all — nothing was ready, or what arrived
    /// was not a question this machine answers.
    ///
    /// # Errors
    ///
    /// As `alo_nearby::Answering::answer_one`.
    pub fn answer_one(&self, ready: &[bool]) -> Result<Option<SocketAddr>, NotNearby> {
        let Some(responder) = ready
            .iter()
            .position(|said| *said)
            .and_then(|which| self.responders.get(which))
        else {
            return Ok(None);
        };
        responder.answering.answer_one()
    }

    /// Wait until one of the responders has a question, and answer it.
    ///
    /// For a caller that waits on nothing else — a test, and
    /// `crate::wire::Wire::answer_discovery`. A machine with one responder reads
    /// it directly, so a caller that set a read timeout on the socket it handed
    /// in gets that timeout back; a machine with several waits on all of them at
    /// once.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] when discovery is answered on no network at all,
    /// and as [`answer_one`](Self::answer_one) otherwise.
    pub fn answer_the_first_that_speaks(&self) -> Result<Option<SocketAddr>, NotNearby> {
        match self.responders.as_slice() {
            [] => Err(NotNearby::TheNetwork(
                "discovery is answered on no network at all".to_owned(),
            )),
            [one] => one.answering.answer_one(),
            _ => {
                let nothing: [Option<BorrowedFd<'_>>; 0] = [];
                let waiting_on = self.waiting_on();
                let (_, ready) = crate::unix::ready_and(&nothing, &waiting_on, None)
                    .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
                self.answer_one(&ready)
            }
        }
    }
}

/// An IPv4 datagram socket at `port` on every address, held to the interface the
/// kernel numbers `interface`.
///
/// The one thing `std` cannot do here, and the same option `crate::looking` and
/// `crate::listeners` set: `SO_BINDTOIFINDEX`, through `socket2`, **before** the
/// bind. The kernel then hands this socket only the datagrams that arrived on
/// that interface, and sends what it answers out of it whatever the route says.
///
/// # Errors
///
/// What the kernel answered: an interface that went away since the networks were
/// read, and everything a bind can fail with.
fn held_to(port: u16, interface: NonZeroU32) -> Result<UdpSocket, std::io::Error> {
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
    socket.bind_device_by_index_v4(Some(interface))?;
    // Shared with whatever else on this machine answers on the discovery port, as
    // the one socket before these was.
    socket.set_reuse_address(true)?;
    let at: SocketAddr = (Ipv4Addr::UNSPECIFIED, port).into();
    socket.bind(&at.into())?;
    Ok(socket.into())
}

/// Join `socket` to the discovery group on the network whose address is
/// `address`.
///
/// A membership the socket already holds there is **taken afresh** — left, and
/// joined again — and never read as joined, as `crate::joining` has it over
/// IPv6: `EADDRINUSE` says the socket holds a record at that interface's number,
/// and a record an interface deleted and laid again at that number left behind
/// has no interface in the group behind it.
///
/// # Errors
///
/// What the kernel answered the join with — the second one, where there were
/// two: no interface at that address, and everything a join can fail with.
pub(crate) fn joined(socket: &UdpSocket, address: Ipv4Addr) -> Result<(), std::io::Error> {
    match socket.join_multicast_v4(&THE_ADDRESS, &address) {
        Err(why) if why.kind() == std::io::ErrorKind::AddrInUse => {
            left(socket, address);
            socket.join_multicast_v4(&THE_ADDRESS, &address)
        }
        joined => joined,
    }
}

/// Leave the discovery group on the network whose address is `address`,
/// whether or not an interface is still there.
///
/// Nothing is said when the kernel refuses: its refusal is that the socket held
/// no membership there, and then there was nothing to leave.
fn left(socket: &UdpSocket, address: Ipv4Addr) {
    drop(socket.leave_multicast_v4(&THE_ADDRESS, &address));
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{Ipv4Addr, UdpSocket};
    use std::num::NonZeroU16;
    use std::time::Duration;

    use alo_nearby::advertising::{
        a_question, a_question_for_workspaces, about, about_a_workspace,
    };
    use alo_nearby::{MachineId, Presence, WorkspacePresence};

    use super::{Responders, Responding, joined, left};
    use crate::interfaces_that_went::Went;
    use crate::networks::{
        IFF_LOOPBACK, IFF_MULTICAST, IFF_RUNNING, IFF_UP, Interface, Network, listening_networks,
    };

    /// This machine, for these tests.
    fn here() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// What it says about itself.
    fn presence() -> Presence {
        Presence::of(here(), 7_610)
    }

    /// The port a test's workspace answers on.
    fn a_workspace() -> NonZeroU16 {
        NonZeroU16::new(8_443).unwrap()
    }

    /// A port nothing else on this machine is on.
    fn a_free_port() -> u16 {
        UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    /// A socket of a test's own, to ask questions from.
    fn asking() -> UdpSocket {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        socket
    }

    /// Ask `question` at `port` on loopback, answer it, and hand back every
    /// packet that came back within a short wait.
    fn asked(responders: &Responders, question: &[u8], port: u16) -> Vec<Vec<u8>> {
        let asking = asking();
        asking
            .send_to(question, (Ipv4Addr::LOCALHOST, port))
            .unwrap();
        let responding = Responding::of(responders);
        responding.answer_the_first_that_speaks().unwrap();
        let mut back = Vec::new();
        let mut heard = [0_u8; 1_500];
        while let Ok((how_many, _)) = asking.recv_from(&mut heard) {
            back.push(heard.get(..how_many).unwrap().to_vec());
        }
        back
    }

    /// An interface the kernel reports, up and carrying multicast.
    fn interface(index: u32, name: &str, addresses: &[Ipv4Addr]) -> Interface {
        Interface {
            index,
            name: name.to_owned(),
            flags: IFF_UP | IFF_RUNNING | IFF_MULTICAST,
            addresses: addresses.to_vec(),
            ipv6: Vec::new(),
        }
    }

    /// **Discovery is answered once per network, held to each — and loopback is
    /// one of them**, so a question to `127.0.0.1` is answered as it is today.
    /// The group is joined on the networks that carry multicast and not on
    /// loopback.
    #[test]
    fn discovery_is_answered_once_per_network_and_loopback_is_one() {
        let mut said = Vec::new();
        let port = a_free_port();
        let responders =
            Responders::bound(port, presence(), &mut |line| said.push(line.to_owned()));
        let answered = responders.answered_on();
        assert!(
            answered
                .iter()
                .any(|network| network.address().is_loopback()),
            "loopback is not answered on: {answered:?} {said:?}"
        );
        assert!(
            !responders
                .joined()
                .iter()
                .any(|network| network.address().is_loopback()),
            "the group was joined on loopback: {:?}",
            responders.joined()
        );
        assert_eq!(
            Responding::of(&responders).waiting_on().len(),
            answered.len()
        );

        let back = asked(&responders, &a_question().unwrap(), port);
        assert_eq!(back, vec![about(&presence()).unwrap()], "{said:?}");
    }

    /// **A network the kernel numbers zero takes no socket, is a line in the
    /// service log, and the others still answer** — a question that arrived
    /// there would be answered by nobody rather than by the route.
    #[test]
    fn a_network_that_will_not_take_a_socket_is_a_line_and_the_others_still_answer() {
        let port = a_free_port();
        let responders = Responders::bound(port, presence(), &mut |_| {});
        let loopback = listening_networks(&[Interface {
            index: 1,
            name: "lo".to_owned(),
            flags: IFF_UP | IFF_RUNNING | IFF_LOOPBACK,
            addresses: vec![Ipv4Addr::LOCALHOST],
            ipv6: Vec::new(),
        }]);
        let nowhere =
            listening_networks(&[interface(0, "nowhere", &[Ipv4Addr::new(10, 65, 0, 1)])]);
        let both: Vec<_> = nowhere.iter().chain(&loopback).cloned().collect();

        let mut said = Vec::new();
        responders.answer_on(&both, &[], &Went::nothing(), &mut |line| {
            said.push(line.to_owned())
        });
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(
            said.first().unwrap().contains("nowhere"),
            "{said:?} does not name the network that took no socket"
        );
        assert_eq!(
            responders
                .answered_on()
                .iter()
                .map(Network::index)
                .collect::<Vec<_>>(),
            loopback.iter().map(Network::index).collect::<Vec<_>>(),
            "{said:?}"
        );
        // And the one that did take a socket answers, which is the half that
        // matters: one refusal does not silence the machine.
        assert_eq!(
            asked(&responders, &a_question().unwrap(), port),
            vec![about(&presence()).unwrap()]
        );
    }

    /// **A machine whose networks cannot be read answers on no network at all**:
    /// nothing is bound, nothing is waited on, and asking one is refused rather
    /// than answered by the route.
    #[test]
    fn a_machine_with_no_networks_answers_on_none() {
        let responders = Responders::bound(a_free_port(), presence(), &mut |_| {});
        responders.answer_on(&[], &[], &Went::nothing(), &mut |line| panic!("{line}"));
        assert!(responders.answered_on().is_empty());
        assert!(Responding::of(&responders).waiting_on().is_empty());
        assert!(
            Responding::of(&responders)
                .answer_the_first_that_speaks()
                .is_err(),
            "a machine answering nowhere answered anyway"
        );
    }

    /// **What is said is the same bytes on every network and in both families**,
    /// the machine and the workspace alike: a machine that said something
    /// different on one network would be two machines to whoever asked on both.
    ///
    /// The answers are compared **where they are made**, which on one host is the
    /// only place every responder can be reached: a datagram to this machine's
    /// own address on another interface is delivered over loopback, and this
    /// kernel's first network namespace has IPv6 switched off altogether
    /// (`docs/quirks.md`), so neither the other networks' sockets nor the IPv6
    /// one can be asked here. What can be asked is asked — over IPv4 on loopback,
    /// off a real socket. The two cables carrying one private range are
    /// `crate::a_discovery_answer_leaves_on_the_network_it_arrived_on`, and the
    /// IPv6 answer asked for real is `crate::two_machines_with_no_ipv4`.
    #[test]
    fn what_is_said_is_the_same_bytes_on_every_network_and_in_both_families() {
        let port = a_free_port();
        let over_ipv6 = crate::unix::a_shared_ipv6_datagram_socket_on(port).unwrap();
        let responders = Responders::bound(port, presence(), &mut |_| {})
            .also_answering_on(over_ipv6)
            .unwrap()
            .hosting(a_workspace(), &mut |line| panic!("{line}"));
        assert_eq!(responders.workspace(), Some(a_workspace()));
        let networks = responders.answered_on();
        assert!(
            networks.len() > 1,
            "a machine on one network is not a machine on two: {networks:?}"
        );

        let the_machine = about(&presence()).unwrap();
        let the_workspace =
            about_a_workspace(&WorkspacePresence::of(here(), a_workspace().get())).unwrap();
        for responder in responders.answering() {
            assert_eq!(
                about(responder.answering.presence()).unwrap(),
                the_machine,
                "{responder:?}"
            );
            assert_eq!(
                about_a_workspace(responder.answering.workspace().unwrap()).unwrap(),
                the_workspace,
                "{responder:?}"
            );
        }

        // And asked for real over IPv4 on loopback: those same two answers come
        // back off a socket.
        let mut back = asked(&responders, &a_question().unwrap(), port);
        back.extend(asked(
            &responders,
            &a_question_for_workspaces().unwrap(),
            port,
        ));
        assert_eq!(back, vec![the_machine, the_workspace]);
    }

    /// **A machine answering on a socket somebody handed in answers on that one
    /// alone**, follows no kernel, and is joined to nothing.
    #[test]
    fn a_socket_handed_in_is_the_only_one() {
        let handed = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = handed.local_addr().unwrap().port();
        let responders = Responders::on(handed, presence()).unwrap();
        assert!(responders.answered_on().is_empty());
        assert!(responders.joined().is_empty());
        responders.changed(&Went::any_of_them(), &mut |line| {
            panic!("a handed-in socket followed the kernel: {line}")
        });
        assert_eq!(Responding::of(&responders).waiting_on().len(), 1);
        assert_eq!(
            asked(&responders, &a_question().unwrap(), port),
            vec![about(&presence()).unwrap()]
        );
    }

    /// **Nothing ready is nothing answered**, and a packet that is not a question
    /// is stepped over rather than answered.
    #[test]
    fn nothing_ready_is_nothing_answered_and_a_non_question_is_stepped_over() {
        let handed = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        handed
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        let port = handed.local_addr().unwrap().port();
        let responders = Responders::on(handed, presence()).unwrap();
        let responding = Responding::of(&responders);
        assert!(responding.answer_one(&[false]).unwrap().is_none());
        assert!(responding.answer_one(&[]).unwrap().is_none());
        assert!(asked(&responders, b"who is there?", port).is_empty());
    }

    /// **A network that goes is let go of and a network that comes back is
    /// answered on again**, which is what following the kernel is for.
    #[test]
    fn a_network_that_goes_is_let_go_of_and_one_that_comes_is_answered_on() {
        let port = a_free_port();
        let responders = Responders::bound(port, presence(), &mut |_| {});
        let networks = responders.answered_on();
        assert!(!networks.is_empty());

        responders.answer_on(&[], &[], &Went::nothing(), &mut |line| panic!("{line}"));
        assert!(
            responders.answered_on().is_empty(),
            "a network that went is still answered on"
        );
        responders.answer_on(&networks, &[], &Went::nothing(), &mut |line| {
            panic!("{line}")
        });
        assert_eq!(responders.answered_on(), networks);
    }

    /// Whether the responders have said they moved, without waiting for it, and
    /// emptied.
    fn said_it_moved(responders: &Responders) -> bool {
        let moved = responders.moved().unwrap();
        let [said] = crate::unix::ready(&[Some(moved.waiting_on())], Some(Duration::ZERO)).unwrap();
        moved.heard();
        said
    }

    /// **The responders say when they move, and only then**: a network let go of
    /// and a network answered on each wake the thread waiting on the responders,
    /// and a notification that changes nothing wakes nobody — a machine whose
    /// networks do not change costs nothing for following them.
    #[test]
    fn the_responders_say_when_they_move_and_only_then() {
        let port = a_free_port();
        let responders = Responders::bound(port, presence(), &mut |line| panic!("{line}"));
        let networks = responders.answered_on();
        assert!(!networks.is_empty());
        // Answering on the networks it had at start is a move nobody was
        // waiting for yet; it is emptied here.
        said_it_moved(&responders);

        responders.answer_on(&networks, &[], &Went::nothing(), &mut |line| {
            panic!("{line}")
        });
        assert!(
            !said_it_moved(&responders),
            "nothing changed, and the waiting thread was woken"
        );
        responders.answer_on(&[], &[], &Went::nothing(), &mut |line| panic!("{line}"));
        assert!(
            said_it_moved(&responders),
            "a network let go of said nothing"
        );
        responders.answer_on(&[], &[], &Went::nothing(), &mut |line| panic!("{line}"));
        assert!(!said_it_moved(&responders));
        responders.answer_on(&networks, &[], &Went::nothing(), &mut |line| {
            panic!("{line}")
        });
        assert!(
            said_it_moved(&responders),
            "a network answered on said nothing"
        );
    }

    /// **A network that will not take a socket is not a move**: nothing was
    /// added, so nothing is said, and the refusal is the line in the log.
    #[test]
    fn a_network_that_will_not_take_a_socket_is_not_a_move() {
        let responders = Responders::bound(a_free_port(), presence(), &mut |_| {});
        let before = responders.answered_on();
        said_it_moved(&responders);
        let nowhere =
            listening_networks(&[interface(0, "nowhere", &[Ipv4Addr::new(10, 65, 0, 1)])]);
        let both: Vec<_> = nowhere.iter().chain(&before).cloned().collect();
        let mut said = Vec::new();
        responders.answer_on(&both, &[], &Went::nothing(), &mut |line| {
            said.push(line.to_owned())
        });
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(
            !said_it_moved(&responders),
            "a network that took no socket woke the waiting thread"
        );
    }

    /// **A socket somebody handed in never moves**, so there is nothing to wait
    /// on for it.
    #[test]
    fn a_socket_handed_in_has_nothing_to_say_about_moving() {
        let handed = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let responders = Responders::on(handed, presence()).unwrap();
        assert!(responders.moved().is_none());
        let hosted = responders.hosting(a_workspace(), &mut |line| panic!("{line}"));
        assert!(hosted.moved().is_none());
    }

    /// Whether the interface numbered `index` is in the discovery group, as
    /// `/proc/net/igmp` lists it: a line naming the interface, then one line per
    /// group, each the address's four bytes as the kernel holds them.
    fn in_the_group(index: u32) -> bool {
        let group = format!(
            "{:08X}",
            u32::from_ne_bytes(alo_nearby::THE_ADDRESS.octets())
        );
        let listed = std::fs::read_to_string("/proc/net/igmp").unwrap();
        let mut on = None;
        for line in listed.lines() {
            if line.starts_with(|first: char| first.is_ascii_digit()) {
                on = line
                    .split_whitespace()
                    .next()
                    .and_then(|at| at.parse().ok());
            } else if line.split_whitespace().next() == Some(group.as_str()) && on == Some(index) {
                return true;
            }
        }
        false
    }

    /// The loopback network, as the kernel on this host reports it.
    fn loopback() -> Network {
        listening_networks(&crate::route_messages::reported_by_the_kernel().unwrap())
            .into_iter()
            .find(|network| network.address().is_loopback())
            .unwrap()
    }

    /// **A join the kernel refuses `EADDRINUSE` is taken afresh and never read as
    /// joined, and a responder let go of leaves its group then and there.**
    ///
    /// On loopback, the one interface every host has, and in one test because
    /// every step reads the same interface's membership. The refusal for a record
    /// a deleted interface left behind is measured on a real cable by
    /// `crate::a_cable_re_laid_between_two_readings`; what is measured here is
    /// that the refusal is answered with a membership that is really there.
    #[test]
    fn a_join_refused_as_already_held_is_taken_afresh_and_a_responder_let_go_of_leaves() {
        let loopback = loopback();
        let std::net::IpAddr::V4(address) = loopback.address() else {
            panic!("{loopback:?}");
        };
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        socket
            .join_multicast_v4(&alo_nearby::THE_ADDRESS, &address)
            .unwrap();

        joined(&socket, address).unwrap();
        assert!(in_the_group(loopback.index()));
        assert_eq!(
            socket
                .join_multicast_v4(&alo_nearby::THE_ADDRESS, &address)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AddrInUse,
            "a membership taken afresh is not held"
        );
        left(&socket, address);
        assert!(
            !in_the_group(loopback.index()),
            "a membership left is still there"
        );
        // Leaving where nothing is held is nothing, and a join afterwards is an
        // ordinary one.
        left(&socket, address);
        joined(&socket, address).unwrap();
        left(&socket, address);

        // A responder joined on loopback, let go of while a round still holds
        // its socket open: the group is left at once, not when that round ends.
        let responders = Responders::bound(a_free_port(), presence(), &mut |_| {});
        responders.answer_on(&[], &[], &Went::nothing(), &mut |line| panic!("{line}"));
        let only_loopback = [loopback.clone()];
        responders.answer_on(
            &only_loopback,
            &only_loopback,
            &Went::nothing(),
            &mut |line| panic!("{line}"),
        );
        assert_eq!(responders.joined(), vec![loopback.clone()]);
        assert!(in_the_group(loopback.index()));
        let a_round_still_waiting = Responding::of(&responders);
        responders.answer_on(&[], &[], &Went::nothing(), &mut |line| panic!("{line}"));
        assert!(
            !in_the_group(loopback.index()),
            "a responder let go of left its membership to the close of a socket a round still held"
        );
        drop(a_round_still_waiting);
    }

    /// **A join where no interface is is refused**, never counted as joined.
    #[test]
    fn a_join_where_no_interface_is_is_refused() {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        assert!(joined(&socket, Ipv4Addr::new(192, 0, 2, 77)).is_err());
    }

    /// The kernel's message that the interface numbered `index` was deleted.
    fn a_deletion_of(index: u32) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(&32_u32.to_ne_bytes());
        message.extend_from_slice(&17_u16.to_ne_bytes());
        message.extend_from_slice(&[0; 14]);
        message.extend_from_slice(&index.to_ne_bytes());
        message.extend_from_slice(&[0; 8]);
        message
    }

    /// The descriptor of each socket `responding` waits on, beside the number of
    /// the interface it is held to, in the order of those numbers.
    fn descriptors(responders: &Responders, responding: &Responding) -> Vec<(u32, i32)> {
        use std::os::fd::AsRawFd;
        let mut held: Vec<_> = responders
            .answered_on()
            .iter()
            .map(Network::index)
            .zip(responding.waiting_on().iter().map(AsRawFd::as_raw_fd))
            .collect();
        held.sort_unstable();
        held
    }

    /// The same networks, in the kernel's order, whatever order they were
    /// answered on in.
    fn sorted(mut networks: Vec<Network>) -> Vec<u32> {
        networks.sort_by_key(Network::index);
        networks.iter().map(Network::index).collect()
    }

    /// **A responder whose interface the kernel said went is let go of even where
    /// its number is reported again**, and that network is answered on by a new
    /// socket — a move the answering thread is told of — while a responder whose
    /// interface did not go keeps its socket. A round that lost what the kernel
    /// said answers every network again on new sockets.
    #[test]
    fn a_responder_whose_interface_went_is_answered_again_on_a_new_socket() {
        let port = a_free_port();
        let responders = Responders::bound(port, presence(), &mut |line| panic!("{line}"));
        let networks = responders.answered_on();
        let loopback = loopback();
        said_it_moved(&responders);

        // The rounds hold the sockets open, so a descriptor let go of is not
        // handed out again for the new one.
        let before = Responding::of(&responders);
        let was = descriptors(&responders, &before);
        let mut went = Went::nothing();
        went.heard(&a_deletion_of(loopback.index()));
        responders.answer_on(&networks, &[], &went, &mut |line| panic!("{line}"));
        assert_eq!(sorted(responders.answered_on()), sorted(networks.clone()));
        assert!(
            said_it_moved(&responders),
            "a responder answered again said nothing"
        );
        let after = Responding::of(&responders);
        let is = descriptors(&responders, &after);
        for ((index, was), (_, is)) in was.iter().zip(&is) {
            if *index == loopback.index() {
                assert_ne!(
                    was, is,
                    "the socket held to an interface that went was kept"
                );
            } else {
                assert_eq!(was, is, "a socket whose interface did not go was replaced");
            }
        }
        drop((before, after));
        assert_eq!(
            asked(&responders, &a_question().unwrap(), port),
            vec![about(&presence()).unwrap()]
        );

        let before = Responding::of(&responders);
        let was = descriptors(&responders, &before);
        responders.answer_on(&networks, &[], &Went::any_of_them(), &mut |line| {
            panic!("{line}")
        });
        assert_eq!(sorted(responders.answered_on()), sorted(networks));
        assert!(said_it_moved(&responders));
        let is = descriptors(&responders, &Responding::of(&responders));
        assert!(
            was.iter().all(|descriptor| !is.contains(descriptor)),
            "a round that lost what the kernel said kept a socket: {was:?} {is:?}"
        );
    }

    /// **Hosting a workspace keeps the responders' voice**: the thread is told of
    /// moves on the responders that answer, which are the hosting ones.
    #[test]
    fn hosting_a_workspace_keeps_saying_when_the_responders_move() {
        let responders = Responders::bound(a_free_port(), presence(), &mut |_| {})
            .hosting(a_workspace(), &mut |line| panic!("{line}"));
        said_it_moved(&responders);
        responders.answer_on(&[], &[], &Went::nothing(), &mut |line| panic!("{line}"));
        assert!(said_it_moved(&responders));
    }
}
