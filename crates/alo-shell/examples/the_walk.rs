//! **Every new surface, walked** — one person's way through this shell, on a
//! real nested compositor under a Wayland parent, with a raster read back off
//! each frame that was drawn.
//!
//! `docs/autonomy/v0-5-the-shell-plan.md` task 14: *one walk through the nested
//! compositor — sign in, dock a second display, divide the screen, take a
//! screenshot with a blur, receive a notification, lock, unlock by keyboard
//! with the screen reader on — produces a raster at each step and the exact
//! sequence of spoken and shown text.*
//!
//! # Why a fixture and a test, and which holds which half
//!
//! The **raster** half is here, because it needs a Wayland parent and a GLES
//! context and no `#[test]` on any machine can be given those without becoming
//! a thing the gates refuse to run. What this proves is that each surface
//! really draws: every step submits a frame through the parent's own EGL and
//! reads back the pixels that frame was painted into, before the buffer is
//! swapped away. A step whose surface drew nothing comes back as an empty
//! picture and this fixture says so.
//!
//! The **sequence** half is a test —
//! `crates/alo-shell/tests/every_new_surface_walked.rs` — because a sentence is
//! the same sentence on every machine, and a walk nobody can run on their own
//! laptop is a walk nobody checks. It holds the exact order of what is spoken
//! and shown against the table in the report, which is the house pattern that
//! `alo-access` and `alo-sleeping` already use for their own walks.
//!
//! # What a certified machine has seen of this
//!
//! Nothing. The parent here is `weston --backend=headless` and the renderer is
//! llvmpipe; no panel has shown any of it. That is this task's constraint and
//! it is repeated in the report rather than left to be inferred.

// The same real Wayland client the protocol tests and the offscreen probe use.
// It carries more than this walk asks of it — a walk that needs two windows
// mapped uses neither its popups nor its subsurfaces — and the fields for those
// are read by its other callers rather than by this one.
#[cfg(target_os = "linux")]
#[expect(
    dead_code,
    reason = "this shared client fixture carries more than one caller uses"
)]
#[path = "../tests/support/application.rs"]
mod application;

/// Socket location the walk's own clients connect to, reached by
/// `tests/support/application.rs` as `crate::Fixture`.
#[cfg(target_os = "linux")]
pub struct Fixture {
    /// The walk's private display, distinct from the parent's socket.
    pub path: std::path::PathBuf,
}

/// Main-thread graphics initialisation is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("the walk failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// The whole walk, one step at a time.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::Nested;

    let mut nested = Nested::new("alo — every new surface, walked", (1366, 768))?;
    nested.keep_each_frame(true);
    let mut met: Vec<Step> = Vec::new();

    sign_in(&mut nested, &mut met)?;
    divide_between_two_windows(&mut nested, &mut met)?;
    the_desktop(&mut nested, &mut met)?;
    lock_and_unlock(&mut nested, &mut met)?;

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
        "{} steps walked on a nested parent, each read back off the frame it drew; \
         physical display unverified",
        met.len()
    );
    Ok(())
}

/// One step of the walk, and what came back off the frame it drew.
#[cfg(target_os = "linux")]
struct Step {
    /// What a person was doing.
    moment: &'static str,
    /// How big the frame was.
    size: (u32, u32),
    /// How many of its pixels were not the clear colour.
    painted: usize,
}

#[cfg(target_os = "linux")]
impl Step {
    /// This step's frame, counted rather than described.
    ///
    /// **Counting painted pixels rather than comparing to a picture** is
    /// deliberate: what this fixture is for is *the surface drew*, and a
    /// stored reference image would make it a test of the artwork, which the
    /// raster tests beside each surface already are. A frame that came back
    /// entirely the clear colour is the failure worth catching here — it is
    /// what a surface that refused, or was laid out off the output, looks
    /// like.
    fn of(moment: &'static str, pixels: &alo_shell::ScanoutPixels) -> Self {
        let (width, height) = pixels.size();
        // XRGB8888 little-endian, so the three colour bytes come first and the
        // fourth is the ignored one. A pixel is painted when any of the three
        // is not the black this scene is cleared to.
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

    /// Whether this step's surface put nothing on the output.
    fn drew_nothing(&self) -> bool {
        self.painted == 0
    }
}

#[cfg(target_os = "linux")]
impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (width, height) = self.size;
        write!(
            f,
            "{:<44} {width}x{height}, {} pixels painted",
            self.moment, self.painted
        )
    }
}

