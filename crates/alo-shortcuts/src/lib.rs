//! What the keyboard does, and what a person changed it to.
//!
//! `docs/features.md` promises at v0.01 that alo OS has keyboard shortcuts *and
//! that a person can change them*. The second half is the reason this crate
//! exists as a model rather than as a table in the compositor: once bindings can
//! be changed, two of them can want the same keys, and what happens then is a
//! decision somebody has to make on purpose.
//!
//! **The last binding does not win.** [`Shortcuts::bind`] refuses a chord
//! something else already has and names what has it; a release that adds a
//! default onto a chord a person already moved something onto cannot take it
//! from them; and two bindings of their own on one chord fire nothing at all.
//! The three rules and the reasoning behind each are in [`shortcuts`], and the
//! two types a clash arrives as are in [`clash`].
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`modifier`] | What is held down, and why Shift is not enough on its own |
//! | [`key`] | The closed list of keys a shortcut can be built on |
//! | [`chord`] | One combination, and the three it refuses to be |
//! | [`action`] | What a shortcut does — the system's list, not an application's |
//! | [`defaults`] | What alo OS ships with, and why it ships in the code |
//! | [`changes`] | What a person changed, which is all that is written down |
//! | [`shortcuts`] | The two resolved, and every question asked of them |
//! | [`clash`] | Two actions wanting the same keys |
//!
//! ```
//! use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};
//!
//! let mut shortcuts = Shortcuts::shipped();
//! let ctrl_alt_space = Chord::checked(
//!     Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
//!     Key::Space,
//! )?;
//!
//! // The agent answers to Super+A until somebody says otherwise.
//! assert_eq!(shortcuts.chord_for(Action::TheAgent).map(|c| c.to_string()),
//!            Some("Super+A".to_owned()));
//! shortcuts.bind(Action::TheAgent, ctrl_alt_space)?;
//! assert_eq!(shortcuts.action_for(ctrl_alt_space), Some(Action::TheAgent));
//!
//! // Super+Left is how a window goes to the left half, and it says so.
//! let taken = shortcuts.chord_for(Action::SnapLeft).expect("it ships bound");
//! let refused = shortcuts.bind(Action::Launcher, taken).unwrap_err();
//! assert_eq!(refused.to_string(),
//!            "Super+Left is already Put the window on the left half \
//!             — change that one first, or use another key");
//!
//! // Only the change is written down; the rest comes from the running release.
//! assert_eq!(shortcuts.changes().len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Three things this crate is deliberately not
//!
//! **It is not an application's keyboard.** A system shortcut is a chord the
//! compositor takes before any window sees it, so every one of them is a key
//! taken away from every application on the machine. That is why [`Action`] is
//! short, why it grows only with the features that need it, and why the chords
//! copy, cut and paste are worked by cannot be taken at all — `docs/features.md`
//! promises those work across applications at v0.01, and here that promise is a
//! refusal rather than a sentence.
//!
//! **It is not a keyboard layout.** A [`Key`] is the key that prints that
//! character on the keyboard in front of the person, not a position on an
//! American one: on a French keyboard `Super+Q` is the key marked Q. What that
//! leaves for the compositor is written down in [`key`] — a layout that prints
//! no Latin letters at all has no key marked Q, and matching the shortcut
//! against the person's Latin layout is a lookup that belongs where the keyboard
//! is read.
//!
//! **It does not press anything.** Nothing here has a side effect, reads a
//! clock, or knows what a window is. It answers *what does this chord do* and
//! *what does this action answer to*; doing it is the compositor's, and it does
//! not exist yet.
//!
//! # The agent
//!
//! There is no connection between this crate and `alo-capability`, and that is
//! not an omission. A shortcut is a person pressing a key on their own machine:
//! it is not a verb, it needs no grant, and it is never proposed. The one place
//! the agent appears is [`Action::TheAgent`], which is the chord that summons
//! the overlay — and a person who has switched the agent off (ADR 0009) simply
//! has an action in the list that nothing answers, which is why it can be
//! cleared like any other.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod action;
pub mod changes;
pub mod chord;
pub mod clash;
pub mod defaults;
pub mod key;
pub mod modifier;
pub mod shortcuts;

pub use action::Action;
pub use changes::{Changed, Changes};
pub use chord::{Chord, ChordError};
pub use clash::{Clash, Taken};
pub use defaults::{Defaults, DefaultsError};
pub use key::Key;
pub use modifier::{Modifier, Modifiers};
pub use shortcuts::{Binding, Shortcuts};
