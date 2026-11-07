//! **`alo-app`: the command a person types to use their applications**
//! (`docs/contracts/terminal-commands.md`, accepted 2026-11-06).
//!
//! ```text
//! alo-app list                                  what is installed, and what each offers
//! alo-app show <application> [<ability>]        what one application can be asked to do
//! alo-app <application> <ability> [values]      do one of its declared abilities
//! alo-app <application> controls | read         what its windows show
//! alo-app <application> press <kind> <name>     press one of its controls
//! alo-app <application> open | focus | close    its window
//! ```
//!
//! `<application>` is an identifier, a short name, `front` — the window the
//! person was working in before this terminal — or `alo`, alo OS's own
//! surfaces. A value of `-` is read from the pipe, one call per line.
//!
//! **A typed command runs at once, because the person typed it**: no grant and
//! no OK, because law 2 binds the agent and not the person. What it does not
//! do is go any other road than alo's own. A declared ability is validated and
//! carried out by `alo-adapters`' own code, a control is found and pressed by
//! the accessibility fallback's own walk, and a window is acted on by the
//! desktop's own door — so a person is never told something alo would not be.
//!
//! Every act and every refusal of one is written in the person's own record,
//! `~/.local/state/alo/commands.jsonl`, with what the application sent while it
//! ran as the desktop counted it (ADR 0148). Looking — `list`, `show`,
//! `controls`, `read` — is never written down.
//!
//! # What is not here yet
//!
//! `type`, `set`, `choose`, `key`, `click` and `drag`; the screen road; what an
//! application publishes about itself on the session bus; and `coverage`. The
//! contract's section 6 says what each waits on, and `show` says the two roads
//! it does not take rather than leaving them out.
#![forbid(unsafe_code)]

pub mod acting;
#[cfg(unix)]
pub mod at_the_daemon;
#[cfg(unix)]
pub mod at_the_door;
pub mod command;
pub mod counted;
pub mod declared;
pub mod door;
pub mod ended;
pub mod listing;
pub mod looking;
pub mod machine;
pub mod naming;
pub mod on_the_window;
pub mod outcome;
pub mod pipe;
pub mod pressing;
pub mod showing;
pub mod subject;
pub mod typed;
pub mod words;

pub use command::run;
pub use ended::Ended;
pub use machine::{APlace, Knowing, Machine};
pub use words::declare_into;
