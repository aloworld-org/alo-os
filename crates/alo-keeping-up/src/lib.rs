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
//! - **What applying one is.** [`Deployments`] is what the base reports it has
//!   on the disk — the build booted, the one staged and the one before — so
//!   *what am I running* is the machine's answer rather than anything
//!   remembered. [`Staging`] is the one instruction the base is given, decided
//!   as a value against a [`Source`] and the deployments as they are *now*:
//!   the offered build by digest, the signature policy enforced, no argument
//!   naming a path, and a restart only when the person asked for one.
//!   [`NotStaged`] is every refusal before anything runs.
//! - **What changed across a restart.** [`Since`] compares the build last known
//!   with the build booted, and an update is exactly a different one — from
//!   which, to which — with nothing invented when nothing was known.
//!
//! # What is not here
//!
//! **No clock, no scheduler and no fetching.** Nothing in this crate reads the
//! time, starts a thread, opens a socket or writes a file; its dependency list
//! is four crates long and a test reads it. **The doing of an update is
//! `alo-updating`'s**: it runs the base with the arguments [`Staging`] wrote,
//! reads the status [`Deployments`] parses, and keeps the fact [`Since`]
//! compares. When to check is a later task that builds on these types.
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
pub mod deployments;
pub mod digest;
pub mod never;
pub mod since;
pub mod source;
pub mod staging;
pub mod standing;
pub mod when;
pub mod words;

#[cfg(test)]
mod testing;

pub use checking::{NotACheck, Offered, a_check_at};
pub use deployments::{Deployments, NotRunningABuild};
pub use digest::{Digest, NotADigest};
pub use never::{Cause, Disturbance, Forbidden, THE_RULE, TheRule};
pub use since::Since;
pub use source::{NotASource, Source};
pub use staging::{NotStaged, Staging};
pub use standing::{Ready, Running, Standing};
pub use when::WhenItApplies;
pub use words::{EVERY_WORD, WordsError, declare_into, keeping_up_words};
