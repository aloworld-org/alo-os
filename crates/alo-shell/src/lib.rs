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
//! and light and dark as `alo-appearance` answers them, with deep teal never
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

pub mod a_change_waiting_reaches_the_person;
/// The chord that asks why a command is waiting, which was the only road to an
/// answer that was already built.
mod a_chord_asks_why_a_command_is_waiting;
mod a_chord_moves_between_applications;
mod a_chord_moves_the_keyboard_between_areas;
mod a_chord_opens_a_terminal;
mod a_chord_opens_the_bar;
mod a_chord_opens_the_record;
mod a_chord_reaches_its_action;
mod a_chord_sends_the_keyboard_to_a_window;
mod a_click_brings_a_window_back;
mod a_display_has_gone;
pub mod a_folder_is_picked_in_settings;
mod a_frame_per_display;
pub mod a_key_fills_in_a_provider;
pub mod a_key_reaches_a_task_window;
mod a_key_reaches_settings;
mod a_key_reaches_the_bar;
mod a_key_reaches_the_docks_surfaces;
mod a_key_reaches_the_market;
mod a_key_reaches_the_record;
mod a_key_reaches_the_task_in_front;
mod a_key_types_into_more_apps;
mod a_new_window_is_placed;
mod a_person_pins_and_unpins;
#[cfg(test)]
mod a_picture_hashed;
mod a_press_in_the_menu;
mod a_press_on_a_setup_sheet;
mod a_press_on_an_edge;
mod a_press_on_the_launcher;
mod a_press_on_the_sign_in_screen;
mod a_provider_being_added_is_drawn;
pub mod a_provider_is_added_in_settings;
mod a_sheet_that_fits;
mod a_task_window_hands_on_a_chord;
/// Scrolling a task window's conversation.
mod a_task_window_scrolls;
/// A task window's edge, carrying Close.
mod a_task_windows_edge;
mod a_wheel_over_a_sheet;
mod a_window_onto_a_list;
/// alo's words read for the emphasis a model marks in Markdown.
///
/// **Public because `task_text::WhatWasSaid` carries its marks**, as
/// `task_text` is public for `task_raster`'s reason.
pub mod alos_marked_words;
/// Closing a task window, and which ones may close.
pub mod closing_a_task_window;
pub use a_provider_is_added_in_settings::WhyNotSaved;
mod a_reader_presses;
pub use a_reader_presses::WhatAReaderPressCameTo;
mod a_scale_held_to_what_draws;
/// Typing into a task window's reply box.
mod a_screen_arrives_or_leaves;
mod access_action;
mod access_bus;
mod access_contrast;
mod access_magnifier;
mod access_nodes;
mod access_roles;
mod access_serving;
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
mod buffers_clients_hand_over;
mod canvas_a_place_remembers_time;
mod canvas_a_restore_travels;
mod canvas_arrow_pan;
mod canvas_camera;
mod canvas_command;
mod canvas_dragged_into_a_place;
mod canvas_fixed_controls;
mod canvas_never_lost;
mod canvas_pan;
mod canvas_pinch;
mod canvas_place;
mod canvas_remembered;
mod canvas_resize;
mod canvas_shift_wheel;
mod canvas_show_all;
mod canvas_space_drag;
mod canvas_the_room_show_all_fits_into;
mod canvas_the_world;
mod canvas_wheel_zoom;
mod capture_flatten;
mod capture_paint;
mod capture_raster;
/// What one key press means in the Dock's composer.
pub mod composer_keys;
mod cursor;
mod cursor_mask;
mod default_cursor;
#[cfg(test)]
mod descriptor_testing;
mod desktop_list;
mod desktop_look;
mod desktop_membership;
mod desktop_paint;
mod desktop_raster;
mod desktop_seat;
mod desktop_swipes;
#[cfg(test)]
mod desktop_testing;
mod direct_desktop;
mod direct_input_loop;
mod direct_keyboard;
mod direct_loop;
mod direct_output;
mod direct_pointer;
/// The wire client the protocol tests drive, declared **once**.
///
/// Two `#[path]` includes of one file are two modules holding two copies, and
/// each copy's unused half is dead code — which is how a shared test helper
/// turns into seven clippy errors the moment a second test wants it. Declared
/// here so every test module reaches the same one.
#[cfg(test)]
#[path = "direct_protocol_client.rs"]
mod direct_protocol_client;
mod direct_seat;
mod direct_session;
mod direct_sign_in;
mod direct_target;
mod display_lifecycle;
mod display_resources;
/// One display's scale, and the one road a logical measure takes to its pixels.
mod display_scale;
mod division_raster;
/// The Dock's own surface: a rounded panel with a hairline and a shadow,
/// rasterised with alpha because neither is a rectangle.
mod dock_history_mark;
mod dock_raster;
mod dock_surface;
mod dock_tooltip;
/// How a rounded panel is lifted off what is behind it — its corner and its
/// shadow — at the display scale its geometry is laid out in.
mod how_a_panel_is_raised;
mod the_keyboard_on_the_docks_surfaces;
pub mod what_is_on_the_clipboard;
mod where_the_docks_surface_is;