#[cfg(target_os = "linux")]
impl std::fmt::Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.moment)
    }
}

/// Lock the session, then unlock it from the keyboard.
///
/// The lock frame is the one surface that is drawn **instead of** everything
/// else rather than above it — `crate::scene_drawing::paint` sends it straight
/// to the lock texture and imports no client — so a raster of it is the
/// strongest single thing this walk reads back: nothing underneath can have
/// painted those pixels.
#[cfg(target_os = "linux")]
fn lock_and_unlock(
    nested: &mut alo_shell::Nested,
    met: &mut Vec<Step>,
) -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::{Appearance, Background, DisplayId, Picture, Scheme, TextScale};
    use alo_shell::{
        Contrast, FrameTarget, LockBackground, LockLook, LockPressed, LockSurface, SignInKey,
        SignInLook, WindowControlLabels,
    };
    use std::time::{Duration, SystemTime};

    let mut accounts = alo_accounts::Accounts::none()?;
    accounts
        .created("ada", 1000, "the walk's own password")
        .map_err(|_| "cannot create the walk's account")?;
    let signed = accounts
        .signs_in("ada", "the walk's own password")
        .map_err(|_| "sign-in refused")?;
    let session = alo_accounts::Session::opened(signed, 1000).map_err(|_| "session refused")?;
    let words = alo_strings::Strings::of(alo_saying::everything_this_machine_can_say()?);
    let seat =
        alo_locking::Seat::<String>::opened(session).locked(&mut alo_overlay::Summoning::closed());
    let mut surface = LockSurface::of(seat, words.clone()).map_err(|_| "seat was not locked")?;

    let mut appearance = Appearance::shipped();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/artwork/wallpapers/alo-quiet-horizon.png")
        .canonicalize()?;
    appearance.set_background(Background::Picture(
        Picture::file(path).map_err(|_| "invalid asset path")?,
    ));
    let display = DisplayId::named("the-walk").map_err(|_| "invalid display")?;
    let region = alo_formats::Regionally::reading("en").map_err(|_| "invalid region")?;
    let timezone = alo_formats::Timezone::named("Europe/Berlin").map_err(|_| "invalid zone")?;
    let look = LockLook {
        appearance: SignInLook {
            contrast: Contrast::AsDesigned,
            scheme: Scheme::Dark,
            scale: TextScale::ordinary(),
        },
        region: &region,
        timezone: &timezone,
        strings: &words,
    };
    let mut labels = WindowControlLabels::new()?;

    nested.pump()?;
    let screen = surface
        .snapshot(
            SystemTime::now(),
            &appearance,
            &display,
            Some(alo_locking::Battery::reading(72, true)?),
            &alo_egress::Indicator::default(),
        )
        .ok_or("no lock screen")?;
    let size = nested.size();
    let background = LockBackground::prepare(screen.image(), (size.w, size.h), Duration::ZERO)?;
    nested.submit_lock(&surface, &screen, &background, &mut labels, &look)?;
    met.push(Step::of(
        "the screen is locked",
        nested
            .the_frame_just_drawn()
            .ok_or("the lock frame was not kept")?,
    ));

    // The keyboard alone opens it. `LockPressed::Opened` is the unlock, and it
    // is reached through `Server::sign_in_key` — the same road the sign-in
    // screen takes, never a client's.
    surface = match surface.pressed(SignInKey::Enter, || {
        Err(alo_greeting::NotReadable {
            at: "unused".into(),
            why: "no submission".into(),
        })
    }) {
        LockPressed::Still(surface) => *surface,
        LockPressed::Opened { .. } => return Err("opened on an empty password".into()),
    };
    nested.pump()?;
    let screen = surface
        .snapshot(
            SystemTime::now(),
            &appearance,
            &display,
            Some(alo_locking::Battery::reading(72, true)?),
            &alo_egress::Indicator::default(),
        )
        .ok_or("no lock screen after the key")?;
    nested.submit_lock(&surface, &screen, &background, &mut labels, &look)?;
    met.push(Step::of(
        "a key is pressed at the locked screen",
        nested
            .the_frame_just_drawn()
            .ok_or("the second lock frame was not kept")?,
    ));
    Ok(())
}

