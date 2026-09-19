//! `[proxy]` — the one proxy this machine reaches the network through, as
//! somebody typed it, and whose it is.
//!
//! ADR 0016: the organisation **bounds**, the person **chooses**. A proxy is the
//! shape of that rule with nothing to choose *within* — a road out either goes
//! through the company's proxy or it does not — so what arrives from this
//! section is `alo_proxy::Kept`, which is the setting and who set it, and a
//! person on a machine an organisation manages is refused a change **in words
//! naming who set it** rather than shown a field that quietly does nothing.
//! That refusal is `alo_proxy::NotChanged`'s and nothing here decides it, as
//! nothing here decides which way a road out then goes. This file is only the
//! door from a few lines in `/etc/alo/agentd.toml` to that value, exactly as
//! `crate::permitted_places` is for where an application may come from.
//!
//! # Three ways out, and there is no fourth
//!
//! ```toml
//! [proxy]
//! goes-through = "an-address"
//! http = "http://proxy.example.com:8080"
//! https = "http://proxy.example.com:8080"
//! except = ["intranet.example.com", ".example.test"]
//! sign-in-as = "anna"
//! password-in-keyring = "the company proxy"
//! ```
//!
//! [`THE_WAY_OUT`] says which of [`EVERY_WAY`] this machine takes, and it is
//! written rather than worked out from which other keys are present: a section
//! whose meaning depended on what somebody had commented out is a section where
//! deleting a line changes where the machine's traffic goes.
//!
//! | `goes-through` | what else the section carries |
//! |---|---|
//! | `"nothing"` | nothing at all — every road out goes straight out |
//! | `"an-address"` | [`THE_HTTP`], [`THE_HTTPS`], [`THE_EXCEPTIONS`], and the two sign-in keys |
//! | `"a-configuration"` | [`THE_CONFIGURATION`], and nothing else |
//!
//! **A key that the way out would ignore is refused**, which is
//! `crate::describing`'s rule about a `region` beside a bound that has no use
//! for one: a key somebody believes is sending their traffic through a proxy and
//! is not is the quiet failure this whole section is built to avoid.
//!
//! # A password is never in this file, and that is a refusal rather than a rule
//!
//! ADR 0022 keeps a credential in the keyring. So the section names **where the
//! password is** ([`THE_KEYRING_NAME`]) and the name to sign in as
//! ([`THE_SIGN_IN`]) — and the two ways somebody would otherwise put a password
//! on this disk are each refused by name:
//!
//! - [`THE_PASSWORD`] is **declared in order to be refused**. Without it, a
//!   `password = "…"` line would be answered as a key nobody declared, which is
//!   the same sentence a typo gets and sends whoever reads it looking for a
//!   spelling mistake. It is not one: it is a credential in `/etc`, and it is
//!   worth its own sentence.
//! - `http://anna:hunter2@proxy.example.com:8080` is how a proxy credential is
//!   usually pasted. `alo_proxy::ProxyAddress` refuses that shape, and it is
//!   refused here too — before the address is built, so the refusal names the
//!   file rather than describing a value that was never made.
//!
//! Neither refusal repeats what was typed, because a password in a refusal is a
//! password in a service log.
//!
//! # Absent is a proxy nobody set
//!
//! No section at all is [`Option::None`], which reaches `crate::Described` as
//! *nobody set a proxy on this machine* — and never as
//! `alo_proxy::TheProxy::None`. The two would take the same roads today and are
//! not the same fact: *straight out* is something somebody wrote down, and
//! writing it on a person's behalf would be this file answering a question about
//! their machine that only they can answer. `alo_software::Bound::Nobodys` is
//! the same distinction one section above.
//!
//! # Who set it is who owns the file
//!
//! Exactly as `[questions]` and `[applications]`: root is an organisation
//! (`alo_proxy::SetBy::AnOrganisation`), the person is their own machine
//! (`alo_proxy::SetBy::ThisPerson`), and what the setting says never enters into
//! it. A person who writes a proxy into their own description is not told an
//! administrator set it.

