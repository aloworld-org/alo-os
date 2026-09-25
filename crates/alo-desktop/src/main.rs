//! The desktop a person arrives at after they sign in.
//!
//! `alo-compositor` is the other half of a machine's morning: the sign-in
//! screen, before anybody is there, ending when a session opens. This runs
//! inside that session and draws the person's own desktop — the dock, the
//! status area, and what is leaving this machine.
//!
//! They are two programs because they belong to two different people. One is
//! the machine's, standing in front of somebody who has not signed in; the
//! other is the person's, and starting it is their session's job.
//!
//! # What it is told
//!
//! The same three facts a machine differs by, in its unit file, and the process
//! refuses to start without them: which card, which keyboard, and the runtime
//! directory systemd made. Not the uid — whose session this is, is *whose
//! session this is*, and a desktop that took a person's number from a variable
//! could be started for the wrong one.
//!
//! # What it draws
//!
//! The dock, the status area and the egress indicator, drawn on the processor
//! (`alo_shell`'s software painter). **The four readings are this machine's**,
//! taken by `crate::readings` from the entry point each owning crate already
//! has: the clock, the battery, how far the machine reaches and how loud it is.
//!
//! A reading that could not be taken is an **absence**, not a zero: the status
//! area leaves it out, and the reason goes to the service log. A battery at
//! nought per cent and a machine with no battery are different things, and a
//! person acts on them differently.
//!
//! # Why the reading lives here and not in the shell
//!
//! The shell shows and does not measure — a compositor that opened `/sys` would
//! be a compositor measuring. That is shell task 15's constraint, and it is why
//! this process is a package of its own rather than another binary inside
//! `alo-shell`: a binary in that directory reading a battery would be that call
//! whatever the module boundary said.
//!
//! **A client's window cannot be drawn at all yet.** The software painter
//! imports nothing, and every frame carrying a mapped window is refused by
//! name. The desktop itself needs no importing, which is why it can stand up
//! before that is answered.

#[cfg(target_os = "linux")]
mod readings;

