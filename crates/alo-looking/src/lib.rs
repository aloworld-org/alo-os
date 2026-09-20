//! Finding out there is an update.
//!
//! `alo-keeping-up` decided what an update is and what it may never do;
//! `alo-updating` applies one and goes back from one. Neither of them ever
//! **looked**, and that is what this crate is: the one act that asks the place
//! this machine's builds come from whether there is a newer version, and the
//! one answer a surface reads back afterwards.
//!
//! # What is here
//!
//! - **Where this machine asks** — [`Place`], read from `image/pinned.toml`
//!   through `alo_image::ThePin` rather than written down again here. One
//!   repository has one answer about where a build comes from; an
//!   organisation's own mirror is a later release rather than a second constant
//!   in this file.
//! - **When** — [`Because`], and the whole list is two: a person asked, or the
//!   machine started. **There is no schedule, no timer and no thread in this
//!   crate**, and a test reads its source to keep it that way. A check is a
//!   departure, and a machine nobody is using has no business making one.
//! - **The act** — [`look`]. It puts the errand on the indicator before it asks
//!   anything and takes it off after the answer, whichever way it ends, and the
//!   `alo_keeping_up::Offered` it hears can be heard no other way.
//! - **And afterwards, in a record** — [`Noting`], which the check is written
//!   into before it comes off the indicator, on every road out of it including
//!   every refusal. Law 1 is two halves, *visible at the moment it happens* and
//!   *afterwards in a record*, and until this existed a check had only the
//!   first.
//! - **What it fetches** — [`ThePlace`] is two questions: which names that
//!   place holds, and which build one of them is. **Never the build itself**:
//!   there is no container library here and nothing in this crate writes what a
//!   place sent it. [`TheRegistry`] is the real one, over the machine's own
//!   proxy.
//! - **Whether the place vouched for what it offers** — [`vouched_for`], read
//!   out of the names the place has already answered with, so the answer costs
//!   nothing that leaves this machine. It travels with the offer, which is how
//!   a person who would not be able to apply an update learns that **before**
//!   they choose rather than as a refusal afterwards. It verifies nothing and
//!   permits nothing: the machine's signature policy is the authority, asked
//!   by the base at the moment of staging.
//! - **What it answers** — [`Found`], or one of exactly five refusals
//!   ([`NoAnswer`]) with a sentence each. A machine with no way out at all says
//!   so **once** ([`SaidOnce`]), because the alternative is a line repeated at
//!   every asking about a situation the person already knows about.
//! - **What is kept** — [`Kept`], one file under `/var`, so a surface reads the
//!   answer back without asking again and putting a second line on somebody's
//!   indicator for a question answered a minute ago. A kept answer names the
//!   build it was about, and [`TheAnswer::said`] refuses rather than showing
//!   *an update is ready* on a machine that has moved on.
//!
//! # What is not here
//!
//! **Nothing downloads a build, and nothing stages one.** Applying an update is
//! the person's choice and `alo_updating::apply`'s work. Nothing here decides
//! *when* to check, either: [`look`] is a function, and its caller is whoever
//! put the question.
//!
//! **No setting that turns checking off.** `alo-keeping-up`'s rule since task 1
//! of the plan: a machine that can be left unpatched by a checkbox is a fleet's
//! worst liability, and the choice a person has is *when an update applies*,
//! never *whether to be told there is one*.
//!
//! **No member meaning *urgent*.** Not in [`Because`], not in [`Found`], not in
//! [`NoAnswer`]. *Urgent* is the word every interrupting system used.
//!
//! # Saying it in a language
//!
//! Nothing here has a `Display` a person reads. [`words`] is the four sentences
//! this crate adds, collected into the machine's one vocabulary by
//! `alo-saying`; the answers themselves are `alo-keeping-up`'s and are not
//! worded a second time.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod asking;
pub mod because;
pub mod found;
pub mod kept;
pub mod looking;
pub mod noting;
pub mod place;
pub mod refusing;
pub mod registry;
pub mod release;
pub mod said_once;
pub mod vouching;
pub mod words;

#[cfg(test)]
mod testing;

pub use asking::ThePlace;
pub use because::Because;
pub use found::Found;
pub use kept::{Kept, NoLongerTrue, NotKept, THE_ANSWER, TheAnswer};
pub use looking::look;
pub use noting::Noting;
pub use place::{NotAPlace, Place};
pub use refusing::NoAnswer;
pub use registry::{TheRegistry, WHILE_SOMEBODY_WAITS};
pub use release::{NotARelease, Release};
pub use said_once::{SaidOnce, Say};
pub use vouching::{the_name_vouching_for, vouched_for};
pub use words::{EVERY_WORD, WordsError, declare_into, looking_words};
