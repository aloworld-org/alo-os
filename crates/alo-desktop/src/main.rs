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
        /// Where this person's canvas layout is kept, if they have a folder.
        ///
        /// **[`None`] is a real session and not a fault.** A login with no home
        /// directory, or one whose `XDG_CONFIG_HOME` is relative with no
        /// `HOME`, has nowhere to keep anything — `alo_choosing` answers that
        /// by name. Such a session gets a canvas where the applications put
        /// their windows, which is what happens today for everybody, and
        /// nothing is written anywhere.
        ///
        /// Worked out once, because where a person's folder is does not change
        /// under a running session.
        layout_at: Option<std::path::PathBuf>,
        /// Where this person's changed shortcuts live, if they have a folder.
        ///
        /// Beside the layout and read the same way: one folder, one question
        /// per file, and `None` on a session with nowhere to keep anything.
        shortcuts_at: Option<std::path::PathBuf>,
        /// Where the dock is, and so where the status area is.
        dock: alo_dock::Dock,
        /// The windows this person has put aside, which the panel at the edge
        /// shows.
        ///
        /// **Still empty, and still measured rather than assumed** — but no longer because
        /// this is only the drawing half. That clause was true until
        /// [`TheDesktop::the_pointer_is_now`] arrived below: this binary now acts on a
        /// person's pointer, so it is not merely drawing.
        ///
        /// What is missing is narrower and worth naming exactly: **nothing yet *puts* a
        /// window aside.** `alo_shell::Server::put_this_window_aside` is written and has only
        /// its integration tests, because the gesture that would call it does not exist. So a
        /// person on this machine has a panel they can point at and nothing to point at in
        /// it.
        ///
        /// A person who has put nothing aside has an empty panel and no rail is drawn for
        /// one, so what is on the screen is true rather than a placeholder — and the panel's
        /// reserved column exists either way, because the panel owns its edge whether or not
        /// anything is in it.
        put_aside: alo_put_aside::Panel,
        /// Which put-aside window the person is looking at, if any.
        ///
        /// **Held beside the `Panel` rather than inside it**, because a peek is not a
        /// property of the panel's contents — `alo-put-aside` keeps `peek_at` free of
        /// `&mut Panel` on purpose, so that looking at a window cannot alter what is put
        /// aside. One peek for the session: at most one window is being looked at at a
        /// time, and a second would be two answers to one person's pointer.
        peeking: alo_dock::Peeking,
        /// Whether the panel is on the screen, and why.
        ///
        /// **Starts covered**, which is the honest state at sign-in: nothing has been put
        /// aside and nobody has reached for the edge. `alo_dock::Revealing::covered` is the
        /// constructor for *a surface at rest under a full-screen window*, and a fresh session
        /// is the same shape — the panel owns its edge and is not showing.
        ///
        /// Held beside the panel rather than inside it, for the reason the peek is: whether a
        /// surface is revealed is not a property of what is put aside. A person with three
        /// windows away and a concealed panel is an ordinary state.
        revealing: alo_dock::revealing::Revealing,
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
            // Where the canvas layout lives, if this person has a folder to
            // keep one in. Read from the environment rather than assumed: the
            // same two variables `alo-choosing` is given everywhere else.
            let layout_at = alo_choosing::where_the_folder_is(
                std::env::var_os("XDG_CONFIG_HOME").as_deref(),
                std::env::var_os("HOME").as_deref(),
            )
            .map(|folder| folder.join(alo_arranging::keeping::THE_FILE));
            if layout_at.is_none() {
                eprintln!(
                    "alo-desktop: this session has no folder, so the canvas layout is not kept"
                );
            }
            // And the person's own chords, from the same folder. A session
            // without one still has every shipped shortcut — see
            // `the_shortcuts` below.
            let shortcuts_at = alo_choosing::where_the_folder_is(
                std::env::var_os("XDG_CONFIG_HOME").as_deref(),
                std::env::var_os("HOME").as_deref(),
            )
            .map(|folder| folder.join(alo_shortcuts::keeping::THE_FILE));
            Ok(Self {
                layout_at,
                shortcuts_at,
                dock: alo_dock::Dock::shipped(),
                put_aside: alo_put_aside::Panel::new(),
                // Nobody is looking at anything yet, which is what a session starts as.
                peeking: alo_dock::Peeking::at_nothing(),
                // Nothing put aside and nobody at the edge, so the panel is not showing.
                revealing: alo_dock::revealing::Revealing::covered(),
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
        /// What this person's chords mean, read from their own folder.
        ///
        /// **What this release ships, with the person's changes over it.** A
        /// session with no folder, or whose shortcuts file does not read, gets
        /// the shipped chords rather than none: a machine where `⊞`+0 stopped
        /// working because a file was hand-edited wrong would have taken the
        /// keyboard away over a typo.
        ///
        /// **A damaged file is said out loud and does not stop the session**,
        /// which is the same choice `the_layout_they_left` makes below and for
        /// the same reason.
        fn the_shortcuts(&mut self) -> alo_shortcuts::Shortcuts {
            let shipped = alo_shortcuts::Shortcuts::shipped();
            let Some(at) = self.shortcuts_at.as_deref() else {
                return shipped;
            };
            match alo_shortcuts::keeping::read(at) {
                Ok(changes) => shipped.with(changes),
                Err(why) => {
                    eprintln!("alo-desktop: this person's shortcuts did not read — {why}");
                    shipped
                }
            }
        }

        /// The canvas layout this person left, read from their own folder.
        ///
        /// **A damaged file is said out loud and does not stop the session.**
        /// Somebody whose layout file was hand-edited wrong gets their windows
        /// where the applications put them and a line saying why, rather than a
        /// desktop that will not start — and the next layout change replaces
        /// the damaged file with a good one, so the damage costs them a
        /// remembered arrangement and nothing else.
        fn the_layout_they_left(&mut self) -> alo_arranging::Arrangement {
            let Some(at) = self.layout_at.as_deref() else {
                return alo_arranging::Arrangement::fresh();
            };
            let (arrangement, why) = alo_arranging::keeping::at_sign_in(at);
            if let Some(why) = why {
                eprintln!("alo-desktop: the canvas layout did not read — {why}");
            }
            arrangement
        }

        /// The canvas layout moved, so it is kept.
        ///
        /// **Called when it changed and not every frame** — the shell compares
        /// what it last said with what is true now, the same way it does for
        /// the accessibility tree. So this is a write per actual rearrangement
        /// rather than per frame, and the owner's ruling of 2026-10-03 asks for
        /// exactly that: a save after meaningful layout changes rather than on
        /// a clean shutdown alone, because the session a person loses is the one
        /// that did not end cleanly.
        ///
        /// **A refusal is said and does not stop the desktop.** A full disk
        /// costs a person their remembered layout; a compositor that stopped
        /// compositing over it would cost them the machine.
        fn the_layout_is_now(&mut self, arrangement: alo_arranging::Arrangement) {
            let Some(at) = self.layout_at.as_deref() else {
                return;
            };
            // **The moment the layout moved**, which is what a Place's
            // ribbon is indexed by. Read here rather than inside `keep`, so
            // that what a test holds is a moment it chose and not a clock
            // (`alo_arranging::Arrangement::following`).
            let when = std::time::SystemTime::now();
            if let Err(why) = alo_arranging::keeping::keep(at, &arrangement, when) {
                eprintln!("alo-desktop: the canvas layout was not kept — {why}");
            }
        }

        /// Keep the panel's reveal machine up to date with where the pointer is.
        ///
        /// **The machine is `alo-dock`'s and the state is this binary's**, the same division as
        /// the peek. `Revealing` decides what being at the edge or on the surface means — that
        /// any one region is enough, so leaving one cannot conceal the panel while another
        /// still holds it — and this binary only keeps the answer.
        fn the_panel_is_revealed(&mut self, by: alo_dock::revealing::ThePointer) {
            self.revealing = self.revealing.the_pointer_is(by);
        }

        /// A pointer over a preview is a person looking at the window in it.
        ///
        /// **The rule is `alo-put-aside`'s and the `Panel` is this binary's**, which is the
        /// whole of why this method is here rather than in the compositor:
        /// `alo_put_aside::peeking_at_a_preview` decided on 2026-09-30 what a peek may change
        /// and what letting go leaves behind, and the compositor holds no `Panel` to ask it
        /// with. The classification arrives already made, because the crate that laid the panel
        /// out is the one that knows which rectangle a point is in.
        ///
        /// **A peek at a window the panel no longer holds is refused, not invented.** The
        /// classification names what was *drawn* at that point, so a person who pointed at a
        /// preview that has since been brought back is pointing at a picture of something gone.
        /// `peek_at` answers `ItIsNotThere`, the peek is left exactly as it was, and that is the
        /// crate's own rule doing the work rather than this binary second-guessing it.
        ///
        /// **Leaving the panel ends a peek; the gaps between previews do not.** A pointer in
        /// the panel's own region but on no preview is ground a person crosses on the way to
        /// one, and ending the peek there would make a peek impossible to hold while reaching
        /// for it.
        fn the_pointer_is_now(
            &mut self,
            on: alo_shell::which_preview_the_pointer_is_on::OnThePanel,
        ) {
            // **The decision is `alo-shell`'s and the state is this binary's**, which is why
            // this is two lines rather than a match. A match here would be a second copy of
            // the rule in a binary that cannot test itself, and the next person would have two
            // answers to one question — the fault this repository spent 2026-10-01 removing.
            if let Ok(what) =
                alo_shell::what_a_classification_does_to_a_peek(on, self.peeking, &self.put_aside)
            {
                self.peeking = what.now();
            }
        }

        /// The panel this desktop keeps, lent for the one act that changes it.
        ///
        /// **`Some`, because this desktop has a panel** — the field above. The trait defaults
        /// to `None` for a desktop that has none, and the display probe is one of those.
        fn the_panel(&mut self) -> Option<&mut alo_put_aside::Panel> {
            Some(&mut self.put_aside)
        }

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
                put_aside: &self.put_aside,
                // **The machine's own answer**, asked once a frame. `Revealing` decides this
                // from the edge, the surface, the keyboard, a drag and an open menu; this
                // binary holds it and the compositor draws what it says.
                panel_is_revealed: self.revealing.is_revealed(),
                filling_the_screen: false,
                // **One to one, and it is owed rather than chosen.** 100 is the
                // right answer for a display drawing one pixel per logical one,
                // and it is what every surface effectively got until
                // 2026-10-02: `division_raster` took a scale and its only
                // caller passed a literal.
                //
                // What is missing is not the conversion, which now works, but a
                // *source*. This binary holds no `Screens` and no
                // `alo_displays::Scale`, so it has nothing true to put here for
                // a dense screen. The real one is `ScreenPlace`'s own scale,
                // for whichever display this frame is drawn on. Until this
                // binary reads the displays, a two-times screen draws a
                // division at half the room it owns — and saying so here is the
                // difference between a default and a silence.
                display_scale: 100,
                look: self.look,
                strings: &self.strings,
                egress: &self.egress,
                running: &self.running,
                filling: &self.filling,
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
                // **Empty here, and answered on the way to the frame.** This
                // struct holds what the crates that own each thing said, and
                // nothing here owns the windows: they are the compositor's,
                // and this binary has no server in it to ask. `alo-shell`'s
                // direct desktop is the one place holding both a server and
                // this frame, and it replaces this with
                // `Server::window_areas` before anything is drawn.
                //
                // So this is not the value the dock is asked about. It is
                // what the frame says before the only caller that can answer
                // has answered — and an empty slice is the honest thing to
                // say meanwhile, because it is also exactly what a desktop
                // with nothing open would report.
                windows: &[],
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
