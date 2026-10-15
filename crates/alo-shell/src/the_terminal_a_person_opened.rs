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
}

impl TheTerminal {
    /// Start the person's own shell on a grid this big.
    ///
    /// # Errors
    /// Whatever stopped the pseudoterminal from opening or the shell from
    /// running.
    pub(crate) fn opened(size: HowBig) -> Result<Self, alo_terminal::NoShell> {
        Ok(Self {
            shell: AShellOnAPseudoterminal::the_persons_own(size)?,
            screen: TerminalOutput::of(size),
            finished: false,
        })
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
                Ok(WhatArrived::NothingYet) => return changed,
                Ok(WhatArrived::ItFinished) => {
                    self.finished = true;
                    return changed;
                }
                // **A failure is reported and the terminal kept.** What is on
                // the screen is what the shell said, and throwing it away
                // because a later read failed would lose a person's work. The
                // error stream rather than silence, because *never a silent
                // fallback*.
                Err(why) => {
                    eprintln!("alo-shell: the terminal's shell could not be read: {why}");
                    self.finished = true;
                    return changed;
                }
            }
        }
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
    pub(crate) fn screen(&self) -> &TerminalOutput {
        &self.screen
    }

    /// Whether the shell has gone.
    pub(crate) fn has_finished(&self) -> bool {
        self.finished
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
        match TheTerminal::opened(size) {
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

    /// Take whatever the terminal's shell has written, once a frame.
    ///
    /// Answers whether anything changed, so a caller can leave the frame alone.
    pub(crate) fn the_terminal_was_written_to(&mut self) -> bool {
        self.alos_own_terminal
            .as_mut()
            .is_some_and(TheTerminal::what_the_shell_wrote)
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