/// **Sign in**, at the screen a person meets before anything is theirs.
///
/// The sign-in screen is the whole output — `crate::scene_drawing::paint` sends
/// it above no client, because there are none yet — so a raster of it is the
/// same strong thing the lock frame is: nothing underneath can have drawn those
/// pixels.
#[cfg(target_os = "linux")]
fn sign_in(
    nested: &mut alo_shell::Nested,
    met: &mut Vec<Step>,
) -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::{Scheme, TextScale};
    use alo_shell::{Contrast, SignInLook, SignInScreen, Signing, WindowControlLabels};
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::fs::PermissionsExt as _;

    let runtime = tempfile::tempdir()?;
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))?;
    // A door that answers whatever it is asked, in the opener's own words. No
    // session is opened on the machine running this walk.
    let door_at = runtime.path().join("sign-in.sock");
    let listener = std::os::unix::net::UnixListener::bind(&door_at)?;
    std::thread::spawn(move || {
        for connection in listener.incoming().flatten() {
            let mut line = String::new();
            if BufReader::new(&connection).read_line(&mut line).is_ok() {
                let answer = format!("{}\n", alo_sessiond::Answered::Opened.written());
                drop((&connection).write_all(answer.as_bytes()));
            }
        }
    });

    let words = alo_strings::Strings::of(alo_saying::everything_this_machine_can_say()?);
    let mut store = alo_accounts::Accounts::none()?;
    store.created("ada", 1000, "the walk's own password")?;
    let mut labels = WindowControlLabels::new()?;
    let look = SignInLook {
        contrast: Contrast::AsDesigned,
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
    };
    let mut screen = SignInScreen::of(
        Ok(alo_greeting::Greeting::of(
            store,
            1000,
            alo_greeting::TheOpenersDoor::at(&door_at),
        )),
        words,
    );

    nested.pump()?;
    nested.submit_sign_in(&screen, &mut labels, look)?;
    met.push(Step::of(
        "the sign-in screen, before any account is chosen",
        nested
            .the_frame_just_drawn()
            .ok_or("the sign-in frame was not kept")?,
    ));

    // A name typed one key at a time, because that is how a person types it and
    // because the screen's own state machine is what decides when a name is a
    // name. Nothing is submitted to the door until Enter.
    for key in ['a', 'd', 'a'] {
        screen = match screen.pressed(alo_shell::SignInKey::Letter(key)) {
            Signing::Still(next) => *next,
            Signing::HandedOver(_) => return Err("a session was opened by a letter".into()),
        };
    }
    nested.pump()?;
    nested.submit_sign_in(&screen, &mut labels, look)?;
    met.push(Step::of(
        "a name typed at the sign-in screen",
        nested
            .the_frame_just_drawn()
            .ok_or("the typed-name frame was not kept")?,
    ));
    Ok(())
}

