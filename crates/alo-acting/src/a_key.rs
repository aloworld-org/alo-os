//! **The keys alo may press in a window**, a closed list (the owner's *"Yes do
//! all"*, 2026-11-17): Enter, Escape, Tab and the four arrows.
//!
//! Each has one name — the one a model sends, the record keeps and the door
//! carries, spelled as a person's own `alo-app … key` command spells it
//! (`docs/contracts/terminal-commands.md`) — and one XKB symbol, the one the
//! compositor's layout gives the key it presses. **Nothing outside this list
//! can be named**, so no combination, no letter and no shortcut reaches a
//! window by this road.

/// One key alo may press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AKey {
    /// Enter (Return).
    Enter,
    /// Escape.
    Escape,
    /// Tab.
    Tab,
    /// The up arrow.
    Up,
    /// The down arrow.
    Down,
    /// The left arrow.
    Left,
    /// The right arrow.
    Right,
}

impl AKey {
    /// All seven, in the order they are offered.
    pub const ALL: [Self; 7] = [
        Self::Enter,
        Self::Escape,
        Self::Tab,
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
    ];

    /// The name a model sends, the record keeps and the door carries.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Enter => "enter",
            Self::Escape => "escape",
            Self::Tab => "tab",
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// The key whose name this is, if it is one of the seven — matched
    /// exactly, so `Enter` and `ctrl+s` are not keys.
    #[must_use]
    pub fn called(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.named() == name)
    }

    /// The symbol XKB knows the key by.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Enter => "Return",
            Self::Escape => "Escape",
            Self::Tab => "Tab",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}
