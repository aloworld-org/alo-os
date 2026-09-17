//! **The one file that names the fine-tuning stack.**
//!
//! `alo_models::ollama` is the only file that knows the model runtime exists,
//! and `alo_converting::engine` the only one that knows the converter. This is
//! the third of that kind: everything else in this workspace speaks of *a
//! fine-tune*, and only this file knows what runs it.
//!
//! **Rented, pinned, unmodified** (ADR 0011). Where the stack cannot do
//! something, that is a finding and not a patch — and one such finding is
//! already written down here: [`WHY_NOT_THE_OBVIOUS_LOCAL_TOOL`].
//!
//! # Why this stack
//!
//! It produces a **LoRA adapter as a separate file** and never writes to the
//! base model, which [ADR 0048](../../../docs/decisions/0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md)
//! requires: what one granted folder taught must be a thing a person can delete
//! on its own. It writes that adapter in the **ordinary LoRA safetensors
//! format**, so a person can take what their documents taught to another
//! machine or another tool — a format of our own would be a lock-in we are
//! selling against.
//!
//! It also runs where alo OS runs. An engine that only works on macOS would
//! prove the shape on a stack this product can never carry.

/// **The fine-tuning stack, by name and exact version.**
///
/// The image pins these by digest; this is where the version a machine runs is
/// written down in the product, and the two must agree.
pub const THE_STACK: [(&str, &str); 5] = [
    ("torch", "2.9.0+cpu"),
    ("transformers", "4.57.1"),
    ("peft", "0.18.0"),
    ("accelerate", "1.11.0"),
    ("safetensors", "0.6.2"),
];

/// **The digests of the two that decide the shape**, as published on PyPI and
/// read on 2026-09-17. `peft` is what makes an adapter rather than a merged
/// model; `transformers` is what loads the base without writing to it.
///
/// Torch is not here because the image pins the wheel it installs, and which
/// wheel that is depends on the machine's architecture — the image's question,
/// not this file's.
pub const THE_TWO_THAT_DECIDE_THE_SHAPE: [(&str, &str); 2] = [
    (
        "peft-0.18.0-py3-none-any.whl",
        "624f69ca6393b765ccc6734adda7ca57d80b238f0900a42c357d8b67a03d62ff",
    ),
    (
        "transformers-4.57.1-py3-none-any.whl",
        "b10d05da8fa67dc41644dbbf9bc45a44cb86ae33da6f9295f5fbf5b7890bd267",
    ),
];

/// **What the adapter is written as**, and why it is not ours.
pub const THE_ADAPTER_FORMAT: &str = "LoRA safetensors: adapter_model.safetensors beside \
                                      adapter_config.json, the format `peft` writes and every \
                                      tool that reads an adapter expects";

/// **Why the obvious local tool is not the engine.**
///
/// `llama.cpp` is already on the machines this lane gates on — it serves the
/// models the catalogue holds, and ADR 0035 measured it. Its `llama-finetune`
/// looks like the answer and is not: it trains **every weight** and writes a
/// whole new model with `-o`, and its `--lora` flag only *loads* an adapter
/// somebody else made. There is no option that produces one.
///
/// So a fine-tune through it would either write the base weights or emit a
/// second copy of the model with the person's documents merged in — the shape
/// ADR 0048 exists to prevent, and the one that cannot be undone afterwards.
/// Read as of llama.cpp 0.4.0 (build 10809, commit 5266f24da), 2026-09-17.
pub const WHY_NOT_THE_OBVIOUS_LOCAL_TOOL: &str = "llama.cpp's llama-finetune trains every weight \
                                                  and writes a whole model; its --lora flag loads \
                                                  an adapter and never makes one, so every \
                                                  fine-tune through it merges (llama.cpp 0.4.0, \
                                                  build 10809)";

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Every version is exact**, because *the latest* is not a pin.
    #[test]
    fn every_version_is_one_version() {
        assert_eq!(THE_STACK.len(), 5);
        for (what, version) in THE_STACK {
            assert!(!version.is_empty(), "{what} has no version");
            assert!(
                version
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_digit()),
                "{what} is pinned to `{version}`, which is not a version"
            );
            assert!(
                !version.contains('^') && !version.contains('*') && !version.contains('>'),
                "{what} is pinned to a range, and a range is what the image cannot check"
            );
        }
    }

    /// **The two that decide the shape are pinned by digest**, in sixty-four
    /// hexadecimal characters.
    #[test]
    fn the_two_that_decide_the_shape_are_pinned_by_digest() {
        for (wheel, digest) in THE_TWO_THAT_DECIDE_THE_SHAPE {
            assert_eq!(digest.len(), 64, "{wheel}");
            assert!(
                digest.chars().all(|letter| letter.is_ascii_hexdigit()),
                "{wheel}"
            );
            let (named, _) = wheel.split_once('-').expect("a wheel names its package");
            assert!(
                THE_STACK.iter().any(|(what, _)| *what == named),
                "{wheel} is pinned and is not in the stack"
            );
        }
    }

    /// **The finding about the obvious tool says what it does instead**, so
    /// nobody has to rediscover it with a training run.
    #[test]
    fn the_finding_about_the_obvious_tool_says_what_it_does_instead() {
        assert!(WHY_NOT_THE_OBVIOUS_LOCAL_TOOL.contains("every weight"));
        assert!(WHY_NOT_THE_OBVIOUS_LOCAL_TOOL.contains("merges"));
        assert!(THE_ADAPTER_FORMAT.contains("safetensors"));
    }
}
