//! What a client may ask `alo-agentd`, what it may not, and what it is told.
//!
//! This is the daemon's door, from both sides of it. Bytes arrive on a socket
//! and this crate says whether they are a request and which of two callers may
//! have made it; what happened goes back through it and this crate says what
//! shape that takes. Nothing here executes anything, decides anything about a
//! grant, or reads a clock.
//!
//! ```
//! use alo_protocol::{FromAnAgent, FromAPerson, NotUnderstood, ToAPerson, ToAnAgent, protocol_words};
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
//!
//! // What goes back is worded by the daemon and crosses saying whether
//! // anybody translated it — so a shell can tell a person's own language
//! // from the English nobody has got to yet.
//! let told = ToAnAgent::refused(&refused.said(&strings));
//! let back = ToAnAgent::read(&told.written().expect("an answer writes"))
//!     .expect("an answer for an agent");
//! assert!(!back.refusal().expect("a refusal").is_translated());
//!
//! // And the person's own list has no shape on the agent's door at all.
//! let waiting = ToAPerson::Declined.written().expect("an answer writes");
//! assert_eq!(
//!     ToAnAgent::read(&waiting),
//!     Err(NotUnderstood::NotAnAnswerForAnAgent),
//! );
//! # }
//! ```
//!
//! # The two doors, and why there are two
//!
//! [`FromAnAgent`] is what an agent asks during a turn: a read, a change to put
//! to the person, and a question for a model. [`FromAPerson`] is what a person's
//! shell sends: yes or no to one change by its number, and what is waiting.
//!
//! They are two types because they arrive from two sides of one machine, and a
//! door that took both would be a door where the side that proposed a change
//! could approve it. ADR 0001 §5 would then be true of the capability model and
//! false of the socket in front of it. [`asked`] is where the whole of that
//! argument lives, together with what it does **not** claim: which side a caller
//! is really on is peer credentials on a Unix socket, and that is `alo-agentd`'s.
//!
//! **The answers divide the same way**, and [`told`] is where that is argued:
//! [`ToAnAgent`] has no shape for what the person is being asked, so a daemon
//! cannot put one side's answer on the other side's connection even by mistake.
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
//! **Here:** the envelope ([`FORMAT`], [`LONGEST`], [`LONGEST_ANSWER`], one
//! message per line), the four doors, the [`Argument`]s a request carries, what
//! the machine [`Done`] and the [`Wording`] every sentence crosses in — and
//! [`NotUnderstood`], the nine ways a message is not one this side can act on,
//! each said in the language the reader has.
//!
//! **What goes back was its own decision, and the decision was that
//! `alo_files::Answer` does not gain a `Serialize`.** A path is not always text,
//! so a derived one would fail on somebody's filename rather than on nobody's;
//! [`naming`] has the whole argument and the rule that replaced it, which is
//! `alo-files`' own — a name that cannot be shown is **counted**, never dropped
//! silently and never made into an error.
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
mod done;
mod frame;
mod naming;
mod person;
mod refusing;
mod standing;
#[cfg(test)]
mod testing;
mod thing;
mod to_a_person;
mod to_an_agent;
mod told;
mod wording;
pub mod words;

pub use agent::FromAnAgent;
pub use argument::Argument;
pub use done::Done;
pub use frame::{FORMAT, LONGEST, LONGEST_ANSWER};
pub use person::FromAPerson;
pub use refusing::NotUnderstood;
pub use standing::Standing;
pub use thing::{Kind, Thing};
pub use to_a_person::ToAPerson;
pub use to_an_agent::ToAnAgent;
pub use wording::{CameFrom, Wording};
pub use words::{EVERY_WORD, Word, WordsError, declare_into, protocol_words};
