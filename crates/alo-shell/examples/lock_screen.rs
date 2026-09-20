//! Developer-only nested lock probe using the real renderer and approved artwork.
//! This creates an in-memory account and never changes the host's accounts or image.

/// Run on Linux with a real Wayland parent. Never claims certified-machine evidence.
#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::{Appearance, Background, DisplayId, Picture, Scheme, TextScale};
    use alo_shell::{
        Contrast, FrameTarget, LockBackground, LockLook, LockSurface, Nested, SignInLook,
        WindowControlLabels,
    };
    use std::time::{Duration, SystemTime};
    let mut accounts = alo_accounts::Accounts::none()?;
    accounts
        .created("probe", 1000, "local rendering probe password")
        .map_err(|_| "cannot create probe account")?;
    let signed = accounts
        .signs_in("probe", "local rendering probe password")
        .map_err(|_| "probe sign-in refused")?;
    let session =
        alo_accounts::Session::opened(signed, 1000).map_err(|_| "probe session refused")?;
    let words = alo_strings::Strings::of(alo_saying::everything_this_machine_can_say()?);
    let seat =
        alo_locking::Seat::<String>::opened(session).locked(&mut alo_overlay::Summoning::closed());
    let mut surface = LockSurface::of(seat, words.clone()).map_err(|_| "seat was not locked")?;
    surface.arrives("private notification never drawn".into());
    let mut appearance = Appearance::shipped();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/artwork/wallpapers/alo-quiet-horizon.png")
        .canonicalize()?;
    appearance.set_background(Background::Picture(
        Picture::file(path).map_err(|_| "invalid asset path")?,
    ));
    let display = DisplayId::named("nested-lock-probe").map_err(|_| "invalid probe display")?;
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
    let mut nested = Nested::new("alo lock-screen development probe", (800, 600))?;
    let mut labels = WindowControlLabels::new()?;
    for tick in 0..10 {
        if tick == 5 {
            surface = match surface.pressed(alo_shell::SignInKey::Enter, || {
                Err(alo_greeting::NotReadable {
                    at: "unused".into(),
                    why: "no submission".into(),
                })
            }) {
                alo_shell::LockPressed::Still(surface) => *surface,
                alo_shell::LockPressed::Opened { .. } => {
                    return Err("opened without credentials".into());
                }
            };
        }
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
        if tick == 8 {
            let mismatch = LockBackground::prepare(screen.image(), (320, 480), Duration::ZERO)?;
            if nested
                .submit_lock(&surface, &screen, &mismatch, &mut labels, &look)
                .is_ok()
            {
                return Err("a stale output-sized background was accepted".into());
            }
        }
        nested.submit_lock(&surface, &screen, &background, &mut labels, &look)?;
        std::thread::sleep(Duration::from_millis(50));
    }
    println!(
        "Ten opaque nested lock frames submitted; private notifications held; physical display unverified."
    );
    Ok(())
}

/// Linux owns this development probe.
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("the lock-screen probe requires Linux and Wayland");
}
