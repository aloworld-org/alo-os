//! What a locked session is, and what the lock screen may show.
//!
//! A lock screen is the one surface a stranger at the desk is guaranteed to
//! see. On most systems it leaks: the names of recent documents in
//! notifications, the agent's last question, an approval waiting to be tapped.
//! This crate holds what locking decides, as types, so that none of those is a
//! thing a shell could draw there by accident or a setting could turn on.
//!
//! | | |
//! |---|---|
//! | [`Seat`] | The person's session: open, or locked — and never signed out by locking |
//! | [`LockScreen`] | The four things a lock screen may show, and the egress light |
//! | [`Arrived`] | A notification, shown on an open seat and held on a locked one |
//! | [`Unlocking`] | Signing in again, through `alo-greeting`, and what came of it |
//! | [`NotWhileLocked`] | The agent's key and a waiting approval, refused at a locked machine |
//! | [`Running`], [`WhileLocked`] | What keeps running behind the lock: everything |
//! | [`Battery`] | The battery, as the lock screen may show it |
//! | [`SomebodyElse`] | The road to the greeter, which carries no session at all |
//! | [`words`] | The two things this crate says |
//!
//! # The rules, one clause each
//!
//! 1. **A locked session keeps running** — applications, turns already
//!    approved, downloads. [`Seat::locked`] is handed nothing it could stop.
//! 2. **The lock screen shows the time, the lock image, the battery, and that
//!    the machine is locked**, and nothing else that would reach a person.
//!    [`LockScreen`] has room for those four and has no generic parameter.
//! 3. **A notification arriving while locked is held and never drawn there**,
//!    with no preview variant a setting could turn on ([`Arrived::Held`]).
//! 4. **The agent overlay cannot be summoned while locked**
//!    ([`Seat::press_the_agents_key`]), and one open at the moment of locking
//!    closes.
//! 5. **A proposal waiting for approval cannot be answered from the lock
//!    screen** ([`Seat::approve`], [`Seat::decline`], [`Seat::ask`]), because
//!    approval is the signed-in person's act (ADR 0001).
//! 6. **The egress indicator still fires while locked**, drawn without naming
//!    what left ([`LockScreen::lamp`]).
//! 7. **Unlocking is authenticated by the same composition signing in uses**
//!    ([`Seat::unlocks`] is `alo_greeting::Greeting::signs_in`) and never by a
//!    second, weaker road.
//! 8. **A locked session is not a signed-out one.** Nothing here ends a
//!    session, and `tests/nothing_here_ends_a_session.rs` holds that it
//!    cannot.
//! 9. **A lock screen may offer a road to the greeter, and nothing else new**
//!    ([`Seat::somebody_else`]). What a lock screen may *show* is clause 2 and
//!    is unchanged — [`LockScreen`] still has room for exactly four things;
//!    what it may *offer* is a separate question, answered by a value that
//!    carries no session, no person and no notification
//!    ([ADR 0061](../../../docs/decisions/0061-a-locked-screen-offers-a-road-to-the-greeter.md)).
//!
//! # Nothing here draws
//!
//! The lock image is `alo-appearance`'s value, read. Where the clock goes and
//! how the lamp looks are the shell's. What this crate fixes is what the shell
//! is not free to decide differently: what may be on the screen, what waits,
//! and who may get past it.
//!
//! # And nothing here decides sleep
//!
//! Locking before a machine suspends is the sleeping plan's decision (task 2 of
//! the session-and-displays plan), made from [`Seat::locked`]. This crate knows
//! nothing about lids, timers or `logind`.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod approving;
pub mod arriving;
pub mod battery;
mod locked;
pub mod refusing;
pub mod running;
pub mod screen;
pub mod seat;
pub mod somebody_else;
mod summoning;
mod the_locked_session;
pub mod unlocking;
pub mod words;

#[cfg(test)]
mod testing;

pub use arriving::Arrived;
pub use battery::{Battery, NotABattery};
pub use locked::Locked;
pub use refusing::NotWhileLocked;
pub use running::{EVERYTHING_RUNNING, Running, WhileLocked};
pub use screen::LockScreen;
pub use seat::Seat;
pub use somebody_else::SomebodyElse;
pub use unlocking::Unlocking;
pub use words::{EVERY_WORD, LOCKED, SOMEBODY_ELSE, Word, WordsError, declare_into, locking_words};
