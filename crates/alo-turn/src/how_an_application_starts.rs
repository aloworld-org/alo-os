//! **How an application starts**, for the record of an approved `open`.
//!
//! The owner's decision of 2026-10-31 (ADR 0108's dated section): an approved
//! `open_application` starts the application **the ordinary way a Dock press
//! does** — over the bus where its entry says `DBusActivatable=true`, and from
//! its entry's own `Exec` line otherwise — and *the record says which road*.
//!
//! This crate touches no disk, so the answer is asked of whatever holds the
//! machine's desktop entries: `alo-agentd` reads them with `alo_launching`, the
//! same decisions the desktop's own sender makes. A test supplies a list.
//!
//! **The road is read from the entry, before the door is asked.** That makes
//! it what the record can honestly say — *how this application starts* — and
//! it lets an entry with no way to start be **refused by name before anything
//! is asked**, rather than approved, asked, and silently started by nothing.
//! The one thing it does not see is the desktop's fallback when the bus has
//! nothing to start: the desktop's journal says that line, and this record
//! names the road the entry asks for.

/// The two ways an installed application is started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheRoad {
    /// `Activate` on its own bus name; its entry says `DBusActivatable=true`.
    OverTheBus,
    /// Its entry's own `Exec` line, in a unit of its own in the person's
    /// systemd manager.
    FromItsCommand,
}

/// Why no road starts an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoRoad {
    /// No entry by that identifier is there now.
    NoEntry,
    /// The entry is there and gives no way to start it — no `Exec` and no bus
    /// activation, a malformed `Exec`, `Hidden`, or one asking for a terminal.
    NothingStartsIt,
}

/// Whatever can say how an installed application starts.
pub trait SaysHowAnApplicationStarts {
    /// How the application with this identifier starts, or why nothing does.
    ///
    /// # Errors
    ///
    /// [`NoRoad`] when nothing should be asked to start it.
    fn how_it_starts(&self, identifier: &str) -> Result<TheRoad, NoRoad>;
}
