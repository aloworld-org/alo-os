//! **What the shell says on the journal about a window's band and its
//! controls**, so a walk can find the band and press each control.
//!
//! The investigation of 2026-11-21 found the walk never pointed at or clicked
//! a window control — it closed everything with Alt+F4, *8 key presses and 0
//! button presses* — so a band that was not on the screen at all, as Text
//! Editor's was, was never seen. A walk cannot read where a band is off a
//! picture, so the one band the shell draws every window's band through says
//! where it is, and what became of each press. Agreed with the window-controls
//! lane on 2026-11-21; declared here once, and read by both.
//!
//! # The lines
//!
//! - `alo-shell: a band is drawn: <kind> "<id>" at <x>,<y> <w>x<h>` — when its
//!   place on the screen changes, in the output's physical pixels from its top
//!   left; and `alo-shell: a band is no longer drawn: <kind> "<id>"` when it
//!   stops, so a band that falls off the screen is a line to fail on;
//! - `alo-shell: a band's controls are drawn: <kind> "<id>": "<name>" at
//!   <x>,<y> <w>x<h>, …` — once the pointer is in the band, only the controls
//!   that kind has, by their English source words, never translated;
//! - `alo-shell: a band's control was pressed: <kind> "<id>": "<name>" by
//!   <pointer|reader|keys>`, said once from the one press road;
//! - `alo-shell: the window panel opened for <kind> "<id>"`,
//!   `alo-shell: a window was put aside: <kind> "<id>"` and
//!   `alo-shell: a window was closed: <kind> "<id>"` — for an application, the
//!   last when it really unmaps, so one that refuses to close is never a pass.
//!
//! `<id>` is an application's own id, and for alo's own windows a fixed name:
//! `terminal`, `settings`, `record`, or `task <n>`. Quoted as Rust quotes a
//! string, so nothing in an id can start a line of its own.

/// The words a line saying a band is drawn begins with.
pub const A_BAND_IS_DRAWN: &str = "alo-shell: a band is drawn:";

/// The words a line saying a band is no longer drawn begins with.
pub const A_BAND_IS_NO_LONGER_DRAWN: &str = "alo-shell: a band is no longer drawn:";

/// The words a line saying where a band's controls are begins with.
pub const A_BANDS_CONTROLS_ARE_DRAWN: &str = "alo-shell: a band's controls are drawn:";

/// The words a line saying a band's control was pressed begins with.
pub const A_BANDS_CONTROL_WAS_PRESSED: &str = "alo-shell: a band's control was pressed:";

/// The words a line saying a window's panel opened begins with.
pub const THE_WINDOW_PANEL_OPENED_FOR: &str = "alo-shell: the window panel opened for";

/// The words a line saying a window was put aside begins with.
pub const A_WINDOW_WAS_PUT_ASIDE: &str = "alo-shell: a window was put aside:";

/// The words a line saying a window was closed begins with.
pub const A_WINDOW_WAS_CLOSED: &str = "alo-shell: a window was closed:";

/// The control that opens a window's panel, in its English source words.
pub const WINDOW_ACTIONS: &str = "Window actions";

/// The control that puts a window aside.
pub const MINIMISE: &str = "Minimise";

/// The control that closes a window.
pub const CLOSE: &str = "Close";

/// Which kind of window a band is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheKind {
    /// An application's own window.
    Application,
    /// alo's terminal.
    Terminal,
    /// Settings.
    Settings,
    /// The record of what alo did.
    Record,
    /// One of alo's task windows.
    Task,
}

impl TheKind {
    /// Every kind, in the order a line may name them.
    pub const ALL: [Self; 5] = [
        Self::Application,
        Self::Terminal,
        Self::Settings,
        Self::Record,
        Self::Task,
    ];

    /// The one word a line names this kind by.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Terminal => "terminal",
            Self::Settings => "settings",
            Self::Record => "record",
            Self::Task => "task",
        }
    }

    /// The kind a line's word names, if it names one.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.word() == word)
    }
}

/// Where something is drawn, in the output's physical pixels from its top
/// left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnTheScreen {
    /// From the left.
    pub x: i32,
    /// From the top.
    pub y: i32,
    /// How wide.
    pub w: i32,
    /// How tall.
    pub h: i32,
}

impl OnTheScreen {
    /// Its middle, where a walk points.
    #[must_use]
    pub const fn middle(self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    /// As a line says it: `<x>,<y> <w>x<h>`.
    fn said(self) -> String {
        format!("{},{} {}x{}", self.x, self.y, self.w, self.h)
    }

    /// Read from how a line says it.
    fn read(said: &str) -> Option<Self> {
        let (at, size) = said.split_once(' ')?;
        let (x, y) = at.split_once(',')?;
        let (w, h) = size.split_once('x')?;
        Some(Self {
            x: x.parse().ok()?,
            y: y.parse().ok()?,
            w: w.parse().ok()?,
            h: h.parse().ok()?,
        })
    }
}

/// A band's controls, each by its name and where it is drawn, in the order a
/// line says them.
pub type TheControls = Vec<(String, OnTheScreen)>;

/// Who pressed a band's control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressedBy {
    /// The pointer.
    Pointer,
    /// A screen reader.
    Reader,
    /// The keys: Alt+F4 and its kin.
    Keys,
}

impl PressedBy {
    /// The one word a line names it by.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Pointer => "pointer",
            Self::Reader => "reader",
            Self::Keys => "keys",
        }
    }
}