use alo_proxy::{
    ConfigurationAddress, Exceptions, Kept, NotAConfiguration, NotAnAddress, ProxyAddress,
    SpokenTo, TheProxy, WhereThePasswordIs,
};
use serde::Deserialize;

use crate::refusing::NotDescribed;
use crate::trusting::WhoDescribedIt;

/// The key which way out this machine takes is written under.
pub const THE_WAY_OUT: &str = "proxy.goes-through";

/// The key the proxy for roads spoken in clear is written under.
pub const THE_HTTP: &str = "proxy.http";

/// The key the proxy for roads spoken encrypted is written under.
pub const THE_HTTPS: &str = "proxy.https";

/// The key the places that go straight out anyway are written under.
pub const THE_EXCEPTIONS: &str = "proxy.except";

/// The key the address of an automatic configuration is written under.
pub const THE_CONFIGURATION: &str = "proxy.configuration";

/// The key the name the proxy wants is written under.
pub const THE_SIGN_IN: &str = "proxy.sign-in-as";

/// The key the keyring name that password is kept under is written under.
pub const THE_KEYRING_NAME: &str = "proxy.password-in-keyring";

/// The key a password would be written under, which exists to be refused.
pub const THE_PASSWORD: &str = "proxy.password";

/// Every road out goes straight out.
const NOTHING: &str = "nothing";

/// An address per scheme, with the places excepted from it.
const AN_ADDRESS: &str = "an-address";

/// An address a configuration is at, which answers per road.
const A_CONFIGURATION: &str = "a-configuration";

/// Every way out this service reads, for a refusal that can list them.
pub const EVERY_WAY: [&str; 3] = [NOTHING, AN_ADDRESS, A_CONFIGURATION];

/// Which of [`EVERY_WAY`] a section names, once the spelling has been believed.
///
/// Read out of the text **before** any other key is looked at, so that a
/// section moves from *what somebody typed* to *which of three things this is*
/// in one place — and so that the keys beside it are judged against a way out
/// this service knows rather than against a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WayOut {
    /// Every road out goes straight out.
    Nothing,
    /// An address per scheme, with the places excepted from it.
    AnAddress,
    /// An address a configuration is at.
    AConfiguration,
}

impl WayOut {
    /// The way out this text names, or nothing if it names none.
    fn named(said: &str) -> Option<Self> {
        match said {
            NOTHING => Some(Self::Nothing),
            AN_ADDRESS => Some(Self::AnAddress),
            A_CONFIGURATION => Some(Self::AConfiguration),
            _ => None,
        }
    }
}

/// The proxy this machine reaches the network through, exactly as it was typed.
///
/// The **spelling is this file's**, not `alo_proxy::TheProxy`'s, and that is the
/// same decision `crate::describing` made about `alo_models::SourcePolicy`, with
/// a sharper reason here: `alo_proxy::ProxyAddress` derives its own
/// `Deserialize` over the fields it holds, so a description that named one that
/// way would be a host nobody checked and a `password` field one line from a
/// credential in `/etc`. Everything below goes through that crate's own
/// constructors, which is where a proxy address is refused.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct TheProxySection {
    /// Which of [`EVERY_WAY`] this machine takes.
    goes_through: String,
    /// Where roads spoken in clear go.
    http: Option<String>,
    /// Where roads spoken encrypted go.
    https: Option<String>,
    /// The places that go straight out anyway.
    except: Option<Vec<String>>,
    /// Where the automatic configuration is.
    configuration: Option<String>,
    /// The name the proxy wants.
    sign_in_as: Option<String>,
    /// The keyring name that password is kept under. **Never a password.**
    password_in_keyring: Option<String>,
    /// A password, declared so that one written here is answered as the
    /// credential it is rather than as a key nobody declared.
    ///
    /// `toml::Value` rather than `String` so that `password = 1234` is the same
    /// refusal: what matters is that somebody wrote a password into `/etc`, not
    /// what shape it arrived in.
    password: Option<toml::Value>,
}

