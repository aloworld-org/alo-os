//! **The canvas, walked** — task 10 of
//! `docs/autonomy/the-smallest-canvas-worth-showing.md`, and the last task in
//! that plan.
//!
//! *One walk: open three applications, drag one, resize another, pan, zoom out to
//! Show all, zoom back into one and work in it, then the same by keyboard alone.
//! A raster at each step and the exact sequence recorded as a table the test reads
//! out of the published report.*
//!
//! # Which half is here
//!
//! The **raster** half. Each step submits a frame through a real nested parent's
//! EGL and reads back the pixels that frame was painted into, before the buffer
//! is swapped away, exactly as `crate::the_walk_check` does for the shell's
//! surfaces. The **sequence** half is
//! `crates/alo-shell/tests/the_canvas_walked.rs`, which holds the order of the
//! steps against the table in the report — a test, because a sequence is the same
//! sequence on every machine and a walk nobody can run on their own laptop is a
//! walk nobody checks.
//!
//! # Counted, not compared to a picture
//!
//! Painted pixels per step, for `the_walk_check`'s reason: this fixture answers
//! *the canvas drew*, and a stored reference image would make it a test of the
//! artwork. But two steps here also assert **numbers** rather than merely
//! non-emptiness, because the canvas has already proved that *fewer pixels* is
//! too weak a check — a zoom applied to sizes but not to positions draws fewer
//! pixels and is wrong, and that fault passed *smaller than life size* once
//! already.
//!
//! # Two of the six steps have no keyboard form, and that is scope rather than a gap
//!
//! *Then the same by keyboard alone* cannot mean all six. **Dragging and resizing
//! a frame have no keyboard form in v0.5 and none is promised.** ADR 0065's *every
//! one of them has a keyboard form* attaches to its own list — zoom, pan, fit the
//! Place, fill the screen with what is selected, work inside a frame — and
//! `docs/features.md:422`, which is **[v1]**, attaches *each with a keyboard form*
//! to the same three. `docs/design/the-shortcuts-and-the-edges.md` does give
//! `Alt + F7` and `Alt + F8` for moving and resizing a window precisely, and those
//! rows are under **Working with windows**, not under *Moving around the canvas*;
//! no `alo_shortcuts::Action` exists for either.
//!
//! So the keyboard half of this walk is the canvas's own movement — pan, zoom,
//! *Show all*, and reaching a frame with no pointer at all — and the two steps it
//! cannot mirror are named in the report rather than left to be noticed. Building
//! them here would be inventing v0.5 scope to make a walk symmetrical, which
//! CLAUDE.md's *scope is gated* forbids and which is how a minimum stops being
//! one.
//!
//! # What a certified machine has seen of this
//!
//! Nothing. The parent is `weston --backend=headless` and the renderer is
//! llvmpipe. No panel has shown any of it, and the report says so rather than
//! leaving it to be inferred.

use std::os::unix::fs::PermissionsExt as _;
use std::sync::mpsc;
use std::time::Duration;

use smithay::backend::input::{ButtonState, KeyState};

/// The Linux button a drag is made with.
const BTN_LEFT: u32 = 0x110;
/// `KEY_RIGHT`, as a keyboard sends it.
const RIGHT: u32 = 106;
/// The nested parent's output, which every fit is measured against.
const VIEWPORT: (i32, i32) = (1366, 768);
/// How long a client waits for the server's software rendering.
///
/// The same two minutes `crate::the_walk_check` settled on, for its reason: a
/// software-rendered 1366×768 frame per pump is slower than any patience short of
/// this, and a client's deadline shorter than the server's work is a fixture
/// failing about its own timing.
const PATIENCE: Duration = Duration::from_secs(120);

/// One step of the walk, and what came back off the frame it drew.
struct Step {
    /// What a person was doing.
    moment: &'static str,
    /// How big the frame was.
    size: (u32, u32),
    /// How many of its pixels were not the clear colour.
    painted: usize,
}