/// `<kind> "<id>"`, as every line names a window.
fn the_window(kind: TheKind, id: &str) -> String {
    format!("{} {id:?}", kind.word())
}

/// The line saying `kind`'s band, `id`, is drawn `at`.
#[must_use]
pub fn a_band_is_drawn(kind: TheKind, id: &str, at: OnTheScreen) -> String {
    format!(
        "{A_BAND_IS_DRAWN} {} at {}",
        the_window(kind, id),
        at.said()
    )
}

/// The line saying `kind`'s band, `id`, is no longer drawn.
#[must_use]
pub fn a_band_is_no_longer_drawn(kind: TheKind, id: &str) -> String {
    format!("{A_BAND_IS_NO_LONGER_DRAWN} {}", the_window(kind, id))
}

/// The line saying where each of a band's `controls` is drawn.
#[must_use]
pub fn a_bands_controls_are_drawn(
    kind: TheKind,
    id: &str,
    controls: &[(&str, OnTheScreen)],
) -> String {
    let each: Vec<String> = controls
        .iter()
        .map(|(name, at)| format!("{name:?} at {}", at.said()))
        .collect();
    format!(
        "{A_BANDS_CONTROLS_ARE_DRAWN} {}: {}",
        the_window(kind, id),
        each.join(", ")
    )
}

/// The line saying a band's control, `name`, was pressed, and by whom.
#[must_use]
pub fn a_bands_control_was_pressed(kind: TheKind, id: &str, name: &str, by: PressedBy) -> String {
    format!(
        "{A_BANDS_CONTROL_WAS_PRESSED} {}: {name:?} by {}",
        the_window(kind, id),
        by.word()
    )
}

/// The line saying a window's panel opened.
#[must_use]
pub fn the_window_panel_opened_for(kind: TheKind, id: &str) -> String {
    format!("{THE_WINDOW_PANEL_OPENED_FOR} {}", the_window(kind, id))
}

/// The line saying a window was put aside.
#[must_use]
pub fn a_window_was_put_aside(kind: TheKind, id: &str) -> String {
    format!("{A_WINDOW_WAS_PUT_ASIDE} {}", the_window(kind, id))
}

/// The line saying a window was closed.
#[must_use]
pub fn a_window_was_closed(kind: TheKind, id: &str) -> String {
    format!("{A_WINDOW_WAS_CLOSED} {}", the_window(kind, id))
}

/// What follows `phrase` in `line`, wherever on the journal's line it begins.
fn after<'a>(line: &'a str, phrase: &str) -> Option<&'a str> {
    line.find(phrase)
        .map(|at| at + phrase.len())
        .and_then(|at| line.get(at..))
        .map(str::trim_start)
}

/// A quoted string at the start of `said`, and what follows it — read as Rust
/// quotes one, its `\"` and `\\` unescaped.
fn a_quoted(said: &str) -> Option<(String, &str)> {
    let rest = said.strip_prefix('"')?;
    let mut read = String::new();
    let mut escaped = false;
    for (at, character) in rest.char_indices() {
        if escaped {
            read.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some((read, rest.get(at + 1..)?));
        } else {
            read.push(character);
        }
    }
    None
}

/// `<kind> "<id>"` at the start of `said`, and what follows.
fn a_window(said: &str) -> Option<(TheKind, String, &str)> {
    let (word, rest) = said.split_once(' ')?;
    let kind = TheKind::named(word)?;
    let (id, rest) = a_quoted(rest)?;
    Some((kind, id, rest))
}

/// What a line saying a band is drawn says: its kind, its id, and where.
#[must_use]
pub fn read_a_band_is_drawn(line: &str) -> Option<(TheKind, String, OnTheScreen)> {
    let (kind, id, rest) = a_window(after(line, A_BAND_IS_DRAWN)?)?;
    let at = OnTheScreen::read(rest.strip_prefix(" at ")?.trim_end())?;
    Some((kind, id, at))
}

/// What a line saying a band is no longer drawn says: its kind and its id.
#[must_use]
pub fn read_a_band_is_no_longer_drawn(line: &str) -> Option<(TheKind, String)> {
    let (kind, id, rest) = a_window(after(line, A_BAND_IS_NO_LONGER_DRAWN)?)?;
    rest.trim().is_empty().then_some((kind, id))
}

/// What a line saying where a band's controls are says: its kind, its id, and
/// each control by name with where it is, in the order said.
#[must_use]
pub fn read_a_bands_controls_are_drawn(line: &str) -> Option<(TheKind, String, TheControls)> {
    let (kind, id, rest) = a_window(after(line, A_BANDS_CONTROLS_ARE_DRAWN)?)?;
    let mut rest = rest.strip_prefix(": ")?;
    let mut controls = Vec::new();
    loop {
        let (name, after_the_name) = a_quoted(rest)?;
        let after_at = after_the_name.strip_prefix(" at ")?;
        let (said, next) = match after_at.split_once(", ") {
            Some((said, next)) => (said, Some(next)),
            None => (after_at.trim_end(), None),
        };
        controls.push((name, OnTheScreen::read(said)?));
        match next {
            Some(next) => rest = next,
            None => return Some((kind, id, controls)),
        }
    }
}

#[cfg(test)]
#[path = "said_of_a_band_tests.rs"]
mod tests;
