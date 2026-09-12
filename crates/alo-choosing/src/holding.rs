//! Every provider about to be written is held to its own crate's rule again,
//! at the one place a provider is written.
//!
//! `alo_models::Provider::checked` is where an address is judged: not one at
//! all, or `http://` to anywhere but this machine — *a key on the wire in
//! clear* — is refused there, in that crate's own words. The trouble is that
//! `alo_models::Provider` has public fields. A settings surface that builds one
//! by hand, or that copies a checked one and edits its address, holds a value
//! nothing has judged, and `crate::Choosing` took that value on the strength of
//! a type whose name says *checked*.
//!
//! Until this file, such a provider was refused all the same — but by
//! `crate::writing`'s round trip, which reads the text back through the
//! reader and finds the reader will not take it. That is the wrong refusal:
//! `choosing.change.not-expressible` says *this alo OS could not write that
//! choice*, which is a defect in alo OS, and what happened is that a person
//! typed `http://` in front of an address on their own network. They read a
//! sentence about a fault in the machine and never the one telling them what to
//! change.
//!
//! # The one place, so no second path can bypass it
//!
//! It is asked in [`crate::writing::written`], which is the only function that
//! turns settings into the text of a file, and which every door of
//! `crate::Choosing` — present and future — goes through on its way to the
//! disk. A check at `Choosing::adding` alone would hold for that door and be
//! forgotten at the next one; a check here holds because there is no other way
//! to the file. The rule is `docs/features.md`'s v0.5 line: *an address that is
//! not https is refused rather than warned about, unless it is a service on
//! this machine.*
//!
//! # Nothing here decides what an address is
//!
//! This file names no scheme, no host and no exception. What is loopback is
//! `alo_models::address`'s answer, and what `Provider::checked` refuses is
//! that crate's, asked again rather than copied — so `127.0.0.1`, `::1` and
//! `localhost` are accepted over `http://` for ADR 0021's reason and an
//! address on the machine's own network is not, and both are decided in one
//! place. There is no allow-list of hostnames here and nothing reads an
//! environment variable; `tests/an_address_that_is_not_https.rs` reads this
//! file to make sure of it.
//!
//! # What it does not change
//!
//! How a provider already in somebody's file is **read**. `crate::written`
//! builds every provider it reads through `Provider::checked` and has since
//! providers existed, so a file with such an address in it is refused whole on
//! the way in exactly as it always was, and nothing here rewrites a person's
//! file behind them. The refusal this file adds is at the next write.

use alo_models::ProviderError;

use crate::settings::Settings;

/// Every provider these settings would write, held to `alo_models`' rule.
///
/// # Errors
///
/// The first provider on the list that `alo_models::Provider::checked` would
/// not have made, and why — in `alo-models`' own words, because it is that
/// crate's rule and this one adds nothing to it.
pub(crate) fn every_provider_holds(settings: &Settings) -> Result<(), ProviderError> {
    for provider in &settings.providers().configured {
        alo_models::Provider::checked(
            &provider.name,
            &provider.endpoint,
            provider.region.clone(),
            provider.key.clone(),
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_models::{Brought, Provider, Providers, Region};

    use crate::setup::Setup;

    /// A provider built the way nothing should build one: by hand, with an
    /// address nobody has judged.
    fn by_hand(endpoint: &str) -> Provider {
        Provider {
            name: "Somewhere".to_owned(),
            endpoint: endpoint.to_owned(),
            region: Region::Unknown,
            key: Some(crate::written::a_key_for("Somewhere")),
            models: Vec::new(),
        }
    }

    /// Settings holding exactly this provider.
    fn settings_with(provider: Provider) -> Settings {
        let mut providers = Providers::default();
        providers.add(provider).unwrap();
        Settings::of(
            None,
            Brought::default(),
            providers,
            Vec::new(),
            Setup::NotAnswered,
        )
        .unwrap()
    }

    /// **An address that is not https is refused**, in `alo-models`' words,
    /// even when the value arrived past `Provider::checked`.
    #[test]
    fn an_address_that_is_not_https_does_not_hold() {
        for endpoint in [
            "http://api.example.com",
            "http://192.168.1.10:11434",
            "http://10.0.0.5",
            "http://nas.local:8080/v1",
        ] {
            assert_eq!(
                every_provider_holds(&settings_with(by_hand(endpoint))).unwrap_err(),
                ProviderError::InsecureEndpoint,
                "{endpoint}"
            );
        }
    }

    /// **And so is something that is not an address at all**, which is the
    /// other way a by-hand value can be wrong.
    #[test]
    fn something_that_is_not_an_address_does_not_hold() {
        for endpoint in ["api.example.com", "ftp://api.example.com", ""] {
            assert_eq!(
                every_provider_holds(&settings_with(by_hand(endpoint))).unwrap_err(),
                ProviderError::NotAnAddress,
                "{endpoint:?}"
            );
        }
    }

    /// **A service on this machine holds on any scheme**, and so does any
    /// https address — this file adds no rule of its own.
    #[test]
    fn this_machine_and_https_hold() {
        for endpoint in [
            "http://127.0.0.1:11434",
            "http://[::1]:8080",
            "http://localhost:8000/v1",
            "https://127.0.0.1:8443",
            "https://api.mistral.ai",
            "https://192.168.1.10:8443",
        ] {
            assert_eq!(
                every_provider_holds(&settings_with(by_hand(endpoint))),
                Ok(()),
                "{endpoint}"
            );
        }
    }

    /// **Settings with no provider in them hold**, which is most of them.
    #[test]
    fn settings_with_no_provider_hold() {
        assert_eq!(every_provider_holds(&Settings::untouched()), Ok(()));
    }
}