/// **The desktop**: a second display docked, the screen divided, a capture being
/// marked with a blur, and a notification arriving.
///
/// Four steps in one function because they are four states of **one** surface —
/// the ordinary desktop — and each is submitted and read back on its own. What
/// changes between them is what the frame is handed, which is the point: every
/// one of these is another crate's answer drawn, and this compositor decides
/// none of them.
#[cfg(target_os = "linux")]
fn the_desktop(
    nested: &mut alo_shell::Nested,
    met: &mut Vec<Step>,
) -> Result<(), Box<dyn std::error::Error>> {
    use alo_capturing::{Mark, Region, What};
    use alo_desktops::{DisplayId as DesktopDisplay, Promises};
    use alo_dividing::{Area, Point, WindowId, area::Size};
    use alo_dock::Dock;
    use alo_notifying::arriving::from_alo_os;
    use alo_notifying::deciding::arrives;
    use alo_notifying::quiet::Quiet;
    use alo_shell::{
        DesktopLook, EgressStatus, FillingWindow, RunningWindow, Server, WindowControlLabels,
    };
    use std::os::unix::fs::PermissionsExt as _;

    let words = alo_strings::Strings::of(alo_saying::everything_this_machine_can_say()?);
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut labels = WindowControlLabels::new()?;
    let look = DesktopLook::of(
        &alo_appearance::Appearance::shipped(),
        &alo_access::TurnedOn::nothing(),
        alo_appearance::TimeOfDay::checked(9, 41).map_err(|why| format!("{why:?}"))?,
        alo_strings::Direction::LeftToRight,
    );

    // The indicator is told that nothing is leaving. A desktop whose indicator
    // has never been told anything is refused before any of it is drawn, which
    // is `frame_pictures`' own rule and the right one.
    let mut egress = EgressStatus::on_an_output();
    alo_indicator::Indicating::nowhere().show(Some(&mut egress), &alo_egress::Indicator::default());

    let runtime = tempfile::tempdir()?;
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind(runtime.path(), "the-walk")?;

    // **A second display docked.** `alo-desktops` gives it desktops and
    // `alo-dividing` gives back the division it left, which on a screen nobody
    // has seen before is none. There is no surface of its own to draw: nothing
    // in this compositor paints `screens_raster`'s pictures, which is a gap
    // named in the report rather than papered over. What is drawn is the first
    // display's frame afterwards.
    let a_screen = Area::of(Point::at(0, 0), Size::of(1366, 768))?;
    let promises = || -> Result<Promises, Box<dyn std::error::Error>> {
        Ok(Promises::of(
            WindowId::from_compositor(1),
            WindowId::from_compositor(2),
            WindowId::from_compositor(3),
        )?)
    };
    server.display_arrived(
        DesktopDisplay::from_compositor(1),
        "the-walks-built-in-screen",
        a_screen,
        promises()?,
        &|_| None,
    )?;
    server.display_arrived(
        DesktopDisplay::from_compositor(2),
        "the-walks-docked-screen",
        a_screen,
        Promises::of(
            WindowId::from_compositor(4),
            WindowId::from_compositor(5),
            WindowId::from_compositor(6),
        )?,
        &|_| None,
    )?;
    // A display nobody has divided has no division, and that is what the desk
    // says: `Server::dividing` answers `None`. A `Division` is not `Clone` — it
    // carries an identity and a count of its changes that two copies would both
    // claim — so what the frame is handed is the undivided tree of this display,
    // which is what the session holds.
    if server
        .dividing(DesktopDisplay::from_compositor(1))
        .is_some()
    {
        return Err("a display nobody divided came back divided".into());
    }
    let division = alo_dividing::Division::of(a_screen);
    let nothing_offered = alo_dividing::Offer::Nothing;

    // **Fixed readings, deliberately.** What a person sees here comes from
    // `alo-power`, `alo-networks`, `alo-sound` and `alo-formats` on their own
    // machine and is handed to the shell; this walk asks whether the surface
    // draws, not what this machine's battery is.
    let status = alo_shell::StatusItems::shown(
        "09:41".to_owned(),
        Some(alo_power::Reading::taken(
            alo_power::reading::Charge::reported(64)?,
            alo_power::reading::Charging::Discharging,
            None,
            std::time::SystemTime::UNIX_EPOCH,
        )),
        alo_networks::Reaching::reported(
            alo_networks::HowFar::AllOfIt,
            alo_networks::Metered::NotSaid,
        ),
        Some(alo_sound::Volume::of(35)?),
    );
    // Everything the desktop is handed that does not change between these four
    // steps, in one place, so each step names only what is different about it.
    let desk = TheDesk {
        dock: &dock,
        look,
        strings: &words,
        egress: &egress,
        running: &running,
        filling: &filling,
        status: &status,
        division: &division,
        offer: &nothing_offered,
    };

    nested.pump()?;
    nested.submit_with_desktop(
        &[],
        &[],
        &alo_shell::Cursor::Default,
        None,
        &mut labels,
        desk.frame(None, &[]),
        None,
        None,
    )?;
    met.push(Step::of(
        "a second display docked, and the desktop after it",
        nested
            .the_frame_just_drawn()
            .ok_or("the docked-display frame was not kept")?,
    ));

    // **A capture, with a blur over part of it.** `alo-capturing` decides what
    // the capture covers and what has been drawn on it; the tools this compositor
    // draws are its answers laid out. The blur is a `Mark` like any other here:
    // what makes it destructive is `alo_shell::burnt_in`, at the moment the
    // picture is saved, and that is `crate::capture_flatten`'s business rather
    // than this frame's.
    let whole_screen = What::TheWholeScreen;
    let marks = [Mark::Blur {
        over: Region::of(240, 160, 420, 260)?,
    }];
    nested.pump()?;
    nested.submit_with_desktop(
        &[],
        &[],
        &alo_shell::Cursor::Default,
        None,
        &mut labels,
        desk.frame(
            Some(alo_shell::Capturing {
                choosing: Some(&whole_screen),
                marked: &marks,
            }),
            &[],
        ),
        None,
        None,
    )?;
    met.push(Step::of(
        "a capture of the screen, with a blur marked on it",
        nested
            .the_frame_just_drawn()
            .ok_or("the capture frame was not kept")?,
    ));

    // **A notification arrives.** It goes through the one door that makes a
    // `Shown` — `alo_notifying::deciding::arrives` — so a machine that is locked,
    // shared or quiet holds it and nothing reaches this frame. There is nothing
    // for the compositor to decide, which is why this walk asks the crate rather
    // than building a card.
    let mut seat = an_open_seat()?;
    let mut missed = alo_notifying::Missed::nothing();
    let arrived = arrives(
        from_alo_os(
            "A screenshot was saved",
            "In Pictures, as a PNG.",
            &[("Show me", "show")],
        )
        .map_err(|why| format!("{why:?}"))?,
        &mut seat,
        Quiet::No,
        &mut missed,
    );
    let showing = match arrived {
        alo_notifying::Became::Shown(shown) => [shown],
        alo_notifying::Became::Held(why) => {
            return Err(format!("an open machine held a notification: {why:?}").into());
        }
    };
    nested.pump()?;
    nested.submit_with_desktop(
        &[],
        &[],
        &alo_shell::Cursor::Default,
        None,
        &mut labels,
        desk.frame(None, &showing),
        None,
        None,
    )?;
    met.push(Step::of(
        "a notification arrives on the desktop",
        nested
            .the_frame_just_drawn()
            .ok_or("the notification frame was not kept")?,
    ));
    Ok(())
}

