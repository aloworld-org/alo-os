//! What the machine's proxy is published to applications as.
//!
//! *Honoured by applications* is half of `docs/features.md`'s promise, and the
//! half that cannot be kept by anything alo OS decides on its own: an
//! application is somebody else's software, running sandboxed, and it honours a
//! proxy the way it already knows how. There are two ways it already knows, and
//! alo OS uses both rather than inventing a third.
//!
//! | | |
//! |---|---|
//! | this file | the setting, as an application's own environment gives it |
//! | `portal.rs` | the setting, as the proxy portal answers one address at a time |
//!
//! Both names of every variable, because programs disagree about which they
//! read and a machine-wide proxy that reached half of them would be worse than
//! no proxy at all — a person would fix the half that was broken and never find
//! out about the other.
//!
//! # No credential is published, and that is a decision rather than a gap
//!
//! What goes into an application's environment carries the proxy's address and
//! **never its password**. Handing a company's proxy credential to every
//! sandboxed application on the machine would be a grant nobody made, to
//! software nobody here wrote, in the one place a sandbox cannot take it back.
//! An application that meets a proxy wanting a name is asked for one by the
//! proxy itself and asks the person, which is what it already does everywhere
//! else.
//!
//! What this costs is named rather than hidden: on a network whose proxy wants
//! a credential, an application is asked for it once per application instead of
//! never. `docs/autonomy/updates/one-proxy-machine-wide.md` says so.
//!
//! # An automatic configuration cannot be a variable, and is not faked into one
//!
//! An environment says *this proxy, for everything*. An automatic configuration
//! says *it depends where you are going*, and there is no address that means
//! that. So [`Published::OnlyWhenAsked`] is the honest answer, and the road for
//! those applications is the portal — which answers per address, which is
//! exactly what the configuration does.
//!
//! Evaluating the configuration once and publishing whatever it said about some
//! address nobody asked about would be a machine deciding, on a network it
//! cannot see, that one answer covers every road. That is the guess this file
//! refuses to make.

use crate::exceptions::Exceptions;
use crate::reaching::Scheme;
use crate::setting::TheProxy;

/// The places an application never proxies, whatever the setting says.
///
/// `crate::carried::NEVER_THROUGH_A_PROXY` is the same list for the programs
/// alo OS starts itself. It is here as well as there because the two lists are
/// given to different things and would otherwise be one place to forget.
pub const NEVER_THROUGH_A_PROXY: &str = crate::carried::NEVER_THROUGH_A_PROXY;

