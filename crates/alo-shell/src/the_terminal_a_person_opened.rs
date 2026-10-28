//! alo's own terminal: the shell it runs, the screen it keeps, and when it has
//! the keyboard.
//!
//! # Why the compositor holds it at all
//!
//! [ADR 0114](../../docs/decisions/0114-the-terminals-engine-is-pinned-and-its-face-is-bundled.md)
//! settles it: *not a terminal application. No window of its own, no GPU path,
//! no winit. The compositor draws it, as it draws everything else.* And the
//! reason is a promise rather than a preference —
//! [ADR 0113](../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
//! says a gutter beside the transcript belongs to the terminal and **no program
//! can write into it**, which nothing running as somebody else's application
//! could ever guarantee.
//!
//! # It is the person's, and no agent reaches it
//!
//! `alo-terminal` runs the person's own `$SHELL`. Nothing in this file is a
//! verb, nothing here is on the broker's list, and there is no road from an
//! agent to any of it. Law 2 binds the agent and explicitly permits this: *alo
//! OS ships a terminal, because a system that does not trust its owner with a
//! shell is a toy.* Whether an agent may ever use one is ADR 0111's question
//! and is not answered here.
//!
//! # Nothing is read for meaning
//!
//! Bytes come out of a child process and go into a grid. **No byte is matched
//! against a pattern, inspected or sent anywhere**, and nothing in this file
//! reaches a model or the network.

use alo_terminal::{AShellOnAPseudoterminal, HowBig, TerminalOutput, WhatArrived};

/// The grid a shell is told about before anything has laid out a window.
///
/// **Not a screen size, and not a measurement.** A shell has to be told *some*
/// size the moment it starts — a program asking `stty size` before the first
/// frame must get an answer — and at that moment no window has been laid out
/// and no face has been measured, so there is nothing to derive one from.
///
/// So this is the conventional starting grid, and **the drawing replaces it**
/// with the real one: how many columns and lines a terminal has is the
/// transcript's width divided by the face's own advance and its height divided
/// by `alo_terminal::measures::A_LINE`, both measured at the size the display
/// is drawing. `measures`' own test asserts the design's transcript holds
/// **32** lines and not 24, which is there to make exactly this point: 80 by 24
/// is a figure a test chooses, not one a window has.
///
/// The scrollback is a choice rather than a measurement — `docs/features.md`
/// promises scrollback a person can come back to, and does not say how much —
/// so it is stated here as a choice and is the one number in this file somebody
/// may reasonably want to argue with.
pub(crate) const A_SENSIBLE_FIRST_GRID: HowBig = match HowBig::of(80, 24, 1_000) {
    Ok(size) => size,
    // **Unreachable, and a `const` block is what proves it.** `HowBig::of`
    // refuses only a zero, and neither of those is zero, so this arm cannot be
    // taken — and because this is a constant, a change that made it reachable
    // would fail the **build** rather than a test.
    Err(_) => panic!("eighty by twenty-four is a terminal"),
};