impl Step {
    /// This step's frame, counted rather than described.
    fn of(moment: &'static str, pixels: &alo_shell::ScanoutPixels) -> Self {
        let (width, height) = pixels.size();
        // XRGB8888 little-endian: the three colour bytes first, the fourth
        // ignored. Painted means any of the three is not the black this scene is
        // cleared to.
        let painted = pixels
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|[blue, green, red, _]| *blue != 0 || *green != 0 || *red != 0)
            .count();
        Self {
            moment,
            size: (width, height),
            painted,
        }
    }

    /// Whether this step's canvas put nothing on the output.
    fn drew_nothing(&self) -> bool {
        self.painted == 0
    }
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (width, height) = self.size;
        write!(
            f,
            "{:<52} {width}x{height}, {} pixels painted",
            self.moment, self.painted
        )
    }
}

impl std::fmt::Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.moment)
    }
}

/// The whole walk, one step at a time.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut nested = alo_shell::Nested::new("alo — the canvas, walked", (1366, 768))?;
    nested.keep_each_frame(true);
    let mut met: Vec<Step> = Vec::new();

    the_canvas(&mut nested, &mut met)?;

    for step in &met {
        println!("{step}");
    }
    let empty: Vec<&Step> = met.iter().filter(|step| step.drew_nothing()).collect();
    if !empty.is_empty() {
        return Err(format!(
            "{} of {} steps drew nothing: {empty:?}",
            empty.len(),
            met.len()
        )
        .into());
    }
    println!(
        "{} steps walked on the canvas against a real parent, each read back off \
         the frame it drew; dragging and resizing have no keyboard form in v0.5 \
         and none is promised; physical display unverified",
        met.len()
    );
    Ok(())
}

/// Open three applications and walk the canvas with them.
fn the_canvas(
    nested: &mut alo_shell::Nested,
    met: &mut Vec<Step>,
) -> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::Server;

    let runtime = tempfile::tempdir()?;
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "the-canvas-walked",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..Default::default()
        },
    )?;
    server.render(nested, 0)?;
    server.enable_pointer()?;

    // **Three applications, on one connection each.** Three rather than two
    // because the plan says three, and because *Show all* fitting every frame is
    // a different claim with three than with two: two frames side by side fit any
    // viewport, and the third is what makes the fit have to shrink.
    let fixture = crate::Fixture {
        path: server.socket_path().to_owned(),
    };
    let (ready, mapped) = mpsc::channel::<()>();
    let (hold, released) = mpsc::channel::<()>();
    let clients = std::thread::spawn(move || {
        let mut opened = Vec::new();
        for _ in 0..3 {
            let mut app = crate::application::Application::new(&fixture);
            app.configure();
            app.attach();
            app.sync();
            opened.push(app);
        }
        assert!(ready.send(()).is_ok());
        // Stay connected for the whole walk: a client that leaves takes its frame
        // with it, and every step after would be reading a canvas with fewer
        // frames on it than the step claims.
        assert!(released.recv_timeout(PATIENCE).is_ok());
        for app in &mut opened {
            app.sync();
        }
    });
    // **Dispatch while waiting, or it is a deadlock.** `crate::the_walk_check`
    // says it in as many words — *the only thread that answers a roundtrip is
    // this one, so a wait that does not dispatch is a deadlock* — and the first
    // version of this fixture blocked on the channel instead. Three clients then
    // sat in a handshake nobody was answering until their socket reset, and the
    // only thing printed was their panic.
    let waiting = std::time::Instant::now();
    while server.mapped_surfaces().count() < 3 {
        if waiting.elapsed() > PATIENCE {
            return Err(format!(
                "three applications did not map within the deadline; {} did",
                server.mapped_surfaces().count()
            )
            .into());
        }
        server.dispatch()?;
        nested.pump()?;
        server.render(nested, 0)?;
    }
    let _ = mapped.recv_timeout(PATIENCE);
    nested.pump()?;
    server.render(nested, 0)?;

    // **The walk's own steps run inside a closure so the clients are always let
    // go.** The first version returned `?` straight out of here, so any refusal
    // left three clients waiting the full two minutes and then panicking on a
    // reset socket — and their panic was the only thing printed. The server's
    // actual reason was lost, which took a run to notice and is the same fault as
    // a refusal that says nothing.
    let frames = frames_of(&server)?;
    let walked = walk_it(nested, met, &mut server, &frames);
    // Let them go, dispatching while they do, for the same reason as above: a
    // client's last sync needs this thread to answer it.
    let _ = hold.send(());
    let leaving = std::time::Instant::now();
    while !clients.is_finished() && leaving.elapsed() < PATIENCE {
        let _ = server.dispatch();
        let _ = nested.pump();
    }
    let _ = clients.join();
    walked
}

