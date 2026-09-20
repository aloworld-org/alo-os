//! The ordinary desktop on a real nested compositor under a Wayland parent.
//!
//! Explicit developer fixture, never installed as anybody's session. It draws:
//! the dock alone with nothing leaving; the dock with a question leaving for a
//! provider, its line growing from the status area; the window of what is
//! running, opened on two readings of this machine's own kernel and read again;
//! the window of what is filling a folder on a temporary disk, with a folder
//! opened; both windows at once; and a folder that is gone — on all four dock
//! edges, light at nine and dark at eight in the evening as a person's
//! appearance decides, read left to right and right to left. A virtual output
//! proves the drawing path, not a panel: a certified machine has not seen this
//! desktop.

/// Main-thread graphics initialisation is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("desktop check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Draw every standing of the desktop in both schemes and both directions.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_access::TurnedOn;
    use alo_appearance::{Accent, Appearance, Following, Shipped, TimeOfDay};
    use alo_capability::Grantee;
    use alo_dock::{Dock, Edge};
    use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
    use alo_indicator::{Drew, Indicating};
    use alo_measuring::Reading;
    use alo_models::Region;
    use alo_shell::{
        Cursor, DesktopFrame, DesktopLook, EgressStatus, FillingKey, FillingShows, FillingWindow,
        Nested, RunningShows, RunningWindow, WindowControlLabels,
    };
    use alo_strings::{Direction, Strings};
    use std::time::{Duration, Instant};

    let strings = Strings::of(alo_saying::everything_this_machine_can_say()?);
    let mut appearance = Appearance::shipped();
    appearance.follow(Following::from(Shipped::the_evening_schedule()));
    appearance.set_accent(Accent::Indigo);
    let times = [
        TimeOfDay::checked(9, 0).map_err(|error| format!("{error:?}"))?,
        TimeOfDay::checked(20, 0).map_err(|error| format!("{error:?}"))?,
    ];

    let held = tempfile::tempdir()?;
    let documents = held.path().join("Documents");
    std::fs::create_dir_all(documents.join("letters").join("old"))?;
    std::fs::write(
        documents.join("letters").join("to-ada.txt"),
        vec![b'a'; 1000],
    )?;
    std::fs::write(
        documents.join("letters").join("old").join("draft.txt"),
        vec![b'd'; 500],
    )?;
    std::fs::write(documents.join("photo.jpg"), vec![0; 40_000])?;

    let mut nested = Nested::new("alo desktop check", (1366, 768))?;
    nested.pump()?;
    let mut labels = WindowControlLabels::new()?;

    let mut quiet = EgressStatus::on_an_output();
    if let Drew::Refused(refused) =
        Indicating::nowhere().show(Some(&mut quiet), &Indicator::default())
    {
        return Err(refused.said(&strings).into_text().into());
    }
    let mut indicator = Indicator::default();
    let a_provider = Destination::provider("alo", Region::Declared("the EU".to_owned()))
        .map_err(|error| format!("{error:?}"))?;
    let departing = indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::because(&Grantee::named("@mail"), Why::Asking, a_provider),
            std::time::SystemTime::now(),
        )
        .map_err(|error| format!("{error:?}"))?;
    let mut lit = EgressStatus::on_an_output();
    if let Drew::Refused(refused) = Indicating::nowhere().show(Some(&mut lit), &indicator) {
        return Err(refused.said(&strings).into_text().into());
    }

    let mut submitted = 0;
    let mut draw = |what: &str,
                    egress: &EgressStatus,
                    running: &RunningWindow,
                    filling: &FillingWindow,
                    nested: &mut Nested|
     -> Result<(), Box<dyn std::error::Error>> {
        for edge in Edge::ALL {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);
            for now in times {
                for reading in [Direction::LeftToRight, Direction::RightToLeft] {
                    let until = Instant::now() + Duration::from_millis(60);
                    while Instant::now() < until {
                        nested.pump()?;
                        nested.submit_with_desktop(
                            &[],
                            &[],
                            &Cursor::Default,
                            None,
                            &mut labels,
                            DesktopFrame {
                                dock: &dock,
                                look: DesktopLook::of(
                                    &appearance,
                                    &TurnedOn::nothing(),
                                    now,
                                    reading,
                                ),
                                strings: &strings,
                                egress,
                                running,
                                filling,
                            },
                            None,
                            None,
                        )?;
                        submitted += 1;
                        std::thread::sleep(Duration::from_millis(16));
                    }
                }
            }
        }
        let running_said = match running.shows() {
            RunningShows::Nothing => "closed".to_owned(),
            RunningShows::Running { running, .. } => format!(
                "{} running and {} ended",
                running.processes().len(),
                running.gone().len()
            ),
            RunningShows::Refusal(why) => format!("\"{}\"", why.said(&strings).text()),
        };
        let filling_said = match filling.shows() {
            FillingShows::Nothing => "closed".to_owned(),
            FillingShows::Holding {
                holding, opened, ..
            } => format!(
                "{} bytes in {}, {} folders open",
                holding.tree.size,
                holding.tree.name,
                opened.len()
            ),
            FillingShows::Refusal(why) => format!("\"{}\"", why.said(&strings).text()),
        };
        println!("Drew {what}: running {running_said}; filling {filling_said}");
        Ok(())
    };

    let closed_running = RunningWindow::closed();
    let closed_filling = FillingWindow::closed();
    draw(
        "the dock alone",
        &quiet,
        &closed_running,
        &closed_filling,
        &mut nested,
    )?;
    draw(
        "the dock with a question leaving",
        &lit,
        &closed_running,
        &closed_filling,
        &mut nested,
    )?;

    let earlier = Reading::now();
    let taken = Instant::now();
    std::thread::sleep(Duration::from_millis(500));
    let mut running = RunningWindow::closed();
    running.opened(earlier, Reading::now(), taken.elapsed());
    draw(
        "what is running",
        &quiet,
        &running,
        &closed_filling,
        &mut nested,
    )?;
    let again = Instant::now();
    std::thread::sleep(Duration::from_millis(500));
    running.read_again(Reading::now(), again.elapsed());
    draw(
        "what is running, read again",
        &lit,
        &running,
        &closed_filling,
        &mut nested,
    )?;

    let mut filling = FillingWindow::closed();
    filling.opened(&documents);
    draw(
        "what is filling a folder",
        &quiet,
        &closed_running,
        &filling,
        &mut nested,
    )?;
    filling.pressed(FillingKey::Next);
    filling.pressed(FillingKey::Open);
    draw(
        "a folder opened",
        &quiet,
        &closed_running,
        &filling,
        &mut nested,
    )?;
    draw("both windows", &lit, &running, &filling, &mut nested)?;

    std::fs::remove_dir_all(&documents)?;
    filling.pressed(FillingKey::CountAgain);
    draw(
        "a folder that is gone",
        &quiet,
        &running,
        &filling,
        &mut nested,
    )?;

    if !indicator.ended(departing) {
        return Err("the departure did not end".into());
    }
    println!("{submitted} frames submitted");
    Ok(())
}
