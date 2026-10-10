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
//!
//! # `--save-to <folder>`, and what the pictures in it are
//!
//! Off unless asked for. With it, one picture is written for each standing in
//! each scheme and each reading — what was drawn, rather than the count of
//! frames that is all this otherwise reports. It exists because thirteen
//! fixtures in this folder draw and **nobody has said what a person makes of
//! them**: whether the dock reads as a dock, whether the egress line is
//! findable without being told it is there. That is not a test and cannot be
//! one. It needs eyes, and eyes need pictures.
//!
//! # The frame count is not a constant, and must never be asserted
//!
//! The loop that draws one standing ends **on a clock, not on a count**: it
//! submits frames for 60 milliseconds and then stops. So the number this
//! prints is a measure of how fast the machine was, and it is different every
//! time. Measured here on one machine in one sitting, with `--save-to` off:
//!
//! ```text
//! 32 frames submitted
//! 34 frames submitted
//! 33 frames submitted
//! ```
//!
//! This is written down because the number looks like a property and is not
//! one. `docs/autonomy/updates/what-the-probes-draw-today.md` records *33
//! frames* from an earlier run, and that is one sample of a varying quantity
//! rather than a fact about the renderer. **A test that asserts it will go red
//! on a slower machine**, and the person reading this file will be the one
//! trying to work out why.
//!
//! Keeping a frame paints the scene a second time, so `--save-to` should cost
//! frames — both runs with it on gave 32. **That is inside the spread of the
//! runs with it off, so this says the flag is not free and does not claim to
//! have measured what it costs.**
//!
//! # Three things are true of every picture
//!
//! The first is the one that matters to anybody reading this file later:
//!
//! 1. **it is not the screenshot promise.** That is a `[v0.5]` feature with a
//!    person's own capture, a grant and a record. Nothing a person or an agent
//!    can reach gained the ability to write a frame to disk — see
//!    `support/saving_frames.rs`, which says exactly who can ask;
//! 2. **it is what the renderer produced, not what reached a screen.**
//!    `alo_shell::Nested::keep_each_frame` paints the same scene a second time
//!    into a buffer that can be read, and `alo-shell`'s readback is documented
//!    as supplying *unbound uploads, not presentation or flips*;
//! 3. **it is software rendered here.** Under WSLg Mesa falls back —
//!    `ZINK: failed to choose pdev` — so the geometry, the text layout and the
//!    colours are the design's, while filtering and blending need not match
//!    what a graphics card would do.

