//! The entry itself: an address, a region, and the chain it discloses.

use alo_models::{Provider, ProviderError, Region, SecretRef};

/// What a person sees when it answers. A name, like any provider's.
pub const ITS_NAME: &str = "alo";

/// Where its API is.
pub const ITS_ADDRESS: &str = "https://api.alo.computer/v1";

/// **Where it says it runs** — as the service states it, which is what
/// `Region::Declared` means for every provider. This machine cannot verify a
/// region, ours no more than anybody's, and a region we checked differently
/// would be the special case ADR 0014 forbids.
pub const WHERE_IT_SAYS_IT_RUNS: &str = "France";

/// **The chain it discloses**: who answers, with whose model, where.
///
/// ADR 0014's disclosure rule. A person asking alo a question is told
/// *answered by alo, using Mistral, in France* — the same sentence a reseller
/// must show, because reselling somebody else's model without saying so is what
/// this line exists to prevent, and it applies to us first.
pub const THE_CHAIN_IT_DISCLOSES: &str = "answered by alo, using Mistral, in France";

/// **The entry**, built through the same door a person's own provider is built
/// through.
///
/// # Errors
/// [`ProviderError`] exactly as any provider's would be. It is checked rather
/// than trusted for the same reason: this crate is data, and data can be wrong.
pub fn entry(key: Option<SecretRef>) -> Result<Provider, ProviderError> {
    Provider::checked(
        ITS_NAME,
        ITS_ADDRESS,
        Region::Declared(WHERE_IT_SAYS_IT_RUNS.to_owned()),
        key,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **It is a provider, built the way every provider is built.**
    #[test]
    fn it_is_an_ordinary_provider_entry() {
        let provider = entry(None).unwrap();
        assert_eq!(provider.name, ITS_NAME);
        assert_eq!(provider.endpoint, ITS_ADDRESS);
        assert_eq!(
            provider.region,
            Region::Declared(WHERE_IT_SAYS_IT_RUNS.to_owned())
        );
        assert!(
            provider.key.is_none(),
            "a key is a person's, not ours to ship"
        );
    }

    /// **Its address leaves this machine, and nothing here pretends otherwise.**
    #[test]
    fn its_address_is_not_this_machine() {
        let provider = entry(None).unwrap();
        assert_ne!(
            provider.source(),
            alo_models::InferenceSource::ThisMachine,
            "alo's own service reported as this machine would be the largest possible lie about \
             where a question went"
        );
        assert!(ITS_ADDRESS.starts_with("https://"));
    }

    /// **The chain says who answers, whose model, and where** — the sentence a
    /// reseller owes, which we owe first.
    #[test]
    fn the_chain_it_discloses_names_who_whose_model_and_where() {
        assert!(THE_CHAIN_IT_DISCLOSES.contains("alo"));
        assert!(THE_CHAIN_IT_DISCLOSES.contains("Mistral"));
        assert!(THE_CHAIN_IT_DISCLOSES.contains(WHERE_IT_SAYS_IT_RUNS));
    }
}
