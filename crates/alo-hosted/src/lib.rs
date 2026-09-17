//! **alo's own service, as one more provider entry.**
//!
//! [ADR 0014](../../../docs/decisions/0014-alos-own-model-is-a-provider-like-any-other.md):
//! alo hosts models in the EU and sells access by subscription, and this
//! operating system treats that service as **exactly one more provider** — same
//! indicator, same provenance line, same policy, same refusals, same behaviour
//! when the money runs out, **no default, no pre-selection and no special case
//! anywhere in the code.**
//!
//! # This crate is data
//!
//! An address, where the service says it runs, and the chain it discloses. No
//! type of its own, no client, no branch: the provider types every other
//! provider uses, filled in. **If alo's service needed a type, that would be
//! the special case ADR 0014 forbids**, and the day somebody adds one is the
//! day the promise stops being structural and becomes a policy nobody enforces.
//!
//! # Why the rules are tests rather than intentions
//!
//! ADR 0014 says the quiet part out loud: **if alo sells inference, alo profits
//! when questions leave the machine.** The pressures it lists — make ours the
//! default, make it look safer, let the indicator treat it gently, let a failing
//! local model become a paid call — are ordinary commercial pressures, and they
//! will arrive with somebody who means well.
//!
//! So each is a test in `tests/`, not a paragraph here:
//!
//! - the address is not privileged in `alo-egress`;
//! - a policy refusing hosted inference refuses ours, in the same words;
//! - **no identifier, constant or branch outside this crate names it** — read
//!   out of the shipped source of every crate;
//! - running out of credit is reported once and falls back nowhere (ADR 0008);
//! - cancelling leaves a machine that works.
//!
//! # What is not here
//!
//! The service is not built in this repository and is not claimed to run. The
//! account and the billing live outside it. **What the machine knows is an
//! address, a key in the keyring, a region and whether the last request was
//! accepted** — nothing about an account, a balance or a person's plan.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod entry;

pub use entry::{ITS_ADDRESS, ITS_NAME, THE_CHAIN_IT_DISCLOSES, WHERE_IT_SAYS_IT_RUNS, entry};
