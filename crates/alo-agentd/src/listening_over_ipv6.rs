//! The one IPv6-only listener on the port presence advertises, and whether another
//! program holding the port over IPv6 keeps it from being bound.
//!
//! *Machines find each other with zero configuration.* The port is listened on
//! over IPv4 once per network, each listener held to its interface
//! (`crate::listeners`), and over IPv6 by **one** listener held to nothing. That
//! one is the only way a machine on a network with no IPv4 address — two machines
//! joined by a cable and nothing else, whose only addresses are link-local — reaches
//! this one's port.
//!
//! Before this file it was bound once, at start, and never again. If another
//! program held `[::]` at the port then — an installer, or a service restarting as
//! the machine boots — the service log said the port was bound over IPv4 alone, and
//! nothing tried it again: not when a network changed, and not when the kernel said
//! that program let go (`crate::told_of_a_port_let_go`, which joins the IPv6 group
//! as well). A machine whose only network is link-local was found there and could
//! not reach this one until the service restarted.
//!
//! # A separate thing tried again, not one of the refused networks
//!
//! The IPv4 listeners are remembered as refused **by interface number**, because
//! each is held to an interface and goes when its interface goes. The IPv6 listener
//! is held to no interface, is not in the list of networks the kernel reports, and
//! must not be let go of when a network goes. Counting it as one of the refused
//! networks would mean inventing an interface number it does not have, and teaching
//! the rule that forgets a network that went to leave that number alone. So it is
//! its own small state here — listened on, held by another program, or not to be
//! had — and `crate::listeners` tries it again on the same two occasions it tries a
//! refused network: when the kernel says a program let go of the port, and when a
//! network changed.
//!
//! # Only a port held by another program is tried again
//!
//! What the kernel refuses with `EADDRINUSE` is somebody else holding the port, and
//! that ends when they let go. Anything else — a kernel with IPv6 left out of it
//! answers `EAFNOSUPPORT` — does not end while the service runs, so it is said once
//! as it always was, and never tried again: a let-go of the port is no reason to ask
//! a kernel with no IPv6 for an IPv6 socket.
//!
//! The refusal is said once, when it happens; a try that is refused again is not said
//! again, because hearing a socket at the port destroyed is a reason to try and never
//! proof the port is free — the other program's own connections close at that port
//! too. The bind, when it comes, is said once.

use std::io::ErrorKind;
use std::sync::{Arc, Mutex, PoisonError};

use crate::listeners::Listener;

/// Where the IPv6 listener stands.
#[derive(Debug)]
enum Standing {
    /// Bound, and listened on.
    Listening(Arc<Listener>),
    /// Refused because another program holds the port over IPv6: tried again.
    HeldElsewhere,
    /// Not to be had while the service runs — said once, never tried again.
    NotHere,
    /// Never asked for, on a wire listening on a listener somebody handed in.
    NotAsked,
}

/// The IPv6-only listener on the port, bound or waiting to be.
#[derive(Debug)]
pub(crate) struct OverIpv6 {
    /// Where it stands. Locked only for the length of one call here, and never
    /// while another lock of `crate::listeners` is taken after it.
    standing: Mutex<Standing>,
    /// The port it is bound at.
    port: u16,
}

impl OverIpv6 {
    /// Bind the IPv6-only listener at `port`, saying a refusal in the service log.
    ///
    /// `until` is what the log says about when a refused port is tried again, which
    /// depends on whether the kernel will say the port was let go of. What refused
    /// the bind is handed back beside it, for a machine that could listen nowhere at
    /// all.
    pub(crate) fn bound(
        port: u16,
        until: &str,
        said: &mut dyn FnMut(&str),
    ) -> (Self, Option<std::io::Error>) {
        let (standing, refused) = match crate::unix::an_ipv6_only_listener_on(port) {
            Ok(listener) => (
                Standing::Listening(Arc::new(Listener::unheld(listener))),
                None,
            ),
            Err(why) if why.kind() == ErrorKind::AddrInUse => {
                said(&format!(
                    "the port presence advertises could not be bound over IPv6 ({why}); it is bound over IPv4 alone, and a machine on a network with no IPv4 address cannot reach this one until it can be, {until}"
                ));
                (Standing::HeldElsewhere, Some(why))
            }
            Err(why) => {
                said(&format!(
                    "the port presence advertises could not be bound over IPv6 ({why}); it is bound over IPv4 alone, and a machine on a network with no IPv4 address cannot reach this one"
                ));
                (Standing::NotHere, Some(why))
            }
        };
        (
            Self {
                standing: Mutex::new(standing),
                port,
            },
            refused,
        )
    }

    /// Nothing listened on over IPv6 and nothing ever tried, for a wire on a
    /// listener somebody handed in.
    pub(crate) const fn not_asked(port: u16) -> Self {
        Self {
            standing: Mutex::new(Standing::NotAsked),
            port,
        }
    }

    /// A handle onto the listener, when it is bound.
    pub(crate) fn listener(&self) -> Option<Arc<Listener>> {
        match &*self.standing() {
            Standing::Listening(listener) => Some(Arc::clone(listener)),
            Standing::HeldElsewhere | Standing::NotHere | Standing::NotAsked => None,
        }
    }