/// alo's own terminal, open on this machine.
///
/// **One, not a list.** `docs/features.md` promises sessions inside a terminal
/// rather than many terminal windows, and the design draws a session strip
/// inside one window. A second window is not a thing this file refuses — it is
/// a thing nothing has asked for, and adding the list before the thing exists
/// would be inventing a shape.
pub(crate) struct TheTerminal {
    /// The person's shell, on its own pseudoterminal.
    shell: AShellOnAPseudoterminal,
    /// What the shell has written, as a grid of cells.
    screen: TerminalOutput,
    /// Whether the shell has gone.
    ///
    /// **Kept rather than asked each time**, because the answer is
    /// irreversible and a reader that asked again after the far end closed
    /// would be asking a question whose answer it already had.
    finished: bool,
    /// What alo has said into this transcript, beneath what the shell wrote.
    ///
    /// Empty until a person asks. `docs/features.md`'s *a command looks stuck;
    /// **ask**, and get …* — the answers were built and nothing could ask, so
    /// this is where an answer goes once one has.
    ///
    /// **Held by the terminal and not by the drawing**, because it outlives a
    /// frame: a person asks once and reads the answer for as long as it is true.
    /// A drawing that worked it out each frame would be asking the kernel about a
    /// process sixty times a second, which is the background reader
    /// `CLAUDE.md` calls a bug.
    what_alo_said: alo_terminal::WhatAloSaid,
    /// A run of tests recognised in what the program printed, if there is one.
    ///
    /// **Worked out when the output changes and not when it is drawn**, for the
    /// reason above: a drawing that recognised afresh each frame would walk the
    /// whole screen sixty times a second to answer a question whose answer only
    /// changes when a program writes.
    ///
    /// [`None`] for nearly everything a terminal ever shows, and that is the
    /// ordinary case rather than a failure — the design draws its text fallback
    /// first and says so in its own words.
    a_test_result: Option<alo_terminal::ATestResult>,
    /// A benchmark comparison recognised in what the program printed, if there
    /// is one.
    ///
    /// Recognised beside the test result and for the same reason: this walks
    /// the screen, so doing it in the drawing would walk it every frame to
    /// answer a question that changes only when a program writes.
    a_benchmark: Option<alo_terminal::ABenchmark>,
    /// A file diff recognised in what the program printed, if there is one.
    a_diff: Option<alo_terminal::ADiff>,
    /// The folder the shell is in, written as a person reads it, for the strip
    /// across the top — `~/Projects/launch`.
    ///
    /// **Asked of the kernel when the shell has written something, and at no
    /// other time**, for the reason the three fields above give. A `cd` is
    /// always followed by a new prompt, so the strip follows the person from
    /// folder to folder; nothing reads it while the shell is quiet, and nothing
    /// is told when it changes.
    the_folder: Option<String>,
}

impl TheTerminal {
    /// Start the person's own shell on a grid this big, in the home folder
    /// `home` names — the session's `$HOME`.
    ///
    /// **A home that cannot be used is said, not hidden.** The shell still
    /// starts, in `/`, and the reason goes to the error stream, because *never
    /// a silent fallback*: a person whose terminal opens somewhere unexpected
    /// is owed a line somebody mending the machine can find.
    ///
    /// # Errors
    /// Whatever stopped the pseudoterminal from opening or the shell from
    /// running.
    pub(crate) fn opened(
        size: HowBig,
        home: Option<&std::ffi::OsStr>,
    ) -> Result<Self, alo_terminal::NoShell> {
        let shell = AShellOnAPseudoterminal::the_persons_own(size, home)?;
        if let alo_terminal::WhereItStarts::NotTheirHome(why) = shell.where_it_started() {
            eprintln!("alo-shell: alo's own terminal did not open at home: {why}");
        }
        let mut terminal = Self {
            shell,
            screen: TerminalOutput::of(size),
            finished: false,
            what_alo_said: alo_terminal::WhatAloSaid::nothing(),
            a_test_result: None,
            a_benchmark: None,
            a_diff: None,
            the_folder: None,
        };
        terminal.the_folder = terminal.where_the_shell_is_now();
        Ok(terminal)
    }

    /// What the shell has written since the last ask, applied to the screen.
    ///
    /// **Called once a frame and never blocking.** The master end is
    /// non-blocking, so a shell with nothing to say costs one read that finds
    /// nothing — which is the ordinary case and must not stop the compositor.
    ///
    /// Answers whether anything changed, so a caller can skip redrawing a
    /// terminal nobody has typed in. **`false` for a shell that has finished**:
    /// there is nothing further to draw and saying `true` forever would redraw
    /// a dead terminal every frame.
    pub(crate) fn what_the_shell_wrote(&mut self) -> bool {
        if self.finished {
            return false;
        }
        // **Read until it stops giving**, not once. A single read takes at most
        // one buffer, and a command that printed more than that would arrive a
        // frame at a time — visible as a terminal filling in slowly for no
        // reason.
        let mut changed = false;
        loop {
            match self.shell.what_it_wrote(&mut self.screen) {
                Ok(WhatArrived::Bytes(_)) => changed = true,
                Ok(WhatArrived::NothingYet) => break,
                Ok(WhatArrived::ItFinished) => {
                    self.finished = true;
                    break;
                }
                // **A failure is reported and the terminal kept.** What is on
                // the screen is what the shell said, and throwing it away
                // because a later read failed would lose a person's work. The
                // error stream rather than silence, because *never a silent
                // fallback*.
                Err(why) => {
                    eprintln!("alo-shell: the terminal's shell could not be read: {why}");
                    self.finished = true;
                    break;
                }
            }
        }

        // **Recognised once, here, and only when something arrived.** This walks
        // the screen, so doing it in the drawing would walk it every frame to
        // answer a question that changes only when a program writes.
        //
        // **The three roads out of the loop above used to be three `return`s**,
        // and this is why they are now one `break`: a recognition on two of
        // three paths is the shape of fault where a result appears for a running
        // command and vanishes when it exits.
        if changed {
            let lines = self.screen.lines();
            self.a_test_result = alo_terminal::a_test_result_in(&lines);
            self.a_benchmark = alo_terminal::a_benchmark_in(&lines);
            self.a_diff = alo_terminal::a_diff_in(&lines);
            self.the_folder = self.where_the_shell_is_now();
        }
        changed
    }

