//! What a person changed about their keyboards, and nothing else.
//!
//! ADR 0038's first clause: **only the difference is written**. A person who
//! has never opened this part of Settings has no file, a person who added a
//! Greek keyboard has a file with a list of two in it, and nothing writes down
//! what the release already ships. So a later release that changes what it
//! ships changes it for everybody who never disagreed, which is the whole point
//! of keeping the difference rather than the state.

use serde::{Deserialize, Serialize};

use crate::compose_key::ComposeKey;
use crate::layout::Layout;
use crate::methods::Writing;

/// What a person changed. Every field absent is a person who changed nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Changes {
    /// The keyboards they have, in the order they switch through them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layouts: Option<Vec<Layout>>,
    /// Which key composes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compose: Option<ComposeKey>,
    /// The ways of writing they added.
    #[serde(
        default,
        rename = "input-methods",
        skip_serializing_if = "Option::is_none"
    )]
    pub methods: Option<Vec<Writing>>,
}

impl Changes {
    /// Whether this person has changed nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A person who changed nothing writes nothing.
    #[test]
    fn a_person_who_changed_nothing_writes_nothing() {
        let nothing = Changes::default();
        assert!(nothing.is_empty());
        assert_eq!(toml::to_string(&nothing).unwrap(), "");
    }

    /// What is written is a person's own keyboards, and it reads back as what
    /// was written.
    #[test]
    fn what_is_written_reads_back_as_itself() {
        let changed = Changes {
            layouts: Some(vec![
                Layout::named("de").unwrap(),
                Layout::variant_of("us", "intl").unwrap(),
            ]),
            compose: Some(ComposeKey::Menu),
            methods: Some(vec![Writing::Japanese]),
        };
        assert!(!changed.is_empty());
        let written = toml::to_string(&changed).unwrap();
        assert!(written.contains("us(intl)"), "{written}");
        assert!(written.contains("input-methods"), "{written}");
        assert_eq!(toml::from_str::<Changes>(&written).unwrap(), changed);
    }

    /// **A hand-edited file that names something that is not a keyboard is
    /// refused**, rather than read as a keyboard nothing on the machine has.
    #[test]
    fn a_hand_edited_keyboard_that_is_not_one_is_refused() {
        for written in [
            "layouts = [\"de fr\"]",
            "layouts = [\"us(intl\"]",
            "layouts = \"de\"",
            "compose = \"RightAlt\"",
            "input-methods = [\"Cantonese\"]",
            "keyboards = [\"de\"]",
        ] {
            assert!(toml::from_str::<Changes>(written).is_err(), "{written}");
        }
    }
}