/// A seat nobody has locked, for the one account this walk makes.
#[cfg(target_os = "linux")]
fn an_open_seat()
-> Result<alo_locking::Seat<alo_notifying::Notification>, Box<dyn std::error::Error>> {
    let mut store = alo_accounts::Accounts::none()?;
    store.created("ada", 1000, "the walk's own password")?;
    let who = store
        .signs_in("ada", "the walk's own password")
        .map_err(|why| format!("{why:?}"))?;
    Ok(alo_locking::Seat::opened(
        alo_accounts::Session::opened(who, 1000).map_err(|why| format!("{why:?}"))?,
    ))
}

/// The parts of a desktop frame that do not change across this walk's steps.
///
/// Held together so each step names only what is different about it — a
/// capture being marked, a notification arriving — rather than restating nine
/// borrows and inviting one of them to drift between steps.
#[cfg(target_os = "linux")]
struct TheDesk<'a> {
    /// The person's dock.
    dock: &'a alo_dock::Dock,
    /// Their appearance at this moment, and the way they read.
    look: alo_shell::DesktopLook,
    /// The vocabulary they read.
    strings: &'a alo_strings::Strings,
    /// What the egress indicator was last told.
    egress: &'a alo_shell::EgressStatus,
    /// The window of what is running.
    running: &'a alo_shell::RunningWindow,
    /// The window of what is filling the disk.
    filling: &'a alo_shell::FillingWindow,
    /// The clock, battery, network and volume.
    status: &'a alo_shell::StatusItems,
    /// How this display is divided.
    division: &'a alo_dividing::Division,
    /// What a drop would do, which is nothing here.
    offer: &'a alo_dividing::Offer,
}

#[cfg(target_os = "linux")]
impl<'a> TheDesk<'a> {
    /// This desktop, with the two things a step changes about it.
    fn frame(
        &self,
        capturing: Option<alo_shell::Capturing<'a>>,
        notifications: &'a [alo_notifying::Shown],
    ) -> alo_shell::DesktopFrame<'a> {
        alo_shell::DesktopFrame {
            dock: self.dock,
            look: self.look,
            strings: self.strings,
            egress: self.egress,
            running: self.running,
            filling: self.filling,
            status: self.status,
            in_use: &[],
            notifications,
            capturing,
            division: self.division,
            offer: self.offer,
        }
    }
}