    /// The folder the shell is in, asked of the kernel now and written with `~`
    /// for the person's home.
    ///
    /// **The home is the one the shell started in**, and only when that was
    /// the person's: a shell that fell back to `/` has no home of theirs to
    /// abbreviate, and every folder is then written in full.
    fn where_the_shell_is_now(&self) -> Option<String> {
        let home = match self.shell.where_it_started() {
            alo_terminal::WhereItStarts::TheirHome(home) => Some(home.as_path()),
            alo_terminal::WhereItStarts::NotTheirHome(_) => None,
        };
        self.shell
            .where_the_shell_is()
            .map(|folder| alo_terminal::the_folder_as_a_person_reads_it(&folder, home))
    }

    /// The folder the shell was in when it last wrote, as a person reads it.
    ///
    /// [`None`] where the kernel would not say.
    pub(crate) fn the_folder(&self) -> Option<&str> {
        self.the_folder.as_deref()
    }

    /// A file diff recognised in what the program printed, if there is one.
    ///
    /// [`None`] for nearly everything, which is the ordinary case.
    pub(crate) const fn a_diff(&self) -> Option<&alo_terminal::ADiff> {
        self.a_diff.as_ref()
    }

    /// **What the shell has said about its prompt since this was last asked.**
    ///
    /// Drained, so one mark reaches one caller once.
    pub(crate) fn what_the_shell_said(
        &mut self,
    ) -> Vec<
        Result<
            alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid,
            alo_terminal::what_the_shell_says_about_the_prompt::NotRead,
        >,
    > {
        self.shell.what_the_shell_said()
    }

    /// **Put alo's explanation of a command beneath the transcript.**
    ///
    /// Replaces whatever alo last said in this terminal, which is the same rule
    /// the waiting answer keeps: alo has one voice here and the newest thing it
    /// said is the thing that is true.
    pub(crate) fn alo_explains(&mut self, what: alo_terminal::WhatACommandWillDo) {
        self.what_alo_said.explains(what);
    }

    /// A benchmark comparison recognised in what the program printed, if there
    /// is one.
    pub(crate) const fn a_benchmark(&self) -> Option<&alo_terminal::ABenchmark> {
        self.a_benchmark.as_ref()
    }

    /// A run of tests recognised in what the program printed, if there is one.
    pub(crate) const fn a_test_result(&self) -> Option<&alo_terminal::ATestResult> {
        self.a_test_result.as_ref()
    }

    /// What a person typed, sent to the shell.
    ///
    /// **Bytes, not a command.** What arrives is what the keyboard produced,
    /// including control characters and an arrow key's escape sequence, and
    /// nothing is appended: a caller that wants a newline sends one.
    ///
    /// Answers whether the shell took it. A shell that has gone takes nothing,
    /// which is a true answer and not a failure.
    pub(crate) fn a_person_typed(&mut self, bytes: &[u8]) -> bool {
        if self.finished {
            return false;
        }
        // **Typing puts the newest output back on screen**, which is the
        // design's *current input stays reachable*. Typing into a view of the
        // past and watching nothing appear is the most confusing thing a
        // scrolled terminal can do, and every terminal does this.
        self.screen.back_to_the_newest();
        match self.shell.a_person_typed(bytes) {
            Ok(()) => true,
            Err(why) => {
                eprintln!("alo-shell: what was typed could not reach the terminal: {why}");
                self.finished = true;
                false
            }
        }
    }