/// The three frames this walk moves, in the order they opened.
fn frames_of(
    server: &alo_shell::Server,
) -> Result<
    Vec<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
    Box<dyn std::error::Error>,
> {
    let frames: Vec<_> = server.mapped_surfaces().cloned().collect();
    if frames.len() != 3 {
        return Err(format!("three applications opened and {} are mapped", frames.len()).into());
    }
    Ok(frames)
}

/// Every step of the walk, with the clients held open around it.
#[expect(
    clippy::too_many_lines,
    reason = "one walk is one sequence; splitting it would hide the order, which \
              is the thing task 10's acceptance is about"
)]
fn walk_it(
    nested: &mut alo_shell::Nested,
    met: &mut Vec<Step>,
    server: &mut alo_shell::Server,
    frames: &[smithay::reexports::wayland_server::protocol::wl_surface::WlSurface],
) -> Result<(), Box<dyn std::error::Error>> {
    // Spread them, so the walk has a canvas rather than a stack. Far enough apart
    // that *Show all* has something to fit and a pan has somewhere to go.
    for (frame, at) in frames.iter().zip([(80, 80), (520, 300), (960, 560)]) {
        server.place_window(frame, at)?;
    }
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(0)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the opening frame was not kept")?,
    ));

    // **One dragged, by its name.** ADR 0071 makes the name band the only handle,
    // so this is the gesture a person has: press on the band above the frame and
    // move. Asserted by the frame's own origin rather than by the picture, because
    // a drag that moved the wrong frame would draw a plausible canvas.
    let dragged = frames.first().ok_or("a frame to drag")?.clone();
    let before = alo_shell::window_buffer_origin(&dragged);
    // The band is directly above the frame and as wide as it
    // (`alo_shell::the_names_band`), so its middle is half a band above the
    // frame's own top edge. Computed from the two public numbers rather than read
    // from the shell's private getter, which is also what keeps this fixture
    // honest about aiming where a person would.
    let band = (before.x + 8.0, before.y - alo_shell::the_names_band() / 2.0);
    server.pointer_motion(band.0, band.1, 1)?;
    if server.pointer_button(BTN_LEFT, ButtonState::Pressed, 2)? {
        return Err("a press on the shell's own name band was delivered to a client".into());
    }
    server.pointer_motion(band.0 + 120.0, band.1 + 40.0, 3)?;
    let _ = server.pointer_button(BTN_LEFT, ButtonState::Released, 4)?;
    let after = alo_shell::window_buffer_origin(&dragged);
    if (after.x - before.x).abs() < 1.0 && (after.y - before.y).abs() < 1.0 {
        return Err(format!("a drag of the name band left the frame at {after:?}").into());
    }
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(1)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the dragged frame was not kept")?,
    ));

    // **Another resized, from its own corner band**, which is the road task 4
    // built and the only one there is: ADR 0071 refuses `xdg_toplevel.resize`.
    // **Pressed on the band, not begun by name.** `Server::begin_a_resize` would
    // have been one call, and it refuses unless a button is already held — which
    // is right, and which is why the first version of this step failed with *a
    // resize on the shell's own corner band did not begin* after the drag above
    // released the button. Going through `pointer_button` is better evidence
    // anyway: it is the road a mouse takes, through the hit test, and it is the
    // one that had no production caller until task 4's second commit.
    let resized = frames.get(1).ok_or("a frame to resize")?.clone();
    let corner = alo_shell::window_buffer_origin(&resized);
    // The fixture's windows carry a 16x16 buffer, so the bottom-right corner band
    // begins at the frame's own far edge and reaches `the_resize_corner()` beyond
    // it. Aim at its middle.
    let band = (
        corner.x + 16.0 + alo_shell::the_resize_corner() / 2.0,
        corner.y + 16.0 + alo_shell::the_resize_corner() / 2.0,
    );
    server.pointer_motion(band.0, band.1, 5)?;
    if server.pointer_button(BTN_LEFT, ButtonState::Pressed, 6)? {
        return Err("a press on the shell's own resize band was delivered to a client".into());
    }
    // **Proof that a resize is actually under way**, which the raster cannot give.
    // A 16x16 buffer moved or re-anchored paints the same number of pixels, so this
    // step's picture is evidence that the canvas drew and nothing more — the three
    // steps before it all report 832 painted for exactly that reason. What is
    // available publicly is the transaction's own exclusivity: a second resize,
    // on any frame, must be refused while one is held. It answers `false` for other
    // reasons too, so it is asked **before and after** — refused now, accepted once
    // the button is up — and only the pair of answers means a transaction was live.
    let another = frames.get(2).ok_or("a third frame")?.clone();
    if server.begin_a_resize(&another, alo_shell::FrameEdge::Top) {
        return Err("a second resize began while the corner band held one".into());
    }
    server.pointer_motion(band.0 + 60.0, band.1 + 40.0, 7)?;
    let _ = server.pointer_button(BTN_LEFT, ButtonState::Released, 8)?;
    // And the band is not simply dead: with the button up and the pointer still on
    // it, the same gesture is taken again. Without this the refusal above would be
    // satisfied by a resize road that never works at all.
    server.pointer_motion(band.0, band.1, 9)?;
    if server.pointer_button(BTN_LEFT, ButtonState::Pressed, 10)? {
        return Err("a second press on the resize band was delivered to a client".into());
    }
    if server.begin_a_resize(&another, alo_shell::FrameEdge::Top) {
        return Err("the resize band took a press and held no transaction".into());
    }
    let _ = server.pointer_button(BTN_LEFT, ButtonState::Released, 11)?;
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(2)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the resized frame was not kept")?,
    ));

    // **Panned**, by the wheel over empty canvas, which is task 5's first road.
    let looking = server.the_camera().at();
    server.pan_the_canvas(160, 90).ok_or("a pan on the plane")?;
    if server.the_camera().at() == looking {
        return Err("a pan left the camera where it was".into());
    }
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(3)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the panned frame was not kept")?,
    ));

    // **Zoomed out to *Show all*.** The claim is not that something was drawn: it
    // is that every frame is inside the viewport afterwards, which is task 6's
    // acceptance and the one sentence the whole plan has to earn.
    server
        .show_all_on_the_canvas()
        .ok_or("show all with three frames open")?;
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(4)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the show-all frame was not kept")?,
    ));
    all_of_it_fits(server, VIEWPORT, "show all by pointer")?;
    let far_out = server.the_camera().zoom();

    // **Zoomed back into one and worked in it.** Worked means the application
    // received something: a canvas a person can look into and not type into is
    // the refusal task 1 recorded and task 2 removed, so this step is where that
    // removal is proved end to end.
    server
        .zoom_the_canvas(
            alo_canvas::Zoom::of(1000).map_err(|why| format!("{why}"))?,
            (683, 384),
        )
        .ok_or("a zoom back to life size")?;
    if server.the_camera().zoom() == far_out {
        return Err("zooming back in left the camera where show all put it".into());
    }
    server.keyboard_focus(Some(&dragged))?;
    let typed = server.keyboard_key(RIGHT, KeyState::Pressed, 12)?;
    let _ = server.keyboard_key(RIGHT, KeyState::Released, 13)?;
    if !typed {
        return Err("a key with a frame focused reached no application".into());
    }
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(5)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the worked-in frame was not kept")?,
    ));

    // ---- and now the same by keyboard alone ----
    //
    // No pointer call past this line. Every remaining step is a key, which is
    // what *the same by keyboard alone* can honestly mean in v0.5 — see the
    // header for the two steps that have no keyboard form and why.

    // **Panned by the arrows**, which landed with task 5 on 2026-09-30 and was
    // the road that plan was marked done without.
    server.keyboard_focus(None)?;
    let looking = server.the_camera().at();
    let handed_on = server.keyboard_key(RIGHT, KeyState::Pressed, 14)?;
    let _ = server.keyboard_key(RIGHT, KeyState::Released, 15)?;
    if handed_on {
        return Err("an arrow over the canvas was handed to an application".into());
    }
    if server.the_camera().at() == looking {
        return Err("an arrow with nothing focused left the camera where it was".into());
    }
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(6)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the arrow-panned frame was not kept")?,
    ));

    // **Show all, and back in, by their chords.** `Super + 0` and `Super + Plus`
    // in the shipped defaults, reached through the same `dispatch_canvas_command`
    // a real key press reaches.
    let shortcuts = alo_shortcuts::Shortcuts::shipped();
    let before_show_all = server.the_camera().zoom();
    let looking_before = server.the_camera().at();
    server
        .dispatch_canvas_command(
            &shortcuts,
            chord(alo_shortcuts::Action::ShowAllOnTheCanvas, &shortcuts)?,
            std::time::SystemTime::now(),
        )
        .map_err(|why| format!("show all by keyboard: {why}"))?;
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(7)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the keyboard show-all frame was not kept")?,
    ));
    // **Checked as a camera, not as a pixel count, and the first version of this
    // was wrong in a way worth keeping.** It compared this step's painted pixels
    // against the previous step's and demanded fewer, on the reasoning that *show
    // all* shrinks three frames to fit. It failed: 2224 against 576. The baseline
    // was the arrow-panned step, where two of the three frames sit partly off a
    // 1366-wide output — so *show all* paints **more** by bringing them back into
    // view while also drawing them smaller. Two variables moved and the assertion
    // named one.
    //
    // *A zoom the drawing never received* is proved in pixels by
    // `crate::the_walk_check`, which zooms with the camera otherwise still: 512
    // painted at life size against 84 at four tenths. That is a controlled
    // comparison and this is not the place to make a worse copy of it. What this
    // step owes is what task 6's acceptance says — that *show all* took the camera
    // further out and left every frame on the plane reachable.
    all_of_it_fits(server, VIEWPORT, "show all by keyboard")?;
    if server.the_camera().zoom() == before_show_all && server.the_camera().at() == looking_before {
        return Err("show all from the keyboard moved the camera nowhere at all".into());
    }

    server
        .dispatch_canvas_command(
            &shortcuts,
            chord(alo_shortcuts::Action::ZoomTheCanvasIn, &shortcuts)?,
            std::time::SystemTime::now(),
        )
        .map_err(|why| format!("zoom in by keyboard: {why}"))?;
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(8)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the keyboard zoom frame was not kept")?,
    ));

    // **A frame reached and focused with no pointer**, which is task 7's
    // acceptance and the last thing the keyboard half owes: a canvas that needs a
    // touchpad excludes people (EN 301 549).
    // Reached with `NextWindow`, which is the chord a person has, and **proved by
    // a key arriving** rather than by asking the compositor what it thinks is
    // focused. A frame that was selected without taking focus would answer the
    // first question and fail the one that matters, which is whether somebody can
    // now type into it.
    server.keyboard_focus(None)?;
    server
        .dispatch_window_command(
            &shortcuts,
            chord(alo_shortcuts::Action::NextWindow, &shortcuts)?,
        )
        .map_err(|why| format!("reaching a frame by keyboard: {why}"))?;
    let arrived = server.keyboard_key(RIGHT, KeyState::Pressed, 16)?;
    let _ = server.keyboard_key(RIGHT, KeyState::Released, 17)?;
    if !arrived {
        return Err(
            "a frame reached by keyboard took no focus, so nothing could be typed into it".into(),
        );
    }
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        moment(9)?,
        nested
            .the_frame_just_drawn()
            .ok_or("the keyboard-focused frame was not kept")?,
    ));

    Ok(())
}

