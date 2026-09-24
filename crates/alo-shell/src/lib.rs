//! Native shell's Wayland server core (ADR 0002).
//!
//! Owns a private display socket, XDG toplevel buffer lifetimes and a nested
//! Wayland/GLES rendering backend with keyboard routing and an optional pointer
//! seat core and client cursor rendering. Popup protocol tracking is opt-in;
//! parent leave notifications and session entry remain separate
//! components. This library exposes no agent capability,
//! context capture, command execution or clipboard protocol.
//!
//! The sign-in screen (`SignInScreen`) is drawn here and decided in
//! `alo-greeting`: it takes keystrokes through the seat, lends a name and a
//! password to `alo_greeting::Greeting`, shows only the sentences that crate
//! hands back, and hands over to the session that opens.
//!
//! The egress indicator (`EgressStatus`) is drawn here and decided in
//! `alo-egress`: `alo_indicator::Indicating` hands it the machine's own
//! indicator whenever that changes, and every session frame submitted with
//! `Nested::submit_with_egress_status` draws its lines at the far end of the
//! dock, above every client, and nothing while nothing is leaving.
//!
//! The approval surface (`ApprovalScreen`) is drawn here and decided in
//! `alo-approving`: a change an agent proposed goes up as the sentence that
//! crate hands the compositor, unedited, with two answers and neither
//! selected; each answer goes back through `alo_approving::Approving` once;
//! a change that arrives while another is open waits behind it; and a frame
//! submitted with `Nested::submit_with_approval` carries the question above
//! every client and the egress indicator above the question.
//!
//! The record window (`RecordWindow`) is drawn here and composed in
//! `alo-recounting`: the whole record, read off the disk, drawn most recent
//! first with each entry's clause at its head and the machine's own words
//! after it — refusals as plainly as what ran, and alo OS's own errands with no
//! agent invented for them. It opens by hand, with no agent involved, and
//! asking the agent *what did you do?* opens the same account; nothing here
//! writes to the record, filters it or summarises it.
//!
//! The ordinary desktop (`DesktopFrame`) is drawn here and decided elsewhere:
//! the dock on the edge `alo-dock` names, laid out for each display's own size,
//! with its status area at the far end holding the egress indicator; the accent
//! and light and dark as `alo-appearance` answers them, with terracotta never
//! offered; and two windows drawn from `alo-measuring` — what is running
//! (`RunningWindow`) and what is filling the disk (`FillingWindow`) — each
//! number the one the kernel or the count gave. The dock grants, approves and
//! revokes nothing, and neither window acts on what it shows.
//!
//! Several screens (`Screens`) are drawn here and arranged in `alo-displays`:
//! where each output sits, how large it draws and which is the main one are
//! that crate's answers, restored for a set of screens a person has arranged
//! before; each screen draws its own background, its own dock on the edge
//! `alo-dock` names, and is warmed by night light on its own. Where the windows
//! of an unplugged screen belong is `alo-displays`' answer too, and nothing
//! here adjusts an arrangement.
//!
//! The recovery screen (`RecoveryScreen`) is drawn here and decided in
//! `alo-keeping-up`: whether going back to yesterday's machine can be offered
//! at all is `GoingBack::offered`, decided before anything is offered, and a
//! machine that cannot go back reads that crate's own reason instead of an
//! offer. It is reached when the desktop will not compose
//! (`Nested::submit_desktop_or_recovery`), needs nobody signed in, opens no
//! file, and carries nothing out: choosing hands back what was decided and the
//! moment the person chose, and `alo-keeping-up`'s `Returning` through the
//! broker does the rest.
//!
//! Settings (`SettingsWindow`) is drawn here and decided in the crates that own
//! each setting: what answers questions (`alo-setting-up`, `alo-choosing`),
//! appearance, the dock and shortcuts (each through its own `keeping`), and
//! what has been granted to what — grants and pairings in one list, revoked
//! with `alo-changing`'s one call. Every value it writes goes through the
//! crate that owns it, a section whose file did not read is never written
//! over, and nothing is drawn disabled.

#![cfg(target_os = "linux")]

