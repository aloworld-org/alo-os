//! **Which network interfaces this machine has, and the addresses they hold**,
//! as the kernel reports them on its routing socket.
//!
//! Three files, one question each:
//!
//! - `interface` — what one interface is: its index, name, flags and
//!   addresses;
//! - `route_messages` — the two requests and how the kernel's answers are
//!   read, as a function of bytes so a message the kernel never sent is still a
//!   test;
//! - `route_socket` — the one place a socket is opened, a request sent and the
//!   answer read back.
//!
//! # Who reads it
//!
//! `alo-agentd`, which joins discovery on every network the machine is on, and
//! `alo-desktop`, which tells Settings this machine's own addresses at the
//! moment a person opens the form that adds a provider (ADR 0021, owner's note
//! of 2026-10-30). They read the same bytes the same way because they read them
//! here. **What is joined, and what an address means, is decided by each caller**
//! and by nothing in this crate.
//!
//! # Asked, never watched
//!
//! `route_messages::reported_by_the_kernel` opens a socket, asks once, reads
//! the answer and closes it. Nothing here subscribes to changes or keeps a
//! socket open; a caller that wants to know when the networks change has its
//! own socket for that.
//!
//! On any host but Linux this crate is empty.

#[cfg(target_os = "linux")]
pub mod interface;
#[cfg(target_os = "linux")]
pub mod route_messages;
#[cfg(target_os = "linux")]
mod route_socket;
