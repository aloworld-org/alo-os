//! Which way one road out goes, decided once, before anything opens.
//!
//! One function, and every road out of this machine goes through it. That is
//! what *machine-wide* is made of: not a setting each part of the system reads
//! and interprets, but one decision the whole system asks for, with the same
//! answer for the browser, for installing an application, for an update and for
//! a question put to a provider.
//!
//! # The order, and why it is this order
//!
//! 1. **This machine goes straight out**, whatever anybody set. A model
//!    answering here (ADR 0007) is reached on loopback, and a proxy setting
//!    that broke it would stop the product working the day somebody typed a
//!    company address into a settings panel.
//! 2. **An exception goes straight out.** It is a person's or an
//!    organisation's own list of what the proxy is not for.
//! 3. **Otherwise the setting decides** — the address for this scheme, or the
//!    automatic configuration, asked through a program that holds nothing.
//!
//! The first two are asked **before** the evaluator, and that is deliberate:
//! a machine on a company network reaches its own intranet and its own loopback
//! constantly, and sending each of those to a program — and through it to the
//! company's own script — would be a running account of what the machine is
//! doing, paid to whoever wrote the script.
//!
//! # The road is named, and naming it is not decoration
//!
//! [`the_way`] takes a [`Road`], and there is no way to ask without naming one.
//! It is the same mechanism `alo_egress::Errand` is: a closed list is what makes
//! *every road* checkable rather than aspirational, and a road that reached the
//! network without naming itself would be the one nobody could hold to this.
//!
//! The answer does not depend on which road it is, and that is the point rather
//! than an oversight — **one proxy, machine-wide**. A setting that sent updates
//! one way and questions another would be two settings wearing one name.

use crate::automatic::{TheEvaluator, understood};
use crate::reaching::Reaching;
use crate::refusing::NotOnTheRoad;
use crate::road::{Road, Way};
use crate::setting::TheProxy;

