//! One approval, and the sentence a person approves.
//!
//! ADR 0001 §5: *a read runs inside the turn; any change to the machine is
//! proposed with a sentence describing it and waits for one approval. What a
//! person approves is that sentence, and an approval is never a session.*
//! `alo-turn` and `alo-capability` have carried proposals and approvals since
//! the beginning — checked, worded from the validated arguments, spent when
//! they are answered, recorded when they run — and until this crate **no
//! surface had ever shown one**. `ROADMAP.md`'s v0.01 exit gate has *approve
//! the sentence, see it happen* in the middle of it, and the sentence had
//! nowhere to appear.
//!
//! This crate is that surface's model, and deliberately nothing more:
//!
//! - [`Asked`] — the question as a person reads it: the proposal's own
//!   sentence, whose change it is, and how long is left, made from a change
//!   that is really waiting and from nothing else;
//! - [`Compositor`] — the port whatever owns the screen implements to be handed
//!   one, with no rendering in it;
//! - [`Approving`] — the change in front of somebody and the one answer it
//!   gets: approve, and it happens once; say no, and it does not happen at all;
//! - [`NotAsked`] and [`NotAnswered`] — every way it can go nowhere, each with
//!   a sentence, because a change that could not be put to anybody and said
//!   nothing about it is an agent that appears to have ignored an instruction.
//!
//! ```
//! use alo_approving::{Answered, Approving, Asked, Asks, Compositor, NotAsked, SurfaceRefused};
//! use alo_capability::{Given, Grant, Grants, Reach};
//! use alo_context::Context;
//! use alo_egress::Indicator;
//! use alo_files::{OnThisMachine, Reaching};
//! use alo_record::Record;
//! use alo_saying::everything_this_machine_can_say;
//! use alo_strings::Strings;
//! use alo_turn::{Bounding, Doing, Done, Machine, NoBoundary, Turning};
//! use std::time::{Duration, SystemTime};
//!
//! /// A screen, keeping whatever question it was last given.
//! #[derive(Default)]
//! struct Screen {
//!     showing: Option<Asked>,
//! }
//! impl Compositor for Screen {
//!     fn ask(&mut self, asked: Asked) -> Result<(), SurfaceRefused> {
//!         self.showing = Some(asked);
//!         Ok(())
//!     }
//! }
//!
//! # struct NothingIsBounded;
//! # impl Bounding for NothingIsBounded {
//! #     fn carrying_out(&mut self, _r: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
//! #         Ok(doing.done())
//! #     }
//! #     fn carrying_out_a_departure(
//! #         &mut self,
//! #         _to: &[std::net::SocketAddr],
//! #         doing: &mut dyn FnMut(),
//! #     ) -> Result<(), NoBoundary> {
//! #         doing();
//! #         Ok(())
//! #     }
//! # }
//! let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
//! let hour = Duration::from_secs(60 * 60);
//! let strings =
//!     Strings::of(everything_this_machine_can_say().expect("everything alo OS can say"));
//!
//! // A machine with a grant over one folder, and a turn under way on it.
//! let mut grants = Grants::default();
//! let over_the_invoices = Grant::checked(
//!     "@files",
//!     Reach::Folder("/home/anna/Invoices".into()),
//!     now,
//!     hour,
//! )
//! .expect("a folder a person picked");
//! grants.grant(over_the_invoices);
//! let mut record = Record::default();
//! let mut indicator = Indicator::default();
//! let mut bounding = NothingIsBounded;
//! let mut machine = Machine::carrying_out_file_verbs(
//!     &strings,
//!     &OnThisMachine,
//!     &mut bounding,
//!     &mut indicator,
//!     &mut record,
//! )
//! .expect("the verbs this machine offers");
//! let mut turning =
//!     Turning::beginning(Context::at_invocation(now), "@files", hour, &mut grants, &mut machine)
//!         .expect("a turn on a machine with an agent");
//!
//! // The agent proposes a change, and it is put to the person as one sentence.
//! let id = turning.proposing(
//!     "rename_file",
//!     &[
//!         ("file", Given::text("/home/anna/Invoices/march.pdf")),
//!         ("name", Given::text("march-final.pdf")),
//!     ],
//!     &grants,
//!     hour,
//!     now,
//! )
//! .expect("a change the grants permit");
//!
//! let mut screen = Screen::default();
//! let mut approving = Approving::nothing_to_answer();
//! let Asks::Asked(asked) = approving.ask(Some(&mut screen), &turning, id, now) else {
//!     unreachable!("a change that was waiting was not put to anybody")
//! };
//! assert_eq!(
//!     asked.sentence().text(),
//!     "rename /home/anna/Invoices/march.pdf to march-final.pdf",
//! );
//!
//! // They say no. Nothing happens, nobody is asked why, and the record says so.
//! assert_eq!(approving.decline(&mut turning, now), Answered::Declined);
//!
//! // One answer is one answer: there is nothing left in front of them.
//! assert!(!approving.is_asking());
//! assert!(matches!(approving.approve(&mut turning, &grants, now), Answered::Refused(_)));
//!
//! // And with no compositor, the answer is a sentence rather than silence.
//! assert_eq!(
//!     approving.ask(None, &turning, id, now),
//!     Asks::Refused(NotAsked::NoCompositor),
//! );
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`asked`] | The question as a person reads it, and where it can come from |
//! | [`surface`] | The compositor's half: what it is given, and how it refuses |
//! | [`approving`] | The change in front of somebody, and the one answer it gets |
//! | [`refusing`] | Every way it goes nowhere, and the sentence for each |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # There is one door, and it is a change the machine checked
//!
//! The plan's acceptance for this task is a sentence about provenance — *a
//! proposed change is shown as the sentence `alo-turn` renders* — so provenance
//! is what the types here are shaped around rather than what their
//! documentation promises.
//!
//! [`Asked::of`] takes an `alo_capability::Waiting`, and [`Approving::ask`]
//! reads one off the turn and makes the [`Asked`] itself. Neither type has a
//! public field, a `From`, a deserialiser or a constructor from text, so a
//! compositor can only ever be handed a sentence that came out of a
//! `Proposal::checked` — a change the grants already permit, of a verb on the
//! closed list, filled from arguments that survived validation. A description a
//! model wrote has nowhere to enter, and the compile-fail examples on [`Asked`]
//! are that argument as tests: a second door added later stops being a design
//! discussion and starts being a failing build.
//!
//! # One approval is one execution, and it is held in two places
//!
//! [`Approving`] takes the question off the screen **before** the turn is
//! touched, so a second answer never reaches `alo_turn::Turning::approving` at
//! all. Underneath, `alo_capability::Approvals::approve` takes the proposal off
//! its list in the act of answering it, so an answer that arrives another way —
//! a second shell, a stale panel, a keyboard and a mouse at once — finds
//! nothing either. The surface's rule is a convenience for the person; the
//! list's rule is the guarantee, and this crate does not rely on its own.
//!
//! # Three things this crate is deliberately not
//!
//! **It is not the approval.** Nothing here decides whether a change may run,
//! and nothing here runs one. `alo-capability` checks the proposal and spends
//! the approval, `alo-turn` asks the grants again at the moment of execution and
//! writes the record, and every answer given here goes through that one road. A
//! surface that could execute what it drew would be a second executor with no
//! evidence behind it.
//!
//! **It does not draw.** No size, no place, no colour, no buttons, no modality,
//! no animation. What a person sees is the compositor's, exactly as
//! `alo_indicator::Compositor` leaves the light's appearance to whatever owns
//! the screen. What is fixed here is what a shell must not be free to decide
//! differently: what the question says, whose it is, when it may not be asked,
//! and what happens when it is answered.
//!
//! **It is not reachable by an agent.** There is no verb here and there could
//! not be one. A surface an agent could drive or answer would be an agent
//! approving its own changes, which is ADR 0001 §5 defeated in one call.
//! `alo-protocol` has nothing that reaches this, and `alo-shortcuts` says the
//! same of the key that opens the overlay.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence a person can be shown is declared in [`words`] and answered
//! through a `said` in the language they read: the two answers, and the three
//! refusals nobody else knows about. Everything else — the change itself, and
//! every way the capability model can refuse one — arrives already worded by
//! whoever decided it, which is what keeps one moment from having two accounts.
//!
//! There is no diagnostic English anywhere in this crate: there is no way for a
//! compositor and this model to disagree about whether a surface exists, because
//! nothing here is told that one went away. It finds out by being refused the
//! next time it asks, which is a fact rather than a report.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod approving;
pub mod asked;
pub mod refusing;
pub mod surface;
pub mod words;

#[cfg(test)]
mod testing;

pub use approving::{Answered, Approving, Asks};
pub use asked::Asked;
pub use refusing::{NotAnswered, NotAsked};
pub use surface::{Compositor, SurfaceRefused};
pub use words::{EVERY_WORD, Word, WordsError, approving_words, declare_into};
