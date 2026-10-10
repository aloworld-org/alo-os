//! The desktop on this machine's own display: the dock, the windows, and what
//! is leaving, above whatever clients are mapped.
//!
//! The nested twin is `crate::nested_desktop`, which draws the same desktop
//! inside somebody else's compositor. What differs is where the pixels go and
//! nothing else: the dock, the status area, the two windows and the egress
//! indicator are the same pictures, made by the same `frame_pictures` from the
//! same `DesktopFrame`.
//!
//! # A client's frame is still the server's own
//!
//! This does not submit a desktop *instead of* the clients. It goes through
//! `Server::render_frame` exactly as an ordinary frame does — so the output is
//! published, membership is kept and every client that was drawn gets its frame
//! callback — and hands the desktop's layers to the backend on the way past.
//! A desktop that bypassed that would be a compositor whose clients slowly
//! stopped drawing, for a reason nobody would find.

use crate::scene_native::NativeLayers;
use crate::what_the_server_tells_a_frame::what_only_the_server_knows;
use crate::{
    Cursor, DesktopFrame, DirectLoopError, FrameTarget, Popup, RenderError, Server, SessionError,
    WindowControlLabels, direct_input_loop::LoopInput,
};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

/// What is on this display now, asked once a frame.
///
/// A trait rather than a struct of nine fields because **nothing here owns any
/// of it**: the dock is `alo-dock`'s, the appearance `alo-appearance`'s, the
/// readings `alo-measuring`'s and the division `alo-dividing`'s. The compositor
/// is handed what they say and draws it, and a type that held them would be a
/// compositor keeping its own copy of somebody else's answer.
pub trait TheDesktop {
    /// Everything one frame of the ordinary desktop needs.
    fn now(&self) -> DesktopFrame<'_>;

    /// Read anything that changes, before the next frame is drawn.
    ///
    /// **A status area whose clock never moves looks broken rather than
    /// unfinished**, and it is the one surface a person checks to find out
    /// whether their machine is telling them the truth. So the lane asks before
    /// every frame, and whatever is on the other side decides how often that is
    /// worth acting on — this crate does not know what a reading costs and must
    /// not decide how often one is taken.
    ///
    /// Nothing by default: a desktop of fixed values is a real one, and the
    /// display probe is exactly that.
    fn refreshed(&mut self) {}

    /// Where the pointer is, as far as the put-aside panel is concerned.
    ///
    /// **The classification, never the coordinate.** This crate laid the panel out, so this
    /// crate knows which rectangle a pointer is in; the crate on the other side holds the
    /// `Panel` and calls the rule about what a peek may change.
    /// `alo_put_aside::the_region_the_panel_claims` asks for exactly that division in its own
    /// words: *coordinate classification belongs to the caller; the machine consumes that
    /// classification.* So the compositor never learns what a peek looks like, and the desktop
    /// never learns where the slots are.
    ///
    /// **Nothing by default**, the same as [`Self::refreshed`] and for the same reason: a
    /// desktop with no panel is a real desktop, and a default that did something would make
    /// every new implementor responsible for a surface it may not have.
    ///
    /// **What it costs, since the answer belongs on the other side.** Called **once per input
    /// batch**, after the seat has settled, from wherever the pointer ended up — not once per
    /// motion event. A person crossing the panel produces many events and gets one answer,
    /// because the extra answers would all say the same thing.
    ///
    /// It is still called when nothing has changed, and deliberately: `refreshed`'s own note
    /// says *this crate does not know what a reading costs and must not decide how often one is
    /// taken*, and the same holds here. An implementor that finds reacting expensive is the one
    /// that can compare this classification against the last and return early. The compositor
    /// cannot, because it does not know what a repeat means to the surface.
    fn the_pointer_is_now(&mut self, _on: crate::which_preview_the_pointer_is_on::OnThePanel) {}

    /// The panel this desktop keeps, for the one act that has to change it.
    ///
    /// **Lent mutably, and only here.** Every other seam in this trait hands the compositor
    /// a borrow to draw from or a classification to consume; putting a window aside is the
    /// one thing that alters what the desktop holds, and it cannot be done through
    /// `now()` because that lends the panel immutably as part of a frame.
    ///
    /// **`None` by default**, the same as the two above and for the same reason: a desktop
    /// with no panel is a real desktop — the display probe is one — and a default that
    /// answered `Some` would make every implementor responsible for a surface it may not
    /// have. A desktop that answers `None` simply never puts a window aside.
    fn the_panel(&mut self) -> Option<&mut alo_put_aside::Panel> {
        None
    }
    /// What the pointer is to the put-aside panel's own edge and surface.
    ///
    /// **A classification, never a coordinate**, for the reason
    /// [`Self::the_pointer_is_now`] gives: this crate laid the panel out, so it knows which
    /// rectangle a point is in; the crate on the other side holds the state machine that
    /// decides what being there means.
    ///
    /// `alo_dock::Revealing` is that machine and it had **no caller at all** until this
    /// existed — the panel was simply always drawn. Advancing it is the desktop's, because a
    /// `Revealing` is session state and the compositor holds none.
    ///
    /// **Nothing by default**, as with the two above: a desktop with no panel has no panel to
    /// reveal.
    fn the_panel_is_revealed(&mut self, _by: alo_dock::revealing::ThePointer) {}