/// Which way this road out goes.
///
/// # Errors
/// [`NotOnTheRoad`] when the machine's proxy is an automatic configuration and
/// this machine could not work out from it where the road goes. **The road is
/// then not taken** — never taken straight out instead, which would send a
/// company's traffic around its own rule with nobody told.
pub fn the_way(
    proxy: &TheProxy,
    road: Road,
    reaching: &Reaching,
    evaluating: &dyn TheEvaluator,
) -> Result<Way, NotOnTheRoad> {
    // Named so that a road cannot be taken without saying which it is, and read
    // here so that the compiler agrees it was. What it never does is change the
    // answer: one proxy, machine-wide.
    let _ = road;

    if reaching.is_this_machine() {
        return Ok(Way::Straight);
    }
    match proxy {
        TheProxy::None => Ok(Way::Straight),
        TheProxy::Manual { exceptions, .. } => {
            if exceptions.let_through(reaching) {
                return Ok(Way::Straight);
            }
            Ok(proxy
                .for_(reaching.scheme())
                .cloned()
                .map_or(Way::Straight, Way::Through))
        }
        TheProxy::Automatic { at } => {
            let answered = evaluating
                .asked(at, &reaching.as_an_address(), reaching.host())
                .map_err(NotOnTheRoad::CouldNotBeWorkedOut)?;
            understood(&answered).map_err(NotOnTheRoad::CouldNotBeWorkedOut)
        }
    }
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
    use crate::reaching::Scheme;
    use crate::testing::{Answering, NeverAsked};

    fn an_address() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
    }

    fn going_to(host: &str) -> Reaching {
        Reaching::over(Scheme::Https, host).unwrap()
    }

    fn automatic() -> TheProxy {
        TheProxy::Automatic {
            at: ConfigurationAddress::checked("http://wpad.example.com/c").unwrap(),
        }
    }

    /// **Every road takes the proxy.** The answer is the same one for all seven
    /// — one proxy, machine-wide — and this is the test that says so for each
    /// of them rather than for the one somebody happened to write.
    #[test]
    fn every_road_out_goes_through_the_one_proxy() {
        let proxy = TheProxy::one(an_address());
        for road in Road::EVERY {
            assert_eq!(
                the_way(&proxy, road, &going_to("files.example.com"), &NeverAsked).unwrap(),
                Way::Through(an_address()),
                "{road:?}"
            );
        }
    }

    /// **No proxy is no proxy on every road**, and nothing is asked of an
    /// evaluator to find that out.
    #[test]
    fn with_no_proxy_every_road_goes_straight_out() {
        for road in Road::EVERY {
            assert_eq!(
                the_way(
                    &TheProxy::None,
                    road,
                    &going_to("files.example.com"),
                    &NeverAsked
                )
                .unwrap(),
                Way::Straight,
                "{road:?}"
            );
        }
    }

    /// **This machine is never sent through the proxy**, whatever is set — so a
    /// model answering here keeps answering when somebody types a company
    /// address into a settings panel.
    #[test]
    fn this_machine_is_never_sent_through_the_proxy() {
        for proxy in [TheProxy::one(an_address()), automatic()] {
            for here in ["localhost", "127.0.0.1", "::1"] {
                assert_eq!(
                    the_way(
                        &proxy,
                        Road::AskingAProvider,
                        &Reaching::over(Scheme::Http, here).unwrap(),
                        &NeverAsked
                    )
                    .unwrap(),
                    Way::Straight,
                    "{here}"
                );
            }
        }
    }

    /// An excepted place goes straight out, and the evaluator is not asked
    /// about it either — an intranet is not something to report to a script.
    #[test]
    fn an_excepted_place_goes_straight_out_without_anything_being_asked() {
        let proxy = TheProxy::one(an_address()).excepting(Exceptions::of(["example.com"]).unwrap());
        assert_eq!(
            the_way(
                &proxy,
                Road::InstallingAnApplication,
                &going_to("files.example.com"),
                &NeverAsked
            )
            .unwrap(),
            Way::Straight
        );
        assert_eq!(
            the_way(
                &proxy,
                Road::InstallingAnApplication,
                &going_to("dl.flathub.test"),
                &NeverAsked
            )
            .unwrap(),
            Way::Through(an_address())
        );
    }

    /// A manual setting that names one scheme and not the other sends the one
    /// it named and lets the other straight out.
    #[test]
    fn a_scheme_with_no_address_goes_straight_out() {
        let proxy = TheProxy::Manual {
            http: Some(an_address()),
            https: None,
            exceptions: Exceptions::none(),
        };
        assert_eq!(
            the_way(
                &proxy,
                Road::SigningIn,
                &Reaching::over(Scheme::Http, "identity.example.com").unwrap(),
                &NeverAsked
            )
            .unwrap(),
            Way::Through(an_address())
        );
        assert_eq!(
            the_way(
                &proxy,
                Road::SigningIn,
                &Reaching::over(Scheme::Https, "identity.example.com").unwrap(),
                &NeverAsked
            )
            .unwrap(),
            Way::Straight
        );
    }

    /// An automatic configuration is asked where the road is going, and its
    /// answer is the way out.
    #[test]
    fn an_automatic_configuration_answers_per_road() {
        let asked = Answering::saying("PROXY proxy.example.com:8080");
        assert_eq!(
            the_way(
                &automatic(),
                Road::UpdatingAnApplication,
                &going_to("files.example.com"),
                &asked
            )
            .unwrap(),
            Way::Through(an_address())
        );
        assert_eq!(
            asked.asked_about(),
            vec![(
                "http://wpad.example.com/c".to_owned(),
                "https://files.example.com/".to_owned(),
                "files.example.com".to_owned()
            )]
        );
    }

    /// **A configuration that cannot be worked out refuses, on every road**,
    /// and never quietly becomes straight out. This is the refusal the
    /// constraint asks for, and the one that matters on a network where the
    /// proxy is a rule rather than a route.
    #[test]
    fn a_configuration_that_cannot_be_worked_out_refuses_on_every_road() {
        for asked in [
            Answering::refusing(NotEvaluated::NothingEvaluatesIt),
            Answering::refusing(NotEvaluated::DidNotAnswer {
                said: "no".to_owned(),
            }),
            Answering::saying("SOCKS proxy.example.com:1080"),
            Answering::saying(""),
        ] {
            for road in Road::EVERY {
                let refused = the_way(&automatic(), road, &going_to("files.example.com"), &asked)
                    .unwrap_err();
                assert!(
                    matches!(refused, NotOnTheRoad::CouldNotBeWorkedOut(_)),
                    "{road:?}: {refused:?}"
                );
            }
        }
    }

    /// An automatic configuration answering *straight out* is straight out —
    /// the one case where the refusal above must not fire.
    #[test]
    fn an_automatic_configuration_may_answer_straight_out() {
        assert_eq!(
            the_way(
                &automatic(),
                Road::FetchingAModel,
                &going_to("models.example.com"),
                &Answering::saying("DIRECT")
            )
            .unwrap(),
            Way::Straight
        );
    }

    /// **Nothing but the host and the scheme reaches the script.** What a
    /// person is doing on the machine is not something a company's script is
    /// told, and there is nowhere in the call for it to have arrived from.
    #[test]
    fn the_script_is_told_where_the_road_goes_and_nothing_about_what_is_on_it() {
        let asked = Answering::saying("DIRECT");
        the_way(
            &automatic(),
            Road::AskingAProvider,
            &going_to("api.example.com"),
            &asked,
        )
        .unwrap();
        for (_, address, host) in asked.asked_about() {
            assert_eq!(address, "https://api.example.com/");
            assert_eq!(host, "api.example.com");
        }
    }
}
