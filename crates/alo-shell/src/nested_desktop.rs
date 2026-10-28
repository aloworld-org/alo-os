//! The ordinary desktop on the nested compositor: the frame every other surface
//! sits in.
//!
//! [`Nested::submit_with_desktop`] draws a session's frame with the dock on the
//! edge `alo-dock` names and its status area at the far end, the egress
//! indicator growing from that status area, the two desktop windows when a
//! person has them open, and — above all of those — the record window and a
//! waiting question when there are any. The dock and the indicator are laid out
//! from **one** `alo_dock::Dock`, so the status area and the indicator cannot
//! disagree about where the far end is.
//!
//! [`Nested::pump_running`] and [`Nested::pump_filling`] take the parent
//! window's keys through the seat for whichever desktop window is open.

use alo_strings::Strings;
use smithay::backend::{
    input::{Event, InputEvent, KeyboardKeyEvent},
    winit::WinitEvent,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::desktop_raster::{DesktopPicture, filling_shows, running_shows};
use crate::egress_status_raster::EgressStatusPicture;
use crate::nested_egress_status::status_picture;
use crate::{
    ApprovalFrame, DesktopLook, EgressStatus, EgressStatusFrame, FillingPressed, FillingWindow,
    FrameTarget, InputError, Nested, RecordFrame, RenderError, RunningPressed, RunningWindow,
    Server, WindowControlLabels,
};

/// Where a nested frame reads alo's own terminal from, when one should be drawn.
///
/// **This exists because the nested backend drew no terminal at all until
/// 2026-10-17.** [`Nested::submit_with_desktop`] passed `None` for the terminal
/// picture, so a terminal a chord had opened was taking keys and never reaching
/// a frame — while `direct_desktop` drew it from the same helper. (*Drew* was
/// wrong: it built the picture and never handed it to the painter, which stayed
/// true until 2026-10-27 — see `DesktopPictures::layers` below.) The effect was
/// that **no gate could ever see alo's own terminal**, because the only backend
/// a build machine can run is this one, and the four promises the terminal keeps
/// were checkable on real hardware and nowhere else.
///
/// The terminal is read from the [`Server`] rather than handed in as a picture
/// because the picture type is this crate's own: a caller outside it could not
/// build one, which is why the parameter could not simply be opened up.
pub struct TheTerminalToDraw<'a> {
    /// The server whose chord opened the terminal.
    pub server: &'a crate::Server,
    /// The display this frame is for, as its appearance names it. A terminal is
    /// laid out against one display's size and scale, so the name is not
    /// decoration.
    pub display: &'a str,
}

