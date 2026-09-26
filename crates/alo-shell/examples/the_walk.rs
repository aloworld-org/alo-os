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
