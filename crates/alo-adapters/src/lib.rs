//! An application adapter, loaded against the contract, and carried out
//! through the application's own interface.
//!
//! `docs/contracts/app-adapters.md` is what an adapter author builds against,
//! and [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) §6 is
//! why it exists: an installed application becomes an agent — `@text_editor`,
//! `@gimp` — whose verbs are **verbs**, typed, approved one sentence at a time,
//! recorded, and reaching only what a person granted. This crate is task 5 of
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`.
//!
//! # The road, in order
//!
//! 1. **Declared** — an [`Adapter`] is data: its application, the releases it
//!    supports, its mechanism, its words and its verbs, each verb with typed
//!    arguments, a sentence, a by-hand road and the method it is carried out
//!    by.
//! 2. **Loaded** — [`load`] refuses, in words its author can act on
//!    ([`NotLoaded`]), everything the contract forbids: **a verb that takes a
//!    script, a command or free text, or hands an argument to something that
//!    interprets it**; screenshots and synthetic input; a verb with no by-hand
//!    road (ADR 0009); a path outside a grant; an adapter for a person's own
//!    application (ADR 0043). What passes is declared through
//!    `alo_capability::Verb::checked`, like every verb on the machine.
//! 3. **Offered, proposed, approved, redeemed** — by `alo-capability`, exactly
//!    as any verb. [`Adapters::offered_to`] offers an agent only the adapters
//!    whose application it holds a grant over.
//! 4. **Driven** — [`Driving::of`] takes the redeemed authority by value, asks
//!    the grants for the application again, asks whether it is installed, and
//!    builds the one [`Message`] the call becomes.
//! 5. **Delivered, and recorded** — [`Driving::deliver`] sends it once through
//!    [`Delivers`]; on Linux that is `SessionBus`. What ran and what was
//!    refused are written by `alo-record` from the same authority and refusal
//!    every other verb's are.
//!
//! # Registering adapter verbs needed no change to `alo-capability`
//!
//! The plan says a change there would stop this task at a decision. None was
//! needed: an adapter verb is a `Verb`, its name is `adapter.verb` (dots are
//! part of a verb identifier already), its grant is over its path arguments,
//! and the grant over its application is asked where the application is
//! reached — here — the way `alo_applications::Reaching` asks about installed
//! applications. The report says what *would* need a change: declaring an
//! adapter from a file at run time, whose words are not `'static`.
//!
//! # For an application with no adapter: the accessibility fallback
//!
//! Task 6. `docs/features.md`: *any application with no adapter is still
//! readable and operable through its AT-SPI tree*. Two verbs
//! ([`mod@fallback_verbs`]): `accessible.read_window`, a read of what an
//! application's windows show, and `accessible.activate_control`, a change that
//! presses one control named by its kind and the name it shows. Both reach only a
//! granted application with no adapter of its own ([`ReadingWindows`],
//! [`Activating`]); both walk the application's tree at that moment and keep
//! nothing ([`walking`]); **neither ever asks a password field for its
//! contents**, and neither can ask where anything is on the screen
//! ([`AccessibilityTree`] has no such question). A control with no name and an
//! area an application draws itself are said in words ([`Limit`]). On Linux the
//! tree is the session's own (`AccessibleSession`).
//!
//! # What is not here
//!
//! An adapter declaring the `api` or `accessibility` mechanism is refused when
//! loaded: the fallback is not an adapter, and a verb the machine cannot carry
//! out is never offered. No turn offers adapter or fallback verbs yet: that is
//! `alo-agentd`'s wiring, as for printing and installing.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(target_os = "linux")]
pub mod accessibility_bus;
pub mod accessibility_tree;
pub mod activating;
pub mod adapter;
pub mod adapter_arg;
pub mod adapter_verb;
pub mod adapters;
pub mod becoming_code;
pub mod bus_names;
pub mod delivering;
pub mod driving;
mod fallback_reach;
pub mod fallback_verbs;
pub mod fallback_words;
pub mod file_address;
pub mod invocation;
pub mod loading;
pub mod mechanism;
pub mod message;
pub mod not_loaded;
mod not_pressed;
mod not_walked;
pub mod pressable;
pub mod reading_windows;
pub mod role;
#[cfg(target_os = "linux")]
pub mod session_bus;
pub mod shown;
pub mod states;
pub mod text_editor;
pub mod verbs;
pub mod walking;
#[cfg(target_os = "linux")]
mod whose_connection;
pub mod words;

#[cfg(target_os = "linux")]
pub use accessibility_bus::AccessibleSession;
pub use accessibility_tree::{AccessibilityTree, Facts, NodeAt, Running, TreeFault};
pub use activating::{Activating, PRESSING, Pressed};
pub use adapter::Adapter;
pub use adapter_arg::{AdapterArg, Kind, Offer};
pub use adapter_verb::{AdapterVerb, ONLY_ITS_APPLICATION, Reaches};
pub use adapters::Adapters;
pub use delivering::{Delivers, NotDelivered};
pub use driving::{Driven, Driving};
pub use fallback_verbs::{ACTIVATE_CONTROL, READ_WINDOW, fallback_verbs};
pub use invocation::{DBusMethod, Invocation, Part};
pub use loading::{Loaded, load};
pub use mechanism::Mechanism;
pub use message::{Argument, Message};
pub use not_loaded::NotLoaded;
pub use pressable::Pressable;
pub use reading_windows::{ReadingWindows, WindowsRead};
pub use role::Role;
#[cfg(target_os = "linux")]
pub use session_bus::SessionBus;
pub use shown::{Contents, Limit, Seen, Shown, ShownWindow};
pub use states::States;
pub use text_editor::TEXT_EDITOR;
pub use verbs::{adapter_verbs, declare_into, shipped_adapters};
pub use words::{EVERY_WORD, WordsError, adapter_words};