    /// One keypress, turned into bytes and sent to the shell.
    ///
    /// **The screen answers the cursor-key mode and the shell takes the
    /// bytes**, both from this one struct, which is why the conversion lives
    /// here rather than at the call site: a caller holding only `&mut Server`
    /// cannot borrow the screen to ask the mode and the shell to write at the
    /// same time, and the two fields of one struct can.
    ///
    /// Answers whether anything was sent. A key that produces no bytes sends
    /// nothing, which is ordinary.
    pub(crate) fn a_person_typed_a_key(
        &mut self,
        typed: alo_terminal::what_a_person_typed::WhatWasTyped,
        held: alo_terminal::what_a_person_typed::Held,
    ) -> bool {
        let bytes =
            alo_terminal::what_a_person_typed::what_the_shell_receives(typed, held, &self.screen);
        if bytes.is_empty() {
            return false;
        }
        self.a_person_typed(&bytes)
    }

    /// Lay this terminal out at a new size.
    ///
    /// **The order is the whole of it: the grid first, then the child.**
    /// Telling the child sends it `SIGWINCH` and it redraws *immediately* — so
    /// a child told before the grid was laid out again writes its new frame
    /// into a grid that is still the old shape. Every Unix terminal does these
    /// two steps in this order, and there is no third step.
    ///
    /// Answers whether anything changed, which is **the throttle**: a caller
    /// works the size out every frame and this does nothing on the frames where
    /// it is the same. A resize per frame would make a full-screen program
    /// unusable.
    pub(crate) fn laid_out_again(
        &mut self,
        size: HowBig,
        cell: alo_terminal::ACellInPixels,
    ) -> bool {
        if self.finished || !self.screen.laid_out_again(size) {
            return false;
        }
        if let Err(why) = self.shell.the_window_changed(size, cell) {
            // **Said rather than swallowed, and the grid keeps the new size.**
            // The grid is what gets drawn, so a person sees the right shape
            // even on a machine whose pseudoterminal refused the `ioctl`; what
            // they lose is programs knowing, which is what this line is for.
            eprintln!("alo-shell: the terminal's shell could not be told its new size: {why}");
        }
        true
    }

    /// The screen, for whatever draws it.
    /// **A person asked why the command in front is waiting, so alo looks and
    /// says.**
    ///
    /// `docs/features.md` promises *a command looks stuck; **ask**, and get
    /// **waiting for the keyboard**, **for the network**, **for another process
    /// or a lock**, **still computing**, or **unknown with the evidence
    /// available***. The five answers were built on 2026-10-14, the asking was
    /// wired on 2026-10-17, and **nothing called it**. This is the call.
    ///
    /// Answers whether anything changed, so a caller redraws only when it did —
    /// the rule every other method here follows.
    ///
    /// # Nothing waiting is an answer, and it clears the last one
    ///
    /// A person at a prompt has nothing to ask about, and a command that has
    /// finished since they last asked has no answer either. Both empty what alo
    /// said rather than leaving it, because **a reply about a command that has
    /// gone would sit on screen looking current** — and a person would act on
    /// it. `crate::alos_terminal_was_asked_why_tests` holds that.
    pub(crate) fn asked_why_the_command_in_front_is_waiting(&mut self) -> bool {
        let before = self.what_alo_said.clone();
        match self.shell.why_the_command_in_front_is_waiting() {
            Some(waiting) => self.what_alo_said.says(waiting.why.the_word().key()),
            None => self.what_alo_said.nothing_to_say(),
        }
        before != self.what_alo_said
    }

    /// What alo has said into this transcript.
    pub(crate) const fn what_alo_said(&self) -> &alo_terminal::WhatAloSaid {
        &self.what_alo_said
    }

    /// The screen, for whatever draws it.
    pub(crate) fn screen(&self) -> &TerminalOutput {
        &self.screen
    }

    /// Whether the shell has gone.
    pub(crate) fn has_finished(&self) -> bool {
        self.finished
    }

    /// What the shell running here is called, for the note under the
    /// transcript.
    ///
    /// Read from the running process rather than from `$SHELL`: those can
    /// differ and only one of them is what a person is typing into.
    pub(crate) fn what_the_shell_is_called(&self) -> Option<String> {
        self.shell.what_the_shell_is_called()
    }

