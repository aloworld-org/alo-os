//! **A text a change would put somewhere, as the person is asked to approve
//! it**: which argument carries it, and all of it, exactly as it arrived.
//!
//! ADR 0001, amended 2026-11-13: what a person approves is the sentence **and**
//! the text beneath it, because the sentence only points at the text (*type the
//! text below into “hello.txt” in VSCodium*). So a change that carries a text
//! carries it whole on the wire, to the card that shows it. Nothing here cuts,
//! summarises or refuses it: there are no input limits (the owner, 2026-11-13).
//!
//! **What travels is the text, never a drawing of it.** Which characters a
//! person could miss, and how long it is, are worked out from the text by
//! `alo_capability::TheWholeText` on the side that shows it, so the card and
//! the daemon cannot disagree about what was marked.

use serde::{Deserialize, Serialize};

/// One text a change carries, by the argument it arrived in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ATextToApprove {
    /// The argument it arrived in — `text` for `accessible.type_text`.
    argument: String,
    /// All of it, exactly as it arrived.
    text: String,
}

impl ATextToApprove {
    /// The text `argument` carries.
    #[must_use]
    pub fn of(argument: &str, text: &str) -> Self {
        Self {
            argument: argument.to_owned(),
            text: text.to_owned(),
        }
    }

    /// The argument it arrived in.
    #[must_use]
    pub fn argument(&self) -> &str {
        &self.argument
    }

    /// All of it, exactly as it arrived.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// It, ready to be shown whole: every line, how long it is, and every
    /// character a person could miss marked where it is.
    #[must_use]
    pub fn shown(&self) -> alo_capability::TheWholeText {
        alo_capability::TheWholeText::of(&self.text)
    }
}
