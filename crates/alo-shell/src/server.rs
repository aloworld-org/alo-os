//! Display ownership and bounded, nonblocking dispatch for graphics backends.

use std::{io, path::Path, sync::Arc};

use smithay::reexports::wayland_server::{Display, protocol::wl_surface::WlSurface};
use smithay::utils::{Physical, Point, Rectangle};

use crate::{
    SocketError,
    socket::Socket,
    surfaces::{ClientState, Surfaces},
};

/// A native Wayland display, independent of the nested or direct graphics backend.
///
/// The caller supplies a private runtime directory and a fresh session name.
/// No environment is mutated and no client process is launched. Drop closes
/// clients and removes the owned socket. Dispatch must be driven by the backend.
pub struct Server {
    /// Submitted opaque label and pointer releases owned independently of pixels.
    pub(crate) control_overlay: crate::window_control_overlay::Overlay,
    /// Native control ownership is separate from client pointer grabs.
    pub(crate) control_press: Option<crate::window_control_input::Press>,
    /// Explicitly composed strip and mapping-bound native label focus.
    pub(crate) control_presentation: Option<crate::window_control_presentation::Presentation>,
    /// Successfully submitted reader/page identity and exact hit bounds.
    pub(crate) reader_presentation: Option<crate::window_control_reader_frame::ReaderPresentation>,
    /// The display is private so every inserted client has our client state.
    display: Display<Surfaces>,
    /// Protocol state and mapped toplevels.
    pub(crate) surfaces: Surfaces,
    /// Listener and private directory lifetime.
    socket: Socket,
    /// One output and its successfully submitted surface membership.
    /// **One per display, keyed by the output's own name.**
    ///
    /// `OutputMetadata::name` is session-unique, so it is what tells two
    /// displays apart — and until 2026-10-05 holding one of these was what
    /// made a second display impossible rather than merely undrawn:
    /// `Presentation::validate_target` refuses a target whose identity differs
    /// from the one it has seen, so handing the server a second display's
    /// frame was answered `OutputIdentityChanged`. That refusal is right
    /// **within** one display — a connector whose make and model changed under
    /// the same name is a fault — and it was standing in for *this compositor
    /// has one output*.
    ///
    /// `docs/autonomy/more-than-one-display-plan.md` task 3.
    pub(crate) presentations: std::collections::BTreeMap<String, crate::presentation::Presentation>,
    /// **What number each display is known by**, keyed by the name it
    /// advertises.
    ///
    /// `display_lifecycle`'s `THE_DISPLAY` was a constant 1 until 2026-10-05,
    /// with a note saying it would go the moment a second output was
    /// advertised. `crate::display_lifecycle::Server::the_number_for` is what
    /// replaced it, and task 8 is why.
    pub(crate) display_numbers: std::collections::BTreeMap<String, alo_desktops::DisplayId>,
    /// **Where this session's screens are**, as the desktop arranged them.
    ///
    /// `None` until a desktop answers with one. The compositor describes the
    /// displays it has presented and `alo-displays` decides where they sit —
    /// see `crate::the_session_holds_its_screens`, and
    /// `more-than-one-display-plan.md` task 3a for why tasks 5, 6 and 7 all
    /// wait on it.
    pub(crate) screens: Option<crate::Screens>,
    /// Stable mapped-root order for trusted window cycling.
    pub(crate) switch_order: crate::window_switch::SwitchOrder,
    /// **How each display is divided and what desktops are on it.**
    ///
    /// Held here because it is a session's, and kept across windows opening
    /// and closing and displays arriving and leaving. Nothing about a layout
    /// or a desktop is decided in this crate: `crate::server_desk` says what
    /// that means and which crate answers which question.
    pub(crate) desk: crate::server_desk::Desk,
    /// **Where the fixed controls are, as the last draw laid them out.**
    ///
    /// Held here because a drag has to ask, and the Dock's bounds otherwise exist
    /// only inside the raster path at draw time — which is why
    /// `crate::canvas_never_lost`'s rule had no caller until this field existed.
    /// `crate::canvas_fixed_controls` carries the reasoning and the units.
    pub(crate) fixed_controls:
        std::collections::BTreeMap<String, crate::canvas_fixed_controls::FixedControls>,
    /// **Where the put-aside panel is, as the last draw laid it out.**
    ///
    /// Held for exactly the reason `fixed_controls` above is, and the parallel is
    /// the argument: the panel's slots are computed in `crate::panel_raster` and
    /// nowhere else, so outside a frame they do not exist. A pointer arriving
    /// between frames has nothing to be tested against, which is why
    /// `crate::which_preview_the_pointer_is_on` had no geometry to ask about
    /// until this field existed.
    ///
    /// `None` until the first draw, and that is the honest answer rather than an
    /// empty picture: a panel nobody has laid out has no slots *and* no reserved
    /// column, and a default would claim an edge that has not been measured on
    /// this display. `crate::peeking_at_a_put_aside_window` answers `Ok(None)`
    /// there instead of inventing one.
    pub(crate) panel_as_drawn: Option<crate::which_preview_the_pointer_is_on::ThePanelAsDrawn>,
    /// **Where each display's Dock is, as the last draw laid it out.**
    ///
    /// The same shape and the same argument as `panel_as_drawn` above and
    /// `fixed_controls` before it: the band and its slots are computed in
    /// `crate::where_the_dock_is`, called from the draw, so outside a frame they
    /// do not exist. A press arriving between frames has nothing to be tested
    /// against, which is why nothing in this compositor could press the Dock
    /// until 2026-10-10.
    ///
    /// **Keyed by display, like `fixed_controls` and unlike `panel_as_drawn`.** A
    /// person with a laptop and a screen beside it has a Dock on each, laid out
    /// for each display's own size, and a single slot here would let a press on
    /// one display be answered with the other's geometry. The pointer's location
    /// is global and these bands are too.
    ///
    /// Empty until the first draw, which is the honest answer rather than a
    /// default: a Dock nobody has laid out has no slots, and a guess would claim
    /// a band that has not been measured on this display.
    pub(crate) dock_as_drawn:
        std::collections::BTreeMap<String, crate::where_the_dock_is::TheDocksPlaces>,
    /// Where each display's **open surface** was drawn, when one was.
    ///
    /// Beside `dock_as_drawn` and not inside it, because the two have different
    /// lifetimes: a band is drawn on every frame and a popover only while it is
    /// open. A field inside the band's record would have to carry an `Option`
    /// that every reader checked anyway, and an entry left behind by a surface
    /// that has closed is a press taken by something no longer on the screen.
    ///
    /// **Cleared by a draw with nothing open**, which is how it cannot go
    /// stale: the draw that closes a surface is the draw that removes this.
    pub(crate) the_docks_surface_as_drawn:
        std::collections::BTreeMap<String, crate::where_the_docks_surface_is::TheSurfaceAsDrawn>,
    /// Whether the list of applications the bar had no room for is open.
    ///
    /// **One home, and it is here rather than on the desktop**, because the
    /// thing that turns it over is a press and presses arrive here. The draw
    /// reads it; nothing else writes it.
    ///
    /// It is **not** per display. The overflow is a list a person opened, not a
    /// property of a screen, and two displays showing two open lists for one
    /// press would be two answers to one question.
    pub(crate) the_dock_was_asked_for: crate::what_the_dock_has_open::WhatTheDockWasAskedFor,
    /// Windows the **person** has asked to put aside, not yet acted on.
    ///
    /// # Why a request rather than the act
    ///
    /// `Server::put_this_window_aside` needs the `Panel`, and the `Panel` belongs to
    /// whoever implements `crate::TheDesktop` — the compositor holds no session state, by
    /// that trait's own argument. But the two places a person asks are
    /// `crate::window_command` (the keyboard) and `crate::window_control_input` (the
    /// button on a window's own controls), and both are `Server` methods with no desktop
    /// in scope.
    ///
    /// So the ask is recorded here and performed in `crate::direct_desktop`, which is the one
    /// place holding both. The same shape as `panel_as_drawn` above, in the other direction:
    /// that one carries what the draw knew to a question asked later, this one carries what
    /// the input knew to an act performed later.
    ///
    /// **A list rather than one, because two presses in one batch are two windows.** Keeping
    /// only the latest would silently drop a window a person asked to put away, which is the
    /// one outcome this whole surface exists to prevent.
    pub(crate) asked_to_put_aside: Vec<WlSurface>,
    /// What a person pressed on a window's edge, not yet carried out.
    ///
    /// §6 of `docs/design/the-external-window-edge.md`. Recorded here and met
    /// in `crate::direct_desktop` for `asked_to_put_aside`'s reason above:
    /// Minimise needs the live `Panel`, and input has no desktop in scope.
    /// Close and Maximise join it rather than acting in the press handler
    /// because their mechanisms answer with their own error types, and an input
    /// path that returned `InputError` for a refused close would be
    /// relabelling somebody else's failure.
    ///
    /// **A list, like the one above, because two presses in one batch are two
    /// windows** — and because a person may press Close on one window and
    /// Minimise on another before either frame is drawn.
    pub(crate) asked_on_an_edge: Vec<(WlSurface, crate::window_edge::OnTheEdge)>,
    /// Which applications a person asked to start, not yet started.
    ///
    /// `docs/features.md` [v0.01] *clicking an application — **nothing open:**
    /// it starts.* Recorded here and carried out off the input path, for
    /// `asked_to_put_aside`'s reason and one of its own.
    ///
    /// **Starting an application is a bus call that waits for the bus to start
    /// it.** `alo_portals::activated` sends `Activate` to the application's own
    /// name, and the bus launches it if it is not running — which is the right
    /// mechanism and is not a fast one. Made from the press handler it would
    /// hold the compositor's input loop for however long an application takes
    /// to answer, so **every window on the machine would stop drawing while one
    /// started.** A person pressing a Dock icon would see the desktop freeze.
    ///
    /// So the press records the ask and returns, and whoever has an async
    /// context sends it. That is the same division `asked_on_an_edge` makes and
    /// for a stricter reason: that one waits because input has no desktop, this
    /// one waits because input must not block.
    ///
    /// **A list, like the two above**, because two presses in one batch are two
    /// applications — and because the one outcome to avoid is silently dropping
    /// something a person asked for.
    pub(crate) asked_to_start: Vec<alo_dock::AppId>,
    /// Which application this machine's terminal is, when something has told
    /// it.
    ///
    /// **[`None`] is the honest state and not a placeholder**, exactly as
    /// `asks_an_application_to_start` below is: which applications are
    /// installed is a scan of the machine, `crates/alo-shell` shows and never
    /// measures, and a session nobody has told has no terminal to open rather
    /// than a guess at one. See `crate::a_chord_opens_a_terminal`.
    pub(crate) the_terminal: Option<alo_dock::AppId>,
    /// What each application draws on the Dock, where it draws anything.
    ///
    /// **Empty is the decided fallback and not a fault**: every application
    /// then draws its first letter, which `docs/design/the-alo-dock.md` rules
    /// is the answer for one with no artwork. See
    /// `crate::what_each_application_draws` for why it is handed in.
    pub(crate) what_each_application_draws: std::sync::Arc<alo_dock::artwork::TheArtwork>,
    /// **alo's own terminal**, when a person has opened one.
    ///
    /// Distinct from [`Self::the_terminal`] above, and the pair is worth
    /// reading together: that one is *which installed application this
    /// machine's terminal is*, a third-party program started on the bus; this
    /// one is **alo's**, drawn by this compositor, running the person's shell
    /// on a pseudoterminal this process owns.
    ///
    /// Both exist because the installed one was the only terminal there was
    /// until alo had its own. The chord now opens this one first — see
    /// `crate::a_chord_opens_a_terminal` for the judgement and why nothing is
    /// taken away by it.
    ///
    /// **[`None`] is a machine where nobody has opened a terminal**, which is
    /// how every machine starts. Nothing opens one on a person's behalf.
    pub(crate) alos_own_terminal: Option<crate::the_terminal_a_person_opened::TheTerminal>,
    /// The name of the person this desktop is for, as they typed it when they
    /// made their account — told once at sign-in by
    /// `Server::its_person_is_called`, and shown in the terminal's strip.
    ///
    /// **[`None`] is no name, never the login's**: the desktop runs as `alo`
    /// on every machine, which is nobody's name (ADR 0135).
    pub(crate) whose_desktop: Option<String>,
    /// How this machine asks an application to start, when something has told
    /// it.
    ///
    /// **[`None`] is the honest state and not a placeholder.** The sender is
    /// `alo_portals::activated`, and `alo-portals` is a dev-dependency of this
    /// crate rather than a real one — the compositor has no business reaching
    /// portal policy. So the implementation is supplied from outside by
    /// whoever stands the desktop up, and until it is, **nothing starts and
    /// every ask is reported as not started.**
    ///
    /// That is the safe direction. A machine with no way to start an
    /// application says so for every press; the alternative — a default that
    /// silently succeeds — would be a press a person believes worked.
    /// *Absent is stronger than defaulted.*
    ///
    /// Set once rather than threaded through every frame, because it is
    /// configuration and not state: it is the same thing on the first frame
    /// and the thousandth, and a parameter on the draw path would be four
    /// layers of signature for a value that never changes.
    pub(crate) asks_an_application_to_start:
        Option<crate::an_application_is_started::AWayToStartApplications>,
    /// What this person's Dock is pinned to.
    ///
    /// **Empty until `its_dock_is_filled_from` is called**, which is at
    /// stand-up and nowhere else — the pins are configuration, like the way to
    /// start an application above, and re-deriving them per frame would undo a
    /// removal on the frame after it was made. See
    /// `crate::what_the_dock_is_filled_with`.
    pub(crate) what_the_dock_is_pinned_to: alo_dock::Holding,
    /// The provider a person is filling in, while they are.
    ///
    /// **`None` is the ordinary state and is also the mode**: there is no
    /// flag saying whether a form is open, because a form being open is a
    /// form existing. See `crate::a_provider_is_added_in_settings`.
    pub(crate) filling_in: Option<alo_setting_up::a_provider_is_filled_in::AProviderBeingAddded>,
    /// What a person copied somewhere else, on its way to a field of ours.
    ///
    /// **Held across frames rather than per press**, because a paste is asked
    /// for in one frame and arrives in another: the selection belongs to
    /// another client, which writes when it gets round to it.
    pub(crate) clipboard: crate::what_is_on_the_clipboard::TheClipboard,
    /// A provider a person pressed Enter on, waiting for the loop to save it.
    ///
    /// **Put aside rather than saved where the press arrived**, because
    /// saving is two writes to the machine and this crate shows rather than
    /// measures. The same shape `asked_alo` has for a question.
    pub(crate) asked_to_save_a_provider:
        Option<alo_setting_up::a_provider_is_filled_in::WhatToSave>,
    /// Why the provider on screen was not saved, when it was not.
    ///
    /// **Beside the form rather than inside it**, because the form holds what a
    /// person typed and this holds what the machine answered. They clear at
    /// different moments: what they typed stays until it saves, and the answer
    /// goes the instant they touch a key, because by then it is about a form
    /// that no longer exists.
    pub(crate) a_provider_was_not_saved:
        Option<crate::a_provider_is_added_in_settings::WhyNotSaved>,
    /// What a person has typed into *More apps*' search field.
    ///
    /// **Here and not beside the open-surface state**, which is where the
    /// focused cell lives. A person who closes the overview and opens it again
    /// has started a new search — and that is a fact about the surface opening,
    /// so `the_overflow_closes` clears this rather than the type carrying it.
    pub(crate) what_was_typed_into_more_apps: alo_dock::WhatTheyTyped,
    /// A pinned list a person changed, waiting to be written down.
    ///
    /// **Recorded here rather than written**, for `asked_to_start`'s reason
    /// exactly: this crate keeps no settings file of its own, so the desktop
    /// that owns the folder takes it out. The draw lane drains it each frame,
    /// so a change made by a press is on the disk before the next one.
    ///
    /// [`None`] is *nothing to write*, which is the ordinary state. It is not
    /// the same as `Some(vec![])` — that is **a person who emptied their Dock**
    /// and is the one write that must not be skipped on the grounds that there
    /// is nothing in it.
    pub(crate) pins_owed_to_the_disk: Option<Vec<alo_dock::ADockEntry>>,
    /// Which part of the screen the keyboard is in.
    ///
    /// [`None`] is the canvas, which is where it is when the shell has not
    /// taken it — see `crate::which_area_has_the_keyboard` for why the canvas
    /// is an absence rather than a variant.
    pub(crate) which_area_has_the_keyboard: crate::which_area_has_the_keyboard::TheFocusedArea,
    /// Which slot on the bar the keyboard is on, while the Dock is the focused
    /// area.
    ///
    /// **Separate from the surfaces' own focus**, which lives inside
    /// `WhatTheDockWasAskedFor`. A person tabbing along the bar has focused a
    /// slot and opened nothing; a person in *More apps* has a surface open and
    /// a cell focused in it. One field for both would make *which thing does
    /// Enter act on* a question with two answers.
    pub(crate) which_slot_on_the_bar: Option<usize>,
    /// Which row of the alo Bar the keyboard is on, if any.
    ///
    /// **Not the same thing as `which_slot_on_the_bar`**, which is a control on
    /// the Dock's own bar. This is a row in the panel the alo Bar opens, and
    /// the two are never both meaningful: the Dock's controls are walked with
    /// Tab, the Bar's rows with the arrows.
    ///
    /// [`None`] is somebody typing rather than choosing, which is every frame
    /// until they press an arrow.
    pub(crate) which_row_in_the_bar: Option<usize>,
    /// What the Settings entry is called, in this person's language.
    ///
    /// **Kept because a search has to match what is drawn.** A native entry has
    /// no identifier — the owner's ruling forbids inventing one — so its label
    /// is the only string it can be found by, and the label is translated. The
    /// `Server` has no vocabulary, so the one word it needs is handed to it at
    /// sign-in and kept: it does not change for the life of a session.
    ///
    /// **Empty until it is told.** A session that never says has a Settings
    /// entry that no search matches, which is wrong and visible rather than
    /// wrong and silent — the alternative, falling back to some identifier,
    /// would be the invented name again.
    pub(crate) the_settings_entry_is_called: String,
    /// What the Applications entry is called, in this person's language.
    ///
    /// **A second field rather than one, because there are two native entries
    /// now and one string cannot be both.** Until 2026-10-17
    /// `Server::what_it_is_called` answered *the Settings label* for any entry
    /// with no application, which was true while Settings was the only one — and
    /// the moment the market joined the fresh Dock, the market was **drawn and
    /// searched under Settings' name**. A test looking for the Polish
    /// *Ustawienia* found two rows, which is how it was caught.
    ///
    /// The same family as every *replaced rule with callers left behind*:
    /// `app().is_none()` used to mean Settings and does not any more.
    ///
    /// **Empty until it is told**, for the field above's reason exactly.
    pub(crate) the_applications_entry_is_called: String,
    /// The task windows this person has open.
    ///
    /// **Empty until somebody asks for something**, which is the honest state
    /// of a machine nobody has spoken to.
    pub(crate) the_tasks: crate::the_tasks_a_person_opened::TheTasks,
    /// The change waiting for this person to answer, and which answer their
    /// keyboard is on.
    ///
    /// **One per machine and not one per task**, for the reason the composer
    /// below gives about the keyboard: there is one person reading, so there
    /// is one question, whichever task proposed it. Settled by the owner on
    /// 2026-10-26.
    ///
    /// `alo_tasks::permissions` is why it is not a queue — *a list of pending
    /// approvals is a list a person clears by clicking, and clicking through
    /// is how somebody approves the fourth one without reading it.* Nothing is
    /// lost: the daemon holds the rest, and answering one brings up the next.
    pub(crate) the_change_waiting: crate::a_change_waiting_reaches_the_person::AChangeWaiting,
    /// Which task window holds the keyboard, if one does.
    ///
    /// **The one a person just asked for, never one a question arrived in** —
    /// `crate::the_tasks_seat` has the reason, and it is a safety reason: a
    /// proposal that could take the keyboard could be approved by somebody
    /// typing *Tab* and *Enter* into a document.
    pub(crate) the_task_with_the_keyboard: Option<crate::the_tasks_a_person_opened::WhichTask>,
    /// The Dock's composer, as a field a person types a request into.
    ///
    /// **One per machine and not one per display**, because there is one
    /// keyboard: a person typing a request is typing it once, and two composers
    /// holding half a sentence each is a bug nobody could describe.
    pub(crate) the_composer: crate::the_composer_a_person_types_in::TheComposer,
    /// What a person asked alo for, waiting to become a task.
    ///
    /// **Recorded here rather than acted on**, exactly as `asked_to_start` is:
    /// building a task needs to know which model answers on this machine, and
    /// that is a reading of a person's settings. This crate shows and never
    /// measures.
    pub(crate) asked_alo: Vec<String>,
    /// Whether each window's external edge is revealed, one machine per window.
    ///
    /// §5 of `docs/design/the-external-window-edge.md`. The edge is **per
    /// window**, so the state is too: a pointer resting on one window's edge
    /// says nothing about another's, and a single `Revealing` here would reveal
    /// every edge on the screen at once.
    ///
    /// **A list and not a map**, keyed by the surface itself. `WlSurface` is not
    /// `Ord`, the count is the number of windows a person has open, and a list
    /// is what `asked_to_put_aside` above already is. `crate::the_window_edge_reveals`
    /// is the only thing that reads or writes it, and it prunes the surfaces
    /// that are no longer mapped on the way through — so a window that closes
    /// takes its reveal state with it rather than leaving one for whatever
    /// surface is allocated at that address next.
    pub(crate) edges_revealed: Vec<(WlSurface, alo_dock::revealing::Revealing)>,
    /// Where the keyboard is on each window's edge.
    ///
    /// §6 of `docs/design/the-external-window-edge.md`, and the sibling of
    /// `edges_revealed` above in every respect: per window for that field's
    /// reason, a list for that field's reason, and pruned on the same walk of
    /// the mapped surfaces. `crate::the_edge_takes_the_keyboard` is the only
    /// thing that reads or writes it.
    ///
    /// **Only windows the keyboard has reached are in it.** A missing entry
    /// already means `Nowhere`, so storing that would grow the list for every
    /// window a key was pressed near and change no answer.
    pub(crate) keyboard_on_edges: Vec<(WlSurface, crate::window_edge_keys::TheKeyboardOnTheEdge)>,
    /// Where the **person** clicked the panel, not yet acted on.
    ///
    /// The point of the press, rather than the pointer read again later. A peek asks where the
    /// pointer *is*; a click asks where it **was when the button went down** — re-reading the
    /// live pointer when the ask is met would act on whichever preview the pointer had moved
    /// to, which is a different window from the one the person chose.
    ///
    /// Recorded here and met in `crate::direct_desktop` for the same reason as
    /// `asked_to_put_aside`: deciding *which* preview needs the live `Panel` for the identity
    /// check, and input has no desktop in scope.
    pub(crate) asked_to_bring_back: Vec<Point<i32, Physical>>,
    /// Buttons whose press the panel took, so their release is taken too.
    ///
    /// **A client must never see a release for a press it did not see.** The press is swallowed
    /// where the panel claims it, so the matching release has to be swallowed as well or the
    /// focused client receives an unpaired release and believes a button it never saw held has
    /// gone up.
    pub(crate) clicks_the_panel_took: std::collections::HashSet<u32>,
    /// **What this person's chords mean**, told once as the session stands up.
    ///
    /// Held here for the reason `asked_to_put_aside` gives in the other
    /// direction: a key arrives at the seat, where no desktop is in scope, and
    /// what a chord means is the person's own settings, which belong to
    /// whoever implements `crate::TheDesktop`. `crate::a_chord_reaches_its_action`
    /// carries the argument.
    ///
    /// `None` until a desktop says otherwise, which is what keeps the sign-in
    /// screen, the nested lane and every test on the road they already had: a
    /// seat that has been told no shortcuts looks none up.
    pub(crate) shortcuts: Option<alo_shortcuts::Shortcuts>,
    /// **The settings window this session owns**, shut until a chord opens it.
    ///
    /// Held here for the reason the shortcuts above are, one step further on: a
    /// chord arrives at the seat, and until this field existed there was nowhere
    /// on a running machine for the window it names to live.
    /// `crate::settings_command::Server::dispatch_settings_command` took one as
    /// a `&mut` argument and **nothing in production had one to pass** — the
    /// road was built and the destination was never constructed.
    /// `the-shell-plan.md` task 18 is that gap.
    ///
    /// **Not an `Option`**, because *shut* is a state a `SettingsWindow` already
    /// has and a second spelling of it would be a branch no caller can act on
    /// differently — the same argument `remembered` below makes for itself.
    pub(crate) settings: crate::SettingsWindow,
    /// **Where this person's settings are kept**, told by the desktop.
    ///
    /// `None` until a desktop says, which keeps the sign-in screen, the nested
    /// lane and every test on the road they already had.
    ///
    /// **Told rather than read**, which is this file's own rule for the
    /// shortcuts and holds here for the same reason:
    /// `crate::a_chord_reaches_its_action` says *a compositor that opened a
    /// person's settings file would be a compositor measuring*. So the folder,
    /// the grants and the pairings arrive from `crate::TheDesktop`, and this
    /// crate opens none of them to find out where they are.
    pub(crate) settings_places: Option<crate::SettingsPlaces>,
    /// **The record window**, which a chord or a row in Settings opens.
    ///
    /// Not an `Option` for `settings`' reason: *shut* is a state the window
    /// already has.
    pub(crate) record: crate::RecordWindow,
    /// **Where this machine keeps its record**, told by the desktop, or why it
    /// could not say.
    ///
    /// `None` until a desktop says, which keeps every other road as it was.
    /// Told rather than read for `settings_places`' reason: the description
    /// that names the record's path is a file, and this crate opens none.
    pub(crate) recounting: Option<Result<alo_recounting::Recounting, alo_recounting::NotRecounted>>,
    /// **How this person writes a time and a day**, told by the desktop at
    /// sign-in and again by Settings when they choose a region.
    ///
    /// Until a desktop says, the rule `crate::TheirFormats` holds for a
    /// person who has chosen nothing and named no language — never one region
    /// for everybody. See `crate::the_formats_are_told`.
    pub(crate) formats: crate::TheirFormats,
    /// Whether the formats changed since the desktop was last told.
    pub(crate) formats_untold: bool,
    /// **What this person's Places remember**, as the last keep left it.
    ///
    /// Held here for the same reason the shortcuts above are: a chord is
    /// answered at the seat, where no desktop is in scope, and the file this
    /// came out of is the desktop's to read. `crate::canvas_a_place_remembers_time`
    /// carries the argument.
    ///
    /// **A fresh arrangement rather than an `Option`**, because *nothing is
    /// remembered yet* and *this Place has no earlier state* are the same
    /// answer to the only question asked of it, and a second spelling of it
    /// would be a branch no caller can act on differently.
    pub(crate) remembered: alo_arranging::Arrangement,
    /// **The moment the Place in front is showing**, while somebody is walking
    /// backwards through what it remembers.
    ///
    /// `None` when nobody is walking, which is every moment until the first
    /// press and again after the next rearrangement.
    /// `crate::canvas_a_place_remembers_time` holds the argument for why a
    /// cursor is needed at all — without one the chord is a toggle between two
    /// canvases rather than a walk.
    pub(crate) walking_back_from: Option<std::time::SystemTime>,
    /// **What a touchpad gesture means, as `alo-desktops` recognises it.**
    ///
    /// One recogniser for the seat: at most one gesture is in flight at a
    /// time, and a second would be two answers to one person's fingers. What a
    /// swipe does with what it answers is `crate::desktop_swipes`.
    pub(crate) gestures: alo_desktops::gestures::Gestures,
    /// **The window of what a person can install**, shut until they open it.
    ///
    /// Held here for [`Self::settings`]' reason exactly: a press arrives at the
    /// Dock, where no desktop is in scope, and until this field existed there
    /// was nowhere on a running machine for the window to live. `MarketWindow`
    /// had **zero production mentions** when it was written and when its raster
    /// landed — the model was built, the drawing was built, and no person could
    /// reach either.
    ///
    /// **Not an `Option`**, for the same reason Settings is not: *shut* is a
    /// state a `MarketWindow` already has, and a second spelling of it would be
    /// a branch no caller can act on differently.
    pub(crate) market: crate::MarketWindow,
    /// **What this machine already has installed**, as whoever holds the
    /// machine last told this seat.
    ///
    /// Told rather than read, which is [`Self::settings_places`]' own rule and
    /// holds here for a harder reason: reading it means running the sandbox's
    /// tool and waiting for it, and this is the compositor's input loop. A
    /// press that waited on a subprocess would stop every window on the machine
    /// drawing — the shape of fault the terminal's own `Drop` was found to have
    /// on 2026-10-16. `alo_software::what_this_machine_has` is the reader, and
    /// it belongs in the process that can afford to block.
    ///
    /// **Empty until something says otherwise, and what that costs is bounded.**
    /// `alo_software::Catalogue::decided` already refuses anything in
    /// `shipped.toml`, so an untold seat cannot offer a person the seven
    /// applications their machine came with. What it can do is offer *Install*
    /// for something installed from the market on an earlier boot. That is
    /// wrong, and it is visibly wrong rather than quietly wrong: the install
    /// itself refuses, and the window says so.
    pub(crate) what_this_machine_has: Vec<String>,
    /// **What a person pressed Install on**, waiting for the lane to start it.
    ///
    /// Held here for [`Self::asked_alo`]'s reason exactly: the press arrives at
    /// the seat, the work is somebody else's, and until this field existed
    /// `MarketPressed::AskedToInstall` was handed back to a caller that dropped
    /// it — which `crate::a_key_reaches_the_market`'s header says out loud.
    ///
    /// **The press is the event, never the window's state.**
    /// `MarketWindow::what_they_asked_for` stays answered until the host says
    /// what became of it, so a lane that read *that* would start the same
    /// installation on every frame until it finished. One press is one
    /// installation, which is the capability model's *one approval causes
    /// exactly one execution* applied to a person's own keypress.
    ///
    /// **One at a time**, because the window refuses a second ask while one is
    /// going — so this is an `Option` and not a queue, and a second press
    /// replacing an unstarted first cannot happen.
    pub(crate) asked_to_install: Option<alo_software::Listed>,
    /// **How many input events this seat could not make sense of.**
    ///
    /// Kept because a lane that survives a refusal must be able to say it
    /// happened: `InputError::is_about_one_event` is what decides a refusal is
    /// survivable, and its own note is that swallowing one silently is worse
    /// than ending loudly — a pointer that does nothing for ever is what hid
    /// the missing `enable_pointer` call until a machine was booted.
    ///
    /// **A count and not the events**, because the events are a person's
    /// keystrokes and pointer and this crate keeps none of them. What a reader
    /// needs is *how many times this machine did not understand its own
    /// input*, which a number answers and a log of keys would not.
    ///
    /// Never reset. A session that refused a thousand events in its first
    /// minute and none since is a different machine from one refusing them
    /// now, and nothing here can tell those apart — so it counts, and whoever
    /// reads it reads it beside the moment it is read.
    pub(crate) input_it_could_not_make_sense_of: usize,
    /// **How many key presses this seat routed somewhere.**
    ///
    /// The control the other two are read against:
    /// `crate::what_the_seat_made_of_its_input` carries the argument, which is
    /// that *buttons: 0* means one thing beside keys that are standing still
    /// and the opposite beside keys that are climbing.
    ///
    /// A count and never reset, for the two reasons the field above gives.
    pub(crate) key_presses_this_seat_routed: usize,
    /// **How many pointer button presses this seat routed somewhere.**
    ///
    /// The number that was missing on 2026-10-18, when a press landing on a
    /// Dock entry opened nothing and no reading on the machine could say
    /// whether the press had arrived at all.
    ///
    /// A count and never reset, for the two reasons the refusal field gives.
    pub(crate) button_presses_this_seat_routed: usize,
    /// **How many presses landed on something this compositor draws itself.**
    ///
    /// Beside the field above and deliberately separate from it: one says a
    /// press reached this machine, the other says it reached something on it.
    /// A machine that could only report the first spent 2026-10-18 unable to
    /// say which of two faults it had.
    pub(crate) presses_that_landed_on_the_furniture: usize,
    /// **How many windows this person has open that the last frame did not
    /// draw.**
    ///
    /// Nought on every ordinary frame. Anything else is a control that did
    /// something and showed nothing — which a person meets as a press that did
    /// not work, and which looks the same on a screen capture as a press that
    /// never arrived.
    ///
    /// Set by the draw rather than counted here, because only the draw knows
    /// both halves: whether the person has it open, and whether this frame
    /// made a picture of it.
    pub(crate) windows_opened_and_not_drawn: usize,
}

