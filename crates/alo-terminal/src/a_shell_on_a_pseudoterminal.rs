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
//! # Nothing here interprets anything, with one named exception
//!
//! Bytes come out of a child process and go into a grid. **No byte is read for
//! meaning, matched against a pattern, or sent anywhere.** A terminal that
//! inspected its own output to be clever would be a terminal a person could
//! not reason about, and `docs/features.md` promises the opposite first of
//! everything: *ordinary commands are not reinterpreted as AI requests.*
//!
//! **The exception, added 2026-10-18, is the marks a shell sends on purpose.**
//! [`crate::what_the_shell_says_about_the_prompt`] reads OSC 133 off the same
//! bytes — see [ADR 0123](../../../docs/decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
//! and `docs/contracts/shell-integration-marks.md`. **It does not weaken the
//! sentence above**, and the boundary is worth stating because it is the one
//! somebody will be tempted to move:
//!
//! - it reads **only** sequences a shell wrote in order to be read, never a
//!   program's output, and never what the person typed;
//! - a mark **buys nothing** — it cannot run, grant or approve anything, so a
//!   forged one makes alo explain a command nobody typed and nothing more;
//! - **no command is reinterpreted.** alo explains a line when the person asks
//!   for it by pressing a key, and the mark is how the line reaches alo rather
//!   than a reason to act on it;
//! - **nothing happens without the person installing a snippet.** With no
//!   snippet there are no marks and this reads nothing, which is why the
//!   feature says so rather than guessing.
//!
//! # Why reading answers three things rather than two
//!
//! A read from a pseudoterminal has three outcomes a caller must tell apart:
//! something arrived, nothing has arrived **yet**, and the shell has gone. A
//! `Result<usize>` blurs the last two into one — and a caller that treated *no
//! bytes yet* as *finished* would close a terminal the moment a program paused
//! to think. [`WhatArrived`] is those three, by name.

use crate::why_a_command_is_waiting::WhyItIsWaiting;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::{Read as _, Write as _};

use alacritty_terminal::event::WindowSize;
use alacritty_terminal::grid::Dimensions as _;
use alacritty_terminal::tty::{self, Options, Pty, Shell};

use crate::how_big_the_grid_is::HowBig;
use crate::terminal_output_becomes_a_grid::TerminalOutput;
use crate::where_a_shell_starts::WhereItStarts;

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
    /// The second parser, reading the shell's own marks off the same bytes.
    ///
    /// Held here rather than beside the grid because a mark is a **shell**
    /// saying where it is, and this is the thing that has a shell. Holding it
    /// across reads is what makes a sequence split over two reads still arrive.
    hearing: crate::what_the_shell_says_about_the_prompt::WhatTheShellSays,
    /// What the shell has said since a caller last asked.
    ///
    /// Drained rather than read, so one mark reaches one caller once — the same
    /// rule the agent's door keeps for answers.
    heard: Vec<
        Result<
            crate::what_the_shell_says_about_the_prompt::AShellSaid,
            crate::what_the_shell_says_about_the_prompt::NotRead,
        >,
    >,
    /// Which folder the shell was started in, kept so it can be asked.
    started_in: WhereItStarts,
}

