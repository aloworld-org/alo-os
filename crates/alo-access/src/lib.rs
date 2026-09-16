//! **What each accessibility setting is, and what it changes.**
//!
//! `ROADMAP.md` v0.5: *Access: screen reader, magnifier, high contrast,
//! keyboard-only operation of everything*, and `docs/features.md`: *the AT-SPI
//! tree the agent uses is the one a screen reader uses; EN 301 549 conformance
//! is the same work, not extra work.* Task 1 of
//! `docs/autonomy/v0-5-access-and-language-plan.md`.
//!
//! # One setting changes one thing
//!
//! There is **no accessibility mode**. A mode is a second machine, worse than
//! the first, that a person is sent to and then has to leave to do something it
//! left out — and the person who needs it is the one who can least afford the
//! journey. So each setting here is one value, it changes one thing, and
//! everything else about the machine stays as it was.
//!
//! What each changes is named as **a value another crate reads**
//! ([`WhatItChanges`]): the palette and the text scale are `alo-appearance`'s,
//! the key filter is the keyboard crate's when it exists, and the rest are read
//! by whoever draws. Nothing here draws anything, and nothing here is a
//! preference about how the machine looks — that is `alo-appearance`, and a
//! person who wants larger text because they like it is making a different
//! request from a person who cannot read the small one, even when the value is
//! the same.
//!
//! # Before anybody has signed in
//!
//! **A person who needs a screen reader to set the machine up must not need one
//! to find the setting.** Every setting here can be turned on at the sign-in
//! screen and during setup, before any account exists — which is why this crate
//! keeps two copies: the person's own, under their settings folder (ADR 0038),
//! and the machine's, which sign-in reads and which belongs to no account.
//! [`keeping`] has both, and neither knows where the folder is: it is handed the
//! path.
//!
//! | | |
//! |---|---|
//! | [`Setting`] | The closed list, and what each one changes |
//! | [`TurnedOn`] | What a person has turned on, and the only road to changing one |
//! | [`HighContrast`] | A palette of its own, held to WCAG AAA over every pair the shell draws |
//! | [`KeyFilter`] | Sticky, slow and bounce keys, as the values a keyboard reads |

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod high_contrast;
pub mod keeping;
pub mod key_filter;
pub mod setting;
pub mod tree;
pub mod turned_on;
pub mod voices;
pub mod words;

pub use high_contrast::{HighContrast, THE_PAIRS_THE_SHELL_DRAWS};
pub use key_filter::{KeyFilter, NotADelay};
pub use setting::{Setting, WhatItChanges};
pub use tree::{Control, Role, State, Surface, the_approval_in_reading_order};
pub use turned_on::{Magnification, NotAMagnification, TurnedOn};
pub use voices::{A_VOICE_FOR_EACH, has_a_voice};
