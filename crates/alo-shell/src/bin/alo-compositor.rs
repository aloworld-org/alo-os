//! The compositor a machine boots to: the display, the sign-in screen, and the
//! session that opens behind it.
//!
//! `alo_shell` is every decision the screen is made of; this is the process
//! that makes them in order on a real machine. It is deliberately thin, because
//! what a process is — where its configuration comes from, and what an exit
//! code means — is the only part of this a test cannot reach: it takes a
//! graphics card from the login seat, binds a Wayland socket in the runtime
//! directory systemd made for it, and knocks on a door in `/run`.
//!
//! # Until this existed, an installed machine booted to a console
//!
//! `crates/alo-shell` had no binary. The sign-in screen, its keys, its drawing
//! and the session it hands over to were all built and all tested, and nothing
//! a machine could start put them in order. This is that order.
//!
//! # Where a machine says what it is
//!
//! In its unit file, and this process refuses to start without it. Three things
//! differ from machine to machine — which graphics card, whose machine it is,
//! and which keyboard is in front of it — and a compositor that guessed any of
//! them would be guessing about the display a person looks at, the account they
//! sign in to, or the letters their password is made of. `RUNTIME_DIRECTORY` is
//! systemd's own variable and not one of ours.
//!
//! # It says one kind of thing, and it says it to a service log
//!
//! Nothing this file writes is read by the person using the machine. What they
//! read is on the screen, in their own language, drawn by `alo-shell`. What is
//! here is for whoever is standing the machine up, in English: which card was
//! opened, which uid a session was asked for, and why a screen did not appear.
//!
//! # And it runs on Linux
//!
//! A graphics card through libseat is not a thing another host has.

/// What this process does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod running {
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;

    use alo_greeting::TheOpenersDoor;
    use alo_shell::{AMachineToStandOn, DirectFrame, Stood, what_this_machine_can_say};

    /// The name of this compositor's socket inside its runtime directory.
    ///
    /// Not configurable: it is this process's own name for its own socket, and
    /// a machine with two of them would have two compositors.
    const THE_SOCKET: &str = "sign-in";

    /// How often a frame is drawn while somebody is typing at the screen.
    ///
    /// Sixty a second, which is the rate the screen is redrawn at whether or
    /// not anything changed. It is more than a mostly still picture needs; what
    /// it buys is that a letter appears as it is typed rather than when
    /// something else happens to wake the loop. A screen that redrew only on
    /// change is the right answer and it needs damage tracking this compositor
    /// does not have yet.
    const A_FRAME: std::time::Duration = std::time::Duration::from_micros(16_667);

    /// Stand the sign-in screen up, and say what became of it.
    ///
    /// `FAILURE` is a machine nobody can sign in to: a unit that did not say
    /// what this machine is, a card the seat would not lend, a socket that
    /// would not bind. A person who typed a wrong password is **not** a failure
    /// of this process — it is an answer, and it is on the screen in front of
    /// them.
    pub fn main() -> ExitCode {
        let said = match what_this_machine_is() {
            Ok(said) => said,
            Err(why) => {
                eprintln!("alo-compositor: this machine cannot be signed in to: {why}");
                return ExitCode::FAILURE;
            }
        };
        let machine = AMachineToStandOn {
            display: &said.display,
            accounts: Path::new(alo_accounts::THE_ACCOUNTS),
            person: said.person,
            door: TheOpenersDoor::on_this_machine(),
            runtime: &said.runtime,
            socket: THE_SOCKET,
            layout: &said.layout,
        };
        let words = match what_this_machine_can_say() {
            Ok(words) => words,
            Err(why) => {
                eprintln!("alo-compositor: this machine cannot be signed in to: {why}");
                return ExitCode::FAILURE;
            }
        };

        eprintln!(
            "alo-compositor: the sign-in screen is going up on {}, for uid {}, on a {} keyboard",
            machine.display.display(),
            machine.person,
            machine.layout
        );

        let started = std::time::Instant::now();
        let mut last = None;
        match alo_shell::stand_the_sign_in_screen_up(machine, words, || {
            let now = started.elapsed();
            match last {
                Some(then) if now < then + A_FRAME => DirectFrame::Idle,
                _ => {
                    last = Some(now);
                    // Milliseconds on this machine's own monotonic clock,
                    // wrapping at 32 bits as Wayland requires.
                    #[expect(
                        clippy::cast_possible_truncation,
                        reason = "the wrap is the protocol's, not an accident of arithmetic"
                    )]
                    DirectFrame::Render(now.as_millis() as u32)
                }
            }
        }) {
            Ok(Stood::SomebodySignedIn { person }) => {
                // The session is open and it is `alo-sessiond` that holds it,
                // not this process. What goes on that person's display is the
                // next thing to be built; until it is, this says plainly that
                // it signed somebody in and stopped, rather than leaving a
                // black screen to be read as a crash.
                eprintln!(
                    "alo-compositor: a session is open for uid {person}, and this process has \
                     nothing yet to draw in it"
                );
                ExitCode::SUCCESS
            }
            Ok(Stood::NobodyDid) => {
                eprintln!("alo-compositor: the sign-in screen ended with nobody signed in");
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("alo-compositor: this machine cannot be signed in to: {why}");
                ExitCode::FAILURE
            }
        }
    }

    /// What this machine is, as its unit file says it.
    ///
    /// Owned rather than borrowed because these outlive the reading, and each
    /// one is refused by name: a unit missing `ALO_DISPLAY` and a unit whose
    /// `ALO_PERSON` is not a number send whoever is reading the log to two
    /// different lines of the same file.
    struct Said {
        /// The graphics card.
        display: PathBuf,
        /// The runtime directory systemd made.
        runtime: PathBuf,
        /// Whose machine it is.
        person: u32,
        /// Which keyboard is in front of it.
        layout: String,
    }

    /// Read the three things a machine differs by, and systemd's own directory.
    fn what_this_machine_is() -> Result<Said, String> {
        Ok(Said {
            display: PathBuf::from(named("ALO_DISPLAY")?),
            runtime: PathBuf::from(named("RUNTIME_DIRECTORY")?),
            person: named("ALO_PERSON")?
                .parse()
                .map_err(|_| "ALO_PERSON is not a uid".to_owned())?,
            layout: named("ALO_KEYBOARD")?,
        })
    }

    /// One variable the unit file has to set, or the sentence saying it did not.
    fn named(variable: &str) -> Result<String, String> {
        std::env::var(variable).map_err(|_| {
            format!(
                "{variable} is not set, and this compositor's unit file is where a machine says \
                 what it is"
            )
        })
    }
}

/// The compositor, on the machine it is for.
#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    running::main()
}

/// Anywhere else, and it says so rather than pretending.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "alo-compositor takes a graphics card from a login seat and there is none on this host: \
         alo OS is Linux (ADR 0011), and a process that exited cleanly here would be telling a \
         supervisor that this machine has a sign-in screen on it"
    );
    std::process::ExitCode::FAILURE
}