mod access_bus;
mod access_contrast;
mod access_magnifier;
mod access_nodes;
mod access_roles;
mod active_session;
mod approval_answers;
mod approval_keys;
mod approval_paint;
mod approval_queue;
mod approval_raster;
mod approval_screen;
mod approval_seat;
mod approval_shown;
#[cfg(test)]
mod approval_testing;
mod atomic_inventory;
mod atomic_output;
mod atomic_test;
mod booting;
mod cursor;
mod default_cursor;
mod desktop_list;
mod desktop_look;
mod desktop_paint;
mod desktop_raster;
mod desktop_seat;
#[cfg(test)]
mod desktop_testing;
mod direct_desktop;
mod direct_input_loop;
mod direct_keyboard;
mod direct_loop;
mod direct_output;
mod direct_pointer;
mod direct_seat;
mod direct_session;
mod direct_sign_in;
mod direct_target;
mod display_resources;
mod division_raster;
mod dock_raster;
mod drawing;
mod drm_events;
mod drm_inventory;
mod egress_status;
mod egress_status_mark;
mod egress_status_paint;
mod egress_status_place;
mod egress_status_raster;
#[cfg(test)]
mod egress_status_testing;
mod filling_keys;
mod filling_rows;
mod filling_window;
mod flip_gate;
mod in_use_mark;
mod in_use_paint;
mod in_use_raster;
mod keyboard;
mod libinput_routing;
mod libinput_scroll;
mod lock_background;
mod lock_clock;
mod lock_image_fit;
mod lock_pixels;
mod lock_raster;
mod lock_surface;
mod lock_texture;
mod nested;
mod nested_approval;
mod nested_control_input;
mod nested_desktop;
mod nested_egress_status;
mod nested_lock;
mod nested_pointer;
mod nested_reader_frame;
mod nested_reader_input;
mod nested_reader_session;
mod nested_record;
mod nested_recovery;
mod nested_settings;
mod nested_sign_in;
mod offscreen;
mod output_metadata;
mod output_retirement;
mod painted;
mod painted_text;
mod pointer;
mod popup_grabs;
mod popup_placement;
mod popups;
mod presentation;
mod readback;
mod record_keys;
mod record_lines;
mod record_paint;
mod record_raster;
mod record_room;
mod record_seat;
mod record_shown;
#[cfg(test)]
mod record_testing;
mod record_window;
mod recovery_keys;
mod recovery_paint;
mod recovery_raster;
mod recovery_reached;
mod recovery_screen;
mod recovery_seat;
#[cfg(test)]
mod recovery_testing;
mod resize_transaction;
mod resource_device;
mod running_keys;
mod running_rows;
mod running_window;
mod scanout;
mod scanout_buffer;
mod scanout_frame;
mod scene;
mod scene_drawing;
mod scene_native;
mod scene_replacement;
mod scene_scanout;
mod screen_background;
mod screens;
mod screens_raster;
#[cfg(test)]
mod screens_testing;
mod seat_input;
mod server;
mod session_desktop;
mod session_device;
mod session_input;
mod settings_answering;
mod settings_chord;
mod settings_granted;
mod settings_keepers;
mod settings_kept;
mod settings_keys;
mod settings_lines;
mod settings_paint;
mod settings_paired;
mod settings_places;
mod settings_raster;
mod settings_seat;
#[cfg(test)]
#[path = "../tests/unit_fixtures/settings_testing.rs"]
mod settings_testing;
mod settings_window;
mod shortcut_dispatch;
mod sign_in_entry;
mod sign_in_keys;
mod sign_in_paint;
mod sign_in_password;
mod sign_in_raster;
mod sign_in_screen;
mod sign_in_seat;
mod socket;
mod software_scanout;
pub mod status_items;
mod status_items_raster;
mod status_row;
mod surfaces;
mod window_activation;
mod window_close;
mod window_command;
mod window_control_feedback;
mod window_control_focus;
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

pub use status_items::StatusItems;
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

