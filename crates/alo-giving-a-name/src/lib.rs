//! Gives a person their own name, in a file only they can read, at every
//! sign-in.
//!
//! [ADR 0135](../../../docs/decisions/0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)
//! is why this exists. The name a person typed when they made their account is
//! in the account store, which is root's and mode `0600` because it holds every
//! password hash; their desktop runs as their own login and must not read it.
//! So at every sign-in this reads that one name, by number, and writes it where
//! only that person can read it:
//!
//! ```text
//! /var/lib/alo-names/<number>   0600, owned by <number>
//! ```
//!
//! | | |
//! |---|---|
//! | [`given_to`] | Read the name, write it beside, give it away, put it in place |
//! | [`NotGiven`] | Why it did not happen, which is always the machine being unable |
//!
//! # Its shape is `alo-giving-a-folder`'s, on purpose
//!
//! [ADR 0119](../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
//! decided that a capability belongs to the smallest thing that can hold it,
//! and this is the second thing that holds `CAP_CHOWN` under that rule rather
//! than the folder helper growing a second job. `alo-sessiond` starts it as
//! `alo-giving-a-name@<number>.service` with a number it has already checked
//! against the accounts, and the capability lives in that unit.
//!
//! # What it reads, and what it will not do
//!
//! **It reads the account store and nothing else** — no environment, nothing
//! from a network, nothing from a person. It asks the store one question, the
//! one the store already answers for the sign-in screen: *what is account N
//! called* ([`alo_accounts::Accounts::name_of`]).
//!
//! **No name ever crosses a socket.** `alo-sessiond` hands systemd a number,
//! systemd hands this a number, and the name goes from one file on this disk to
//! another — so the privileged component that opens sessions still never learns
//! who is signing in.
//!
//! **It never removes a name.** An account that is gone is never signed in to,
//! so this is never started for it.

#![cfg(target_os = "linux")]

mod failing;
mod giving;

pub use failing::NotGiven;
pub use giving::given_to;
