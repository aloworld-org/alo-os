//! A provider somebody added themselves — an API they already pay for.
//!
//! [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) makes a
//! hosted API a first-class place for a question to be answered, and says alo OS
//! ships no default that chooses one. This is the other half of that: the
//! settings surface where a person adds Mistral, or their own endpoint, or
//! whatever they already have a contract with.
//!
//! Two things shape the type, and both are security decisions rather than
//! conveniences.
//!
//! **The key is never here.** A [`Provider`] holds a *reference* to a secret in
//! the keyring, never the secret. That is not tidiness: a configuration
//! structure ends up in logs, in error reports, in backups and in a support
//! bundle somebody emails, and a key that was never in it cannot leak from any
//! of those. `CLAUDE.md` says credentials never appear in logs or errors; the
//! reliable way to keep that promise is to make it structurally impossible
//! rather than remembering it at every call site.
//!
//! **The region is stated, never guessed.** Whoever adds a provider says where
//! it runs, or does not. `api.example.fr` is not evidence of anything, and a
//! product that inferred a region from a domain name would hand somebody a
//! reassuring label while putting them in breach.

use serde::{Deserialize, Serialize};

use crate::source::{InferenceSource, Region};

/// Where a key lives in the keyring. Not the key.
///
/// An opaque handle so that nothing downstream can mistake it for a credential
/// or be tempted to log it "just to see".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretRef(String);

impl SecretRef {
    /// A reference to the secret stored under this name.
    #[must_use]
    pub fn named(name: &str) -> Self {
        Self(name.trim().to_owned())
    }

    /// The name to look up in the keyring.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why a provider could not be added.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProviderError {
    /// A provider with no name cannot be shown to anybody or chosen.
    #[error("give the provider a name — it is what you will see when it answers")]
    Unnamed,
    /// The endpoint is not a URL we can use.
    #[error("that does not look like an address: it should start with https://")]
    NotAnAddress,
    /// A key would travel in clear over this connection.
    #[error(
        "this address is not https, so your key and your questions would travel unencrypted — use https, or a service on this machine"
    )]
    InsecureEndpoint,
    /// The same name is already configured, so an answer could not say which
    /// one produced it.
    #[error("you already have a provider called {0}")]
    AlreadyAdded(String),
}

/// A provider a person configured.
///
/// `Serialize` on purpose: this is written to a settings file — which is
/// exactly why it holds [`SecretRef`] and not a key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provider {
    /// What a person calls it, and what they will see when it answers.
    pub name: String,
    /// The base address of its API.
    pub endpoint: String,
    /// Where it runs, **as whoever added it stated**. Unknown when they did
    /// not, which is a fact rather than a failure.
    pub region: Region,
    /// Where the key lives, when the service needs one. Absent for services
    /// that do not, such as a runtime on this machine.
    pub key: Option<SecretRef>,
    /// The model names this provider offers, as it names them.
    #[serde(default)]
    pub models: Vec<String>,
}

impl Provider {
    /// Check a provider a person has just typed in.
    ///
    /// # Errors
    /// [`ProviderError`] describing what to fix, in words that say what to do
    /// rather than what went wrong.
    pub fn checked(
        name: &str,
        endpoint: &str,
        region: Region,
        key: Option<SecretRef>,
    ) -> Result<Self, ProviderError> {
        let name = name.trim();
        let endpoint = endpoint.trim().trim_end_matches('/');
        if name.is_empty() {
            return Err(ProviderError::Unnamed);
        }
        if !endpoint.starts_with("https://") && !endpoint.starts_with("http://") {
            return Err(ProviderError::NotAnAddress);
        }
        // http is allowed only to this machine. A local runtime on loopback
        // never leaves the machine, so there is nothing to encrypt; anything
        // else over http would put the key and the question on the wire in
        // clear, and "it is only our internal network" is how that gets
        // shipped.
        if endpoint.starts_with("http://") && !is_loopback(endpoint) {
            return Err(ProviderError::InsecureEndpoint);
        }
        Ok(Self {
            name: name.to_owned(),
            endpoint: endpoint.to_owned(),
            region,
            key,
            models: Vec::new(),
        })
    }

    /// How this provider appears when it answers something (ADR 0008).
    #[must_use]
    pub fn source(&self) -> InferenceSource {
        if is_loopback(&self.endpoint) {
            return InferenceSource::ThisMachine;
        }
        InferenceSource::Hosted {
            provider: self.name.clone(),
            region: self.region.clone(),
        }
    }
}

/// Whether an address points at this machine.
fn is_loopback(endpoint: &str) -> bool {
    let host = endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    host.starts_with("127.0.0.1")
        || host.starts_with("localhost")
        || host.starts_with("[::1]")
        || host.starts_with("::1")
}

/// The providers configured on this machine.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Providers {
    /// In the order they were added.
    #[serde(default)]
    pub configured: Vec<Provider>,
}

