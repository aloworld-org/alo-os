//! What a person is asked at setup, and what their answer does.
//!
//! [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md)
//! gave setup a fourth choice — *no model, no provider, no agent* — with the
//! same weight as the other three and no persuasion attached.
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! settled that the local one is listed first because it is what the machine
//! can already do, and that **nothing is pre-selected**. Until this crate
//! neither had anywhere to happen: there was no setup flow in this repository
//! at all, and `docs/autonomy/v0-01-evidence.md` records that against three
//! separate promises.
//!
//! This is that flow **as a value**, in the shape `alo-approving` and
//! `alo-overlay` took their surfaces: decided without drawing one, so that the
//! drawing is the compositor lane's and the rules are testable now.
//!
//! ```
//! use alo_setting_up::{Answer, Offered, SettingUp, THE_FOUR, TheAgent};
//!
//! # let folder = std::env::temp_dir().join("alo-setting-up-doctest");
//! # let _ = std::fs::remove_dir_all(&folder);
//! # std::fs::create_dir_all(&folder).expect("a folder for this example");
//! # let at = folder.join("alo").join("settings.toml");
//! // A machine on its first morning: no settings file, and nothing chosen.
//! let mut setting_up = SettingUp::at(&at).expect("a machine nobody has configured");
//!
//! // The four, the local one first — and **nothing is selected**.
//! assert_eq!(THE_FOUR[0], Offered::OnThisMachine);
//! assert_eq!(setting_up.selected(), None);
//! assert!(!at.exists(), "opening setup wrote in somebody's settings");
//!
//! // Pressing on without choosing does not choose.
//! assert!(setting_up.answer(&Answer::NotAtAll).is_err());
//!
//! // They decline, which is an answer rather than a skip.
//! setting_up.select(Offered::NotAtAll);
//! setting_up.answer(&Answer::NotAtAll).expect("an answer their settings take");
//!
//! // Setup is finished, nothing answers questions, and the agent's surfaces
//! // are absent rather than greyed out.
//! assert!(setting_up.is_answered());
//! assert!(setting_up.settings().chosen().is_none());
//! assert_eq!(setting_up.the_agent(), TheAgent::Absent);
//!
//! // And it is asked once: a second answer does not overwrite the first.
//! setting_up.select(Offered::OnThisMachine);
//! assert!(setting_up.answer(&Answer::NotAtAll).is_err());
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`offered`] | The four configurations, their order, and what each one says |
//! | [`answering`] | The four as they are answered, each with what it needs |
//! | [`setting_up`] | Setup in front of somebody: what is selected, and the one answer it gets |
//! | [`agent`] | Whether this machine has an agent at all, afterwards |
//! | [`refusing`] | Every way an answer goes nowhere, and the sentence for each |
//! | [`nudging`] | The check that nothing here leans on anybody |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # Three things this crate may not do, and cannot
//!
//! **It does not pre-select.** [`SettingUp::at`] takes a path and nothing else,
//! and there is no constructor that takes a selection. ADR 0025 rejected
//! Option C — *setup pre-selects local* — because a pre-selected control is
//! persuasion by geometry and makes *not at all* the answer you have to undo
//! something to take. A later change that pre-selected anything would be that
//! decision reversed, and it would have to be written into `crate::setting_up`
//! where a reviewer would find it.
//!
//! **It does not write anywhere but the person's own settings.** Every answer
//! goes out through `alo_choosing::Choosing`, which is the one writer of that
//! file in this workspace; there is no path, no serialiser and no second store
//! anywhere in this crate. That is read off the manifest by
//! `tests/nothing_here_writes_anywhere_else.rs` rather than promised here.
//!
//! **It does not choose anything for anybody.** It offers what the four are and
//! writes what somebody answered. It does not install a model, does not test a
//! provider, does not ask what this machine can run, and does not decide what
//! the machine arrives carrying — which is the weights work and is blocked on a
//! catalogue entry that clears the verb-driving bar.
//!
//! # Two states that look the same and are not
//!
//! A machine nobody has configured and a machine whose owner said *not at all*
//! both have nothing answering questions. They are different machines: one is
//! waiting to be asked and the other is finished. `alo_choosing::Setup` is the
//! bit that tells them apart, and without it setup would be shown again to
//! every person who declined it — ADR 0009's *no nagging* broken by the one
//! mechanism guaranteed to meet all of them.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence a person can be shown is declared in [`words`] and answered
//! through a `said` in the language they read: the question, the four choices,
//! the line beside each, and the five refusals nobody else knows about. What a
//! person's own settings refuse arrives already worded by `alo-choosing`, which
//! is what keeps one moment from having two accounts.
//!
//! And [`nudging`] is the rule ADR 0009 and ADR 0025 both name, as a check:
//! **nothing setup says asks anybody to buy anything or leans on one of the
//! four.** A flow that selects nothing and then calls one of them *the
//! recommended option* has moved the persuasion out of the geometry and into
//! the sentence, where no structural rule could see it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod agent;
pub mod answering;
pub mod nudging;
pub mod offered;
pub mod refusing;
pub mod setting_up;
pub mod words;

#[cfg(test)]
mod testing;

pub use agent::TheAgent;
pub use answering::Answer;
pub use nudging::{EVERYTHING_THAT_LEANS, Leaning, Leant, what_would_lean_on_a_person};
pub use offered::{Offered, THE_FOUR};
pub use refusing::NotSetUp;
pub use setting_up::SettingUp;
pub use words::{EVERY_WORD, Word, WordsError, declare_into, setting_up_words};
