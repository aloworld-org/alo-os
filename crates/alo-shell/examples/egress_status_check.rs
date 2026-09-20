//! The egress indicator on a real nested compositor under a Wayland parent.
//!
//! Explicit developer fixture, never installed as anybody's session. It
//! refuses a frame before the status area is told what is leaving, then draws
//! a quiet machine, a question put to a provider, one put to a paired machine,
//! alo OS's own errand beside an agent's egress, and more departures than the
//! window has room for — on the dock's four edges, light and dark — through the
//! parent's EGL, and ends on a quiet machine again. Every departure is really
//! permitted by an `alo_egress::Indicator` and handed over by
//! `alo_indicator::Indicating`. A virtual output proves the drawing path, not
//! a panel: a certified machine has not seen this surface.

/// Main-thread graphics initialisation is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("egress indicator check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Draw each standing on every edge in both schemes.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::{Scheme, TextScale};
    use alo_capability::Grantee;
    use alo_dock::{Dock, Edge};
    use alo_egress::{Destination, EgressPolicy, Errand, Indicator, Leaving, OnItsOwn, Why};
    use alo_indicator::{Drew, Indicating};
    use alo_models::{InferenceSource, Region};
    use alo_shell::{
        Contrast, Cursor, EgressStatus, EgressStatusFrame, EgressStatusLook, Nested, RenderError,
        WindowControlLabels,
    };
    use alo_strings::{Direction, Strings};
    use std::time::{Duration, Instant, SystemTime};

    let strings = Strings::of(alo_saying::everything_this_machine_can_say()?);
    let mut nested = Nested::new("alo egress indicator check", (1366, 768))?;
    nested.pump()?;
    let mut labels = WindowControlLabels::new()?;
    let mut status = EgressStatus::on_an_output();
    let mut indicating = Indicating::nowhere();
    let mut indicator = Indicator::default();
    let now = SystemTime::now();
    let mail = Grantee::named("@mail");

    let mut dock = Dock::shipped();
    let look = |scheme| EgressStatusLook {
        contrast: Contrast::AsDesigned,
        scheme,
        scale: TextScale::ordinary(),
        reading: Direction::LeftToRight,
    };
    match nested.submit_with_egress_status(
        &[],
        &[],
        &Cursor::Default,
        None,
        &mut labels,
        EgressStatusFrame {
            status: &status,
            strings: &strings,
            dock: &dock,
            look: look(Scheme::Light),
        },
    ) {
        Err(RenderError::EgressStatusUnknown) => {
            println!("Refused a frame whose indicator had not been told what is leaving");
        }
        other => return Err(format!("an untold indicator was not refused: {other:?}").into()),
    }

    let mut submitted = 0;
    let mut draw = |what: &str,
                    indicator: &Indicator,
                    status: &mut EgressStatus,
                    indicating: &mut Indicating,
                    dock: &mut Dock,
                    nested: &mut Nested|
     -> Result<(), Box<dyn std::error::Error>> {
        if let Drew::Refused(refused) = indicating.show(Some(status), indicator) {
            return Err(refused.said(&strings).into_text().into());
        }
        for edge in Edge::ALL {
            dock.set_edge(edge);
            for scheme in [Scheme::Light, Scheme::Dark] {
                let until = Instant::now() + Duration::from_millis(120);
                while Instant::now() < until {
                    nested.pump()?;
                    nested.submit_with_egress_status(
                        &[],
                        &[],
                        &Cursor::Default,
                        None,
                        &mut labels,
                        EgressStatusFrame {
                            status,
                            strings: &strings,
                            dock,
                            look: look(scheme),
                        },
                    )?;
                    submitted += 1;
                    std::thread::sleep(Duration::from_millis(16));
                }
            }
        }
        println!("Drew {what} on every edge, light and dark");
        for line in indicator.showing() {
            println!("  {}", line.said(&strings).text());
        }
        Ok(())
    };

    draw(
        "a quiet machine",
        &indicator,
        &mut status,
        &mut indicating,
        &mut dock,
        &mut nested,
    )?;

    let provider = indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::asking(
                &mail,
                &InferenceSource::Hosted {
                    provider: "alo".to_owned(),
                    region: Region::Declared("the EU".to_owned()),
                },
            )
            .map_err(|refused| refused.said(&strings).into_text())?,
            now,
        )
        .map_err(|refused| refused.said(&strings).into_text())?;
    draw(
        "a question put to a provider",
        &indicator,
        &mut status,
        &mut indicating,
        &mut dock,
        &mut nested,
    )?;
    indicator.ended(provider);

    let paired = indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::asking(
                &mail,
                &InferenceSource::PairedMachine {
                    machine: "workstation-2".to_owned(),
                },
            )
            .map_err(|refused| refused.said(&strings).into_text())?,
            now,
        )
        .map_err(|refused| refused.said(&strings).into_text())?;
    let errand = indicator.beginning_on_its_own(
        OnItsOwn::for_(
            Errand::CheckingForAnUpdate,
            Destination::at("updates.alo.example")
                .map_err(|refused| refused.said(&strings).into_text())?,
        ),
        now,
    );
    draw(
        "a paired machine beside alo OS's own errand",
        &indicator,
        &mut status,
        &mut indicating,
        &mut dock,
        &mut nested,
    )?;

    let mut many = Vec::new();
    for _ in 0..30 {
        many.push(
            indicator
                .beginning(
                    &EgressPolicy::Anywhere,
                    Leaving::because(
                        &Grantee::named("@files"),
                        Why::Fetching,
                        Destination::at("alo.example")
                            .map_err(|refused| refused.said(&strings).into_text())?,
                    ),
                    now,
                )
                .map_err(|refused| refused.said(&strings).into_text())?,
        );
    }
    draw(
        "more than the window holds",
        &indicator,
        &mut status,
        &mut indicating,
        &mut dock,
        &mut nested,
    )?;

    for departing in many {
        indicator.ended(departing);
    }
    indicator.ended(paired);
    indicator.ended_on_its_own(errand);
    draw(
        "a quiet machine again",
        &indicator,
        &mut status,
        &mut indicating,
        &mut dock,
        &mut nested,
    )?;

    println!(
        "{submitted} egress indicator frames submitted through the parent's EGL; physical display unverified"
    );
    Ok(())
}