    /// What this person's chords mean, asked for once as the session starts.
    ///
    /// **The shell asks and does not read**, for the reason
    /// [`Self::the_layout_they_left`] gives at length: a settings file in a
    /// person's folder is a call into the machine, and shell task 15's
    /// constraint is that none is added from `crates/alo-shell`.
    ///
    /// **The default is what this release ships**, which is the honest default
    /// here and not an empty one: a desktop that was never told about a
    /// person's changes still has every shipped chord, and a desktop that
    /// answered `Shortcuts::over(Defaults::none())` would be a machine where
    /// `⊞`+0 does nothing until somebody edits a file.
    ///
    /// Asked **once**, as the layout is: a binding a person changes in Settings
    /// is told to the session through `Server::the_shortcuts_are`, which is the
    /// road that exists for it, rather than by re-reading a file every frame.
    fn the_shortcuts(&mut self) -> alo_shortcuts::Shortcuts {
        alo_shortcuts::Shortcuts::shipped()
    }

    /// **What this person has settled about their displays** — their
    /// appearance, their night light and the arrangements they have kept.
    ///
    /// # Why this asks for settings and not for an arrangement
    ///
    /// Which displays exist, how many pixels each has and how big its glass is
    /// are facts the **compositor** learns, and nothing else can know them.
    /// Where those displays sit relative to one another is a **decision**
    /// `alo-displays` makes — and making it needs this person's kept
    /// `Changes`, an `Appearance` and a `Tonight`, none of which the shell may
    /// read, because *the shell shows and never measures* and a file in a
    /// person's folder is a reading like any other.
    ///
    /// So **the desktop hands over the three readings and the shell builds the
    /// arrangement**, through [`crate::TheirDisplays::the_screens_of`].
    ///
    /// # This replaced `the_screens_of`, which could not be implemented
    ///
    /// Until 2026-10-08 this method took a `Vec<alo_displays::Reported>` and
    /// answered with a [`crate::Screens`] — and **`alo-desktop` has no
    /// `alo-displays` dependency**, so the only crate obliged to implement it
    /// could name neither the argument nor the answer. Every machine took the
    /// default, `Server::the_screens()` was [`None`] everywhere, and five
    /// tasks of `more-than-one-display-plan.md` were unreachable in production
    /// while reading as done. `more-than-one-display-plan.md` task 10.
    ///
    /// Everything this needs is re-exported from this crate, which
    /// `alo-desktop` already depends on, so answering costs it no new edge.
    ///
    /// **[`None`] by default**, which is honest for a desktop that was never
    /// given a display model: every surface keeps laying out against the one
    /// viewport it already uses, exactly as before. A desktop that answers
    /// `None` is not broken; it is a desktop with nothing of the person's to
    /// offer.
    fn their_displays(&mut self) -> Option<crate::TheirDisplays> {
        None
    }

    /// Where this person's settings are kept, asked for once as the session
    /// starts.
    ///
    /// **`None` by default, and that is the whole of what keeps every other
    /// desktop on the road it had.** The sign-in screen, the nested lane and
    /// every test implement this trait and none of them has a person's folder;
    /// a desktop that answers `None` opens no Settings and its chords fall
    /// through to the canvas exactly as before.
    ///
    /// Asked once, as the shortcuts and the layout are, and told to the seat
    /// through [`Server::the_settings_places_are`](crate::Server::the_settings_places_are).
    /// **The shell asks and does not read**: where a person's grants, pairings
    /// and folder are is the desktop's to know, because a compositor that
    /// opened a person's settings file would be a compositor measuring.
    fn the_settings_places(&mut self) -> Option<crate::SettingsPlaces> {
        None
    }

    /// The canvas layout this person left, asked for once as the session
    /// starts.
    ///
    /// **The shell asks and does not read.** `alo-desktop`'s own manifest gives
    /// the reason: the shell shows and never measures, and shell task 15's
    /// constraint is that no call into the machine is added from
    /// `crates/alo-shell`. A file in a person's folder is that kind of call, so
    /// the binary reads it and hands the answer over — the same road the
    /// battery, the sound, the network and the locale already travel.
    ///
    /// **The default is a canvas nobody has arranged**, so a desktop that
    /// remembers nothing is still a desktop: every window opens where its
    /// application put it, which is what happens today.
    fn the_layout_they_left(&mut self) -> alo_arranging::Arrangement {
        alo_arranging::Arrangement::fresh()
    }

    /// The canvas layout is now this, after something a person did moved it.
    ///
    /// **Told rather than saved here**, for the same reason as above. The
    /// binary decides what to do with it, which is to keep it in the person's
    /// folder.
    ///
    /// **Not only at the end of a session.** The owner's ruling of 2026-10-03
    /// asks for a save after meaningful layout changes rather than on a clean
    /// shutdown alone, because the session a person loses is the one that did
    /// not end cleanly. So this is called when the layout has actually changed
    /// — not every frame, and not once at the end.
    ///
    /// **The default does nothing**, and that is the honest default: a desktop
    /// that was not asked to remember anything should not be made to.
    ///
    /// # What comes back is not what went in
    ///
    /// The answer is **what is now kept**, which is this layout with each
    /// Place's earlier states carried forward and the one being replaced added
    /// to them. The shell builds an arrangement from the live canvas each time,
    /// so what it hands over has no memory; what comes back does, and that is
    /// what a person's ribbon is drawn from.
    ///
    /// **Returned rather than assembled on both sides.** The shell holds the
    /// arrangement it last kept and could carry the series forward itself, and
    /// then two places would be computing what a Place remembers — agreeing by
    /// construction today and by luck after the first edit to either.
    /// `alo_arranging::keeping::keep` is the one assembler and this is how its
    /// work reaches the session.
    ///
    /// The default answers the argument unchanged, which is the truth for a
    /// desktop that keeps nothing: a layout nobody wrote down remembers
    /// nothing earlier than itself.
    fn the_layout_is_now(
        &mut self,
        arrangement: alo_arranging::Arrangement,
    ) -> alo_arranging::Arrangement {
        arrangement
    }
}

