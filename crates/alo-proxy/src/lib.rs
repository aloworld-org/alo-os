//! One proxy, machine-wide, honoured.
//!
//! `docs/features.md` v0.5: **corporate proxy support, machine-wide and
//! honoured by applications. A great many company networks have no other route
//! out.** That last sentence is why this crate is small and why nothing in it
//! is optional: on those networks a machine that does not honour the proxy is a
//! machine that does nothing at all, and a machine that honours it in some
//! places and not others is worse — the person fixes the half they can see.
//!
//! ```
//! use alo_proxy::{Carried, Kept, ProxyAddress, Road, Scheme, SpokenTo, Reaching, TheProxy, Way};
//! # use alo_proxy::{ConfigurationAddress, NotEvaluated, TheEvaluator};
//! # struct Nothing;
//! # impl TheEvaluator for Nothing {
//! #     fn asked(&self, _: &ConfigurationAddress, _: &str, _: &str) -> Result<String, NotEvaluated> {
//! #         Err(NotEvaluated::NothingEvaluatesIt)
//! #     }
//! # }
//! # fn main() {
//! // What somebody was handed on a slip of paper, and set once.
//! let proxy = ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080)
//!     .expect("an address");
//! let kept = Kept::by_this_person(TheProxy::one(proxy.clone()));
//!
//! // Every road out asks the same setting, and is answered the same way.
//! let files = Reaching::over(Scheme::Https, "dl.example.org").expect("a host");
//! for road in Road::EVERY {
//!     let way = alo_proxy::the_way(kept.proxy(), road, &files, &Nothing)
//!         .expect("nothing refuses it");
//!     assert_eq!(way, Way::Through(proxy.clone()));
//! }
//!
//! // The machine itself is never sent through it, whatever is set.
//! let here = Reaching::over(Scheme::Http, "127.0.0.1").expect("a host");
//! assert!(
//!     alo_proxy::the_way(kept.proxy(), Road::AskingAProvider, &here, &Nothing)
//!         .expect("nothing refuses it")
//!         .is_straight()
//! );
//!
//! // And what the road out is given carries the proxy, not a destination.
//! let carried = Carried::of(Way::Through(proxy));
//! assert_eq!(
//!     carried.as_an_address().as_deref(),
//!     Some("http://proxy.example.com:8080"),
//! );
//! # }
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`TheProxy`] | the machine's one setting: none, an address per scheme with exceptions, or an automatic configuration |
//! | [`Kept`] | whose it is — an organisation's on a machine it manages, the person's on their own (ADR 0016) |
//! | [`Road`] | every road out of this machine, as a closed list |
//! | [`the_way`] | which way one road goes, decided once |
//! | [`signed_in`] | that road, signed in to where the proxy asks who you are — **the one door a credential travels through** |
//! | [`TheMachinesPasswords`] | where a machine-wide proxy password is kept, as a unit was given it (ADR 0059) |
//! | [`TheMachinesCredentials`] | the same store, as a person sets one on the machine that is theirs (ADR 0060) |
//! | [`Carried`] | what that road is then given: variables for a program, an address for a client |
//! | [`Published`] | what an application is given, so it honours the same proxy |
//! | [`looked_up`] | the same answer, one address at a time, for the portal applications already ask |
//! | [`TheEvaluator`] | where an automatic configuration is worked out — a **separate program that holds no grant** |
//!
//! # The five things this crate promises, and where each is kept
//!
//! | | |
//! |---|---|
//! | **Every road out takes it** | [`Road`] is closed and [`the_way`] is the one door; `tests/every_road_out_takes_it.rs` walks both |
//! | **A password goes to the keyring, never to a file** | [`ProxyAddress`] holds [`WhereThePasswordIs`] and has no field a password could arrive in (ADR 0022) |
//! | **A proxy that asks who you are is signed in to, on every road** | [`signed_in`] is the one door, and a test in `signing_in.rs` reads the workspace for a second caller of [`Carried::with_the_password`] (ADR 0059) |
//! | **The indicator names the real destination** | a [`Way`] carries the proxy and **no destination**: what an `alo_egress::Destination` is built from is the [`Reaching`] the caller already had |
//! | **A configuration is evaluated where nothing is granted** | `evaluator.rs` is a separate program with a cleared environment, and a machine without one **refuses** rather than going straight out |
//!
//! # What this crate is not
//!
//! **Not a proxy server**, and not a client either. It opens no socket and
//! starts no thread. What it does is decide, once, which way each road out goes
//! — and the crates that take those roads ask it: `alo-software` for installing
//! and updating applications, `alo-updating` and `alo-looking-once` for the
//! system's own update, `alo-models` for a provider and `alo-agentd` for a
//! turn's question. The one thing it reads off a disk is the password a proxy
//! asks for ([`TheMachinesPasswords`]), and a machine whose proxy asks for none
//! never causes a read at all. The one thing it writes is that same password,
//! when a person sets one on their own machine ([`TheMachinesCredentials`]),
//! and it is written by the tool the base already has rather than by anything
//! here (ADR 0060 §4).
//!
//! **Not the machine's description.** Reading an organisation's proxy out of
//! `/etc/alo/agentd.toml` is `alo-agentd`'s, where that file is read;
//! [`Kept`] is the value it hands over.
//!
//! **Not the portal's implementation.** [`looked_up`] is the answer; owning the
//! name on the bus is `alo-portals`, which this crate reads and never edits.
//!
//! # Saying it in a language
//!
//! Nothing here has a `Display` that a person reads, and no refusal is a
//! `String` this crate assembled. [`words`] is everything it can say; the road
//! to it is `said` on each refusal, taking the vocabulary the person in front
//! of the machine actually reads.
//!
//! The order matters as much as it does in `alo-egress`: **the road is decided
//! before it is worded**, so a machine whose translations failed to load
//! refuses exactly what it refused before.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod address;
pub mod automatic;
pub mod carried;
pub mod deciding;
pub mod evaluator;
pub mod exceptions;
pub mod kept;
pub mod password;
pub mod portal;
pub mod provisioned;
pub mod provisioning;
pub mod published;
pub mod reaching;
pub mod refusing;
pub mod road;
pub mod setting;
pub mod signing_in;
pub mod words;