/// Where the Dock is on one display: the band on its edge and the slots
/// inside it, worked out once for the draw and for a press.
/// What a press on the Dock does: the decision, with no act in it.
mod a_press_on_the_dock;
mod a_window_the_person_opened;
mod an_application_is_started;
mod an_edge_in_its_displays_units;
mod an_icon_is_drawn_on_its_badge;
mod what_each_application_draws;
mod what_the_dock_is_filled_with;
mod what_the_machine_offers_now;
pub use an_application_is_started::{
    AWayToStartApplications, AsksAnApplicationToStart, WhoAskedToStart,
};
pub use what_the_machine_offers_now::{AnApplicationHere, WhatTheMachineOffersNow};
/// Super+Tab and Alt+Tab going round every window, the terminal and the task
/// windows included.
mod a_chord_goes_round_every_window;
/// A press on a task window's one button: Stop (ADR 0139).
mod a_press_on_the_one_button;
/// Where the application door's socket goes, and what it refuses rather than
/// serve. Public because `docs/contracts/application-door.md` publishes the
/// path and the modes, and a test holds this crate to that document.
/// The application door itself: who is at it, what they asked, and what they
/// are told. Public because `docs/contracts/application-door.md` publishes the
/// wire, and because whoever wires a compositor together serves the door.
/// The compositor's hand for the three application acts the door carries.
mod an_application_verb_is_carried_out;
/// Closing a running task asks: Stop and close, or Cancel (ADR 0139).
mod asking_before_closing;
pub mod bar_raster;
mod dock_room;
mod dock_windows;
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
mod every_windows_edge;
mod filling_keys;
mod filling_rows;
mod filling_window;
mod first_start_sheet;
pub mod first_start_words;
mod flip_gate;
mod frame_edges;
mod frame_handle;
mod frame_name;
mod how_code_runs_in_settings;
pub mod how_long_a_grant_lasts;
mod in_use_mark;
mod in_use_paint;
mod in_use_raster;
/// Vector artwork inked into a raster at its box's size.
mod inking_a_shape;
mod keyboard;
mod keyboard_sheet;
mod language_sheet;
mod libinput_gestures;
mod libinput_routing;
mod libinput_scroll;
mod lock_background;
mod lock_clock;
mod lock_pixels;
mod lock_raster;
mod lock_surface;
mod lock_texture;
mod market_keys;
mod market_measures;
/// How the window of what a person can install is drawn.
///
/// **Public for `task_raster`'s reason exactly**: a caller builds
/// [`market_raster::AMarketOnTheDisplay`] and hands it to a frame, so the
/// description has to be nameable from outside this crate.
pub mod market_raster;
mod market_window;
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
mod notification_paint;
mod notification_raster;
mod offscreen;
mod output_metadata;
mod output_retirement;
mod overview_sheet;
mod painted;
mod painted_text;
mod painted_text_wrapping;
mod panel_raster;
mod peeking_at_a_put_aside_window;
mod pointer;
mod popup_grabs;
mod popup_placement;
mod popups;
mod presentation;
mod putting_a_window_aside;
mod readback;
mod record_clock;
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
mod region_sheet;
mod resize_cursor;
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
mod server_desk;
mod session_desktop;
mod session_device;
mod session_input;
mod settings_answering;
mod settings_canvas;
mod settings_categories;
mod settings_chord;
mod settings_command;
mod settings_drawn_controls;
mod settings_drawn_page;
mod settings_drawn_picker;
mod settings_drawn_row;
mod settings_formats;
mod settings_frame;
mod settings_granted;
mod settings_keepers;
mod settings_kept;
mod settings_keys;
mod settings_measures;
mod settings_page;
mod settings_page_accessibility;
mod settings_page_alo;
mod settings_page_appearance;
mod settings_page_folders;
mod settings_page_keyboard;
mod settings_page_region;
mod settings_page_running;
mod settings_page_terminal;
mod settings_paint;
mod settings_paired;
mod settings_palette;
mod settings_picker;
mod settings_places;
mod settings_raster;
mod settings_seat;
mod settings_shell_integration;
#[cfg(test)]
#[path = "../tests/unit_fixtures/settings_testing.rs"]
mod settings_testing;
mod settings_time_zone;
mod settings_type;
mod settings_window;
mod setup_sheet_in_pixels;
mod setup_sheet_layout;
mod setup_sheet_painting;
mod setup_sheet_surface;
mod shortcut_dispatch;
mod sign_in_entry;
mod sign_in_in_pixels;
mod sign_in_keys;
mod sign_in_layout;
mod sign_in_paint;
mod sign_in_password;
mod sign_in_pointer;
mod sign_in_pointing;
mod sign_in_raster;
mod sign_in_screen;
mod sign_in_seat;
mod sign_in_surface;
pub mod sign_in_words;
mod socket;
mod software_scanout;
pub mod status_items;
mod status_row;
/// Stop, on the road a request took (ADR 0139).
mod stopping_what_was_asked;
mod surfaces;
/// Everything on the Dock that a person reads, said once a frame.
mod tab_moves_among_the_bars_controls;
/// The order a desktop's task windows are painted in: each one whole.
mod task_layers;
/// The colours a task window draws in, mapped from the design's tokens.
mod task_look;
/// A task window's furniture, rasterised.
pub mod task_raster;
/// The type styles a task window is written in, and the words a window says.
///
/// **Public because `task_raster::ATaskOnTheDisplay` is**, and it carries these
/// types: a caller that builds a task window has to be able to name the words
/// it puts in one. `examples/desktop_check.rs` is the first such caller outside
/// this crate.
pub mod task_text;
pub mod the_application_door;
pub mod the_application_doors_place;
pub mod the_bar_finds;
mod the_bars_pressable_slots;
mod the_bars_seat;
pub mod the_cards_words;
/// What a task window says when closing a running task asks.
pub mod the_closing_words;
/// The Dock's composer as a field a person types a request into.
pub mod the_composer_a_person_types_in;
mod the_desk_is_reconciled;
mod the_dock_as_a_reader_meets_it;
mod the_docks_name_is_held;
pub mod the_docks_words;
mod the_door_on_the_desktop;
mod the_edge_takes_the_keyboard;
/// The egress indicator's two arrows, from the design file's SVG.
mod the_egress_indicators_arrows;
mod the_first_account;
pub use the_first_account::{ASecret, OnTheSheet, TheAccountSheet, TheAccountShown};
pub mod the_first_start;
mod the_first_starts_icons;
mod the_formats_are_told;
mod the_keyboard_a_person_chose;
pub mod the_keyboard_picker;
mod the_language_a_person_chose;
pub mod the_language_set_up_in;
pub mod the_languages_offered;
mod the_launchers_icons;
pub mod the_machines_keyboard;
pub mod the_markets_words;
mod the_panel_reveals;
/// The person pressed Stop in a task (ADR 0139).
pub mod the_person_stops_a_task;
pub mod the_pickers_words;
mod the_pointer_in_pixels;
pub mod the_providers_words;
mod the_region_a_person_chose;
pub mod the_regions_offered;
/// The reply box's five icons, read from the design file's SVG and drawn at
/// the display's scale.
mod the_reply_boxs_icons;
pub mod the_reply_boxs_words;
mod the_session_holds_its_screens;
mod the_settings_icons;
pub mod the_settings_words;
mod the_sign_in_screens_scale;
/// The task windows a person opened by asking alo for something.
pub mod the_tasks_a_person_opened;
mod the_tasks_seat;
/// Everything in a task window that a person reads, said once a frame.
mod the_tasks_words;
/// alo's own terminal: the shell it runs, the screen it keeps, and when it has
/// the keyboard. The compositor holds it because ADR 0114 says the compositor
/// draws it, and ADR 0113's gutter could not be promised by an application.
mod the_terminal_a_person_opened;
/// alo's own terminal, drawn: the window, its transcript and the cells in it.
/// Every figure comes from `alo_terminal::measures` through this display's own
/// scale; the window's size is a share of the display, because the design's
/// 1040x800 is what it was drawn at and not what a window is.
mod the_terminals_cells_are_drawn;
/// The terminal window's own body: the panel its transcript sits on and the
/// hairline round it, in the design's colours, so the terminal reads as a window
/// rather than as text on the desktop.
mod the_terminals_frame;
/// The strip across the top of alo's own terminal: the folder the shell is in,
/// where the shell runs, and who is signed in.
mod the_terminals_strip;
mod the_window_edge_is_pointed_at;
mod the_window_edge_reveals;
mod the_zone_a_person_chose;
pub mod the_zone_picker;
pub mod the_zone_through_timedated;
mod their_displays;
/// How one person writes a moment: what they chose, the language they read,
/// and the zone they are in. Here rather than in `alo-formats`, whose header
/// says why.
mod their_formats;
mod top_controls_region;
/// What a follow-up typed in a task window carries to alo (ADR 0138).
mod what_a_follow_up_carries;
/// What a person typed, on its way to their agent and back (ADR 0109).
pub mod what_a_person_asked_goes_to_the_agent;
/// From *a person asked for this application* to *it is here*.
pub mod what_a_person_installs;
pub mod what_alo_asks_about_a_command;
mod what_an_answer_did;
/// Which window each of the three acts asks for, decided without a compositor.
mod what_an_application_act_asks_for;
#[cfg(test)]
mod what_an_application_act_asks_for_tests;
/// What becomes of a reply sent from a task window's reply box.
pub mod what_becomes_of_a_reply;
pub mod what_the_agent_may_reach;
mod what_the_dock_has_open;
/// One keypress, read as what a person typed: the compositor's half of the
/// terminal's keyboard, where the keymap is. `alo_terminal` turns the answer
/// into the bytes a shell receives, where no keymap is needed.
mod what_the_keyboard_said;
mod what_the_seat_made_of_its_input;
mod what_the_server_tells_a_frame;
mod where_a_task_window_opens;
mod where_a_window_opens;
pub mod where_the_bar_opens;
mod where_the_dock_is;
mod where_the_market_opens;
pub mod which_area_has_the_keyboard;
mod which_card_the_seat_holds;
mod which_display_a_window_is_on;
pub mod which_preview_the_pointer_is_on;
mod which_surface_claims_a_point;
/// Which kind of window is in front: the one a person went to (2026-10-31).
mod which_window_is_in_front;
/// The name of the person a desktop is for, told once at sign-in (ADR 0135).
mod whose_desktop_this_is;
/// Whether a person is arriving at the screen, asked of `logind` and systemd before the
/// sign-in screen ends as a failure.
pub mod whose_screen_it_is;
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
mod window_dividing;
mod window_edge;
mod window_edge_keys;
mod window_edge_paint;
mod window_edge_picture;
mod window_edge_pointing;
mod window_edge_reading;
mod window_edge_title;
mod window_edge_tooltip;
mod window_edge_who_draws;
mod window_full_screen;
mod window_maximize;
mod window_menu;
mod window_menu_keys;
mod window_menu_paint;
mod window_menu_reading;
mod window_minimize;
mod window_mode;
mod window_mode_plan;
mod window_move;
mod window_name_band;
mod window_number;
mod window_placement;
mod window_raise;
mod window_resize;
mod window_size;
mod window_switch;
mod zone_sheet;

