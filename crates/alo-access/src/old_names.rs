//! **A name a command was given before still reaches what it named.**
//!
//! `alo-app alo … --in settings` is a public contract
//! (`docs/contracts/terminal-commands.md`, *alo OS's own surfaces*), and until
//! 2026-11-12 Settings was described by a fixed list: the window, *what can be
//! changed*, and one switch per [`Setting`], named by [`Setting::word`]. Since
//! then Settings is described by its pages as they are drawn, and a drawn row
//! may be called something else — *stop things sliding and fading* is drawn as
//! *Reduced motion*.
//!
//! The owner's decision of 2026-11-12: **every name the command accepted keeps
//! working**, and where the drawn name differs the command says the name will
//! change. So the tree a reader and a command both read carries, beside a
//! drawn name, the name it had:
//!
//! - [`THE_OLD_NAME_ATTRIBUTE`] on the node that **is** a setting's row, whose
//!   value is that setting's old name in the person's language; and
//! - one [`the_not_drawn_yet_attribute`] per setting **no page draws yet**, on
//!   Settings' own node, so a command naming it is told it is not there rather
//!   than told nothing matched.
//!
//! **Keyed on the [`Setting`], never on a list of strings.** The shell asks each
//! row which setting it is; a setting whose row is built moves from the second
//! attribute to the first with nothing here changing.

use crate::setting::Setting;

/// The attribute a node carries when it is the row of a setting whose old name
/// differs from what is drawn — the old name, said in the person's language.
pub const THE_OLD_NAME_ATTRIBUTE: &str = "alo-formerly";

/// What every not-drawn-yet attribute's key begins with.
const NOT_DRAWN_YET: &str = "alo-not-drawn-yet:";

/// **The key Settings' own node carries for a setting no page draws yet**, whose
/// value is that setting's old name in the person's language.
#[must_use]
pub fn the_not_drawn_yet_attribute(setting: Setting) -> String {
    format!("{NOT_DRAWN_YET}{}", setting.on_the_wire())
}

/// The setting an attribute's key says is not drawn yet, or [`None`] for any
/// other attribute.
#[must_use]
pub fn not_drawn_yet(key: &str) -> Option<Setting> {
    let wire = key.strip_prefix(NOT_DRAWN_YET)?;
    Setting::ALL
        .into_iter()
        .find(|setting| setting.on_the_wire() == wire)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every setting has its own key, and the key reads back as it** — and
    /// an attribute that is not one reads as none.
    #[test]
    fn every_setting_reads_back_from_its_key_and_nothing_else_does() {
        let mut keys: Vec<String> = Setting::ALL
            .into_iter()
            .map(the_not_drawn_yet_attribute)
            .collect();
        for setting in Setting::ALL {
            assert_eq!(
                not_drawn_yet(&the_not_drawn_yet_attribute(setting)),
                Some(setting)
            );
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), Setting::ALL.len(), "two settings share a key");
        for not in [
            THE_OLD_NAME_ATTRIBUTE,
            crate::THE_SURFACE_ATTRIBUTE,
            "alo-not-drawn-yet:",
            "alo-not-drawn-yet:a-setting",
            "reduced-motion",
        ] {
            assert_eq!(not_drawn_yet(not), None, "{not}");
        }
    }
}
