//! A person's own shell, running on a pseudoterminal.
//!
//! # Theirs, not ours
//!
//! The shell this starts is **the one the person has** — their `$SHELL`, with
//! their prompt, their aliases, their history and their environment.
//! `docs/features.md`: *the person's shell, custom prompt, aliases, history and
//! environment remain theirs.* So the ordinary way to start one names no
//! program at all and lets the system answer; naming one is for a test, which
//! is why that road says so in its name.
//!
//! # Nothing here interprets anything
//!
//! Bytes come out of a child process and go into a grid. **No byte is read for
//! meaning, matched against a pattern, or sent anywhere.** A terminal that
//! inspected its own output to be clever would be a terminal a person could
//! not reason about, and `docs/features.md` promises the opposite first of
//! everything: *ordinary commands are not reinterpreted as AI requests.*
//!
//! # Why reading answers three things rather than two
//!
//! A read from a pseudoterminal has three outcomes a caller must tell apart:
//! something arrived, nothing has arrived **yet**, and the shell has gone. A
//! `Result<usize>` blurs the last two into one — and a caller that treated *no
//! bytes yet* as *finished* would close a terminal the moment a program paused
//! to think. [`WhatArrived`] is those three, by name.

use std::collections::HashMap;
use std::io::{Read as _, Write as _};
use std::path::PathBuf;

use alacritty_terminal::event::WindowSize;
use alacritty_terminal::grid::Dimensions as _;
use alacritty_terminal::tty::{self, Options, Pty, Shell};

use crate::how_big_the_grid_is::HowBig;
use crate::terminal_output_becomes_a_grid::TerminalOutput;

/// How much is taken from the shell in one read.
///
/// **A buffer, not a limit.** Whatever is left stays in the kernel's own queue
/// and arrives on the next call, so nothing is dropped by choosing this badly —
/// a small number only means more calls.
const AT_ONCE: usize = 4_096;

/// What one read of the shell's output found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatArrived {
    /// This many bytes, already applied to the screen.
    Bytes(usize),
    /// **Nothing yet, and the shell is still there.** Ordinary: a program
    /// thinking, or waiting for the person to type.
    NothingYet,
    /// **The shell has gone.** Its end of the pseudoterminal is closed, and no
    /// further read will ever find anything.
    ItFinished,
}