/// **Divide the screen** between two real windows, and draw the result.
///
/// The one step that needs real clients, because a division divides *between*
/// windows: `alo_dividing::Division::divide_with_next` takes the focused window
/// and the next one in switch order, and with one window open it refuses by
/// name. So this maps two, focuses one, presses the chord's road, and draws the
/// display the division decided — every rectangle in it `alo-dividing`'s and
/// none of it this compositor's.
#[cfg(target_os = "linux")]
fn divide_between_two_windows(
    nested: &mut alo_shell::Nested,
    met: &mut Vec<Step>,
) -> Result<(), Box<dyn std::error::Error>> {
    use alo_desktops::DisplayId as DesktopDisplay;
    use alo_dividing::Side;
    use alo_shell::Server;
    use std::os::unix::fs::PermissionsExt as _;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    let runtime = tempfile::tempdir()?;
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "the-walk-dividing",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..Default::default()
        },
    )?;
    server.render(nested, 0)?;

    // **No `display_arrived` here, and that is the point.** The frame submitted
    // above already told the session it had a display: `Server::render` reaches
    // `crate::display_lifecycle`, which is what gives a display its desktops and
    // restores the division it left. Calling it again is refused by name —
    // *display 1 already has desktops, which plugging it in again would
    // discard* — which is how this step found out it did not need to.
    if server
        .desktops_on(DesktopDisplay::from_compositor(1))
        .is_none()
    {
        return Err("a submitted frame did not give the session its display".into());
    }

    // Two clients, each one window, mapped and configured. Two rather than one
    // window on one client, because what a person divides is two applications'
    // windows and the switch order this walks is the order they opened in.
    let fixture = Fixture {
        path: server.socket_path().to_owned(),
    };
    let (ready, mapped) = mpsc::channel::<()>();
    let (hold, released) = mpsc::channel::<()>();
    let (settled, took_their_shares) = mpsc::channel::<()>();
    let client = std::thread::spawn(move || {
        let mut first = application::Application::new(&fixture);
        first.configure();
        first.attach();
        first.sync();
        let mut second = application::Application::new(&fixture);
        second.configure();
        second.attach();
        second.sync();
        assert!(ready.send(()).is_ok());
        // Stay connected while the server divides: a client that disconnected
        // here would take its window with it, and the division would then
        // refuse with *nothing to share with* — which is what happened when
        // this patience was ten seconds. Software rendering a 1366×768 frame
        // per pump is slower than that, and a client's deadline that is shorter
        // than the server's work is a test failing about its own timing.
        assert!(released.recv_timeout(Duration::from_secs(120)).is_ok());
        // **Take the share the division gave.** A client that never answers its
        // configure keeps the buffer it had, and the frame then shows one
        // 16×16 window where a divided screen should be — which is exactly what
        // this step drew before these two lines existed. Acknowledging and
        // attaching again is what a real application does with a configure, and
        // it is the difference between a division in a tree and a division on a
        // screen.
        first.configure();
        first.attach();
        first.sync();
        second.configure();
        second.attach();
        second.sync();
        assert!(settled.send(()).is_ok());
    });

    let waiting = Instant::now();
    while server.mapped_surfaces().count() < 2 {
        if waiting.elapsed() > Duration::from_secs(120) {
            return Err("two clients did not map within the deadline".into());
        }
        server.dispatch()?;
        nested.pump()?;
        server.render(nested, 0)?;
    }
    let _ = mapped.recv_timeout(Duration::from_secs(120));

    let focused = server
        .mapped_surfaces()
        .next()
        .cloned()
        .ok_or("no window to focus")?;
    server.keyboard_focus(Some(&focused))?;
    server.divide_focused_with_next(&focused, Side::Left)?;

    let division = server
        .dividing(DesktopDisplay::from_compositor(1))
        .ok_or("the chord divided nothing")?;
    let shares = division.shares();
    if shares.len() != 2 {
        return Err(format!("a division between two windows has {} shares", shares.len()).into());
    }
    // Let the clients answer their configures, dispatching while they do: the
    // only thread that answers a roundtrip is this one, so a wait that does not
    // dispatch is a deadlock.
    let _ = hold.send(());
    let settling = Instant::now();
    loop {
        server.dispatch()?;
        nested.pump()?;
        match took_their_shares.try_recv() {
            Ok(()) => break,
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err("the walk's clients left before taking their shares".into());
            }
        }
        if settling.elapsed() > Duration::from_secs(60) {
            return Err("the walk's clients did not take their shares".into());
        }
    }

    let roots: Vec<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface> =
        server.mapped_surfaces().cloned().collect();
    if roots.len() != 2 {
        return Err(format!("{} windows were drawn, not two", roots.len()).into());
    }
    // Both windows keep the 16×16 buffer the client fixture makes: what this
    // shows is that each is drawn at **the origin its share gave it**, not that
    // a client grew into its share. A client that resizes to the size it was
    // configured with is what the probe's removed tiling stages did and what
    // task 17 of the shell plan is for.
    nested.pump()?;
    server.render(nested, 0)?;
    met.push(Step::of(
        "the screen divided between two windows",
        nested
            .the_frame_just_drawn()
            .ok_or("the divided frame was not kept")?,
    ));

    let leaving = Instant::now();
    while !client.is_finished() {
        if leaving.elapsed() > Duration::from_secs(60) {
            return Err("the walk's clients did not finish".into());
        }
        server.dispatch()?;
        nested.pump()?;
    }
    client.join().map_err(|_| "the walk's clients refused")?;
    Ok(())
}
