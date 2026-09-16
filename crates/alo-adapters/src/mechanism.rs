//! How an adapter reaches its application — the contract's four mechanisms.
//!
//! `docs/contracts/app-adapters.md` names four and says they are not equal. The
//! declaration names one, and [`crate::loading`] decides whether this machine
//! loads it:
//!
//! - **D-Bus** is carried out today, by [`crate::delivering`];
//! - **the application's automation API** and **the accessibility tree** are
//!   declared and refused with a sentence saying nothing on this machine carries
//!   them out yet — a verb the machine cannot carry out is never offered (the
//!   turn's rule in `docs/contracts/agent-verbs.md`);
//! - **screenshots and synthetic input** are refused whatever else is true. The
//!   contract calls them unauditable and the plan puts them at v1, last resort,
//!   disabled by policy; no adapter loaded here uses them.

/// The mechanism an adapter declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mechanism {
    /// The application's own automation interface, in its own language.
    Api,
    /// The application's accessibility tree.
    Accessibility,
    /// An interface the application publishes on the session bus.
    DBus,
    /// Screenshots and synthetic input.
    Synthetic,
}

impl Mechanism {
    /// The word the contract's manifest uses for it.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Accessibility => "accessibility",
            Self::DBus => "dbus",
            Self::Synthetic => "synthetic",
        }
    }
}
