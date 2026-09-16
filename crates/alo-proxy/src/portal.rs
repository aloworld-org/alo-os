//! The machine's proxy, answered one address at a time, for the portal
//! applications already ask.
//!
//! `published.rs` is the other half: a proxy an application reads out of its own
//! environment, which is what most software does and what an automatic
//! configuration cannot be squeezed into. This half answers the question an
//! application asks about **one address**, which is the shape an automatic
//! configuration is already in — and the shape a sandboxed application can ask
//! without being handed the whole setting.
//!
//! # What is here, and what is deliberately not
//!
//! What is here is the **answer**: given the machine's setting and an address,
//! the list of ways out, written the way the portal's callers read them.
//! Serving it — owning the name on the bus, reading the request, handing the
//! answer back — is `alo-portals`, and this crate does not edit that one. The
//! wiring is named in `docs/autonomy/updates/one-proxy-machine-wide.md`, as
//! task 3 of the same plan named the wiring of its own answer into the
//! open-with portal.
//!
//! # `direct://`, and why a refusal is not one
//!
//! An application asking where an address goes is answered with a list of ways
//! out; the way that means *straight out* is written `direct://`. A road this
//! machine **refused** is not that, and must never be answered as it: an
//! application told `direct://` would go straight out, which is exactly what
//! refusing was for. So [`looked_up`] answers a refusal as an error, and
//! `alo-portals` turns one into the portal's own, which the application sees as
//! a failure rather than as permission.
//!
//! # No credential leaves this machine through here
//!
//! The answer names the proxy's address and never a name or a password —
//! `published.rs` argues why at length, and the rule is the same: the portal
//! answers software nobody here wrote.

use crate::automatic::TheEvaluator;
use crate::deciding::the_way;
use crate::reaching::{NotReachable, Reaching};
use crate::refusing::NotOnTheRoad;
use crate::road::Road;
use crate::setting::TheProxy;

/// How *straight out* is written in an answer.
pub const STRAIGHT_OUT: &str = "direct://";

/// Why an application's question about an address was not answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotLookedUp {
    /// What it asked about is not a road out of this machine.
    NotARoadOut(NotReachable),
    /// The machine's proxy could not be worked out for it, so the road is
    /// refused — **never answered as straight out**.
    Refused(NotOnTheRoad),
}

/// Where an application's address goes, as the portal's callers read it.
///
/// One entry, because this machine decides one way out per road and does not
/// offer a list to fall back through: `crate::automatic` says why falling back
/// by ourselves would be this machine deciding a company's rule had stopped
/// applying.
///
/// # Errors
/// [`NotLookedUp`], which `alo-portals` turns into the portal's own failure.
/// **Never an answer of straight out** for a road that was refused.
pub fn looked_up(
    proxy: &TheProxy,
    address: &str,
    evaluating: &dyn TheEvaluator,
) -> Result<Vec<String>, NotLookedUp> {
    let reaching = Reaching::of(address).map_err(NotLookedUp::NotARoadOut)?;
    // An application's road out is not one of the seven alo OS takes for itself,
    // and the setting is the same setting — one proxy, machine-wide. It is
    // named as the road an agent's question takes because that is the one on
    // the list that is not alo OS's own errand.
    let way = the_way(proxy, Road::AskingAProvider, &reaching, evaluating)
        .map_err(NotLookedUp::Refused)?;
    Ok(vec![way.through().map_or_else(
        || STRAIGHT_OUT.to_owned(),
        crate::address::ProxyAddress::written,
    )])
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::address::{ProxyAddress, SpokenTo};
    use crate::automatic::{ConfigurationAddress, NotEvaluated};
    use crate::exceptions::Exceptions;
    use crate::password::WhereThePasswordIs;
    use crate::testing::{Answering, NeverAsked};

    fn an_address() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
    }

    fn automatic() -> TheProxy {
        TheProxy::Automatic {
            at: ConfigurationAddress::checked("http://wpad.example.com/c").unwrap(),
        }
    }

    /// An application asking where an address goes is told the proxy, or told
    /// straight out — the same setting every road out of this machine takes.
    #[test]
    fn an_application_is_answered_with_the_machines_one_proxy() {
        assert_eq!(
            looked_up(
                &TheProxy::one(an_address()),
                "https://files.example.com/a/page",
                &NeverAsked
            )
            .unwrap(),
            vec!["http://proxy.example.com:8080".to_owned()]
        );
        assert_eq!(
            looked_up(&TheProxy::None, "https://files.example.com/", &NeverAsked).unwrap(),
            vec![STRAIGHT_OUT.to_owned()]
        );
    }

    /// This machine and an excepted place are answered straight out, so an
    /// application reaching a local runtime or an intranet is not sent through
    /// the proxy either.
    #[test]
    fn this_machine_and_an_excepted_place_are_answered_straight_out() {
        let proxy = TheProxy::one(an_address()).excepting(Exceptions::of(["example.com"]).unwrap());
        for address in [
            "http://127.0.0.1:11434/v1/models",
            "https://files.example.com/",
        ] {
            assert_eq!(
                looked_up(&proxy, address, &NeverAsked).unwrap(),
                vec![STRAIGHT_OUT.to_owned()],
                "{address}"
            );
        }
    }

    /// **A refused road is an error and never an answer of straight out.** An
    /// application told straight out would do the one thing refusing was for.
    #[test]
    fn a_refused_road_is_never_answered_as_straight_out() {
        for asked in [
            Answering::refusing(NotEvaluated::NothingEvaluatesIt),
            Answering::saying("SOCKS proxy.example.com:1080"),
        ] {
            let refused =
                looked_up(&automatic(), "https://files.example.com/", &asked).unwrap_err();
            assert!(matches!(refused, NotLookedUp::Refused(_)), "{refused:?}");
        }
    }

    /// An address that is not a road out of this machine is refused rather than
    /// answered about.
    #[test]
    fn an_address_that_is_not_a_road_out_is_refused_rather_than_answered() {
        for address in ["file:///etc/passwd", "   ", "not-an-address"] {
            let refused =
                looked_up(&TheProxy::one(an_address()), address, &NeverAsked).unwrap_err();
            assert!(matches!(refused, NotLookedUp::NotARoadOut(_)), "{address}");
        }
    }

    /// **No credential reaches an application through the portal**, whatever
    /// the setting holds.
    #[test]
    fn no_credential_reaches_an_application_through_the_portal() {
        let signing_in = an_address()
            .signing_in(
                "anna",
                WhereThePasswordIs::named("the company proxy").unwrap(),
            )
            .unwrap();
        let answered = looked_up(
            &TheProxy::one(signing_in),
            "https://files.example.com/",
            &NeverAsked,
        )
        .unwrap();
        for entry in &answered {
            assert!(!entry.contains("anna"), "{entry}");
            assert!(!entry.contains('@'), "{entry}");
        }
        assert_eq!(answered, vec!["http://proxy.example.com:8080".to_owned()]);
    }

    /// An automatic configuration answers per address, which is the shape the
    /// portal already asks in.
    #[test]
    fn an_automatic_configuration_answers_the_portal_per_address() {
        let asked = Answering::saying("PROXY proxy.example.com:8080");
        assert_eq!(
            looked_up(&automatic(), "https://files.example.com/x", &asked).unwrap(),
            vec!["http://proxy.example.com:8080".to_owned()]
        );
        assert_eq!(
            asked
                .asked_about()
                .first()
                .map(|(_, address, _)| address.clone()),
            Some("https://files.example.com/".to_owned()),
            "the script is told where the road goes and not what is on it"
        );
    }
}
