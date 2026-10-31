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
/// Which edge, as a word for a picture's file name.
///
/// **Not `alo_dock::words`, and that is deliberate.** Those are the four
/// sentences a *person* reads in a settings panel — *Along the bottom*, *Down the
/// left side* — and they are translated. A file name on a developer's disk is not
/// a sentence a person reads, and putting a translated phrase into one would make
/// the name of a picture depend on the language the machine happens to be set to.
///
/// Matched exhaustively rather than through a default arm, so a fifth edge is a
/// compile error here and not a picture called *the dock alone on the  edge*.
fn an_edges_name(edge: alo_dock::Edge) -> &'static str {
    match edge {
        alo_dock::Edge::Bottom => "bottom",
        alo_dock::Edge::Left => "left",
        alo_dock::Edge::Right => "right",
        alo_dock::Edge::Top => "top",
    }
}

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
    // **More applications than any edge of this display can hold**, so the
    // overflow control is on the bar and something is behind it. Thirty is past
    // what a 1366-wide bottom bar holds (23 slots) and well past what a 768-tall
    // side bar does (13), so one list serves all four edges.
    //
    // Their names start with different letters as far as the alphabet goes, for
    // the reason `a_few_applications` gives: a picture of thirty identical
    // letters would not show that the slots hold their own applications.
    let a_crowded_dock = {
        let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1)
            .map_err(|error| format!("{error:?}"))?;
        let mut windows = alo_dock::Windows::none();
        // **Sixty, which is the number the owner asks this to be checked at.**
        // *Sixty applications must scroll inside the same bounded viewport*,
        // and thirty did not reach the cap on a 768-high output, so a picture
        // of thirty could not show whether the cap held.
        for number in 0..60_u64 {
            let letter = char::from(b'a' + u8::try_from(number % 26).unwrap_or(0));
            let named = format!("{letter}{number}");
            windows.opened(alo_dock::Window::of(
                alo_dock::WindowId::numbered(number),
                Some(alo_dock::AppId::named(&named).map_err(|error| format!("{error:?}"))?),
                &named,
                patch,
                alo_dock::HowItSits::OnTheCanvas,
            ));
        }
        // **Four pinned, so both of the design's groups are drawn.** With none
        // pinned the popover shows one heading and a picture of it could not
        // show that the groups are laid out at all.
        let mut holding = alo_dock::Holding::nothing();
        for named in ["Files", "Browser", "Apps", "Settings"] {
            holding.pin(alo_dock::AppId::named(named).map_err(|error| format!("{error:?}"))?);
        }
        holding.showing(&windows)
    };

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

    // **Said once, because the drawing path may not word a sentence.** The same
    // rule `direct_desktop` follows: `tests/desktop_source.rs` holds that no
    // `desktop_*` or `dock_*` file names `Word::` or `.say(`.

    let mut submitted = 0;
    let mut draw = |what: &str,
                    dock: &Dock,
                    on_the_dock: &[alo_dock::OnTheDock],
                    in_more_apps: &[alo_dock::OnTheDock],
                    the_dock_has_open: &alo_shell::WhatTheDockHasOpen,
                    put_aside: &alo_put_aside::Panel,
                    egress: &EgressStatus,
                    running: &RunningWindow,
                    filling: &FillingWindow,
                    nested: &mut Nested|
     -> Result<(), Box<dyn std::error::Error>> {
        // **Said inside the draw, because a window picker's words are about
        // this frame**: its heading carries the application's name and its rows
        // carry each window's state. The Dock's own labels do not change and
        // are said again here only because they travel in one bundle, which
        // cannot be half-filled.
        let the_docks_words =
            alo_shell::the_docks_words::TheDocksWords::said(&strings, the_dock_has_open);
        // A display nobody has divided, which is what this probe's is: the
        // Server holds no division yet and the session that will is a task of
        // its own.
        let division = alo_dividing::Division::of(alo_dividing::Area::of(
            alo_dividing::area::Point::at(0, 0),
            alo_dividing::area::Size::of(1920, 1080),
        )?);
        {
            // **The Dock is the caller's, not this closure's.** It built
            // `Dock::shipped()` here until 2026-10-10, which is the bottom edge
            // and nothing else — so the four-edge promise in `docs/features.md`
            // had no picture of three of its four answers, and the one thing a
            // fixture is for is eyes.
            for (scheme, now) in times {
                for (which_way, reading) in [
                    ("read left to right", Direction::LeftToRight),
                    ("read right to left", Direction::RightToLeft),
                ] {
                    // **One task window, so there is one to look at.** The
                    // whole of this feature was built, tested and never seen;
                    // this fixture's own header says why that matters —
                    // *it needs eyes, and eyes need pictures.*
                    //
                    // Example content, exactly as the design's component
                    // description says twice: nothing here is a simulated
                    // completed action, and the words are a caller's to say.
                    let files: Vec<String> = Vec::new();
                    let said = [
                        alo_shell::task_text::WhatWasSaid {
                            whose: alo_tasks::WhoseBlock::ThePersons,
                            words: "Tidy my invoices folder and tell me what you moved.",
                            marks: &[],
                            // A check of the drawing, with no change waiting in it.
                            answers: None,
                        },
                        alo_shell::task_text::WhatWasSaid {
                            whose: alo_tasks::WhoseBlock::Alos,
                            words: "I\u{2019}ll look through them and say what I changed.",
                            marks: &[],
                            // A check of the drawing, with no change waiting in it.
                            answers: None,
                        },
                        alo_shell::task_text::WhatWasSaid {
                            whose: alo_tasks::WhoseBlock::AnAccount,
                            words: "Reading your folder",
                            marks: &[],
                            // A check of the drawing, with no change waiting in it.
                            answers: None,
                        },
                    ];
                    let a_task_window = [alo_shell::task_raster::ATaskOnTheDisplay {
                        // **Cut to this display rather than the design's
                        // 1040 x 800.** This fixture draws on a 1366 x 768
                        // output and the design's size does not fit: the first
                        // picture had the window's bottom 92 pixels off the
                        // screen. In production `where_a_task_window_opens`
                        // does this cutting; here the figure is written down so
                        // the fixture is honest about drawing a window smaller
                        // than the design.
                        at: smithay::utils::Rectangle::new((60, 24).into(), (1040, 700).into()),
                        how_many_files: 0,
                        words: alo_shell::task_text::TheWordsInATaskWindow {
                            title: "Tidy my invoices folder",
                            kind: "alo task",
                            status: "alo is working",
                            model: "",
                            destination: "on this machine",
                            invitation: "Reply to alo or change the direction\u{2026}",
                            scope_note: "Permissions and model choices apply to this task.",
                            files: &files,
                            how_many_files: "",
                        },
                        said: said.to_vec(),
                        scroll: alo_tasks::TheScroll::at_the_newest(),
                        the_one_button: alo_tasks::TheOneButton::Stop,
                        place: 0,
                    }];
                    // **One market window, so there is one to look at.** The
                    // model was built with no caller, the raster was built with
                    // no caller, and 36 pixel tests passed over a window **no
                    // person and no fixture had ever seen**. This fixture's own
                    // header is the reason that matters: *it needs eyes, and
                    // eyes need pictures.*
                    //
                    // Open on the decided catalogue — the real eight entries
                    // from `catalogue.toml`, not invented rows — so what is on
                    // the screen is what a person would meet.
                    let mut the_market = alo_shell::MarketWindow::closed();
                    the_market.opened(alo_software::Catalogue::decided()?, Vec::new());
                    let the_markets_words = alo_shell::TheMarketsWords::said(&strings);
                    // **Placed by hand, and the figure is written down**, for
                    // the task window above's reason exactly:
                    // `where_the_market_opens` does this cutting in production
                    // and is not reachable from an example. The design's width
                    // is 632 and four rows of cards fit this output; drawing it
                    // at the bottom right leaves the Dock and the task window
                    // visible beside it, which is the point of one picture
                    // holding all three.
                    let the_market_window = alo_shell::market_raster::AMarketOnTheDisplay {
                        at: smithay::utils::Rectangle::new((700, 40).into(), (632, 688).into()),
                        showing: the_market.shows(),
                        words: &the_markets_words,
                    };
                    let until = Instant::now() + Duration::from_millis(60);
                    while Instant::now() < until {
                        nested.pump()?;
                        nested.submit_with_desktop(
                            &[],
                            &[],
                            &Cursor::Default,
                            &[],
                            &mut labels,
                            DesktopFrame {
                                what_each_application_draws: &alo_dock::artwork::NOTHING,
                                tasks: &a_task_window,
                                market: Some(the_market_window),
                                display_scale: 100,
                                // **The keyboard is not on the bar in this
                                // picture.** This check draws a desktop with
                                // windows on it; where the keyboard is, is a
                                // separate subject and `dock_raster`'s own
                                // tests hold it.
                                the_keyboard_is_on_the_bar: None,
                                the_docks_name_is_for: None,
                                the_bar: None,
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
                                on_the_dock,
                                // **The overview, which this fixture's windows
                                // make the same list.** Every one of them is on
                                // the canvas, so nothing is excluded from the
                                // Dock and the two lists coincide — which is
                                // why the caller passes one slice for both.
                                in_more_apps,
                                the_dock_has_open,
                                the_docks_words: &the_docks_words,
                                dock,
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
                                put_aside,
                                // A fixture that is not about revealing draws the panel, so what it lays out is
                                // the rail rather than an empty column.
                                panel_is_revealed: true,
                                filling_the_screen: false,
                            },
                            None,
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
    let shipped = Dock::shipped();

    // **The dock alone, on each of the four edges a person may choose.**
    //
    // `docs/features.md`'s *Where the Dock goes* promises bottom, left, right
    // and top, and until 2026-10-10 this fixture drew only the one the release
    // ships on. Three of the four placements had no picture — and
    // `dock_raster`'s own test for them calls `the_band_on` directly for the
    // reason it gives: a drawing test that goes through `Dock::shipped` can only
    // ever exercise one. So the edge is set here, which is the one place a
    // *picture* of the other three can come from.
    //
    // The edge is named in the picture's own name, because four files called
    // *the dock alone* differing only in where the bar sits is four files nobody
    // can tell apart.
    for edge in alo_dock::Edge::EVERY {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        draw(
            &format!("the dock alone on the {} edge", an_edges_name(edge)),
            &dock,
            &a_few_applications,
            &a_few_applications,
            &alo_shell::NOTHING,
            nothing_put_aside(),
            &quiet,
            &closed_running,
            &closed_filling,
            &mut nested,
        )?;
    }
    // **The overflow, on each of the four edges.** The owner's ruling of
    // 2026-10-10 asks for it to be verified on all four, and the edges really
    // differ: a 1366-wide bottom bar holds more than a 768-tall side one, so
    // the number of slots drawn before the control is not the same picture
    // four times.
    for edge in alo_dock::Edge::EVERY {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        draw(
            &format!("the dock overflowing on the {} edge", an_edges_name(edge)),
            &dock,
            &a_crowded_dock,
            &a_crowded_dock,
            &alo_shell::MORE_APPS,
            nothing_put_aside(),
            &quiet,
            &closed_running,
            &closed_filling,
            &mut nested,
        )?;
    }

    // **One application's windows**, which is the popover's other face.
    //
    // Four windows of one application, two of them put aside, so the picture
    // shows both of the states a row can be in — *Open* and *Minimised* — which
    // is the owner's *including its minimised windows with their state clearly
    // identified*. A standing with every window on the canvas would draw one
    // state four times and show nothing about the other.
    let a_browser_with_four_windows = {
        let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1)
            .map_err(|error| format!("{error:?}"))?;
        let mut windows = alo_dock::Windows::none();
        for (number, called, sits) in [
            (1_u64, "Launch website", alo_dock::HowItSits::OnTheCanvas),
            (2, "Travel plans", alo_dock::HowItSits::PutAside),
            (3, "Research", alo_dock::HowItSits::OnTheCanvas),
            (4, "Tomorrow's reading", alo_dock::HowItSits::PutAside),
        ] {
            windows.opened(alo_dock::Window::of(
                alo_dock::WindowId::numbered(number),
                Some(alo_dock::AppId::named("Browser").map_err(|error| format!("{error:?}"))?),
                called,
                patch,
                sits,
            ));
        }
        windows
    };
    let the_browsers_windows = alo_shell::WhatTheDockHasOpen::TheWindowsOf {
        picker: alo_dock::picking::ThePicker::of(
            alo_dock::previews::Previews::of(
                &a_browser_with_four_windows,
                &alo_dock::AppId::named("Browser").map_err(|error| format!("{error:?}"))?,
                alo_dock::previews::Opened::ByPressingTheIcon,
            ),
            alo_appearance::TextScale::default(),
            alo_dock::Room::pixels(1_000),
        ),
        // **The second row, so the picture shows the ring.** A check that only
        // ever drew the unfocused list would be a check that passes with no
        // focus treatment drawn at all — and the ring is the whole of what tells
        // a person where an arrow key just put them.
        focused: Some(1),
    };
    draw(
        "one application's windows",
        &shipped,
        &a_crowded_dock,
        &a_crowded_dock,
        &the_browsers_windows,
        nothing_put_aside(),
        &quiet,
        &closed_running,
        &closed_filling,
        &mut nested,
    )?;

    // **Both right-hand surfaces at once**, which is the only way to look at
    // the thing the owner's ruling of 2026-10-11 separates.
    //
    // The right-hand panel holds *windows a person minimised* — one item per
    // window, so two windows of one application are two items — and the Dock's
    // overflow holds *applications the bar had no room for*, as a popover
    // anchored to its control. They had never been drawn in one frame, and
    // *looking the same* was the fault: the popover hung off the bar's far end,
    // a bar wide enough to overflow has its far end near the display's, and the
    // panel is at the display's far end. Nothing was wrong with either
    // surface's contents.
    //
    // So this picture is the check. Two items in the panel from one
    // application, a crowded Dock with its list open, and a gap between the two
    // that a person can see.
    draw(
        "the overflow list beside two minimised windows of one application",
        &shipped,
        &a_crowded_dock,
        &a_crowded_dock,
        &alo_shell::MORE_APPS,
        two_windows_of_one_application(),
        &quiet,
        &closed_running,
        &closed_filling,
        &mut nested,
    )?;

    draw(
        "the dock with a question leaving",
        &shipped,
        &a_few_applications,
        &a_few_applications,
        &alo_shell::NOTHING,
        nothing_put_aside(),
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
        &shipped,
        &a_few_applications,
        &a_few_applications,
        &alo_shell::NOTHING,
        nothing_put_aside(),
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
        &shipped,
        &a_few_applications,
        &a_few_applications,
        &alo_shell::NOTHING,
        nothing_put_aside(),
        &lit,
        &running,
        &closed_filling,
        &mut nested,
    )?;

    let mut filling = FillingWindow::closed();
    filling.opened(&documents);
    draw(
        "what is filling a folder",
        &shipped,
        &a_few_applications,
        &a_few_applications,
        &alo_shell::NOTHING,
        nothing_put_aside(),
        &quiet,
        &closed_running,
        &filling,
        &mut nested,
    )?;
    filling.pressed(FillingKey::Next);
    filling.pressed(FillingKey::Open);
    draw(
        "a folder opened",
        &shipped,
        &a_few_applications,
        &a_few_applications,
        &alo_shell::NOTHING,
        nothing_put_aside(),
        &quiet,
        &closed_running,
        &filling,
        &mut nested,
    )?;
    draw(
        "both windows",
        &shipped,
        &a_few_applications,
        &a_few_applications,
        &alo_shell::NOTHING,
        nothing_put_aside(),
        &lit,
        &running,
        &filling,
        &mut nested,
    )?;

    std::fs::remove_dir_all(&documents)?;
    filling.pressed(FillingKey::CountAgain);
    draw(
        "a folder that is gone",
        &shipped,
        &a_few_applications,
        &a_few_applications,
        &alo_shell::NOTHING,
        nothing_put_aside(),
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
/// **Two windows of one application, put aside.**
///
/// The shape the owner's ruling of 2026-10-11 separates the two right-hand
/// surfaces on: *several minimised windows from one application remain separate
/// items* in the panel, while the Dock shows that application once. So this
/// panel draws two rows and the Dock draws one icon, from the same two windows
/// — which is what makes a picture of them side by side worth looking at.
///
/// Both windows name `Notes`, and they differ in title and id. A fixture with
/// two *different* applications would draw the same two rows and prove nothing.
fn two_windows_of_one_application() -> &'static alo_put_aside::Panel {
    static HOLDING: std::sync::OnceLock<alo_put_aside::Panel> = std::sync::OnceLock::new();
    HOLDING.get_or_init(|| {
        let mut panel = alo_put_aside::Panel::new();
        for (which, titled) in [(1_u64, "Monday"), (2, "Tuesday")] {
            let Ok(app) = alo_dock::AppId::named("Notes") else {
                return panel;
            };
            let Ok(patch) = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 800, 600) else {
                return panel;
            };
            let _ = panel.put_aside(
                &alo_dock::Window::of(
                    alo_dock::WindowId::numbered(which),
                    Some(app),
                    titled,
                    patch,
                    alo_dock::HowItSits::PutAside,
                ),
                alo_canvas::Zoom::LIFE_SIZE,
                alo_canvas::Place::FIRST,
                alo_put_aside::whether_it_is_private::Privacy::Ordinary,
            );
        }
        panel
    })
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
