//! The runtime a question on this machine is put to, held by its own type.
//!
//! `alo_models::found_on_this_machine` finds the pinned runtime, and until an
//! agent turn asked for its next request nothing here needed to know which
//! runtime it was: a question in words goes to any `ModelRuntime` the same way.
//! [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md)
//! changed that — the pinned runtime is asked for an agent's next request held
//! to the protocol's envelope, and only the pinned runtime can be — so what is
//! found is kept as what it is, and handed to a turn as
//! `alo_turn::Answers::ThePinnedRuntime`.
//!
//! # Nothing shipped here makes one (ADR 0019)
//!
//! Holding the type by name is not a way to point the agent anywhere: this
//! crate constructs no runtime of its own outside its tests, and
//! `no_runtime_is_made_anywhere_this_daemon_ships` reads the shipped source to
//! hold that. The runtime comes from `found_on_this_machine`, whose address is
//! `alo-models`' own knowledge, or from nowhere.

use alo_models::{ModelRuntime, Ollama};
use alo_turn::Answers;

/// The runtime on this machine that answers a question.
#[derive(Debug)]
pub enum TheRuntime {
    /// The runtime alo OS pins, as `alo_models::found_on_this_machine` found
    /// it. The only kind a running daemon holds.
    Pinned(Ollama),
    /// A runtime a test stands in, which answers without one being installed.
    ///
    /// **`cfg(test)`, for `Questions::already_found`'s reason**: a door that
    /// took any runtime would be the address field ADR 0019 refuses, reached a
    /// different way.
    #[cfg(test)]
    StoodIn(Box<dyn ModelRuntime>),
}

impl TheRuntime {
    /// What a turn is handed to put a question to this runtime.
    ///
    /// The pinned runtime by name, so that an agent's next request can be held
    /// to the envelope; a stand-in by its trait, which is never held to it.
    #[must_use]
    pub fn answers(&self) -> Answers<'_> {
        match self {
            Self::Pinned(ollama) => Answers::ThePinnedRuntime(ollama),
            #[cfg(test)]
            Self::StoodIn(runtime) => Answers::Runtime(runtime.as_ref()),
        }
    }

    /// The runtime by its trait, for a door that asks it only in words — a
    /// question from a paired machine (`crate::questioned`).
    #[must_use]
    pub fn in_words(&self) -> &dyn ModelRuntime {
        match self {
            Self::Pinned(ollama) => ollama,
            #[cfg(test)]
            Self::StoodIn(runtime) => runtime.as_ref(),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_models::Catalogue;
    use std::path::Path;

    /// **The pinned runtime reaches a turn by name, and a stand-in never
    /// does** — the difference between a next request held to the envelope
    /// and one asked freely.
    #[test]
    fn the_pinned_runtime_reaches_a_turn_as_itself() {
        let pinned = TheRuntime::Pinned(Ollama::new(Catalogue::built_in().unwrap()));
        assert!(matches!(pinned.answers(), Answers::ThePinnedRuntime(_)));
        let stood_in = TheRuntime::StoodIn(crate::testing::a_runtime_saying(Ok("four".to_owned())));
        assert!(matches!(stood_in.answers(), Answers::Runtime(_)));
    }

    /// **Nothing this daemon ships makes a runtime of its own**, so holding the
    /// type by name points nothing anywhere (ADR 0019). Read off every source
    /// file up to its tests, and not at all in the file that is only for tests.
    #[test]
    fn no_runtime_is_made_anywhere_this_daemon_ships() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut read = 0;
        for entry in std::fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|extension| extension != "rs")
                || path.file_name().is_some_and(|name| name == "testing.rs")
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            let shipped = text.split("\nmod tests {").next().unwrap_or_default();
            for making in ["Ollama::at", "Ollama::new", "Ollama {", "Ollama::default"] {
                assert!(
                    !shipped.contains(making),
                    "{} makes a runtime with `{making}`",
                    path.display()
                );
            }
            read += 1;
        }
        assert!(read > 20, "only {read} source files were read");
    }
}
