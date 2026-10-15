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
use crate::what_a_cell_looks_like::{AColour, HowItLooks};

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

    /// Lay this screen out at a new size.
    ///
    /// **The engine reflows**, which is the hard half and the reason ADR 0114
    /// chose a pinned engine rather than writing one: rewrapping lines a
    /// program has already written, and the scrollback above them, when the
    /// width changes. Terminals differ on whether they do it at all, and the
    /// ones that do have the bugs.
    ///
    /// Answers whether anything changed, so a caller can tell *it is now this
    /// size* from *it was already*. **That distinction is the whole throttle**:
    /// a resize signals the program, and one per frame would make a full-screen
    /// program unusable.
    pub fn laid_out_again(&mut self, size: HowBig) -> bool {
        if size == self.size {
            return false;
        }
        self.term.resize(size);
        self.size = size;
        true
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

    /// How far back the view is, in lines above the newest.
    ///
    /// Zero means the newest output is on screen, which is where a terminal
    /// starts and where it returns the moment somebody types.
    #[must_use]
    pub fn how_far_back(&self) -> usize {
        self.term.grid().display_offset()
    }

    /// Move the view by this many lines: positive is back into what is
    /// remembered, negative is forward towards the newest.
    ///
    /// Answers whether the view actually moved, so a caller can tell a scroll
    /// that did something from one at the end of what is remembered — which is
    /// what decides whether a wheel notch is this terminal's to consume.
    ///
    /// **The engine clamps**, so scrolling past the oldest line or past the
    /// newest is not an error and needs no arithmetic here. Asking it what
    /// changed rather than working it out is the rule the rest of this file
    /// follows: the engine owns the grid and this crate owns the interface in
    /// front of it.
    pub fn look_back(&mut self, lines: i32) -> bool {
        let was = self.how_far_back();
        self.term
            .scroll_display(alacritty_terminal::grid::Scroll::Delta(lines));
        self.how_far_back() != was
    }

    /// Put the newest output back on screen.
    ///
    /// Answers whether anything moved. **Every terminal does this when
    /// somebody types**, because typing into a view of the past and watching
    /// nothing appear is the most confusing thing a scrolled terminal can do.
    pub fn back_to_the_newest(&mut self) -> bool {
        let was = self.how_far_back();
        self.term
            .scroll_display(alacritty_terminal::grid::Scroll::Bottom);
        self.how_far_back() != was
    }

    /// The grid line a displayed row is showing, given how far back the view
    /// is.
    ///
    /// **The whole of scrolling is this one conversion**, which is why it is
    /// one function rather than a `- back` at each of the places that read the
    /// grid. Those would drift, and the symptom would be a cursor drawn a line
    /// away from the text it is in.
    ///
    /// The engine numbers what is remembered **negatively**: `Line(-1)` is the
    /// row just above the top of the screen, which is what
    /// `Grid::display_iter` starts from at `Line(-display_offset)`. So row `n`
    /// of the display is grid line `n - back`.
    ///
    /// [`None`] past the bottom of the screen. The top needs no check — a
    /// negative line is a real line of what is remembered, and the engine
    /// clamps `display_offset` to what it actually has.
    fn the_grid_line_showing(&self, row: usize) -> Option<Line> {
        if row >= self.size.screen_lines() {
            return None;
        }
        let row = i32::try_from(row).ok()?;
        let back = i32::try_from(self.how_far_back()).ok()?;
        Some(Line(row - back))
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
        let line = self.the_grid_line_showing(line)?;
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

    /// One line, as runs of cells that share an appearance.
    ///
    /// **Runs rather than cells, because shaping is per run.** A line of eighty
    /// cells shaped one cell at a time is eighty buffers and eighty roundings;
    /// a line shaped as one buffer cannot carry two colours. Grouping
    /// consecutive cells that look the same is what makes both possible, and it
    /// is what every terminal does.
    ///
    /// **Trailing blank cells are dropped**, which [`Self::line`] deliberately
    /// keeps. That one answers *what is on this line* and a caller measuring a
    /// grid needs the full width; this one answers *what is drawn*, and a run
    /// of eighty spaces in the default colours is eighty cells of nothing.
    /// A blank cell with a **background** is not blank and is kept.
    ///
    /// Empty for a line past the bottom of the screen, as [`Self::line`]
    /// answers [`None`].
    #[must_use]
    pub fn runs_on(&self, line: usize) -> Vec<ACellRun> {
        use alacritty_terminal::index::{Column, Point};
        let Some(which) = self.the_grid_line_showing(line) else {
            return Vec::new();
        };
        let mut runs: Vec<ACellRun> = Vec::new();
        // **Whether a run was cut by a space since the last character.** A run
        // carries its text and the column it starts at, so every character in
        // it must sit at `from_column` plus its own offset — and a space that
        // is skipped without ending the run breaks exactly that.
        let mut cut = false;
        for column in 0..self.size.columns() {
            let Ok(at) = i32::try_from(column) else { break };
            let cell = &self.term.grid()[Point::new(which, Column(at.unsigned_abs() as usize))];
            let looks = HowItLooks::of(cell);
            // **A space in the default colours draws nothing**, so it is not
            // drawn — and the run it interrupts ends there. What follows
            // begins a new run at its own column, which is what keeps the
            // text where the program put it.
            let is_nothing =
                cell.c == ' ' && looks.ground == AColour::TheDefault && !looks.underline;
            if is_nothing {
                cut = true;
                continue;
            }
            match runs.last_mut() {
                Some(last) if last.looks == looks && !cut => last.text.push(cell.c),
                _ => runs.push(ACellRun {
                    from_column: column,
                    text: cell.c.to_string(),
                    looks,
                }),
            }
            cut = false;
        }
        runs
    }

    /// Where the cursor is and what it looks like, or [`None`] when the
    /// program has hidden it.
    ///
    /// **Hidden is a real state and the common one.** A program hides the
    /// cursor with `DECTCEM` (`\x1b[?25l`) while it redraws, and every
    /// full-screen program does it on every frame — so a terminal that drew a
    /// cursor regardless would show one flickering in the wrong place
    /// throughout an editor's redraw.
    ///
    /// This replaces [`Self::cursor`], which answered a position and could not
    /// say *there is no cursor*. That one is kept because something may want
    /// the position whatever the program asked for.
    #[must_use]
    pub fn where_the_cursor_is(&self) -> Option<WhereTheCursorIs> {
        use alacritty_terminal::term::TermMode;
        use alacritty_terminal::vte::ansi::CursorShape;
        if !self.term.mode().contains(TermMode::SHOW_CURSOR) {
            return None;
        }
        let shape = match self.term.cursor_style().shape {
            CursorShape::Block => ACursorShape::Block,
            CursorShape::HollowBlock => ACursorShape::HollowBlock,
            CursorShape::Underline => ACursorShape::Underline,
            CursorShape::Beam => ACursorShape::Beam,
            // **The engine has a `Hidden` shape as well as the mode flag**, and
            // either means the same thing to a person: nothing is drawn. Two
            // roads to one answer, folded here so no caller has to know there
            // were two.
            CursorShape::Hidden => return None,
        };
        let (column, line) = self.cursor();

        // **The cursor scrolls with the text it is in.** Its grid position is
        // a line of the live screen, so a view scrolled back by `back` shows
        // it `back` rows further down — and far enough back it leaves the
        // screen entirely, which is what every terminal does and the only
        // honest answer: the cursor is not on this screen.
        let line = line.checked_add(self.how_far_back())?;
        if line >= self.size.screen_lines() {
            return None;
        }
        Some(WhereTheCursorIs {
            column,
            line,
            shape,
        })
    }

    /// The character under the cursor, for whatever draws over it.
    ///
    /// **A block cursor has to put it back.** Filling the cell hides what is
    /// there, so a terminal draws the character again in the ground colour on
    /// top — which is what inverting a cell means and what every terminal does.
    /// Without it, the letter a person is sitting on disappears.
    ///
    /// [`None`] past the end of the screen, and a space where the cell is
    /// empty — which is the ordinary case, because the cursor usually sits
    /// after the last character of a line.
    #[must_use]
    pub fn the_character_under(&self, at: WhereTheCursorIs) -> Option<char> {
        self.line(at.line)?.chars().nth(at.column)
    }

    /// Whether the program has asked for **application cursor keys**.
    ///
    /// A program sets this with `DECCKM` (`\x1b[?1h`), and it changes what the
    /// arrow keys must send: `\x1b[A` ordinarily, `\x1bOA` while it is on.
    /// Full-screen programs — an editor, a pager, anything using readline's
    /// own line editing — turn it on, so a terminal that ignored it would send
    /// arrows that those programs do not recognise. **The usual symptom is
    /// arrow keys printing `^[[A` instead of moving the cursor**, which is the
    /// single most recognisable broken-terminal behaviour there is.
    ///
    /// It is asked of the engine rather than tracked here, because the engine
    /// is what parses the sequence that sets it — a second copy of this state
    /// would be a second answer to the same question.
    #[must_use]
    pub fn arrows_are_in_application_mode(&self) -> bool {
        self.term
            .mode()
            .contains(alacritty_terminal::term::TermMode::APP_CURSOR)
    }
}

/// A run of cells on one line that share an appearance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ACellRun {
    /// Which column it starts at, counted from the left of the screen.
    pub from_column: usize,
    /// The characters, in order.
    pub text: String,
    /// How they are drawn.
    pub looks: HowItLooks,
}

/// Where the cursor is and what it looks like.
///
/// **Ours rather than the engine's**, for the reason ADR 0114 gives about the
/// whole interface: the engine sits behind something of ours, and a compositor
/// matching on `vte::ansi::CursorShape` would be a compositor that cannot be
/// built without the engine. The shapes are the ones the terminal protocol has,
/// which is not a choice either of us gets to make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhereTheCursorIs {
    /// Which column, counted from the left of the screen.
    pub column: usize,
    /// Which line, counted from the top of the screen.
    pub line: usize,
    /// What it should look like.
    pub shape: ACursorShape,
}

/// What a cursor looks like, as the program asked for it.
///
/// A program chooses with `DECSCUSR` (`\x1b[<n> q`). **An editor changes this**
/// — a bar in insert mode, a block in normal mode is the convention — so a
/// terminal that drew one shape always would be telling a person the wrong
/// thing about the mode they are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ACursorShape {
    /// A filled block over the cell.
    Block,
    /// An outline, drawn when the terminal does not have the keyboard.
    HollowBlock,
    /// A line under the cell.
    Underline,
    /// A line down the left of the cell.
    Beam,
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
