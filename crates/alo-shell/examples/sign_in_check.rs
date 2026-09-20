//! The sign-in screen on a real nested compositor under a Wayland parent.
//!
//! Explicit developer fixture, never installed as anybody's session. It draws
//! every screen the greeter can stand at through the parent's EGL, in both
//! schemes, pumps the parent's real keyboard through the seat while nobody
//! types — and requires that nothing proceeds on silence — and, with
//! `--interactive SECONDS`, lets a person at the parent window type a name and
//! a password until a session is handed over or the time runs out.
//!
//! The account it makes is `ada` with the password `nested-check`, on a store
//! that lives only in this process. The door is a socket in a private
//! directory answered with `alo-sessiond`'s own words; no session is opened on
//! the machine running the check. A virtual output proves the drawing path, not
//! a panel: a certified machine has not seen this screen.

/// Main-thread graphics initialisation is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sign-in screen check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Draw each standing, pump in silence, and optionally take a person's keys.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_accounts::Accounts;
    use alo_appearance::{Scheme, TextScale};
    use alo_greeting::{Greeting, NotReadable, TheOpenersDoor};
    use alo_shell::{
        Contrast, Nested, Server, SignInLook, SignInScreen, Signing, WindowControlLabels,
    };
    use alo_strings::Strings;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};

    let interactive = std::env::args()
        .skip_while(|arg| arg != "--interactive")
        .nth(1)
        .map(|seconds| seconds.parse::<u64>())
        .transpose()?;

    let mut nested = Nested::new("alo sign-in screen check", (800, 600))?;
    nested.pump()?;
    let runtime = tempfile::tempdir()?;
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "sign-in-check",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..Default::default()
        },
    )?;

    // A door that opens whatever it is asked for, in the opener's own words.
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

    let words = || -> Result<Strings, Box<dyn std::error::Error>> {
        Ok(Strings::of(alo_saying::everything_this_machine_can_say()?))
    };
    let mut store = Accounts::none()?;
    store.created("ada", 1000, "nested-check")?;
    let mut labels = WindowControlLabels::new()?;

    let standings = [
        (
            "a name and a password",
            SignInScreen::of(
                Ok(Greeting::of(store, 1000, TheOpenersDoor::at(&door_at))),
                words()?,
            ),
        ),
        (
            "make an account",
            SignInScreen::of(
                Ok(Greeting::of(
                    Accounts::none()?,
                    1000,
                    TheOpenersDoor::at(&door_at),
                )),
                words()?,
            ),
        ),
        (
            "accounts that would not be read",
            SignInScreen::of(
                Err(NotReadable {
                    at: "/etc/alo/accounts.toml".to_owned(),
                    why: "a check that refuses on purpose".to_owned(),
                }),
                words()?,
            ),
        ),
    ];

    let mut kept = None;
    let mut submitted = 0;
    for (what, mut screen) in standings {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let look = SignInLook {
                contrast: Contrast::AsDesigned,
                scheme,
                scale: TextScale::ordinary(),
            };
            let until = Instant::now() + Duration::from_millis(300);
            while Instant::now() < until {
                screen = match nested.pump_sign_in(&mut server, screen)? {
                    Signing::Still(screen) => *screen,
                    Signing::HandedOver(_) => {
                        return Err(format!("{what}: a session opened with nobody typing").into());
                    }
                };
                nested.submit_sign_in(&screen, &mut labels, look)?;
                submitted += 1;
                std::thread::sleep(Duration::from_millis(16));
            }
        }
        println!(
            "Drew the sign-in screen standing at {what}, light and dark; nothing proceeded on silence"
        );
        if kept.is_none() {
            kept = Some(screen);
        }
    }

    if let (Some(seconds), Some(mut screen)) = (interactive, kept) {
        println!("Type `ada`, Enter, `nested-check`, Enter at the parent window within {seconds}s");
        let look = SignInLook {
            contrast: Contrast::AsDesigned,
            scheme: Scheme::Light,
            scale: TextScale::ordinary(),
        };
        let until = Instant::now() + Duration::from_secs(seconds);
        loop {
            if Instant::now() > until {
                return Err("nobody signed in before the time ran out".into());
            }
            screen = match nested.pump_sign_in(&mut server, screen)? {
                Signing::Still(screen) => *screen,
                Signing::HandedOver(session) => {
                    println!(
                        "Handed over to a session for uid {}; the screen stopped drawing",
                        session.uid()
                    );
                    break;
                }
            };
            if let Some(trouble) = screen.for_the_maintainer() {
                eprintln!("{trouble}");
            }
            nested.submit_sign_in(&screen, &mut labels, look)?;
            submitted += 1;
            std::thread::sleep(Duration::from_millis(16));
        }
    }

    println!(
        "{submitted} sign-in frames submitted through the parent's EGL; physical display unverified"
    );
    Ok(())
}