/// What this terminal calls itself to the programs it runs.
///
/// **`clear`, `less`, `vim`, `top` and everything else built on curses reads
/// this and will not run without it.** Until 2026-10-17 the child was given no
/// environment at all, so `clear` answered *TERM environment variable not set.*
/// and a person's terminal could not clear its own screen. Found by typing
/// `clear` into one on a compositor and reading the screen back — which is step
/// 13 of the boot walk, and would have been found there tomorrow instead.
///
/// **Why `xterm-256color` and not a name of ours.** A name here is a claim about
/// which escape sequences this terminal understands, and a program believes it.
/// What `xterm-256color` promises is what the engine behind this really
/// implements. That engine's upstream ships an `alacritty` terminfo and names
/// itself that; this repository ships no terminfo, and a name no machine can
/// look up leaves a program falling back to something far poorer.
///
/// **And the image really carries it**, which is the half this crate cannot
/// check: a terminfo database belongs to an installed system. Measured by hand
/// on 2026-10-17 against the pinned base — it has
/// `/usr/share/terminfo/x/xterm-256color` and `/usr/sbin/clear`. That
/// measurement does not run again; what does is
/// `alo-image`'s `what_the_image_owes_the_terminal.rs`, which fails if the base
/// is changed under it or stops being pinned by digest.
///
/// **It is not a setting, and a person can still change it**: this is what the
/// child *starts* with, and a line in their own shell startup replaces it.
const WHAT_THIS_TERMINAL_IS_CALLED: &str = "xterm-256color";

/// The environment a shell started here is given.
///
/// Its own function so a test can read it without starting a shell, which needs
/// a pseudoterminal and a machine to put one on.
fn what_the_child_is_told() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("TERM".to_owned(), WHAT_THIS_TERMINAL_IS_CALLED.to_owned());
    env
}

impl AShellOnAPseudoterminal {
    /// Start the person's own shell, in their home folder.
    ///
    /// **Names no program**, so the system answers with theirs. `home` is the
    /// session's `$HOME`, and the shell starts there — which is the behaviour
    /// of every terminal a person has used. Until 2026-10-28 this said *the
    /// working directory is where this process already is*, and on a machine
    /// that is `/`, because the compositor is started there.
    ///
    /// **A home that cannot be used still gets a shell**, in `/`, and
    /// [`Self::where_it_started`] says why; `crate::where_a_shell_starts` gives
    /// the reasons for both halves of that.
    ///
    /// # Errors
    /// [`NoShell::ItWouldNotStart`] if the pseudoterminal could not be opened
    /// or the shell could not be run.
    pub fn the_persons_own(size: HowBig, home: Option<&OsStr>) -> Result<Self, NoShell> {
        Self::started(size, None, WhereItStarts::for_a_home(home))
    }

    /// Which folder this shell was started in, and whether it was the person's
    /// home.
    ///
    /// **Where it started, not where it is.** A person who types `cd` has
    /// moved, and this does not follow them; it answers the one question a
    /// caller has at the moment a terminal opens — *did it open at home, and if
    /// not, why not* — so the answer can be said rather than lost.
    #[must_use]
    pub const fn where_it_started(&self) -> &WhereItStarts {
        &self.started_in
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
        Self::started(size, Some(shell), WhereItStarts::for_a_home(None))
    }

