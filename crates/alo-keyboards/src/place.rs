//! Where this machine keeps the keyboard that is in front of it, as opposed to
//! the keyboards each person who signs in has chosen.
//!
//! [`crate::keeping`] is handed a path because *where a person's folder is* is
//! not a question about keyboards. This file is the one path this crate does
//! name, and it names it because of who needs the answer: **the sign-in
//! screen, where there is no person yet**.
//!
//! # Why a machine has a keyboard of its own
//!
//! A person's `keyboards.toml` cannot answer the greeter. It lives in their
//! folder, their folder is theirs because they signed in, and they sign in by
//! typing a password — through a layout something had to choose before anybody
//! knew who they were. So the keyboard in front of a machine is a fact about
//! the machine, settled when it was first started, and kept where a process
//! running as root before any session can read it.
//!
//! It is the same file in the same shape, read by [`crate::keeping::read`] and
//! written by [`crate::keeping::keep`], in a different folder. A second format
//! for the same five values would be a second thing to get wrong.
//!
//! # What named it before this
//!
//! `Environment=ALO_KEYBOARD=gb` in `alo-compositor.service`, which was the
//! same on every machine this repository has ever written and could not be
//! changed by anything on a running one. A person on a French keyboard typed
//! their password through a British layout, and the only way out was to edit a
//! unit file as root — which is to say, to already be signed in.

/// This machine's own `keyboards.toml`.
///
/// `/etc/alo` is where this system keeps what is true of the machine rather
/// than of a person: `alo_accounts::THE_ACCOUNTS` and `agentd.toml` are its
/// neighbours. It is root-owned, which is what makes it readable by a greeter
/// and unwritable by anybody who has not been let in.
pub const THE_MACHINES_KEYBOARDS: &str = "/etc/alo/keyboards.toml";