impl Providers {
    /// Add one, refusing a name that is already taken.
    ///
    /// # Errors
    /// [`ProviderError::AlreadyAdded`] when the name is in use — two providers
    /// called the same thing would make "answered by X" ambiguous, and the
    /// whole point of naming the source is that it is not.
    pub fn add(&mut self, provider: Provider) -> Result<(), ProviderError> {
        if self
            .configured
            .iter()
            .any(|p| p.name.eq_ignore_ascii_case(&provider.name))
        {
            return Err(ProviderError::AlreadyAdded(provider.name));
        }
        self.configured.push(provider);
        Ok(())
    }

    /// Remove one by name, saying whether it was there.
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.configured.len();
        self.configured
            .retain(|p| !p.name.eq_ignore_ascii_case(name.trim()));
        self.configured.len() != before
    }

    /// One by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Provider> {
        self.configured
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name.trim()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on a None is the failure being reported"
)]
mod tests {
    use super::*;

    fn eu() -> Region {
        Region::Declared("the EU".to_owned())
    }

    /// The load-bearing property: a settings structure that is written to disk,
    /// logged and included in support bundles never contains a key.
    #[test]
    fn a_key_is_never_in_the_provider_only_a_reference_to_it() {
        let p = Provider::checked(
            "Mistral",
            "https://api.mistral.ai",
            eu(),
            Some(SecretRef::named("provider/mistral")),
        )
        .unwrap();

        let debugged = format!("{p:?}");
        let serialised = serde_json::to_string(&p).unwrap();
        for rendering in [debugged, serialised] {
            assert!(rendering.contains("provider/mistral"), "{rendering}");
            // The type cannot hold a key, so no rendering of it can leak one.
            // This test exists to fail loudly if somebody ever adds a field.
            assert!(!rendering.to_lowercase().contains("sk-"), "{rendering}");
            assert!(
                !rendering.to_lowercase().contains("secret\":\""),
                "{rendering}"
            );
        }
    }

    /// A key over http is a key on the wire in clear. "It is only our internal
    /// network" is how that gets shipped, so it is refused rather than warned
    /// about.
    #[test]
    fn an_unencrypted_address_is_refused_unless_it_is_this_machine() {
        let err = Provider::checked("Somewhere", "http://api.example.com", Region::Unknown, None)
            .unwrap_err();
        assert_eq!(err, ProviderError::InsecureEndpoint);
        assert!(err.to_string().contains("unencrypted"), "{err}");

        // A runtime on this machine never puts anything on a wire.
        assert!(
            Provider::checked("Local", "http://127.0.0.1:11434", Region::Unknown, None).is_ok()
        );
        assert!(
            Provider::checked("Local", "http://localhost:11434", Region::Unknown, None).is_ok()
        );
    }

    /// The region is stated or unknown. `api.example.fr` is not evidence.
    #[test]
    fn a_region_is_never_guessed_from_the_address() {
        let p = Provider::checked("Somewhere", "https://api.example.fr", Region::Unknown, None)
            .unwrap();
        assert_eq!(p.region, Region::Unknown);
        assert!(!p.source().is_in("the EU"));
        assert!(
            p.source().describe().contains("has not said where it runs"),
            "{}",
            p.source().describe()
        );
    }

    /// A provider on this machine is not "hosted" however it was typed in: the
    /// answer never leaves, and the indicator must say so.
    #[test]
    fn a_provider_on_this_machine_reports_as_this_machine() {
        let p =
            Provider::checked("Local", "http://127.0.0.1:11434", Region::Unknown, None).unwrap();
        assert_eq!(p.source(), InferenceSource::ThisMachine);
        assert!(!p.source().causes_egress());
    }

    #[test]
    fn a_provider_needs_a_name_and_an_address() {
        assert_eq!(
            Provider::checked("  ", "https://api.example.com", Region::Unknown, None).unwrap_err(),
            ProviderError::Unnamed
        );
        assert_eq!(
            Provider::checked("Somewhere", "api.example.com", Region::Unknown, None).unwrap_err(),
            ProviderError::NotAnAddress
        );
    }

    /// Two providers with one name would make "answered by X" ambiguous, and
    /// naming the source is the whole point.
    #[test]
    fn the_same_name_cannot_be_added_twice() {
        let mut ps = Providers::default();
        let one = Provider::checked("Mistral", "https://api.mistral.ai", eu(), None).unwrap();
        let two = Provider::checked("mistral", "https://other.example.com", eu(), None).unwrap();
        assert!(ps.add(one).is_ok());
        assert_eq!(
            ps.add(two).unwrap_err(),
            ProviderError::AlreadyAdded("mistral".to_owned())
        );
    }

    #[test]
    fn a_provider_can_be_found_and_removed_by_name_however_it_is_capitalised() {
        let mut ps = Providers::default();
        ps.add(Provider::checked("Mistral", "https://api.mistral.ai", eu(), None).unwrap())
            .unwrap();
        assert!(ps.get("MISTRAL").is_some());
        assert!(ps.remove(" mistral "));
        assert!(ps.get("Mistral").is_none());
        assert!(!ps.remove("mistral"));
    }

    /// The error text is read by somebody who has just mistyped something, so
    /// it says what to do rather than what is wrong.
    #[test]
    fn the_errors_say_what_to_do() {
        assert!(
            ProviderError::Unnamed
                .to_string()
                .contains("give the provider a name")
        );
        assert!(ProviderError::NotAnAddress.to_string().contains("https://"));
    }
}