impl TheProxySection {
    /// This section as the proxy it states, attributed to whoever wrote the
    /// file — or the first reason it states none.
    ///
    /// # Errors
    ///
    /// [`NotDescribed`], and every one of them stops the service: a machine
    /// whose organisation set a proxy it could not read would be a machine
    /// going straight out onto a network where that is the thing the proxy
    /// exists to prevent. Nothing here is ever answered as *no proxy*.
    pub(crate) fn checked(self, who: WhoDescribedIt) -> Result<Kept, NotDescribed> {
        // Answered first, before the way out itself, because a password in
        // /etc is what somebody has to go and remove whatever else the section
        // says — and because the next thing this file would do with the rest of
        // the section is read the keys beside it.
        if self.password.is_some() {
            return Err(NotDescribed::AProxyPasswordInTheFile { what: THE_PASSWORD });
        }
        let proxy = self.the_way_out()?;
        Ok(match who {
            WhoDescribedIt::AnAdministrator => Kept::by_an_organisation(proxy),
            WhoDescribedIt::ThePerson => Kept::by_this_person(proxy),
        })
    }

    /// The setting this section states.
    ///
    /// The way out is matched first, so a spelling this service does not know is
    /// answered as the typo it is rather than as whichever key beside it turned
    /// out to be missing — `crate::describing` answers `may-go` before the
    /// `region` written beside it for the same reason.
    fn the_way_out(self) -> Result<TheProxy, NotDescribed> {
        let Some(way) = WayOut::named(&self.goes_through) else {
            return Err(NotDescribed::NoWayOutNamedThat {
                said: self.goes_through,
                every: EVERY_WAY.join(", "),
            });
        };
        match way {
            WayOut::Nothing => {
                self.nothing_beside(&[
                    (self.http.is_some(), THE_HTTP),
                    (self.https.is_some(), THE_HTTPS),
                    (self.except.is_some(), THE_EXCEPTIONS),
                    (self.configuration.is_some(), THE_CONFIGURATION),
                    (self.sign_in_as.is_some(), THE_SIGN_IN),
                    (self.password_in_keyring.is_some(), THE_KEYRING_NAME),
                ])?;
                Ok(TheProxy::None)
            }
            WayOut::AnAddress => {
                self.nothing_beside(&[(self.configuration.is_some(), THE_CONFIGURATION)])?;
                self.an_address()
            }
            WayOut::AConfiguration => {
                self.nothing_beside(&[
                    (self.http.is_some(), THE_HTTP),
                    (self.https.is_some(), THE_HTTPS),
                    (self.except.is_some(), THE_EXCEPTIONS),
                    (self.sign_in_as.is_some(), THE_SIGN_IN),
                    (self.password_in_keyring.is_some(), THE_KEYRING_NAME),
                ])?;
                self.a_configuration()
            }
        }
    }

