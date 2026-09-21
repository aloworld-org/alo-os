//! Notifications, and do-not-disturb.
//!
//! A notification is the one thing on a screen that arrives uninvited, and the
//! thing most likely to put a private sentence in front of somebody else — a
//! stranger at a locked desk, a room watching a shared screen, everybody who
//! ever watches a recording. So what this crate decides is not how one looks.
//! It is **who may send one, when one is shown, and what is never said about
//! one that is not.**
//!
//! | | |
//! |---|---|
//! | [`arriving`] | The three doors: an application under a grant, the agent, alo OS |
//! | [`Notification`], [`Action`] | What one is: who, a title, a body, at most three things to do |
//! | [`Sender`] | An application, the agent, or alo OS — and an application can never be the agent |
//! | [`Picked`] | One of those things, picked — addressed to the sender and to nobody else |
//! | [`Quiet`], [`Because`] | Do-not-disturb: whether notifications are being held, and why |
//! | [`QuietHours`] | The stretch of the clock a person set aside |
//! | [`deciding::arrives`], [`Became`], [`Why`] | What became of one that arrived, and why |
//! | [`Shown`] | What a shell is handed for one that is shown: mark, word, then colour |
//! | [`Missed`], [`Waiting`] | What a person missed, in this session's memory, until they dismiss it |
//! | [`Changes`], [`Settings`], [`keeping`] | The person's two settings, kept in `notifying.toml` (ADR 0038) |
//! | [`NotSent`] | Why one was never accepted, in words naming the program |
//! | [`words`] | The twenty-three sentences this crate says |
//!
//! # The rules, one clause each
//!
//! 1. **A notification is who sent it, a title, a body and its actions**, and
//!    there is no fifth thing it may carry ([`Notification`]).
//! 2. **An application's notification arrives under its own grant**, judged by
//!    `alo-portals` against what the person granted
//!    ([`arriving::from_an_application`]). An `Allowed` for any other portal is
//!    refused rather than read as permission to notify.
//! 3. **Do-not-disturb holds every notification and shows none**
//!    ([`deciding::arrives`]), turned on by the person, by the hours they set
//!    aside, or **by the machine while the screen is being shared or recorded**
//!    — which is [`Quiet::now`]'s first question and has no setting behind it.
//! 4. **While locked, task 1's rule applies and nothing is shown.**
//!    [`deciding::arrives`] hands the notification to
//!    `alo_locking::Seat::arrives` before it looks at anything else, and what
//!    comes back carries nothing to draw.
//! 5. **What a person missed waits until they dismiss it, and is never
//!    synced** — it is in this session's own memory and reaches no disk at all
//!    ([`Missed`]).
//! 6. **An agent's own notifications are marked as the agent's**, with its mark
//!    and its word before its colour ([`Shown`], ADR 0010) — and an application
//!    cannot wear either ([`Sender::the_agent`]).
//! 7. **An application cannot notify with an action that answers an approval.**
//!    An [`Action`] is a name and a label; a [`Picked`] goes to the sender and
//!    nowhere else; and this crate does not depend on `alo-approving` at all.
//!    `tests/a_notification_cannot_answer_an_approval.rs` holds all three
//!    against a change that is really waiting.
//!
//! # Nothing here draws
//!
//! Where a notification appears, how long it stays, what it sounds like and
//! what happens when a person swipes it away are the shell's. What this crate
//! fixes is what the shell is not free to decide differently.
//!
//! # And no notification is ever read by an agent as context
//!
//! `tests/the_agent_never_reads_a_notification.rs` reads every manifest in this
//! workspace and the shipped source of the crates an agent's request is
//! answered in: none of them reaches this one. ADR 0001's context is offered at
//! the moment of invocation and is never harvested, and *what has been arriving
//! for you today* is exactly the kind of thing a background reader would want.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod action;
pub mod arriving;
mod changes;
pub mod deciding;
pub mod keeping;
pub mod missed;
mod notification;
mod picked;
pub mod quiet;
pub mod quiet_hours;
mod refusing;
mod sender;
mod shown;
mod unkept;
mod unreadable;
pub mod words;

#[cfg(test)]
mod testing;

pub use action::{Action, MOST_THINGS_TO_DO, NotAnAction};
pub use changes::{Changes, Setting, Settings};
pub use deciding::{Became, Why};
pub use missed::{Missed, NotificationId, Waiting};
pub use notification::Notification;
pub use picked::Picked;
pub use quiet::{Because, Quiet};
pub use quiet_hours::{NotAStretch, QuietHours};
pub use refusing::NotSent;
pub use sender::Sender;
pub use shown::Shown;
pub use unkept::{FileNotRead, FileNotWritten};
pub use unreadable::NotRead;
pub use words::{EVERY_WORD, Word, WordsError, declare_into, notifying_words};
