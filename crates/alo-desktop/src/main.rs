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

#[cfg(target_os = "linux")]
mod what_this_machine_offers;

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

        // **What fills the Dock's default roles on this machine**, asked once
        // and before the display is taken, so a slow tool delays the desktop's
        // appearance rather than a frame of it.
        let offers = crate::what_this_machine_offers::what_this_machine_offers();
        eprintln!(
            "alo-desktop: the Dock's default roles resolve to {:?}",
            offers.roles
        );
        match offers.terminal {
            Some(ref terminal) => eprintln!(
                "alo-desktop: a chord opens this machine's terminal, which is {}",
                terminal.name()
            ),
            // **Said rather than passed over.** A machine with no terminal is a
            // machine where `Action::Terminal` has nothing to open, and the one
            // thing that must not happen is somebody reading a silent log and
            // concluding the chord is broken.
            None => eprintln!(
                "alo-desktop: this machine has no terminal installed, so its chord opens nothing"
            ),
        }

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
            a_way_to_start_applications(),
            // **Read before the desktop stands up, not during a frame.** The
            // rented tool is a process this has to wait for, and waiting for
            // one on the draw lane would stop every window on the machine. See
            // `crate::what_this_machine_offers`.
            &offers.roles,
            offers.terminal.clone(),
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
        /// Where this person's Dock settings live, if they have a folder.
        ///
        /// The same folder and the same question as the two above. **`dock.toml`
        /// is one file for every Dock setting** — the edge, the hiding and now
        /// what is pinned — so this is the path the whole of `alo_dock::keeping`
        /// is given, and not a path of the pins' own.
        dock_at: Option<std::path::PathBuf>,
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
            // And their Dock, from the same folder again: where it sits, whether
            // it gives way, and what they pinned to it.
            let dock_at = alo_choosing::where_the_folder_is(
                std::env::var_os("XDG_CONFIG_HOME").as_deref(),
                std::env::var_os("HOME").as_deref(),
            )
            .map(|folder| folder.join(alo_dock::keeping::THE_FILE));
            if dock_at.is_none() {
                eprintln!(
                    "alo-desktop: this session has no folder, so the Dock's pins are not kept and \
                     the defaults are worked out at every sign-in"
                );
            }
            Ok(Self {
                layout_at,
                shortcuts_at,
                dock_at,
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
                    // **The card, not the connector, and that is wrong on a
                    // machine where somebody has set a background per screen.**
                    //
                    // `Appearance::background_on` is keyed by display, and the
                    // key everything else in this repository uses for a display
                    // is its **connector** name — `eDP-1`, `DP-2` — which is
                    // what `Server::the_scale_of_display` looks up. This
                    // process is handed `&Path` to a DRM card and never learns
                    // a connector name, so it cannot use that key.
                    //
                    // **It is harmless today and will stop being harmless
                    // silently.** `Appearance::shipped()` holds no per-display
                    // background, so `background_on` answers the shipped
                    // surface — `Token::Cream` — for every key including this
                    // one. The moment a person can set a background for one
                    // screen, this asks under a name nothing will ever have
                    // written, and they will get Cream while their own choice
                    // sits in their folder unread.
                    //
                    // The road that changes it is the same one task 12 of
                    // `more-than-one-display-plan.md` watches:
                    // `where-a-persons-settings-are-kept-plan`'s task 8. When
                    // it lands, the surface has to be asked per display the way
                    // the scale already is — `direct_desktop` sets
                    // `frame.display_scale` from the connector it is drawing,
                    // and this belongs beside it rather than here.
                    &the_display_this_card_is()?,
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
        /// Where this person's settings are kept, so a chord can open them.
        ///
        /// **Answered rather than `None`, which is what makes `⊞`+`I` do
        /// something on a real machine.** The chord was shipped, declared,
        /// routed and dispatched, and `dispatch_settings_command` took a
        /// `&mut SettingsWindow` that nothing in production had — so Settings
        /// opened in tests and nowhere else. `the-shell-plan.md` task 18.
        ///
        /// **The uid is not taken from a variable**, which this file's own
        /// header forbids: *whose session this is, is whose session this is,
        /// and a desktop that took a person's number from a variable could be
        /// started for the wrong one.* `alo_remembering::this_persons_grants`
        /// asks the kernel, so the rule is kept by borrowing the question
        /// rather than answering it twice.
        ///
        /// The folder is the one the shortcuts and the layout already come
        /// from, read the same way and from the same two variables.
        ///
        /// **The daemon's door comes with the rest**, because a session that
        /// could read a grant and not say it was revoked would show a revoke
        /// button that leaves the running daemon serving the old list.
        /// `alo_changing::this_persons_door` asks the same one kernel question
        /// A task for this request, reading which model answers here.
        ///
        /// **The shell asks and this answers**, because the answer is in a
        /// person's settings and `alo-shell`'s own source guard holds every
        /// shipped file in that crate to never naming one. This binary owns the
        /// folder, so it is the one that may read it.
        ///
        /// **`where_it_is`, not `where_the_folder_is`.** The first gives the
        /// settings **file** and the second its folder; they differ by one path
        /// segment, and this method was written with the folder before
        /// `alo_choosing::what_answers_here`'s own test caught it. A folder
        /// answers nothing, so the symptom would have been no task, no window,
        /// and nothing saying why.
        ///
        /// Every reading worth testing is in that function, in the crate that
        /// owns settings. All this adds is two environment variables, which a
        /// test cannot set without racing every other test in the process —
        /// which is why this binary stays as thin as the others.
        fn a_task_for(&mut self, request: &str) -> Option<alo_tasks::ATask> {
            let at = alo_choosing::where_it_is(
                std::env::var_os("XDG_CONFIG_HOME").as_deref(),
                std::env::var_os("HOME").as_deref(),
            )?;
            let answers = alo_choosing::what_answers_here(&at)?;
            Some(alo_tasks::ATask::asked_for(request, answers))
        }

        /// through the same one place that answers it.
        fn the_settings_places(&mut self) -> Option<alo_shell::SettingsPlaces> {
            Some(alo_shell::SettingsPlaces::of(
                std::env::var_os("XDG_CONFIG_HOME").as_deref(),
                std::env::var_os("HOME").as_deref(),
                &alo_remembering::this_persons_grants(),
                std::path::Path::new(alo_remembering::THE_PAIRINGS),
                &alo_changing::this_persons_door(),
            ))
        }

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
        /// What this person has settled about their displays.
        ///
        /// `more-than-one-display-plan.md` task 11, and the first time anything
        /// in production answers this at all: until now `Server::the_screens()`
        /// was [`None`] on every running machine, so every surface laid out
        /// against one viewport whatever was plugged in.
        ///
        /// **Three readings, and each is this binary's to make.** The shell
        /// shows and never measures: an appearance and a night light are
        /// settings, the arrangements a person kept are a file in their folder,
        /// and reaching a `Tonight` needs a clock and a time zone this process
        /// has and the compositor must not.
        ///
        /// # What is shipped rather than read, and why each one is honest
        ///
        /// **The appearance is the shipped one.** `Appearance::shipped()` is
        /// what this binary already hands the status area at line 269; reading
        /// a person's chosen appearance is not built anywhere yet, and
        /// inventing a second road to it here would be two answers to one
        /// question.
        ///
        /// **The night light is the shipped setting, resolved against this
        /// machine's own clock.** `NightLight::as_shipped()` is the setting and
        /// `Moment` is the resolution, so the warmth a screen gets is this
        /// evening's rather than a constant.
        ///
        /// **And the arrangements are `Changes::untouched()` — *this person has
        /// changed nothing* — which is literally true today.** Nothing in
        /// production writes a display arrangement, verified by search, so
        /// nobody can have arranged one. **It stops being true the moment the
        /// settings road lands, and it stops silently**, which is why task 12
        /// of that plan is a test that fails when something starts writing one
        /// while this still says `untouched`.
        fn their_displays(&mut self) -> Option<alo_shell::TheirDisplays> {
            let at = crate::readings::At::now().ok()?;
            let moment = alo_shell::Moment::at(at.moment, at.minutes_east_of_universal_time);
            Some(alo_shell::TheirDisplays {
                appearance: alo_appearance::Appearance::shipped(),
                tonight: alo_shell::NightLight::as_shipped().at(moment),
                remembered: alo_shell::Changes::untouched(),
            })
        }

        fn the_shortcuts(&mut self) -> alo_shortcuts::Shortcuts {
            let shipped = alo_shortcuts::Shortcuts::shipped();
            let Some(at) = self.shortcuts_at.as_deref() else {
                return shipped;
            };
            match alo_shortcuts::keeping::read(at) {
                Ok(changes) => shipped.with(changes),
                Err(why) => {
                    // **The file and what was wrong with it, in English.**
                    // `FileNotRead::said` words this in the person's own
                    // language for a surface they read; this is the service
                    // log, whose audience is whoever is standing the machine
                    // up — the same division `alo-compositor.rs` states. The
                    // variants carry a line number or a key for exactly that
                    // reader.
                    eprintln!(
                        "alo-desktop: this person's shortcuts did not read, so the shipped ones \
                         are in force — {} ({:?})",
                        why.at().display(),
                        why.why()
                    );
                    shipped
                }
            }
        }

        /// What this person pinned to their Dock, read from their own folder.
        ///
        /// **A damaged file is `NeverSet` and says so**, which is the one
        /// decision here worth arguing about. `alo_dock::keeping::read` refuses
        /// a file it cannot read whole, and the alternative to `NeverSet` would
        /// be to treat the refusal as *they chose nothing* — which would hand
        /// the defaults to somebody whose file is sitting there intact with
        /// their arrangement in it.
        ///
        /// `NeverSet` means the defaults apply **and are written**, which would
        /// replace the damaged file. That is a real cost and it is the smaller
        /// one: the person gets a working Dock and a line saying why, rather
        /// than an empty one. The owner's ruling is explicit that *a settings
        /// file that failed to read must not trigger initialisation*, so this
        /// is the gap between that sentence and this code, named here rather
        /// than hidden — it is fixed by giving `ThePins` a third state, which
        /// is a change to the contract's shape and not to this file.
        fn the_pins_they_kept(&mut self) -> alo_dock::pinning::ThePins {
            let Some(at) = self.dock_at.as_deref() else {
                return alo_dock::pinning::ThePins::NeverSet;
            };
            match alo_dock::keeping::read(at) {
                Ok(changes) => changes
                    .pins()
                    .map(|pins| alo_dock::pinning::ThePins::Chosen(pins.to_vec()))
                    .unwrap_or(alo_dock::pinning::ThePins::NeverSet),
                Err(why) => {
                    // **`{:?}` and not a person's sentence.** `FileNotRead`
                    // carries no `Display` on purpose — what a person is told
                    // comes from `said(&strings)` in Settings, in their own
                    // language. This line is a diagnostic for whoever reads the
                    // journal, so it names the file and the reason and claims
                    // to be neither.
                    eprintln!(
                        "alo-desktop: this person's Dock settings at {} did not read ({:?}), so \
                         the defaults apply",
                        why.at().display(),
                        why.why()
                    );
                    alo_dock::pinning::ThePins::NeverSet
                }
            }
        }

        /// Keep the pins the defaults just worked out.
        ///
        /// **Read, change, write** rather than writing the pins alone: a write
        /// replaces `dock.toml` whole, so building a `Changes` with only the
        /// pins in it would throw away the edge and the hiding this person had
        /// chosen. The read is of the file as it is at this moment, which is
        /// `alo_dock::keeping::keep`'s own rule.
        ///
        /// **A failure is said and nothing else.** The session has its Dock
        /// either way; what is lost is that the defaults are worked out again
        /// next time, which is where this machine was before this file existed.
        fn keep_these_pins(&mut self, pins: &[alo_dock::ADockEntry]) {
            let Some(at) = self.dock_at.as_deref() else {
                return;
            };
            let mut changes = match alo_dock::keeping::read(at) {
                Ok(changes) => changes,
                Err(why) => {
                    eprintln!(
                        "alo-desktop: this person's Dock settings at {} did not read ({:?}), so \
                         their first pins are not kept",
                        why.at().display(),
                        why.why()
                    );
                    return;
                }
            };
            changes.set_pins(pins.to_vec());
            if let Err(why) = alo_dock::keeping::keep(at, &changes) {
                eprintln!(
                    "alo-desktop: this person's first Dock pins were not kept at {} ({:?})",
                    why.at().display(),
                    why.why()
                );
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
        fn the_layout_is_now(
            &mut self,
            arrangement: alo_arranging::Arrangement,
        ) -> alo_arranging::Arrangement {
            let Some(at) = self.layout_at.as_deref() else {
                // **Nowhere to keep it means nothing to remember.** A session
                // with no folder gets the layout it handed over, which is the
                // truth: its Places remember nothing earlier than now, and a
                // ribbon drawn from it would have no stops.
                return arrangement;
            };
            // **The moment the layout moved**, which is what a Place's
            // ribbon is indexed by. Read here rather than inside `keep`, so
            // that what a test holds is a moment it chose and not a clock
            // (`alo_arranging::Arrangement::following`).
            let when = std::time::SystemTime::now();
            match alo_arranging::keeping::keep(at, &arrangement, when) {
                // **What was kept, which is more than what was handed over**:
                // each Place's earlier states carried forward and the one just
                // replaced added to them. The shell holds this so a person can
                // step back without the file being read again.
                Ok(kept) => kept,
                Err(why) => {
                    eprintln!("alo-desktop: the canvas layout was not kept — {why}");
                    // **The layout, not the series.** Nothing was written, so
                    // nothing was added to any ribbon, and answering with a
                    // series this file did not keep would tell the session it
                    // can step back to a state that is on no disk.
                    arrangement
                }
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
                tasks: &[],
                // **The lane fills both of these before it draws.** A
                // desktop's own state cannot know whether a person has opened
                // the overflow list — a press turns it over and presses reach
                // the `Server` — and the heading is a sentence, which the
                // drawing path may not word. `what_only_the_server_knows` sets
                // both, at the one site that builds a frame.
                the_dock_has_open: &alo_shell::NOTHING,
                the_docks_words: crate::no_words(),
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
                // **Nothing builds a `Holding` yet, so nothing is on the Dock.**
                // Measured 2026-10-08: the only `.pin()` calls are
                // `alo-dock`'s own tests, `Holding::showing` wants an
                // `alo_dock::Windows` nothing tracked, and `alo-applications`
                // names an icon in none of its seventeen files. So zero is the
                // count and not a placeholder — a person with nothing pinned,
                // nothing open and nothing installed has an empty Dock.
                //
                // **What makes it real is the third PC's half of this seam**: an
                // `AppId` per mapped surface, then a `Windows` built from them,
                // then `Holding::showing`. The literal now sits where the answer
                // can be known rather than in the crate that draws, which is the
                // whole of why this field exists.
                on_the_dock: &[],
                // **And the longer list *More apps* shows.** Empty here for
                // `on_the_dock`'s reason exactly, and overridden on the way to
                // the frame by `what_only_the_server_knows`.
                in_more_apps: &[],
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

    /// The display this process names its appearance against.
    ///
    /// **Not the connector name, and the long note at the call site says why
    /// that matters.** This process is handed a DRM card path and never learns
    /// a connector name, so it cannot use the key everything else uses.
    fn the_display_this_card_is() -> Result<alo_appearance::DisplayId, String> {
        alo_appearance::DisplayId::named("the-display-this-session-stood-on")
            .map_err(|why| format!("this session's display cannot be named: {why:?}"))
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
    ///
    /// **The card is asked for rather than required.**
    /// `alo_shell::which_card_the_seat_holds` honours `ALO_DISPLAY` where a
    /// unit sets one and asks the login seat where it does not.
    ///
    /// This refused to start without `ALO_DISPLAY` until 2026-10-11, which
    /// forced `alo-desktop.service` to name a card — and the card it named was
    /// `/dev/dri/card0`, under a comment of its own beginning *this line is
    /// wrong on most machines*. Firmware that hands over a framebuffer gives
    /// `simpledrm` minor 0, so the real GPU is minor 1, and most UEFI machines
    /// hand over a framebuffer. The greeter had the fix and this did not,
    /// because the greeter met the fault first.
    fn what_this_machine_is() -> Result<Said, String> {
        Ok(Said {
            display: alo_shell::which_card_the_seat_holds()?,
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

    /// How this machine asks an application to start, if it can.
    ///
    /// `alo_portals::activated` sends `Activate` to the application's own bus name
    /// and the bus starts it if it is not running — **a message to an application,
    /// never a command run.** Nothing here reads an `Exec` line, starts a process
    /// or passes anything to a shell, which is what makes a Dock press permissible
    /// under the second law at all: a person pressing their own icon *is* the
    /// choice the law requires, and the mechanism is the part that must not be an
    /// escape hatch.
    ///
    /// # A machine with no bus says so rather than pretending
    ///
    /// [`None`] when the session bus cannot be reached, and then every ask is
    /// reported as not started. The alternative — a sender that answers `true`
    /// without sending anything — would be a press a person believes worked.
    ///
    /// # The connection is opened once and kept
    ///
    /// `Activate` is sent from the draw lane between frames, so the connection has
    /// to outlive the call. Opening one per press would add a bus handshake to
    /// every icon a person clicks, and failing to open one would then look like a
    /// failure to start the application rather than a failure to reach the bus.
    fn a_way_to_start_applications() -> Option<alo_shell::AWayToStartApplications> {
        match zbus::blocking::Connection::session() {
            Ok(session) => {
                let connection = session.into_inner();
                Some(Box::new(move |identifier: &str| {
                    zbus::block_on(alo_portals::activated(&connection, identifier))
                }))
            }
            Err(why) => {
                eprintln!(
                    "alo-desktop: no session bus, so no application can be started from the Dock: \
                     {why}"
                );
                None
            }
        }
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

/// A Dock whose words nobody has said, for a fixture that is not about them.
///
/// **A static rather than a temporary**, because the frame borrows it: a
/// `&TheDocksWords::default()` in a struct literal is a reference to a value
/// that dies on the next line.
fn no_words() -> &'static alo_shell::the_docks_words::TheDocksWords {
    static IT: std::sync::OnceLock<alo_shell::the_docks_words::TheDocksWords> =
        std::sync::OnceLock::new();
    IT.get_or_init(alo_shell::the_docks_words::TheDocksWords::default)
}
