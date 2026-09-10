//! Afterwards, ask what it did.
//!
//! `ROADMAP.md`'s v0.01 exit gate ends *ask what it did and get an answer from
//! the record*. `alo-record` has held every execution and every refusal since
//! the beginning — [ADR 0001](../../../docs/decisions/0001-the-capability-model.md)
//! §7 as working code — and `alo-keeping` has written it to a disk and read it
//! back. Until this crate **nothing read it back to a person**.
//!
//! This crate is that surface's model, and deliberately nothing more:
//!
//! - [`Recounting`] — the record on this machine, found where the machine says
//!   it keeps one, asked a question and answered from the file every time;
//! - [`Account`] — what the record answers: the lines, and what the record says
//!   about its own completeness;
//! - [`AtMost`] — how much of it is answered at once, which is not optional;
//! - [`Told`] and [`Outcome`] — one entry as a person reads it, made from an
//!   entry that was written down and from nothing else;
//! - [`Compositor`] — the port whatever owns the screen implements to be handed
//!   one, with no rendering in it;
//! - [`NotRecounted`] — every way the question goes unanswered, each with a
//!   sentence, because somebody who asks what their machine did and is shown
//!   nothing has been told something about their machine.
//!
//! ```
//! use alo_capability::Grantee;
//! use alo_keeping::Writing;
//! use alo_record::{Asking, Entry, Only};
//! use alo_recounting::{Account, AtMost, Compositor, NotRecounted, Outcome, Recounting, Recounts,
//!     SurfaceRefused};
//! use alo_saying::everything_this_machine_can_say;
//! use alo_strings::Strings;
//! use std::time::{Duration, SystemTime};
//!
//! /// A screen, keeping whatever account it was last given.
//! #[derive(Default)]
//! struct Screen {
//!     showing: Option<Account>,
//! }
//! impl Compositor for Screen {
//!     fn show(&mut self, account: Account) -> Result<(), SurfaceRefused> {
//!         self.showing = Some(account);
//!         Ok(())
//!     }
//! }
//!
//! let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
//! let strings =
//!     Strings::of(everything_this_machine_can_say().expect("everything alo OS can say"));
//!
//! // The daemon keeps the record on a disk. Nothing in this crate writes to it.
//! let folder = std::env::temp_dir().join(format!("alo-recounting-page-{}", std::process::id()));
//! std::fs::create_dir_all(&folder).expect("somewhere to keep a record");
//! let kept_at = folder.join("record.jsonl");
//! let _ = std::fs::remove_file(&kept_at);
//! let mut writing = Writing::opening(&kept_at).expect("a record to write to");
//! writing
//!     .keep(&Entry::turned_away(
//!         "tidy_everything",
//!         "there is no verb called tidy_everything",
//!         &Grantee::named("@files"),
//!         now,
//!     ))
//!     .expect("something the machine turned away");
//! drop(writing);
//! # #[cfg(unix)]
//! # {
//! #     use std::os::unix::fs::PermissionsExt;
//! #     let ours = std::fs::Permissions::from_mode(0o600);
//! #     std::fs::set_permissions(&kept_at, ours).expect("a record of our own");
//! # }
//!
//! // Afterwards, somebody asks what it did.
//! let recounting = Recounting::kept_at(&kept_at);
//! let mut screen = Screen::default();
//! let asked = recounting.show(Some(&mut screen), &Asking::anything(), AtMost::ONE_SITTING);
//! let Recounts::Shown(account) = asked else {
//!     unreachable!("a record that was there was not put in front of anybody")
//! };
//!
//! // What was refused reads back as refused, in the machine's words — and what
//! // the model asked for is quoted, never worded as though alo OS said it.
//! let told = account.told().first().expect("one thing happened");
//! assert_eq!(told.outcome(), Outcome::NeverBecameACall);
//! assert_eq!(told.sentence(), None);
//! assert!(told.asked_for().is_some_and(|asked| asked.is("tidy_everything")));
//! assert!(!told.outcome().said(&strings).is_a_bug());
//!
//! // And the record says whether it is all of what happened, beside the answer.
//! assert!(account.goes_all_the_way_back());
//!
//! // With no compositor, the answer is a sentence rather than silence.
//! assert_eq!(
//!     recounting.show(None, &Asking::anything(), AtMost::ONE_SITTING),
//!     Recounts::Refused(NotRecounted::NoCompositor),
//! );
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`recounting`] | The record on this machine, asked — and read off the disk every time |
//! | [`where_it_is`] | Where the machine says it keeps one, so a person can ask at all |
//! | [`bounding`] | How much of a record one answer holds |
//! | [`account`] | What it answers, and what it says about itself |
//! | [`told`] | One entry as a person reads it, and what became of it |
//! | [`surface`] | The compositor's half: what it is given, and how it refuses |
//! | [`refusing`] | Every way the question goes unanswered, and the sentence for each |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # The four promises, and each is an absence rather than a rule
//!
//! **The answer comes off the disk.** [`Recounting`] holds a path and nothing
//! else: no record, no entries, no answer from last time, and no constructor
//! that takes any of those. There is no shape in which it could answer from
//! memory of the session, which is what the plan's first acceptance asks —
//! because the two things that would differ from the file are exactly the two
//! that matter, an entry the daemon failed to write and a record somebody has
//! since shortened.
//!
//! **The disk it comes off is one this machine will believe, and there is no
//! more of it than a person can read.** [`Recounting::about`] reads through
//! `alo_keeping::Reading::believed_at`, which asks who may have written the
//! file before it reads a word of it, and it takes an [`AtMost`] that is not
//! optional. Neither is a rule laid over the surface: there is no second door
//! that reads an unbelieved file, and none that answers with a year.
//!
//! **A refusal reads back as a refusal.** All ten things that can happen have a
//! clause of their own, derived from `alo_record::Happened` by an exhaustive
//! match, and the three ways of being stopped stay three: *nobody was asked*,
//! *the person said no* and *the grants said no at the last moment* are
//! different facts about a machine. A kind of entry added to the record and not
//! given a clause here is a build that fails, rather than a line somebody reads
//! as blank.
//!
//! **Nothing in the answer is a sentence a model wrote.** [`Told::of`] takes an
//! `alo_record::Entry` and there is no other door — no constructor from text,
//! no public field, no `From`, no deserialiser, and compile-fail examples on
//! [`Told`] that turn adding one into a failing build. What it carries is the
//! sentence the machine generated from validated arguments, the refusal's own
//! words as the person was shown them, and this crate's own clause. The one
//! piece of text a model ever wrote — a verb name that never became a call — is
//! [`Told::asked_for`] and never [`Told::sentence`], because there is no
//! sentence: nothing was validated to generate one from. Where that text is
//! quoted a second time — inside `alo-capability`'s *there is no verb called
//! `…`* — the sentence around it is the machine's, written before anything was
//! asked of it, and what is quoted has been through `alo_record::Line` and can
//! rewrite nothing.
//!
//! # Three things this crate is deliberately not
//!
//! **It is not the record.** Nothing here writes an entry, removes one, or
//! could. `alo-record` keeps what happened, `alo-keeping` is the only thing
//! that can shorten it, and this crate opens the file to read. A surface that
//! could edit what it displayed would be evidence with a keyboard attached to
//! it.
//!
//! **It does not draw.** No size, no place, no colour, no columns, no date
//! format. What a person sees is the compositor's, exactly as
//! `alo_indicator::Compositor` leaves the light's appearance to whatever owns
//! the screen. What is fixed here is what a shell must not be free to decide
//! differently: what each line says, what the record says about itself, and
//! what is shown when there is nothing to show.
//!
//! **It is not reachable by an agent.** There is no verb here and there could
//! not be one. An agent able to read the record is an agent able to learn what
//! it has already been refused and shape the next attempt around it, and ADR
//! 0001 §4's *context is offered, never watched* is why nothing here is
//! reachable from a turn. `alo-protocol` has nothing that reaches this.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence a person can be shown is declared in [`words`] and answered
//! through a `said` in the language they read: ten clauses, one remark, two
//! refusals. Everything else — the sentence describing a change, why it was
//! refused, where something went, whether the record goes all the way back —
//! arrives already worded by whoever decided it, which is what keeps one moment
//! from having two accounts.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod account;
pub mod bounding;
pub mod recounting;
pub mod refusing;
pub mod surface;
pub mod told;
pub mod where_it_is;
pub mod words;

#[cfg(test)]
mod testing;

pub use account::Account;
pub use bounding::AtMost;
pub use recounting::{Recounting, Recounts};
pub use refusing::NotRecounted;
pub use surface::{Compositor, SurfaceRefused};
pub use told::{Outcome, Told};
pub use where_it_is::{NotSaid, THE_DESCRIPTION, where_the_record_is};
pub use words::{EVERY_WORD, Word, WordsError, declare_into, recounting_words};
