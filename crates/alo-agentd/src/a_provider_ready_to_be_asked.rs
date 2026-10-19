//! Rule, then road, then key — once, for every road that asks a provider.
//!
//! A question going to a provider has three things decided before it goes, and
//! **the order they are decided in is the point**, not an implementation
//! detail:
//!
//! 1. **the organisation's rule**, from `alo_answering::Answering::chosen`. A
//!    question the organisation's bound refuses is refused before the keyring
//!    is opened, so a policy refusal never reaches for the person's credential
//!    — there is nothing to reach it for, and a store touched on the way to
//!    saying no is a store touched for nothing.
//! 2. **which way out of this machine the road goes**, from
//!    [`TheRoadOut::to`]. Asked after the rule and before the key, which is
//!    the same ordering argument twice: a question with no road to go on never
//!    reaches for the person's credential. A road that could not be decided is
//!    **refused** rather than quietly going straight out — on a company
//!    network that would be a machine sending its company's traffic around the
//!    company's own rule with nobody told.
//! 3. **the key**, from the keyring, by the name `alo_models::SecretRef`
//!    holds. This is the only place in the daemon where a credential exists at
//!    all, and it lives no longer than [`a_provider_ready_to_be_asked`]'s own
//!    frame.
//!
//! # Why this is a file and not three lines in each caller
//!
//! Two roads reach a provider: an agent's question, through
//! [`crate::doing`], and a person's own question, which ADR 0125 put on the
//! daemon rather than in the command. **Two orderings of *rule, road, key* is
//! how one of them one day reaches for a credential to answer a question a
//! rule had already refused**, which is exactly the fault the ordering above
//! exists to make impossible. So there is one ordering, here, and the roads
//! differ only in the words they refuse in.
//!
//! This is the same argument that made `alo_turn`'s egress sequence one file
//! rather than six turn methods, and it is made the same way: **a closure**.
//! The road and the key are borrowed by the `alo_asking::Hosted` that goes to
//! the model, so they cannot be handed back to a caller to hold; instead the
//! caller's own work runs *inside* the frame that holds them, and they are
//! gone when it returns.
//!
//! # What this file does not do
//!
//! It words nothing. [`NotPrepared`] carries the three refusals out as the
//! values that decided them, because an agent and a person are told different
//! things by different crates — and a refusal rendered here would be a second
//! place those sentences are chosen.

use alo_answering::Answering;
use alo_asking::Hosted;
use alo_models::{NotAllowed, Provider, SourcePolicy};
use alo_secrets::NotStored;
use alo_turn::Answers;

use crate::questions::{Questions, WhoseKeyring};
use crate::the_road_out::{NotTaken, TheRoadOut};

/// Why a provider question never left this machine, as the value that decided
/// it.
///
/// Three different things for a person to go and do, so three variants and not
/// one sentence: a rule somebody set, a road this machine could not decide, and
/// a key the store would not give up. Each caller words its own, because an
/// agent at a socket and a person at a terminal are told these by different
/// crates.
#[derive(Debug)]
pub enum NotPrepared {
    /// The organisation's bound refuses the place this person chose.
    ///
    /// Carried before anything opened the keyring, which is the whole reason
    /// the rule is asked first.
    ARuleRefused(NotAllowed),
    /// This machine could not decide which way out the question would go.
    ///
    /// Not *there is no road* — a question that should go straight out has a
    /// road and takes it. This is a machine that was asked and could not say,
    /// and a question is refused rather than sent on a road nobody chose.
    NoRoadOut(NotTaken),
    /// The keyring would not give up the key the provider is named with.
    ///
    /// Nothing was sent. `alo_secrets::NotStored` is the store's own statement
    /// of which of the four reasons it was.
    TheKeyIsNotStored(NotStored),
}