/// Why a shell could not be started, or could not be read.
#[derive(Debug, thiserror::Error)]
pub enum NoShell {
    /// The pseudoterminal could not be opened or the shell could not be run.
    #[error("this machine could not start a shell on a pseudoterminal: {0}")]
    ItWouldNotStart(#[source] std::io::Error),
    /// Reading from or writing to it failed for a reason that is not *nothing
    /// yet* and not *it finished*.
    #[error("the shell's pseudoterminal could not be read or written: {0}")]
    ItWouldNotAnswer(#[source] std::io::Error),
}

/// A shell running on a pseudoterminal of its own.
pub struct AShellOnAPseudoterminal {
    /// The engine's pseudoterminal, holding the child and the master end.
    pty: Pty,
}

impl AShellOnAPseudoterminal {
    /// Start the person's own shell.
    ///
    /// **Names no program**, so the system answers with theirs. The working
    /// directory is where this process already is, which is the behaviour of
    /// every terminal a person has used.
    ///
    /// # Errors
    /// [`NoShell::ItWouldNotStart`] if the pseudoterminal could not be opened
    /// or the shell could not be run.
    pub fn the_persons_own(size: HowBig) -> Result<Self, NoShell> {
        Self::started(size, None, None)
    }

    /// Start one named program on a pseudoterminal instead of the person's
    /// shell.
    ///
    /// **This is for a test and the name says so.** A terminal a person opens
    /// runs their shell; a test needs something that writes a known string and
    /// exits, which is a different thing and should not be reachable by
    /// accident from the ordinary road.
    ///
    /// # Errors
    /// As [`Self::the_persons_own`].
    pub fn running(size: HowBig, program: &str, args: &[&str]) -> Result<Self, NoShell> {
        let shell = Shell::new(
            program.to_owned(),
            args.iter().map(|arg| (*arg).to_owned()).collect(),
        );
        Self::started(size, Some(shell), None)
    }

    /// The one road the two above take.
    fn started(
        size: HowBig,
        shell: Option<Shell>,
        working_directory: Option<PathBuf>,
    ) -> Result<Self, NoShell> {
        let options = Options {
            shell,
            working_directory,
            // **The child's output is not drained when it exits.** Draining is
            // for a terminal that is closing and wants the last of what a
            // program said; this is a library, and whoever owns the terminal
            // decides that.
            drain_on_exit: false,
            env: HashMap::new(),
        };
        let pty =
            tty::new(&options, how_the_child_is_told(size), 0).map_err(NoShell::ItWouldNotStart)?;
        Ok(Self { pty })
    }

    /// Take whatever the shell has written and apply it to the screen.
    ///
    /// Call it as often as you like. **The master end is non-blocking**, so
    /// this returns at once whether or not anything was there.
    ///
    /// # Errors
    /// [`NoShell::ItWouldNotAnswer`] for a failure that is neither *nothing
    /// yet* nor *it finished* — those two are answers rather than errors, for
    /// the reason this module's header gives.
    pub fn what_it_wrote(&mut self, onto: &mut TerminalOutput) -> Result<WhatArrived, NoShell> {
        let mut buffer = [0_u8; AT_ONCE];
        match self.pty.file().read(&mut buffer) {
            // **Zero bytes from a pseudoterminal means the far end is gone.**
            Ok(0) => Ok(WhatArrived::ItFinished),
            Ok(how_many) => {
                onto.was_written(buffer.get(..how_many).unwrap_or_default());
                Ok(WhatArrived::Bytes(how_many))
            }
            Err(why) => match why.kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted => {
                    Ok(WhatArrived::NothingYet)
                }
                // **The shell exiting is reported as an I/O error on Linux**,
                // not as end of file: the last process holding the far end
                // closed it, and a read on a pseudoterminal master with no
                // slave gives `EIO`. Reporting that as a failure would make
                // every terminal that ever closed look broken.
                _ if why.raw_os_error() == Some(5) => Ok(WhatArrived::ItFinished),
                _ => Err(NoShell::ItWouldNotAnswer(why)),
            },
        }
    }

    /// Tell the child its terminal is a new size.
    ///
    /// **This is the one `ioctl` the whole thing turns on.** Writing the size
    /// to the pseudoterminal does two things, and only the first is ours: the
    /// kernel stores it, so `TIOCGWINSZ` — which is what `stty size` and
    /// `tput cols` read — answers the new numbers; and **the kernel sends
    /// `SIGWINCH` to the foreground process group**, which is what makes a
    /// full-screen program wake up and redraw. No terminal on any Unix does
    /// this differently, because there is no other road.
    ///
    /// **Call it after the grid has been laid out again, never before.** A
    /// program redraws the moment it gets the signal, and a redraw that
    /// arrives while the grid is still the old size lands in the wrong shape.
    ///
    /// # Errors
    /// [`NoShell::ItWouldNotAnswer`] if the pseudoterminal refused it.
    pub fn the_window_changed(&mut self, size: HowBig, cell: ACellInPixels) -> Result<(), NoShell> {
        use alacritty_terminal::event::OnResize as _;
        let mut told = how_the_child_is_told(size);
        told.cell_width = cell.width;
        told.cell_height = cell.height;
        self.pty.on_resize(told);
        Ok(())
    }

    /// What a person typed, sent to the shell.
    ///
    /// **Bytes, not a string, and not a command.** What arrives here is what
    /// the keyboard produced — which includes control characters, an arrow
    /// key's escape sequence, and a paste. Nothing is appended: a caller that
    /// wants a newline sends one, because a caller that wants a tab or a
    /// Ctrl-C does not.
    ///
    /// # Errors
    /// [`NoShell::ItWouldNotAnswer`] if the write failed.
    pub fn a_person_typed(&mut self, bytes: &[u8]) -> Result<(), NoShell> {
        let mut door = self.pty.file();
        door.write_all(bytes).map_err(NoShell::ItWouldNotAnswer)?;
        door.flush().map_err(NoShell::ItWouldNotAnswer)
    }

    /// The shell this terminal is running.
    ///
    /// The direct child, and **not usually the process a person is waiting
    /// on** — see [`Self::the_command_in_front`]. It is here because stopping
    /// a terminal and asking about its shell are different questions with
    /// different right answers, and a caller that wants this one should be
    /// able to say so.
    #[must_use]
    pub fn the_shell_itself(&self) -> u32 {
        self.pty.child().id()
    }

    /// What the shell on this terminal is called, as the kernel records it.
    ///
    /// **Read from the running process rather than from `$SHELL`**, because
    /// those can differ and only one of them is what a person is typing into.
    /// The design draws it in the note under the transcript — `zsh · UTF-8` —
    /// and a note naming a shell that is not the one running would be worse
    /// than no note.
    ///
    /// [`None`] where the kernel will not say, which is not an error: a shell
    /// that has just exited has no name to give.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn what_the_shell_is_called(&self) -> Option<String> {
        crate::what_the_kernel_says::what_this_process_is_called(self.the_shell_itself())
    }

