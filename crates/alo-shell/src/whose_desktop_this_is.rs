//! The name of the person a desktop is for, told once at sign-in.
//!
//! [ADR 0135](../../../docs/decisions/0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)
//! is the road: the name a person typed when they made their account is given
//! to them at every sign-in in a file only they can read, the desktop binary
//! reads it once through `TheDesktop::whose_desktop_this_is`, and the stand-up
//! tells the `Server` here. The terminal's strip is what shows it today —
//! `Local · anna`.
//!
//! **Told rather than read**, for `Server::its_settings_entry_is_called`'s
//! reason: this crate shows and never measures, and a file is a reading.
//!
//! **And never the login's name.** The desktop runs as the account `alo` on
//! every machine, so a fallback to the process's own user would put `alo` in
//! the strip of everybody who has not been given a name — which is nobody's
//! name, and reads as the product's. No name is drawn as no name.

impl crate::Server {
    /// The person this desktop is for is called this.
    ///
    /// An empty name is no name: it is kept as [`None`], so the strip says
    /// `Local` alone rather than `Local · ` with nothing after it.
    pub fn its_person_is_called(&mut self, name: String) {
        self.whose_desktop = Some(name).filter(|name| !name.is_empty());
    }

    /// The name of the person this desktop is for, if it has been told one.
    pub(crate) fn whose_desktop_this_is(&self) -> Option<&str> {
        self.whose_desktop.as_deref()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "whose_desktop_this_is_tests.rs"]
mod tests;
