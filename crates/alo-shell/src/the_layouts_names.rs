//! **The names the person's keyboard layout was made from**, kept so it can be
//! made again.
//!
//! `crate::typing_into_the_window_in_front` sends one client a layout of its
//! own for a moment and then has to send that client the person's layout back.
//! Smithay holds the person's layout and gives it out only behind `unsafe`,
//! which this workspace forbids, so the names it was compiled from are kept
//! here as they were given, and compiled again the way Smithay compiles them
//! (`XkbConfig::compile_keymap`: the same five names and no flags). The same
//! names in the same process give the same layout.

use smithay::input::keyboard::{XkbConfig, xkb};

/// The five names an XKB layout is compiled from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TheLayoutsNames {
    /// The rules file.
    rules: String,
    /// The keyboard's model.
    model: String,
    /// The layout, or layouts.
    layout: String,
    /// The variant of each.
    variant: String,
    /// Options, such as which key switches layout.
    options: Option<String>,
}

impl TheLayoutsNames {
    /// The names this configuration was given.
    pub(crate) fn of(config: &XkbConfig<'_>) -> Self {
        Self {
            rules: config.rules.to_owned(),
            model: config.model.to_owned(),
            layout: config.layout.to_owned(),
            variant: config.variant.to_owned(),
            options: config.options.clone(),
        }
    }

    /// The layout these names make — [`None`] if this machine's tables no
    /// longer have it.
    pub(crate) fn compiled(&self) -> Option<xkb::Keymap> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        xkb::Keymap::new_from_names(
            &context,
            &self.rules,
            &self.model,
            &self.layout,
            &self.variant,
            self.options.clone(),
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::TheLayoutsNames;
    use smithay::input::keyboard::{XkbConfig, xkb};

    #[test]
    fn the_names_make_the_same_layout_smithay_was_given() {
        let config = XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        };
        let names = TheLayoutsNames::of(&config);
        let made = names
            .compiled()
            .map(|keymap| keymap.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1));
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let theirs = xkb::Keymap::new_from_names(&context, "", "", "us", "", None, 0)
            .map(|keymap| keymap.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1));
        assert!(made.is_some());
        assert_eq!(made, theirs);
    }
}