pub use peeking_at_a_put_aside_window::{ThePeek, what_a_classification_does_to_a_peek};
pub use putting_a_window_aside::NotAside;
pub use status_items::StatusItems;
/// Where alo puts a window when alo is the one choosing.
///
/// `docs/design/where-a-new-window-opens.md` is the contract. Public because
/// its caller must hold the view and the person's reading direction, and
/// `Surfaces` holds neither — see the file's header for where it belongs.
pub use what_the_dock_has_open::{MORE_APPS, NOTHING, WhatTheDockHasOpen};
pub use what_the_seat_made_of_its_input::WhatTheSeatMadeOfIt;
pub use where_a_window_opens::{A_DELIBERATE_OFFSET, A_SMALL_GAP, where_a_window_opens};
pub use which_card_the_seat_holds::which_card_the_seat_holds;
pub use window_activation::WindowActivationError;
pub use window_close::WindowCloseError;
pub use window_dividing::NotDivided;
pub use window_edge::{
    A_TARGET_IS, ARTWORK_IS, BETWEEN_TARGETS, Decorations, EdgeControl, FULL_CONTROLS_NEED,
    HowItFits, OnTheEdge, THE_COMPACT_EDGE_IS, THE_DRAG_STARTS_AT, THE_GRIP_IS, THE_PADDING_IS,
    THE_REGION_IS_TALL, THE_STRIP_IS_TALL, THE_STRIP_STARTS_AT, WindowEdge, edge_of,
    the_usable_output,
};
/// What the external window edge is drawn out of: rectangles and roles.
///
/// Separate from the geometry because they are two reasons to change - the
/// design moving a control, and the design recolouring one.
/// **The keyboard's road onto the edge itself**, which is what opens the menu
/// and what holds the edge revealed while a person is using it.
pub use window_edge_keys::{AnEdgeKey, TheKeyboardOnTheEdge, WhatTheEdgeKeyDid};
pub use window_edge_paint::{Pointing, Solid, solids};
/// One window edge, laid out in owned pixels and ready for a frame.
pub use window_edge_picture::{EdgePicture, OverTheEdge};
/// **What a pointer is on, when it is on a window's edge** - the step
/// between a position and the control a press acts on. Nothing asked it
/// until 2026-10-11, which is why every edge drew correctly, announced
/// itself correctly to a reader, and could not be clicked.
pub use window_edge_pointing::{UnderThePointer, pointing_at, what_is_under};
/// **What a reader is told about a window's edge**, which is what a window
/// *has* rather than what is currently drawn: the owner's *without first
/// hovering* clause of 2026-10-09 is only true if those are different
/// questions. `crate::access_nodes` hangs these under each open window.
pub use window_edge_reading::{the_controls_of, what_a_reader_is_told, what_it_does};
pub use window_edge_title::{AN_ELLIPSIS, FittedTitle, fitted};
/// A window's title on its edge: what is shown, and what is still said aloud.
/// **The whole title, for somebody who can see and is reading it now.**
///
/// One of three roads to the same string: this is the convenient one, the
/// window menu carries it as selectable text, and the ellipsis says there is
/// more. A tooltip is transient by nature, so it cannot be read slowly, cannot
/// be copied, and on a touch screen there is no hover to produce it.
pub use window_edge_tooltip::{
    A_LINE_IS_TALL, AROUND_THE_WORDS, AT_MOST_LINES, AT_MOST_WIDE, Because, ShowingTheTitle,
    Tooltip, tooltip_of,
};
/// **Who draws a frame's header**, in the one place both the drawing and the
/// reading ask. Its body is what changes when the toplevel's decoration state
/// is read per frame; see the file's header.
pub use window_edge_who_draws::who_draws_a_frame;
pub use window_full_screen::WindowFullScreenError;
pub use window_maximize::WindowMaximizeError;
/// **The window menu**, which is what makes the compact edge honest: the
/// actions it moves off the edge are reachable here rather than nowhere.
pub use window_menu::{
    A_ROW_IS_TALL, A_ROW_IS_WIDE, BELOW_THE_EDGE, MenuRow, THE_MENU_IS_WIDE,
    THE_TITLE_BLOCK_IS_TALL, WhatARowDoes, WhatTheWindowAllows, WindowMenu, the_menu_of,
};
/// **The keyboard's road to those same actions**, and the control focus goes
/// back to when the menu closes.
pub use window_menu_keys::{
    AMenuKey, TheKeyboardInTheMenu, WhatTheKeyDid, the_edge_stays_revealed,
};
/// **And what it draws**, which is what makes it a menu rather than a list of
/// rectangles nobody can see.
pub use window_menu_paint::{PointingAtTheMenu, solids as menu_solids};
/// **And what a reader is told about it**: the panel's own name, and its rows.
pub use window_menu_reading::{
    WhatAReaderMeetsInTheMenu, every_word_this_surface_says, what_a_reader_meets,
};
pub use window_minimize::WindowMinimizeError;
pub use window_mode::WindowModeError;

