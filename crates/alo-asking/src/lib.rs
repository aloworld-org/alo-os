//! Putting a question to a model, with what leaves this machine decided and
//! shown first.
//!
//! Thirteen crates in this workspace decide correctly about a question and none
//! of them sends one: `alo-models` knows *where* an answer may come from,
//! `alo-answering` knows what happens when that place cannot answer, and
//! `alo-egress` knows what may leave and shows it while it does. **There was no
//! method anywhere in this repository that put a question to a model.** This is
//! that method, and it is written as the joining-up of three decisions already
//! made rather than as a fourth.
//!
//! ```no_run
//! use alo_answering::Answering;
//! use alo_asking::{Asking, Hosted, Question};
//! use alo_capability::Grantee;
//! use alo_egress::Indicator;
//! use alo_models::{Provider, Region, Secret, SourcePolicy};
//! use std::time::SystemTime;
//!
//! # fn main() {
//! let policy = SourcePolicy::Anywhere;
//! let mistral = Provider::checked(
//!     "Mistral",
//!     "https://api.mistral.ai",
//!     Region::Declared("the EU".to_owned()),
//!     None,
//! )
//! .expect("a provider somebody added");
//! let key = Secret::typed("sk-live-…").expect("a key they pasted");
//! let hosted = Hosted::provider(&mistral, Some(&key));
//!
//! // Where it may go is decided first, against the rule this machine is under.
//! let answering =
//!     Answering::chosen(hosted.named_source(), &policy).expect("the rule permits it");
//! let question =
//!     Question::asked("may the tenant sublet?", "mistral-small-latest").expect("a question");
//!
//! // And only then is it put anywhere. The indicator is shown the egress
//! // before a socket opens, and the departure comes back so what left can be
//! // written down.
//! let mut indicator = Indicator::default();
//! match Asking::by(&Grantee::named("@mail"), answering, &[], &policy)
//!     .to_a_provider(&question, &hosted, &mut indicator, SystemTime::now())
//! {
//!     Ok(asked) => {
//!         // `record.keep(Entry::left(asked.departing()))` goes here.
//!         let answer = asked.ended(&mut indicator);
//!         // And `answer.came_from(&strings)` is shown beside `answer.text()`.
//!         let _ = answer;
//!     }
//!     // Four things, and each one is a different thing to do about it —
//!     // `refusing` has the table. Three of them mean nothing was sent.
//!     Err(not_asked) => eprintln!("{not_asked:?}"),
//! }
//! # }
//! ```
//!
//! # The hosted provider first, and deliberately
//!
//! ADR 0008 makes a hosted API a first-class choice rather than a fallback, so
//! this is the supported path built in the order it can actually be
//! *exercised*: a provider answers over https from any machine, while a model on
//! this one needs a runtime installed and gigabytes downloaded before it can say
//! anything at all. Building the harder-to-run path first is how a method ends
//! up shaped around the case nobody tested.
//!
//! It also puts the sharper half of the promise first. A local model that fails
//! is a bad afternoon; **a hosted one is where the egress line, the region, the
//! refusal in the rule's own words and *never a silent fallback* all have to be
//! true at once**, and none of those can be proven against a runtime on
//! loopback.
//!
//! **A question answered on this machine does not come through here.** It causes
//! no egress, so there is no departure to make and nothing for law 1 to show —
//! `alo_egress::Leaving::asking` refuses to make one, which is the zero-egress
//! claim as a type. Asking the runtime is its own item and its own path, and
//! this door says so rather than growing a branch for it.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`Question`] | What was asked, held the way a key is: it goes in and does not come out |
//! | [`Hosted`] | The only thing that knows what a provider's API looks like |
//! | [`Asking`] | The one door, and the order the four steps happen in |
//! | [`Asked`] | An answer, and the departure it came with |
//! | [`DidNotAnswer`] | A question that left and did not come back, and the departure it left with |
//! | [`NotAsked`] | The four things that can come back instead, and what to do about each |
//! | [`Answer`] | What came back, and — always — where it came from |
//!
//! # Three things that are not here
//!
//! **No decision about where a question goes.** Not one. The place arrives as an
//! `alo_answering::Answering`, which is the only type meaning *this question may
//! be answered here*, and this crate spends it. A second attempt somewhere else
//! needs an offer a person answered, and there is no function here that could
//! make one.
//!
//! **No record.** This crate hands the departure back and writes nothing down —
//! `alo-record` is reachable from none of the crates it observes, and that stays
//! true here. What it hands back is the only thing `alo_record::Entry::left` can
//! be made from, so what left is still written; [`asked`] has the argument.
//!
//! **No text of anybody's kept anywhere.** A question and an answer are made out
//! of somebody's own work, and ADR 0001 §7 keeps neither. Both types here have
//! no `Serialize` and a `Debug` written by hand, which is `alo_models::Secret`'s
//! shape applied to the two things that pass through this crate.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod answer;
pub mod asked;
pub mod asking;
pub mod hosted;
pub mod question;
pub mod refusing;
pub mod unanswered;
pub mod words;

#[cfg(test)]
mod testing;

pub use answer::Answer;
pub use asked::Asked;
pub use asking::Asking;
pub use hosted::Hosted;
pub use question::{NotAQuestion, Question};
pub use refusing::{Miswired, NotAsked};
pub use unanswered::DidNotAnswer;
pub use words::{EVERY_WORD, Word, WordsError, asking_words, declare_into};