    /// The process the pseudoterminal has in front, if it will say.
    ///
    /// **This, and not the shell, is what *why is it waiting* is about.** A
    /// shell sits reading its own terminal whenever a person is at a prompt,
    /// so asking about the shell answers *waiting for the keyboard* every
    /// time, including while a command is running — the one case the question
    /// exists for.
    ///
    /// Answers [`None`] where the terminal will not say, which is not an
    /// error: a pseudoterminal with no foreground group, or one being torn
    /// down, has no answer to give and a caller should treat that as *nothing
    /// to report* rather than as a fault.
    ///
    /// **When it answers the same as [`Self::the_shell_itself`], nothing is
    /// running** — the shell is in front, and the person is at a prompt.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn the_command_in_front(&self) -> Option<u32> {
        let who = rustix::termios::tcgetpgrp(self.pty.file()).ok()?;
        u32::try_from(who.as_raw_nonzero().get()).ok()
    }
}

/// How big one cell is, in whole pixels.
///
/// **A terminal reports this to the child and we have been reporting zero.**
/// `TIOCSWINSZ` carries `ws_xpixel` and `ws_ypixel` beside the rows and
/// columns, and a program that draws a picture — a graph, an image, anything
/// using a graphics protocol — asks for them to work out how many pixels a cell
/// is worth. Zero is what a terminal reports when it does not know, and it was
/// the truthful answer while nothing had measured a cell.
///
/// Something measures one now, so the truth is available and this is how it is
/// carried. **Still zero when a caller has not measured one**, because an
/// invented cell size is worse than an admitted absence: a program would draw
/// at a scale nobody chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ACellInPixels {
    /// How wide, or zero for *nobody has measured one*.
    pub width: u16,
    /// How tall, or zero.
    pub height: u16,
}

/// What the child is told about the terminal it is running in.
///
/// **The two pixel figures are zero here**, and
/// [`AShellOnAPseudoterminal::the_window_changed`] is where a caller that has
/// measured a cell fills them in. At the moment a shell starts nothing has
/// drawn the terminal, so this machine does not know how big a cell is — and
/// zero is what every terminal reports when it has no answer, which programs
/// that care check for. Inventing a plausible figure would be telling a
/// program something nobody measured.
fn how_the_child_is_told(size: HowBig) -> WindowSize {
    WindowSize {
        num_lines: u16::try_from(size.screen_lines()).unwrap_or(u16::MAX),
        num_cols: u16::try_from(size.columns()).unwrap_or(u16::MAX),
        cell_width: 0,
        cell_height: 0,
    }
}

#[cfg(test)]
#[path = "a_shell_on_a_pseudoterminal_tests.rs"]
mod tests;