/// What a frame needs to draw the ordinary desktop.
#[derive(Clone, Copy)]
pub struct DesktopFrame<'a> {
    /// The person's dock, which decides the edge and the status area.
    pub dock: &'a alo_dock::Dock,
    /// The task windows open on this display, each with the rectangle somebody
    /// already placed it at.
    ///
    /// **An empty slice is the true answer on a machine nobody has asked
    /// anything**, not a placeholder — the same seam `windows` takes, and for
    /// the same reason: only the caller knows which display's windows these
    /// are.
    pub tasks: &'a [crate::task_raster::ATaskOnTheDisplay<'a>],
    /// The window of what a person can install, when it is open on this
    /// display.
    ///
    /// **[`None`] is a shut window and also a display it is not on**, and the
    /// two are the same thing to a drawing: there is one of these windows per
    /// seat, so at most one display shows it. Which one is the caller's to
    /// know, exactly as `tasks` and `windows` above are.
    pub market: Option<crate::market_raster::AMarketOnTheDisplay<'a>>,
    /// The person's appearance at this moment, and the way they read.
    pub look: DesktopLook,
    /// The vocabulary the person reads.
    pub strings: &'a Strings,
    /// What the egress indicator in the status area was last told.
    pub egress: &'a EgressStatus,
    /// The window of what is running, open or closed.
    pub running: &'a RunningWindow,
    /// The window of what is filling the disk, open or closed.
    pub filling: &'a FillingWindow,
    /// What on this machine is watching or listening, as `alo-in-use` read it
    /// off the media server. Empty while nothing is, which draws nothing.
    pub in_use: &'a [alo_in_use::Line],
    /// The notifications to show, as `alo-notifying` handed them over. A
    /// notification a locked, shared or quiet machine is holding never becomes
    /// one of these, so there is nothing here to decide.
    pub notifications: &'a [alo_notifying::Shown],
    /// The capture tools, while somebody is choosing what to capture or marking
    /// what they captured, and [`None`] when nobody is.
    pub capturing: Option<crate::capture_raster::Capturing<'a>>,
    /// How this display is divided, as `alo-dividing` decided it. Handed in for
    /// the same reason the readings are, and because nothing holds one yet —
    /// the `Server`'s division is task 16's, on the session that stands the
    /// desktop up. A display nobody has divided is `Division::of` its own area.
    pub division: &'a alo_dividing::Division,
    /// What letting go of a dragged window would do, as `alo-dividing`
    /// proposed it, or `Offer::Nothing` while nothing is being dragged.
    pub offer: &'a alo_dividing::Offer,
    /// Every mapped window on this display, in this display's physical
    /// pixels, so the dock can be asked whether one needs the room it sits
    /// in.
    ///
    /// Handed in for the same reason the readings and the division are: the
    /// caller is the only one that knows which display's windows these are,
    /// and what scale puts them in the band's own space. An empty slice is
    /// the true answer on a desktop with nothing open — the dock stays.
    /// `crate::dock_room` holds the rule and
    /// `docs/design/when-the-dock-gives-way.md` the reasoning.
    pub windows: &'a [smithay::utils::Rectangle<i32, smithay::utils::Physical>],
    /// The windows a person put aside, which the panel at the edge shows.
    ///
    /// Handed in for the same reason the rest are. `alo_put_aside::Panel` holds
    /// what is in the panel and **no geometry at all**, deliberately, so the
    /// conversion from a design's figures to a display's pixels happens once in
    /// the crate that knows the display. An empty panel is the true answer on a
    /// desk where nothing has been put aside, not a placeholder.
    pub put_aside: &'a alo_put_aside::Panel,
    /// Whether the put-aside panel is on the screen at this moment.
    ///
    /// **`alo_dock::Revealing::is_revealed`, asked of whoever holds the machine.** It decides
    /// this from the edge, the surface, the keyboard, a drag and an open menu — any one of
    /// which is enough, so leaving one cannot conceal the panel while another still holds it.
    ///
    /// Handed in like the rest, and for the same reason: a compositor that held the machine
    /// would be keeping its own copy of somebody else's answer. A desktop that never advances
    /// it answers `false` for ever, which draws a panel that keeps its column and never its
    /// rail — the honest picture for a desktop with no pointer.
    pub panel_is_revealed: bool,
    /// Whether a window on this display is filling the screen, so the Dock and
    /// the panel give way to it.
    ///
    /// Handed in for the reason the rest are: the caller is the only one that
    /// knows which display these windows are on. A Server answers it with
    /// a_window_is_filling_the_screen.
    pub filling_the_screen: bool,
    /// Everything on the Dock that a person reads, already said.
    ///
    /// **Handed in rather than looked up, because the drawing path may not word
    /// a sentence.** `tests/desktop_source.rs` holds that no file named
    /// `desktop_*` or `dock_*` names `Word::` or `.say(`. The lane says them
    /// once a frame and hands the results over, exactly as it does for a
    /// window's control labels.
    ///
    /// **One bundle rather than five fields**, so that a frame cannot be built
    /// with four of them set: see `crate::the_docks_words`.
    pub the_docks_words: &'a crate::the_docks_words::TheDocksWords,
    /// Whether the list of applications the Dock had no room for is open.
    ///
    /// Handed in for the reason the rest are: a person opened it with a press,
    /// presses reach the `Server`, and a desktop's own state cannot know. The
    /// `Server` answers it with `Server::what_the_dock_has_open`.
    ///
    /// **False is the ordinary frame.** A Dock with room for everything has no
    /// control to open one with, so this is only ever true on a Dock that has
    /// something behind it.
    pub the_dock_has_open: &'a crate::WhatTheDockHasOpen,
    /// What each application draws on the Dock, where it draws anything.
    ///
    /// **Filled from the `Server` by `crate::what_the_server_tells_a_frame`**,
    /// like `market` and `the_bar` beside it, because which file holds an
    /// application icon is a reading of the machine and this crate may not take
    /// one. `TheArtwork::NOTHING` is a Dock of first letters, which is the
    /// decided fallback (`docs/design/the-alo-dock.md`) and not a failure.
    pub what_each_application_draws: &'a alo_dock::artwork::TheArtwork,
    /// **How many physical pixels this display draws for one logical one**, in
    /// hundredths: 100 is one to one, 200 is a dense screen.
    ///
    /// Handed in for the reason the rest are — only the caller knows which
    /// display this is, and `alo_displays::ScreenPlace::scale` is where it comes
    /// from.
    ///
    /// # Until 2026-10-02 a display's scale reached nothing that draws
    ///
    /// `crate::division_raster::picture` has always taken a scale and converted
    /// with it, and **its only call site passed the literal `1`** with a comment
    /// saying *one is the scale this display is laid out at* — true of a one-to-one
    /// screen and of nothing else. `desktop_raster::picture` received a size and
    /// no scale, so it could not pass the real one: the seam was short by an
    /// argument rather than wrong.
    ///
    /// **The failure that leaves is under-conversion, not double conversion.** A
    /// division on a two-times display was drawn at half the room it owns.
    /// Nothing downstream could convert twice, because nothing downstream
    /// received a scale at all — *this lane reported the opposite to the owner
    /// before measuring it.*
    ///
    /// Hundredths rather than a float, matching `alo_displays::Scale`, because a
    /// fractional display scale is a real thing — 125 and 150 are ordinary — and
    /// an integer factor would silently floor them to one.
    pub display_scale: u16,
    /// How many applications the Dock is holding, so its band is the width its
    /// contents need.
    ///
    /// # The same shape as `display_scale` above, one argument further on
    ///
    /// `dock_raster::picture` has always taken a count and sized the band from
    /// it — `Room::a_bar_holding(holding)` — and **its call site passed the
    /// literal `0`**, under a comment saying *nothing in this crate decides what
    /// the Dock holds yet: `alo_dock::Holding` answers that and is not plumbed
    /// into a compositor. A bar holding nothing is narrow, which is true rather
    /// than a placeholder.* **That comment is correct**, and the seam was short
    /// by an argument rather than wrong.
    ///
    /// # What a zero here is, and what it is not
    ///
    /// **Zero is the honest answer today and it is not this field's default.**
    /// Measured 2026-10-08: nothing outside `alo-dock` ever builds a `Holding`,
    /// a `Holding` needs an `alo_dock::Windows` nothing tracked, and
    /// `alo-applications` has no icon concept in seventeen files. So a dock
    /// holding nothing was true, and the 16 logical pixels it draws —
    /// `MARGIN + MARGIN`, with nothing between — were the right 16 pixels.
    ///
    /// **What was wrong was that it could not become anything else.** A constant
    /// where a count belongs does not correct itself when the inputs arrive, and
    /// two of them in two files is how they drift apart.
    ///
    /// # Why the desktop counts and the shell does not
    ///
    /// *The shell shows and never measures.* Which applications are on the Dock
    /// is the person's — pinned, open, or neither — and `Holding::showing`
    /// answers it from a `Windows` whose **order is most recently used first,
    /// across every application**. That order is the whole of *click an app to
    /// return to where you last used it*, and it is session state this crate
    /// does not hold.
    ///
    /// So the desktop builds the list and hands it over, exactly as it hands
    /// over `display_scale` and the four readings.
    ///
    /// **It was a count until 2026-10-10**, and the count was a projection of
    /// this: `Server::how_many_the_dock_holds` built the list, took its length
    /// and threw the rest away. That was the right shape while nothing drew an
    /// icon — the bar's width is all a count can give — and it stopped being
    /// right the day an application with no artwork gained a letter to show.
    ///
    /// The figure the design expects: the Dock's implementation contract at
    /// Figma node `348:28451` gives a 32-pixel glyph in a 44 target, pitched 56
    /// — which is `ICON 48 + GAP 8`, the pitch `alo-dock` already uses. See
    /// `docs/autonomy/updates/the-dock-has-an-implementation-contract.md`.
    pub on_the_dock: &'a [alo_dock::OnTheDock],
    /// What *More apps* lists: every pinned application, then every other one
    /// with a window — **including those whose windows are all minimised and
    /// which are therefore not in [`Self::on_the_dock`] at all.**
    ///
    /// `alo_dock::Holding::in_the_overview`, which is `on_the_dock` before the
    /// membership rule. A separate field rather than something derived here,
    /// because the difference between the two lists is exactly the applications
    /// this one has and that one does not, so it cannot be recovered from that
    /// one.
    ///
    /// The owner's ruling of 2026-10-11, ADR 0103: *More apps is now an
    /// application overview, not merely a list of icons that overflowed.*
    pub in_more_apps: &'a [alo_dock::OnTheDock],
    /// Which slot on the Dock's bar the keyboard is on, if it is on the bar.
    ///
    /// **Server state, like `market`**: it is written by F6 and by Tab, which
    /// are presses, and presses reach the `Server`. A desktop's own state
    /// cannot know it, so `crate::what_the_server_tells_a_frame` fills it and
    /// every other builder of this frame leaves it [`None`] — which is the
    /// true answer for a fixture nobody has pressed F6 on.
    pub the_keyboard_is_on_the_bar: Option<usize>,
    /// The alo Bar, when a person has it open and has typed something.
    ///
    /// **[`None`] is a Bar nobody is using**, which is most frames: the panel
    /// opens over the canvas only while the composer has the keyboard and
    /// something has been typed into it. An empty field shows the Dock's own
    /// prompt and no panel at all.
    ///
    /// **Server state, like `market`** — what was typed and what it found are
    /// decided by presses, and presses reach the `Server`.
    pub the_bar: Option<crate::bar_raster::TheBarOnTheDisplay<'a>>,
}