/// What this process does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod running {
    use std::path::PathBuf;
    use std::process::ExitCode;

    use alo_shell::{
        ADisplayToStandOn, DesktopFrame, DesktopLook, DirectFrame, EgressStatus, FillingWindow,
        RunningWindow, TheDesktop,
    };

    /// The name of this desktop's socket inside its runtime directory.
    ///
    /// What a client of this person's session connects to.
    const THE_SOCKET: &str = "wayland-alo";

    /// How often a frame is drawn.
    ///
    /// Sixty a second, whether or not anything changed. A desktop that redrew
    /// only on change is the right answer and needs damage tracking this
    /// compositor does not have yet.
    const A_FRAME: std::time::Duration = std::time::Duration::from_micros(16_667);

    /// Stand this person's desktop up, and say what became of it.
    ///
    /// `FAILURE` is a session with no desktop in it: a unit that did not say
    /// what this machine is, a card the seat would not lend, a socket that
    /// would not bind.
    pub fn main() -> ExitCode {
        let said = match what_this_machine_is() {
            Ok(said) => said,
            Err(why) => {
                eprintln!("alo-desktop: this session cannot have a desktop: {why}");
                return ExitCode::FAILURE;
            }
        };
        let mut desktop = match ThisPersonsDesktop::made() {
            Ok(desktop) => desktop,
            Err(why) => {
                eprintln!("alo-desktop: this session cannot have a desktop: {why}");
                return ExitCode::FAILURE;
            }
        };

        eprintln!(
            "alo-desktop: the desktop is going up on {}, on a {} keyboard; the status area shows \
             this machine's own four readings, and leaves out any it could not take — the line \
             above says which those were and why",
            said.display.display(),
            said.layout
        );

        let started = std::time::Instant::now();
        let mut last = None;
        match alo_shell::stand_the_desktop_up(
            ADisplayToStandOn {
                display: &said.display,
                runtime: &said.runtime,
                socket: THE_SOCKET,
                layout: &said.layout,
            },
            &mut desktop,
            || {
                let now = started.elapsed();
                match last {
                    Some(then) if now < then + A_FRAME => DirectFrame::Idle,
                    _ => {
                        last = Some(now);
                        #[expect(
                            clippy::cast_possible_truncation,
                            reason = "the wrap is the protocol's, not an accident of arithmetic"
                        )]
                        DirectFrame::Render(now.as_millis() as u32)
                    }
                }
            },
        ) {
            Ok(stood) => {
                eprintln!("alo-desktop: the display has ended: {:?}", stood.display);
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("alo-desktop: this session cannot have a desktop: {why}");
                ExitCode::FAILURE
            }
        }
    }

    /// Everything on this person's desktop, as the crates that own each said it.
    struct ThisPersonsDesktop {
        /// Where the dock is, and so where the status area is.
        dock: alo_dock::Dock,
        /// The colours and the way this person reads.
        look: DesktopLook,
        /// Every word on it.
        strings: alo_strings::Strings,
        /// What the machine's indicator has said about what is leaving.
        egress: EgressStatus,
        /// Shut, because nothing on a machine opens it yet.
        running: RunningWindow,
        /// The same.
        filling: FillingWindow,
        /// The four readings, as this machine gave them.
        status: alo_shell::StatusItems,
        /// A display nobody has divided, which is what this one is: the
        /// `Server` holds no division and task 16 is the one that gives it one.
        division: alo_dividing::Division,
        /// The way this person writes a time, kept so every refresh uses it.
        region: alo_formats::Regionally,
        /// When the readings were last taken.
        read_at: std::time::Instant,
    }

    impl ThisPersonsDesktop {
        /// Everything a desktop is, read once at the start.
        fn made() -> Result<Self, String> {
            // **Read once here and again every frame** — see `read_again`.
            let at = crate::readings::At::now()?;
            let (readings, missing) = crate::readings::taken(&at, &the_region()?)?;
            eprintln!(
                "alo-desktop: the status area's readings — {}",
                missing.said()
            );
            let strings = alo_strings::Strings::of(
                alo_saying::everything_this_machine_can_say().map_err(|why| why.to_string())?,
            );
            // **The indicator is told before the first frame, including that
            // nothing is leaving.** A desktop whose indicator has never been
            // told anything is refused before any of it is drawn, because
            // *nothing yet* is not the same answer as *nothing* — and a status
            // area that could not say what is leaving is the one thing this
            // product may not put on a screen.
            let mut egress = EgressStatus::on_an_output();
            alo_indicator::Indicating::nowhere()
                .show(Some(&mut egress), &alo_egress::Indicator::default());
            Ok(Self {
                dock: alo_dock::Dock::shipped(),
                look: DesktopLook::of(
                    &alo_appearance::Appearance::shipped(),
                    &alo_access::TurnedOn::nothing(),
                    the_hour(&at)?,
                    alo_strings::Direction::LeftToRight,
                ),
                strings,
                egress,
                running: RunningWindow::closed(),
                filling: FillingWindow::closed(),
                status: readings,
                division: a_display_nobody_has_divided()?,
                region: the_region()?,
                read_at: std::time::Instant::now(),
            })
        }
    }

    /// How often the four readings are taken again.
    ///
    /// **Once a second, not once a frame.** A frame is drawn sixty times a
    /// second and a battery does not move sixty times a second; asking the
    /// media server and the network manager that often would be this process
    /// spending a person's machine on a number that did not change. A second is
    /// short enough that a clock showing minutes is never wrong.
    const HOW_OFTEN: std::time::Duration = std::time::Duration::from_secs(1);

    impl TheDesktop for ThisPersonsDesktop {
        /// Take the four again, if it is time to.
        ///
        /// A refusal does not clear what was there: a media server that did not
        /// answer this second is not a reason to take the volume off a person's
        /// screen, and the reading it replaces is a second old. What a failure
        /// does change is the next successful reading, which is the truth
        /// arriving late rather than an absence arriving early.
        fn refreshed(&mut self) {
            if self.read_at.elapsed() < HOW_OFTEN {
                return;
            }
            self.read_at = std::time::Instant::now();
            let Ok(at) = crate::readings::At::now() else {
                return;
            };
            if let Ok((readings, _)) = crate::readings::taken(&at, &self.region) {
                self.status = readings;
            }
        }

        fn now(&self) -> DesktopFrame<'_> {
            DesktopFrame {
                dock: &self.dock,
                look: self.look,
                strings: &self.strings,
                egress: &self.egress,
                running: &self.running,
                filling: &self.filling,
                status: &self.status,
                // **Nothing is watching or listening, and that is read rather
                // than assumed** — `alo_in_use::InUse::read_from` asks the
                // machine's media server, and nothing on a machine with no
                // applications on it yet has a camera or a microphone open.
                // Asking the server for real belongs with the other readings
                // this binary hands over, task 15 of the shell plan.
                in_use: &[],
                // **Nothing has sent one**, which is different from holding
                // them: `alo_notifying::arrives` is where a notification
                // becomes one to show, and nothing on this machine calls it
                // yet. A portal that lets an application send one is the
                // applications plan's, not this binary's.
                notifications: &[],
                capturing: None,
                division: &self.division,
                offer: &alo_dividing::Offer::Nothing,
            }
        }
    }

    /// The clock this machine really keeps, and the honest absence of the rest.
    ///
    /// The clock is here and the other three are not because of what each looks
    /// like when it is wrong. A clock that never moves is a machine a person can
    /// see is dead; a battery reading that never moves is a machine lying about
    /// how much time they have left. Every one of the four is this machine's,
    /// and the ones it could not read are absent rather than invented.
    ///
    /// The way a person writes a time. English in Great Britain until a person
    /// has somewhere to say otherwise — where that is kept is a settings
    /// question and not this process's to answer.
    fn the_region() -> Result<alo_formats::Regionally, String> {
        alo_formats::Regionally::reading("en")
            .and_then(|reading| reading.in_region("GB"))
            .map_err(|why| format!("a region nobody writes in: {why:?}"))
    }

    /// The time of day the appearance is asked about, from the same clock.
    ///
    /// The same moment the readings were taken at, so night light and the
    /// status area's clock cannot disagree about what time it is.
    fn the_hour(at: &crate::readings::At) -> Result<alo_appearance::TimeOfDay, String> {
        alo_appearance::TimeOfDay::checked(at.hour, at.minute)
            .map_err(|why| format!("a time of day this machine does not have: {why:?}"))
    }

    /// The whole display as one share, since nothing has divided it.
    fn a_display_nobody_has_divided() -> Result<alo_dividing::Division, String> {
        let area = alo_dividing::Area::of(
            alo_dividing::area::Point::at(0, 0),
            alo_dividing::area::Size::of(1920, 1080),
        )
        .map_err(|why| format!("a display of no size: {why:?}"))?;
        Ok(alo_dividing::Division::of(area))
    }

    /// What this machine is, as its unit file says it.
    struct Said {
        /// The graphics card.
        display: PathBuf,
        /// The runtime directory systemd made.
        runtime: PathBuf,
        /// Which keyboard is in front of it.
        layout: String,
    }

    /// Read the two things a machine differs by, and systemd's own directory.
    fn what_this_machine_is() -> Result<Said, String> {
        Ok(Said {
            display: PathBuf::from(named("ALO_DISPLAY")?),
            runtime: PathBuf::from(named("RUNTIME_DIRECTORY")?),
            layout: named("ALO_KEYBOARD")?,
        })
    }

    /// One variable the unit file has to set, or the sentence saying it did not.
    fn named(variable: &str) -> Result<String, String> {
        std::env::var(variable).map_err(|_| {
            format!(
                "{variable} is not set, and this desktop's unit file is where a machine says what \
                 it is"
            )
        })
    }
}

/// The desktop, on the machine it is for.
#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    running::main()
}

/// Anywhere else, and it says so rather than pretending.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "alo-desktop takes a graphics card from a login seat and there is none on this host: alo \
         OS is Linux (ADR 0011), and a process that exited cleanly here would be telling a \
         supervisor that somebody has a desktop on this machine"
    );
    std::process::ExitCode::FAILURE
}
