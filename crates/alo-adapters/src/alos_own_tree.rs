//! **The names alo OS's own surfaces carry on the accessibility tree**, which a
//! person's command finds them by — kept here, beside the walk that reads them.
//!
//! The shell publishes them and this crate reads them, so one of the two has to
//! own the spelling and the other has to use it. They lived in `alo-access`
//! until 2026-11-12, and that is what had to change: `alo-access` depends on
//! `alo-approving`, which depends on `alo-turn`, so while this crate depended on
//! `alo-access` **a turn could never offer the accessibility fallback's two
//! verbs** — `alo-turn` depending on this crate would have been a cycle. They
//! moved here, and `alo-access` re-exports them unchanged, so every crate that
//! names them through `alo_access::` still does and reads the same string.
//!
//! What a key means stays `alo-access`'s: which [`THE_OLD_NAME_ATTRIBUTE`]
//! value a setting carries, and which settings a not-drawn-yet key may name, are
//! decided there (`alo_access::old_names`). This file says only how the keys are
//! spelt.

/// The attribute the shell's published tree carries on each surface's own node,
/// whose value is the surface's typed name — how a person's command finds the
/// surface they named, on the one tree a screen reader reads too.
pub const THE_SURFACE_ATTRIBUTE: &str = "alo-surface";

/// The attribute a node carries when it is the row of a setting whose old name
/// differs from what is drawn — the old name, said in the person's language.
pub const THE_OLD_NAME_ATTRIBUTE: &str = "alo-formerly";

/// What every not-drawn-yet attribute's key begins with: one per setting no
/// page of Settings draws yet, on Settings' own node.
pub const NOT_DRAWN_YET: &str = "alo-not-drawn-yet:";

/// The setting's wire name a not-drawn-yet key carries, or [`None`] for any
/// other key.
///
/// **Only the spelling is read here.** Whether the name is one of alo OS's
/// settings is `alo_access::not_drawn_yet`'s question; the walk does not need
/// to ask it, because the only tree that carries these keys is the shell's,
/// and the shell writes one only for a setting it has.
#[must_use]
pub fn the_setting_not_drawn_yet(key: &str) -> Option<&str> {
    key.strip_prefix(NOT_DRAWN_YET)
        .filter(|setting| !setting.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A not-drawn-yet key reads back as the setting it names, and nothing
    /// else reads as one — an empty name included.
    #[test]
    fn only_a_not_drawn_yet_key_names_a_setting() {
        assert_eq!(
            the_setting_not_drawn_yet("alo-not-drawn-yet:reduced-motion"),
            Some("reduced-motion")
        );
        for not in [
            THE_SURFACE_ATTRIBUTE,
            THE_OLD_NAME_ATTRIBUTE,
            NOT_DRAWN_YET,
            "reduced-motion",
            "alo-not-drawn-yet",
        ] {
            assert_eq!(the_setting_not_drawn_yet(not), None, "{not}");
        }
    }
}