impl crate::DirectSession {
    /// Stand the desktop up on this machine's display, and keep it there.
    ///
    /// Draws for every frame `next` asks for until it says stop or the seat
    /// takes the display away. Clients are dispatched and drawn underneath, and
    /// their input is routed to them exactly as `run_compositor_with_input`
    /// routes it: this lane changes what is drawn above the clients, never who
    /// hears the keyboard.
    ///
    /// # Errors
    /// [`SessionError`] when the seat refuses the display. Everything the
    /// display lifetime reported is in the result.
    pub fn desktop(
        &mut self,
        server: &mut Server,
        desktop: &mut dyn TheDesktop,
        labels: &mut WindowControlLabels,
        strings: &alo_strings::Strings,
        mut next: impl FnMut() -> crate::DirectFrame,
    ) -> Result<crate::ActiveSessionResult<crate::DirectLoopResult>, SessionError> {
        server.clear_input();
        // Opened once, here, rather than retried every frame: a machine with no
        // accessibility bus will not grow one mid-session, and asking sixty
        // times a second would be a D-Bus call per frame answering the same no.
        let reader = crate::TheReaderIsTold::opened(
            server,
            strings,
            &[alo_access::Surface::Desktop],
            // **Nothing is turned on and that is read rather than
            // assumed.** This binary has no settings file behind it
            // yet; when it has one, what a person turned on comes
            // from there and a reader is told which way each switch
            // is set without another line changing here.
            &alo_access::TurnedOn::nothing(),
        )
        .ok();
        let manager = self.input_session();
        self.with_active_device(|fd, poll| {
            let setup = (|| {
                // **Every display this device has, in preference order.**
                // `more-than-one-display-plan.md` task 4: task 2 built the
                // discovery and nothing outside a test had ever called it, so
                // a machine with two monitors lit one. Asked **once** rather
                // than once for the first and again for the rest — two
                // discoveries would be two answers to *what is plugged in*,
                // and they can differ across a hotplug between them.
                //
                // The first is this loop's own, because `run_with_input` holds
                // one target and six implementations hang off that signature.
                // The rest belong to `Desk`, which draws them in its own
                // `present` and retires them in `retire_the_rest`. A refusal
                // with no display at all is `discover_every`'s, unchanged.
                let mut every = crate::discover_every_atomic_output(fd)?;
                let output = every.remove(0);
                poll()?;
                let (width, height) = output.output.mode.size();
                let painter = crate::software_scanout::SoftwarePainter::new()?;
                // **A painter each, because a `Target` owns its painter**, and
                // a borrow each of the one descriptor, which is what
                // `Inventory` is. A display whose painter cannot be made is
                // left out rather than taking the session down with it: the
                // machine still has the display above, and that is exactly the
                // isolation this task is about.
                let others = every
                    .into_iter()
                    .filter_map(|output| {
                        let painter = crate::software_scanout::SoftwarePainter::new().ok()?;
                        Some(crate::direct_target::Target::new(
                            painter,
                            crate::drm_inventory::Inventory(fd),
                            output,
                        ))
                    })
                    .collect::<Vec<_>>();
                // **Buffer sharing is promised here and not in
                // `Surfaces::new`, because the line above is where a renderer
                // starts existing.** The owner ruled on 2026-10-02 that DMA-BUF
                // is advertised after the renderer is initialised and never
                // before: a global put up at birth promises an import this
                // shell may have no way to perform, and a client discovers that
                // only when its buffer comes back refused.
                //
                // The formats are the renderer's own, asked of it through
                // `crate::direct_target::ScenePainter::importable_formats`. A
                // renderer that can import nothing advertises nothing, which is
                // why there is no condition written here.
                server.advertise_importable_buffers(
                    crate::direct_target::ScenePainter::importable_formats(&painter),
                );
                let input = crate::direct_input_loop::RoutedInput {
                    owner: crate::SeatInput::new(manager)?,
                    extent: (i32::from(width), i32::from(height)),
                };
                Ok::<_, DirectLoopError>((output, painter, input, others))
            })();
            // **What this person's chords mean, told to the seat before the
            // loop owns it.** Until this line existed no chord reached a
            // running desktop at all: every link of the shortcut chain was
            // production code and nothing entered it, so fifteen actions were
            // finished, tested and unreachable — `the-shell-plan.md` task 19.
            // See `crate::a_chord_reaches_its_action`.
            //
            // **Before the match rather than beside the layout below**, because
            // the loop borrows the server for its whole run and the arm that
            // starts it is inside that borrow. Asked once here for the same
            // reason the layout is asked once there: what a person bound is not
            // a reading that goes stale.
            server.the_shortcuts_are(desktop.the_shortcuts());
            // And where this person's settings are kept, asked in the same
            // breath and for the same reason. A desktop that answers `None`
            // leaves the seat exactly as it was.
            if let Some(places) = desktop.the_settings_places() {
                server.the_settings_places_are(places);
            }
            match setup {
                Ok((output, painter, input, others)) => crate::direct_loop::run_with_input(
                    server,
                    crate::direct_target::Target::new(
                        painter,
                        crate::drm_inventory::Inventory(fd),
                        output,
                    ),
                    poll,
                    &mut next,
                    {
                        // **Asked before the desktop is moved into the loop**,
                        // and once rather than per frame: a layout is what a
                        // person left, not a reading that goes stale.
                        // **Read once and used twice**: the places waiting to
                        // be claimed, and the ribbon those Places arrived with.
                        // Two reads would be two answers to *what did this
                        // person leave*, and the second would be taken after
                        // the first had already begun being claimed.
                        let layout = desktop.the_layout_they_left();
                        let left = crate::WhereTheyLeftIt::from(layout.clone());
                        Desk {
                            input,
                            others,
                            desktop,
                            labels,
                            strings,
                            left,
                            displays_described: Vec::new(),
                            told: alo_arranging::Arrangement::fresh(),
                            series: layout,
                            reader,
                        }
                    },
                ),
                Err(error) => crate::DirectLoopResult {
                    outcome: Err(error),
                    input_cleanup: None,
                    input_flush: Some(server.flush()),
                    retirement: None,
                    rest_retirement: Vec::new(),
                    flush: None,
                },
            }
        })
    }
}