    /// Refuse the first of these keys that is written, because this way out
    /// would ignore it.
    fn nothing_beside(&self, keys: &[(bool, &'static str)]) -> Result<(), NotDescribed> {
        for &(written, key) in keys {
            if written {
                return Err(NotDescribed::AProxyKeyThatDoesNothing {
                    goes_through: self.goes_through.clone(),
                    key,
                });
            }
        }
        Ok(())
    }

    /// An address per scheme, with the places excepted from it.
    fn an_address(self) -> Result<TheProxy, NotDescribed> {
        let signing_in = match (self.sign_in_as, self.password_in_keyring) {
            (None, None) => None,
            (Some(name), Some(kept)) => Some((
                name,
                WhereThePasswordIs::named(&kept)
                    .map_err(|why| NotDescribed::NotAKeyringName { why })?,
            )),
            // A name with nowhere to find its password, or a keyring name with
            // nobody to sign in as, is a setting that would fail on the first
            // road out — `alo_proxy::ProxyAddress::signing_in` takes both — so
            // it fails here, where somebody can still read the file.
            (Some(_), None) => {
                return Err(NotDescribed::HalfASignIn {
                    written: THE_SIGN_IN,
                    missing: THE_KEYRING_NAME,
                });
            }
            (None, Some(_)) => {
                return Err(NotDescribed::HalfASignIn {
                    written: THE_KEYRING_NAME,
                    missing: THE_SIGN_IN,
                });
            }
        };

        let http = an_address_at(self.http.as_deref(), THE_HTTP, signing_in.as_ref())?;
        let https = an_address_at(self.https.as_deref(), THE_HTTPS, signing_in.as_ref())?;
        if http.is_none() && https.is_none() {
            // A manual setting naming neither scheme sends everything straight
            // out while saying it goes through an address, which is the one
            // reading of this section nobody could have meant.
            return Err(NotDescribed::NoProxyAddressNamed);
        }

        let exceptions = match self.except {
            None => Exceptions::none(),
            Some(places) => excepting(&places)?,
        };
        Ok(TheProxy::Manual {
            http,
            https,
            exceptions,
        })
    }

    /// An address an automatic configuration is at.
    fn a_configuration(self) -> Result<TheProxy, NotDescribed> {
        let Some(said) = self.configuration else {
            return Err(NotDescribed::NoConfigurationNamed);
        };
        let at = ConfigurationAddress::checked(&said).map_err(|why| match why {
            NotAConfiguration::CarriesAPassword => NotDescribed::AProxyPasswordInTheFile {
                what: THE_CONFIGURATION,
            },
            _ => NotDescribed::NotAConfigurationAddress { said },
        })?;
        Ok(TheProxy::Automatic { at })
    }
}

/// The proxy written under this key, where one is written at all.
///
/// `signing_in` is the name and the keyring name the whole section states, which
/// is why it is a parameter rather than something read here: a proxy that wants
/// a name wants it on both schemes, and two sign-ins in one section would be two
/// answers to one question.
fn an_address_at(
    said: Option<&str>,
    key: &'static str,
    signing_in: Option<&(String, WhereThePasswordIs)>,
) -> Result<Option<ProxyAddress>, NotDescribed> {
    let Some(said) = said else {
        return Ok(None);
    };
    let address = a_proxy(said, key)?;
    let Some((name, kept)) = signing_in else {
        return Ok(Some(address));
    };
    let address = address
        .signing_in(name, kept.clone())
        .map_err(|why| match why {
            // A name with a password written into it, which is the other half
            // of the paste `a_proxy` refuses in the address itself.
            NotAnAddress::CarriesAPassword => {
                NotDescribed::AProxyPasswordInTheFile { what: THE_SIGN_IN }
            }
            _ => NotDescribed::NotASignInName {
                said: name.to_owned(),
            },
        })?;
    Ok(Some(address))
}

/// `http://host:port` or `https://host:port`, as a proxy.
///
/// Written out as a whole address because that is how a company hands somebody
/// their proxy, and taken apart here rather than by whatever opens the
/// connection later: `alo_proxy::ProxyAddress` is three checked values, and the
/// road from one line of TOML to those three is this function and nowhere else.
fn a_proxy(said: &str, key: &'static str) -> Result<ProxyAddress, NotDescribed> {
    let said = said.trim();
    // Before anything is taken apart, because a pasted credential is the one
    // thing here that is worse than an address that will not parse.
    if said.contains('@') {
        return Err(NotDescribed::AProxyPasswordInTheFile { what: key });
    }
    let (spoken_to, rest) = if let Some(rest) = said.strip_prefix("https://") {
        (SpokenTo::Https, rest)
    } else if let Some(rest) = said.strip_prefix("http://") {
        (SpokenTo::Http, rest)
    } else {
        return Err(not_a_proxy(key, said));
    };
    // `rsplit_once` rather than `split_once`, so that a proxy written as an
    // IPv6 number — `http://[2001:db8::1]:8080` — keeps its colons and gives up
    // only the last one.
    let (host, port) = rest
        .rsplit_once(':')
        .ok_or_else(|| not_a_proxy(key, said))?;
    let port: u16 = port.parse().map_err(|_| not_a_proxy(key, said))?;
    ProxyAddress::checked(spoken_to, host, port).map_err(|why| match why {
        NotAnAddress::CarriesAPassword => NotDescribed::AProxyPasswordInTheFile { what: key },
        _ => not_a_proxy(key, said),
    })
}

/// What is written under this key is not a proxy address.
fn not_a_proxy(key: &'static str, said: &str) -> NotDescribed {
    NotDescribed::NotAProxyAddress {
        what: key,
        said: said.to_owned(),
    }
}

/// The places that go straight out anyway.
///
/// Each is checked on its own first, so that the refusal names the entry
/// somebody has to go and fix: `alo_proxy::Exceptions::of` refuses the whole
/// list — which is the right answer for a settings panel showing the list back —
/// and says nothing about which line of a file it was.
fn excepting(places: &[String]) -> Result<Exceptions, NotDescribed> {
    for place in places {
        Exceptions::of([place.as_str()]).map_err(|_| NotDescribed::NotAnExceptedPlace {
            said: place.clone(),
        })?;
    }
    Exceptions::of(places.iter().map(String::as_str)).map_err(|_| {
        NotDescribed::NotAnExceptedPlace {
            said: places.join(", "),
        }
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_proxy::{Reaching, Scheme, SetBy};

    /// The section as it would arrive from a file.
    fn written(text: &str) -> Result<TheProxySection, toml::de::Error> {
        toml::from_str(text)
    }

    /// That section as the proxy an administrator set.
    fn an_administrators(text: &str) -> Result<Kept, NotDescribed> {
        written(text)
            .unwrap()
            .checked(WhoDescribedIt::AnAdministrator)
    }

    /// A road out to somewhere, for asking the exceptions about.
    fn going_to(host: &str) -> Reaching {
        Reaching::over(Scheme::Https, host).unwrap()
    }

    /// **The address an organisation wrote is the proxy every road out asks**,
    /// on both schemes, and it is the organisation's.
    #[test]
    fn the_address_an_organisation_wrote_is_the_proxy() {
        let kept = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             https = \"http://proxy.example.com:8080\"\n",
        )
        .unwrap();
        assert_eq!(kept.set_by(), SetBy::AnOrganisation);
        for scheme in Scheme::EVERY {
            let address = kept.proxy().for_(scheme).unwrap();
            assert_eq!(address.written(), "http://proxy.example.com:8080");
            assert_eq!(address.name(), None);
        }
    }

    /// **The same section in the person's own file is theirs**, and they may
    /// change it — which is the whole of what `set_by` decides.
    #[test]
    fn the_same_section_the_person_wrote_is_their_own() {
        let kept = written("goes-through = \"nothing\"")
            .unwrap()
            .checked(WhoDescribedIt::ThePerson)
            .unwrap();
        assert_eq!(kept.set_by(), SetBy::ThisPerson);
        assert_eq!(kept.proxy(), &TheProxy::None);
        assert!(kept.changed_by_the_person(TheProxy::None).is_ok());
    }

    /// **Nothing is a way out that was written down**, and it is not the same
    /// answer as no section at all — which never reaches this file.
    #[test]
    fn nothing_is_a_way_out_somebody_wrote() {
        let kept = an_administrators("goes-through = \"nothing\"").unwrap();
        assert_eq!(kept.proxy(), &TheProxy::None);
        assert_eq!(kept.set_by(), SetBy::AnOrganisation);
    }

    /// One scheme may be named and not the other: *straight out for those* is a
    /// configuration a company network really has.
    #[test]
    fn one_scheme_may_be_named_and_not_the_other() {
        let kept = an_administrators(
            "goes-through = \"an-address\"\nhttps = \"https://proxy.example.com:3128\"\n",
        )
        .unwrap();
        assert_eq!(kept.proxy().for_(Scheme::Http), None);
        assert_eq!(
            kept.proxy().for_(Scheme::Https).unwrap().written(),
            "https://proxy.example.com:3128"
        );
    }

    /// The places excepted are read as the list they are, and a host that
    /// merely ends the same way is not one of them.
    #[test]
    fn the_places_excepted_are_read_as_a_list() {
        let kept = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             except = [\"intranet.example.com\", \".example.test\"]\n",
        )
        .unwrap();
        let exceptions = kept.proxy().exceptions().unwrap();
        assert!(exceptions.let_through(&going_to("intranet.example.com")));
        assert!(exceptions.let_through(&going_to("files.example.test")));
        assert!(!exceptions.let_through(&going_to("notexample.test")));
    }

    /// An automatic configuration is the address it is at, and nothing else.
    #[test]
    fn an_automatic_configuration_is_the_address_it_is_at() {
        let kept = an_administrators(
            "goes-through = \"a-configuration\"\nconfiguration = \"http://wpad.example.com/c\"\n",
        )
        .unwrap();
        assert!(kept.proxy().configuration().is_some());
        assert_eq!(kept.proxy().for_(Scheme::Http), None);
    }

    /// **The name and the keyring name reach the address, and the file holds no
    /// password** — ADR 0022 asked of the section rather than of one key.
    #[test]
    fn the_sign_in_names_the_keyring_and_never_a_password() {
        let kept = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             https = \"http://proxy.example.com:8080\"\n\
             sign-in-as = \"anna\"\n\
             password-in-keyring = \"the company proxy\"\n",
        )
        .unwrap();
        for scheme in Scheme::EVERY {
            let address = kept.proxy().for_(scheme).unwrap();
            assert_eq!(address.name(), Some("anna"));
            assert_eq!(
                address.password().map(WhereThePasswordIs::as_str),
                Some("the company proxy")
            );
            assert_eq!(address.written(), "http://proxy.example.com:8080");
        }
    }

    /// **A password written into the file is refused by name**, in both the
    /// shapes somebody would write one — and neither refusal repeats it.
    #[test]
    fn a_password_written_into_the_file_is_refused_by_name() {
        let inline = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             sign-in-as = \"anna\"\n\
             password = \"hunter2\"\n",
        )
        .unwrap_err();
        assert!(
            matches!(inline, NotDescribed::AProxyPasswordInTheFile { what } if what == THE_PASSWORD),
            "{inline:?}"
        );
        assert!(inline.to_string().contains(THE_PASSWORD), "{inline}");
        assert!(inline.to_string().contains(THE_KEYRING_NAME), "{inline}");
        assert!(!inline.to_string().contains("hunter2"), "{inline}");

        let pasted = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://anna:hunter2@proxy.example.com:8080\"\n",
        )
        .unwrap_err();
        assert!(
            matches!(pasted, NotDescribed::AProxyPasswordInTheFile { what } if what == THE_HTTP),
            "{pasted:?}"
        );
        assert!(!pasted.to_string().contains("hunter2"), "{pasted}");

        // And in the name, which is the other half of the same paste.
        let in_the_name = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             sign-in-as = \"anna:hunter2\"\n\
             password-in-keyring = \"the company proxy\"\n",
        )
        .unwrap_err();
        assert!(
            matches!(in_the_name, NotDescribed::AProxyPasswordInTheFile { what } if what == THE_SIGN_IN),
            "{in_the_name:?}"
        );
        assert!(
            !in_the_name.to_string().contains("hunter2"),
            "{in_the_name}"
        );