impl Server {
    /// Flush queued protocol events without accepting or dispatching client work.
    pub(crate) fn flush(&mut self) -> io::Result<()> {
        self.display.flush_clients()
    }

    /// Handle kept private to the library's protocol-global initialization.
    pub(crate) fn display_handle(&self) -> smithay::reexports::wayland_server::DisplayHandle {
        self.display.handle()
    }
    /// Bind `runtime/name/wayland`, refusing existing sessions and unsafe paths.
    pub fn bind(runtime: &Path, name: &str) -> Result<Self, SocketError> {
        let display = Display::new().map_err(io::Error::other)?;
        let surfaces = Surfaces::new(&display.handle());
        let socket = Socket::bind(runtime, name)?;
        Ok(Self {
            control_overlay: Default::default(),
            control_press: None,
            control_presentation: None,
            reader_presentation: None,
            display,
            surfaces,
            socket,
            display_numbers: std::collections::BTreeMap::new(),
            // No desktop has arranged anything yet.
            screens: None,
            presentations: std::collections::BTreeMap::new(),
            switch_order: Default::default(),
            fixed_controls: std::collections::BTreeMap::new(),
            // Nothing has been drawn yet, which is why this is `None` rather than an
            // empty picture — see the field's own note.
            panel_as_drawn: None,
            dock_as_drawn: std::collections::BTreeMap::new(),
            the_docks_surface_as_drawn: std::collections::BTreeMap::new(),
            the_dock_was_asked_for: crate::what_the_dock_has_open::WhatTheDockWasAskedFor::Nothing,
            // Nobody has asked for anything yet.
            asked_to_put_aside: Vec::new(),
            asked_on_an_edge: Vec::new(),
            asked_to_start: Vec::new(),
            the_terminal: None,
            alos_own_terminal: None,
            // Nobody has said whose desktop this is; the stand-up does.
            whose_desktop: None,
            asks_an_application_to_start: None,
            // Nothing has been worked out yet; the stand-up fills it.
            what_the_dock_is_pinned_to: alo_dock::Holding::nothing(),
            // Nobody is adding a provider until they ask to.
            filling_in: None,
            // Nothing has been pasted, and nothing is being waited for.
            clipboard: crate::what_is_on_the_clipboard::TheClipboard::new(),
            // Nobody has asked for a provider to be saved.
            asked_to_save_a_provider: None,
            // Nothing has failed to save, which is not the same as having saved.
            a_provider_was_not_saved: None,
            what_each_application_draws: std::sync::Arc::new(alo_dock::artwork::TheArtwork::none()),
            what_was_typed_into_more_apps: alo_dock::WhatTheyTyped::nothing(),
            pins_owed_to_the_disk: None,
            // The keyboard starts with the work, which is where a person's
            // hands already are.
            which_area_has_the_keyboard: None,
            which_row_in_the_bar: None,
            which_slot_on_the_bar: None,
            the_settings_entry_is_called: String::new(),
            the_applications_entry_is_called: String::new(),
            the_tasks: crate::the_tasks_a_person_opened::TheTasks::none_yet(),
            the_task_with_the_keyboard: None,
            the_change_waiting:
                crate::a_change_waiting_reaches_the_person::AChangeWaiting::nothing_yet(),
            the_composer: crate::the_composer_a_person_types_in::TheComposer::empty(),
            asked_alo: Vec::new(),
            edges_revealed: Vec::new(),
            keyboard_on_edges: Vec::new(),
            asked_to_bring_back: Vec::new(),
            clicks_the_panel_took: std::collections::HashSet::new(),
            gestures: Default::default(),
            // Nobody has told this seat what a chord means, so it takes none.
            shortcuts: None,
            // Shut, and a person's chord is what opens it.
            settings: crate::SettingsWindow::closed(),
            // Nobody has told this seat where a person's settings are kept.
            settings_places: None,
            // Shut, and a chord or a row in Settings is what opens it.
            record: crate::RecordWindow::on_an_output(),
            // Nobody has told this seat where the record is kept.
            recounting: None,
            // Nobody has told this seat how this person writes a time.
            formats: crate::TheirFormats::of(alo_formats::Chosen::default(), None),
            formats_untold: false,
            // Shut, and a press on the person's own Dock is what opens it.
            market: crate::MarketWindow::closed(),
            // Nobody has told this seat what the machine already has.
            what_this_machine_has: Vec::new(),
            // Nobody has asked for anything to be installed.
            asked_to_install: None,
            // Nothing has been refused, because nothing has arrived.
            input_it_could_not_make_sense_of: 0,
            // And nothing has been routed either, for the same reason.
            key_presses_this_seat_routed: 0,
            button_presses_this_seat_routed: 0,
            presses_that_landed_on_the_furniture: 0,
            windows_opened_and_not_drawn: 0,
            // Nothing has been kept yet, so no Place remembers anything.
            remembered: alo_arranging::Arrangement::fresh(),
            // Nobody is walking backwards through anything yet.
            walking_back_from: None,
            desk: crate::server_desk::Desk::new(),
        })
    }

