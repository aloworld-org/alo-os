//! **The application door, against a real window a real program drew.**
//!
//! Everything else that tests this door uses a stand-in on one side or the
//! other. `alo-shell`'s own tests drive the door with a hand that records what
//! it was asked; `alo-agentd`'s drive a real socket with a stand-in compositor.
//! Both were honest and neither can answer the one question that matters:
//! **does a window move?**
//!
//! `ROADMAP.md` has said, in those words, that the three verbs *have never run
//! against a real application on a booted machine* — *every test of them uses a
//! stand-in on a real socket, because the compositor needs a Wayland parent,
//! and a stand-in is not a window.* This is the fixture that stops that being
//! true.
//!
//! # What it does
//!
//! 1. Starts alo OS's own compositor, nested under a Wayland parent.
//! 2. Starts a program nobody on this project wrote — `weston-terminal` by
//!    default — pointed at that compositor's socket.
//! 3. Waits for it to map a window, and reads **the identifier the program
//!    itself set**, never one this fixture chose.
//! 4. Serves the application door and connects to it the way `alo-agentd`
//!    does, from a thread, writing one line per act.
//! 5. Asks `open`, then `focus`, then `close` — and says whether the window
//!    went.
//!
//! # How to run it
//!
//! ```text
//! sudo apt-get install -y weston libgl1-mesa-dri libegl-mesa0
//! sudo mkdir -p /run/weston && sudo chmod 700 /run/weston
//! sudo env XDG_RUNTIME_DIR=/run/weston weston --backend=headless \
//!     --width=1366 --height=768 --socket=alo-parent --idle-time=0 &
//!
//! XDG_RUNTIME_DIR=/run/weston WAYLAND_DISPLAY=alo-parent \
//! LIBGL_ALWAYS_SOFTWARE=1 \
//!   cargo run -p alo-shell --example the_door_moves_a_real_window
//! ```
//!
//! `--run <program>` names a different client. Any Wayland client will do, and
//! one that refuses tells you something too.
//!
//! # What it is not
//!
//! **Not a session, and not a turn.** The door is served directly here and the
//! lines are written by this fixture, so there is no grant, no approval and no
//! record — `alo-agentd` is what puts a turn's authority in front of this, and
//! `alo-turn`'s own tests hold that. What this adds is the only thing those
//! cannot: a real client on the other end.
//!
//! **Nothing is drawn, and that is deliberate rather than a shortcut.** A
//! window moving is a fact about the compositor's own state — which surfaces
//! are mapped, which has the keyboard, which was asked to close — and none of
//! it needs a pixel. Drawing would add a dependency on the parent *presenting*
//! us, and under a headless parent with its own desktop shell it does not: the
//! first frame submits and the second blocks forever waiting on a callback that
//! never comes, which is what the first run of this fixture did. `nested_check`
//! and `a_real_application` are the fixtures that draw; this one is the fixture
//! that moves a window, and it needs the socket and the dispatch loop and
//! nothing else.
//!
//! **Not a certified machine.** A nested compositor under a headless parent,
//! software rendered. `CLAUDE.md` asks for hardware and this is not it — but it
//! is a real compositor, a real socket and a real window, which is three of the
//! four things that were stand-ins.
//!
//! **And `open` here can only bring something forward.** Starting an
//! application that is not running goes through the bus
//! (`alo_portals::activated`), and this fixture is not a session with a bus. So
//! `open` is asked of a program that is **already** running, which is the
//! bring-it-forward half of that verb and the half a compositor answers. The
//! starting half is the Dock's road and is proved where the Dock is proved.

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("the door moves a real window: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// The output this fixture draws into.
#[cfg(target_os = "linux")]
const THE_OUTPUT_IS: (u32, u32) = (1366, 768);

/// Start the compositor, start a program, and move its window through the door.
#[cfg(target_os = "linux")]
#[expect(
    clippy::too_many_lines,
    reason = "one fixture, one walk: splitting it would put the compositor's loop in one \
              function and the door's answers in another, and the whole point is that they are \
              the same walk"
)]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_acting::{Acting, Answer, Request};
    use alo_applications::Application;
    use alo_shell::the_application_door::TheApplicationDoor;
    use alo_shell::the_application_doors_place::ThePlace;
    use alo_shell::{Nested, Server};
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::os::unix::net::UnixStream;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    let program = what_was_asked();

    // The compositor, nested under whatever parent the environment names.
    let mut nested = Nested::new("alo OS, moving a real window", THE_OUTPUT_IS)?;
    nested.pump()?;
    let runtime = tempfile::tempdir()?;
    std::fs::set_permissions(
        runtime.path(),
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700),
    )?;
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "alo-door-moves-a-window",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..Default::default()
        },
    )?;
    server.enable_pointer()?;
    server.render(&mut nested, 0)?;
    let socket = server.socket_path().to_owned();
    println!("the compositor is listening on {}", socket.display());

    // The program, told where to connect and nothing else about us.
    let mut started = std::process::Command::new(&program)
        .env("WAYLAND_DISPLAY", &socket)
        .env_remove("DISPLAY")
        .spawn()
        .map_err(|why| format!("could not start {program}: {why}"))?;
    println!("started {program}");

    // Draw until it has put a window on the screen. A deadline rather than a
    // number of frames: how long a program takes to open its first window is
    // its business.
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut named = None;
    while Instant::now() < deadline && named.is_none() {
        nested.pump_seat(&mut server)?;
        server.dispatch()?;
        // **The identifier the program set, not one this fixture chose.** If
        // this fixture named the application, it would be proving that a line
        // it wrote reaches a window it labelled — which is what the stand-in
        // tests already prove. The whole point is that the name comes from the
        // other side.
        named = server
            .the_windows_the_dock_sees()
            .each()
            .find_map(|window| window.app().map(|app| app.name().to_owned()));
    }
    let Some(named) = named else {
        let _ = started.kill();
        return Err(format!(
            "{program} mapped no window with an application identifier within 20s, so there is \
             nothing for the door to be asked about"
        )
        .into());
    };
    println!("the program calls itself {named}");
    let which = Application::identified(&named)
        .map_err(|why| format!("{named} is not an identifier this machine accepts: {why:?}"))?;

    // The door, in a place of this fixture's own rather than `/run/alo/<uid>`:
    // that directory is the daemon's and no daemon is running here.
    let doors = tempfile::tempdir()?;
    let place = ThePlace::beneath(doors.path(), rustix::process::geteuid().as_raw());
    std::fs::create_dir_all(place.directory())?;
    let mut door = TheApplicationDoor::served(place.clone(), rustix::process::geteuid().as_raw())
        .map_err(|why| format!("the door would not be served: {why}"))?;
    println!("the door is at {}", place.socket().display());

    // A caller on the other side, the way `alo-agentd` is one: connect, write
    // one line, read one word, drop it. From a thread, because the compositor's
    // loop has to keep running for anything to be answered — which is the
    // whole reason the door never blocks.
    let (asking, asked) = mpsc::channel::<(String, String)>();
    let at = place.socket().to_owned();
    let acts = [
        Acting::Open(which.clone()),
        Acting::Focus(which.clone()),
        Acting::Close(which.clone()),
    ];
    std::thread::spawn(move || {
        for acting in acts {
            let line = Request::to(acting).written();
            // One connection per act: ADR 0108's rule, and what `alo-agentd`
            // really does.
            let answer = (|| -> Result<String, String> {
                let mut talking = UnixStream::connect(&at).map_err(|why| why.to_string())?;
                talking
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .map_err(|why| why.to_string())?;
                talking
                    .write_all(format!("{line}\n").as_bytes())
                    .map_err(|why| why.to_string())?;
                talking.flush().map_err(|why| why.to_string())?;
                let mut heard = String::new();
                BufReader::new(&talking)
                    .read_line(&mut heard)
                    .map_err(|why| why.to_string())?;
                Ok(heard.trim().to_owned())
            })()
            .unwrap_or_else(|why| format!("<not answered: {why}>"));
            if asking.send((line, answer)).is_err() {
                return;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    });

    // The compositor's loop, answering the door as it draws — which is where
    // the door is read from on a real machine.
    let mut answers = Vec::new();
    let patience = Instant::now() + Duration::from_secs(30);
    while Instant::now() < patience && answers.len() < 3 {
        nested.pump_seat(&mut server)?;
        server.dispatch()?;
        door.answered(&mut server);
        while let Ok((line, answer)) = asked.try_recv() {
            println!("  asked {line:?} → answered {answer:?}");
            answers.push((line, answer));
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    // Did the window really go? Pump a little longer: `close` **asks**, and the
    // program decides.
    let gone_by = Instant::now() + Duration::from_secs(10);
    let mut still_there = true;
    while Instant::now() < gone_by && still_there {
        nested.pump_seat(&mut server)?;
        server.dispatch()?;
        still_there = server
            .the_windows_the_dock_sees()
            .each()
            .any(|window| window.app().is_some_and(|app| app.name() == named));
        std::thread::sleep(Duration::from_millis(50));
    }

    println!("\n--- what the door did to a real window ---");
    for (line, answer) in &answers {
        println!("  {line:<40} {answer}");
    }
    let acted = answers
        .iter()
        .filter(|(_, answer)| Answer::read(answer) == Some(Answer::Acted))
        .count();
    println!("  acted on {acted} of {} acts", answers.len());
    println!(
        "  the window is {}",
        if still_there {
            "still open — the program declined to close, which is its right"
        } else {
            "gone"
        }
    );
    println!("--- (one run, under a nested parent; not a certified machine) ---");

    let _ = started.kill();
    let _ = started.wait();
    door.taken_away();

    if answers.len() < 3 {
        return Err(format!("only {} of 3 acts were answered within 30s", answers.len()).into());
    }
    if acted < 3 {
        return Err(format!(
            "{acted} of 3 acts answered `acted`; the lines above say which did not"
        )
        .into());
    }
    if still_there {
        return Err("every act was carried out and the window is still open".into());
    }
    Ok(())
}

/// Which program to start, from `--run`.
#[cfg(target_os = "linux")]
fn what_was_asked() -> String {
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--run"
            && let Some(program) = arguments.next()
        {
            return program;
        }
    }
    "weston-terminal".to_owned()
}