/// Where a frame's name band is, for a host that has decided to reveal one.
///
/// Public for the same reason  is: the promise at
///   is that a frame's name *appears when they
/// point at it, select it or zoom out*, so **when** is the host's and this
/// crate answers only **where**.
pub use window_name_band::{BAND_IS_TALL, band_of, is_reachable};
pub use window_placement::{
    WindowPlacementError, has_been_placed, place_unless_already_placed, where_a_window_is,
    window_buffer_origin,
};
pub use window_raise::WindowRaiseError;
pub use window_resize::{ResizeEdge, ResizeGeometry, ResizeGeometryError};
pub use window_size::WindowSizeError;
pub use window_switch::{WindowSwitchDirection, WindowSwitchError};

pub use a_frame_per_display::{DrawnPerDisplay, retire_each_display};
pub use access_action::Pressed;
pub use access_bus::{NotRead, ReadAloudBus};
pub use access_contrast::Contrast;
pub use access_magnifier::{NotMagnified, magnified, magnifying};
pub use access_nodes::ReadAloudTree;
pub use access_serving::TheReaderIsTold;
pub use active_session::ActiveSessionResult;
pub use approval_keys::ApprovalKey;
pub use approval_raster::ApprovalLook;
pub use approval_screen::{ApprovalAnswer, ApprovalOutcome, ApprovalScreen, ApprovalShows};
pub use atomic_output::{
    AtomicOutput, AtomicOutputError, discover_atomic_output, discover_every_atomic_output,
};
pub use booting::{
    AMachineToStandOn, Stood, WouldNotStand, stand_the_sign_in_screen_up, what_this_machine_can_say,
};
pub use capture_flatten::{NotFlattened, burnt_in};
// `DesktopFrame::capturing` is a public field of this type, so without these a
// caller outside this crate could only ever pass `None` — the capture tools were
// drawable by this crate and by nobody else, including the fixture whose whole
// job is to draw a frame with them on it. What is exported is what a caller must
// hand in; every picture the tools become stays private.
pub use canvas_command::CanvasCommandError;
pub use canvas_fixed_controls::{FixedControlsDrawn, Recovery};
pub use canvas_never_lost::{
    A_USABLE_HANDLE, enough_of_it_is_reachable, the_longest_reachable_run,
};
pub use canvas_remembered::WhereTheyLeftIt;
pub use capture_raster::{CaptureLook, Capturing};
pub use cursor::Cursor;
pub use desktop_look::DesktopLook;
pub use direct_desktop::TheDesktop;
pub use direct_keyboard::DirectKeyEvent;
pub use direct_loop::{DirectFrame, DirectLoopError, DirectLoopResult};
pub use direct_output::{DirectOutput, DirectOutputError, discover_output};
pub use direct_pointer::{DirectPinch, DirectPointerEvent};
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
pub use frame_edges::{FrameEdge, the_resize_corner, the_resize_edge};
pub use frame_handle::the_names_band;
pub use frame_name::FrameName;
pub use keyboard::InputError;
pub use market_keys::MarketKey;
pub use market_window::{Browsing, HowItIsGoing, MarketPressed, MarketShows, MarketWindow};
pub use nested::Nested;
pub use nested_approval::ApprovalFrame;
pub use nested_control_input::NestedControlInput;
pub use nested_desktop::{DesktopFrame, TheTerminalToDraw};
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
pub use record_clock::RecordClock;
pub use record_keys::RecordKey;
pub use record_room::RecordLook;
pub use record_window::{RecordOpened, RecordShows, RecordWindow};
pub use recovery_keys::RecoveryKey;
pub use recovery_raster::RecoveryLook;
pub use recovery_reached::Reached;
pub use recovery_screen::{RecoveryChosen, RecoveryScreen, RecoveryShows, THE_TWO_MOMENTS};
pub use resize_cursor::{the_cursor_over, the_hotspot_of};
pub use running_keys::RunningKey;
pub use running_window::{RunningPressed, RunningShows, RunningWindow};
pub use scanout::ActiveScanout;
pub use scanout_frame::XrgbFrame;
pub use scene_replacement::SceneReplacement;
pub use scene_scanout::ActiveScene;
pub use screens::{ScreenPlace, Screens, TheRoom};
pub use the_markets_words::TheMarketsWords;
pub use their_formats::{TheDay, TheirFormats, WhereTheRegionCameFrom};

