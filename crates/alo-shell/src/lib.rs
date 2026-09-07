//! Native shell's Wayland server core (ADR 0002).
//!
//! Owns a private display socket and XDG toplevel buffer lifetimes. A backend
//! drives dispatch and consumes mapped surfaces; rendering, input and session
//! entry are separate components. This library exposes no agent capability,
//! context capture, command execution or clipboard protocol.

#![cfg(target_os = "linux")]

mod server;
mod socket;
mod surfaces;

pub use server::Server;
pub use socket::SocketError;