        // And in the address of an automatic configuration.
        let in_the_configuration = an_administrators(
            "goes-through = \"a-configuration\"\n\
             configuration = \"http://anna:hunter2@wpad.example.com/c\"\n",
        )
        .unwrap_err();
        assert!(
            matches!(in_the_configuration, NotDescribed::AProxyPasswordInTheFile { what } if what == THE_CONFIGURATION),
            "{in_the_configuration:?}"
        );
        assert!(
            !in_the_configuration.to_string().contains("hunter2"),
            "{in_the_configuration}"
        );
    }

    /// **A way out this service does not know is refused**, and it is answered
    /// before whichever key was written beside it.
    #[test]
    fn a_way_out_this_service_does_not_know_is_refused_first() {
        let refused = an_administrators(
            "goes-through = \"an-adress\"\nconfiguration = \"http://wpad.example.com/c\"\n",
        )
        .unwrap_err();
        assert!(
            matches!(refused, NotDescribed::NoWayOutNamedThat { ref said, .. } if said == "an-adress"),
            "{refused:?}"
        );
        for way in EVERY_WAY {
            assert!(refused.to_string().contains(way), "{refused}");
        }
    }

    /// **A key the way out would ignore is refused**, rather than left as a
    /// line somebody believes is sending their traffic through a proxy.
    #[test]
    fn a_key_the_way_out_would_ignore_is_refused() {
        for (section, key) in [
            (
                "goes-through = \"nothing\"\nhttp = \"http://proxy.example.com:8080\"\n",
                THE_HTTP,
            ),
            (
                "goes-through = \"nothing\"\npassword-in-keyring = \"the company proxy\"\n",
                THE_KEYRING_NAME,
            ),
            (
                "goes-through = \"an-address\"\n\
                 http = \"http://proxy.example.com:8080\"\n\
                 configuration = \"http://wpad.example.com/c\"\n",
                THE_CONFIGURATION,
            ),
            (
                "goes-through = \"a-configuration\"\n\
                 configuration = \"http://wpad.example.com/c\"\n\
                 except = [\"example.com\"]\n",
                THE_EXCEPTIONS,
            ),
        ] {
            let refused = an_administrators(section).unwrap_err();
            assert!(
                matches!(refused, NotDescribed::AProxyKeyThatDoesNothing { key: k, .. } if k == key),
                "{key}: {refused:?}"
            );
            assert!(refused.to_string().contains(key), "{refused}");
        }
    }

    /// **A way out with nothing to go through is refused**, in both shapes: an
    /// address that names none, and a configuration that is nowhere.
    #[test]
    fn a_way_out_with_nothing_to_go_through_is_refused() {
        assert!(matches!(
            an_administrators("goes-through = \"an-address\"").unwrap_err(),
            NotDescribed::NoProxyAddressNamed
        ));
        assert!(matches!(
            an_administrators("goes-through = \"a-configuration\"").unwrap_err(),
            NotDescribed::NoConfigurationNamed
        ));
    }

    /// **Half a sign-in is refused**, because a proxy that wants a name and has
    /// nowhere to find its password fails on the first road out.
    #[test]
    fn half_a_sign_in_is_refused() {
        let no_password = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             sign-in-as = \"anna\"\n",
        )
        .unwrap_err();
        assert!(
            matches!(no_password, NotDescribed::HalfASignIn { written, missing }
                if written == THE_SIGN_IN && missing == THE_KEYRING_NAME),
            "{no_password:?}"
        );

        let no_name = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             password-in-keyring = \"the company proxy\"\n",
        )
        .unwrap_err();
        assert!(
            matches!(no_name, NotDescribed::HalfASignIn { written, missing }
                if written == THE_KEYRING_NAME && missing == THE_SIGN_IN),
            "{no_name:?}"
        );
    }

    /// **Something that is not a proxy address is refused where it was typed**,
    /// naming the key, rather than in the middle of somebody's work by whatever
    /// opens the connection.
    #[test]
    fn something_that_is_not_a_proxy_address_is_refused_by_its_key() {
        for said in [
            "proxy.example.com:8080",
            "socks://proxy.example.com:1080",
            "http://proxy.example.com",
            "http://proxy.example.com:0",
            "http://proxy.example.com:99999",
            "http://proxy.example.com:8080/go",
            "http://proxy_example.com:8080",
            "",
        ] {
            let refused = an_administrators(&format!(
                "goes-through = \"an-address\"\nhttp = \"{said}\"\n"
            ))
            .unwrap_err();
            assert!(
                matches!(refused, NotDescribed::NotAProxyAddress { what, .. } if what == THE_HTTP),
                "{said:?}: {refused:?}"
            );
            assert!(refused.to_string().contains(THE_HTTP), "{refused}");
        }
    }

    /// A proxy written as an IPv6 number keeps its colons, which is the one
    /// address shape taking the last colon rather than the first exists for.
    #[test]
    fn a_proxy_written_as_an_ipv6_number_is_a_proxy() {
        let kept = an_administrators(
            "goes-through = \"an-address\"\nhttp = \"http://[2001:db8::1]:8080\"\n",
        )
        .unwrap();
        assert_eq!(
            kept.proxy().for_(Scheme::Http).unwrap().written(),
            "http://[2001:db8::1]:8080"
        );
    }

    /// **A place excepted that is not one is refused, and the refusal names
    /// it** — the whole list is never half-honoured.
    #[test]
    fn a_place_excepted_that_is_not_one_is_refused_and_named() {
        let refused = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             except = [\"intranet.example.com\", \"build box\"]\n",
        )
        .unwrap_err();
        assert!(
            matches!(refused, NotDescribed::NotAnExceptedPlace { ref said } if said == "build box"),
            "{refused:?}"
        );
        assert!(refused.to_string().contains("build box"), "{refused}");
    }

    /// **A keyring name that is not one is refused**, in the words the crate
    /// that keeps it already wrote.
    #[test]
    fn a_keyring_name_that_is_not_one_is_refused() {
        let refused = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             sign-in-as = \"anna\"\n\
             password-in-keyring = \"   \"\n",
        )
        .unwrap_err();
        assert!(
            matches!(refused, NotDescribed::NotAKeyringName { .. }),
            "{refused:?}"
        );
        assert!(refused.to_string().contains(THE_KEYRING_NAME), "{refused}");
    }

    /// **A name the proxy could not be signed in as is refused**, by its own
    /// key rather than as an address — they are two lines to go and fix.
    #[test]
    fn a_name_the_proxy_could_not_be_signed_in_as_is_refused() {
        let refused = an_administrators(
            "goes-through = \"an-address\"\n\
             http = \"http://proxy.example.com:8080\"\n\
             sign-in-as = \"   \"\n\
             password-in-keyring = \"the company proxy\"\n",
        )
        .unwrap_err();
        assert!(
            matches!(refused, NotDescribed::NotASignInName { .. }),
            "{refused:?}"
        );
        assert!(refused.to_string().contains(THE_SIGN_IN), "{refused}");
    }

    /// **The shape is refused by the reader**: no way out, and a key nobody
    /// declared.
    #[test]
    fn a_section_not_in_this_shape_is_refused_by_the_reader() {
        assert!(
            written("http = \"http://proxy.example.com:8080\"")
                .unwrap_err()
                .to_string()
                .contains("goes-through")
        );
        assert!(
            written("goes-through = \"nothing\"\nno-proxy = [\"x\"]")
                .unwrap_err()
                .to_string()
                .contains("no-proxy")
        );
        assert!(written("goes-through = 3").is_err());
    }

    /// The keys are named once, because each of them is in a refusal somebody
    /// reads and in the contract they wrote the file against.
    #[test]
    fn the_keys_are_named_once() {
        assert_eq!(THE_WAY_OUT, "proxy.goes-through");
        assert_eq!(THE_HTTP, "proxy.http");
        assert_eq!(THE_HTTPS, "proxy.https");
        assert_eq!(THE_EXCEPTIONS, "proxy.except");
        assert_eq!(THE_CONFIGURATION, "proxy.configuration");
        assert_eq!(THE_SIGN_IN, "proxy.sign-in-as");
        assert_eq!(THE_KEYRING_NAME, "proxy.password-in-keyring");
        assert_eq!(THE_PASSWORD, "proxy.password");
        assert_eq!(EVERY_WAY, ["nothing", "an-address", "a-configuration"]);
    }
}