    /// The one road the two above take.
    ///
    /// **The folder is always named**, never left to the engine. Left out, the
    /// child inherits whatever folder this process is standing in, which is how
    /// a person's terminal came to open at the top of the disk.
    fn started(
        size: HowBig,
        shell: Option<Shell>,
        started_in: WhereItStarts,
    ) -> Result<Self, NoShell> {
        let options = Options {
            shell,
            working_directory: Some(started_in.the_folder().to_path_buf()),
            // **The child's output is not drained when it exits.** Draining is
            // for a terminal that is closing and wants the last of what a
            // program said; this is a library, and whoever owns the terminal
            // decides that.
            drain_on_exit: false,
            env: what_the_child_is_told(),
        };
        let pty =
            tty::new(&options, how_the_child_is_told(size), 0).map_err(NoShell::ItWouldNotStart)?;
        Ok(Self {
            pty,
            hearing: crate::what_the_shell_says_about_the_prompt::WhatTheShellSays::new(),
            heard: Vec::new(),
            started_in,
        })
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
                let arrived = buffer.get(..how_many).unwrap_or_default();
                // **The same bytes, twice, and the grid is unaffected.** The
                // reader only looks; it consumes nothing and changes nothing,
                // so the grid sees exactly what it saw before this existed.
                self.heard.extend(self.hearing.reading(arrived));
                onto.was_written(arrived);
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

    /// **Everything the shell has said since this was last asked**, and nothing
    /// twice.
    ///
    /// Empty on the overwhelming majority of calls: a shell with no integration
    /// snippet installed never sends a mark, and one that has it sends a handful
    /// per command rather than per read.
    ///
    /// **Drained**, so one mark reaches one caller once. A caller that read
    /// without draining would explain the same command on every frame for as
    /// long as the person left it on the screen.
    ///
    /// Each item is a mark **or the reason one could not be read** — a caller
    /// that drops the errors is throwing away the only signal that a command was
    /// too long to read, and would show nothing where the person is owed a
    /// sentence.
    pub fn what_the_shell_said(
        &mut self,
    ) -> Vec<
        Result<
            crate::what_the_shell_says_about_the_prompt::AShellSaid,
            crate::what_the_shell_says_about_the_prompt::NotRead,
        >,
    > {
        std::mem::take(&mut self.heard)
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

    /// The folder the shell on this terminal is in now, as the kernel records
    /// it.
    ///
    /// **The shell's folder and not the command in front's**, because the
    /// shell is where `cd` acts: a program a person started may wander off to
    /// other folders, and the strip says where the person will be when it
    /// finishes.
    ///
    /// [`None`] where the kernel will not say, which is not an error: a shell
    /// that has just exited is in no folder.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn where_the_shell_is(&self) -> Option<std::path::PathBuf> {
        crate::what_the_kernel_says::where_this_process_is(self.the_shell_itself())
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

    /// Why the command in front is waiting, from what this machine can find
    /// out about it.
    ///
    /// `docs/features.md`: *a command looks stuck; **ask**, and get waiting for
    /// the keyboard, for the network, for another process or a lock, still
    /// computing, or unknown with the evidence available — instead of killing
    /// something that was nearly done.*
    ///
    /// # This is the asking, and it is three calls nothing was making
    ///
    /// [`Self::the_command_in_front`] knew which process to ask about,
    /// [`crate::what_the_kernel_says::what_can_be_found_out_about`] knew how to
    /// gather the evidence, and [`crate::why_it_is_waiting`] knew how to read
    /// it. All three were built, tested against real processes, and **joined by
    /// nothing** — so a person could not ask, which is the whole of what that
    /// promise is.
    ///
    /// # When there is nothing to answer
    ///
    /// [`None`] in three cases, none of which is a fault:
    ///
    /// - **the person is at a prompt**, so the shell itself is in front and no
    ///   command is waiting for anything. Asking about the shell would answer
    ///   *waiting for the keyboard* every time, which
    ///   [`Self::the_command_in_front`] explains at length;
    /// - the terminal would not say which process is in front;
    /// - the machine would not say anything about that process, which happens
    ///   to a command that finished between the two questions.
    ///
    /// A caller shows nothing in all three. *Nothing is waiting* and *nothing
    /// could be found out* are different, and the second is
    /// [`WhyItIsWaiting::NobodyCanTell`] — which is an answer and arrives as
    /// `Some`.
    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn why_the_command_in_front_is_waiting(&self) -> Option<WhatIsWaiting> {
        let process = self.the_command_in_front()?;
        if process == self.the_shell_itself() {
            return None;
        }
        let evidence = crate::what_the_kernel_says::what_can_be_found_out_about(process)?;
        Some(WhatIsWaiting {
            process,
            why: crate::why_it_is_waiting(&evidence),
        })
    }
}

/// A command that looks stuck, and why it is waiting.
///
/// **Carries which process the answer is about**, because the promise does not
/// stop at the answer: *then show the prompt, inspect the connection, reveal
/// what is blocking it, or stop.* Every one of those four acts on something, and
/// this is the something. What it is called is
/// [`crate::what_the_kernel_says::what_this_process_is_called`], asked when a
/// surface needs the name rather than carried here where most callers would not
/// read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatIsWaiting {
    /// The process the answer is about: the one the terminal has in front.
    pub process: u32,
    /// Why it is waiting, which is one of the five and never a guess.
    pub why: WhyItIsWaiting,
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

/// How long the shell is given to go on its own before it is made to.
///
/// Forty polls of five milliseconds. Long enough that a shell honouring
/// `SIGHUP` finishes its exit hooks — measured at one or two polls for `zsh` and
/// `bash` — and short enough that a person closing a window does not notice.
/// **It is a bound and not a wait**: the common case returns on the first poll
/// that finds the child gone.
#[cfg(target_os = "linux")]
const POLLS_BEFORE_IT_IS_MADE_TO: usize = 40;

/// How long one of those polls is.
#[cfg(target_os = "linux")]
const BETWEEN_POLLS: std::time::Duration = std::time::Duration::from_millis(5);

#[cfg(target_os = "linux")]
impl Drop for AShellOnAPseudoterminal {
    /// **Make sure the shell is gone before the engine waits for it, because
    /// the engine waits for ever.**
    ///
    /// `alacritty_terminal`'s own `Drop for Pty` is three statements: send
    /// `SIGHUP`, unregister its signal handler, and `let _ = self.child.wait()`.
    /// That last one has **no deadline**, and `SIGHUP` is a signal a process may
    /// ignore — a shell started from a process that was itself ignoring it
    /// inherits that disposition and never dies. The wait then never returns.
    ///
    /// **This is a hang in the product and not only in a test.** A terminal is
    /// dropped when a person closes its window, on the thread that draws every
    /// window on the machine, so a shell that ignores `SIGHUP` would stop the
    /// whole desktop — not slow it, stop it. It was found as a test hanging for
    /// ten minutes in CI and killed by the harness, which is the shape
    /// `CLAUDE.md` warns about: *a test that hangs is not a test that fails*, so
    /// it reported no assertion and named nothing.
    ///
    /// # What this does, in the order it matters
    ///
    /// 1. **`SIGHUP` first**, because that is what closing a terminal means and
    ///    a shell that honours it runs its exit hooks. Nothing here takes that
    ///    away from a well-behaved shell.
    /// 2. **Reap it as it goes**, with `WNOHANG` so no call can block, for at
    ///    most `POLLS_BEFORE_IT_IS_MADE_TO` polls — forty of five milliseconds.
    /// 3. **`SIGKILL` if it is still there.** `SIGKILL` cannot be caught,
    ///    blocked or ignored, which is the whole reason it is the one used at the
    ///    end rather than a second `SIGHUP`.
    ///
    /// The engine's `Drop` then runs, signals a process that has gone, and its
    /// `wait` returns at once with an error it already ignores.
    ///
    /// **Every failure here is deliberately dropped.** This is a destructor: a
    /// child that has already gone, a pid that will not convert and a signal
    /// that cannot be sent are all *the shell is not running*, which is the
    /// outcome being asked for. There is nobody to report to and nothing to
    /// retry.
    fn drop(&mut self) {
        let Ok(raw) = i32::try_from(self.the_shell_itself()) else {
            return;
        };
        let Some(which) = rustix::process::Pid::from_raw(raw) else {
            return;
        };

        let _ = rustix::process::kill_process(which, rustix::process::Signal::HUP);
        for _ in 0..POLLS_BEFORE_IT_IS_MADE_TO {
            match rustix::process::waitpid(Some(which), rustix::process::WaitOptions::NOHANG) {
                // Reaped, or there is no such child any more. Either way the
                // engine's wait has nothing to wait for.
                Ok(Some(_)) | Err(_) => return,
                // Still running. The engine would block here for ever.
                Ok(None) => std::thread::sleep(BETWEEN_POLLS),
            }
        }
        let _ = rustix::process::kill_process(which, rustix::process::Signal::KILL);
    }
}

#[cfg(test)]
#[path = "a_shell_on_a_pseudoterminal_tests.rs"]
mod tests;