    /// Absolute socket path; pass only to applications belonging to this session.
    pub fn socket_path(&self) -> &Path {
        &self.socket.path
    }

    /// Accept at most 16 clients, dispatch pending requests and flush responses.
    ///
    /// Never blocks waiting for a client. Protocol errors disconnect the offending
    /// client; display I/O failures are returned to the backend. The accept budget
    /// ensures a connection flood cannot make acceptance itself an endless loop.
    pub fn dispatch(&mut self) -> io::Result<()> {
        if let Some(listener) = &self.socket.listener {
            for _ in 0..16 {
                let Some(stream) = listener.accept()? else {
                    break;
                };
                self.display
                    .handle()
                    .insert_client(stream, Arc::new(ClientState::default()))?;
            }
        }
        self.display.dispatch_clients(&mut self.surfaces)?;
        self.surfaces.prune();
        // The desktops and the divisions follow the windows: one that opened
        // joins the desktop being looked at, and one that closed loses its
        // share rather than leaving a tree holding a window nobody can see.
        self.the_windows_are_now_these();
        self.switch_order.refresh(self.surfaces.buffered());
        self.display.flush_clients()
    }

    /// Visible, configured toplevel roots with buffers, in front-to-back order.
    /// New roles initially follow existing roles; explicit raising changes order.
    ///
    /// This is an internal renderer input, not an agent context or window API.
    /// Minimization, buffer removal, destruction and disconnect exclude roots here.
    pub fn mapped_surfaces(&self) -> impl Iterator<Item = &WlSurface> {
        self.surfaces.mapped()
    }

