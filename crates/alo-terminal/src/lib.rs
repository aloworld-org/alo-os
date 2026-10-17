//! A terminal's output, turned into a grid of cells something can draw.
//!
//! **This crate has no AI in it, and that is the point rather than a stage it
//! is at.** `docs/features.md`'s first terminal promise is that *the terminal
//! remains fully useful when AI is disabled, unavailable or unconfigured*, and
//! the way to keep that promise is to build the terminal without one and let
//! everything else be added above it.
//!
//! # What is ours and what is the engine's
//!
//! [ADR 0114](../../../docs/decisions/0114-the-terminals-engine-is-pinned-and-its-face-is-bundled.md):
//! the VT state machine, the grid, the scrollback and the pseudoterminal are
//! `alacritty_terminal`, pinned and unpatched. Writing them here would be
//! writing an engine, which `CLAUDE.md` forbids — and the clause exists so a
//! small team does not end up maintaining a terminal emulator as a side effect
//! of shipping a terminal.
//!
//! **What this crate owns is the interface in front of it**: a size, a thing
//! bytes go into, and a reading of what the grid now says. Nothing above knows
//! which engine is underneath, which is what makes the pin a pin.
//!
//! # Why bytes in and cells out, and nothing else yet
//!
//! A terminal is three jobs that are usually one program: run a shell on a
//! pseudoterminal, interpret the bytes it writes, and draw the result. This
//! crate does the middle one, completely, and the other two are separate
//! changes — because the middle one can be *tested without a display or a
//! child process*, and the other two cannot.
//!
//! So what is here is finished rather than started: a shell runs on a
//! pseudoterminal, the bytes it writes arrive, a grid changes, and what the
//! grid says can be read back and asserted. **What is not here is drawing**,
//! which needs a compositor and is the next change.

mod a_shell_on_a_pseudoterminal;
mod how_big_the_grid_is;
pub mod measures;
#[cfg(target_os = "linux")]
mod stopping_the_command_in_front;
mod terminal_output_becomes_a_grid;
pub mod the_connection_it_is_waiting_on;
pub mod what_a_cell_looks_like;
pub mod what_a_person_typed;
pub mod what_is_blocking_it;
pub mod what_the_kernel_says;
pub mod who_is_speaking;
pub mod why_a_command_is_waiting;
pub mod words;

pub use a_shell_on_a_pseudoterminal::WhatIsWaiting;
pub use a_shell_on_a_pseudoterminal::{
    ACellInPixels, AShellOnAPseudoterminal, NoShell, WhatArrived,
};
pub use how_big_the_grid_is::{HowBig, NotASize};
pub use stopping_the_command_in_front::NotStopped;
pub use terminal_output_becomes_a_grid::{
    ACellRun, ACursorShape, TerminalOutput, WhereTheCursorIs,
};
pub use the_connection_it_is_waiting_on::{
    SocketAddr, TheConnection, the_connection_being_read_by,
};
pub use what_is_blocking_it::{WhatIsBlockingIt, what_is_blocking};
pub use who_is_speaking::{ARegion, WhoIsSpeaking, regions_of};
pub use why_a_command_is_waiting::{
    TheEvidence, WhatItReadsFrom, WhyItIsWaiting, why_it_is_waiting,
};
