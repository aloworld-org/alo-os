//! Native shell's Wayland server core (ADR 0002).
//!
//! Owns a private display socket, XDG toplevel buffer lifetimes and a nested
//! Wayland/GLES rendering backend with keyboard routing and an optional pointer
//! seat core. Parent leave notifications, cursors and session entry remain separate
//! components. This library exposes no agent capability,
//! context capture, command execution or clipboard protocol.

#![cfg(target_os = "linux")]

mod drawing;
mod keyboard;
mod nested;
mod nested_pointer;
mod pointer;
mod presentation;
mod server;
mod socket;
mod surfaces;

pub use keyboard::InputError;
pub use nested::Nested;
pub use nested_pointer::NestedPointerEvent;
pub use presentation::{FrameTarget, RenderError};
pub use server::Server;
pub use socket::SocketError;