    /// Move the view back into what is remembered, or forward towards the
    /// newest. Whether it moved.
    pub(crate) fn look_back(&mut self, lines: i32) -> bool {
        self.screen.look_back(lines)
    }
}

impl crate::Server {
    /// Open alo's own terminal, or leave the one already open alone.
    ///
    /// **Pressing the chord twice does not start a second shell.** The design
    /// draws one terminal window with a session strip inside it, so a second
    /// press is a person reaching for the terminal they have — and starting
    /// another shell would be a process they did not ask for.
    ///
    /// Answers whether a terminal is now open, so a caller can tell *there is
    /// one* from *this machine could not start one*.
    pub(crate) fn open_alos_own_terminal(&mut self, size: HowBig) -> bool {
        if self.alos_own_terminal.is_some() {
            return true;
        }
        // **The session's own `$HOME`**, which is the person's: the compositor
        // runs in their session, and `crate::settings_places` reads the same
        // variable for the same person.
        match TheTerminal::opened(size, std::env::var_os("HOME").as_deref()) {
            Ok(terminal) => {
                self.alos_own_terminal = Some(terminal);
                true
            }
            // **Said out loud rather than silently doing nothing.** A person
            // who pressed a bound chord and got nothing cannot tell that from
            // the chord not arriving, and those are the two cases worth
            // separating in a log.
            Err(why) => {
                eprintln!("alo-shell: alo's own terminal could not be opened: {why}");
                false
            }
        }
    }

    /// Whether alo's own terminal is taking the keyboard.
    ///
    /// **The question the input lane asks per batch**, in the shape
    /// `Server::settings_is_taking_the_keys` and
    /// `Server::the_dock_is_taking_the_keys` already use. False unless a
    /// terminal is open and its shell is still running, so on an ordinary
    /// frame this costs one `is_some_and` and every key goes the road it went
    /// before this existed.
    ///
    /// **A terminal whose shell has gone does not hold the keyboard.**
    /// Otherwise a person would be typing into a window that cannot answer,
    /// with no way to tell.
    pub(crate) fn the_terminal_is_taking_the_keys(&self) -> bool {
        self.alos_own_terminal
            .as_ref()
            .is_some_and(|terminal| !terminal.has_finished())
    }

    /// Scroll alo's own terminal by this many lines, if one is open.
    ///
    /// Positive is back into what is remembered, negative is forward towards
    /// the newest. Answers whether the view moved — which is what decides
    /// whether a wheel notch was this terminal's to spend or belongs to
    /// whatever is behind it.
    pub(crate) fn alos_own_terminal_scrolled(&mut self, lines: i32) -> bool {
        self.alos_own_terminal
            .as_mut()
            .is_some_and(|terminal| terminal.look_back(lines))
    }

    /// **A person asked why the command in front is waiting.**
    ///
    /// Answers whether what alo said changed, so a caller leaves the frame alone
    /// when it did not — the shape `alos_own_terminal_scrolled` above has, for
    /// the same reason.
    ///
    /// Answers `false` when no terminal is open: there is nothing to ask about
    /// and nothing to redraw.
    pub(crate) fn alos_terminal_was_asked_why(&mut self) -> bool {
        self.alos_own_terminal
            .as_mut()
            .is_some_and(TheTerminal::asked_why_the_command_in_front_is_waiting)
    }

    /// What alo's own terminal's shell is called, if one is open.
    pub(crate) fn what_alos_own_terminals_shell_is_called(&self) -> Option<String> {
        self.alos_own_terminal
            .as_ref()
            .and_then(TheTerminal::what_the_shell_is_called)
    }

    /// Take whatever the terminal's shell has written, once a frame.
    ///
    /// Answers whether anything changed, so a caller can leave the frame alone.
    ///
    /// # Why this is public, and what it is not
    ///
    /// **A backend that never calls this draws an empty terminal whatever the
    /// shell has said.** Nothing else reads the pseudoterminal: `direct_desktop`
    /// calls it once a frame and is the only caller on a real machine, so a
    /// fixture driving the nested backend has to call the same road or watch a
    /// grid that never fills. It was crate-private until 2026-10-17, which is
    /// why no nested check had ever seen a character the shell wrote.
    ///
    /// It is **not** a way to wait for output. It takes what has already
    /// arrived and answers whether that was anything; a caller that wants to see
    /// a command's result calls it until it says so, with its own bound on how
    /// long it is prepared to wait.
    pub fn the_terminal_was_written_to(&mut self) -> bool {
        self.alos_own_terminal
            .as_mut()
            .is_some_and(TheTerminal::what_the_shell_wrote)
    }

