//! What happens when the place a question was to be answered cannot answer it.
//!
//! `docs/features.md` promises at v0.01: **★ Never a silent fallback. A local
//! model that fails does not quietly become an API call — failing to answer is
//! recoverable, a person's records leaving the building because a download was
//! corrupt is not.**
//! [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) rejects
//! falling back outright, and names it *the single most tempting convenience
//! here*. This crate is that refusal as working code.
//!
//! It is the piece that was missing between two that already existed.
//! `alo_models::InferenceSource` says where an answer *would* come from and
//! `alo-egress` decides about a departure that is already happening; neither
//! had anything to say about the moment in between, when the place a person
//! chose does not answer and something has to decide what happens next.
//!
//! # The decision this crate turns on: it is a change, not a setting
//!
//! *Asking somewhere else* could have been a thing a person switches on in
//! advance — *when the local model fails, use my provider* — or a thing they
//! are asked at the moment. It is the second, and ADR 0008 is why.
//!
//! A setting turned on in advance **is** the fallback that ADR was written to
//! reject; it is the same behaviour with a checkbox in front of it. The
//! objection was never that alo OS decides badly, it is that the person is not
//! there: their records leave the building at the moment of a failure they
//! never saw, which is *nothing leaves silently* failing in the one case it
//! exists for. A checkbox somebody ticked in March does not make them present
//! in June.
//!
//! So the shape is ADR 0001 §5's — **one sentence, one approval, one attempt,
//! and an approval is never a session** — brought to a place §5 does not itself
//! reach. §5 binds an agent changing this machine; a person's question is not
//! that. What is borrowed is the shape, and [`Offer`] says in its own
//! documentation why it is deliberately not an `alo_capability::Proposal`.
//!
//! # The journey
//!
//! | | |
//! |---|---|
//! | [`Answering`] | The only type meaning *this question may be answered here*, and the two doors into it |
//! | [`WentWrong`] | What can go wrong where a question was put, and what cannot go wrong there |
//! | [`Failed`] | It did not answer: what a person reads, and the one door out |
//! | [`Elsewhere`] | Where else this machine may ask, and the doors a rule has closed |
//! | [`Offer`] | One place that could be asked instead, and the sentence a person approves |
//! | [`NotOffered`] | An offer that was not this failure's, carrying the failure back |
//!
//! A question travels [`Answering`] → [`Failed`] → [`Offer`] → [`Answering`],
//! and each step can only be reached from the one before. **There is no path
//! from a failure to a second attempt that does not go through an offer a
//! person answered**, and that absence is the guarantee: a machine that fell
//! back would need no new type at all, only a second call in the same function.
//!
//! # Three things that are not here, and each one is the guarantee
//!
//! **No serde.** Nothing in this crate can be read back off a disk, so an
//! approval to ask somewhere else cannot arrive from a file — it is given by a
//! person at the moment, or it does not exist. `alo-context` refuses to
//! deserialise for the same reason one step earlier.
//!
//! **Nothing that can reach anywhere.** No HTTP client, no socket, no runtime,
//! not even behind a feature. The crate that decides where a failed question
//! may go next cannot itself go there — which is what makes *nothing here ever
//! asks a second place* a claim somebody can check by reading `Cargo.toml`
//! rather than by reading every function. It is `alo-keeping`'s argument about
//! `alo-record`: a promise about the absence of code is only worth what the
//! code around it is small enough to prove.
//!
//! **The question.** Not in any type here. What is decided is *where*, and a
//! crate that held *what* would be one honest entry away from being the place
//! somebody's questions accumulate. Nothing here holds text anybody outside
//! this repository wrote either — not a model's name, not what a provider said
//! about itself — so every sentence it produces is one alo OS wrote rather than
//! one it passed on.
//!
//! # What this crate does not do
//!
//! **It asks nothing.** There is no method here that puts a question to a
//! model, because there is nothing in this repository that puts a question to a
//! model: `alo_models::ModelRuntime` loads, unloads, fetches and lists, and the
//! asking arrives with the daemon. What is here is the decision that has to be
//! settled before then, so that when the asking is written it is an
//! implementation of a settled model rather than the place the model gets
//! decided by accident — which is how a fallback gets written, every time.
//!
//! **It shows nothing and writes nothing down.** The indicator is
//! `alo-egress`'s and the record is `alo-record`'s: a taken offer becomes a
//! departure like any other, on the indicator while it happens and in the
//! record afterwards, and
//! `tests/from_a_question_that_failed_to_what_left.rs` walks that whole journey.
//! A failure that nobody answered leaves neither, because nothing happened.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod answering;
pub mod elsewhere;
pub mod failed;
pub mod offer;
pub mod refusing;
pub mod words;
pub mod wrong;

#[cfg(test)]
mod testing;

pub use answering::Answering;
pub use elsewhere::Elsewhere;
pub use failed::Failed;
pub use offer::Offer;
pub use refusing::NotOffered;
pub use words::{EVERY_WORD, Word, WordsError, answering_words, declare_into};
pub use wrong::{NotWhatFailed, WentWrong};
