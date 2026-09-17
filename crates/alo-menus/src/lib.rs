//! The context menu: what the thing under the pointer offers, as a closed list.
//!
//! `docs/autonomy/v0-5-hands-on-the-desktop-plan.md` task 4: *a context menu is
//! a closed list of actions the thing under the pointer offers, each an action a
//! person could reach another way (ADR 0009), and **no menu entry sends anything
//! to the agent without the person choosing the entry that says so***.
//!
//! Three sentences, and each is a file:
//!
//! - **Closed** — [`action`]. The list is fourteen long and lives here. No
//!   application adds a row, rewords one, or puts its own sentence where a
//!   person expects the system's.
//! - **Reachable another way** — [`also_by`]. Every entry says where else a
//!   person does the same thing, from a closed list of roads that exist, and the
//!   compiler holds every action to answering. A capability that lived in a
//!   right-click would be one that somebody driving the machine from a keyboard
//!   does not have.
//! - **One entry reaches the agent, and it says so** — [`choosing`]. On a
//!   machine with no agent it is not there at all, greyed out being an
//!   advertisement wearing a disabled state (ADR 0009).
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`subject`] | What is under the pointer: a file, some text, a window, the desktop |
//! | [`action`] | Everything a menu can offer, closed, and which one reaches the agent |
//! | [`also_by`] | Where else a person reaches each of them |
//! | [`menu`] | One menu: built from a subject, and what choosing an entry does |
//! | [`choosing`] | Done here, or offered to the agent — and never both |
//! | [`refusing`] | The one way choosing does nothing, with a sentence |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! ```
//! use alo_menus::{Action, Chosen, Menu, NotChosen, Subject, TheAgent};
//!
//! // A right-click on a file, on a machine with an agent.
//! let menu = Menu::over(Subject::AFile, TheAgent::OnThisMachine);
//! assert!(menu.offers(Action::Rename));
//!
//! // Choosing an ordinary row does it here, and offers nobody anything.
//! assert!(!menu.chosen(Action::Rename)?.reached_the_agent());
//!
//! // Only the row that says so reaches the agent.
//! assert_eq!(
//!     menu.chosen(Action::AskTheAgentAboutThis),
//!     Ok(Chosen::OfferedToTheAgent(Action::AskTheAgentAboutThis)),
//! );
//!
//! // And on a machine with no agent, that row does not exist to be chosen.
//! let alone = Menu::over(Subject::AFile, TheAgent::NotOnThisMachine);
//! assert_eq!(
//!     alone.chosen(Action::AskTheAgentAboutThis),
//!     Err(NotChosen::NotOnThisMenu { action: Action::AskTheAgentAboutThis }),
//! );
//! # Ok::<(), NotChosen>(())
//! ```
//!
//! # Nothing here draws
//!
//! No pointer, no rows, no rectangle. A menu here is a subject and a list of
//! actions; where it opens, how it looks and how it is read aloud are the
//! shell's. Nothing here decides what an entry *does* either — choosing
//! [`Action::Open`] answers that the person chose to open something, and the
//! file manager opens it.
//!
//! # Nothing here is context an agent is offered
//!
//! A menu is a person's own list of choices. Opening one, closing one and
//! reading one reach nothing: ADR 0001 §4's context is the focused window, the
//! selection and the open document, at the moment of invocation, and a menu is
//! none of the three. The one entry that offers anything does it because a
//! person chose it, and what is then offered lasts one question — `alo-context`
//! and `alo-handing` are where that is enforced, and this crate depends on
//! neither.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod action;
pub mod also_by;
pub mod choosing;
pub mod menu;
pub mod refusing;
pub mod subject;
pub mod words;

#[cfg(test)]
mod testing;

pub use action::Action;
pub use also_by::AlsoBy;
pub use choosing::Chosen;
pub use menu::{Menu, TheAgent};
pub use refusing::NotChosen;
pub use subject::Subject;
pub use words::{Word, WordsError, declare_into, menu_words};
