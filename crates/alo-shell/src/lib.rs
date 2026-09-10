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
mod nested_control_input;
mod nested_pointer;
mod nested_reader_frame;
mod nested_reader_input;
mod offscreen;
mod output_metadata;
mod output_retirement;
mod pointer;
mod popup_grabs;
mod popup_placement;
mod popups;
mod presentation;
mod readback;
mod resize_transaction;
mod resource_device;
mod scanout;
mod scanout_buffer;
mod scanout_frame;
mod scene;
mod scene_drawing;
mod scene_native;
mod scene_replacement;
mod scene_scanout;
mod seat_input;
mod server;
mod session_device;
mod session_input;
mod shortcut_dispatch;
mod socket;
mod surfaces;
mod window_activation;
mod window_close;
mod window_command;
mod window_control_feedback;
mod window_control_frame;
mod window_control_input;
mod window_control_label;
mod window_control_label_expansion;
mod window_control_label_page_raster;
mod window_control_label_pages;
mod window_control_label_paint;
mod window_control_label_target;
mod window_control_name_fallback;
mod window_control_overlay;
mod window_control_paint;
mod window_control_presentation;
mod window_control_reader;
mod window_control_reader_chrome;
mod window_control_reader_frame;
mod window_control_reader_input;
mod window_control_reader_interaction;
mod window_control_reader_keys;
mod window_control_reader_navigation;
mod window_control_reader_pointer;
mod window_control_reader_scene;
mod window_control_reader_selection;
pub mod window_control_reader_words;
mod window_control_routing;
mod window_control_scene;
mod window_control_snapshot;
mod window_controls;
mod window_maximize;
mod window_minimize;
mod window_mode;
mod window_mode_plan;
mod window_move;
mod window_placement;
mod window_press;
mod window_raise;
mod window_resize;
mod window_size;
mod window_switch;
mod window_tiling;

pub use window_activation::WindowActivationError;
pub use window_close::WindowCloseError;
pub use window_maximize::WindowMaximizeError;
pub use window_minimize::WindowMinimizeError;
pub use window_mode::WindowModeError;
pub use window_placement::{WindowPlacementError, window_buffer_origin};
pub use window_raise::WindowRaiseError;
pub use window_resize::{ResizeEdge, ResizeGeometry, ResizeGeometryError};
pub use window_size::WindowSizeError;
pub use window_switch::{WindowSwitchDirection, WindowSwitchError};
pub use window_tiling::{TileGeometry, TileGeometryError, TileSide};

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
pub use nested_control_input::NestedControlInput;
pub use nested_pointer::NestedPointerEvent;
pub use nested_reader_frame::NestedReaderFrame;
pub use offscreen::{PreparedScanout, render_control_scanout, render_scanout};
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
pub use shortcut_dispatch::ShortcutDispatchError;
pub use socket::SocketError;
pub use window_command::WindowCommandError;
pub use window_control_frame::WindowControlFrame;
pub use window_control_frame::WindowControlLabelFrame;
pub use window_control_input::{
    WindowControlPressError, WindowControlRelease, WindowControlReleaseError,
};
pub use window_control_label::{
    LabelGeometry, WindowControlLabel, WindowControlLabelError, WindowControlLabels,
};
pub use window_control_label_pages::{
    WindowControlLabelPage, WindowControlLabelPages, WindowControlPageError,
};
pub use window_control_label_target::{WindowControlLabelSelection, WindowControlLabelTarget};
pub use window_control_reader::{
    WindowControlReader, WindowControlReaderPage, WindowControlReaderStyle,
};
pub use window_control_reader_chrome::PreparedWindowControlReaderChrome;
pub use window_control_reader_frame::WindowControlReaderFrame;
pub use window_control_reader_input::WindowControlReaderInput;
pub use window_control_reader_interaction::WindowControlReaderInteraction;
pub use window_control_reader_navigation::{
    WindowControlReaderChrome, WindowControlReaderNavigation,
};
pub use window_control_reader_scene::WindowControlReaderScene;
pub use window_control_routing::{
    PaintedWindowControls, WindowControlPointerEvent, WindowControlRoute, WindowControlRouteError,
};
pub use window_control_scene::WindowControlScene;
pub use window_control_snapshot::{WindowControlSnapshot, WindowControlSnapshotError};
pub use window_controls::{
    WindowControl, WindowControlFeedback, WindowControlLayout, WindowControlLayoutError,
};

pub use seat_input::{InputDispatchError, InputUpdate, SeatInput};
pub use session_input::{SessionInput, SessionInputStatus};

pub use window_control_reader_keys::{ReaderKeyCommand, ReaderKeyRoute, WindowControlReaderKeys};
pub use window_control_reader_pointer::{
    ReaderPointerFeedback, ReaderPointerHit, WindowControlReaderPointer,
};
