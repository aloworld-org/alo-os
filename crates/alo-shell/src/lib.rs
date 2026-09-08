//! Native shell's Wayland server core (ADR 0002).
//!
//! Owns a private display socket, XDG toplevel buffer lifetimes and a nested
//! Wayland/GLES rendering backend with keyboard routing and an optional pointer
//! seat core and client cursor rendering. Popup protocol tracking is opt-in;
//! parent leave notifications and session entry remain separate
//! components. This library exposes no agent capability,
//! context capture, command execution or clipboard protocol.

#![cfg(target_os = "linux")]

mod active_session;
mod atomic_inventory;
mod atomic_output;
mod atomic_test;
mod cursor;
mod default_cursor;
mod direct_input_loop;
mod direct_keyboard;
mod direct_loop;
mod direct_output;
mod direct_pointer;
mod direct_seat;
mod direct_session;
mod direct_target;
mod display_resources;
mod drawing;
mod drm_events;
mod drm_inventory;
mod flip_gate;
mod keyboard;
mod libinput_routing;
mod libinput_scroll;
mod nested;
mod nested_pointer;
mod offscreen;
mod output_metadata;
mod output_retirement;
mod pointer;
mod popup_grabs;
mod popup_placement;
mod popups;
mod presentation;
mod readback;
mod resource_device;
mod scanout;
mod scanout_buffer;
mod scanout_frame;
mod scene;
mod scene_drawing;
mod scene_replacement;
mod scene_scanout;
mod seat_input;
mod server;
mod session_device;
mod session_input;
mod socket;
mod surfaces;
mod window_close;

pub use window_close::WindowCloseError;

pub use active_session::ActiveSessionResult;
pub use atomic_output::{AtomicOutput, AtomicOutputError, discover_atomic_output};
pub use cursor::Cursor;
pub use direct_keyboard::DirectKeyEvent;
pub use direct_loop::{DirectFrame, DirectLoopError, DirectLoopResult};
pub use direct_output::{DirectOutput, DirectOutputError, discover_output};
pub use direct_pointer::DirectPointerEvent;
pub use direct_seat::DirectSeatEvent;
pub use direct_session::DirectSession;
pub use direct_target::{DirectShutdownError, DirectTarget};
pub use display_resources::{DisplayResources, ResourceError, ResourceFailure};
pub use drm_events::{DisplayEvent, FlipComplete, read_display_events};
pub use flip_gate::FlipGate;
pub use keyboard::InputError;
pub use nested::Nested;
pub use nested_pointer::NestedPointerEvent;
pub use offscreen::{PreparedScanout, render_scanout};
pub use output_metadata::OutputMetadata;
pub use popups::Popup;
pub use presentation::{FrameTarget, RenderError};
pub use readback::{ReadbackError, RowOrder, ScanoutPixels, readback_xrgb};
pub use scanout::ActiveScanout;
pub use scanout_frame::XrgbFrame;
pub use scene_replacement::SceneReplacement;
pub use scene_scanout::ActiveScene;
pub use server::Server;
pub use session_device::SessionError;
pub use socket::SocketError;

pub use seat_input::{InputDispatchError, InputUpdate, SeatInput};
pub use session_input::{SessionInput, SessionInputStatus};
