//! **What alo's own copy of a code editor is set to so that it does not look
//! for updates or send telemetry** — the owner's of 2026-11-15, after a walk
//! where the egress indicator showed alo's copy of VSCodium sending.
//!
//! Each costs the person nothing: Flatpak updates the application, so the
//! editor's own check has nothing to do, and alo OS sends no telemetry and
//! wants none sent from inside a copy it started.
//!
//! **Neither was seen sending.** Measured once by hand on 2026-11-15, in WSL,
//! VSCodium 1.143.17232 from Flathub, every connection made through a logging
//! proxy: its `product.json` names no update address and no telemetry
//! address, and nothing went to either. These keep it so if a later build
//! adds one. What it was seen sending is
//! `docs/quirks/what-alos-copy-of-vscodium-sends.md`'s.
//!
//! **Only in alo's own copy**, for `crate::typed_as_it_is`'s reason: the file
//! is inside the profile [`crate::alos_copy()`] gives that copy.

/// Each setting, by VSCodium's own name, and its value written as JSON.
pub const ASKS_FOR_NOTHING_IT_NEED_NOT: [(&str, &str); 2] = [
    ("update.mode", r#""none""#),
    ("telemetry.telemetryLevel", r#""off""#),
];

#[cfg(test)]
mod tests {
    use super::ASKS_FOR_NOTHING_IT_NEED_NOT;

    /// The two the owner named, with the values that turn them off.
    #[test]
    fn updates_and_telemetry_are_off() {
        assert!(ASKS_FOR_NOTHING_IT_NEED_NOT.contains(&("update.mode", r#""none""#)));
        assert!(ASKS_FOR_NOTHING_IT_NEED_NOT.contains(&("telemetry.telemetryLevel", r#""off""#)));
    }
}