/// The desktop lane: the clients' own input, and the desktop drawn above them.
struct Desk<'a> {
    /// The ordinary routing, unchanged — a client hears the keyboard here.
    input: crate::direct_input_loop::RoutedInput,
    /// Every display but the one the loop itself holds.
    ///
    /// **Empty on a machine with one display**, which is every machine this
    /// lane can test on, and the reason the first target stays where it was:
    /// `run_with_input` takes one, six `LoopInput` implementations hang off
    /// that signature, and the sign-in lane genuinely has one display and
    /// should not pay for this one.
    ///
    /// Each owns its own painter and borrows the same device descriptor —
    /// `Inventory` is a `BorrowedFd`, so N displays are N painters and N
    /// borrows of one `fd`, never N descriptors.
    others: Vec<
        crate::direct_target::Target<
            crate::software_scanout::SoftwarePainter,
            crate::drm_inventory::Inventory<'a>,
        >,
    >,
    /// What the crates that decide each of these say is on the display now.
    desktop: &'a mut dyn TheDesktop,
    /// The bundled font every word on the desktop is laid out with.
    labels: &'a mut WindowControlLabels,
    /// This machine's own sentences, for the names a reader is told.
    strings: &'a alo_strings::Strings,
    /// The layout this person left, offered to frames as they arrive.
    ///
    /// **Asked for once, as the session starts.** A remembered place is claimed
    /// by the first window of its application to open and is then gone, which
    /// is the rule `alo-arranging` states: two windows of one application share
    /// one remembered place, because `app_id` is what survives a session and a
    /// `wl_surface` is not.
    left: crate::WhereTheyLeftIt,
    /// **What this person's Places remember**, as the last keep left it.
    ///
    /// Starts as the layout read at sign-in, which already carries whatever
    /// ribbon the file held, and is replaced by what `the_layout_is_now` hands
    /// back. This is the one copy in the session, and
    /// `crate::canvas_a_place_remembers_time` is what reads it.
    series: alo_arranging::Arrangement,
    /// The displays last described to the desktop, so a re-arrangement
    /// happens when the set of them changes and not every frame.
    ///
    /// The same shape as `told` below and for the same reason: a frame is not
    /// a change, and a display drawing its thousandth identical frame must
    /// not cost an arrangement.
    displays_described: Vec<alo_displays::Reported>,
    /// The layout last told to the desktop, so a save happens on a change.
    ///
    /// **Not every frame.** The owner's ruling asks for a save after meaningful
    /// layout changes, and a frame is not a change — a person reading a page
    /// moves nothing. So the arrangement is compared with this and the desktop
    /// is told only when they differ, which also makes *meaningful* a thing the
    /// code decides rather than a word in a document.
    told: alo_arranging::Arrangement,
    /// The tree on the accessibility bus, where there is one to serve it on.
    ///
    /// `None` on a machine with no accessibility bus, which is the ordinary case
    /// for somebody who has never turned a reader on — ADR 0063, *a machine that
    /// cannot run the engine says so rather than failing*. The desktop runs
    /// either way; what changes is whether anybody can read it.
    reader: Option<crate::TheReaderIsTold>,
}