/// The label for this step, read from the order both halves of task 10 share.
///
/// Indexed rather than written out, so a moment renamed in one place cannot stay
/// spelled the old way here — which is the fixture's half of *a step that changes
/// without the table fails.*
fn moment(which: usize) -> Result<&'static str, Box<dyn std::error::Error>> {
    crate::the_canvas_walk_moments::EVERY_MOMENT
        .get(which)
        .copied()
        .ok_or_else(|| format!("the walk has no step {which}").into())
}

/// Every frame's two screen corners under the camera as it stands.
///
/// The instrument task 6's own test uses — `Camera::screen_of` on a frame's origin
/// and its opposite corner — because *inside the viewport* is a statement about
/// where frames land on the glass and nothing else answers it.
fn every_frame_on_the_glass(server: &alo_shell::Server) -> Vec<((i32, i32), (i32, i32))> {
    let camera = server.the_camera();
    server
        .the_frames_on_the_plane()
        .into_iter()
        .filter_map(|frame| {
            Some((
                camera.screen_of(frame.at())?,
                camera.screen_of(frame.opposite()?)?,
            ))
        })
        .collect()
}

/// Refuse unless every frame is inside a viewport this size, none clipped.
///
/// **This is what *Show all* promises**, and the promise is not about a direction.
/// The first version of this walk demanded that *show all* zoom **out**, because
/// the plan's sentence says *zoom out to Show all* — and it zoomed in to
/// `Zoom(1684)`, correctly: three small frames clustered on a large output are
/// fitted by magnifying them. *Show all* fits an extent, and out is the common
/// case rather than the rule. The phrasing was encoded and the behaviour was not.
fn all_of_it_fits(
    server: &alo_shell::Server,
    viewport: (i32, i32),
    when: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let corners = every_frame_on_the_glass(server);
    if corners.is_empty() {
        return Err(format!("{when}: show all was asked about no frames").into());
    }
    for ((left, top), (right, bottom)) in corners {
        if left < 0 || top < 0 {
            return Err(
                format!("{when}: a frame starts off the top-left at ({left}, {top})").into(),
            );
        }
        if right > viewport.0 || bottom > viewport.1 {
            return Err(
                format!("{when}: a frame is clipped, ending at ({right}, {bottom})").into(),
            );
        }
    }
    Ok(())
}

/// The chord the shipped defaults give this action.
///
/// Read from the defaults rather than written here, so a rebinding in
/// `alo_shortcuts::defaults` cannot leave this walk pressing a key that no longer
/// does anything while still reporting the step as walked.
fn chord(
    action: alo_shortcuts::Action,
    shortcuts: &alo_shortcuts::Shortcuts,
) -> Result<alo_shortcuts::Chord, Box<dyn std::error::Error>> {
    shortcuts
        .chord_for(action)
        .ok_or_else(|| format!("no chord is bound to {action:?}").into())
}
