//! What a client may ask `alo-agentd`, and what it may not.
//!
//! This is the untrusted side of the daemon's door: bytes arrive on a socket,
//! and this crate says whether they are a request and which of two sides may
//! have made it. Nothing here executes anything, decides anything about a
//! grant, or reads a clock.
//!
//! ```
//! use alo_protocol::{FromAnAgent, FromAPerson, NotUnderstood, protocol_words};
//! use alo_strings::Strings;
//!
//! # fn main() {
//! let strings = Strings::of(protocol_words().expect("this crate's own words"));
//!
//! // What an agent asks for is a verb's name and what was given for each
//! // argument. Nothing has been looked up and nothing has been validated —
//! // that is `alo_capability::Verbs::call`, inside the turn.
//! let asked = FromAnAgent::read(
//!     r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/invoices"}]}}}"#,
//! )
//! .expect("a request an agent may make");
//! assert_eq!(asked.verb(), Some("list_folder"));
//! assert_eq!(asked.given().len(), 1);
//!
//! // The same message on the person's side is not an answer a person gives.
//! assert_eq!(
//!     FromAPerson::read(&asked.written().expect("a request writes")),
//!     Err(NotUnderstood::NotForAPerson),
//! );
//!
//! // And the answer to a change is the person's alone. An agent reaching for
//! // it is refused, in the language the person reads.
//! let refused = FromAnAgent::read(r#"{"format":1,"asks":{"approve":{"number":7}}}"#)
//!     .expect_err("an agent cannot approve anything");
//! assert_eq!(
//!     refused.said(&strings).text(),
//!     "an agent cannot answer a question that was put to a person",
//! );
//! # }
//! ```
//!
//! # The two doors, and why there are two
//!
//! [`FromAnAgent`] is what an agent asks during a turn: a read, a change to put
//! to the person, and a question for a model. [`FromAPerson`] is what a person
//! answers with: yes or no, to one change, by its number.
//!
//! They are two types because they arrive from two sides of one machine, and a
//! door that took both would be a door where the side that proposed a change
//! could approve it. ADR 0001 §5 would then be true of the capability model and
//! false of the socket in front of it. [`asked`] is where the whole of that
//! argument lives, together with what it does **not** claim: which side a caller
//! is really on is peer credentials on a Unix socket, and that is `alo-agentd`'s.
//!
//! # Law 2, where a caller can actually reach
//!
//! There is no request that carries a command, a path to an executable, a
//! script or anything that could be shaped into one — not because a check
//! refuses them, but because there is no field for one to arrive in. A request
//! names a **verb**, which is looked up against the closed list this machine
//! offers, and gives [`Argument`]s, which are text or a whole number.
//!
//! This crate never makes an `alo_capability::Call`. It hands back the name and
//! the values; the turn puts them through the registry. The type that means
//! *validated* is still made in exactly one place, by the crate that owns the
//! list.
//!
//! # What is not on the wire at all
//!
//! **No moment.** Every door in `alo-turn` takes `now` from the machine, so a
//! request that named one could revive a grant that expired an hour ago.
//!
//! **No context.** What the invocation offered is answered by the compositor at
//! the moment the person pressed the key (ADR 0001 §4). A request carrying a
//! document would be an agent handing itself the grant it wanted.
//!
//! **No place for a question to be answered.** ADR 0008 puts that with the
//! person, and it arrives at the turn's door the way the grants do.
//!
//! **No turn.** Which turn a message belongs to is answered by the connection
//! it arrived on. A number for it would be a number an agent could change.
//!
//! # What is here, and what is not
//!
//! **Here:** the envelope ([`FORMAT`], [`LONGEST`], one message per line), the
//! two doors, the [`Argument`]s a request carries, and [`NotUnderstood`] — the
//! seven ways a message is not a request, each said in the language the reader
//! has.
//!
//! **Not here: what goes back.** A read answers with what the machine found, a
//! proposal with a number and a sentence, a question with an answer, and every
//! one of those is made by the daemon out of types that already exist. It is
//! its own decision — an `alo_files::Answer` has a path in it, and a path is not
//! always text — and its own item.
//!
//! **Not here: the process, the socket and who is on the other end.** A
//! long-lived service, the transport, and peer credentials are `alo-agentd`'s,
//! and they need a Linux host to compile as well as to run.
//!
//! `docs/contracts/daemon-protocol.md` is the public surface: what a message
//! looks like, when [`FORMAT`] rises, and what may be added without raising it.

mod agent;
mod argument;
mod asked;
mod frame;
mod person;
mod refusing;
#[cfg(test)]
mod testing;
pub mod words;

pub use agent::FromAnAgent;
pub use argument::Argument;
pub use frame::{FORMAT, LONGEST};
pub use person::FromAPerson;
pub use refusing::NotUnderstood;
pub use words::{EVERY_WORD, Word, WordsError, declare_into, protocol_words};