// **The types a desktop needs to answer `their_displays`, re-exported so that
// answering costs it no new crate edge** — `more-than-one-display-plan.md`
// task 10. `alo-desktop` depends on this crate already and not on
// `alo-displays`, which is why `the_screens_of` could not be implemented by
// the one crate that had to implement it.
//
// Re-exported rather than wrapped: a wrapper would be this crate holding its
// own copy of somebody else's answer, which `TheDesktop`'s own header forbids
// in as many words.
pub use alo_displays::{Changes, Moment, NightLight, Reported, Tonight};
pub use screens_raster::{ScreenPicture, desk};
pub use server::Server;
pub use session_desktop::{ADisplayToStandOn, StoodUp, stand_the_desktop_up};
pub use session_device::SessionError;
pub use settings_answering::{SettingsAnswered, SettingsChoice};
pub use settings_command::{SettingsCommandError, WhatTheChordDid};
pub use settings_granted::SettingsRevoked;
pub use settings_kept::SettingsKept;
pub use settings_keys::SettingsKey;
pub use settings_places::SettingsPlaces;
pub use settings_raster::SettingsLook;
pub use settings_window::{
    LightOrDark, PointerSize, SettingsCategory, SettingsDid, SettingsDoors, SettingsOpened,
    SettingsPicker, SettingsPress, SettingsRow, SettingsSection, SettingsWindow,
};
pub use shortcut_dispatch::ShortcutDispatchError;
pub use sign_in_entry::{NAME_BYTES, SignInField};
pub use sign_in_keys::SignInKey;
/// **The sign-in and unlock screen's geometry**, read from `176-10686.xml`.
pub use sign_in_layout::{
    SignInLayout, THE_ARROW_IS, THE_BACK_TARGET_IS, THE_ENTRY_CORNER_IS, THE_ENTRY_IS,
    THE_GROUP_IS_TALL, THE_GROUP_IS_WIDE, THE_MONOGRAM_IS, THE_SMALLEST_SCREEN_IS,
    THE_SUBMIT_TARGET_IS, sign_in_laid_out,
};
pub use sign_in_password::PASSWORD_BYTES;
// **`OnTheSignInScreen` is deliberately not exported.** It was, until the
// screen-reader gate in `alo-access` read this list: that test treats every
// exported name ending in `Screen` as a surface a person meets, and told me a
// reader would be told nothing about this one. It was right to ask. The answer
// is not an entry in its exception list — the enum is not a surface, it is what
// a hit test answers — and nothing outside this crate calls it, so a public
// name here would be a contract owed forever to no caller.
pub use sign_in_raster::SignInLook;
pub use sign_in_screen::{SignInScreen, SignInShows, Signing};
pub use socket::{SocketError, the_display_a_client_reaches};
pub use their_displays::{THE_SIZES_THIS_COMPOSITOR_DRAWS, TheirDisplays};
pub use top_controls_region::the_top_controls;
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
#[cfg(test)]
mod buffers_clients_hand_over_tests;
#[cfg(test)]
mod lock_raster_tests;
#[cfg(test)]
mod lock_surface_tests;
#[cfg(test)]
mod lock_testing;
#[cfg(test)]
mod the_application_door_tests;
#[cfg(test)]
mod the_application_doors_place_tests;

mod lock_battery;
mod nested_lock_input;