pub use access_bus::{NotRead, ReadAloudBus};
pub use access_contrast::Contrast;
pub use access_magnifier::{NotMagnified, magnified, magnifying};
pub use access_nodes::ReadAloudTree;
pub use active_session::ActiveSessionResult;
pub use approval_keys::ApprovalKey;
pub use approval_raster::ApprovalLook;
pub use approval_screen::{ApprovalAnswer, ApprovalOutcome, ApprovalScreen, ApprovalShows};
pub use atomic_output::{AtomicOutput, AtomicOutputError, discover_atomic_output};
pub use booting::{
    AMachineToStandOn, Stood, WouldNotStand, stand_the_sign_in_screen_up, what_this_machine_can_say,
};
pub use cursor::Cursor;
pub use desktop_look::DesktopLook;
pub use direct_desktop::TheDesktop;
pub use direct_keyboard::DirectKeyEvent;
pub use direct_loop::{DirectFrame, DirectLoopError, DirectLoopResult};
pub use direct_output::{DirectOutput, DirectOutputError, discover_output};
pub use direct_pointer::DirectPointerEvent;
pub use direct_seat::DirectSeatEvent;
pub use direct_session::DirectSession;
pub use direct_sign_in::SignedIn;
pub use direct_target::{DirectShutdownError, DirectTarget};
pub use display_resources::{DisplayResources, ResourceError, ResourceFailure};
pub use drm_events::{DisplayEvent, FlipComplete, read_display_events};
pub use egress_status::EgressStatus;
pub use egress_status_raster::EgressStatusLook;
pub use filling_keys::FillingKey;
pub use filling_window::{FillingPressed, FillingShows, FillingWindow};
pub use flip_gate::FlipGate;
pub use keyboard::InputError;
pub use nested::Nested;
pub use nested_approval::ApprovalFrame;
pub use nested_control_input::NestedControlInput;
pub use nested_desktop::DesktopFrame;
pub use nested_egress_status::EgressStatusFrame;
pub use nested_pointer::NestedPointerEvent;
pub use nested_reader_frame::NestedReaderFrame;
pub use nested_reader_session::NestedReaderSession;
pub use nested_record::RecordFrame;
pub use nested_recovery::RecoveryFrame;
pub use nested_settings::SettingsFrame;
pub use offscreen::{PreparedScanout, render_control_scanout, render_scanout};
pub use output_metadata::OutputMetadata;
pub use popups::Popup;
pub use presentation::{FrameTarget, RenderError};
pub use readback::{ReadbackError, RowOrder, ScanoutPixels, readback_xrgb};
pub use record_keys::RecordKey;
pub use record_room::RecordLook;
pub use record_window::{RecordOpened, RecordShows, RecordWindow};
pub use recovery_keys::RecoveryKey;
pub use recovery_raster::RecoveryLook;
pub use recovery_reached::Reached;
pub use recovery_screen::{RecoveryChosen, RecoveryScreen, RecoveryShows, THE_TWO_MOMENTS};
pub use running_keys::RunningKey;
pub use running_window::{RunningPressed, RunningShows, RunningWindow};
pub use scanout::ActiveScanout;
pub use scanout_frame::XrgbFrame;
pub use scene_replacement::SceneReplacement;
pub use scene_scanout::ActiveScene;
pub use screens::{ScreenPlace, Screens};
pub use screens_raster::{ScreenPicture, desk};
pub use server::Server;
pub use session_desktop::{ADisplayToStandOn, StoodUp, stand_the_desktop_up};
pub use session_device::SessionError;
pub use settings_answering::{SettingsAnswered, SettingsChoice};
pub use settings_granted::SettingsRevoked;
pub use settings_kept::SettingsKept;
pub use settings_keys::SettingsKey;
pub use settings_places::SettingsPlaces;
pub use settings_raster::SettingsLook;
pub use settings_window::{
    SettingsDid, SettingsDoors, SettingsOpened, SettingsPress, SettingsRow, SettingsSection,
    SettingsWindow,
};
pub use shortcut_dispatch::ShortcutDispatchError;
pub use sign_in_entry::{NAME_BYTES, SignInField};
pub use sign_in_keys::SignInKey;
pub use sign_in_password::PASSWORD_BYTES;
pub use sign_in_raster::SignInLook;
pub use sign_in_screen::{SignInScreen, SignInShows, Signing};
pub use socket::SocketError;
pub use window_command::WindowCommandError;
pub use window_control_focus::WindowControlFocus;
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

pub use lock_background::LockBackground;
pub use lock_raster::LockLook;
pub use lock_surface::{LockPressed, LockSurface};

#[cfg(test)]
mod lock_background_tests;
#[cfg(test)]
mod lock_raster_tests;
#[cfg(test)]
mod lock_surface_tests;
#[cfg(test)]
mod lock_testing;

mod lock_background_path;
mod lock_battery;
mod lock_image_decode;
mod nested_lock_input;