    /// Whether another program held the port over IPv6 when it was last tried —
    /// which is what a let-go of the port is worth trying again for.
    #[cfg(test)]
    pub(crate) fn held_elsewhere(&self) -> bool {
        matches!(*self.standing(), Standing::HeldElsewhere)
    }

    /// Try the port again over IPv6 if another program held it, saying the bind
    /// once when it comes and nothing when it is refused again.
    ///
    /// Bound, not to be had, or never asked for, nothing is asked of the kernel.
    pub(crate) fn tried_again(&self, said: &mut dyn FnMut(&str)) {
        let mut standing = self.standing();
        if !matches!(*standing, Standing::HeldElsewhere) {
            return;
        }
        // Refused again, by the other program or by anything else, it stays a
        // port held elsewhere: the refusal was said once, when it happened.
        if let Ok(listener) = crate::unix::an_ipv6_only_listener_on(self.port) {
            *standing = Standing::Listening(Arc::new(Listener::unheld(listener)));
            said(
                "the port presence advertises is bound over IPv6 now that nothing else holds it there; a machine on a network with no IPv4 address can reach this one",
            );
        }
    }

    /// Where it stands, locked.
    fn standing(&self) -> std::sync::MutexGuard<'_, Standing> {
        self.standing.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::TcpListener;

    use super::OverIpv6;

    /// Until what the tests here say.
    const UNTIL: &str =
        "and it is tried again as soon as the kernel says a program let go of the port";

    /// A port held over IPv6 alone by "another program", at whatever port the
    /// kernel chose — or nothing, on a host whose kernel has no IPv6 at all.
    ///
    /// `[::]` and not `::1`: a kernel can have IPv6 and no loopback address in
    /// it (this WSL host is one), and the bind is what these tests are about.
    fn a_port_held_over_ipv6() -> Option<(u16, TcpListener)> {
        let holding = crate::unix::an_ipv6_only_listener_on(0).ok()?;
        Some((holding.local_addr().ok()?.port(), holding))
    }

    /// **A port held over IPv6 is said once, and tried again only while it is
    /// held**: refused again, nothing more is said; free, it binds and that is
    /// said once; bound, a later try asks nothing.
    #[test]
    fn a_port_held_over_ipv6_is_said_once_and_bound_once_when_let_go_of() {
        let Some((port, holding)) = a_port_held_over_ipv6() else {
            return;
        };
        let mut said = Vec::new();
        let (over_ipv6, refused) = OverIpv6::bound(port, UNTIL, &mut |line| {
            said.push(line.to_owned());
        });
        assert_eq!(
            refused.map(|why| why.kind()),
            Some(std::io::ErrorKind::AddrInUse)
        );
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(
            said.first()
                .is_some_and(|line| line.contains("over IPv6") && line.ends_with(UNTIL)),
            "the refusal does not say when it is tried again: {said:?}"
        );
        assert!(over_ipv6.held_elsewhere());
        assert!(over_ipv6.listener().is_none());

        over_ipv6.tried_again(&mut |line| panic!("a refusal was said twice: {line}"));
        assert!(over_ipv6.held_elsewhere());

        drop(holding);
        let mut bound = Vec::new();
        over_ipv6.tried_again(&mut |line| bound.push(line.to_owned()));
        assert_eq!(
            bound,
            [
                "the port presence advertises is bound over IPv6 now that nothing else holds it there; a machine on a network with no IPv4 address can reach this one"
            ]
        );
        assert!(!over_ipv6.held_elsewhere());
        assert!(over_ipv6.listener().is_some());
        over_ipv6.tried_again(&mut |line| panic!("a bind was said twice: {line}"));
        // And it really holds the port now.
        assert!(crate::unix::an_ipv6_only_listener_on(port).is_err());
    }

    /// **A listener handed in is never tried over IPv6**: nothing is bound and
    /// nothing is said.
    #[test]
    fn nothing_asked_is_never_tried() {
        let over_ipv6 = OverIpv6::not_asked(1);
        assert!(!over_ipv6.held_elsewhere());
        over_ipv6.tried_again(&mut |line| panic!("{line}"));
        assert!(over_ipv6.listener().is_none());
    }

    /// **What is not to be had is never tried again** — a kernel with no IPv6
    /// asked on every let-go of the port would be asked for nothing.
    #[test]
    fn what_is_not_to_be_had_is_never_tried_again() {
        let over_ipv6 = OverIpv6 {
            standing: std::sync::Mutex::new(super::Standing::NotHere),
            port: 1,
        };
        assert!(!over_ipv6.held_elsewhere());
        over_ipv6.tried_again(&mut |line| panic!("{line}"));
        assert!(over_ipv6.listener().is_none());
    }

    /// **A port free over IPv6 is bound at once and says nothing.**
    #[test]
    fn a_port_free_over_ipv6_is_bound_and_says_nothing() {
        let Some((port, holding)) = a_port_held_over_ipv6() else {
            return;
        };
        drop(holding);
        let (over_ipv6, refused) =
            OverIpv6::bound(port, UNTIL, &mut |line| panic!("a free port said: {line}"));
        assert!(refused.is_none());
        assert!(over_ipv6.listener().is_some());
        assert!(!over_ipv6.held_elsewhere());
    }
}