    /// **What alo's own terminal's shell has said about its prompt.**
    ///
    /// Empty when there is no terminal, which is the ordinary case, and empty
    /// on nearly every frame when there is one: a shell with no integration
    /// snippet installed never sends a mark.
    pub fn what_the_shell_said(
        &mut self,
    ) -> Vec<
        Result<
            alo_terminal::what_the_shell_says_about_the_prompt::AShellSaid,
            alo_terminal::what_the_shell_says_about_the_prompt::NotRead,
        >,
    > {
        self.alos_own_terminal
            .as_mut()
            .map(TheTerminal::what_the_shell_said)
            .unwrap_or_default()
    }

    /// **Put alo's explanation of a command into its own terminal.**
    ///
    /// Does nothing when there is no terminal, which is what happens to an
    /// answer that arrives after the person closed it.
    pub fn alo_explains(&mut self, what: alo_terminal::WhatACommandWillDo) {
        if let Some(terminal) = self.alos_own_terminal.as_mut() {
            terminal.alo_explains(what);
        }
    }

    /// Make alo's own terminal the size its window has room for.
    ///
    /// **Called once a frame, before anything is drawn**, and it does nothing
    /// on the frames where the answer has not changed — which is every frame
    /// but the ones where the window, the font size or the display's scale
    /// moved. That third one is the one that is easy to forget and is exactly
    /// what happens when a laptop is plugged into another monitor.
    ///
    /// Answers whether it changed, for a caller that wants to know a frame is
    /// worth redrawing.
    pub(crate) fn the_terminal_fits_its_window(
        &mut self,
        fonts: &mut cosmic_text::FontSystem,
        size: (i32, i32),
        scale: f64,
    ) -> bool {
        let Some(terminal) = self.alos_own_terminal.as_mut() else {
            return false;
        };
        // **The scrollback it already keeps**, so laying the grid out again
        // never silently changes how much a person can scroll back to.
        let remembered = terminal.screen().size().remembered();
        match crate::the_terminals_cells_are_drawn::the_grid_that_fits(
            fonts, size, scale, remembered,
        ) {
            Ok((grid, cell)) => terminal.laid_out_again(grid, cell),
            // **A window with no room for a cell leaves the terminal alone.**
            // Not a refusal to draw and not a resize to nothing: the grid it
            // has is the last one that fitted, which is what a person sees
            // while a window is briefly too small.
            Err(_) => false,
        }
    }

    /// alo's own terminal, for a test that needs to type into it.
    ///
    /// **Only for a test, and the name says so.** Nothing in production needs
    /// a mutable terminal through the server: a keypress goes through
    /// [`Self::a_key_reached_alos_own_terminal`] and a frame through
    /// [`Self::the_terminal_was_written_to`], each of which is the one road for
    /// its job. A general mutable accessor would be a second road to both.
    #[cfg(test)]
    pub(crate) fn alos_own_terminal_mut_for_tests(&mut self) -> Option<&mut TheTerminal> {
        self.alos_own_terminal.as_mut()
    }

    /// alo's own terminal, for whatever draws it.
    pub(crate) fn alos_own_terminal(&self) -> Option<&TheTerminal> {
        self.alos_own_terminal.as_ref()
    }

    /// One keypress, sent to alo's own terminal.
    ///
    /// Answers whether it got there, so the input lane can tell *the terminal
    /// took it* from *there is no terminal* and pass the key on in the second
    /// case.
    pub(crate) fn a_key_reached_alos_own_terminal(
        &mut self,
        typed: alo_terminal::what_a_person_typed::WhatWasTyped,
        held: alo_terminal::what_a_person_typed::Held,
    ) -> bool {
        self.alos_own_terminal
            .as_mut()
            .is_some_and(|terminal| terminal.a_person_typed_a_key(typed, held))
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_terminal_a_person_opened_tests.rs"]
mod tests;