/// Writing a drawn frame out, for a fixture asked to with `--save-to`.
#[cfg(target_os = "linux")]
#[path = "support/saving_frames.rs"]
mod saving_frames;

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
    use alo_dock::Dock;
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
    // Each hour carries the scheme a person's own evening schedule puts it in,
    // so that a saved picture is named by what somebody looking at it sees
    // rather than by a number they would have to go and work out.
    let times = [
        (
            "light at 9am",
            TimeOfDay::checked(9, 0).map_err(|error| format!("{error:?}"))?,
        ),
        (
            "dark at 8pm",
            TimeOfDay::checked(20, 0).map_err(|error| format!("{error:?}"))?,
        ),
    ];

    // A name for the one display this fixture draws on. The appearance answers
    // a background per display and has no other accessor, so a name is
    // required; with one display it is only a key.
    let the_one_display = alo_appearance::DisplayId::named("the-one-display")
        .map_err(|error| format!("{error:?}"))?;
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

    // Asked for before anything is drawn, so a mistyped folder stops the run
    // at the start rather than after two minutes of drawing nobody kept.
    let mut saving = saving_frames::SaveTo::from_arguments()?;

    let mut nested = Nested::new("alo desktop check", (1366, 768))?;
    if saving.is_some() {
        // A second painting of each frame into a buffer that can be read. Left
        // on for the whole run rather than turned on for a last frame: the
        // frames of one standing are drawn in a loop that ends on a clock, so
        // which one is last is not known until it has been.
        nested.keep_each_frame(true);
    }
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

    // **Three applications for the Dock to hold**, built the way `alo-dock`
    // would hand them over rather than assembled by hand: `OnTheDock` has no
    // public constructor and should not gain one for a fixture, because a
    // picture of a Dock showing something the Dock would never show is worse
    // than no picture.
    //
    // Their names are ordinary ones a person would recognise, and their first
    // letters differ — which is what makes the picture show three icons rather
    // than one repeated.
    let a_few_applications = {
        let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1)
            .map_err(|error| format!("{error:?}"))?;
        let mut windows = alo_dock::Windows::none();
        for (number, named) in ["Files", "Mail", "Browser"].into_iter().enumerate() {
            windows.opened(alo_dock::Window::of(
                alo_dock::WindowId::numbered(number as u64),
                Some(alo_dock::AppId::named(named).map_err(|error| format!("{error:?}"))?),
                named,
                patch,
                alo_dock::HowItSits::OnTheCanvas,
            ));
        }
        alo_dock::Holding::nothing().showing(&windows)
    };

    let mut submitted = 0;
    let mut draw = |what: &str,
                    egress: &EgressStatus,
                    running: &RunningWindow,
                    filling: &FillingWindow,
                    nested: &mut Nested|
     -> Result<(), Box<dyn std::error::Error>> {
        // A display nobody has divided, which is what this probe's is: the
        // Server holds no division yet and the session that will is a task of
        // its own.
        let division = alo_dividing::Division::of(alo_dividing::Area::of(
            alo_dividing::area::Point::at(0, 0),
            alo_dividing::area::Size::of(1920, 1080),
        )?);
        {
            let dock = Dock::shipped();
            for (scheme, now) in times {
                for (which_way, reading) in [
                    ("read left to right", Direction::LeftToRight),
                    ("read right to left", Direction::RightToLeft),
                ] {
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
                                display_scale: 100,
                                // **Three applications, so there is a Dock to
                                // look at.** This held nothing until
                                // 2026-10-10, with the note *deliberately not
                                // raised to show a wider bar — §8 says it must
                                // not reserve a large empty bar, and an
                                // icon-less wide band is exactly that.* That
                                // reasoning was right and it was about an
                                // **empty** bar: nothing drew an icon, so a
                                // wider band would have been reserved room with
                                // nothing in it.
                                //
                                // Icons are drawn now — an application with no
                                // artwork shows its first letter, by the
                                // owner's ruling of that day — so a bar holding
                                // three is three icons wide and not a reserved
                                // emptiness. Which is the whole point of this
                                // fixture: *it needs eyes, and eyes need
                                // pictures.*
                                on_the_dock: &a_few_applications,
                                dock: &dock,
                                look: DesktopLook::of(
                                    &appearance,
                                    &TurnedOn::nothing(),
                                    now,
                                    reading,
                                    // A name, because the appearance
                                    // answers per display and has no
                                    // other accessor. This fixture has
                                    // one display, so any name serves.
                                    &the_one_display,
                                ),
                                strings: &strings,
                                egress,
                                running,
                                filling,
                                in_use: &[],
                                notifications: &[],
                                capturing: None,
                                division: &division,
                                offer: &alo_dividing::Offer::Nothing,
                                windows: &[],
                                put_aside: nothing_put_aside(),
                                // A fixture that is not about revealing draws the panel, so what it lays out is
                                // the rail rather than an empty column.
                                panel_is_revealed: true,
                                filling_the_screen: false,
                            },
                            None,
                            None,
                        )?;
                        submitted += 1;
                        std::thread::sleep(Duration::from_millis(16));
                    }
                    // One picture for this standing in this scheme and this
                    // reading, after its frames have been submitted — not one
                    // per frame, which would be the same picture four times.
                    if let Some(saving) = saving.as_mut() {
                        let frame = nested
                            .the_frame_just_drawn()
                            .ok_or("keeping was asked for and no frame was kept")?;
                        let at = saving.frame(&format!("{what}, {scheme}, {which_way}"), frame)?;
                        println!("  wrote {}", at.display());
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
    if let Some(saving) = saving {
        println!(
            "{} pictures of what was drawn in {}",
            saving.written(),
            saving.folder().display()
        );
        // Said every time a picture is written, next to the pictures, because
        // this is the sentence that stops one of them being shown as proof
        // that a machine displayed the desktop. It has not.
        println!(
            "software rendered into a buffer that was read back, on a virtual output: \
             what the renderer drew, not what a screen showed"
        );
    }
    Ok(())
}
/// A desk where nothing has been put aside.
///
/// One shared value rather than a temporary at each site, so the borrow does
/// not outlive the panel it names. An empty panel is the true answer for a
/// check that is not about the panel: a person who has put nothing aside has
/// one, and no rail is drawn for it.
fn nothing_put_aside() -> &'static alo_put_aside::Panel {
    static EMPTY: std::sync::OnceLock<alo_put_aside::Panel> = std::sync::OnceLock::new();
    EMPTY.get_or_init(alo_put_aside::Panel::new)
}