    /// Where each mapped window is on this display, in its physical pixels.
    ///
    /// For the one question the dock asks of the windows — whether any needs
    /// the room it is sitting in. [`crate::dock_room`] holds the rule and
    /// `docs/design/when-the-dock-gives-way.md` settles what *a window needs
    /// the room* has to mean.
    ///
    /// **One is the scale this display is laid out at**, the same one
    /// `crate::desktop_raster` multiplies a division by. The conversion is
    /// here rather than inside the comparison so there is one place to change
    /// when a display is laid out at another scale.
    ///
    /// A minimised window is not among these, because `mapped` excludes it: a
    /// window a person put aside is not a window needing the room.
    pub(crate) fn window_areas(&self) -> Vec<Rectangle<i32, Physical>> {
        self.mapped_surfaces()
            .map(|surface| {
                let origin = crate::window_placement::window_buffer_origin(surface);
                let geometry = crate::scene::geometry(surface);
                Rectangle::new(
                    (origin + geometry.loc).to_physical(1.0).to_i32_round(),
                    geometry.size.to_physical(1.0).to_i32_round(),
                )
            })
            .collect()
    }

    /// Number of live toplevel roles, including those not yet mapped.
    pub fn toplevel_count(&self) -> usize {
        self.surfaces.count()
    }

