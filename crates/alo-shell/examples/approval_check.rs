//! The approval surface on a real nested compositor under a Wayland parent.
//!
//! Explicit developer fixture, never installed as anybody's session. It stands
//! a real turn up over two temporary folders, proposes three moves through the
//! file verbs, and draws: nothing while nothing is open, the first question
//! with nothing selected, the same with an answer selected, the second question
//! after the first was approved and the file really moved, a refusal in the
//! turn's own words after an approval with no grant behind it, and nothing
//! again — in light and dark, read left to right and right to left, with the
//! egress indicator carried by every frame. A virtual output proves the drawing
//! path, not a panel: a certified machine has not seen this surface.

/// Main-thread graphics initialisation is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("approval surface check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Draw every standing of the surface in both schemes and both directions.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::{Scheme, TextScale};
    use alo_capability::{Given, Grant, Grants, Reach};
    use alo_context::Context;
    use alo_dock::Dock;
    use alo_egress::Indicator;
    use alo_files::{OnThisMachine, Reaching, Resolving};
    use alo_indicator::{Drew, Indicating};
    use alo_record::Record;
    use alo_shell::{
        ApprovalAnswer, ApprovalFrame, ApprovalKey, ApprovalLook, ApprovalScreen, ApprovalShows,
        Cursor, EgressStatus, EgressStatusFrame, EgressStatusLook, Nested, WindowControlLabels,
    };
    use alo_strings::{Direction, Strings};
    use alo_turn::{Bounding, Doing, Done, Machine, NoBoundary, Turning};
    use std::time::{Duration, Instant, SystemTime};

    /// What `alo_turn::bounding` says a test writes where its reader sees it.
    struct NothingIsBounded;
    impl Bounding for NothingIsBounded {
        fn carrying_out(&mut self, _: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
            Ok(doing.done())
        }
        fn carrying_out_a_departure(
            &mut self,
            _: &[std::net::SocketAddr],
            doing: &mut dyn FnMut(),
        ) -> Result<(), NoBoundary> {
            doing();
            Ok(())
        }
    }

    let strings = Strings::of(alo_saying::everything_this_machine_can_say()?);
    let held = tempfile::tempdir()?;
    let root = Resolving::real(&OnThisMachine, held.path())
        .map_err(|error| format!("{error:?}"))?
        .into_path_buf();
    let invoices = root.join("invoices");
    let archive = root.join("archive");
    std::fs::create_dir_all(&invoices)?;
    std::fs::create_dir_all(&archive)?;
    for month in ["march", "april", "may"] {
        std::fs::write(invoices.join(format!("{month}.pdf")), month)?;
    }

    let now = SystemTime::now();
    let hour = Duration::from_secs(60 * 60);
    let mut grants = Grants::default();
    for folder in [&invoices, &archive] {
        grants.grant(
            Grant::checked("@files", Reach::Folder(folder.clone()), now, hour)
                .map_err(|error| format!("{error:?}"))?,
        );
    }
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        &mut record,
    )
    .map_err(|error| format!("{error:?}"))?;
    let mut turning = Turning::beginning(
        Context::at_invocation(now),
        "@files",
        hour,
        &mut grants,
        &mut machine,
    )
    .map_err(|error| format!("{error:?}"))?;

    let mut nested = Nested::new("alo approval surface check", (1366, 768))?;
    nested.pump()?;
    let mut labels = WindowControlLabels::new()?;
    let dock = Dock::shipped();
    let mut status = EgressStatus::on_an_output();
    if let Drew::Refused(refused) =
        Indicating::nowhere().show(Some(&mut status), &Indicator::default())
    {
        return Err(refused.said(&strings).into_text().into());
    }

    let mut submitted = 0;
    let mut draw = |what: &str,
                    screen: &ApprovalScreen,
                    nested: &mut Nested|
     -> Result<(), Box<dyn std::error::Error>> {
        for scheme in [Scheme::Light, Scheme::Dark] {
            for reading in [Direction::LeftToRight, Direction::RightToLeft] {
                let until = Instant::now() + Duration::from_millis(150);
                while Instant::now() < until {
                    nested.pump()?;
                    nested.submit_with_approval(
                        &[],
                        &[],
                        &Cursor::Default,
                        None,
                        &mut labels,
                        EgressStatusFrame {
                            status: &status,
                            strings: &strings,
                            dock: &dock,
                            look: EgressStatusLook {
                                scheme,
                                scale: TextScale::ordinary(),
                                reading,
                            },
                        },
                        ApprovalFrame {
                            screen,
                            strings: &strings,
                            look: ApprovalLook {
                                scheme,
                                scale: TextScale::ordinary(),
                                reading,
                            },
                        },
                    )?;
                    submitted += 1;
                    std::thread::sleep(Duration::from_millis(16));
                }
            }
        }
        match screen.shows() {
            ApprovalShows::Nothing => println!("Drew {what}: nothing"),
            ApprovalShows::Question { asked, selected } => println!(
                "Drew {what}: {:?} asks \"{}\", selected {selected:?}",
                asked.agent().as_str(),
                asked.sentence().text()
            ),
            ApprovalShows::Refusal(said) => println!("Drew {what}: \"{}\"", said.text()),
        }
        Ok(())
    };

    let mut proposed = Vec::new();
    for month in ["march", "april", "may"] {
        proposed.push(
            turning
                .proposing(
                    "move_file",
                    &[
                        (
                            "file",
                            Given::text(
                                invoices.join(format!("{month}.pdf")).display().to_string(),
                            ),
                        ),
                        ("into", Given::text(archive.display().to_string())),
                    ],
                    &grants,
                    hour,
                    now,
                )
                .map_err(|error| format!("{error:?}"))?,
        );
    }

    let mut screen = ApprovalScreen::on_an_output();
    draw("a surface with nothing open", &screen, &mut nested)?;
    for id in &proposed {
        screen.arrived(&turning, *id, now);
    }
    draw("the first question", &screen, &mut nested)?;

    let still = &grants;
    screen.pressed(ApprovalKey::NextAnswer, &mut turning, still, now);
    screen.pressed(ApprovalKey::NextAnswer, &mut turning, still, now);
    draw(
        "the first question with an answer selected",
        &screen,
        &mut nested,
    )?;

    let carried = screen.pressed(ApprovalKey::Choose, &mut turning, still, now);
    if !invoices.join("march.pdf").exists() && archive.join("march.pdf").exists() {
        println!(
            "Approved once: march.pdf moved ({:?})",
            carried.answered.is_some()
        );
    } else {
        return Err("an approval did not move the file".into());
    }
    draw("the second question", &screen, &mut nested)?;

    screen.answer(
        ApprovalAnswer::Approve,
        &mut turning,
        &Grants::default(),
        now,
    );
    draw(
        "a refusal after an approval with no grant",
        &screen,
        &mut nested,
    )?;

    screen.pressed(ApprovalKey::Choose, &mut turning, still, now);
    screen.answer(ApprovalAnswer::No, &mut turning, still, now);
    draw("a surface after the last no", &screen, &mut nested)?;

    println!("{submitted} frames submitted");
    Ok(())
}
