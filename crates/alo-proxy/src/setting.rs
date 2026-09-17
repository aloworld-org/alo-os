//! The one proxy setting this machine has.
//!
//! **One**, and that is the whole of the first word of this task. Not one per
//! application, not one for the agent and another for the browser, not one the
//! shell keeps and another a daemon reads: [`TheProxy`] is the machine's, every
//! road out asks it (`deciding.rs`), and applications are handed the same answer
//! (`published.rs`, `portal.rs`). A machine with two proxy settings is a machine
//! where a person has fixed the one they could find and is still wondering why
//! half of it does not work.
//!
//! # Three shapes, and there is no fourth
//!
//! | | |
//! |---|---|
//! | [`TheProxy::None`] | every road goes straight out |
//! | [`TheProxy::Manual`] | an address per scheme, and the places excepted from it |
//! | [`TheProxy::Automatic`] | an address a configuration is at, which answers per road |
//!
//! There is no member meaning *work it out from the environment*. What a
//! process happened to be started with is not a setting a person chose, cannot
//! be shown to them, and cannot be changed where they would look for it —
//! `alo-secrets` refuses to read an environment for the same reason, in the
//! file that says a function taking a uid cannot be pointed somewhere by a
//! variable.
//!
//! # A setting holds where a password is, and never a password
//!
//! [`crate::ProxyAddress`] is what it holds, and that type has a
//! [`crate::WhereThePasswordIs`] in it and nothing that could be a credential
//! (ADR 0022). So a setting written to a file, read back, shown, or sent to
//! whoever administers the machine carries a name to look up. The test at the
//! bottom of this file is what says so, against the whole setting rather than
//! against the address alone.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::address::ProxyAddress;
use crate::automatic::ConfigurationAddress;
use crate::exceptions::Exceptions;
use crate::reaching::Scheme;
use crate::words;

/// The proxy this machine uses.
///
/// Deliberately no `Default`: a machine is *told* what its proxy is, and *none*
/// is something written down rather than something nobody got round to. That is
/// `alo_software::Bound`'s argument, and the same one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "proxy")]
pub enum TheProxy {
    /// No proxy: every road out goes straight.
    None,
    /// An address per scheme, and the places excepted from it.
    Manual {
        /// Where roads spoken in clear go. [`Option::None`] means *straight
        /// out for those*, which is a real configuration and not an omission.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        http: Option<ProxyAddress>,
        /// Where roads spoken encrypted go.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        https: Option<ProxyAddress>,
        /// The places that go straight out anyway.
        #[serde(default)]
        exceptions: Exceptions,
    },
    /// An address a configuration is at, which answers per road.
    Automatic {
        /// Where the configuration is.
        at: ConfigurationAddress,
    },
}

impl TheProxy {
    /// One address for both schemes, with nothing excepted.
    ///
    /// The shape a person sets when somebody has handed them one address on a
    /// slip of paper, which is most of the time.
    #[must_use]
    pub fn one(address: ProxyAddress) -> Self {
        Self::Manual {
            http: Some(address.clone()),
            https: Some(address),
            exceptions: Exceptions::none(),
        }
    }

    /// The same, with these places excepted from it.
    #[must_use]
    pub fn excepting(self, exceptions: Exceptions) -> Self {
        match self {
            Self::Manual { http, https, .. } => Self::Manual {
                http,
                https,
                exceptions,
            },
            other => other,
        }
    }

    /// The address set for this scheme, where the setting is a manual one.
    #[must_use]
    pub fn for_(&self, scheme: Scheme) -> Option<&ProxyAddress> {
        match (self, scheme) {
            (Self::Manual { http, .. }, Scheme::Http) => http.as_ref(),
            (Self::Manual { https, .. }, Scheme::Https) => https.as_ref(),
            (Self::None | Self::Automatic { .. }, _) => Option::None,
        }
    }

    /// The places excepted, which is nothing at all for the other two shapes.
    #[must_use]
    pub fn exceptions(&self) -> Option<&Exceptions> {
        match self {
            Self::Manual { exceptions, .. } => Some(exceptions),
            Self::None | Self::Automatic { .. } => Option::None,
        }
    }

    /// Where the configuration is, where the setting is an automatic one.
    #[must_use]
    pub fn configuration(&self) -> Option<&ConfigurationAddress> {
        match self {
            Self::Automatic { at } => Some(at),
            Self::None | Self::Manual { .. } => Option::None,
        }
    }

