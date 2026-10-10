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
            // Nobody has asked for anything yet.
            asked_to_put_aside: Vec::new(),
            edges_revealed: Vec::new(),
            asked_to_bring_back: Vec::new(),
            clicks_the_panel_took: std::collections::HashSet::new(),
            gestures: Default::default(),
            // Nobody has told this seat what a chord means, so it takes none.
            shortcuts: None,
            // Shut, and a person's chord is what opens it.
            settings: crate::SettingsWindow::closed(),
            // Nobody has told this seat where a person's settings are kept.
            settings_places: None,
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
        // **No native scene of its own.** This passed `None` for the window
        // control strip until the strip was retired on 2026-10-10; the
        // external window edge reaches a frame through `NativeLayers::edges`
        // and the direct backend builds it, so an ordinary render has nothing
        // of the shell's to select.
        self.render_frame(target, time)
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
