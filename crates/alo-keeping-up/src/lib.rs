//! What an update is, and what it may never do.
//!
//! `docs/features.md` promises **updates that never interrupt**, and every
//! operating system says that and most mean *we will interrupt you later
//! instead*. The promise is only keepable if what an update may do is decided
//! before anything downloads, and decided as a type rather than as a habit —
//! which is all this crate is.
//!
//! # What is here
//!
//! - **What an update is.** [`Digest`] names one build of the operating
//!   system. [`Running`] is the one this machine is on, [`Offered`] is the one
//!   the place updates come from offers, and [`Standing::between`] says whether
//!   they differ. An update is exactly that difference and nothing more: it
//!   carries no priority, no severity and no deadline, because each of those
//!   is a lever somebody pulls to justify interrupting a person.
//! - **Asking is an errand.** A check for an update leaves this machine, so it
//!   is [`alo_egress::Errand::CheckingForAnUpdate`] on the same indicator as
//!   everything else that leaves ([`a_check_at`]). An [`Offered`] can only be
//!   heard *during* an [`alo_egress::Underway`] for that errand, and the only
//!   maker of one of those is the indicator — so no answer about updates exists
//!   that was not shown while it was fetched. ★ *No telemetry* is checkable only
//!   by somebody who can see what their machine does unasked.
//! - **What an update may never do.** [`THE_RULE`] is the promise as a value:
//!   an update never restarts the machine, never closes an application and
//!   never interrupts what a person is doing. There is no variant meaning
//!   *urgent* that could bypass it, because *urgent* is the word every
//!   interrupting system used.
//! - **When it applies.** [`WhenItApplies`] is the person's choice, and it has
//!   two members: at the next restart they make, or now because they asked.
//!   There is no member meaning *never*, because a machine that can be left
//!   unpatched by a checkbox is a fleet's worst liability, and none meaning
//!   *by itself*, because the choice is *when*, never *whether to be told*.
//!
//! # What is not here
//!
//! **No clock, no scheduler and no fetching.** Nothing in this crate reads the
//! time, starts a thread, opens a socket or writes a file; its dependency list
//! is four crates long and a test reads it. When to check, and the doing of an
//! update, are later tasks that build on these types — deciding them here would
//! be deciding them before the decisions that constrain them.
//!
//! **Not the place updates come from.** [`a_check_at`] takes a
//! [`alo_egress::Destination`]; which one is the installer plan's registry, and
//! an organisation's own mirror is v1. Neither is a permission, so neither is
//! decided here.
//!
//! # Saying it in a language
//!
//! Nothing a person reads has a `Display`. [`words`] is everything this crate
//! can say, collected into the machine's one vocabulary by `alo-saying`, and no
//! sentence in it names the machinery underneath — a person is told that an
//! update is ready, never which build it is or what stages it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod checking;
pub mod digest;
pub mod never;
pub mod standing;
pub mod when;
pub mod words;

#[cfg(test)]
mod testing;

pub use checking::{NotACheck, Offered, a_check_at};
pub use digest::{Digest, NotADigest};
pub use never::{Cause, Disturbance, Forbidden, THE_RULE, TheRule};
pub use standing::{Ready, Running, Standing};
pub use when::WhenItApplies;
pub use words::{EVERY_WORD, WordsError, declare_into, keeping_up_words};