#[cfg(test)]
mod testing;

pub use address::{NotAnAddress, ProxyAddress, SpokenTo};
pub use automatic::{
    ConfigurationAddress, NotAConfiguration, NotEvaluated, TheEvaluator, understood,
};
pub use carried::{Carried, NEVER_THROUGH_A_PROXY};
pub use deciding::the_way;
pub use evaluator::{THE_EVALUATOR, TheRentedEvaluator, arguments, nothing_of_this_machines};
pub use exceptions::{Exceptions, NotAnException};
pub use kept::{Kept, NotChanged, SetBy};
pub use password::{
    NotAName, NotAPassword, Password, THE_PERSONS_PROXY_PASSWORD, WhereThePasswordIs,
};
pub use portal::{NotLookedUp, STRAIGHT_OUT, looked_up};
pub use provisioned::{LONGEST_PASSWORD, TheMachinesPasswords, WHERE_THEY_ARE};
pub use provisioning::{
    LONGEST_HANDED_OVER, NONCE_BYTES, NoRandomness, NotHandedOver, NotProvisioned,
    THE_ENCRYPTED_STORE, THE_TOOL, TheMachinesCredentials, handed_over, what_was_handed_over,
};
pub use published::Published;
pub use reaching::{NotReachable, Reaching, Scheme};
pub use refusing::NotOnTheRoad;
pub use road::{Road, Way};
pub use setting::TheProxy;
pub use signing_in::{NotSignedIn, WhatIsWrong, WhereThePasswordsAre, signed_in};
pub use words::{EVERY_WORD, Word, WordsError, declare_into, proxy_words};
