//! The chat template a catalogue entry's weights must be asked through, where the
//! file the catalogue names does not carry one.
//!
//! Task 8 of `docs/autonomy/v0-5-the-models-measured-plan.md`, and the finding
//! behind it in `docs/quirks.md`: the GGUF `teuken-7b-instruct` names carries no
//! chat template, the pinned runtime warns and answers with its end-of-turn
//! token in the text, and a grade made that way would measure the missing
//! template rather than the weights.
//!
//! # Configured, never invented
//!
//! The constitution lets alo OS configure an engine. What it may not do is put
//! words in a model's mouth that its publisher did not: so a template is carried
//! only **with the address its publisher published it at**, and the loader
//! refuses one without. The text is the publisher's template rendered for the one
//! shape alo OS asks in — a single message from a person — in the runtime's own
//! template language, and the note beside the entry says what was rendered from
//! what.

use serde::Deserialize;

/// Where the person's message goes in a template, as the pinned runtime spells it.
pub const THE_PROMPT: &str = "{{ .Prompt }}";

/// **A chat template, and where its publisher published it.**
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatTemplate {
    /// The template, in the pinned runtime's template language, with
    /// [`THE_PROMPT`] where the person's message goes.
    pub text: String,
    /// The address the model's publisher published the template at — a file in
    /// their own repository, at a revision.
    pub published_at: String,
}

impl ChatTemplate {
    /// What is wrong with this statement, or [`None`] when it holds together.
    pub(crate) fn what_is_wrong_with_it(&self) -> Option<&'static str> {
        if !self.text.contains(THE_PROMPT) {
            return Some(
                "a chat template with nowhere for the question to go: it must carry \
                 {{ .Prompt }} where the person's message is put",
            );
        }
        if !self.published_at.starts_with("https://") || self.published_at.trim().len() <= 8 {
            return Some(
                "a chat template with no address its publisher published it at: alo OS \
                 configures a model with its publisher's template and never with one it wrote",
            );
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sound() -> ChatTemplate {
        ChatTemplate {
            text: "System: a system line\nUser: {{ .Prompt }}\nAssistant: ".to_owned(),
            published_at: "https://huggingface.co/a-publisher/a-model/blob/abc/tokenizer.py"
                .to_owned(),
        }
    }

    #[test]
    fn a_template_with_its_publishers_address_holds() {
        assert_eq!(sound().what_is_wrong_with_it(), None);
    }

    #[test]
    fn a_template_nobody_published_or_with_nowhere_to_ask_is_refused() {
        for (wrong, saying) in [
            (
                ChatTemplate {
                    text: "System: a system line\nUser:\nAssistant:".to_owned(),
                    ..sound()
                },
                "nowhere for the question",
            ),
            (
                ChatTemplate {
                    published_at: String::new(),
                    ..sound()
                },
                "never with one it wrote",
            ),
            (
                ChatTemplate {
                    published_at: "a colleague's notes".to_owned(),
                    ..sound()
                },
                "never with one it wrote",
            ),
        ] {
            assert!(
                wrong
                    .what_is_wrong_with_it()
                    .is_some_and(|why| why.contains(saying)),
                "{wrong:?}"
            );
        }
    }
}