impl Nested {
    /// Route the parent window's keys to the window of what is running, and
    /// hand back what each did — including every request to read again, which
    /// the host answers with [`RunningWindow::read_again`].
    ///
    /// Nothing is routed once the window is closed, or while the parent window
    /// does not have the keyboard, and nothing is forwarded to any client.
    ///
    /// # Errors
    /// [`RenderError::Closed`] when the parent window closed, and
    /// [`RenderError::Input`] when `server` has no keyboard.
    pub fn pump_running(
        &mut self,
        server: &mut Server,
        window: &mut RunningWindow,
    ) -> Result<Vec<RunningPressed>, RenderError> {
        let mut did = Vec::new();
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !focused || !window.is_open() {
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.running_key(code, event.state(), event.time_msec()) {
                Ok(Some(key)) => did.push(window.pressed(key)),
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match failure {
            Some(error) => Err(RenderError::Input(error)),
            None => pumped.map(|()| did),
        }
    }

    /// Route the parent window's keys to the window of what is filling the
    /// disk, for somebody reading `reading`, and hand back what each did.
    ///
    /// # Errors
    /// As [`Nested::pump_running`].
    pub fn pump_filling(
        &mut self,
        server: &mut Server,
        window: &mut FillingWindow,
        reading: alo_strings::Direction,
    ) -> Result<Vec<FillingPressed>, RenderError> {
        let mut did = Vec::new();
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !focused || !window.is_open() {
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.filling_key(code, event.state(), event.time_msec(), reading) {
                Ok(Some(key)) => did.push(window.pressed(key)),
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match failure {
            Some(error) => Err(RenderError::Input(error)),
            None => pumped.map(|()| did),
        }
    }

    /// Submit clients, popups and native controls with the desktop above them,
    /// the record window and a waiting question above the desktop, the egress
    /// indicator above all of it, and the cursor on top.
    ///
    /// # Errors
    /// [`RenderError::EgressStatusUnknown`] and every other refusal the
    /// indicator makes, first; the dock's and the desktop windows' refusals;
    /// every refusal the record window and the approval surface make; and the
    /// backend's own submission failures. A refused frame draws nothing.
    #[expect(
        clippy::too_many_arguments,
        reason = "one session frame: clients, popups, cursor, controls, fonts, the desktop, the record and the question"
    )]
    pub fn submit_with_desktop(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        edges: &[crate::EdgePicture],
        labels: &mut WindowControlLabels,
        desktop: DesktopFrame<'_>,
        record: Option<RecordFrame<'_>>,
        approval: Option<ApprovalFrame<'_>>,
        alos_terminal: Option<TheTerminalToDraw<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let size = self.size();
        // **Built before the rest of the pictures**, because both want `labels`
        // mutably and `direct_desktop` orders them the same way for the same
        // reason.
        let terminal = alos_terminal.and_then(|read| {
            crate::direct_desktop::drawn_terminal(
                read.server,
                labels,
                (size.w, size.h),
                read.display,
                desktop,
                desktop.strings,
            )
        });
        let pictures = frame_pictures(
            desktop,
            record,
            approval,
            labels,
            (size.w, size.h),
            terminal,
        )?;
        self.submit_native_layers(roots, popups, cursor, pictures.layers(edges))
    }
}

/// Every native picture in one desktop frame.
pub(crate) struct DesktopPictures {
    /// alo's own terminal, when one is open and could be drawn.
    pub(crate) terminal: Option<crate::the_terminals_cells_are_drawn::TerminalPicture>,
    /// The egress indicator.
    pub(crate) status: EgressStatusPicture,
    /// What on this machine is watching or listening.
    pub(crate) in_use: crate::in_use_raster::InUsePicture,
    /// The notifications on this frame.
    pub(crate) notifications: crate::notification_raster::NotificationPicture,
    /// The capture tools, when somebody is capturing.
    pub(crate) capturing: Option<crate::capture_raster::CapturePicture>,
    /// The dock and the desktop windows.
    pub(crate) desktop: DesktopPicture,
    /// The record window, when the frame carries one.
    pub(crate) record: Option<crate::record_raster::RecordPicture>,
    /// A question, when the frame carries one.
    pub(crate) approval: Option<crate::approval_raster::ApprovalPicture>,
}

impl DesktopPictures {
    /// **Every picture this frame holds, as the layers a display paints.**
    ///
    /// One place rather than a literal at each backend, because a literal at
    /// each backend is how three of these pictures were built every frame on a
    /// real display and never painted. Until 2026-10-27 `direct_desktop` named
    /// only the desktop, the status area and Settings and let
    /// `NativeLayers::nothing()` fill the rest, so alo's own terminal, the
    /// camera-and-microphone indicator and the notifications were laid out and
    /// thrown away — a chord opened a terminal nobody could see, and nothing
    /// said so because nothing failed. The nested backend named them all, which
    /// is why no gate could tell.
    ///
    /// Settings is not here because it is not a picture of this frame's: the
    /// direct backend draws it beside these and sets it on what this returns.
    pub(crate) fn layers<'a>(
        &'a self,
        edges: &'a [crate::EdgePicture],
    ) -> crate::scene_native::NativeLayers<'a> {
        crate::scene_native::NativeLayers {
            scene: None,
            edges,
            desktop: Some(&self.desktop),
            record: self.record.as_ref(),
            settings: None,
            approval: self.approval.as_ref(),
            status: Some(&self.status),
            in_use: Some(&self.in_use),
            notifications: Some(&self.notifications),
            capturing: self.capturing.as_ref(),
            terminal: self.terminal.as_ref(),
        }
    }
}