/// What an application is told about the machine's proxy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Published {
    /// There is no proxy, so there is nothing to publish.
    NoProxy,
    /// These, as an application's own environment gives them.
    These(Vec<(&'static str, String)>),
    /// Nothing an environment can say: the way out depends on where the
    /// application is going, and only the portal answers that.
    OnlyWhenAsked,
}

impl Published {
    /// What an application is told, from the machine's one setting.
    #[must_use]
    pub fn of(proxy: &TheProxy) -> Self {
        match proxy {
            TheProxy::None => Self::NoProxy,
            TheProxy::Automatic { .. } => Self::OnlyWhenAsked,
            TheProxy::Manual { exceptions, .. } => {
                let mut given = Vec::new();
                for (scheme, names) in [
                    (Scheme::Http, ["http_proxy", "HTTP_PROXY"]),
                    (Scheme::Https, ["https_proxy", "HTTPS_PROXY"]),
                ] {
                    if let Some(address) = proxy.for_(scheme) {
                        for name in names {
                            given.push((name, address.written()));
                        }
                    }
                }
                if given.is_empty() {
                    return Self::NoProxy;
                }
                let never = never_proxied(exceptions);
                given.push(("no_proxy", never.clone()));
                given.push(("NO_PROXY", never));
                Self::These(given)
            }
        }
    }

    /// The variables themselves, which is nothing at all for the other two
    /// answers.
    #[must_use]
    pub fn variables(&self) -> &[(&'static str, String)] {
        match self {
            Self::These(given) => given,
            Self::NoProxy | Self::OnlyWhenAsked => &[],
        }
    }
}

/// The exceptions, and this machine, as one list.
fn never_proxied(exceptions: &Exceptions) -> String {
    let mut written = String::from(NEVER_THROUGH_A_PROXY);
    for under in exceptions.each() {
        written.push(',');
        written.push('.');
        written.push_str(under);
    }
    written
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::address::{ProxyAddress, SpokenTo};
    use crate::password::WhereThePasswordIs;

    fn an_address() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
    }

    /// An application is given the proxy under every name a program reads, and
    /// the places it must not proxy with it.
    #[test]
    fn an_application_is_given_the_proxy_under_every_name_it_might_read() {
        let published = Published::of(&TheProxy::one(an_address()));
        let names: Vec<&str> = published
            .variables()
            .iter()
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            names,
            [
                "http_proxy",
                "HTTP_PROXY",
                "https_proxy",
                "HTTPS_PROXY",
                "no_proxy",
                "NO_PROXY"
            ]
        );
        for (_, value) in published.variables().iter().take(4) {
            assert_eq!(value, "http://proxy.example.com:8080");
        }
    }

    /// **No credential is published.** An application meeting a proxy that
    /// wants one is asked by the proxy, not handed the company's password.
    #[test]
    fn no_credential_is_ever_published_to_an_application() {
        let signing_in = an_address()
            .signing_in(
                "anna",
                WhereThePasswordIs::named("the company proxy").unwrap(),
            )
            .unwrap();
        let published = Published::of(&TheProxy::one(signing_in));
        for (name, value) in published.variables() {
            assert!(!value.contains("anna"), "{name}={value}");
            assert!(!value.contains('@'), "{name}={value}");
            assert!(!value.contains("the company proxy"), "{name}={value}");
        }
    }

    /// The exceptions reach applications too, beside this machine — an
    /// intranet a person excepted is excepted for their browser as well.
    #[test]
    fn the_exceptions_reach_applications_beside_this_machine() {
        let published = Published::of(
            &TheProxy::one(an_address())
                .excepting(crate::exceptions::Exceptions::of(["example.com"]).unwrap()),
        );
        let never = published
            .variables()
            .iter()
            .find(|(name, _)| *name == "no_proxy")
            .map(|(_, value)| value.clone())
            .unwrap();
        assert!(never.contains("localhost"), "{never}");
        assert!(never.contains(".example.com"), "{never}");
    }

    /// No proxy publishes nothing at all, rather than an empty setting an
    /// application would have to interpret.
    #[test]
    fn no_proxy_publishes_nothing() {
        assert_eq!(Published::of(&TheProxy::None), Published::NoProxy);
        assert!(Published::of(&TheProxy::None).variables().is_empty());

        let named_nothing = TheProxy::Manual {
            http: None,
            https: None,
            exceptions: Exceptions::none(),
        };
        assert_eq!(Published::of(&named_nothing), Published::NoProxy);
    }

    /// **An automatic configuration is not faked into a variable.** The honest
    /// answer is that only the portal can say, and this is that answer.
    #[test]
    fn an_automatic_configuration_is_not_published_as_a_variable() {
        let automatic = TheProxy::Automatic {
            at: crate::automatic::ConfigurationAddress::checked("http://wpad.example.com/c")
                .unwrap(),
        };
        assert_eq!(Published::of(&automatic), Published::OnlyWhenAsked);
        assert!(Published::of(&automatic).variables().is_empty());
    }

    /// A setting that names one scheme publishes that one, and says nothing
    /// about the other.
    #[test]
    fn a_setting_that_names_one_scheme_publishes_that_one_alone() {
        let published = Published::of(&TheProxy::Manual {
            http: None,
            https: Some(an_address()),
            exceptions: Exceptions::none(),
        });
        let names: Vec<&str> = published
            .variables()
            .iter()
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            names,
            ["https_proxy", "HTTPS_PROXY", "no_proxy", "NO_PROXY"]
        );
    }
}
