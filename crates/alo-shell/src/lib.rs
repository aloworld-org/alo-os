//! Native shell's Wayland server core (ADR 0002).
//!
//! Owns a private display socket, XDG toplevel buffer lifetimes and a nested
//! Wayland/GLES rendering backend with keyboard routing and an optional pointer
//! seat core and client cursor rendering. Popup protocol tracking is opt-in;
//! parent leave notifications and session entry remain separate
//! components. This library exposes no agent capability,
//! context capture, command execution or clipboard protocol.

#![cfg(target_os = "linux")]

mod atomic_inventory;
mod atomic_output;
mod atomic_test;
mod cursor;
mod direct_output;
mod direct_session;
mod display_resources;
mod drawing;
mod drm_events;
mod drm_inventory;
mod flip_gate;
mod keyboard;
mod nested;
mod nested_pointer;
mod pointer;
mod popup_grabs;
mod popup_placement;
mod popups;
mod presentation;
mod resource_device;
mod scanout;
mod scanout_buffer;
mod scanout_frame;
mod scene;
mod server;
mod session_device;
mod socket;
mod surfaces;

pub use atomic_output::{AtomicOutput, AtomicOutputError, discover_atomic_output};
pub use cursor::Cursor;
pub use direct_output::{DirectOutput, DirectOutputError, discover_output};
pub use direct_session::DirectSession;
pub use display_resources::{DisplayResources, ResourceError, ResourceFailure};
pub use drm_events::{DisplayEvent, FlipComplete, read_display_events};
pub use flip_gate::FlipGate;
pub use keyboard::InputError;
pub use nested::Nested;
pub use nested_pointer::NestedPointerEvent;
pub use popups::Popup;
pub use presentation::{FrameTarget, RenderError};
pub use scanout::ActiveScanout;
pub use scanout_frame::XrgbFrame;
pub use server::Server;
pub use session_device::SessionError;
pub use socket::SocketError;