    /// The string this crate declares for what a person is shown about this
    /// setting.
    #[must_use]
    pub const fn word(&self) -> words::Word {
        match self {
            Self::None => words::THE_PROXY_NONE,
            Self::Manual { .. } => words::THE_PROXY_MANUAL,
            Self::Automatic { .. } => words::THE_PROXY_AUTOMATIC,
        }
    }

    /// What a person is shown about this setting, in the language they read.
    ///
    /// The address itself is never inside the sentence: it is data shown beside
    /// it, as it was written.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::address::SpokenTo;
    use crate::password::WhereThePasswordIs;

    fn an_address() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
    }

    /// One address on a slip of paper becomes one setting covering both
    /// schemes, which is what a person means when they type it once.
    #[test]
    fn one_address_covers_both_schemes() {
        let setting = TheProxy::one(an_address());
        for scheme in Scheme::EVERY {
            assert_eq!(setting.for_(scheme), Some(&an_address()), "{scheme:?}");
        }
        assert!(setting.exceptions().is_some_and(Exceptions::is_none));
        assert_eq!(setting.configuration(), None);
    }

    /// A manual setting may name one scheme and not the other, and *straight
    /// out for those* is a configuration rather than an omission.
    #[test]
    fn a_manual_setting_may_name_one_scheme_and_not_the_other() {
        let setting = TheProxy::Manual {
            http: None,
            https: Some(an_address()),
            exceptions: Exceptions::none(),
        };
        assert_eq!(setting.for_(Scheme::Http), None);
        assert_eq!(setting.for_(Scheme::Https), Some(&an_address()));
    }

    /// No proxy, and an automatic one, each answer nothing about a scheme —
    /// the way out for those is `deciding.rs`'s and not a field here.
    #[test]
    fn the_other_two_shapes_name_no_address_per_scheme() {
        let at = ConfigurationAddress::checked("http://wpad.example.com/c").unwrap();
        for setting in [TheProxy::None, TheProxy::Automatic { at: at.clone() }] {
            for scheme in Scheme::EVERY {
                assert_eq!(setting.for_(scheme), None);
            }
            assert_eq!(setting.exceptions(), None);
        }
        assert_eq!(
            TheProxy::Automatic { at: at.clone() }.configuration(),
            Some(&at)
        );
        assert_eq!(TheProxy::None.configuration(), None);
    }

    /// Exceptions belong to a manual setting and are left alone by the other
    /// two: an automatic configuration decides its own, and there is nothing to
    /// except from no proxy at all.
    #[test]
    fn exceptions_are_a_manual_settings_and_are_left_alone_by_the_others() {
        let excepted = Exceptions::of(["example.com"]).unwrap();
        let manual = TheProxy::one(an_address()).excepting(excepted.clone());
        assert_eq!(manual.exceptions(), Some(&excepted));
        assert_eq!(TheProxy::None.excepting(excepted), TheProxy::None);
    }

    /// **Written down, the whole setting carries where a password is kept and
    /// never a password** — ADR 0022 held against the setting rather than
    /// against one address in it.
    #[test]
    fn the_whole_setting_written_down_carries_no_password() {
        let setting = TheProxy::one(
            an_address()
                .signing_in(
                    "anna",
                    WhereThePasswordIs::named("the company proxy").unwrap(),
                )
                .unwrap(),
        )
        .excepting(Exceptions::of(["example.com"]).unwrap());
        let written = serde_json::to_string(&setting).unwrap();
        assert!(written.contains("the company proxy"), "{written}");
        assert!(!written.contains("hunter2"), "{written}");
        assert_eq!(
            serde_json::from_str::<TheProxy>(&written).ok(),
            Some(setting)
        );
    }

    /// Each shape survives being written down and read back, which is what
    /// *machine-wide* needs: one setting, outliving whatever set it.
    #[test]
    fn each_shape_survives_being_written_down_and_read_back() {
        for setting in [
            TheProxy::None,
            TheProxy::one(an_address()),
            TheProxy::Automatic {
                at: ConfigurationAddress::checked("https://wpad.example.com/c").unwrap(),
            },
        ] {
            let written = serde_json::to_string(&setting).unwrap();
            assert_eq!(
                serde_json::from_str::<TheProxy>(&written).ok(),
                Some(setting),
                "{written}"
            );
        }
    }
}
