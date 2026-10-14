//! What a program wrote, turned into the grid a person sees.
//!
//! A program does not write lines. It writes a byte stream carrying text,
//! control characters and escape sequences — *move to column 1*, *erase to end
//! of line*, *make this bold*, *scroll* — and a terminal is the thing that
//! applies those to a grid of cells in order. That is this file's whole job.
//!
//! # The engine does the hard part, deliberately
//!
//! Feeding bytes to a state machine that edits a grid is not the difficult
//! part; the difficult part is the thousand sequences and the decades of
//! disagreement about what they mean. [ADR 0114] takes that as a pinned
//! engine, so what is written here is a door with a handle on our side.
//!
//! [ADR 0114]: ../../../docs/decisions/0114-the-terminals-engine-is-pinned-and-its-face-is-bundled.md
//!
//! # Nothing is interpreted
//!
//! **No byte that arrives here reaches a model, is parsed for meaning, or is
//! treated as an instruction.** A program's output is a program's output. The
//! only thing done with it is putting characters in cells — and the terminal
//! must stay this way with AI switched off, which `docs/features.md` promises
//! first of everything about it.

use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions as _;
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::Processor;

use crate::how_big_the_grid_is::HowBig;

/// A terminal's screen: bytes go in, cells come out.
///
/// **It owns the parser as well as the grid**, because a parser is a state
/// machine mid-sequence. A caller holding one separately could feed the first
/// half of an escape sequence to one screen and the second half to another,
/// and the only symptom would be a stray character where a colour should have
/// been.
pub struct TerminalOutput {
    /// The engine's grid and its state.
    term: Term<VoidListener>,
    /// The parser, mid-sequence between calls.
    parser: Processor,
    /// What it was built at, kept so a reader can ask without reaching into
    /// the engine.
    size: HowBig,
}

impl TerminalOutput {
    /// An empty screen of this size.
    ///
    /// **Nothing is listening.** `VoidListener` is the engine's own
    /// do-nothing listener, and it is right here: the events it would carry —
    /// a bell, a title change, a clipboard request — are a surface's business
    /// and there is no surface yet. **Taking them now and dropping them would
    /// be worse than not taking them**, because the next person would find a
    /// handler that looked written.
    #[must_use]
    pub fn of(size: HowBig) -> Self {
        Self {
            term: Term::new(Config::default(), &size, VoidListener),
            parser: Processor::default(),
            size,
        }
    }

    /// How big this screen is.
    #[must_use]
    pub const fn size(&self) -> HowBig {
        self.size
    }

    /// Everything a program just wrote.
    ///
    /// Call it with whatever arrived, however it was split. **A sequence cut
    /// in half across two calls is carried over**, which is the parser's own
    /// behaviour and the reason it is held here: bytes arrive from a
    /// pseudoterminal in whatever sizes the kernel hands over, and a terminal
    /// that only worked when a sequence landed whole in one read would work
    /// almost always and fail under load.
    pub fn was_written(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    /// One line of the screen, as the characters now in its cells.
    ///
    /// Counted from the top of the screen, so line 0 is the top line a person
    /// sees. **Trailing blanks are kept** rather than trimmed: a caller
    /// drawing cells wants every cell, and a caller comparing text can trim
    /// what it likes. Trimming here would make the two disagree about how wide
    /// a line is.
    ///
    /// [`None`] for a line past the bottom of the screen.
    #[must_use]
    pub fn line(&self, line: usize) -> Option<String> {
        if line >= self.size.screen_lines() {
            return None;
        }
        let line = Line(i32::try_from(line).ok()?);
        let mut said = String::with_capacity(self.size.columns());
        for column in 0..self.size.columns() {
            said.push(self.term.grid()[Point::new(line, Column(column))].c);
        }
        Some(said)
    }

    /// Every line of the screen, top to bottom.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        (0..self.size.screen_lines())
            .filter_map(|line| self.line(line))
            .collect()
    }

    /// Where the cursor is, as a column and a line counted from the top of the
    /// screen.
    ///
    /// **A program moves this constantly** and something drawing the screen has
    /// to know, so it is asked for here rather than left inside the engine.
    #[must_use]
    pub fn cursor(&self) -> (usize, usize) {
        let at = self.term.grid().cursor.point;
        let line = usize::try_from(at.line.0).unwrap_or(0);
        (at.column.0, line)
    }
}

/// **Written by hand, because the engine's types are not `Debug`** — and
/// because the derived one would have been useless anyway.
///
/// A screen's interesting state is what it *says*, not the megabyte of grid
/// behind it. So this prints the size, the cursor and the lines, which is
/// exactly what somebody staring at a failed assertion wants and is what a
/// derived `Debug` would have buried.
impl std::fmt::Debug for TerminalOutput {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (column, line) = self.cursor();
        out.debug_struct("TerminalOutput")
            .field("size", &self.size)
            .field("cursor", &(column, line))
            .field("lines", &self.lines())
            .finish()
    }
}

#[cfg(test)]
#[path = "terminal_output_becomes_a_grid_tests.rs"]
mod tests;
