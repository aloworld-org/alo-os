//! What the greeter does, before there is anything to draw.
//!
//! `crates/alo-sessiond` is the door
//! ([ADR 0024](../../../docs/decisions/0024-what-a-person-signs-in-at.md)) and
//! `crates/alo-accounts` decides who is here. **Nothing composed them**: a
//! surface had to authenticate, then knock, then render what came back, which
//! is order-sensitive glue of exactly the kind this repository turns into a
//! tested value — and a surface that authenticated and forgot to knock is a
//! screen that takes a correct password and does nothing at all.
//!
//! [`Greeting`] is that composition, and it is everything the greeter does
//! **except drawing**. The screen is `crates/alo-shell`'s and the desktop
//! lane's; nothing here opens a window, measures a font or decides a layout.
//!
//! | | |
//! |---|---|
//! | [`Standing`] | What a greeter shows before anybody types — *make an account*, or a sign-in |
//! | [`Greeting`] | The whole composition: authenticate, open the agreement, knock once |
//! | [`Greeted`] | What a surface draws afterwards: a session, or a sentence |
//! | [`Knocking`], `TheOpenersDoor` | The opener's door, and the real socket behind it |
//! | [`NotAnswered`], [`NotReadable`] | Every way the conversation does not happen, in words |
//! | [`words`] | The four sentences this crate declares, and the English beside each |
//!
//! # What it cannot do, by construction
//!
//! **It cannot open a session.** It knocks, and `alo-sessiond` — the one
//! privileged thing on this machine that may — decides. A greeter that could
//! open one itself would be ADR 0018's *one privileged component* argument
//! thrown away a second time, and this crate holds no capability, speaks to no
//! bus and names no `logind`.
//!
//! **It cannot authenticate anybody a second way.** `alo_accounts::Accounts`
//! is the only thing here that is ever asked whether a password is right, and
//! its refusal is carried rather than reworded — one machine, one answer about
//! who is here.
//!
//! **It cannot make a knock out of anything but a verified sign-in.** The one
//! call to `alo_sessiond::Knock::on_behalf_of` in this crate is in a private
//! function taking an `alo_accounts::Session`, which cannot itself exist unless
//! a password verified *and* the machine description agreed about the number.
//! `tests/a_knock_is_made_from_a_session_and_nothing_else.rs` reads this
//! crate's own source for that, the way `alo-saying` reads for a rented name.
//!
//! **It cannot keep the password.** No type here has a field for one, nothing
//! returns one, and nothing writes one anywhere: it is a `&str` that reaches
//! [`alo_accounts::Accounts::signs_in`] and is never seen again.
//! `tests/nothing_here_keeps_the_password.rs` is the check that reads the
//! crate for it.
//!
//! # Two facts, one refusal, and the same time either way
//!
//! A wrong password and a name with no account are **one sentence** — the one
//! `alo-accounts` already declares — and they take the same work, because this
//! crate adds no branch that could tell them apart. The knock happens strictly
//! after a sign-in that succeeded, so neither refusal touches a socket and
//! neither can be told from the other by a clock.
//!
//! # It says nothing to a service log
//!
//! Everything that went wrong with the machine comes back in the value —
//! [`NotAnswered`] carries the English for whoever maintains the machine, and
//! the sentence for whoever is standing at the screen. A library that printed
//! to standard error would be choosing somebody's log for them; the surface
//! that drew the screen is the process with one.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(unix)]
mod door;
mod greeted;
mod greeting;
mod knocking;
mod refusing;
mod standing;
pub mod words;

#[cfg(unix)]
pub use door::TheOpenersDoor;
pub use greeted::Greeted;
pub use greeting::Greeting;
pub use knocking::Knocking;
pub use refusing::{NotAnswered, NotReadable};
pub use standing::Standing;
pub use words::{
    ACCOUNTS_UNREADABLE, EVERY_WORD, MAKE_AN_ACCOUNT, NOTHING_LISTENING, NOTHING_SAID, Word,
    WordsError, declare_into, greeting_words,
};