impl LoopInput for Desk<'_> {
    /// Route the input, then tell the desktop where the pointer ended up.
    ///
    /// **The clients' routing is untouched**, which is this lane's standing promise: a client
    /// still hears the keyboard and still gets its motion. The panel is told *as well*, not
    /// instead — the question of whether a pointer over the panel should be withheld from the
    /// client belongs to the reveal machine, which does not exist yet, and answering it here
    /// would be the compositor deciding something no crate has decided.
    ///
    /// **The one exception is the window this shell opened itself.** While Settings is open its
    /// keys are Settings', because `crate::settings_seat` says so: *every key is intercepted at
    /// the seat and never forwarded*. The pointer is not, and a batch with no key in it is the
    /// same batch it was. Asked per batch rather than per lifetime, because the press that
    /// closes the window is the last one Settings takes — see
    /// `crate::a_key_reaches_settings`.
    ///
    /// **This lane rather than the seat** for the reason that file gives: the press needs the
    /// person's vocabulary to word a refusal, the server does not hold it, and `Desk` is the one
    /// place a running desktop's strings and its seat are both in scope.
    ///
    /// **After the dispatch, not during it.** One classification per input batch, from the
    /// position the seat settled on, rather than one per motion event — a person crossing the
    /// panel produces many events and one answer, and the extra answers would all say the same
    /// thing. This is also the only place both the input and the desktop are in scope, which is
    /// why it is here rather than inside `crate::pointer`.
    fn dispatch(
        &mut self,
        server: &mut Server,
        poll: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError> {
        let extent = self.input.extent;
        let strings = self.strings;
        self.input.owner.dispatch(
            || poll().map_err(std::io::Error::other),
            |update| {
                if server.settings_is_taking_the_keys() {
                    return crate::a_key_reaches_settings::routed_or_io(
                        server, update, extent, strings,
                    );
                }
                server
                    .libinput_update(update, extent)
                    .map_err(std::io::Error::other)
            },
        )?;
        // **Nothing before the first draw**, because the panel has no geometry until then —
        // see `Server::panel_as_drawn`. A pointer moving on a machine that has not painted is
        // an ordinary moment and not something to invent an answer for.
        // **The frame carries the `Panel`, so the classification is the identity-checked one.**
        // The borrow is taken and dropped before the desktop is told, because `now()` borrows
        // it immutably and `the_pointer_is_now` needs it mutably — `OnThePanel` is `Copy`, so
        // the answer outlives the borrow that produced it.
        let on = server.where_the_pointer_is_on_the_panel(self.desktop.now().put_aside);
        if let Some(on) = on {
            self.desktop.the_pointer_is_now(on);
        }
        // **What a person asked for, performed where the panel is reachable.** The keyboard
        // and the window's own controls record the ask on the server, because neither has a
        // desktop in scope; this is the one place that holds both. See
        // `Server::asked_to_put_aside`.
        server.put_aside_what_was_asked_for(self.desktop.the_panel());
        // **And the other direction, met in the same place and for the same reason.** A click
        // on a preview was claimed where input happens, because whether the panel owns a click
        // is geometry; *which* preview it chose needs the live `Panel` for the identity check,
        // and that is here. See `crate::a_click_brings_a_window_back`.
        //
        // After the put-aside asks rather than before, so a window put aside and clicked in
        // one frame is put aside first and then brought back — the order the person did them
        // in. The reverse would bring back a window that was not in the panel yet and then
        // put it away, leaving it aside when they had asked for it back.
        server.bring_back_what_was_clicked(self.desktop.the_panel());
        // **And what that same position means to the panel's reveal machine.** A separate
        // question from the peek: one asks which preview, this asks whether the edge or the
        // surface is being touched. Both read the one stored draw, so they cannot disagree
        // about where the pointer is.
        if let Some(by) = server.what_the_pointer_is_to_the_panel() {
            self.desktop.the_panel_is_revealed(by);
        }
        Ok(())
    }

    /// Make this frame's pictures and submit them with the clients.
    ///
    /// The record and a question are not carried yet: nothing on a machine
    /// opens either, and a frame that reserved room for them would be drawing
    /// for a state that cannot arrive. They are the same two `Option`s in
    /// `frame_pictures` when something does.
    fn present<T: crate::direct_loop::LoopTarget + crate::presentation::NativeTarget>(
        &mut self,
        server: &mut Server,
        target: &mut T,
        time: u32,
    ) -> Result<(), DirectLoopError> {
        // **The extra displays are borrowed out of `self` first**, so that
        // the rest of this body may keep borrowing the desk's other fields.
        // Taking them instead would lose them on any `?` below, and they own
        // live scanout.
        let others = &mut self.others;
        // Asked before the frame is made, never after: a frame drawn from
        // readings taken after it would show a person the moment before.
        self.desktop.refreshed();
        // **Every frame that has arrived is offered the place it was left in.**
        // Before the frame is drawn, so a window appears where the person left
        // it rather than being moved a frame later under their eyes. A window
        // whose application was not remembered, or whose remembered place has
        // already been claimed, is left where it opened — `put_back_where_it_was`
        // answers whether it moved and the answer is not needed here.
        //
        // The surfaces are collected first because `mapped` borrows the server
        // that the put-back needs mutably.
        if self.left.anything_left() {
            let arrived: Vec<_> = server.mapped_surfaces().cloned().collect();
            for frame in arrived {
                let _moved = server.put_back_where_it_was(&frame, &mut self.left);
            }
        }
        let size = target.size();
        // **The one place holding both the windows and the frame.** The
        // desktop's own state has no server in it, so it cannot say where the
        // windows are; the server cannot say what the person chose about the
        // dock. The dock's question needs both, and they meet here.
        let windows = server.window_areas();
        // **A local, for `windows`' reason exactly.** The frame borrows it, so
        // it has to outlive the frame; built inside `what_only_the_server_knows`
        // it would die at that function's end.
        let on_the_dock = server.what_the_dock_holds(&alo_dock::Holding::nothing());
        let mut frame = self.desktop.now();
        let named = FrameTarget::metadata(target)?.name;
        what_only_the_server_knows(&mut frame, server, &windows, &on_the_dock, &named);
        let pictures = crate::nested_desktop::frame_pictures(
            frame,
            None,
            None,
            self.labels,
            (size.w, size.h),
        )?;
        // **Where the fixed controls ended up, handed to the drag that has to
        // avoid them.** The draw is the only place that knows: these are laid
        // out here and nowhere else. Without this line
        // `crate::canvas_never_lost`'s rule has no bounds to check against,
        // which is why it had no caller at all — see
        // `crate::canvas_fixed_controls`.
        //
        // **Every control this frame laid out, not only the Dock's band.** Until
        // 2026-10-02 the band was the only one handed over, so the rule was in
        // force against one of the three the promise names: a frame could keep
        // its name clear of the Dock and sit entirely under the put-aside panel,
        // and nothing could tell, because the shell had never been given the
        // panel's bounds to check against.
        // **Every other display is laid out at its own size and its own
        // scale.** `more-than-one-display-plan.md` task 5. Until this, the
        // displays beyond the first were drawn the *first* display's dock,
        // status area and panel — laid out once from one size — so a second
        // monitor of a different shape showed furniture built for its
        // neighbour, and a dense one showed it at half the size it owns.
        //
        // Laid out before any of them is drawn, and kept in a list the draw
        // borrows from, because each display's layers have to outlive the
        // wrapper that paints them and all the wrappers go down one road.
        //
        // Settings is **not** among them: it is one window, and a window is
        // on one display. Which one is not a question this task can answer —
        // nothing maps a surface to a display until task 7 — so it stays on
        // the display the loop itself holds, which is the internal panel
        // where discovery found one. Drawing it on each display would be
        // visibly wrong in a way no test here would catch: two Settings
        // windows, one of them unreachable.
        let mut theirs = Vec::with_capacity(others.len());
        for other in others.iter_mut() {
            let named = FrameTarget::metadata(other)?.name;
            let size = FrameTarget::size(other);
            let mut frame = self.desktop.now();
            what_only_the_server_knows(&mut frame, server, &windows, &on_the_dock, &named);
            // **This binding's name is load-bearing**, and was `laid_out`
            // until 2026-10-08. `the_recheck_has_a_caller`'s guard reads the
            // draw's text and asks that every control handed to the rule is
            // read from a `pictures` binding rather than from a constant — the
            // one thing the compiler cannot ask, because a literal rectangle
            // satisfies `E0063` perfectly. A name that does not say `pictures`
            // fails that guard, which is the guard working: rename this and
            // either give it a name that still says what it is, or say in that
            // test where the rectangles now come from. Do not loosen it.
            let its_pictures = crate::nested_desktop::frame_pictures(
                frame,
                None,
                None,
                self.labels,
                (size.w, size.h),
            )?;
            theirs.push((other, its_pictures));
        }
        // **Every display's controls, from the one list that also draws
        // them.** `more-than-one-display-plan.md` task 9.
        //
        // This recorded the loop's own display alone until 2026-10-08, with
        // the per-display layout sitting after it — so the store held one
        // entry whatever the arrangement said, and task 5b's `len() <= 1`
        // safeguard, written to keep a single display behaving exactly as
        // before, handed that one entry to every window on every display. The
        // safeguard was doing the opposite of its job because the writer was
        // only ever called once.
        //
        // **Fixed by moving the layout above this rather than by adding a
        // second call**, so the list that draws and the list that records are
        // the same list and a display cannot be in one without the other. A
        // second call would have fixed today and left the next display to be
        // forgotten the same way.
        //
        // It also puts every display's furniture in front of
        // `bring_back_frames_the_moved_controls_hide` below, which until now
        // asked its question with only the first display's bounds recorded —
        // so a frame hidden by the second display's dock was not found.
        for (other, its_pictures) in &theirs {
            let Ok(its) = FrameTarget::metadata(*other) else {
                continue;
            };
            server.the_fixed_controls_were_drawn(
                &its.name,
                crate::canvas_fixed_controls::FixedControlsDrawn {
                    dock_band: its_pictures.desktop.dock.as_ref().map(|dock| dock.band),
                    panel_reserved: its_pictures.desktop.panel.reserved,
                    what_is_leaving: its_pictures.status.band,
                    top_controls: its_pictures.desktop.top_controls,
                },
                self.desktop.now().look.scale(),
            );
        }
        server.the_fixed_controls_were_drawn(
            &named,
            crate::canvas_fixed_controls::FixedControlsDrawn {
                dock_band: pictures.desktop.dock.as_ref().map(|dock| dock.band),
                panel_reserved: pictures.desktop.panel.reserved,
                // The third of the set. `None` when the indicator drew
                // nothing, which is the ordinary case on a machine with
                // nothing leaving it — and *nothing drawn covers nothing* is
                // the true answer rather than a placeholder.
                what_is_leaving: pictures.status.band,
                top_controls: pictures.desktop.top_controls,
                // **No scale is handed over, because these rectangles are in
                // the room they were laid out from and the handle floor is in
                // the same space.** Measured, not assumed: see
                // `desktop_raster_tests::the_dock_band_and_the_panel_column_do_not_move_with_the_displays_scale`.
                // The division below is the one surface that *does* convert,
                // and it reads the scale itself.
            },
            self.desktop.now().look.scale(),
        );
        // **And if those bounds moved, a frame they now hide is brought back.**
        // The line above has recorded where the controls are since 2026-10-02 and
        // nothing asked the next question: the Dock grows with its icons, the
        // panel's column appears when a window is put aside, and the display or
        // its scale can change under a frame that is not moving at all. After any
        // of those a reachable frame is unreachable, and before this line the
        // detector and the mover that handle it were called by tests alone.
        //
        // **One frame behind, inherently.** These bounds come from the pictures
        // laid out above, so the earliest a change can be acted on is after they
        // exist, and a frame moved here is drawn where it moved to on the next
        // pass. That is a single frame of a window sitting under the Dock, which
        // is what the alternative — laying out twice to move before drawing —
        // would cost every frame to avoid once.
        //
        // The result is dropped **because the telling is not built**, not because
        // it does not matter: the move has already happened and is visible.
        //
        // **This comment named the wrong road until 2026-10-04.** It said the
        // sentence needs *alo OS's first production notification and its words in
        // every shipped language*, and following that leads into a wall that has
        // nothing to do with the canvas: `alo_notifying::deciding::arrives` wants a
        // `Seat<Notification>`, a seat wants an `alo_accounts::Session`, and
        // `SignedIn`'s only constructor is `pub(crate)` — deliberately, so nothing
        // outside the authentication path can mint an identity. `alo-greeting` is a
        // library, no binary carries its session here, and `alo-desktop` depends on
        // none of those crates.
        //
        // The promise does not ask for any of it. `docs/features.md`'s **When the
        // machine moves a window, the person is told** says the notice is *not the
        // notifications portal and not calm notifications*, because *the canvas
        // needs one sentence about one frame*. One sentence about one frame is this
        // crate's own kind of surface, like the egress indicator, and
        // `crate::egress_status_place::Place::of_the_other_end` already records
        // where such a thing goes: the end of the edge a person's eyes learn for
        // **what just happened**.
        //
        // What is genuinely undecided is one clause — how *where it was* reaches a
        // person. `alo_canvas::At` is `{ x, y }`, no crate on the canvas side
        // carries any vocabulary, and a coordinate read aloud is not something
        // anybody can act on. `docs/autonomy/the-canvas-and-its-places.md` holds
        // the three readings. See
        // `crate::canvas_fixed_controls::Server::bring_back_frames_the_moved_controls_hide`,
        // which holds the argument and the `was` that road will need.
        let _ = server.bring_back_frames_the_moved_controls_hide();
        // **And where the panel ended up, for the same reason and in the same place.**
        // Its slots are laid out in `crate::panel_raster` and exist only for this
        // frame; a pointer arriving afterwards has nothing to be tested against
        // without this line, which is why `crate::which_preview_the_pointer_is_on`
        // had no geometry to ask about. Cloned rather than borrowed because the
        // pictures do not outlive the frame and the question is asked after it.
        server.the_panel_was_drawn(crate::which_preview_the_pointer_is_on::ThePanelAsDrawn::of(
            frame.put_aside,
            pictures.desktop.panel.clone(),
        ));
        // **Settings, when a chord has opened it.** `NativeLayers` has carried
        // this slot since it was written and nothing ever filled it, which is
        // the other half of `the-shell-plan.md` task 18: the window had no
        // owner, and even once it had one there was nothing drawing it.
        //
        // **Built from the desktop's own look rather than a second one.** The
        // four figures Settings needs — the scheme, the text scale, which way
        // the person reads and the contrast — are the same four this frame is
        // already drawing everything else with, read back through
        // `DesktopLook`'s accessors. A `SettingsLook` assembled from
        // `alo-appearance` a second time here would be a second answer to *how
        // does this person's machine look*, and the two could differ for one
        // frame after a change.
        //
        // `settings_raster::picture` answers an empty picture for a shut
        // window, so this costs a shut desktop one call and draws nothing.
        let settings = crate::settings_raster::picture(
            server.the_settings_window(),
            frame.strings,
            self.labels,
            (size.w, size.h),
            crate::SettingsLook {
                scheme: frame.look.scheme(),
                scale: frame.look.scale(),
                reading: frame.look.reading(),
                contrast: frame.look.contrast(),
            },
            std::time::SystemTime::now(),
        )?;
        // **The frame is drawn through the per-display road, with one display
        // on it.** `more-than-one-display-plan.md` task 4. This desk builds one
        // target today; `discover_every_atomic_output` is what will make it
        // several, and that is this task's device half.
        //
        // Going through the road now rather than when the second target
        // arrives puts the isolation **in the path a machine actually takes**
        // instead of beside it, so the change that adds a display pushes a
        // target onto a list rather than rewriting how a frame is drawn. A
        // mechanism nothing calls is the fault this repository keeps meeting;
        // one caller on the real road is the cure.
        //
        // The refusal is raised here exactly as `render_frame`'s was, because
        // with one display *nothing drew* and *this display refused* are the
        // same fact. With two they will not be, and this is the line that
        // stops raising and starts reporting.
        let mut layered = Layered {
            target,
            layers: NativeLayers {
                desktop: Some(&pictures.desktop),
                status: Some(&pictures.status),
                settings: Some(&settings),
                ..NativeLayers::nothing()
            },
        };
        //
        // The loop's own display goes first, because discovery put it first
        // and because a refusal on a later one must not delay it.
        let mut painted: Vec<Layered<'_>> = theirs
            .iter_mut()
            .map(|(other, its_pictures)| Layered {
                target: *other,
                layers: NativeLayers {
                    desktop: Some(&its_pictures.desktop),
                    status: Some(&its_pictures.status),
                    ..NativeLayers::nothing()
                },
            })
            .collect();
        let mut displays: Vec<&mut dyn crate::FrameTarget> = Vec::with_capacity(1 + painted.len());
        displays.push(&mut layered);
        for display in &mut painted {
            displays.push(display);
        }
        let became = server.render_each_display(&mut displays, time);
        // **Raised only when nothing reached a screen.** With one display
        // *nothing drew* and *this display refused* are the same fact, and
        // raising is right. With two they are not: a machine whose second
        // monitor refused one frame is a machine a person is still working
        // on, and taking the session down over it would make a spare display
        // more dangerous than no display at all.
        //
        // The name is dropped on the raising road and only there — an error
        // leaving this lane goes to a caller that cannot act on which display
        // it was, and `DrawnPerDisplay` keeps the names for the one that can.
        if !became.anything_drawn()
            && let Some((_named, why)) = became.into_refusals().into_iter().next()
        {
            return Err(why.into());
        }
        // **What a reader is told follows what is open**, and this is the call
        // that was missing: the tree and the bus were both written and tested
        // and nothing outside a test ever built either, so a screen reader on a
        // running machine found no application at all.
        //
        // Cheap every frame on purpose — it compares the window names it last
        // published and touches the bus only when they differ, so a frame being
        // dragged or redrawn costs one comparison of a short list.
        //
        // A refusal is not allowed to stop the desktop. A reader that cannot be
        // reached is a person without a reader; a compositor that stopped
        // compositing over it would be a machine nobody can use at all.
        if let Some(reader) = self.reader.as_mut() {
            let _ = reader.following(
                server,
                self.strings,
                &[alo_access::Surface::Desktop],
                &alo_access::TurnedOn::nothing(),
            );
        }
        // **And the screens are re-arranged when the set of displays changes.**
        //
        // Asked against the descriptions rather than against a frame count, so
        // a display whose mode changed is a new description and a display
        // drawing its thousandth identical frame is not. Compared with what
        // was last handed over, for the reason the layout below gives: a
        // desktop does not know what arranging costs, and this crate must not
        // decide how often one is done.
        //
        // `more-than-one-display-plan.md` task 3a. Nothing reads the
        // arrangement yet — tasks 5, 6 and 7 are what will — and it is kept
        // live from the first frame so that those tasks find it already true
        // rather than having to wire this as well.
        let reported = server.the_displays_as_reported();
        if reported != self.displays_described {
            self.displays_described.clone_from(&reported);
            // **The desktop is asked for the person's settings; the arrangement
            // is made here.** Task 10 of `more-than-one-display-plan.md`. The
            // shell holds what the machine reports and which sizes it can draw;
            // the desktop holds the three things that are the person's.
            let arranged = self
                .desktop
                .their_displays()
                .and_then(|theirs| theirs.the_screens_of(reported));
            server.these_screens_are(arranged);
        }
        // **And the desktop is told when the layout has actually moved**, which
        // is what makes a person's canvas survive a restart.
        //
        // The same shape as the reader above, for the same reason it gives:
        // cheap every frame on purpose, because it compares what it last said
        // with what is true now and speaks only when they differ. A frame being
        // redrawn, a page being read, a pointer crossing the panel — none of
        // those moves a window, so none of those is a save.
        //
        // **This is what the owner's ruling means by a save after meaningful
        // layout changes rather than on a clean shutdown.** A session a person
        // loses is the one that did not end cleanly, so there is nothing to hook
        // at the end that would have helped them.
        //
        // The shell does not write the file. It hands the arrangement over and
        // `alo-desktop` keeps it, because the shell shows and never measures —
        // and a file in a person's folder is a reading like any other.
        let now = server.the_arrangement_now();
        if now != self.told {
            self.told = now.clone();
            // **The comparison above is against the memoryless arrangement,
            // and the series is kept beside it rather than in it.** `told` is
            // what the live canvas says; the series is that plus what every
            // Place remembers. Comparing `now` against the series would differ
            // on every frame the moment a ribbon had one stop on it, and the
            // save would go from one per rearrangement to one per frame.
            self.series = self.desktop.the_layout_is_now(now);
            // And the seat is told, because a chord is answered where no
            // desktop is in scope — the same road the shortcuts travel. See
            // `crate::canvas_a_place_remembers_time`.
            server.these_places_remember(&self.series);
        }
        Ok(())
    }

    /// Disable every display this desk opened beyond the loop's own.
    ///
    /// **Each one is attempted even where an earlier one refused**, which is
    /// the same rule as drawing: a backend that could not be disabled is a
    /// reason to report, never a reason to leave the next display lit. The
    /// loop retires its own target after this returns, and that is what
    /// withdraws every display's `wl_output` — see
    /// `crate::output_retirement`, where task 3 split the two apart.
    fn retire_the_rest(&mut self, _server: &mut Server) -> Vec<(String, crate::RenderError)> {
        let mut displays: Vec<&mut dyn crate::FrameTarget> =
            self.others.iter_mut().map(|it| it as _).collect();
        crate::retire_each_display(&mut displays)
    }

    fn shutdown(self, server: &mut Server) -> Option<std::io::Result<()>> {
        self.input.shutdown(server)
    }
}