/// Every picture for one frame, or the refusal that stops the whole frame —
/// the two indicators' first, because a frame that cannot say what is leaving
/// this machine, or that its camera is on, is refused whatever else it holds.
///
/// **The in-use indicator is laid out before what is leaving**, because it has
/// the status area's corner and the egress rows stack beyond it
/// (`crate::in_use_raster` says why).
///
/// **And the desktop is laid out before all three of them**, which is a change of
/// 2026-10-04 and the reason the order is written down here. The put-aside panel
/// reserves a column at the end of the edge the status corner is at, and until the
/// desktop had been drawn nothing knew how wide that column was — so the three
/// corner surfaces were placed against the output's own width and the indicator's
/// band shared 104 pixels with the column on a real draw. The desktop picture does
/// not read any of the three, so moving it first costs nothing; `crate::egress_status_place`
/// carries the ruling this applies.
pub(crate) fn frame_pictures(
    desktop: DesktopFrame<'_>,
    record: Option<RecordFrame<'_>>,
    approval: Option<ApprovalFrame<'_>>,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    // **The finished picture, drawn by the lane and only carried here.**
    //
    // Two rules put it in this shape rather than letting this function draw it.
    // It cannot be a field on the frame: `DesktopFrame` is `Copy`, is built once
    // and used across a whole lane's draw while that lane writes to the server,
    // so a frame carrying `&Server` data stops five of those writes compiling.
    // And it cannot be drawn *here*: `desktop_source`'s
    // `the_desktop_adds_no_number_of_its_own` holds this file to **no string a
    // person could read**, so the sentence that says a terminal could not be
    // drawn has nowhere to go in this file. Words belong where words are owned.
    //
    // So the lane draws it, says what went wrong if it did, and hands over the
    // result.
    terminal: Option<crate::the_terminals_cells_are_drawn::TerminalPicture>,
) -> Result<DesktopPictures, RenderError> {
    let running = running_shows(desktop.running, desktop.strings);
    let filling = filling_shows(desktop.filling, desktop.strings);
    let drawn = crate::desktop_raster::picture(
        desktop.dock,
        desktop.look,
        crate::desktop_raster::Shown {
            the_dock_has_open: desktop.the_dock_has_open,
            // **Read off the frame**, which is the whole point of the field:
            // `what_the_server_tells_a_frame` put the machine's artwork there.
            // A literal `NOTHING` here is exactly the mistake `market` made two
            // fields below, and it would draw a Dock of letters on a machine
            // with every icon loaded.
            what_each_application_draws: desktop.what_each_application_draws,
            the_docks_words: desktop.the_docks_words,
            running: &running,
            filling: &filling,
            division: desktop.division,
            offer: desktop.offer,
            windows: desktop.windows,
            put_aside: desktop.put_aside,
            panel_is_revealed: desktop.panel_is_revealed,
            filling_the_screen: desktop.filling_the_screen,
            display_scale: desktop.display_scale,
            on_the_dock: desktop.on_the_dock,
            in_more_apps: desktop.in_more_apps,
            // **Read off the frame**, like every other field in this block and
            // unlike `market`, which was written `None` here and cost a release
            // — see the note below it. This one is tested through this
            // function rather than through the raster for that reason.
            // **Read off the frame**, like every other field in this block.
            // `market` is the one that was not, and that is why it is the one
            // with a paragraph under it.
            the_bar: desktop.the_bar,
            the_keyboard_is_on_the_bar: desktop.the_keyboard_is_on_the_bar,
            // **Read off the frame, and a literal `None` until 2026-10-19.**
            //
            // Every other field in this block is taken from the frame; this
            // one was written `None` here, so `Server::market` reached
            // `crate::what_the_server_tells_a_frame`, was set on the frame
            // with care, and was then dropped by the one function that builds
            // the picture for both roads.
            //
            // Measured on a booted machine the same day: a press on the
            // Applications entry took the button, opened the window, drew
            // nothing, and from that moment the machine answered no keys at
            // all -- because an open window takes them and the open window was
            // invisible. Three tests passed either side of this line.
            market: desktop.market,
            tasks: desktop.tasks,
        },
        &mut labels.fonts,
        size,
    )?;
    // The output and the one column the three corner surfaces below must stop
    // before, carried together because neither is useful to them alone. The column is
    // taken from the picture rather than recomputed, so there is no second answer to
    // keep in step — the same reason `crate::top_controls_region` takes it as an
    // argument.
    let room = crate::egress_status_place::TheRoom {
        size,
        panel: Some(drawn.panel.reserved),
    };
    let in_use = crate::in_use_raster::picture(
        desktop.in_use,
        desktop.strings,
        desktop.dock,
        labels,
        room,
        desktop.look.in_use(),
    )?;
    let capturing = desktop
        .capturing
        .map(|tools| crate::capture_raster::picture(tools, size, desktop.look.capture()))
        .transpose()?;
    let notifications = crate::notification_raster::picture(
        desktop.notifications,
        desktop.strings,
        desktop.dock,
        labels,
        room,
        desktop.look.notifications(),
    )?;
    let status = status_picture(
        EgressStatusFrame {
            status: desktop.egress,
            strings: desktop.strings,
            dock: desktop.dock,
            look: desktop.look.egress(),
        },
        labels,
        room,
        in_use.height,
    )?;
    let record = match record {
        Some(record) => Some(crate::record_raster::picture(
            record.window.shows(),
            record.strings,
            labels,
            size,
            record.look,
            record.clock,
        )?),
        None => None,
    };
    let approval = match approval {
        Some(approval) => Some(crate::approval_raster::picture(
            approval.screen.shows(),
            approval.strings,
            labels,
            size,
            approval.look,
        )?),
        None => None,
    };
    Ok(DesktopPictures {
        terminal,
        status,
        in_use,
        notifications,
        capturing,
        desktop: drawn,
        record,
        approval,
    })
}

#[cfg(test)]
#[path = "nested_desktop_tests.rs"]
mod tests;