/// Rule, road and key, in that order, with the caller's work run inside.
///
/// `then` receives what the rule permitted and the `alo_turn::Answers` the
/// question goes to, and runs while the road and the key are still alive. It
/// is not called at all when any of the three refuses, and the refusal comes
/// back as [`NotPrepared`] for the caller to word.
///
/// # The borrowing is the reason for the shape
///
/// `alo_asking::Hosted` borrows the provider, the key and the road, so the
/// three cannot be returned for a caller to assemble — a key handed back is a
/// key living longer than the question that needed it. Running the caller's
/// work in here keeps the credential's whole life inside one frame.
pub fn a_provider_ready_to_be_asked<T>(
    provider: &Provider,
    policy: &SourcePolicy,
    the_road_out: &TheRoadOut,
    keyring: &WhoseKeyring,
    then: impl FnOnce(Answering, &Answers<'_>) -> T,
) -> Result<T, NotPrepared> {
    // **The rule is asked first, and that ordering is the point.** The source
    // is the provider's own, read off the provider the person's list resolved
    // — never assumed, and never `ThisMachine` because the address happened to
    // look local.
    let permission =
        Answering::chosen(provider.source(), policy).map_err(NotPrepared::ARuleRefused)?;

    // **And then which way out of this machine that road goes**, before the
    // keyring and before anything opens. `alo_proxy::the_way` decides it;
    // nothing here does, and `crate::the_road_out` is the one place that asks.
    let taking = the_road_out.to(provider).map_err(NotPrepared::NoRoadOut)?;

    // Held in this frame because `Hosted::provider` borrows it, and it must
    // outlive the ask. This is the only place in the daemon where a credential
    // exists at all, and it lives no longer than this call.
    let held;
    let key = match provider.key.as_ref() {
        None => None,
        Some(reference) => {
            held = Questions::a_key_from(keyring, reference)
                .map_err(NotPrepared::TheKeyIsNotStored)?;
            Some(&held)
        }
    };

    Ok(then(
        permission,
        &Answers::Provider(Hosted::provider(provider, key).taking(&taking)),
    ))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use alo_models::{Region, SecretRef};

    use super::{NotPrepared, a_provider_ready_to_be_asked};
    use crate::questions::WhoseKeyring;
    use crate::the_road_out::TheRoadOut;
    use alo_models::Provider;
    use alo_models::SourcePolicy;

    /// A provider that names a key, so every test below *could* reach a store.
    ///
    /// The key is the instrument: a refusal that names the keyring is a refusal
    /// that opened one, and that is the fault these tests exist to catch.
    fn one_that_wants_a_key() -> Provider {
        Provider::checked(
            "Mine",
            "https://api.example.com",
            Region::Declared("the EU".to_owned()),
            Some(SecretRef::named("provider/mine")),
        )
        .expect("a provider with an https endpoint and a key")
    }

    /// A provider whose address is not somewhere a road out can be decided
    /// about, which is the one road refusal reachable without a proxy.
    fn one_at_an_address_that_is_not_one() -> Provider {
        Provider {
            name: "Mine".to_owned(),
            endpoint: format!("https://{}", "a".repeat(400)),
            region: Region::Declared("the EU".to_owned()),
            key: Some(SecretRef::named("provider/mine")),
            models: Vec::new(),
        }
    }

    #[test]
    fn a_rule_refuses_before_the_keyring_is_opened() {
        let provider = one_that_wants_a_key();
        let mut asked = 0_u8;
        let refused = a_provider_ready_to_be_asked(
            &provider,
            &SourcePolicy::ThisMachineOnly,
            &TheRoadOut::nobodys(),
            // Nobody's: every lookup is `NotStored::Unavailable`, so a key
            // asked for out of order shows up as the *wrong refusal* rather
            // than as nothing at all.
            &WhoseKeyring::Nobodys,
            |_, _| asked += 1,
        )
        .expect_err("a hosted provider was permitted under this-machine-only");

        assert!(
            matches!(refused, NotPrepared::ARuleRefused(_)),
            "the rule refused, so that is the refusal: {refused:?}"
        );
        assert_eq!(asked, 0, "nothing was asked");
    }

    #[test]
    fn a_rule_refuses_before_a_road_is_asked_for() {
        // The address is not one a road can be decided about *and* the policy
        // refuses the provider. Only the earlier of the two may answer.
        let provider = one_at_an_address_that_is_not_one();
        let mut asked = 0_u8;
        let refused = a_provider_ready_to_be_asked(
            &provider,
            &SourcePolicy::ThisMachineOnly,
            &TheRoadOut::nobodys(),
            &WhoseKeyring::Nobodys,
            |_, _| asked += 1,
        )
        .expect_err("a hosted provider at an impossible address was permitted");

        assert!(
            matches!(refused, NotPrepared::ARuleRefused(_)),
            "the rule is asked before the road, so the rule answers: {refused:?}"
        );
        assert_eq!(asked, 0, "nothing was asked");
    }

    #[test]
    fn no_road_out_refuses_before_the_keyring_is_opened() {
        let provider = one_at_an_address_that_is_not_one();
        let mut asked = 0_u8;
        let refused = a_provider_ready_to_be_asked(
            &provider,
            &SourcePolicy::Anywhere,
            &TheRoadOut::nobodys(),
            &WhoseKeyring::Nobodys,
            |_, _| asked += 1,
        )
        .expect_err("a road was decided for an address that is not one");

        assert!(
            matches!(refused, NotPrepared::NoRoadOut(_)),
            "the road refused, so that is the refusal: {refused:?}"
        );
        assert_eq!(asked, 0, "nothing was asked");
    }

    /// Not a test of the ordering, and deliberately not named as one: it
    /// survives the mutation that asks the key first, because what it measures
    /// is that the store's own four reasons come out unreworded.
    #[test]
    fn the_store_refuses_in_its_own_words() {
        let provider = one_that_wants_a_key();
        let mut asked = 0_u8;
        let refused = a_provider_ready_to_be_asked(
            &provider,
            &SourcePolicy::Anywhere,
            &TheRoadOut::nobodys(),
            &WhoseKeyring::Nobodys,
            |_, _| asked += 1,
        )
        .expect_err("a key came out of nobody's keyring");

        assert!(
            matches!(
                refused,
                NotPrepared::TheKeyIsNotStored(alo_secrets::NotStored::Unavailable)
            ),
            "the store is the only one left to refuse: {refused:?}"
        );
        assert_eq!(asked, 0, "nothing was asked");
    }

    #[test]
    fn a_provider_that_names_no_key_reaches_the_ask_with_none() {
        let provider = Provider::checked(
            "Mine",
            "https://api.example.com",
            Region::Declared("the EU".to_owned()),
            None,
        )
        .expect("a provider that needs no credential");

        let leaves = a_provider_ready_to_be_asked(
            &provider,
            &SourcePolicy::Anywhere,
            &TheRoadOut::nobodys(),
            // Nobody's, and it does not matter: a provider that names no key
            // reaches no store, so the one machine in this product where a
            // credential exists is never entered.
            &WhoseKeyring::Nobodys,
            |_, answers| answers.leaves(),
        )
        .expect("rule, road and key all said yes");

        assert!(leaves, "a hosted provider is an egress");
    }
}
