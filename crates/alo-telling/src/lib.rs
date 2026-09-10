//! A machine that cannot reach a model says so once, where it happened, and
//! continues.
//!
//! `docs/features.md` promises at v0.01: **★ And it never nags. A machine that
//! cannot reach a model does not follow somebody around asking them to buy
//! credit — that is the greyed-out panel ADR 0009 already refused, in a
//! different disguise.**
//! [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md)
//! says what that means in full, under *no nagging*: *a machine that cannot
//! reach a model says so once, where it happened, and continues.*
//!
//! Everything that promise needs existed except one thing. `alo-answering`
//! knows the eight ways a place can fail to answer and words every one of them;
//! `alo-asking` and `alo-turn` both hand back the failure whole. **Nothing
//! remembered that a person had already been told.** So a machine whose
//! provider account had emptied would produce the same four lines at the end of
//! every turn, all afternoon — which is the greyed-out panel ADR 0009 rejected,
//! rebuilt one honest sentence at a time.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`unavailable`] | What makes one unavailability the same as another |
//! | [`who_asked`] | Whether a person is waiting, which is what makes saying it again not a nag |
//! | [`told_once`] | The four lines a person reads, in the order they read them |
//! | [`telling`] | What this machine has already said, for as long as a session lasts |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! ```
//! use alo_answering::{Answering, WentWrong};
//! use alo_models::{InferenceSource, Region, SourcePolicy};
//! use alo_telling::{Tell, Telling, WhoAsked};
//!
//! /// A provider somebody pays for, with nothing left in the account.
//! fn the_account_is_empty() -> alo_answering::Failed {
//!     let provider = InferenceSource::Hosted {
//!         provider: "alo".to_owned(),
//!         region: Region::Declared("the EU".to_owned()),
//!     };
//!     Answering::chosen(provider, &SourcePolicy::Anywhere)
//!         .expect("nothing forbids asking there")
//!         .did_not_answer(WentWrong::RanOut, &[], &SourcePolicy::Anywhere)
//!         .expect("an account can run out at a provider")
//! }
//!
//! let mut telling = Telling::nothing_said_yet();
//!
//! // The turn a person started fails, and they are told — once, where it
//! // happened, in four lines.
//! let tell = telling.about(the_account_is_empty(), WhoAsked::ThePerson);
//! let told = tell.to_say().expect("nothing has said this yet");
//! assert_eq!(told.unavailable().why(), WentWrong::RanOut);
//!
//! // Everything the machine does by itself afterwards says nothing at all.
//! for _ in 0..20 {
//!     assert_eq!(
//!         telling.about(the_account_is_empty(), WhoAsked::TheMachine),
//!         Tell::SaidAlready,
//!     );
//! }
//! ```
//!
//! # The rule, in one sentence and then in three
//!
//! **A telling happens when this machine has something to say that it has not
//! said, or when somebody just asked.**
//!
//! - the **source** changing is a different telling, because *nothing answered
//!   on this machine* and *nothing was answered by alo, in the EU* are facts
//!   about two different halves of somebody's arrangements;
//! - the **reason** changing is a different telling, because a machine that
//!   swallowed the second failure would hide the one that mattered — and that
//!   is read strictly, down to the status a service answered with, since the
//!   number is inside the sentence;
//! - the **person asking again** is always a telling, because an answer to a
//!   question somebody just asked is not a reminder, and a key that silently
//!   does nothing is the worst outcome available.
//!
//! # Four things this crate is deliberately not
//!
//! **It never asks anybody to buy anything.** Not a price, not a link, not a
//! provider, not a *top up to continue*. [`words`] has two strings and a test
//! that fails if either of them starts selling something. The line about an
//! account being empty is `alo-answering`'s and says what is so; ADR 0009's
//! *why the money case matters most* is why that difference is not a nicety —
//! the person who cannot pay must not become a second-class user of a computer
//! they own, and the first step towards that is a machine that keeps mentioning
//! it.
//!
//! **It never chooses another source.** ADR 0008's *never a silent fallback*
//! runs in both directions, and *we spent your money elsewhere because the
//! first place was empty* is the worst available version of it. The offers a
//! failure carried travel with the telling, unranked and unchosen, and
//! [`ToldOnce::take`] is `alo_answering::Failed::take` — a person's act,
//! unchanged.
//!
//! **It shows nothing and draws nothing.** No surface, no notification, no
//! queue, no timer. A [`Tell`] is answered to whoever reported the failure and
//! there is no way to send one anywhere else — which is how *where it happened*
//! is kept: a telling exists at the place the turn failed, because there is
//! nowhere else it could be built. What [`Telling`] remembers is an identity
//! with no sentence in it, so nothing this crate holds could be put on a screen
//! at a later moment.
//!
//! **It writes nothing down.** `alo-answering` settled that a question which
//! failed is not something an agent did and not something that left, so an
//! entry per failure would build a log of somebody's questions failing, one
//! honest entry at a time. A crate whose whole subject is *how often has this
//! happened* is the last one that should start keeping the count on a disk.
//!
//! # What it does not cover, said plainly
//!
//! ADR 0009 lists six ways an agent becomes unavailable. This crate is about
//! the ones that arrive as an `alo_answering::Failed` — a question was put
//! somewhere and that place did not answer — which is the money running out,
//! the model not being there, the machine being offline, the provider being
//! down and the key having expired.
//!
//! The sixth is **declining the agent at setup**, and it is not here on
//! purpose: no turn happens on such a machine, so there is nothing to suppress
//! and nothing to say. ADR 0009 answers it in the stronger way — the agent's
//! surfaces are absent rather than present-but-disabled.
//!
//! A **policy refusing a source** is likewise not a telling. The machine can
//! reach a model; a rule says it may not, `alo_egress::NotPermitted` names that
//! rule in its own words, and suppressing it would be this crate quietly
//! hiding a decision an administrator made (ADR 0016).

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod telling;
pub mod told_once;
pub mod unavailable;
pub mod who_asked;
pub mod words;

#[cfg(test)]
mod testing;

pub use telling::{HOW_MANY_IT_REMEMBERS, Tell, Telling};
pub use told_once::{HOW_MANY_LINES, ToldOnce};
pub use unavailable::Unavailable;
pub use who_asked::WhoAsked;
pub use words::{EVERY_WORD, Word, WordsError, declare_into, telling_words};
