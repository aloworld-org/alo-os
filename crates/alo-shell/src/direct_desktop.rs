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

    /// What this seat has made of a person's input, offered every frame.
    ///
    /// **Offered rather than printed**, because this crate is a compositor and
    /// what a machine says to whoever is watching it is the binary's to decide.
    /// `alo-desktop` prints it when it changes; the display probe wants
    /// nothing. The same shape as [`Self::the_indicator`] and
    /// [`Self::the_panel`], and for the same reason.
    ///
    /// **Every frame rather than when the session ends.** It was the session's
    /// end until 2026-10-18, and that is exactly how much use it was: on a
    /// machine with no shell — `serial-getty@ttyS0` is masked in the image on
    /// purpose — a press that opened nothing could not be told apart from a
    /// press that never arrived, because the one number separating them was
    /// printed after the desktop was already gone.
    /// [`crate::WhatTheSeatMadeOfIt`] carries that measurement.
    ///
    /// Three field reads, so a lane may ask on every frame without weighing it.
    ///
    /// Nothing by default, for the reason above.
    fn what_the_seat_made_of_its_input(&mut self, _made: crate::WhatTheSeatMadeOfIt) {}

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

    /// **What this machine is doing on its own**, lent for the acts that change
    /// it.
    ///
    /// There is **one indicator on a machine** and the thing that draws it owns
    /// it, which is why this is lent rather than held here:
    /// `alo_egress::Indicator`'s own header is the argument, and
    /// `alo_turn::Machine` borrows one for exactly the same reason.
    ///
    /// The lane needs it for the one errand a person starts from a window this
    /// shell draws: installing an application. The line goes up before the
    /// fetching begins and comes down when it ends, both on this thread, and
    /// `crate::what_a_person_installs` is where that order is argued.
    ///
    /// **`None` by default**, like [`Self::the_panel`] and for the same reason:
    /// a desktop with no indicator is a real one — the display probe and every
    /// test are in that position — and a default that invented one would be a
    /// surface claiming nothing is leaving on a machine that cannot know.
    ///
    /// **A desktop that answers `None` installs nothing**, and that is the
    /// honest consequence rather than a gap: law 1 says an errand is visible at
    /// the moment it happens, so a machine with nowhere to show one may not
    /// start one. The lane refuses the ask and the window says so.
    fn the_indicator(&mut self) -> Option<&mut alo_egress::Indicator> {
        None
    }

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
    /// A task for this request, if this machine can answer one.
    ///
    /// **The last link between a person pressing Enter and a window opening.**
    /// `crate::Server::a_person_asked_alo` records what somebody typed; this is
    /// how it becomes an `ATask`, and the shell asks rather than reads for
    /// `the_settings_places`' reason exactly: making a task needs to know
    /// **which model answers here**, that answer lives in a person's settings,
    /// and `tests/settings_source.rs` holds the whole shell to never naming a
    /// settings file. *A compositor that opened a person's settings file would
    /// be a compositor measuring.*
    ///
    /// [`None`] is a true answer and the honest default: a machine where nobody
    /// has chosen what answers their questions **cannot** make a task, and the
    /// request stays unanswered rather than a window opening onto a model that
    /// is not there. A desktop that has not been taught to read the choice
    /// returns it too, which is why the default is this and not a panic.
    ///
    /// **Never a fallback.** If the person's choice names a provider this
    /// machine has no entry for, the answer is [`None`] — not *this machine
    /// instead*. Silently answering locally a question somebody pointed
    /// elsewhere, or the reverse, is the one thing
    /// `docs/decisions` and law 1 both forbid.
    fn a_task_for(&mut self, _request: &str) -> Option<alo_tasks::ATask> {
        None
    }

    /// Save a provider a person has just filled in, and say whether it took.
    ///
    /// **Asked rather than done here, for [`Self::a_task_for`]'s reason
    /// exactly.** Saving is two writes to this machine — a key to the keyring
    /// and a provider to the person's own settings — and
    /// `tests/settings_source.rs` holds the whole shell to never naming a
    /// settings file. A compositor that opened a keyring would be a compositor
    /// measuring. So a screen collects what was typed and this carries the
    /// asking; `alo-desktop` is where both writes happen, and the order they
    /// happen in is argued there.
    ///
    /// `false` is the honest default and agrees with [`Self::a_task_for`]'s
    /// [`None`]: a desktop that has not been taught to save cannot, and saying
    /// so is what lets a screen tell a person it did not happen rather than
    /// show them a provider that is not there.
    ///
    /// **Which of four things happened** — not a `bool`, and not the reason
    /// underneath it.
    ///
    /// This said *whether it worked, and not why* until 2026-10-23, for a
    /// reason that was right and a consequence that was not: a refusal crossing
    /// here must not be another crate's error in English this crate did not
    /// write. It does not have to be.
    /// [`crate::a_provider_is_added_in_settings::WhyNotSaved`] is **this
    /// crate's own** four cases, and the caller maps its failures onto them —
    /// so the words stay where words belong and the surface still learns enough
    /// to say something.
    ///
    /// **What the `bool` cost.** On a booted machine a person filled this form
    /// in, pressed Enter, and the screen did not change. The save ran, the
    /// keyring refused, `alo-desktop` wrote *there is no keyring on this
    /// machine, so a provider's key has nowhere to go* — to the service log —
    /// and `false` arrived here carrying nothing a surface could draw. The
    /// caller's own comment said that showing it was separate work. *Separate*
    /// and *silent* are the same thing to whoever is sitting there.
    ///
    /// # Errors
    /// [`crate::a_provider_is_added_in_settings::WhyNotSaved`], which the
    /// surface draws above the form it leaves open.
    fn a_provider_was_filled_in(
        &mut self,
        _saving: &alo_setting_up::a_provider_is_filled_in::WhatToSave,
    ) -> Result<(), crate::a_provider_is_added_in_settings::WhyNotSaved> {
        Err(crate::a_provider_is_added_in_settings::WhyNotSaved::ThereIsNoKeyring)
    }

    /// Whether anything on this machine is chosen to answer a question.
    ///
    /// **Asked without a request, because the answer does not depend on one.**
    /// [`Self::a_task_for`] already knows this — it is the first thing it finds
    /// out — but it can only say so *after* a person has pressed Enter, and by
    /// then the only honest thing left is to drop what they typed. This is the
    /// same fact one moment earlier, while the Bar is still open and they are
    /// still typing.
    ///
    /// **What it is for:** the line under the Bar's field, which otherwise counts
    /// the applications it found. `alo_dock::words::NOTHING_CAN_ANSWER_A_QUESTION`
    /// is what it says. Until 2026-10-22 nothing said anything: a question on a
    /// machine with nothing chosen was taken, dropped, and never mentioned, so a
    /// person pressed Enter and met silence and had no way to find out why.
    ///
    /// **Why not a notification afterwards.** Because alo OS has none, and the
    /// reason is structural rather than unfinished: `alo_notifying::deciding::arrives`
    /// is the only road to a `Shown`, it wants a `Seat<Notification>`, a seat
    /// wants an `alo_accounts::Session`, and `alo-compositor` — the one binary
    /// that holds a real one — signs a person in and exits with *nothing yet to
    /// draw in it*. Measured 2026-10-22. Prevention is both earlier and reachable.
    ///
    /// `false` is the honest default, and it agrees with [`Self::a_task_for`]'s:
    /// a desktop that has not been taught to read the choice cannot answer, and
    /// should say so rather than imply it can.
    fn anything_is_chosen_to_answer(&self) -> bool {
        false
    }

    /// opened a person's settings file would be a compositor measuring.
    fn the_settings_places(&mut self) -> Option<crate::SettingsPlaces> {
        None
    }

    /// Where this machine keeps its record, or why it could not say — asked
    /// once as the session starts, so Super+R and the row in Settings can open
    /// it.
    ///
    /// **The shell asks and does not read**, for the settings places' reason:
    /// the description naming the record's path is a file. `None` leaves the
    /// record unreachable and says so on the error stream when it is asked for.
    fn the_recounting(
        &mut self,
    ) -> Option<Result<alo_recounting::Recounting, alo_recounting::NotRecounted>> {
        None
    }

    /// **How this person writes a time and a day**, asked for once as the
    /// session starts: what they chose in `formats.toml` and the language they
    /// read.
    ///
    /// **The shell asks and does not read**, for the settings places' reason.
    /// `None` leaves the seat on `alo_formats::TheirFormats`' rule for a
    /// person who has chosen nothing — and the record is still written that way
    /// rather than one region's way for everybody.
    fn their_formats(&mut self) -> Option<alo_formats::TheirFormats> {
        None
    }

    /// **They chose another way of writing a time**, in Settings, and the
    /// status area's clock is this desktop's to write.
    ///
    /// Called on the frame after the change and never on a frame where
    /// nothing changed. Nothing by default: a desktop that writes no clock has
    /// nothing to rewrite.
    fn the_formats_are_now(&mut self, _formats: &alo_formats::TheirFormats) {}

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

    /// What this person pinned to their Dock, asked for once as the session
    /// starts.
    ///
    /// **The shell asks and does not read**, for `Self::the_layout_they_left`'s
    /// reason exactly: a file in a person's folder is a reading, and
    /// `crates/alo-shell/tests/settings_source.rs` refuses any shipped shell
    /// file that even names where settings are kept.
    ///
    /// **The default is `NeverSet`, which is the honest value and not a
    /// placeholder**: a desktop that keeps nothing has no stored list, so
    /// *nobody has chosen* is true of it. The defaults then apply at the next
    /// method, exactly as they do on a machine out of the box.
    fn the_pins_they_kept(&mut self) -> alo_dock::pinning::ThePins {
        alo_dock::pinning::ThePins::NeverSet
    }

    /// Keep these pins, because the defaults were just worked out for the first
    /// time.
    ///
    /// **Called at most once in a session's life**, and only when
    /// `WhatToPin::should_be_written_down` said so — which is never for a
    /// person who already has a list, never for a scan that failed, and never
    /// for one that has not finished. That is the owner's fourth clause made
    /// into a call: *do not permanently initialise an empty pin list from
    /// either.*
    ///
    /// **A desktop that keeps nothing does nothing here**, and the session is
    /// correct without it: it draws the defaults and works them out again next
    /// time. That is the state every test and the sign-in screen are in.
    fn keep_these_pins(&mut self, _pins: &[alo_dock::ADockEntry]) {}
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
            // And where the record is, for the same reason and in the same
            // breath.
            if let Some(recounting) = desktop.the_recounting() {
                server.the_record_is_kept(recounting);
            }
            // And how this person writes a time, for the same reason and in the
            // same breath. `crate::the_formats_are_told` says why there is one
            // copy and where it goes.
            if let Some(formats) = desktop.their_formats() {
                server.the_formats_are(formats);
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
                        // **What this machine is drawing to, said once.**
                        //
                        // The field below is seeded with whatever is already
                        // plugged in, deliberately — seeded empty it would
                        // report every display as an arrival. The consequence
                        // nobody had noticed is that the reporting block near
                        // the end of this file runs **only** on a change, so a
                        // machine with one screen that is never unplugged
                        // never says what it draws to **at all**. Not rarely:
                        // never, on the commonest machine there is.
                        //
                        // That block calls itself *the machine's own log, for
                        // whoever is reading one*. Something that speaks only
                        // about exceptions is not an account of state. So the
                        // state is said here, once, and changes go on being
                        // reported there.
                        //
                        // *Found by the desktop lane on 2026-10-16, reading a
                        // boot log for something else and asking why a line
                        // they expected was absent.*
                        let seeded = crate::a_screen_arrives_or_leaves::asked_of(
                            &crate::drm_inventory::Inventory(fd),
                        );
                        match &seeded {
                            Ok(displays) if !displays.is_empty() => eprintln!(
                                "alo-shell: this machine is drawing to {} display(s): {displays:?}",
                                displays.len()
                            ),
                            Ok(_) => eprintln!(
                                "alo-shell: this machine reports no displays, so there is \
                                 nothing to draw to yet"
                            ),
                            Err(why) => eprintln!(
                                "alo-shell: this machine would not say what is plugged in: \
                                 {why}"
                            ),
                        }
                        Desk {
                            input,
                            others,
                            device: fd,
                            // Asked of the same device the displays were just
                            // discovered on, so the first frame compares like
                            // with like.
                            plugged_in: seeded
                                .map(crate::a_screen_arrives_or_leaves::WhatIsPluggedIn::these)
                                .unwrap_or_default(),
                            desktop,
                            // **The door a person's words go out of** (ADR
                            // 0109), at the path the decision published. A
                            // machine with nothing listening there answers
                            // *nothing is listening* and the desktop keeps
                            // drawing, which is what a session before sign-in
                            // looks like.
                            asking:
                                crate::what_a_person_asked_goes_to_the_agent::TheAgentsDoor::default(
                                ),
                            // **The tool that ships, under no organisation's
                            // rule yet.** `alo_software::TheRentedTool` is the
                            // one this product installs with, and
                            // `Bound::Nobodys` is the honest answer on a
                            // machine whose description nothing here has read:
                            // the bound comes off `[applications]` in a file a
                            // root wrote, `alo-agentd` is what reads it, and a
                            // compositor that opened that file would be a
                            // compositor deciding what an organisation
                            // permits. **So a managed machine's rule does not
                            // reach this road yet** — written down rather than
                            // left to be discovered, because the consequence
                            // is that an organisation's limit on where
                            // applications come from is not applied to an
                            // install a person starts from this window. The
                            // places themselves still are: `Enabled::read`
                            // refuses one switched off or set up without
                            // signature checking whatever the bound says.
                            installing: Some(
                                crate::what_a_person_installs::TheInstallingThatIsAsked::with(
                                    std::sync::Arc::new(
                                        alo_software::TheRentedTool::on_this_machine(),
                                    ),
                                    alo_software::Bound::Nobodys,
                                ),
                            ),
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
    /// Where what a person typed goes, and where its answer comes back.
    ///
    /// **Owned by the loop rather than by the `Server`**, because it holds
    /// threads and a channel and the `Server` is handed out as `&mut` all over
    /// this crate. It is also the right place by what it is: the loop is the
    /// thing with a *frame* to drain an answer into, and ADR 0109's whole point
    /// is that nothing waits.
    asking: crate::what_a_person_asked_goes_to_the_agent::TheAgentsDoor,
    /// The road from *a person asked for this application* to *it is here*.
    ///
    /// **Owned by the loop for `asking`'s reason exactly**: it holds threads
    /// and a channel, the `Server` is handed out as `&mut` all over this crate,
    /// and the loop is the thing with a frame to drain an answer into.
    ///
    /// **`None` on a session with nothing to install with.** The sign-in
    /// screen, the display probe and every test are in that position, and a
    /// road built without a tool would be a road that refuses every ask with a
    /// sentence about the tool rather than one about the machine. A person on
    /// such a session cannot open the market either, because
    /// `Catalogue::decided` is the only thing that opens it and the Dock entry
    /// is theirs to pin — so this being `None` is not a surface a person can
    /// reach and be puzzled by.
    installing: Option<crate::what_a_person_installs::TheInstallingThatIsAsked>,
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
    /// The device every display on this desk is on.
    ///
    /// Held so that a display arriving mid-session can be discovered and
    /// painted. `Inventory` is a borrow of this one descriptor, so N displays
    /// are N painters and N borrows rather than N descriptors —
    /// `more-than-one-display-plan.md` task 13.
    device: std::os::fd::BorrowedFd<'a>,
    /// The displays plugged in as of the last frame.
    ///
    /// Seeded from the device as the session starts, so the **first** frame
    /// sees no change. Seeded empty it would report every display as an
    /// arrival and run one discovery nobody needed, which is a session that
    /// saw a change where there was none — the thing task 13's constraint
    /// forbids.
    plugged_in: crate::a_screen_arrives_or_leaves::WhatIsPluggedIn,
    /// The tree on the accessibility bus, where there is one to serve it on.
    ///
    /// `None` on a machine with no accessibility bus, which is the ordinary case
    /// for somebody who has never turned a reader on — ADR 0063, *a machine that
    /// cannot run the engine says so rather than failing*. The desktop runs
    /// either way; what changes is whether anybody can read it.
    reader: Option<crate::TheReaderIsTold>,
}

/// The folders this person has granted, for a request about to be sent.
///
/// A free function rather than a method, because the two call sites are in a
/// trait implementation. `crate::what_the_agent_may_reach` holds why the
/// person's own side is what names them.
fn the_folders_to_send(server: &Server) -> Vec<std::path::PathBuf> {
    server
        .settings_places
        .as_ref()
        .map_or_else(Vec::new, |places| {
            crate::what_the_agent_may_reach::what_the_agent_may_reach(
                places,
                std::time::SystemTime::now(),
            )
        })
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
                // **Which road this event takes**, decided first and refused
                // in one place below. It was four places each mapping its own
                // refusal straight to an `io::Error` — and an `io::Error` out
                // of this closure **ends the display loop**, so every one of
                // the four could take a person's desktop away over a single
                // event. `InputError::is_about_one_event` is the judgement
                // and it is made once, here, because four copies of it is
                // four chances to disagree about what is survivable.
                // **The record window first**, because opening it closes
                // Settings and it is the one open window that answered a
                // person's own chord most recently. Its question is false
                // unless it is open, so an ordinary frame pays one `is_open`.
                let went = if server.the_record_is_taking_the_keys() {
                    crate::a_key_reaches_the_record::a_key_reaches_the_record(
                        server, update, extent,
                    )
                } else if server.settings_is_taking_the_keys() {
                    crate::a_key_reaches_settings::a_key_reaches_settings(
                        server, update, extent, strings,
                    )
                }
                // **Then the window of what a person can install.** Behind
                // Settings and in front of the Dock, which is the order the
                // layers are drawn in rather than an order chosen here —
                // `crate::a_key_reaches_the_market`'s header carries the
                // argument. The question is false unless a person has the
                // window open, so an ordinary frame pays one `is_open`.
                else if server.the_market_is_taking_the_keys() {
                    crate::a_key_reaches_the_market::a_key_reaches_the_market(
                        server, update, extent,
                    )
                }
                // **Then the alo Bar, when it holds the keyboard.** Behind
                // Settings and the market and in front of the Dock's own
                // surfaces, which is the order those layers are drawn in: a
                // person with Settings open who presses a letter is typing
                // into Settings, whatever the Bar was doing behind it.
                //
                // **This arm is the whole of what was missing.**
                // `crate::a_chord_opens_the_bar` took the keyboard for the
                // composer and nothing ever handed it a key — measured on a
                // booted machine on 2026-10-23, twenty-nine characters typed
                // into a field that stayed empty.
                // `crate::a_key_reaches_the_bar` has the account.
                else if server.the_bar_is_taking_the_keys() {
                    crate::a_key_reaches_the_bar::a_key_reaches_the_bar(server, update, extent)
                }
                // **Then the task window holding the keyboard**, behind the Bar
                // because the Bar is drawn over it: a person who presses
                // Super+A in a task window is typing into the Bar, and when the
                // Bar lets go the keys come back here. Until 2026-10-26 nothing
                // routed a key to a task window at all.
                else if server.a_task_is_taking_the_keys() {
                    crate::a_key_reaches_the_task_in_front::a_key_reaches_the_task(
                        server, update, extent, strings,
                    )
                }
                // **Escape closes what the Dock has open**, and only then.
                // Asked after Settings because Settings is in front of the
                // Dock when both are up: a person pressing Escape at a
                // Settings window is cancelling that, whatever the Dock is
                // doing behind it.
                //
                // The question is false unless a Dock surface is open, so on
                // an ordinary frame this costs one `matches!` and the keys go
                // where they went before this existed.
                else if server.the_dock_is_taking_the_keys() {
                    crate::a_key_reaches_the_docks_surfaces::a_key_reaches_the_docks_surfaces(
                        server, update, extent,
                    )
                } else {
                    server.libinput_update(update, extent)
                };
                match went {
                    Ok(()) => Ok(()),
                    // **The machine did not understand one event and goes on
                    // drawing.** Counted, never swallowed: a refusal nobody
                    // can see is what hid the missing `enable_pointer` call
                    // until a machine was booted and its desktop died on the
                    // first pointer event.
                    Err(why) if why.is_about_one_event() => {
                        server.an_input_event_was_refused();
                        Ok(())
                    }
                    // **The seat itself.** The display or its keymap could not
                    // be made, which is not something the next event mends.
                    Err(why) => Err(std::io::Error::other(why)),
                }
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
        // **What was pressed on a window's edge, met before the put-aside asks
        // rather than after.** An edge's Minimise joins `asked_to_put_aside`,
        // which the next line is what performs — so this has to run first or a
        // person's press would wait a whole frame for no reason. §6 of
        // `docs/design/the-external-window-edge.md`.
        // **Every control on the edge reaches a mechanism now.** This used to
        // loop over what came back and say on stderr that the machine had no
        // mechanism for it, which was true of exactly one control: the window
        // menu. It opens, so nothing comes back and there is nothing to say.
        // **What the terminal's shell wrote, taken once a frame.** Here for the
        // same reason as the asks around it: this is the one place that holds
        // both the server and the desktop, and the read must happen whether or
        // not anything is drawn — a command's output has to accumulate in the
        // grid while a person is looking at something else, or switching back
        // to the terminal would show a gap.
        //
        // **It never blocks.** The pseudoterminal's master end is
        // non-blocking, so a shell with nothing to say costs one read that
        // finds nothing. Whether anything changed is read and not acted on
        // yet: the frame is drawn every time regardless, and the answer is
        // what a later change will use to skip redrawing a terminal nobody has
        // typed in.
        let _ = server.the_terminal_was_written_to();

        // **And what its shell said about its prompt**, drained in the same
        // breath because the marks arrive in the bytes that read took.
        //
        // Empty on nearly every frame: a shell with no integration snippet
        // installed never sends a mark, and one that has it sends a handful per
        // command rather than per frame.
        //
        // **A typed line is a person asking what a command will do**, and the
        // only thing that produces one is their own key press — `alo` is not
        // watching the line editor, and there is no mark between a prompt and
        // somebody asking. `docs/contracts/shell-integration-marks.md`.
        for request in
            crate::what_alo_asks_about_a_command::the_requests_in(&server.what_the_shell_said())
        {
            self.asking.ask(
                crate::what_a_person_asked_goes_to_the_agent::WhoAsked::TheTerminal,
                &request,
                &the_folders_to_send(server),
            );
        }
        server.meet_what_was_pressed_on_an_edge();
        // **And the applications a person asked to start**, met here for
        // `asked_to_put_aside`'s reason and one of its own: sending `Activate`
        // waits for the bus to start an application, and the press that asked
        // for it was in the input loop. See `Server::asked_to_start`.
        // **And a pinned list a person changed, written before the next
        // frame.** Here for the same reason as the asks above: the press that
        // changed it was in the input loop, and this is where the desktop that
        // owns the person's folder is reachable.
        //
        // **Not at sign-out.** A machine loses power and a session is killed; a
        // pin somebody made an hour ago should not be the thing that is lost.
        if let Some(owed) = server.take_the_pins_owed_to_the_disk() {
            self.desktop.keep_these_pins(&owed);
        }
        for not_started in server.start_what_was_asked_for_now() {
            // **Said rather than dropped**, the same rule the edge's menu
            // follows two blocks up: a press that reached here and vanished is
            // what a person reports as *the icon does nothing*, so the machine
            // says so. Once per press, because the list is drained.
            //
            // The two causes read alike here and are different: this machine
            // was never given a way to start anything, or the application is
            // not one the bus can start. `Server::asks_an_application_to_start`
            // is where the first is decided, and both end as a refusal rather
            // than as a fallback — `alo_portals::opening` refuses to read an
            // `Exec` line and so does this.
            eprintln!(
                "alo-shell: {} was asked to start and this machine could not start it",
                not_started.name()
            );
        }
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
    /// The record is carried while it is open — Super+R and a row in Settings
    /// open it, since 2026-10-27. A question is not carried yet: nothing on a
    /// machine opens the full-screen approval surface, and a frame that
    /// reserved room for it would be drawing for a state that cannot arrive.
    /// It is the second `Option` in `frame_pictures` when something does.
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
        // **A region chosen in Settings reaches the clock on the next frame**,
        // before the readings are taken, so the first clock drawn after the
        // choice is already written the new way.
        if let Some(formats) = server.the_formats_to_tell() {
            self.desktop.the_formats_are_now(&formats);
        }
        // Asked before the frame is made, never after: a frame drawn from
        // readings taken after it would show a person the moment before.
        self.desktop.refreshed();
        // **And what this seat has made of their input, on the same frame.**
        // Beside the readings because it is one of them: how much of its own
        // input a machine understood is a reading about that machine, and the
        // one a person has when a press does nothing. It changes only when
        // somebody presses something, so whoever acts on it is not woken by a
        // moving clock.
        self.desktop
            .what_the_seat_made_of_its_input(server.what_the_seat_made_of_its_input());
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
        let on_the_dock = server.what_this_persons_dock_holds();
        // **And the longer list *More apps* shows**, for `on_the_dock`'s reason
        // exactly: the frame borrows it, so it has to outlive the frame.
        let in_more_apps = server.what_this_persons_more_apps_holds();
        // **A local, for `on_the_dock`'s reason exactly.** The frame borrows
        // which of the Dock's surfaces is open, and the borrow must not be of
        // the `Server` — this lane calls `&mut server` again further down, to
        // record where the Dock and the fixed controls were drawn.
        let the_dock_has_open = server.what_the_dock_has_open();
        // **Said here because the drawing path may not word a sentence**, and a
        // local for `windows`' reason exactly: the frame borrows it.
        // `tests/desktop_source.rs` holds that no `desktop_*` or `dock_*` file
        // names `Word::` or `.say(` — every sentence a person meets comes from
        // `alo-saying`, and a raster that looked one up would be a second place
        // deciding what a person reads.
        // **Said after the Dock's state is worked out**, because what the
        // picker says depends on which application is open and how many windows
        // it has — a bundle said before that would have to be said again.
        //
        // **And what they typed, with how many rows are left.** Both from the
        // one filtered list above: *nothing matched* is a fact about the text
        // and the rows together, and asking the two in two places is how a
        // surface comes to say *nothing here matches* over a full grid.
        let the_docks_words =
            crate::the_docks_words::TheDocksWords::said(self.strings, &the_dock_has_open)
                .and_what_they_typed(
                    self.strings,
                    server.what_was_typed_into_more_apps(),
                    in_more_apps.len(),
                );
        // **What somebody asked for becomes a task, here, once a frame — and
        // before the frame.** Asking the desktop needs it mutably and `now()`
        // borrows it immutably for as long as the frame lives, so this cannot
        // sit lower down.
        // `a_person_asked_alo` recorded the words; this is the moment they
        // become a window. Taken rather than read, so one Enter opens one
        // window however many frames pass before this runs.
        //
        // **A request the desktop cannot answer is dropped, and that is
        // deliberate.** `TheDesktop::a_task_for` says `None` on a machine where
        // nobody has chosen what answers their questions, and a window opened
        // onto a model that is not there would be worse than none: a person
        // would type into a surface that could never reply. The request is not
        // queued either — queueing it would mean a window appearing later for
        // something they typed and gave up on.
        // **What a person pasted, put in the field, here.** Asked once a frame
        // and almost always answering with nothing — the selection belongs to
        // another client, which writes when it gets round to it, so the press
        // of Control+V and the characters arriving are different frames. See
        // `crate::what_is_on_the_clipboard` for why none of it waits.
        let _ = server.that_paste_arrived();

        // **And a provider a person pressed Enter on is saved, here.** Taken
        // rather than read, so one Enter saves once however many frames pass;
        // and asked of the desktop rather than done, because saving is two
        // writes to this machine and this crate shows rather than measures.
        //
        // **The form is closed only when it worked.** A keyring that is locked
        // or a settings file that will not read both leave what they typed on
        // the screen — which is the one moment a person most needs their key
        // to still be there, because they cannot see it to type it again.
        // **Both arms, because the `&&` here swallowed a refusal.** It read
        //
        //     if let Some(saving) = take_the_provider_to_save()
        //         && self.desktop.a_provider_was_filled_in(&saving) { .. }
        //
        // and short-circuited: a save that failed did nothing at all — the form
        // was not told, nothing was drawn, and the provider had already been
        // taken. A person pressed Enter and the machine answered with silence.
        if let Some(saving) = server.take_the_provider_to_save() {
            match self.desktop.a_provider_was_filled_in(&saving) {
                Ok(()) => server.that_provider_was_saved(),
                Err(why) => server.that_provider_was_not_saved(why),
            }
        }

        for request in server.take_what_was_asked_of_alo() {
            if let Some(task) = self.desktop.a_task_for(&request) {
                let which = server.tasks_mut().and_now_this_one(task);
                // **The window a person just asked for takes the keyboard**, the
                // way any window they open does. The Bar let go of it when they
                // pressed Enter.
                server.this_task_took_the_keyboard(which);
                // **And the request goes to the agent**, which until ADR 0109
                // nothing did: a person pressed Enter, a window opened, and
                // that was the end of the road. `ask` hands it to a thread and
                // returns at once — a frame loop that waited here would stop
                // every window on the machine drawing until a model answered.
                self.asking.ask(
                    crate::what_a_person_asked_goes_to_the_agent::WhoAsked::ATaskWindow(which),
                    &request,
                    &the_folders_to_send(server),
                );
            }
        }
        // **What a person pressed Install on becomes an installation, here,
        // once a frame — and before the frame**, for the reason the ask above
        // is: starting one needs the desktop mutably (for its indicator) and
        // `now()` borrows it immutably for as long as the frame lives.
        //
        // **Taken rather than read**, so one press starts one installation
        // however many frames pass. `Server::asked_to_install` carries the
        // whole argument, and the short of it is that the window goes on
        // showing what was asked for until the host answers — so a lane reading
        // the window's state would start the same download every frame.
        if let Some(listed) = server.take_what_was_asked_to_install() {
            let refused = match (self.installing.as_ref(), self.desktop.the_indicator()) {
                (Some(road), Some(indicator)) => road
                    .asked_for(&listed, indicator, std::time::SystemTime::now())
                    .err(),
                // **A session with no road or nowhere to show the errand
                // installs nothing**, and the person is told rather than left
                // watching a row say *installing it* for ever. Law 1 is why
                // the missing indicator refuses rather than proceeds: an
                // errand that cannot be shown may not happen.
                //
                // Worded by `alo-software` like every other refusal here. The
                // tool answering nothing is the truest of the shapes it has
                // for *this machine could not*, and the English beside it says
                // which half was missing.
                (None, _) | (_, None) => Some(alo_software::Stopped::Refused(
                    alo_software::NotDone::DidNotAnswer {
                        said: "this session has no tool to install with, or nowhere to show the \
                               errand on"
                            .to_owned(),
                    },
                )),
            };
            if let Some(why) = refused {
                server
                    .market
                    .it_was_refused(listed.application().identifier(), why);
            }
        }
        // **And whatever has become of the installations already going.**
        // Drained once a frame, each taking its own line off the indicator, and
        // the window told so a row stops saying *installing it*.
        //
        // The borrow is taken and dropped around the loop rather than held
        // across it, because the indicator is the desktop's and the window is
        // the server's — and nothing here needs both at the same moment.
        let became = match (self.installing.as_ref(), self.desktop.the_indicator()) {
            (Some(road), Some(indicator)) => road.whatever_became_of_them(indicator),
            (None, _) | (_, None) => Vec::new(),
        };
        for what in became {
            match what.outcome {
                Ok(installed) => server
                    .market
                    .it_arrived(installed.application().identifier()),
                Err(why) => server.market.it_was_refused(&what.identifier, why),
            }
        }
        // **And whatever has come back since the last frame**, which is why
        // asking could return at once. Drained rather than read, so one answer
        // reaches one window once.
        for came in self.asking.whatever_came_back() {
            match came.who {
                crate::what_a_person_asked_goes_to_the_agent::WhoAsked::ATaskWindow(which) => {
                    let daemon = server
                        .settings_places
                        .as_ref()
                        .map(|places| alo_changing::TheDaemonsDoor::at(places.daemon()));
                    server.an_answer_came_back(
                        which,
                        came.said,
                        came.the_machines_fault,
                        came.the_turn,
                        daemon
                            .as_ref()
                            .map(|door| door as &dyn alo_changing::Answering),
                        self.strings,
                    );
                }
                // **An explanation of a command, into the terminal that asked.**
                //
                // A machine that could not be asked draws **nothing**, rather
                // than a sentence about the machine where a person expects a
                // sentence about their command. The terminal has no second
                // place to put it — a task window has `Act::done` beside the
                // conversation and this does not — and an error wearing the
                // shape of an explanation is worse than silence here, because
                // the person is deciding whether to press Enter.
                crate::what_a_person_asked_goes_to_the_agent::WhoAsked::TheTerminal => {
                    if !came.the_machines_fault
                        && let Some(what) =
                            crate::what_alo_asks_about_a_command::what_came_back(&came.said)
                    {
                        server.alo_explains(what);
                    }
                }
            }
        }
        let mut frame = self.desktop.now();
        let named = FrameTarget::metadata(target)?.name;
        // **Placed once, and after the frame, because the reading direction is
        // the look's.** A task window
        // that had nowhere to be gets somewhere now; one that already has a
        // position is left exactly where it is, which is the design's own
        // requirement — *later redraws must not unexpectedly reposition it.*
        //
        // The view is the display rather than the camera's, because a task
        // window is drawn against the display like the two desktop windows
        // beside it. `crate::where_a_task_window_opens` records that, and
        // records that whether it ought to live on the canvas instead is a
        // question the owner has not answered.
        let the_view = smithay::utils::Rectangle::new((0, 0).into(), (size.w, size.h).into());
        server
            .tasks_mut()
            .put_whatever_has_nowhere_to_be(the_view, frame.look.reading(), &[]);
        // **Said once a frame, and kept alive for as long as the frame is.**
        // The raster borrows these strings.
        let the_tasks_words = crate::the_tasks_words::TheTasksWords::said(
            server.tasks().each(),
            server.tasks().where_each_went(),
            self.strings,
            server.the_change_waiting.selected(),
        );
        let the_task_windows = the_tasks_words.as_displayed();
        // **A copy of the market window for the frame, and only while it is
        // open.**
        //
        // `MarketShows` borrows the window, which lives on the `Server`, and
        // this lane calls `&mut server` again after the frame is drawn — so a
        // borrow taken here would stop those calls compiling. The task words
        // above answer the same problem the same way: the lane holds something
        // it owns and the frame borrows that.
        //
        // **Copied rather than taken out and put back.** Taking it would put a
        // person's open window on a local for the length of a draw, and any `?`
        // between the take and the put-back would lose it — the window would
        // close itself on a frame that failed to render. A copy cannot do that
        // whatever goes wrong below.
        //
        // **And nothing is copied on an ordinary frame.** `is_open` is false
        // until a person presses the Applications entry on their own Dock,
        // which is most machines most of the time.
        let the_market_window = server.market.is_open().then(|| server.market.clone());
        // **Said once a frame and kept alive as long as the frame is**, for the
        // task words' reason exactly: the raster borrows these strings.
        let the_markets_words = crate::TheMarketsWords::said(self.strings);
        let the_market = the_market_window.as_ref().and_then(|window| {
            let showing = window.shows();
            crate::where_the_market_opens::where_the_market_opens(
                // **The same rectangle as the task windows are placed in, told
                // in the pixels a frame draws into.**
                // `where_a_task_window_opens::as_a_frame_draws_it` is one
                // number to one number and says why: nothing in this shell
                // applies a display scale yet, and a conversion invented here
                // would be the single scaled figure in an unscaled geometry.
                // Reusing it rather than building a second rectangle of the
                // same numbers, so there is one place to change when the shell
                // does scale.
                crate::where_a_task_window_opens::as_a_frame_draws_it(the_view),
                &showing,
                frame.look.scale(),
            )
            .map(|at| crate::market_raster::AMarketOnTheDisplay {
                at,
                showing,
                words: &the_markets_words,
            })
        });
        // **The alo Bar, worked out once a frame and kept alive beside the
        // market's words**, for the same reason: the raster borrows these
        // strings and the frame outlives this block.
        //
        // **Open means the composer has the keyboard and something is in it.**
        // An empty field is the Dock's own prompt and no panel — a list that
        // appeared the moment somebody pressed the chord would flash every
        // application on the machine under their first keystroke.
        let typed_into_the_bar = server.composer().typed().to_owned();
        let bar_is_open =
            server.composer().has_the_keyboard() && !typed_into_the_bar.trim().is_empty();
        let found = if bar_is_open {
            crate::the_bar_finds::what_the_bar_finds(
                &typed_into_the_bar,
                &server.what_this_machine_has,
                self.strings,
            )
        } else {
            Vec::new()
        };
        // **The line under the field, and what it says when nothing can answer.**
        //
        // A person typing a question on a machine where nothing is chosen to
        // answer one used to be told nothing at all: the request was taken on
        // Enter, `a_task_for` said `None`, and the `if let` further down had no
        // `else`. They met silence, pressed again, and met silence again.
        //
        // Asked here rather than at the drop, because here the Bar is still on
        // the screen and they are still typing — and because telling them
        // afterwards would need a notification alo OS cannot make (see
        // `TheDesktop::anything_is_chosen_to_answer` for the measurement).
        //
        // It replaces the count rather than joining it. The applications it
        // found are in the rows directly below, where a person can see them; the
        // one fact those rows cannot carry is that a question will not be
        // answered.
        let where_from = if !bar_is_open {
            String::new()
        } else if self.desktop.anything_is_chosen_to_answer() {
            crate::the_bar_finds::where_these_came_from(found.len(), self.strings)
        } else {
            crate::the_bar_finds::nothing_can_answer_a_question(self.strings)
        };
        let the_bar = bar_is_open
            .then(|| crate::where_the_bar_opens::where_the_bar_opens(size))
            .flatten()
            .map(|places| crate::bar_raster::TheBarOnTheDisplay {
                places,
                typed: &typed_into_the_bar,
                origin: &where_from,
                results: &found,
                chosen: server.which_row_in_the_bar,
                display: size,
            });
        // **An `Arc` of its own, lent to the frame.** The frame holds the
        // reference for as long as it lives and this lane uses the server
        // mutably afterwards, so the borrow cannot come from the server here.
        // One refcount per frame, never a copy of every icon.
        let what_each_application_draws =
            std::sync::Arc::clone(&server.what_each_application_draws);
        what_only_the_server_knows(
            &mut frame,
            server,
            crate::what_the_server_tells_a_frame::WhatTheLaneBuilt {
                windows: &windows,
                on_the_dock: &on_the_dock,
                in_more_apps: &in_more_apps,
                the_dock_has_open: &the_dock_has_open,
                the_docks_words: &the_docks_words,
                what_each_application_draws: &what_each_application_draws,
                tasks: &the_task_windows,
                market: the_market,
                the_bar,
            },
            &named,
        );
        // **Read back from the frame rather than asked again.** The scale the
        // edges are drawn at has to be the scale everything else on this
        // display is drawn at; asking `the_scale_of_display` a second time
        // would be a second answer to one question, and the two could differ
        // for one frame after a person changes it.
        let this_display = (frame.display_scale, (size.w, size.h));
        // **And whether the desk itself changed since the last frame.**
        // `more-than-one-display-plan.md` task 13. Until this line
        // `discover_every_atomic_output` was asked **once**, as the session
        // started: plugging a monitor in left it dark until the next sign-in,
        // and unplugging one left the session drawing to a screen that was no
        // longer there.
        // **Here, before the pictures are made**, so a display that has just
        // arrived gets this frame's desktop, this frame's fixed controls and
        // this frame's render rather than waiting for the next one — and so a
        // display that has gone is out of `others` before anything tries to
        // draw to it. The arrangement is then asked again by the block near
        // the end of this function, because a presented display changes what
        // `the_displays_as_reported` answers and that block already acts on a
        // change. Nothing new decides when an arrangement is made.
        // **Two questions rather than one, and that is the design.**
        // `asked_of` is one cheap query of the connectors and is allowed to be
        // over-eager; `the_desk_is_reconciled` runs the real discovery and is
        // authoritative. A port that appears and wins no CRTC costs one
        // discovery and changes nothing, which is the right way round: the
        // alternative is two answers to *what is plugged in* that can
        // disagree.
        // A device that will not answer leaves the desk exactly as it was. An
        // empty answer and a failed question are not the same thing — read as
        // *no displays*, one refusal would retire every screen the person is
        // looking at.
        let plugged_in = crate::a_screen_arrives_or_leaves::asked_of(
            &crate::drm_inventory::Inventory(self.device),
        );
        if let Ok(plugged_in) = plugged_in
            && matches!(
                self.plugged_in.compared_with(plugged_in),
                crate::a_screen_arrives_or_leaves::TheDesk::Changed { .. }
            )
        {
            // The refusals are carried on the result rather than returned: a
            // display that could not be painted is a display this machine does
            // not have, and it must not take the session down with it.
            let desk = crate::the_desk_is_reconciled::the_desk_is_reconciled(
                server,
                self.device,
                &named,
                others,
            );
            // **Said rather than dropped**, and once per change rather than
            // once per frame — this whole block only runs when the desk really
            // changed, which is what makes reporting here cheap.
            //
            // It was `let _desk = …` while this branch was parked. A result
            // carrying refusals that nothing reads is a display that vanishes
            // with no account of why, which is the one thing the comment above
            // does not license: *it must not take the session down* is an
            // argument for carrying on, not for silence.
            //
            // A screen arriving is also something a **person** should be told
            // about once — `docs/features.md` promises *alo OS says which desk
            // it thinks you are at, once*, and `alo_displays::notes` holds
            // those sentences. This is not that: it is the machine's own log,
            // for whoever is reading one, and the person's half is that
            // promise's to keep.
            for presented in &desk.presented {
                eprintln!("alo-shell: {presented} arrived and is being drawn to");
            }
            for retired in &desk.retired {
                eprintln!("alo-shell: {retired} is gone and has been withdrawn");
            }
            for refused in &desk.refusals {
                eprintln!(
                    "alo-shell: {refused} is reported by this machine and could not be drawn \
                     to, so it has been left out and the screens already drawn to are unchanged"
                );
            }
        }
        // **The terminal is made to fit its window before it is drawn**, and
        // here rather than in the input lane because this is where the display
        // is known: the grid is the transcript's room divided by the cell, and
        // the cell comes from the face at *this display's* scale.
        //
        // Nothing happens on the frames where the answer has not changed,
        // which is every frame but the ones where the window, the font size or
        // the scale moved — and that third one is what happens when a laptop is
        // plugged into another monitor.
        let _ = server.the_terminal_fits_its_window(
            &mut self.labels.fonts,
            (size.w, size.h),
            f64::from(server.the_scale_of_display(&named)) / 100.0,
        );
        // **alo's own terminal, drawn here and not inside `frame_pictures`.**
        // Two rules put it here. That file is held to *no string a person could
        // read* by `desktop_source`, so the sentence below has nowhere to live
        // there; and a terminal that cannot be drawn must be **left out rather
        // than refuse the frame**, which is `DesktopPictures::tasks`' own rule
        // and for its reason — a terminal too small for its own layout is one
        // window's problem, and refusing the frame would take the whole desktop
        // down with it, including the Dock a person needs to do anything about
        // it.
        let terminal = drawn_terminal(
            server,
            self.labels,
            (size.w, size.h),
            &named,
            frame,
            self.strings,
        );
        // **The record window, while it is open**, in this frame and on
        // this display. A chord or a row in Settings opens it.
        let pictures = crate::nested_desktop::frame_pictures(
            frame,
            server.the_records_frame(frame.strings, frame.look),
            None,
            self.labels,
            (size.w, size.h),
            terminal,
        )?;
        // **A window this person has open that this frame did not draw.**
        //
        // **Here, and not where the placement is made.** It stood beside
        // `where_the_market_opens` until 2026-10-19 and asked whether the
        // *placement* was missing — one step too early. The picture is built
        // further down, by `frame_pictures`, which can refuse on its own and
        // which until this change dropped the market outright. So a window
        // that opened, placed, and was never drawn reported **nought windows
        // undrawn**, and `Server::the_market_is_taking_the_keys` read that
        // nought and let the invisible window keep the keyboard.
        //
        // Measured on a booted machine that day, with a control each way: the
        // press took the button count from 3 to 4, the screen stayed at 9323
        // bytes, a further key left the key count at 6 — and the sentence
        // still said nought. A guard that cannot see the fault it was written
        // for is worse than none, because it reads as evidence.
        //
        // `pictures.desktop.market` is what the painter reads, so it is what
        // this asks. Nought is the ordinary answer and the healthy one.
        let open_and_undrawn = server.market.is_open() && pictures.desktop.market.is_none();
        server.a_frame_left_this_many_windows_undrawn(usize::from(open_and_undrawn));
        // **Every open window's edge, which nothing on a machine drew until
        // now.** Built after the pictures because both want the font system
        // mutably and neither outlives this frame. See
        // `crate::every_windows_edge` for why it is per display.
        let edges = crate::every_windows_edge::every_windows_edge(
            server,
            self.labels,
            this_display.1,
            u32::from(this_display.0),
        );
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
            // **An `Arc` of its own, lent to the frame.** The frame holds the
            // reference for as long as it lives and this lane uses the server
            // mutably afterwards, so the borrow cannot come from the server here.
            // One refcount per frame, never a copy of every icon.
            let what_each_application_draws =
                std::sync::Arc::clone(&server.what_each_application_draws);
            what_only_the_server_knows(
                &mut frame,
                server,
                crate::what_the_server_tells_a_frame::WhatTheLaneBuilt {
                    windows: &windows,
                    on_the_dock: &on_the_dock,
                    in_more_apps: &in_more_apps,
                    the_dock_has_open: &the_dock_has_open,
                    the_docks_words: &the_docks_words,
                    what_each_application_draws: &what_each_application_draws,
                    // **None on an extra display, and that is a named gap.** A
                    // task window belongs to the display it was placed on, and
                    // nothing records which that was — the position is one
                    // rectangle per task, not one per task per display. Handing
                    // this list over here would draw the same window on every
                    // display at the same coordinates, which is worse than
                    // drawing it on one: a person would have two of a window
                    // they can only type into once.
                    //
                    // So a task window is on the first display today. Making it
                    // belong to a display is a change to what `TheTasks`
                    // stores, and it is written down rather than guessed at.
                    tasks: &[],
                    // **None on an extra display**, for the reason above and
                    // one more: there is exactly one of these windows per seat,
                    // so drawing it on every display would put a person's
                    // window on two screens at once with one of them
                    // unreachable. It is on the display the loop holds, which
                    // is where Settings is for the same reason.
                    // **None on an extra display**, for the market's reason
                    // and the same one: a person is typing into one Bar, and
                    // drawing the panel on every display would put the rows
                    // they are choosing from on screens their keyboard is not
                    // on. It opens on the display the loop holds.
                    the_bar: None,
                    market: None,
                },
                &named,
            );
            // Read off the frame before it is given away, for the reason the
            // first display's own note gives: one answer to one question.
            let its_scale = frame.display_scale;
            // **This binding's name is load-bearing**, and was `laid_out`
            // until 2026-10-08. `the_recheck_has_a_caller`'s guard reads the
            // draw's text and asks that every control handed to the rule is
            // read from a `pictures` binding rather than from a constant — the
            // one thing the compiler cannot ask, because a literal rectangle
            // satisfies `E0063` perfectly. A name that does not say `pictures`
            // fails that guard, which is the guard working: rename this and
            // either give it a name that still says what it is, or say in that
            // test where the rectangles now come from. Do not loosen it.
            let its_terminal = drawn_terminal(
                server,
                self.labels,
                (size.w, size.h),
                &named,
                frame,
                self.strings,
            );
            let its_pictures = crate::nested_desktop::frame_pictures(
                frame,
                server.the_records_frame(frame.strings, frame.look),
                None,
                self.labels,
                (size.w, size.h),
                its_terminal,
            )?;
            // **This display's own edges**, at this display's own scale. One
            // list shared with the display above would be right for whichever
            // it was built against and silently wrong here — an edge is pixels
            // and `EdgePicture::of` converts to them once, against one output.
            let its_edges = crate::every_windows_edge::every_windows_edge(
                server,
                self.labels,
                (size.w, size.h),
                u32::from(its_scale),
            );
            theirs.push((other, its_pictures, its_edges));
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
        for (other, its_pictures, _its_edges) in &theirs {
            let Ok(its) = FrameTarget::metadata(*other) else {
                continue;
            };
            // **Where this display's Dock is, for the press that asks later.**
            // Beside the fixed controls and for the same reason: the slots are
            // computed in the draw and nowhere else, so outside a frame they do
            // not exist. Every display, because a person may have a Dock on each
            // and the pointer's location is global.
            if let Some(dock) = its_pictures.desktop.dock.as_ref() {
                server.the_docks_slots_were_drawn(&its.name, dock.standing.clone());
            }
            // **And where its open surface was, or that there was none.**
            // Outside the `if let` above deliberately: the two cases that must
            // clear the record are *no Dock on this display* and *a Dock with
            // nothing open*, and a call inside that block would miss the first
            // and leave a press being taken by a popover that is gone.
            server.the_docks_surface_was_drawn(
                &its.name,
                its_pictures
                    .desktop
                    .dock
                    .as_ref()
                    .and_then(|dock| dock.overflow_list),
            );
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
        if let Some(dock) = pictures.desktop.dock.as_ref() {
            server.the_docks_slots_were_drawn(&named, dock.standing.clone());
        }
        // The same for this display, and outside the `if let` for the same
        // reason the one above is.
        server.the_docks_surface_was_drawn(
            &named,
            pictures
                .desktop
                .dock
                .as_ref()
                .and_then(|dock| dock.overflow_list),
        );
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
        // **And a window that has just opened is given somewhere to be.**
        //
        // `docs/design/where-a-new-window-opens.md` is the owner's
        // specification and `crate::where_a_window_opens` implements it; until
        // this line it had no caller, so `new_toplevel`'s Place-and-no-position
        // stood and **the second window opened exactly on the first**. That
        // breaks `docs/features.md`'s `[v0.01] ★ Every goal is a canvas` —
        // *nothing is stacked* — on the second window, which is why this is an
        // existing promise made true rather than a feature added.
        //
        // **Here, and the position in this function is the whole argument.**
        // Three things have to have already happened and one must not have:
        //
        // - the buffer has arrived, so the window has an extent. `server.dispatch()`
        //   ran before `present` and that is where `commit` sets `mapped`.
        // - a window being **restored** has already been put back, a hundred
        //   lines above. So a remembered place wins and arrives here already
        //   placed — the contract's rules 1 and 2, kept by order rather than by
        //   a condition.
        // - **this frame's** fixed controls are recorded, by the two calls just
        //   above. At the top of `present` they would be the previous frame's,
        //   and on the first frame there would be none at all — a window placed
        //   against controls that had not been laid out yet.
        // - and the frame has not been composited: `render_each_display` is
        //   below. So the window is drawn where it belongs on its **first**
        //   frame and a person never sees it at the origin.
        //
        // It is beside `bring_back_frames_the_moved_controls_hide` on purpose:
        // that asks *the controls moved, is an open window now unreachable*,
        // and this asks *a window arrived, where does it go* — the same
        // question about a different thing changing, and both need this frame's
        // bounds.
        //
        // The count is dropped rather than read: a frame on which nothing
        // opened places nothing, which is almost every frame and is not news.
        // `crates/alo-shell/src/a_new_window_is_placed_tests.rs` reads it.
        let _placed =
            server.place_every_window_that_just_opened(&named, size, frame.look.reading());
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
            // **Read off the server**, which is where the form lives. A
            // literal `None` here would draw the rows behind a form a person
            // is typing into -- the same shape of mistake `market` made at a
            // frame-building site and that cost a release.
            // **The form and what the machine answered about it**, read off
            // the same server: a form drawn without the answer is a form that
            // met a person with silence.
            server.a_provider_is_being_filled_in().map(|form| {
                crate::settings_raster::AFormOnScreen {
                    form,
                    not_saved: server.why_the_provider_was_not_saved(),
                }
            }),
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
                settings: Some(&settings),
                ..pictures.layers(&edges)
            },
        };
        //
        // The loop's own display goes first, because discovery put it first
        // and because a refusal on a later one must not delay it.
        let mut painted: Vec<Layered<'_>> = theirs
            .iter_mut()
            .map(|(other, its_pictures, its_edges)| Layered {
                target: *other,
                layers: its_pictures.layers(its_edges),
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

/// alo's own terminal, drawn for one display, or left out with a reason said.
///
/// **One function rather than the same six lines at two call sites**, which is
/// `crate::what_the_server_tells_a_frame`'s own rule about this very lane: it
/// builds a frame twice, once for the display the loop holds and once per other
/// display, and *two of them in two files is how they drift apart*.
///
/// **A terminal that cannot be drawn is left out rather than refusing the
/// frame.** `DesktopPictures::tasks` states that rule and the reason: one
/// window too small for its own layout is one window's problem, and refusing
/// the frame would take the whole desktop down with it — including the Dock a
/// person needs in order to do anything about it. What went wrong is said on
/// the error stream rather than swallowed, because *never a silent fallback*.
///
/// **And it is drawn here rather than in `frame_pictures`** because
/// `desktop_source`'s `the_desktop_adds_no_number_of_its_own` holds that file
/// to *no string a person could read*. The sentence below has nowhere to live
/// there, and the rule is right: words belong where words are owned.
pub(crate) fn drawn_terminal(
    server: &Server,
    labels: &mut crate::WindowControlLabels,
    size: (i32, i32),
    named: &str,
    frame: crate::DesktopFrame<'_>,
    says: &alo_strings::Strings,
) -> Option<crate::the_terminals_cells_are_drawn::TerminalPicture> {
    let terminal = server.alos_own_terminal()?;
    let screen = terminal.screen();
    let scale = f64::from(server.the_scale_of_display(named)) / 100.0;
    match crate::the_terminals_cells_are_drawn::picture(
        &mut labels.fonts,
        crate::the_terminals_cells_are_drawn::Shows {
            screen,
            what_alo_said: terminal.what_alo_said(),
            a_test_result: terminal.a_test_result(),
            a_benchmark: terminal.a_benchmark(),
            a_diff: terminal.a_diff(),
        },
        size,
        scale,
        frame.look.scheme(),
        says,
        server.what_alos_own_terminals_shell_is_called().as_deref(),
    ) {
        Ok(drawn) => Some(drawn),
        Err(why) => {
            eprintln!("alo-shell: alo's own terminal could not be drawn on {named}: {why}");
            None
        }
    }
}