    /// Draw and submit mapped surface trees, then notify only submitted surfaces.
    ///
    /// The target is trusted compositor plumbing, never supplied by an agent.
    /// Dispatch is not interleaved with submission. Failure preserves all pending
    /// frame callbacks. `time` is milliseconds on the session's monotonic clock,
    /// wrapping at 32 bits as required by Wayland. Success means submission to
    /// the backend, not physical presentation or a presentation-time guarantee.
    pub fn render(
        &mut self,
        target: &mut impl crate::FrameTarget,
        time: u32,
    ) -> Result<usize, crate::RenderError> {
        self.render_window_controls(target, None, time)
    }

    /// Shared submission path; callers own native presentation retirement.
    pub(crate) fn render_frame<T: crate::FrameTarget + ?Sized>(
        &mut self,
        target: &mut T,
        time: u32,
    ) -> Result<usize, crate::RenderError> {
        let size = target.size();
        // **The display this frame is for, before anything is validated
        // against it.** A name is how two displays are told apart, so the
        // presentation has to be chosen before its identity check can mean
        // *this display changed* rather than *this is a different display*.
        let named = target.metadata()?;
        named.validate()?;
        let metadata = self
            .presentations
            .entry(named.name.clone())
            .or_default()
            .validate_target(target)?;
        if size.w > 0 && size.h > 0 {
            // Reactive popup negotiation follows the backend's desired extent,
            // even on submission refusal. wl_output describes only submitted modes.
            self.surfaces.popups.output_size = Some(size);
            // **And where every display is**, so a popup is constrained to
            // the screen its parent is on rather than to whichever display
            // drew last — `more-than-one-display-plan.md` task 6.
            //
            // Written once a frame beside the extent above, and from the
            // arrangement rather than from this draw: the extent is what
            // *this* display is, and the question a popup asks is about all
            // of them. A session with no arrangement answers with this one
            // display at the desk's origin, which is exactly what the single
            // extent meant before this existed.
            // Each display's rectangle **and the camera it is looking
            // through** — task 7. The two are written together because a
            // popup needs both and taking them from different displays would
            // constrain a menu to one screen at another's zoom.
            self.surfaces.popups.screens = match self.the_screens() {
                None => vec![crate::popups::ScreenView {
                    rect: smithay::utils::Rectangle::new((0, 0).into(), size),
                    named: String::new(),
                }],
                Some(screens) => screens
                    .each()
                    .map(|place| {
                        let (across, along) = place.room().across_and_along();
                        crate::popups::ScreenView {
                            rect: smithay::utils::Rectangle::new(
                                (place.at().across(), place.at().down()).into(),
                                (across, along).into(),
                            ),
                            named: place.name().name().to_owned(),
                        }
                    })
                    .collect(),
            };
            // **The camera used to be assigned beside the extent and no longer is.**
            // `Popups` held a copy of it, and this line is where the copy was made
            // fresh once a frame — which is what kept it from ever being stale.
            // `crate::Surfaces::camera` is the one home now, and the three places
            // that place a popup take it as an argument, so there is nothing left
            // here to keep in step.
        }
        // The one reader of the camera that draws with it, and the reason the
        // backend is told rather than asked: the first version of this seam
        // shipped a backend holding a camera nobody assigned, drawing a zoom it had
        // never been told about.
        // **This display's own camera.** `more-than-one-display-plan.md`
        // task 7: every display was handed the session's one camera, so two
        // displays were two views of the same thing — mirroring, whatever
        // else had been made per display. A display that has never been
        // panned or zoomed looks at the origin at life size, which is what
        // all of them looked at before any had a camera of its own.
        // **Which camera is a question about whether there is an
        // arrangement.** With one, each display has its own and is drawn
        // through it. Without one there is a single session camera, held
        // under the one key its writer uses — `the_display_being_worked_on`
        // answers the empty name when no display has been placed, because
        // there is no display to name and exactly one to mean.
        //
        // Reading this under the display's own name instead cost a test:
        // the pan was written under the empty key and read under
        // `alo-drm-1`, so *the session panned and the display was never
        // told* — which is the sentence
        // `one_plane_under_one_viewport::a_panned_session_hands_its_camera_to_the_display`
        // had been carrying since before any of this.
        let looking = if self.screens.is_some() {
            metadata.name.as_str()
        } else {
            ""
        };
        target.look_at(self.surfaces.camera_of(looking))?;
        self.surfaces.prune();
        let roots: Vec<_> = self.mapped_surfaces().cloned().collect();
        let cursor = self.cursor();
        let popups = self.popup_surfaces();
        let handle = self.display.handle();
        let submitted = self
            .presentations
            .entry(metadata.name.clone())
            .or_default()
            .render(&handle, target, (&roots, &popups), &cursor, time)?;
        self.surfaces.update_window_mode_output(Some(size));
        // The output this frame went to is the display a session holds its
        // divisions and desktops on; see `crate::display_lifecycle`.
        self.the_display_submitted(&metadata, (size.w, size.h));
        Ok(submitted)
    }
}

impl Server {
    /// Advertise buffer sharing, now that a renderer exists to import with.
    ///
    /// A pass-through, because the ordering is the whole content: the caller is
    /// the graphics backend's setup, which has a renderer and nothing else to
    /// do with wayland globals. See
    /// [`crate::surfaces::Surfaces::advertise_importable_buffers`].
    pub(crate) fn advertise_importable_buffers(
        &mut self,
        formats: smithay::backend::allocator::format::FormatSet,
    ) {
        self.surfaces.advertise_importable_buffers(formats);
    }

    /// Take every buffer a client offered and nobody has tried yet.
    ///
    /// Called once per turn of `crate::direct_loop::run_with_input`, between
    /// dispatching clients and drawing. Leaving one here would leave a client
    /// waiting for an event with no sender.
    pub(crate) fn buffers_awaiting_import(
        &mut self,
    ) -> Vec<crate::buffers_clients_hand_over::HandedOver> {
        self.surfaces.buffers_awaiting_import()
    }
}