/// A display with this frame's own layers waiting on it.
///
/// The same arrangement `crate::window_control_frame`'s own wrapper makes, and
/// for the same reason: the server owns what a frame *is* — which clients, which
/// popups, which cursor, and who is told it was drawn — and this only says what
/// is painted above them.
/// **Not generic over the display, deliberately.** One session's displays are
/// not one type — the loop holds its own target and `Desk` holds the rest —
/// and a generic wrapper would make a list of them impossible to write. The
/// displays differ; what is painted above them does not.
struct Layered<'a> {
    /// The display underneath, which decides whether anything was submitted.
    target: &'a mut dyn crate::presentation::NativeTarget,
    /// This shell's own surfaces for this frame.
    layers: NativeLayers<'a>,
}

impl FrameTarget for Layered<'_> {
    fn metadata(&self) -> Result<crate::OutputMetadata, RenderError> {
        self.target.metadata()
    }
    fn size(&self) -> Size<i32, Physical> {
        self.target.size()
    }
    /// Forwarded to the display underneath, like the extent and the identity:
    /// what this wrapper adds is the layers above the plane, and it decides
    /// nothing about where the plane is.
    fn look_at(&mut self, camera: alo_canvas::Camera) -> Result<(), RenderError> {
        self.target.look_at(camera)
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_popups(roots, &[], &Cursor::Default)
    }
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.target
            .submit_native_layers(roots, popups, cursor, self.layers)
    }
}
